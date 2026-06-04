use std::path::PathBuf;
use std::time::Duration;

use log::{debug, warn};
use serde::{Deserialize, Serialize};

use crate::config::{Config, LocationMode};

const GEO_URL: &str = "https://ipinfo.io/json";
const GEO_TIMEOUT: Duration = Duration::from_secs(5);
const BOOT_ID_PATH: &str = "/proc/sys/kernel/random/boot_id";
const CACHE_FILE: &str = "location.json";

#[derive(Debug, Serialize, Deserialize)]
struct LocationCache {
    latitude: f64,
    longitude: f64,
    date: String,
    boot_id: String,
}

#[derive(Debug, Deserialize)]
struct IpInfoResponse {
    loc: String,
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

    match detect_location() {
        Ok((latitude, longitude)) => {
            debug!("Detected location via IP: ({}, {})", latitude, longitude);
            persist_location_cache(data_dir, latitude, longitude, &today, boot_id.as_deref());
            (latitude, longitude)
        }
        Err(error) => {
            warn!("IP geolocation failed ({}); falling back to last known location", error);
            cache
                .map(|cache| (cache.latitude, cache.longitude))
                .unwrap_or(fallback)
        }
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

fn detect_location() -> Result<(f64, f64), Box<dyn std::error::Error>> {
    let client = reqwest::blocking::Client::builder()
        .timeout(GEO_TIMEOUT)
        .build()?;
    let response: IpInfoResponse = client.get(GEO_URL).send()?.json()?;

    parse_loc(&response.loc)
}

fn parse_loc(loc: &str) -> Result<(f64, f64), Box<dyn std::error::Error>> {
    let mut parts = loc.split(',');
    let latitude = parts.next().ok_or("missing latitude")?.trim().parse::<f64>()?;
    let longitude = parts.next().ok_or("missing longitude")?.trim().parse::<f64>()?;

    Ok((latitude, longitude))
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
    fn test_parse_loc() {
        assert_eq!(parse_loc("37.38,-122.08").unwrap(), (37.38, -122.08));
        assert_eq!(parse_loc(" 52.56 , 13.39 ").unwrap(), (52.56, 13.39));
    }

    #[test]
    fn test_parse_loc_invalid() {
        assert!(parse_loc("not-a-location").is_err());
        assert!(parse_loc("52.56").is_err());
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
}
