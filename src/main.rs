use std::path::PathBuf;
use notify_rust::{Notification, Timeout, Urgency};
use log::{info, warn, error, debug };

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

const NAME: &str = env!("CARGO_PKG_NAME");
const VERSION: &str = env!("CARGO_PKG_VERSION");

impl Application {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        env_logger::init();

        let config_dir = get_config_dir()?;
        let data_dir = get_data_dir()?;
        let config = load_config(config_dir.clone())?;

        info!("Starting out {}", NAME);
        info!("Version: {}", VERSION);

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
            debug!("Hyprsunset is running. Process: {:?}", hyprsunset_process);
            return Ok(());
        }

        let new_hyprsunset_process = std::process::Command::new("systemctl")
            .args(["--user", "start", "hyprsunset"])
            .output()?;
        debug!("Starting hyprsunset. Process: {:?}", new_hyprsunset_process);

        Ok(())
    }

    fn get_sun_times(&self) -> Result<SunTimes, Box<dyn std::error::Error>> {
        Ok(
            match load_cache(&self.config, &self.data_dir) {
                Ok(Some(cache)) => {
                    let cached_sun_times = cache.sun_times;

                    debug!("Cached sun_times in UTC: {:?}", cached_sun_times);

                    cached_sun_times
                },
                Ok(None) => {
                    let url = build_sunrisesunset_url(&self.config);
                    let sun_times = fetch_sunrise_sunset(&url)?;
                    persist_to_cache(&self.config, &self.data_dir, &sun_times)?;

                    debug!("[No cache] Fresh sun_times in UTC: {:?}", sun_times);

                    sun_times
                },
                Err(_) => {
                    let url = build_sunrisesunset_url(&self.config);
                    let sun_times = fetch_sunrise_sunset(&url)?;
                    persist_to_cache(&self.config, &self.data_dir, &sun_times)?;

                    warn!("[Cache error] Fresh sun_times in UTC: {:?}", sun_times);

                    sun_times
                }
            }
        )
    }

    fn manage_screen(&self) -> Result<(), Box<dyn std::error::Error>> {
        let sun_times = self.get_sun_times()?;
        let now = chrono::Utc::now().time();
        let screen_state = calculate_screen_state(now, &sun_times, &self.config);

        let info_log = format!("Setting screen to: {:?}", screen_state);
        info!("{}", &info_log);

        if log::log_enabled!(log::Level::Trace) {
            // Swallow error since it doesn't really matter
            // if it errors out here
            let _ = Notification::new()
                    .summary("Sundial")
                    .body(&info_log)
                    .timeout(Timeout::Milliseconds(6000))
                    .urgency(Urgency::Low)
                    .show()?;
        }

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
    match Application::new()?.run() {
        Ok(()) => { Ok(()) },
        Err(error) => {
            let err = format!("Error: {:?}", error);
            error!("{}", &err);
            Notification::new()
                .summary("Sundial")
                .body(&err)
                .timeout(Timeout::Milliseconds(6000))
                .urgency(Urgency::Critical)
                .show()
                .unwrap();
            panic!("ERROR!ERROR!ERROR!");
        }
    }
}
