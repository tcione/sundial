use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crate::config::Config;
use crate::sun_times::SunTimes;

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct Cache {
    pub sun_times: SunTimes,
}

pub fn get_data_dir() -> Result<PathBuf, Box<dyn std::error::Error>> {
    let dirs = directories::ProjectDirs::from("", "", "sundial")
        .ok_or("Could not find config directory")?;

    let data_dir = dirs.data_dir();
    std::fs::create_dir_all(&data_dir)?;

    Ok(data_dir.to_path_buf())
}


fn cache_file(data_dir: &PathBuf) -> PathBuf {
    let today = chrono::Utc::now().format("%Y-%m-%d").to_string();

    return data_dir.join(format!("cache-{}.json", today))
}

pub fn load_cache(config: &Config, data_dir: &PathBuf) -> Result<Option<Cache>, Box<dyn std::error::Error>> {
    if !config.cache.enabled {
        return Ok(None);
    }

    let cache_file = cache_file(data_dir);

    if !cache_file.exists() {
        return Ok(None);
    }

    let cache_content = std::fs::read_to_string(cache_file)?;
    let cache: Cache = serde_json::from_str(&cache_content)?;

    Ok(Some(cache))
}

pub fn persist_to_cache(config: &Config, data_dir: &PathBuf, sun_times: &SunTimes) -> Result<bool, Box<dyn std::error::Error>> {
    if !config.cache.enabled {
        return Ok(false)
    }

    std::fs::remove_dir_all(&data_dir)?;
    std::fs::create_dir_all(&data_dir)?;

    let cache_file = cache_file(data_dir);
    let cache = Cache { sun_times: sun_times.clone() };
    let cache_content = serde_json::to_string(&cache)?;

    std::fs::write(cache_file, cache_content)?;

    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveTime;
    use crate::config::get_test_config;

    #[test]
    fn test_cache_enabled_persist_and_load() {
        let temp_dir = std::env::temp_dir().join("sundial_test_cache_enabled_persist_load");
        let _ = std::fs::remove_dir_all(&temp_dir);
        std::fs::create_dir_all(&temp_dir).unwrap();

        let mut config = get_test_config();
        config.cache.enabled = true;

        let sun_times = SunTimes {
            sunrise: NaiveTime::from_hms_opt(6, 30, 0).unwrap(),
            sunset: NaiveTime::from_hms_opt(18, 45, 0).unwrap(),
        };

        // no cache file exists
        let load_result = load_cache(&config, &temp_dir);
        assert_eq!(load_result.unwrap(), None);

        let persist_result = persist_to_cache(&config, &temp_dir, &sun_times);
        assert!(persist_result.unwrap());

        let load_result = load_cache(&config, &temp_dir);
        let loaded_cache = load_result.unwrap().unwrap();
        assert_eq!(loaded_cache.sun_times, sun_times);

        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_cache_disabled_persist_and_load() {
        let temp_dir = std::env::temp_dir().join("sundial_test_cache_disabled_persist_load");
        let _ = std::fs::remove_dir_all(&temp_dir);
        std::fs::create_dir_all(&temp_dir).unwrap();

        let mut config = get_test_config();
        config.cache.enabled = false;

        let sun_times = SunTimes {
            sunrise: NaiveTime::from_hms_opt(6, 30, 0).unwrap(),
            sunset: NaiveTime::from_hms_opt(18, 45, 0).unwrap(),
        };

        let persist_result = persist_to_cache(&config, &temp_dir, &sun_times);
        assert_eq!(persist_result.unwrap(), false);

        let load_result = load_cache(&config, &temp_dir);
        assert_eq!(load_result.unwrap(), None);

        let _ = std::fs::remove_dir_all(&temp_dir);
    }
}
