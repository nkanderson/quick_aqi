//! PMSA003I module
//!
//! This module provides supporting functionality for data retrieval
//! and validation from the PMSA003I sensor.

use cortex_m_semihosting::hprintln;
use embassy_stm32::i2c::I2c;
use embassy_stm32::mode::Async;

pub const SENSOR_I2C_ADDR: u8 = 0x12;
const EXPECTED_HEADER: [u8; 2] = [0x42, 0x4D];
const TOTAL_REGISTERS: usize = 32;

/// The Pmsa003iData struct holds all air quality measurements
/// performed by the PMSA003I sensor. Most values are not relevant
/// for the current application.
#[derive(Debug, Default)]
pub struct Pmsa003iData {
    // CF is "Calibration Factory", and generally not useful for our needs.
    _pm1_0_standard: u16, // PM1.0 concentration unit μ g/m3（CF=1，standard particle）
    _pm2_5_standard: u16, // PM2.5 concentration unit μ g/m3（CF=1，standard particle）
    _pm10_standard: u16,  // PM10 concentration unit μ g/m3（CF=1，standard particle）

    // The environmental units take into account factors like ambient pressure.
    // This is typically what is used in an AQI report or forecast.
    _pm1_0_env: u16,    // PM1.0 concentration unit μ g/m3（environmental units）
    pub pm2_5_env: u16, // PM2.5 concentration unit μ g/m3（environmental units）
    _pm10_env: u16,     // PM10 concentration unit μ g/m3  (environmental units)

    // The particle count per volume of air is often used in a cleanroom context.
    _particles_0_3: u16, // Number of particles with diameter beyond 0.3 um in 0.1L of air
    _particles_0_5: u16, // Number of particles with diameter beyond 0.5 um in 0.1L of air
    _particles_1_0: u16, // Number of particles with diameter beyond 1.0 um in 0.1L of air
    _particles_2_5: u16, // Number of particles with diameter beyond 2.5 um in 0.1L of air
    _particles_5_0: u16, // Number of particles with diameter beyond 5.0 um in 0.1L of air
    _particles_10: u16,  // Number of particles with diameter beyond 10 um in 0.1L of air
}

/// Parses raw buffer data from the PMSA003I sensor
/// into a struct with named values.
///
/// # Arguments
///
/// * `buffer` - 32 bytes of data from the sensor
///
/// # Returns
///
/// A Pmsa003iData struct or an error.
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
/// ```
pub fn parse_data(buffer: &[u8]) -> Result<Pmsa003iData, &'static str> {
    if buffer.len() < 32 {
        return Err("Buffer too short, expected at least 32 bytes");
    }

    Ok(Pmsa003iData {
        _pm1_0_standard: u16::from_be_bytes([buffer[4], buffer[5]]),
        _pm2_5_standard: u16::from_be_bytes([buffer[6], buffer[7]]),
        _pm10_standard: u16::from_be_bytes([buffer[8], buffer[9]]),
        _pm1_0_env: u16::from_be_bytes([buffer[10], buffer[11]]),
        pm2_5_env: u16::from_be_bytes([buffer[12], buffer[13]]),
        _pm10_env: u16::from_be_bytes([buffer[14], buffer[15]]),
        _particles_0_3: u16::from_be_bytes([buffer[16], buffer[17]]),
        _particles_0_5: u16::from_be_bytes([buffer[18], buffer[19]]),
        _particles_1_0: u16::from_be_bytes([buffer[20], buffer[21]]),
        _particles_2_5: u16::from_be_bytes([buffer[22], buffer[23]]),
        _particles_5_0: u16::from_be_bytes([buffer[24], buffer[25]]),
        _particles_10: u16::from_be_bytes([buffer[26], buffer[27]]),
    })
}

/// Fetches data in an async manner using a non-blocking I2C instance.
///
/// # Arguments
///
/// * `i2c` - An Embassy Async I2C instance
///
/// # Returns
///
/// A Result containing all retrieved data or an i2c Error.
///
/// # Examples
///
/// ```
/// let mut i2c = I2c::new(
///     p.I2C2,
///     scl,
///     sda,
///     Irqs,
///     p.DMA1_CH4,
///     p.DMA1_CH5,
///     Hertz(100_000),
///     Config::default(),
/// );
///
/// match fetch_data(&mut i2c).await {
///     Ok(sensor_data) => {
///         if let Err(e) = validate_header(&sensor_data[0..2]) {
///             hprintln!("Error validating header: {}", e);
///             continue;
///         }
///     }
///     Err(e) => hprintln!("Error reading registers: {:?}", e),
/// }
/// ```
pub async fn fetch_data(
    i2c: &mut I2c<'_, Async>,
) -> Result<[u8; TOTAL_REGISTERS], embassy_stm32::i2c::Error> {
    let mut buffer = [0u8; TOTAL_REGISTERS];
    i2c.write_read(SENSOR_I2C_ADDR, &[0x00], &mut buffer)
        .await?;
    Ok(buffer)
}

/// Validates the header data retrieved from the PMSA003I sensor.
/// The sensor has hardcoded values of 0x42 and 0x4D in the first
/// two register. This function ensures the retrieved data includes
/// those values.
///
/// # Arguments
///
/// * `header_bytes` - Two bytes of u8 data from the sensor
///
/// # Returns
///
/// Result of Ok(_) or an Err with message.
///
/// # Examples
///
/// ```
/// let mut i2c = I2c::new(
///     p.I2C2,
///     scl,
///     sda,
///     Irqs,
///     p.DMA1_CH4,
///     p.DMA1_CH5,
///     Hertz(100_000),
///     Config::default(),
/// );
///
/// match fetch_data(&mut i2c).await {
///     Ok(sensor_data) => {
///         if let Err(e) = validate_header(&sensor_data[0..2]) {
///             hprintln!("Error validating header: {}", e);
///             continue;
///         }
///     }
///     Err(e) => hprintln!("Error reading registers: {:?}", e),
/// }
/// ```
pub fn validate_header(header_bytes: &[u8]) -> Result<(), &'static str> {
    if header_bytes.is_empty() {
        return Err("Buffer is empty");
    }

    if header_bytes == EXPECTED_HEADER {
        Ok(())
    } else {
        hprintln!(
            "Warning: Invalid header! Got 0x{:02X}{:02X}, expected 0x{:02X}{:02X}",
            header_bytes[0],
            header_bytes[1],
            EXPECTED_HEADER[0],
            EXPECTED_HEADER[1]
        );
        Err("Header validation failed")
    }
}

// #[cfg(test)]
// mod tests {
//     use super::validate_checksum;
//     use defmt::assert_eq;

//     #[test]
//     fn test_validate_checksum() {
//         //
//         // Test validation success
//         //
//         let mut data = [0u8; 32];
//         // Fill first 30 bytes with 1s
//         data[..30].copy_from_slice(&[1; 30]);
//         // Calculate correct checksum
//         let checksum: u16 = data[..30].iter().map(|&b| b as u16).sum();
//         // Store it in big-endian format
//         data[30..32].copy_from_slice(&checksum.to_be_bytes());

//         assert_eq!(validate_checksum(&data), Ok(()));

//         //
//         // Test validation failure
//         //
//         // Incorrect checksum
//         data[30..32].copy_from_slice(&[0x00, 0x00]);

//         assert_eq!(validate_checksum(&data), Err("Checksum validation failed"));

//         //
//         // Incorrect number of bytes
//         //
//         // Less than 32 bytes
//         let data = [0u8; 31];

//         assert_eq!(
//             validate_checksum(&data),
//             Err("Could not validate checksum, incorrect number of bytes received")
//         );
//     }
// }

/// Validates the data and checksum retrieved from the PMSA003I sensor.
/// The sensor provides checksum values against which the payload may be validated.
/// The checksum values are contained in the last 2 bytes returned from the
/// sensor, and are compared against the sum of the first 30 bytes of data.
///
/// # Arguments
///
/// * `checksum_bytes` - Entire array of u8 data from the sensor
///
/// # Returns
///
/// Result of Ok(_) or an Err with message.
///
/// # Examples
///
/// ```
/// let mut i2c = I2c::new(
///     p.I2C2,
///     scl,
///     sda,
///     Irqs,
///     p.DMA1_CH4,
///     p.DMA1_CH5,
///     Hertz(100_000),
///     Config::default(),
/// );
///
/// match fetch_data(&mut i2c).await {
///     Ok(sensor_data) => {
///         if let Err(e) = validate_checksum(&sensor_data[0..=31]) {
///             hprintln!("Error validating checksum: {}", e);
///             continue;
///         }
///     }
///     Err(e) => hprintln!("Error reading registers: {:?}", e),
/// }
/// ```
pub fn validate_checksum(checksum_bytes: &[u8]) -> Result<(), &'static str> {
    if checksum_bytes.len() < 32 {
        return Err("Could not validate checksum, incorrect number of bytes received");
    }

    // Calculate sum of first 30 bytes as checksum
    let mut calculated_sum: u16 = 0;
    for &byte in checksum_bytes.iter().take(30) {
        calculated_sum = calculated_sum.wrapping_add(byte as u16);
    }
    let received_sum = u16::from_be_bytes([checksum_bytes[30], checksum_bytes[31]]);

    if calculated_sum == received_sum {
        Ok(())
    } else {
        Err("Checksum validation failed")
    }
}

/// Debugging helper function to print all data from
/// the PMSA003I sensor. Simply iterates over all data
/// and prints the register address and corresponding data.
///
/// # Arguments
///
/// * `buffer` - Entire array of u8 data from the sensor
///
/// # Examples
///
/// ```
/// match fetch_data(&mut i2c).await {
///     Ok(sensor_data) => {
///         _print_all_regs(&sensor_data);
///     }
///     Err(e) => hprintln!("Error reading registers: {:?}", e),
/// }
/// ```
pub fn _print_all_regs(buffer: &[u8]) {
    for (i, &value) in buffer.iter().enumerate() {
        hprintln!("Register 0x{:02X}: 0x{:02X}", i, value);
    }
}
