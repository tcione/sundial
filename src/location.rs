use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{LazyLock, mpsc};
use std::thread;
use std::time::Duration;

use log::{debug, warn};
use serde::{Deserialize, Serialize};

use crate::config::{Config, LocationMode};

static TIMEZONE_COORDS: LazyLock<HashMap<String, (f64, f64)>> = LazyLock::new(|| {
    let raw: HashMap<String, [f64; 2]> =
        serde_json::from_str(include_str!("../data/timezone_coords.json"))
            .expect("bundled timezone_coords.json is malformed");
    raw.into_iter().map(|(k, v)| (k, (v[0], v[1]))).collect()
});

const BOOT_ID_PATH: &str = "/proc/sys/kernel/random/boot_id";
const CACHE_FILE: &str = "location.json";
const GEOCLUE_TIMEOUT: Duration = Duration::from_secs(10);

#[derive(Debug, Serialize, Deserialize)]
struct LocationCache {
    latitude: f64,
    longitude: f64,
    date: String,
    boot_id: String,
}

pub fn resolve_location(config: &Config, data_dir: &PathBuf) -> (f64, f64) {
    let fallback = parse_config_location(config);

    if config.location.mode == LocationMode::Fixed {
        debug!("Using fixed location from config: {:?}", fallback);
        return fallback;
    }

    let cache = load_location_cache(data_dir);
    let boot_id = current_boot_id();
    let today = chrono::Utc::now().date_naive().to_string();

    if !needs_refresh(cache.as_ref(), &today, boot_id.as_deref()) {
        let cache = cache.unwrap();
        debug!("Using cached location: ({}, {})", cache.latitude, cache.longitude);
        return (cache.latitude, cache.longitude);
    }

    let detected = detect_location_geoclue()
        .inspect_err(|e| warn!("GeoClue2 location failed ({}); trying timezone fallback", e))
        .or_else(|_| detect_location_timezone())
        .inspect_err(|e| warn!("Timezone location failed ({}); falling back to last known location", e));

    match detected {
        Ok((latitude, longitude)) => {
            persist_location_cache(data_dir, latitude, longitude, &today, boot_id.as_deref());
            (latitude, longitude)
        }
        Err(_) => cache.map(|c| (c.latitude, c.longitude)).unwrap_or(fallback),
    }
}

fn needs_refresh(cache: Option<&LocationCache>, today: &str, boot_id: Option<&str>) -> bool {
    match cache {
        None => true,
        Some(cache) => {
            let date_matches = cache.date == today;
            let boot_matches = match boot_id {
                Some(current) => cache.boot_id == current,
                None => true,
            };

            !(date_matches && boot_matches)
        }
    }
}

fn detect_location_geoclue() -> Result<(f64, f64), Box<dyn std::error::Error>> {
    let (tx, rx) = mpsc::channel::<Result<(f64, f64), String>>();

    thread::spawn(move || {
        let _ = tx.send(geoclue_query());
    });

    rx.recv_timeout(GEOCLUE_TIMEOUT)
        .map_err(|_| "GeoClue2 timed out".into())
        .and_then(|r| r.map_err(|e| e.into()))
}

fn geoclue_query() -> Result<(f64, f64), String> {
    use zbus::blocking::{Connection, Proxy};
    use zbus::zvariant::OwnedObjectPath;

    let conn = Connection::system().map_err(|e| e.to_string())?;

    let manager = Proxy::new(
        &conn,
        "org.freedesktop.GeoClue2",
        "/org/freedesktop/GeoClue2/Manager",
        "org.freedesktop.GeoClue2.Manager",
    )
    .map_err(|e| e.to_string())?;

    let client_path: OwnedObjectPath = manager.call("GetClient", &()).map_err(|e| e.to_string())?;

    let client = Proxy::new(
        &conn,
        "org.freedesktop.GeoClue2",
        client_path.as_str(),
        "org.freedesktop.GeoClue2.Client",
    )
    .map_err(|e| e.to_string())?;

    client
        .set_property("DesktopId", "sundial")
        .map_err(|e| e.to_string())?;

    // Subscribe before Start to avoid missing the signal
    let mut signals = client
        .receive_signal("LocationUpdated")
        .map_err(|e| e.to_string())?;

    client
        .call::<_, _, ()>("Start", &())
        .map_err(|e| e.to_string())?;

    let msg = signals.next().ok_or("no LocationUpdated signal received")?;
    let (_, new_path): (OwnedObjectPath, OwnedObjectPath) =
        msg.body().deserialize().map_err(|e| e.to_string())?;

    let location = Proxy::new(
        &conn,
        "org.freedesktop.GeoClue2",
        new_path.as_str(),
        "org.freedesktop.GeoClue2.Location",
    )
    .map_err(|e| e.to_string())?;

    let latitude: f64 = location.get_property("Latitude").map_err(|e| e.to_string())?;
    let longitude: f64 = location.get_property("Longitude").map_err(|e| e.to_string())?;

    let _ = client.call::<_, _, ()>("Stop", &());

    debug!("Detected location via GeoClue2: ({}, {})", latitude, longitude);
    Ok((latitude, longitude))
}

fn detect_location_timezone() -> Result<(f64, f64), Box<dyn std::error::Error>> {
    let tz = system_timezone().ok_or("cannot determine system timezone")?;
    TIMEZONE_COORDS
        .get(&tz)
        .copied()
        .ok_or_else(|| format!("no coordinates for timezone '{}'", tz).into())
        .inspect(|(lat, lon)| debug!("Detected location via timezone '{}': ({}, {})", tz, lat, lon))
}

fn system_timezone() -> Option<String> {
    if let Ok(path) = std::fs::read_link("/etc/localtime") {
        if let Some(s) = path.to_str() {
            if let Some(idx) = s.find("zoneinfo/") {
                return Some(s[idx + 9..].to_string());
            }
        }
    }

    if let Ok(content) = std::fs::read_to_string("/etc/timezone") {
        let tz = content.trim().to_string();
        if !tz.is_empty() {
            return Some(tz);
        }
    }

    std::env::var("TZ").ok()
}


fn parse_config_location(config: &Config) -> (f64, f64) {
    let latitude = config.location.latitude.parse::<f64>().unwrap_or(0.0);
    let longitude = config.location.longitude.parse::<f64>().unwrap_or(0.0);

    (latitude, longitude)
}

fn current_boot_id() -> Option<String> {
    std::fs::read_to_string(BOOT_ID_PATH)
        .ok()
        .map(|id| id.trim().to_string())
}

fn load_location_cache(data_dir: &PathBuf) -> Option<LocationCache> {
    let content = std::fs::read_to_string(data_dir.join(CACHE_FILE)).ok()?;

    serde_json::from_str(&content).ok()
}

fn persist_location_cache(
    data_dir: &PathBuf,
    latitude: f64,
    longitude: f64,
    date: &str,
    boot_id: Option<&str>,
) {
    let cache = LocationCache {
        latitude,
        longitude,
        date: date.to_string(),
        boot_id: boot_id.unwrap_or_default().to_string(),
    };

    if let Ok(content) = serde_json::to_string(&cache) {
        let _ = std::fs::write(data_dir.join(CACHE_FILE), content);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cache(date: &str, boot_id: &str) -> LocationCache {
        LocationCache {
            latitude: 52.56,
            longitude: 13.39,
            date: date.to_string(),
            boot_id: boot_id.to_string(),
        }
    }

    #[test]
    fn test_needs_refresh_without_cache() {
        assert!(needs_refresh(None, "2026-06-04", Some("boot-a")));
    }

    #[test]
    fn test_needs_refresh_when_fresh() {
        let cache = cache("2026-06-04", "boot-a");
        assert!(!needs_refresh(Some(&cache), "2026-06-04", Some("boot-a")));
    }

    #[test]
    fn test_needs_refresh_on_stale_date() {
        let cache = cache("2026-06-03", "boot-a");
        assert!(needs_refresh(Some(&cache), "2026-06-04", Some("boot-a")));
    }

    #[test]
    fn test_needs_refresh_on_new_boot() {
        let cache = cache("2026-06-04", "boot-a");
        assert!(needs_refresh(Some(&cache), "2026-06-04", Some("boot-b")));
    }

    #[test]
    fn test_needs_refresh_ignores_unreadable_boot_id() {
        let cache = cache("2026-06-04", "boot-a");
        assert!(!needs_refresh(Some(&cache), "2026-06-04", None));
    }

    #[test]
    fn test_timezone_coords_known() {
        let (lat, lon) = TIMEZONE_COORDS.get("Europe/Berlin").copied().unwrap();
        assert!((lat - 52.52).abs() < 0.1);
        assert!((lon - 13.40).abs() < 0.1);
    }

    #[test]
    fn test_timezone_coords_unknown() {
        assert!(TIMEZONE_COORDS.get("Invalid/Timezone").is_none());
    }

    #[test]
    fn test_timezone_coords_aliases() {
        assert!(TIMEZONE_COORDS.get("Asia/Kolkata").is_some());
        assert!(TIMEZONE_COORDS.get("Asia/Calcutta").is_some());
        assert_eq!(
            TIMEZONE_COORDS.get("Asia/Kolkata"),
            TIMEZONE_COORDS.get("Asia/Calcutta"),
        );
    }
}
