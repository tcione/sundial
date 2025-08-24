use chrono::{NaiveTime, Duration};
use crate::config::*;
use crate::sun_times::SunTimes;

#[derive(Debug, PartialEq)]
pub struct ScreenState {
    pub temperature: String,
    pub gamma: String,
}

const FADE_DURATION : Duration = Duration::minutes(60);
const FADE_STEPS : i64 = 6;
const STEP_DURATION : i64 = FADE_DURATION.num_minutes() / FADE_STEPS;

fn calculate_fade_state(is_day: bool, target_time: NaiveTime, sun_times: &SunTimes, config: &Config) -> ScreenState {
    let temperature_day = config.screen.day_temperature.parse::<i64>().unwrap();
    let temperature_night = config.screen.night_temperature.parse::<i64>().unwrap();
    let temperature_delta = (temperature_day - temperature_night).abs();
    let temperature_step = temperature_delta / FADE_STEPS;

    let gamma_day = config.screen.day_gamma.parse::<i64>().unwrap();
    let gamma_night = config.screen.night_gamma.parse::<i64>().unwrap();
    let gamma_delta = (gamma_day - gamma_night).abs();
    let gamma_step = gamma_delta / FADE_STEPS;

    let diff_base = if is_day { sun_times.sunset } else { sun_times.sunrise };
    let difference_in_mins = (diff_base - target_time).num_minutes();
    let current_step = difference_in_mins / STEP_DURATION;
    let factor  = FADE_STEPS - current_step;

    let temperature;
    let gamma;
    if is_day {
        temperature = temperature_day - (temperature_step * factor);
        gamma = gamma_day - (gamma_step * factor);
    } else {
        temperature = temperature_night + (temperature_step * factor);
        gamma = gamma_night + (gamma_step * factor);
    }

    ScreenState { temperature: temperature.to_string(), gamma: gamma.to_string() }
}

pub fn calculate_screen_state(target_time: NaiveTime, sun_times: &SunTimes, config: &Config) -> ScreenState {
    let is_day = target_time >= sun_times.sunrise && target_time < sun_times.sunset;
    let fading_into_day = !is_day &&
                          target_time > sun_times.sunrise - FADE_DURATION &&
                          target_time < sun_times.sunrise;
    let fading_into_night = is_day &&
                            target_time > sun_times.sunset - FADE_DURATION &&
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
                "Dawn fade step 1"
            ),
            (
                NaiveTime::from_hms_opt(05, 10, 00).unwrap(),
                "3333".to_string(),
                "83".to_string(),
                "Dawn fade step 2"
            ),
            (
                NaiveTime::from_hms_opt(05, 20, 00).unwrap(),
                "3866".to_string(),
                "86".to_string(),
                "Dawn fade step 3"
            ),
            (
                NaiveTime::from_hms_opt(05, 30, 00).unwrap(),
                "4399".to_string(),
                "89".to_string(),
                "Dawn fade step 4"
            ),
            (
                NaiveTime::from_hms_opt(05, 40, 00).unwrap(),
                "4932".to_string(),
                "92".to_string(),
                "Dawn fade step 5"
            ),
            (
                NaiveTime::from_hms_opt(05, 50, 00).unwrap(),
                "5465".to_string(),
                "95".to_string(),
                "Dawn fade step 6"
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
                "Evening fade step 1"
            ),
            (
                NaiveTime::from_hms_opt(17, 10, 00).unwrap(),
                "5467".to_string(),
                "97".to_string(),
                "Evening fade step 2"
            ),
            (
                NaiveTime::from_hms_opt(17, 20, 00).unwrap(),
                "4934".to_string(),
                "94".to_string(),
                "Evening fade step 3"
            ),
            (
                NaiveTime::from_hms_opt(17, 30, 00).unwrap(),
                "4401".to_string(),
                "91".to_string(),
                "Evening fade step 4"
            ),
            (
                NaiveTime::from_hms_opt(17, 40, 00).unwrap(),
                "3868".to_string(),
                "88".to_string(),
                "Evening fade step 5"
            ),
            (
                NaiveTime::from_hms_opt(17, 50, 00).unwrap(),
                "3335".to_string(),
                "85".to_string(),
                "Evening fade step 6"
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
