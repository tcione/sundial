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

fn main() {
    println!("Sundial starting...");

    match fetch_sunrise_sunset() {
        Ok(sun_times) => {
            println!("Sun times: {:?}", sun_times);
            let now = chrono::Utc::now().time();
            println!("Current UTC time: {:?}", now);
            let is_day = now >= sun_times.sunrise && now < sun_times.sunset;
            println!("Is day: {}", is_day);
        },
        Err(e) => println!("Error: {}", e),
    }

    // TODO: check process status
    // TODO: update temperature if needed
}
