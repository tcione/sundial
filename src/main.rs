use chrono::NaiveTime;
use serde::Deserialize;

#[derive(Debug)]
struct SunTimes {
    sunrise: NaiveTime,
    sunset: NaiveTime,
}

#[derive(Debug)]
struct ScreenState {
    temperature: &'static str,
    gamma: &'static str,
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

const BERLIN_LAT: &str = "52.56";
const BERLIN_LON: &str = "13.39";
const DAY_TEMPERATURE: &str = "6000";
const DAY_GAMMA: &str = "100";
const NIGHT_TEMPERATURE: &str = "2800";
const NIGHT_GAMMA: &str = "80";

fn build_sunrisesunset_url() -> String {
    format!(
        "https://api.sunrisesunset.io/json?lat={}&lng={}&time_format=unix",
        BERLIN_LAT, BERLIN_LON
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

fn calculate_screen_state(target_time: NaiveTime, sun_times: &SunTimes) -> ScreenState {
    let is_day = target_time >= sun_times.sunrise && target_time < sun_times.sunset;

    if is_day {
        return ScreenState {
            temperature: DAY_TEMPERATURE,
            gamma: DAY_GAMMA,
        };
    }

    ScreenState {
        temperature: NIGHT_TEMPERATURE,
        gamma: NIGHT_GAMMA,
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

fn manage_screen() -> Result<(), Box<dyn std::error::Error>> {
    let url = build_sunrisesunset_url();
    let sun_times = fetch_sunrise_sunset(&url)?;
    let now = chrono::Utc::now().time();
    let screen_state = calculate_screen_state(now, &sun_times);

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

    start_hyprsunset()?;
    manage_screen()?;

    Ok(())
}
