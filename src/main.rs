#![no_std]
#![no_main]

use cortex_m_rt::entry;
use embassy_stm32::gpio::{Input, Level, Output, Pull, Speed};
use {defmt_rtt as _, panic_probe as _};

use cortex_m_semihosting::hprintln;
use embassy_stm32::i2c::{Config, I2c};
use embassy_stm32::time::Hertz;
use {defmt_rtt as _, panic_probe as _};

const SENSOR_I2C_ADDR: u8 = 0x12;
const EXPECTED_HEADER: [u8; 2] = [0x42, 0x4D];
const TOTAL_REGISTERS: usize = 32;

#[derive(Debug, Default)]
pub struct Pmsa003iData {
    // CF is "Calibration Factory", and generally not useful for our needs.
    _pm1_0_standard: u16, // PM1.0 concentration unit μ g/m3（CF=1，standard particle）
    _pm2_5_standard: u16, // PM2.5 concentration unit μ g/m3（CF=1，standard particle）
    _pm10_standard: u16,  // PM10 concentration unit μ g/m3（CF=1，standard particle）

    // The environmental units take into account factors like ambient pressure.
    // This is typically what is used in an AQI report or forecast.
    pm1_0_env: u16, // PM1.0 concentration unit μ g/m3（environmental units）
    pm2_5_env: u16, // PM2.5 concentration unit μ g/m3（environmental units）
    pm10_env: u16,  // PM10 concentration unit μ g/m3  (environmental units)

    // The particle count per volume of air is often used in a cleanroom context.
    particles_0_3: u16, // Number of particles with diameter beyond 0.3 um in 0.1L of air
    particles_0_5: u16, // Number of particles with diameter beyond 0.5 um in 0.1L of air
    particles_1_0: u16, // Number of particles with diameter beyond 1.0 um in 0.1L of air
    particles_2_5: u16, // Number of particles with diameter beyond 2.5 um in 0.1L of air
    particles_5_0: u16, // Number of particles with diameter beyond 5.0 um in 0.1L of air
    particles_10: u16,  // Number of particles with diameter beyond 10 um in 0.1L of air
}

#[entry]
fn main() -> ! {
    let p = embassy_stm32::init(Default::default());
    let mut led = Output::new(p.PE9, Level::High, Speed::Low);
    let button = Input::new(p.PA0, Pull::Down);

    // Assign I2C pins
    let scl = p.PA9;
    let sda = p.PA10;

    // Initialize I2C2 with 100kHz speed
    // TODO? May want to use 400kHz
    let mut i2c = I2c::new_blocking(p.I2C2, scl, sda, Hertz(100_000), Config::default());

    // Ping check the device
    hprintln!(
        "Attempting to ping device at address 0x{:02X}",
        SENSOR_I2C_ADDR
    );
    match i2c.blocking_write(SENSOR_I2C_ADDR, &[0x00]) {
        Ok(()) => hprintln!("Device responded to ping"),
        Err(e) => hprintln!("Device did not respond to ping: {:?}", e),
    }

    // Get all data from sensor (addresses 0x00 through 0x1f, 32 total)
    let mut buffer = [0u8; TOTAL_REGISTERS];
    hprintln!("Reading all registers");
    // TODO: These match arms are getting difficult to read, will need to refactor
    match i2c.blocking_write_read(SENSOR_I2C_ADDR, &[0x00], &mut buffer) {
        Ok(()) => {
            // 1. Validate data from addresses 0x0 and 0x1 match 0x42 and 0x4d, the hardcoded header
            match validate_header(&buffer[0..2]) {
                Ok(_) => {
                    hprintln!("Header validated successfully");

                    // 2. Calculate sum of first 30 bytes as checksum
                    let mut calculated_sum: u16 = 0;
                    for i in 0..30 {
                        calculated_sum = calculated_sum.wrapping_add(buffer[i] as u16);
                    }
                    let received_sum = u16::from_be_bytes([buffer[30], buffer[31]]);

                    hprintln!("Calculated checksum: {}", calculated_sum);
                    hprintln!("Received checksum: {}", received_sum);

                    if calculated_sum != received_sum {
                        hprintln!("Warning: Checksum mismatch!");
                    } else {
                        hprintln!("Checksum validated successfully");

                        // 3. Parse big endian data
                        let data = parse_data(&mut buffer).unwrap_or_else(|err| {
                            hprintln!("Error parsing data: {}", err);
                            Pmsa003iData::default()
                        });

                        // Extract PM2.5 concentration
                        let pm25_concentration = data.pm2_5_env;

                        // Convert concentration to AQI
                        let aqi = calculate_aqi(pm25_concentration as f32);
                        hprintln!("PM2.5 concentration: {} µg/m³", pm25_concentration);
                        hprintln!("AQI: {}", aqi);

                        // Print all register values for debugging
                        for (i, &value) in buffer.iter().enumerate() {
                            hprintln!("Register 0x{:02X}: 0x{:02X}", i, value);
                        }
                    }
                }
                Err(_err) => hprintln!(
                    "Warning: Invalid header! Got 0x{:02X}{:02X}, expected 0x{:02X}{:02X}",
                    buffer[0],
                    buffer[1],
                    EXPECTED_HEADER[0],
                    EXPECTED_HEADER[1]
                ),
            }
        }
        Err(e) => hprintln!("Error reading registers: {:?}", e),
    }

    // Reset I2C bus with a stop condition
    // i2c.blocking_write(SENSOR_I2C_ADDR, &[]).ok();

    loop {
        if button.is_high() {
            led.set_high();
        } else {
            led.set_low();
        }
    }
}

// TODO: Add tests - need to determine how best to do that
// in this file with `nostd` (or move this function to another file?)
fn calculate_aqi(pm25: f32) -> u16 {
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

fn parse_data(buffer: &mut [u8]) -> Result<Pmsa003iData, &'static str> {
    if buffer.len() < 32 {
        return Err("Buffer too short, expected at least 32 bytes");
    }

    Ok(Pmsa003iData {
        _pm1_0_standard: u16::from_be_bytes([buffer[4], buffer[5]]),
        _pm2_5_standard: u16::from_be_bytes([buffer[6], buffer[7]]),
        _pm10_standard: u16::from_be_bytes([buffer[8], buffer[9]]),
        pm1_0_env: u16::from_be_bytes([buffer[10], buffer[11]]),
        pm2_5_env: u16::from_be_bytes([buffer[12], buffer[13]]),
        pm10_env: u16::from_be_bytes([buffer[14], buffer[15]]),
        particles_0_3: u16::from_be_bytes([buffer[16], buffer[17]]),
        particles_0_5: u16::from_be_bytes([buffer[18], buffer[19]]),
        particles_1_0: u16::from_be_bytes([buffer[20], buffer[21]]),
        particles_2_5: u16::from_be_bytes([buffer[22], buffer[23]]),
        particles_5_0: u16::from_be_bytes([buffer[24], buffer[25]]),
        particles_10: u16::from_be_bytes([buffer[26], buffer[27]]),
    })
}

fn validate_header(header_bytes: &[u8]) -> Result<(), &'static str> {
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
