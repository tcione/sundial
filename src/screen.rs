use chrono::{NaiveTime, Duration};
use crate::config::Config;
use crate::sun_times::SunTimes;

#[derive(Debug, PartialEq)]
pub struct ScreenState {
    pub temperature: String,
    pub gamma: String,
}

fn calculate_fade_state(is_day: bool, target_time: NaiveTime, sun_times: &SunTimes, config: &Config) -> ScreenState {
    let fade_duration = Duration::minutes(config.screen.fade_duration_in_minutes);
    let fade_steps = fade_duration.num_minutes();

    let temperature_day = config.screen.day_temperature.parse::<i64>().unwrap();
    let temperature_night = config.screen.night_temperature.parse::<i64>().unwrap();
    let temperature_delta = (temperature_day - temperature_night).abs();
    let temperature_step = temperature_delta / fade_steps;

    let gamma_day = config.screen.day_gamma.parse::<i64>().unwrap();
    let gamma_night = config.screen.night_gamma.parse::<i64>().unwrap();
    let gamma_delta = (gamma_day - gamma_night).abs();

    // Gamma is too small of a range (i.e from 80 to 100), so when we progress
    // by the minute, chances are the step lands in 0. To circumvent that, I've
    // decided to convert it to float
    let gamma_step_unrounded = gamma_delta as f64 / fade_steps as f64;
    let gamma_step = format!("{:.2}", gamma_step_unrounded).parse::<f64>().unwrap();

    let diff_base = if is_day { sun_times.sunset } else { sun_times.sunrise };
    let difference_in_mins = (diff_base - target_time).num_minutes();
    let factor  = fade_steps - difference_in_mins;

    let temperature;
    let gamma;
    if is_day {
        temperature = temperature_day - (temperature_step * factor);
        gamma = gamma_day as f64 - (gamma_step * factor as f64);
    } else {
        temperature = temperature_night + (temperature_step * factor);
        gamma = gamma_night as f64 + (gamma_step * factor as f64);
    }

    ScreenState { temperature: temperature.to_string(), gamma: gamma.to_string() }
}

pub fn calculate_screen_state(target_time: NaiveTime, sun_times: &SunTimes, config: &Config) -> ScreenState {
    let fade_duration = Duration::minutes(config.screen.fade_duration_in_minutes);
    let is_day = target_time >= sun_times.sunrise && target_time < sun_times.sunset;
    let fading_into_day = !is_day &&
                          target_time > sun_times.sunrise - fade_duration &&
                          target_time < sun_times.sunrise;
    let fading_into_night = is_day &&
                            target_time > sun_times.sunset - fade_duration &&
                            target_time < sun_times.sunset;

    if fading_into_night || fading_into_day {
        return calculate_fade_state(is_day, target_time, sun_times, config);
    }

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
                NaiveTime::from_hms_opt(02, 00, 00).unwrap(),
                config.screen.night_temperature.clone(),
                config.screen.night_gamma.clone(),
                "Before dawn"
            ),
            (
                NaiveTime::from_hms_opt(05, 00, 00).unwrap(),
                config.screen.night_temperature.clone(),
                config.screen.night_gamma.clone(),
                "Dawn fade at 0 mins"
            ),
            (
                NaiveTime::from_hms_opt(05, 01, 00).unwrap(),
                "2853".to_string(),
                "80.33".to_string(),
                "Dawn fade at 1 min"
            ),
            (
                NaiveTime::from_hms_opt(05, 30, 00).unwrap(),
                "4390".to_string(), // 2800 + (53 * 30)
                "89.9".to_string(), // 80.0 + (0.33 * 30)
                "Dawn fade at 30 mins"
            ),
            (
                NaiveTime::from_hms_opt(05, 59, 00).unwrap(),
                "5927".to_string(),
                "99.47".to_string(),
                "Dawn fade at last minute"
            ),
            (
                NaiveTime::from_hms_opt(06, 00, 00).unwrap(),
                config.screen.day_temperature.clone(),
                config.screen.day_gamma.clone(),
                "Sunrise"
            ),
            (
                NaiveTime::from_hms_opt(10, 00, 00).unwrap(),
                config.screen.day_temperature.clone(),
                config.screen.day_gamma.clone(),
                "Day"
            ),
            (
                NaiveTime::from_hms_opt(17, 00, 00).unwrap(),
                config.screen.day_temperature.clone(),
                config.screen.day_gamma.clone(),
                "Evening fade at 0 mins"
            ),
            (
                NaiveTime::from_hms_opt(17, 01, 00).unwrap(),
                "5947".to_string(),
                "99.67".to_string(),
                "Evening fade at 1 min"
            ),
            (
                NaiveTime::from_hms_opt(17, 30, 00).unwrap(),
                "4410".to_string(), // 6000 - (53 * 30)
                "90.1".to_string(), // 100 - (0.33 * 30)
                "Evening fade at 30 mins"
            ),
            (
                NaiveTime::from_hms_opt(17, 59, 00).unwrap(),
                "2873".to_string(),
                "80.53".to_string(),
                "Evening fade at last min"
            ),
            (
                NaiveTime::from_hms_opt(18, 00, 00).unwrap(),
                config.screen.night_temperature.clone(),
                config.screen.night_gamma.clone(),
                "Sunset"
            ),
            (
                NaiveTime::from_hms_opt(22, 00, 00).unwrap(),
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
