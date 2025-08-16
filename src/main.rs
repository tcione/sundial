fn fetch_sunrise_sunset() -> Result<String, Box<dyn std::error::Error>> {
    let berlin_lat = "52.56";
    let berlin_lon = "13.39";
    let url = format!(
        "https://api.sunrisesunset.io/json?lat={}&lng={}&time_format=military",
        berlin_lat, berlin_lon
    );
    let response = reqwest::blocking::get(&url)?;
    let text = response.text()?;
    Ok(text)
}

fn main() {
    println!("Sundial starting...");

    match fetch_sunrise_sunset() {
        Ok(response) => println!("API Response: {}", response),
        Err(e) => println!("Error: {}", e),
    }
    // TODO: fetch sunrise/sunset data
    // TODO: get current time
    // TODO: determine day/night
    // TODO: check process status
    // TODO: update temperature if needed
}
