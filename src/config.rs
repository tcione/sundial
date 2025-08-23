use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Config {
    pub location: LocationConfig,
    pub screen: ScreenConfig,
    pub cache: CacheConfig,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct LocationConfig {
    pub latitude: String,
    pub longitude: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ScreenConfig {
    pub day_temperature: String,
    pub day_gamma: String,
    pub night_temperature: String,
    pub night_gamma: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct CacheConfig {
    pub enabled: bool,
}

pub const BERLIN_LAT: &str = "52.56";
pub const BERLIN_LON: &str = "13.39";
const DAY_TEMPERATURE: &str = "6000";
const DAY_GAMMA: &str = "100";
const NIGHT_TEMPERATURE: &str = "2800";
const NIGHT_GAMMA: &str = "80";
const CACHE_ENABLED: bool = true;

pub fn get_config_dir() -> Result<PathBuf, Box<dyn std::error::Error>> {
    let dirs = directories::ProjectDirs::from("", "", "sundial")
        .ok_or("Could not find config directory")?;

    let config_dir = dirs.config_dir();
    std::fs::create_dir_all(&config_dir)?;

    Ok(config_dir.to_path_buf())
}

pub fn load_config(config_dir: PathBuf) -> Result<Config, Box<dyn std::error::Error>> {
    let config_file = config_dir.join("config.toml");

    if config_file.exists() {
        let config_content = std::fs::read_to_string(config_file)?;
        let config: Config = toml::from_str(&config_content)?;

        return Ok(config);
    }

    let default_config = Config {
        location: LocationConfig {
            latitude: BERLIN_LAT.to_string(),
            longitude: BERLIN_LON.to_string(),
        },
        screen: ScreenConfig {
            day_temperature: DAY_TEMPERATURE.to_string(),
            day_gamma: DAY_GAMMA.to_string(),
            night_temperature: NIGHT_TEMPERATURE.to_string(),
            night_gamma: NIGHT_GAMMA.to_string(),
        },
        cache: CacheConfig {
            enabled: CACHE_ENABLED,
        },
    };

    let config_toml = toml::to_string(&default_config)?;
    std::fs::write(config_file, config_toml)?;

    Ok(default_config)
}

#[cfg(test)]
pub fn get_test_config() -> Config {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_config_creates_default() {
        let temp_dir = std::env::temp_dir().join("sundial_test_load_config_creates_default");
        let _ = std::fs::remove_dir_all(&temp_dir);
        let _ = std::fs::create_dir_all(&temp_dir);

        let result = load_config(temp_dir.clone());
        assert!(result.is_ok());

        let config = result.unwrap();

        // Default values
        assert_eq!(config.location.latitude, "52.56");
        assert_eq!(config.location.longitude, "13.39");
        assert_eq!(config.screen.day_temperature, "6000");
        assert_eq!(config.screen.day_gamma, "100");
        assert_eq!(config.screen.night_temperature, "2800");
        assert_eq!(config.screen.night_gamma, "80");

        let config_file = temp_dir.join("config.toml");
        assert!(config_file.exists());

        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_load_config_reads_existing() {
        let temp_dir = std::env::temp_dir().join("sundial_test_load_config_reads_existing");
        let _ = std::fs::remove_dir_all(&temp_dir);
        let _ = std::fs::create_dir_all(&temp_dir);

        let config_file = temp_dir.join("config.toml");
        let custom_config_content = r#"
[location]
latitude = "40.71"
longitude = "-74.12"

[screen]
day_temperature = "5500"
day_gamma = "90"
night_temperature = "3000"
night_gamma = "70"

[cache]
enabled = true
"#;
        std::fs::write(&config_file, custom_config_content).unwrap();

        let result = load_config(temp_dir.clone());
        assert!(result.is_ok());

        let config = result.unwrap();

        // Default values
        assert_eq!(config.location.latitude, "40.71");
        assert_eq!(config.location.longitude, "-74.12");
        assert_eq!(config.screen.day_temperature, "5500");
        assert_eq!(config.screen.day_gamma, "90");
        assert_eq!(config.screen.night_temperature, "3000");
        assert_eq!(config.screen.night_gamma, "70");

        let _ = std::fs::remove_dir_all(&temp_dir);
    }
}
