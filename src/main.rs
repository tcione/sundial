use std::path::PathBuf;

mod config;
use config::*;

mod sun_times;
use sun_times::*;

mod screen;
use screen::*;

mod cache;
use cache::*;

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
