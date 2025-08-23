use chrono::NaiveTime;
use serde::{Deserialize, Serialize};
use crate::config::Config;

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct SunTimes {
    pub sunrise: NaiveTime,
    pub sunset: NaiveTime,
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

pub fn build_sunrisesunset_url(config: &Config) -> String {
    format!(
        "https://api.sunrisesunset.io/json?lat={}&lng={}&time_format=unix",
        config.location.latitude,
        config.location.longitude,
    )
}

pub fn fetch_sunrise_sunset(url: &str) -> Result<SunTimes, Box<dyn std::error::Error>> {
    let response = reqwest::blocking::get(url)?;
    let api_response: ApiResponse = response.json()?;

    let sunrise_timestamp: i64 = api_response.results.sunrise.parse()?;
    let sunset_timestamp: i64 = api_response.results.sunset.parse()?;

    let sunrise = chrono::DateTime::from_timestamp(sunrise_timestamp, 0).ok_or("Invalid sunrise timestamp")?.time();
    let sunset = chrono::DateTime::from_timestamp(sunset_timestamp, 0).ok_or("Invalid sunset timestamp")?.time();

    Ok(SunTimes { sunrise, sunset })
}

#[cfg(test)]
mod tests {
    use super::*;
    use mockito::Server;

    #[test]
    fn test_fetch_sunrise_sunset() {
        let mut server = Server::new();
        let url_path = format!("/json?lat={}&lng={}&time_format=unix", "13.54", "43.12");
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
}
