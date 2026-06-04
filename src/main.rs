use std::path::PathBuf;
use notify_rust::{Notification, Timeout, Urgency};
use log::{info, error, debug };

mod config;
use config::{Config, get_config_dir, load_config};

mod sun_times;
use sun_times::{SunTimes, compute_sun_times};

mod screen;
use screen::{calculate_screen_state};

mod cache;
use cache::get_data_dir;

mod location;
use location::resolve_location;

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

    fn get_sun_times(&self) -> SunTimes {
        let (latitude, longitude) = resolve_location(&self.config, &self.data_dir);
        let today = chrono::Utc::now().date_naive();

        let sun_times = compute_sun_times(latitude, longitude, today);
        debug!("Computed sun_times in UTC: {:?}", sun_times);

        sun_times
    }

    fn manage_screen(&self) -> Result<(), Box<dyn std::error::Error>> {
        let sun_times = self.get_sun_times();
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

fn report_failure(error: Box<dyn std::error::Error>) -> ! {
    error!("Sundial failed: {:?}", error);

    let _ = Notification::new()
        .summary("Sundial")
        .body("Sundial isn't working properly. Check the logs: journalctl --user")
        .timeout(Timeout::Milliseconds(6000))
        .urgency(Urgency::Normal)
        .show();

    std::process::exit(1);
}

fn main() {
    let application = match Application::new() {
        Ok(application) => application,
        Err(error) => report_failure(error),
    };

    if let Err(error) = application.run() {
        report_failure(error);
    }
}
