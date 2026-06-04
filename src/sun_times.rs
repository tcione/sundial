use chrono::{NaiveDate, NaiveTime};
use serde::{Deserialize, Serialize};
use sunrise::{Coordinates, SolarDay, SolarEvent};

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct SunTimes {
    pub sunrise: NaiveTime,
    pub sunset: NaiveTime,
}

pub fn compute_sun_times(latitude: f64, longitude: f64, date: NaiveDate) -> SunTimes {
    let default_sunrise = NaiveTime::from_hms_opt(6, 0, 0).unwrap();
    let default_sunset = NaiveTime::from_hms_opt(18, 0, 0).unwrap();

    let Some(coordinates) = Coordinates::new(latitude, longitude) else {
        return SunTimes {
            sunrise: default_sunrise,
            sunset: default_sunset,
        };
    };

    let solar_day = SolarDay::new(coordinates, date);
    let sunrise = solar_day
        .event_time(SolarEvent::Sunrise)
        .map(|moment| moment.time())
        .unwrap_or(default_sunrise);
    let sunset = solar_day
        .event_time(SolarEvent::Sunset)
        .map(|moment| moment.time())
        .unwrap_or(default_sunset);

    SunTimes { sunrise, sunset }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compute_sun_times() {
        let date = NaiveDate::from_ymd_opt(1970, 1, 1).unwrap();
        let result = compute_sun_times(0.0, 0.0, date);

        assert_eq!(result.sunrise, NaiveTime::from_hms_opt(5, 59, 54).unwrap());
        assert_eq!(result.sunset, NaiveTime::from_hms_opt(18, 7, 8).unwrap());
    }

    #[test]
    fn test_compute_sun_times_polar_fallback() {
        let date = NaiveDate::from_ymd_opt(1970, 8, 1).unwrap();
        let result = compute_sun_times(85.0, 0.0, date);

        assert_eq!(result.sunrise, NaiveTime::from_hms_opt(6, 0, 0).unwrap());
        assert_eq!(result.sunset, NaiveTime::from_hms_opt(18, 0, 0).unwrap());
    }
}
