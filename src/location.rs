use std::path::PathBuf;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use log::{debug, warn};
use serde::{Deserialize, Serialize};

use crate::config::{Config, LocationMode};

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
    timezone_coords(&tz)
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

fn timezone_coords(tz: &str) -> Option<(f64, f64)> {
    Some(match tz {
        // Africa
        "Africa/Abidjan" => (5.35, -4.00),
        "Africa/Accra" => (5.55, -0.22),
        "Africa/Addis_Ababa" => (9.02, 38.74),
        "Africa/Algiers" => (36.74, 3.06),
        "Africa/Cairo" => (30.06, 31.25),
        "Africa/Casablanca" => (33.59, -7.62),
        "Africa/Dar_es_Salaam" => (-6.80, 39.29),
        "Africa/Harare" => (-17.83, 31.05),
        "Africa/Johannesburg" => (-26.20, 28.04),
        "Africa/Kinshasa" => (-4.32, 15.32),
        "Africa/Lagos" => (6.45, 3.40),
        "Africa/Luanda" => (-8.84, 13.23),
        "Africa/Lusaka" => (-15.42, 28.28),
        "Africa/Maputo" => (-25.97, 32.59),
        "Africa/Nairobi" => (-1.29, 36.82),
        "Africa/Tripoli" => (32.90, 13.18),
        "Africa/Tunis" => (36.82, 10.17),
        "Africa/Khartoum" => (15.55, 32.53),

        // America
        "America/Adak" => (51.88, -176.63),
        "America/Anchorage" => (61.22, -149.90),
        "America/Argentina/Buenos_Aires" | "America/Buenos_Aires" => (-34.61, -58.38),
        "America/Asuncion" => (-25.29, -57.65),
        "America/Bogota" => (4.71, -74.07),
        "America/Caracas" => (10.48, -66.88),
        "America/Chicago" => (41.85, -87.65),
        "America/Costa_Rica" => (9.93, -84.09),
        "America/Denver" => (39.74, -104.99),
        "America/El_Salvador" => (13.70, -89.20),
        "America/Guatemala" => (14.64, -90.51),
        "America/Guayaquil" => (-2.17, -79.92),
        "America/Halifax" => (44.65, -63.60),
        "America/Havana" => (23.13, -82.38),
        "America/La_Paz" => (-16.50, -68.15),
        "America/Lima" => (-12.05, -77.04),
        "America/Los_Angeles" => (34.05, -118.24),
        "America/Managua" => (12.13, -86.28),
        "America/Mexico_City" => (19.43, -99.13),
        "America/Montevideo" => (-34.88, -56.18),
        "America/Montreal" | "America/Toronto" => (43.65, -79.38),
        "America/New_York" => (40.71, -74.01),
        "America/Panama" => (8.99, -79.52),
        "America/Phoenix" => (33.45, -112.07),
        "America/Puerto_Rico" => (18.47, -66.11),
        "America/Regina" => (50.45, -104.62),
        "America/Santiago" => (-33.46, -70.65),
        "America/Sao_Paulo" => (-23.55, -46.64),
        "America/St_Johns" => (47.56, -52.71),
        "America/Vancouver" => (49.25, -123.12),
        "America/Winnipeg" => (49.90, -97.14),

        // Asia
        "Asia/Almaty" => (43.26, 76.95),
        "Asia/Amman" => (31.96, 35.95),
        "Asia/Baghdad" => (33.34, 44.40),
        "Asia/Baku" => (40.38, 49.89),
        "Asia/Bangkok" => (13.75, 100.52),
        "Asia/Beirut" => (33.87, 35.50),
        "Asia/Bishkek" => (42.87, 74.59),
        "Asia/Calcutta" | "Asia/Kolkata" => (22.57, 88.36),
        "Asia/Damascus" => (33.51, 36.29),
        "Asia/Dhaka" => (23.72, 90.41),
        "Asia/Dubai" => (25.20, 55.27),
        "Asia/Ho_Chi_Minh" | "Asia/Saigon" => (10.82, 106.63),
        "Asia/Hong_Kong" => (22.29, 114.16),
        "Asia/Jakarta" => (-6.21, 106.85),
        "Asia/Jerusalem" | "Asia/Tel_Aviv" => (31.77, 35.22),
        "Asia/Kabul" => (34.53, 69.17),
        "Asia/Karachi" => (24.86, 67.01),
        "Asia/Kathmandu" | "Asia/Katmandu" => (27.72, 85.32),
        "Asia/Kuala_Lumpur" => (3.14, 101.69),
        "Asia/Kuwait" => (29.37, 47.98),
        "Asia/Macau" | "Asia/Macao" => (22.19, 113.54),
        "Asia/Manila" => (14.59, 120.98),
        "Asia/Muscat" => (23.61, 58.59),
        "Asia/Novosibirsk" => (54.99, 82.89),
        "Asia/Phnom_Penh" => (11.56, 104.92),
        "Asia/Rangoon" | "Asia/Yangon" => (16.87, 96.20),
        "Asia/Riyadh" => (24.69, 46.72),
        "Asia/Seoul" => (37.57, 126.98),
        "Asia/Shanghai" => (31.23, 121.47),
        "Asia/Singapore" => (1.29, 103.85),
        "Asia/Taipei" => (25.04, 121.53),
        "Asia/Tashkent" => (41.30, 69.27),
        "Asia/Tehran" => (35.70, 51.42),
        "Asia/Tokyo" => (35.69, 139.69),
        "Asia/Ulaanbaatar" | "Asia/Ulan_Bator" => (47.92, 106.92),
        "Asia/Vladivostok" => (43.10, 131.87),
        "Asia/Yekaterinburg" => (56.85, 60.61),
        "Asia/Yerevan" => (40.18, 44.51),

        // Atlantic
        "Atlantic/Azores" => (37.74, -25.67),
        "Atlantic/Cape_Verde" => (14.93, -23.51),
        "Atlantic/Reykjavik" => (64.13, -21.82),

        // Australia
        "Australia/Adelaide" => (-34.93, 138.60),
        "Australia/Brisbane" => (-27.47, 153.02),
        "Australia/Darwin" => (-12.46, 130.84),
        "Australia/Hobart" => (-42.88, 147.33),
        "Australia/Melbourne" => (-37.81, 144.96),
        "Australia/Perth" => (-31.95, 115.86),
        "Australia/Sydney" => (-33.87, 151.21),

        // Europe
        "Europe/Amsterdam" => (52.37, 4.90),
        "Europe/Athens" => (37.97, 23.73),
        "Europe/Belgrade" => (44.80, 20.47),
        "Europe/Berlin" => (52.52, 13.40),
        "Europe/Brussels" => (50.85, 4.35),
        "Europe/Bucharest" => (44.43, 26.10),
        "Europe/Budapest" => (47.50, 19.04),
        "Europe/Copenhagen" => (55.68, 12.57),
        "Europe/Dublin" => (53.33, -6.25),
        "Europe/Helsinki" => (60.17, 24.94),
        "Europe/Istanbul" => (41.01, 28.96),
        "Europe/Kiev" | "Europe/Kyiv" => (50.45, 30.52),
        "Europe/Lisbon" => (38.72, -9.14),
        "Europe/Ljubljana" => (46.05, 14.51),
        "Europe/London" => (51.51, -0.13),
        "Europe/Luxembourg" => (49.61, 6.13),
        "Europe/Madrid" => (40.42, -3.70),
        "Europe/Minsk" => (53.90, 27.57),
        "Europe/Moscow" => (55.75, 37.62),
        "Europe/Nicosia" => (35.17, 33.37),
        "Europe/Oslo" => (59.91, 10.75),
        "Europe/Paris" => (48.86, 2.35),
        "Europe/Prague" => (50.09, 14.42),
        "Europe/Riga" => (56.95, 24.11),
        "Europe/Rome" => (41.90, 12.48),
        "Europe/Sarajevo" => (43.85, 18.36),
        "Europe/Skopje" => (41.99, 21.43),
        "Europe/Sofia" => (42.70, 23.32),
        "Europe/Stockholm" => (59.33, 18.07),
        "Europe/Tallinn" => (59.44, 24.75),
        "Europe/Tirane" => (41.33, 19.82),
        "Europe/Vilnius" => (54.69, 25.28),
        "Europe/Vienna" => (48.21, 16.37),
        "Europe/Warsaw" => (52.23, 21.01),
        "Europe/Zagreb" => (45.81, 15.98),
        "Europe/Zurich" => (47.38, 8.54),

        // Indian Ocean
        "Indian/Maldives" => (4.17, 73.51),
        "Indian/Mauritius" => (-20.16, 57.50),

        // Pacific
        "Pacific/Auckland" => (-36.87, 174.77),
        "Pacific/Fiji" => (-18.14, 178.44),
        "Pacific/Guam" => (13.47, 144.75),
        "Pacific/Honolulu" => (21.31, -157.86),
        "Pacific/Midway" => (28.21, -177.38),
        "Pacific/Port_Moresby" => (-9.46, 147.18),

        // UTC
        "UTC" | "GMT" | "Etc/UTC" | "Etc/GMT" => (0.0, 0.0),

        _ => return None,
    })
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
        let (lat, lon) = timezone_coords("Europe/Berlin").unwrap();
        assert!((lat - 52.52).abs() < 0.1);
        assert!((lon - 13.40).abs() < 0.1);
    }

    #[test]
    fn test_timezone_coords_unknown() {
        assert!(timezone_coords("Invalid/Timezone").is_none());
    }

    #[test]
    fn test_timezone_coords_aliases() {
        assert!(timezone_coords("Asia/Kolkata").is_some());
        assert!(timezone_coords("Asia/Calcutta").is_some());
        let kolkata = timezone_coords("Asia/Kolkata").unwrap();
        let calcutta = timezone_coords("Asia/Calcutta").unwrap();
        assert_eq!(kolkata, calcutta);
    }
}
