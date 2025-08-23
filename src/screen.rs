use chrono::NaiveTime;
use crate::config::*;
use crate::sun_times::SunTimes;

#[derive(Debug, PartialEq)]
pub struct ScreenState {
    pub temperature: String,
    pub gamma: String,
}

pub fn calculate_screen_state(target_time: NaiveTime, sun_times: &SunTimes, config: &Config) -> ScreenState {
    let is_day = target_time >= sun_times.sunrise && target_time < sun_times.sunset;

    if is_day {
        return ScreenState {
            temperature: config.screen.day_temperature.clone(),
            gamma: config.screen.day_gamma.clone(),
        };
    }

    ScreenState {
        temperature: config.screen.night_temperature.clone(),
        gamma: config.screen.night_gamma.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::get_test_config;

    #[test]
    fn test_calculate_screen_state() {
        let config = get_test_config();
        let sunrise = NaiveTime::from_hms_opt(6, 0, 0).unwrap();
        let sunset = NaiveTime::from_hms_opt(18, 0, 0).unwrap();
        let sun_times = SunTimes { sunrise, sunset };
        let test_cases = vec![
            (
                NaiveTime::from_hms_opt(02, 0, 00).unwrap(),
                config.screen.night_temperature.clone(),
                config.screen.night_gamma.clone(),
                "Before dawn"
            ),
            (
                NaiveTime::from_hms_opt(05, 0, 59).unwrap(),
                config.screen.night_temperature.clone(),
                config.screen.night_gamma.clone(),
                "Right before sunrise"
            ),
            (
                NaiveTime::from_hms_opt(06, 0, 00).unwrap(),
                config.screen.day_temperature.clone(),
                config.screen.day_gamma.clone(),
                "Sunrise"
            ),
            (
                NaiveTime::from_hms_opt(10, 0, 00).unwrap(),
                config.screen.day_temperature.clone(),
                config.screen.day_gamma.clone(),
                "Day"
            ),
            (
                NaiveTime::from_hms_opt(18, 0, 00).unwrap(),
                config.screen.night_temperature.clone(),
                config.screen.night_gamma.clone(),
                "Sunset"
            ),
            (
                NaiveTime::from_hms_opt(22, 0, 00).unwrap(),
                config.screen.night_temperature.clone(),
                config.screen.night_gamma.clone(),
                "Night"
            ),
        ];

        for (time, expected_temperature, expected_gamma, description) in test_cases {
            let screen_state = calculate_screen_state(time, &sun_times, &config);
            let expected_screen_state = ScreenState {
                temperature: expected_temperature,
                gamma: expected_gamma,
            };
            assert_eq!(screen_state, expected_screen_state, "Screen state failed for {}", description);
        }
    }
}
