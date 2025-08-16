use chrono::NaiveTime;
use serde::Deserialize;

#[derive(Debug)]
struct SunTimes {
    sunrise: NaiveTime,
    sunset: NaiveTime,
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

fn is_hyprsunset_running() -> Result<bool, Box<dyn std::error::Error>> {
    let output = std::process::Command::new("pgrep")
        .arg("hyprsunset")
        .output()?;
    Ok(output.status.success())
}

fn start_hyprsunset() -> Result<(), Box<dyn std::error::Error>> {
    let is_hyprsunset_running = is_hyprsunset_running()?;

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
    let is_day = now >= sun_times.sunrise && now < sun_times.sunset;

    let temperature;
    let gamma;

    if is_day {
        temperature = "6000";
        gamma = "100"
    } else {
        temperature = "2800";
        gamma = "80"
    }

    std::process::Command::new("hyprctl")
        .args(["hyprsunset", "temperature", temperature])
        .output()?;
    std::process::Command::new("hyprctl")
        .args(["hyprsunset", "gamma", gamma])
        .output()?;

    return Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Sundial starting...");

    start_hyprsunset()?;
    manage_screen()?;

    Ok(())
}
