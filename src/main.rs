use std::path::PathBuf;

mod config;
use config::{Config, get_config_dir, load_config};

mod sun_times;
use sun_times::{SunTimes, build_sunrisesunset_url, fetch_sunrise_sunset};

mod screen;
use screen::{calculate_screen_state};

mod cache;
use cache::{get_data_dir, load_cache, persist_to_cache};

struct Application {
    config: Config,
    data_dir: PathBuf,
}

impl Application {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let config_dir = get_config_dir()?;
        let data_dir = get_data_dir()?;
        let config = load_config(config_dir.clone())?;

        Ok(Application { config, data_dir })
    }

    pub fn run(&self) -> Result<(), Box<dyn std::error::Error>> {
        self.start_hyprsunset()?;
        self.manage_screen()?;

        Ok(())
    }

    fn start_hyprsunset(&self) -> Result<(), Box<dyn std::error::Error>> {
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

    fn get_sun_times(&self) -> Result<SunTimes, Box<dyn std::error::Error>> {
        Ok(
            match load_cache(&self.config, &self.data_dir) {
                Ok(Some(cache)) => { cache.sun_times },
                Ok(None) => {
                    let url = build_sunrisesunset_url(&self.config);
                    let sun_times = fetch_sunrise_sunset(&url)?;
                    persist_to_cache(&self.config, &self.data_dir, &sun_times)?;
                    sun_times
                },
                Err(_) => {
                    let url = build_sunrisesunset_url(&self.config);
                    let sun_times = fetch_sunrise_sunset(&url)?;
                    persist_to_cache(&self.config, &self.data_dir, &sun_times)?;
                    sun_times
                }
            }
        )
    }

    fn manage_screen(&self) -> Result<(), Box<dyn std::error::Error>> {
        let sun_times = self.get_sun_times()?;
        let now = chrono::Utc::now().time();
        let screen_state = calculate_screen_state(now, &sun_times, &self.config);

        std::process::Command::new("hyprctl")
            .args(["hyprsunset", "temperature", &screen_state.temperature])
            .output()?;
        std::process::Command::new("hyprctl")
            .args(["hyprsunset", "gamma", &screen_state.gamma])
            .output()?;

        Ok(())
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    Application::new()?.run()
}
