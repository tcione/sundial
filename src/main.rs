use chrono::NaiveTime;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

mod config;
use config::*;

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
struct SunTimes {
    sunrise: NaiveTime,
    sunset: NaiveTime,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct Cache {
    sun_times: SunTimes,
}

#[derive(Debug, PartialEq)]
struct ScreenState {
    temperature: String,
    gamma: String,
}

#[derive(Debug, Deserialize)]
struct ApiResponse {
    results: ApiResults,
}

#[derive(Debug, Deserialize)]
struct ApiResults {
    sunrise: String,
    sunset: String,
}

fn get_data_dir() -> Result<PathBuf, Box<dyn std::error::Error>> {
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

fn load_cache(config: &Config, data_dir: &PathBuf) -> Result<Option<Cache>, Box<dyn std::error::Error>> {
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

fn persist_to_cache(config: &Config, data_dir: &PathBuf, sun_times: &SunTimes) -> Result<bool, Box<dyn std::error::Error>> {
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

fn build_sunrisesunset_url(config: &Config) -> String {
    format!(
        "https://api.sunrisesunset.io/json?lat={}&lng={}&time_format=unix",
        config.location.latitude,
        config.location.longitude,
    )
}

fn fetch_sunrise_sunset(url: &str) -> Result<SunTimes, Box<dyn std::error::Error>> {
    let response = reqwest::blocking::get(url)?;
    let api_response: ApiResponse = response.json()?;

    let sunrise_timestamp: i64 = api_response.results.sunrise.parse()?;
    let sunset_timestamp: i64 = api_response.results.sunset.parse()?;

    let sunrise = chrono::DateTime::from_timestamp(sunrise_timestamp, 0).ok_or("Invalid sunrise timestamp")?.time();
    let sunset = chrono::DateTime::from_timestamp(sunset_timestamp, 0).ok_or("Invalid sunset timestamp")?.time();

    Ok(SunTimes { sunrise, sunset })
}

fn calculate_screen_state(target_time: NaiveTime, sun_times: &SunTimes, config: &Config) -> ScreenState {
    let is_day = target_time >= sun_times.sunrise && target_time < sun_times.sunset;

    if is_day {
        return ScreenState {
            temperature: config.screen.day_temperature.clone(),
            gamma: config.screen.day_gamma.clone(),
        };
    }

    ScreenState {
        temperature: config.screen.night_temperature.clone(),
        gamma: config.screen.night_gamma.clone(),
    }
}

fn start_hyprsunset() -> Result<(), Box<dyn std::error::Error>> {
    let hyprsunset_process = std::process::Command::new("pgrep")
        .arg("hyprsunset")
        .output()?;
    let is_hyprsunset_running = hyprsunset_process.status.success();
    if is_hyprsunset_running {
        return Ok(());
    }

    std::process::Command::new("systemctl")
        .args(["--user", "start", "hyprsunset"])
        .output()?;

    Ok(())
}

fn manage_screen(config_dir: PathBuf, data_dir: PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    let config = load_config(config_dir)?;
    let sun_times = match load_cache(&config, &data_dir) {
        Ok(Some(cache)) => { cache.sun_times },
        Ok(None) => {
            let url = build_sunrisesunset_url(&config);
            let sun_times = fetch_sunrise_sunset(&url)?;
            let _ = persist_to_cache(&config, &data_dir, &sun_times);
            sun_times
        },
        Err(_) => {
            let url = build_sunrisesunset_url(&config);
            let sun_times = fetch_sunrise_sunset(&url)?;
            let _ = persist_to_cache(&config, &data_dir, &sun_times);
            sun_times
        }
    };
    let now = chrono::Utc::now().time();
    let screen_state = calculate_screen_state(now, &sun_times, &config);

    std::process::Command::new("hyprctl")
        .args(["hyprsunset", "temperature", &screen_state.temperature])
        .output()?;
    std::process::Command::new("hyprctl")
        .args(["hyprsunset", "gamma", &screen_state.gamma])
        .output()?;

    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Sundial starting...");

    let config_dir = get_config_dir()?;
    let data_dir = get_data_dir()?;

    start_hyprsunset()?;
    manage_screen(config_dir, data_dir)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use mockito::Server;

    fn create_test_config() -> Config {
        Config {
            location: LocationConfig {
                latitude: "52.56".to_string(),
                longitude: "13.39".to_string(),
            },
            screen: ScreenConfig {
                day_temperature: "6000".to_string(),
                day_gamma: "100".to_string(),
                night_temperature: "2800".to_string(),
                night_gamma: "80".to_string(),
            },
            cache: CacheConfig {
                enabled: false,
            }
        }
    }

    #[test]
    fn test_fetch_sunrise_sunset() {
        let mut server = Server::new();
        let url_path = format!("/json?lat={}&lng={}&time_format=unix", BERLIN_LAT, BERLIN_LON);
        let mock = server.mock("GET", url_path.as_str())
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{
              "results": {
                "date": "2025-08-17",
                "sunrise": "1755402810",
                "sunset": "1755455425",
                "first_light": "1755393963",
                "last_light": "1755464272",
                "dawn": "1755400509",
                "dusk": "1755457726",
                "solar_noon": "1755429118",
                "golden_hour": "1755452571",
                "day_length": "14:36:55",
                "timezone": "UTC",
                "utc_offset": 0
              },
              "status": "OK"
            }"#)
            .create();
        let mock_url = format!("{}{}", server.url(), url_path);

        let result = fetch_sunrise_sunset(&mock_url).unwrap();
        let expected_result = SunTimes {
            sunrise: chrono::DateTime::from_timestamp(1755402810, 0).unwrap().time(),
            sunset: chrono::DateTime::from_timestamp(1755455425, 0).unwrap().time(),
        };

        assert_eq!(result, expected_result);
        mock.assert();
    }

    #[test]
    fn test_calculate_screen_state() {
        let config = create_test_config();
        let sunrise = NaiveTime::from_hms_opt(6, 0, 0).unwrap();
        let sunset = NaiveTime::from_hms_opt(18, 0, 0).unwrap();
        let sun_times = SunTimes { sunrise, sunset };
        let test_cases = vec![
            (
                NaiveTime::from_hms_opt(02, 0, 00).unwrap(),
                config.screen.night_temperature.clone(),
                config.screen.night_gamma.clone(),
                "Before dawn"
            ),
            (
                NaiveTime::from_hms_opt(05, 0, 59).unwrap(),
                config.screen.night_temperature.clone(),
                config.screen.night_gamma.clone(),
                "Right before sunrise"
            ),
            (
                NaiveTime::from_hms_opt(06, 0, 00).unwrap(),
                config.screen.day_temperature.clone(),
                config.screen.day_gamma.clone(),
                "Sunrise"
            ),
            (
                NaiveTime::from_hms_opt(10, 0, 00).unwrap(),
                config.screen.day_temperature.clone(),
                config.screen.day_gamma.clone(),
                "Day"
            ),
            (
                NaiveTime::from_hms_opt(18, 0, 00).unwrap(),
                config.screen.night_temperature.clone(),
                config.screen.night_gamma.clone(),
                "Sunset"
            ),
            (
                NaiveTime::from_hms_opt(22, 0, 00).unwrap(),
                config.screen.night_temperature.clone(),
                config.screen.night_gamma.clone(),
                "Night"
            ),
        ];

        for (time, expected_temperature, expected_gamma, description) in test_cases {
            let screen_state = calculate_screen_state(time, &sun_times, &config);
            let expected_screen_state = ScreenState {
                temperature: expected_temperature,
                gamma: expected_gamma,
            };
            assert_eq!(screen_state, expected_screen_state, "Screen state failed for {}", description);
        }
    }

    #[test]
    fn test_cache_persist_and_load() {
        let temp_dir = std::env::temp_dir().join("sundial_test_cache_persist_load");
        let _ = std::fs::remove_dir_all(&temp_dir);
        std::fs::create_dir_all(&temp_dir).unwrap();

        let mut config = create_test_config();
        config.cache.enabled = true;

        let sun_times = SunTimes {
            sunrise: NaiveTime::from_hms_opt(6, 30, 0).unwrap(),
            sunset: NaiveTime::from_hms_opt(18, 45, 0).unwrap(),
        };

        let persist_result = persist_to_cache(&config, &temp_dir, &sun_times);
        assert!(persist_result.unwrap());

        let load_result = load_cache(&config, &temp_dir);
        let loaded_cache = load_result.unwrap().unwrap();
        assert_eq!(loaded_cache.sun_times, sun_times);

        let _ = std::fs::remove_dir_all(&temp_dir);
    }
}
