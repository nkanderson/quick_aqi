//! AQI library
//!
//! This library provides supporting functionality for AQI calculations and
//! translations to EPA specified AQI color ranges. It supports both std and
//! no_std environments, but is best used on systems with hardware floating
//! point support.

#![no_std]

#[cfg(feature = "std")]
extern crate std;

/// Color enum provides colors corresponding to EPA AQI levels
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Color {
    Green,
    Yellow,
    Orange,
    Red,
    Purple,
    DarkPurple,
}

/// Calulate the AQI for the provided PM2.5 value.
///
/// # Arguments
///
/// * `pm25` - The PM 2.5 value from the sensor
///
/// # Returns
///
/// The calculated AQI value using breakpoints and a formula
/// provided by the EPA. These values may be confirmed using
/// the calculator at https://www.airnow.gov/aqi/aqi-calculator-concentration/
///
/// # Examples
///
/// ```
/// let pm25_concentration = 41;
/// let aqi = calculate_aqi(pm25_concentration as f32);
/// assert_eq!(115, aqi);
///
/// let pm25_concentration = 7;
/// let aqi = calculate_aqi(pm25_concentration as f32);
/// assert_eq!(39, aqi);
/// ```
pub fn calculate_aqi(pm25: f32) -> u16 {
    // AQI breakpoints for PM2.5
    // Updated in 2024, see the following from the EPA:
    // https://www.epa.gov/system/files/documents/2024-02/pm-naaqs-air-quality-index-fact-sheet.pdf
    // https://document.airnow.gov/technical-assistance-document-for-the-reporting-of-daily-air-quailty.pdf
    const PM25_BREAKPOINTS: [(f32, f32); 6] = [
        (0.0, 9.0),     // Good
        (9.1, 35.4),    // Moderate
        (35.5, 55.4),   // Unhealthy for Sensitive Groups
        (55.5, 125.4),  // Unhealthy
        (125.5, 225.4), // Very Unhealthy
        (225.5, 500.0), // Hazardous
    ];

    // AQI values corresponding to breakpoints
    const AQI_BREAKPOINTS: [(u16, u16); 6] = [
        (0, 50),    // Good
        (51, 100),  // Moderate
        (101, 150), // Unhealthy for Sensitive Groups
        (151, 200), // Unhealthy
        (201, 300), // Very Unhealthy
        (301, 500), // Hazardous
    ];

    // Find the appropriate breakpoint range
    for i in 0..PM25_BREAKPOINTS.len() {
        let (pm_low, pm_high) = PM25_BREAKPOINTS[i];
        if pm25 >= pm_low && pm25 <= pm_high {
            let (aqi_low, aqi_high) = AQI_BREAKPOINTS[i];

            // Linear interpolation formula transcribed from EPA documentation
            // AQI = ((AQIhigh - AQIlow) / (PMhigh - PMlow)) * (PMactual - PMlow) + AQIlow
            let aqi = ((aqi_high - aqi_low) as f32 / (pm_high - pm_low)) * (pm25 - pm_low)
                + aqi_low as f32;
            return libm::roundf(aqi) as u16;
        }
    }

    // If PM2.5 is above 500, return the maximum AQI value
    500
}

/// Provides a Color enum variant value based on the
/// specified AQI value. Uses the ranges provided by the
/// EPA for mapping AQI to color.
///
/// # Arguments
///
/// * `aqi` - The calculated AQI
///
/// # Returns
///
/// A Color enum variant.
///
/// # Examples
///
/// ```
/// let data = parse_data(&sensor_data).unwrap_or_else(|err| {
///     hprintln!("Error parsing data: {}", err);
///     Pmsa003iData::default()
/// });
///
/// let pm25_concentration = data.pm2_5_env;
/// let aqi = calculate_aqi(pm25_concentration as f32);
/// let color = get_aqi_color(aqi);
/// ```
pub fn get_aqi_color(aqi: u16) -> Color {
    match aqi {
        0..=50 => Color::Green,
        51..=100 => Color::Yellow,
        101..=150 => Color::Orange,
        151..=200 => Color::Red,
        201..=300 => Color::Purple,
        _ => Color::DarkPurple,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_aqi() {
        // These expected values were confirmed using
        // https://www.airnow.gov/aqi/aqi-calculator-concentration/
        assert_eq!(calculate_aqi(0.0), 0);
        assert_eq!(calculate_aqi(4.5), 25);
        assert_eq!(calculate_aqi(9.0), 50);
        assert_eq!(calculate_aqi(35.5), 101);
        assert_eq!(calculate_aqi(45.0), 124);
        assert_eq!(calculate_aqi(55.4), 150);
        assert_eq!(calculate_aqi(55.5), 151);
        assert_eq!(calculate_aqi(90.0), 175);
        assert_eq!(calculate_aqi(125.4), 200);
        assert_eq!(calculate_aqi(125.5), 201);
        assert_eq!(calculate_aqi(175.0), 250);
        assert_eq!(calculate_aqi(225.4), 300);
        assert_eq!(calculate_aqi(225.5), 301);
        assert_eq!(calculate_aqi(500.0), 500);
    }

    #[test]
    fn test_get_aqi_color() {
        assert_eq!(get_aqi_color(0), Color::Green);
        assert_eq!(get_aqi_color(25), Color::Green);
        assert_eq!(get_aqi_color(50), Color::Green);
        assert_eq!(get_aqi_color(51), Color::Yellow);
        assert_eq!(get_aqi_color(75), Color::Yellow);
        assert_eq!(get_aqi_color(100), Color::Yellow);
        assert_eq!(get_aqi_color(101), Color::Orange);
        assert_eq!(get_aqi_color(125), Color::Orange);
        assert_eq!(get_aqi_color(150), Color::Orange);
        assert_eq!(get_aqi_color(151), Color::Red);
        assert_eq!(get_aqi_color(175), Color::Red);
        assert_eq!(get_aqi_color(200), Color::Red);
        assert_eq!(get_aqi_color(201), Color::Purple);
        assert_eq!(get_aqi_color(250), Color::Purple);
        assert_eq!(get_aqi_color(300), Color::Purple);
        assert_eq!(get_aqi_color(301), Color::DarkPurple);
        assert_eq!(get_aqi_color(400), Color::DarkPurple);
        assert_eq!(get_aqi_color(500), Color::DarkPurple);
        assert_eq!(get_aqi_color(999), Color::DarkPurple);
    }
}
