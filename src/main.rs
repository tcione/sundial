use chrono::NaiveTime;
use serde::Deserialize;

#[derive(Debug)]
struct SunTimes {
    sunrise: NaiveTime,
    sunset: NaiveTime,
}

#[derive(Debug)]
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

fn fetch_sunrise_sunset() -> Result<SunTimes, Box<dyn std::error::Error>> {
    let berlin_lat = "52.56";
    let berlin_lon = "13.39";
    let url = format!(
        "https://api.sunrisesunset.io/json?lat={}&lng={}&time_format=unix",
        berlin_lat, berlin_lon
    );
    let response = reqwest::blocking::get(&url)?;
    let api_response: ApiResponse = response.json()?;

    let sunrise_timestamp: i64 = api_response.results.sunrise.parse()?;
    let sunset_timestamp: i64 = api_response.results.sunset.parse()?;

    let sunrise = chrono::DateTime::from_timestamp(sunrise_timestamp, 0).ok_or("Invalid sunrise timestamp")?.time();
    let sunset = chrono::DateTime::from_timestamp(sunset_timestamp, 0).ok_or("Invalid sunset timestamp")?.time();

    Ok(SunTimes { sunrise, sunset })
}

fn calculate_screen_state(target_time: NaiveTime, sun_times: SunTimes) -> ScreenState {
    let is_day = target_time >= sun_times.sunrise && target_time < sun_times.sunset;

    if is_day {
        return ScreenState {
            temperature: String::from("6000"),
            gamma: String::from("100")
        };
    }

    ScreenState {
        temperature: String::from("2800"),
        gamma: String::from("80")
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
    let sun_times = fetch_sunrise_sunset()?;
    let now = chrono::Utc::now().time();
    let screen_state = calculate_screen_state(now, sun_times);

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
