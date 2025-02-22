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

// TODO: Make a struct for the sensor data, maybe one for the
// sensor as a whole

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
    match i2c.blocking_write_read(SENSOR_I2C_ADDR, &[0x00], &mut buffer) {
        Ok(()) => {
            // 1. Validate data from addresses 0x0 and 0x1 match 0x42 and 0x4d, the hardcoded header
            if buffer[0..2] != EXPECTED_HEADER {
                hprintln!(
                    "Warning: Invalid header! Got 0x{:02X}{:02X}, expected 0x{:02X}{:02X}",
                    buffer[0],
                    buffer[1],
                    EXPECTED_HEADER[0],
                    EXPECTED_HEADER[1]
                );
            } else {
                hprintln!("Header validated successfully");

                // 2. Calculate sum of first 30 bytes as checksum
                let mut calculated_sum: u16 = 0;
                for i in 0..30 {
                    calculated_sum = calculated_sum.wrapping_add(buffer[i] as u16);
                }
                let received_sum = ((buffer[30] as u16) << 8) | buffer[31] as u16;

                hprintln!("Calculated checksum: {}", calculated_sum);
                hprintln!("Received checksum: {}", received_sum);

                if calculated_sum != received_sum {
                    hprintln!("Warning: Checksum mismatch!");
                } else {
                    hprintln!("Checksum validated successfully");

                    // 3. Parse big endian data
                    // Extract PM2.5 concentration (example - adjust based on your sensor's data format)
                    let pm25_concentration = ((buffer[6] as u16) << 8) | buffer[7] as u16;

                    // Convert concentration to AQI
                    let aqi = calculate_aqi(pm25_concentration);
                    hprintln!("PM2.5 concentration: {} µg/m³", pm25_concentration);
                    hprintln!("AQI: {}", aqi);

                    // Print all register values for debugging
                    for (i, &value) in buffer.iter().enumerate() {
                        hprintln!("Register 0x{:02X}: 0x{:02X}", i, value);
                    }
                }
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
fn calculate_aqi(pm25: u16) -> u16 {
    // AQI breakpoints for PM2.5
    const BREAKPOINTS: [(u16, u16, u16, u16); 7] = [
        (0, 12, 0, 50),       // Good
        (13, 35, 51, 100),    // Moderate
        (36, 55, 101, 150),   // Unhealthy for Sensitive Groups
        (56, 150, 151, 200),  // Unhealthy
        (151, 250, 201, 300), // Very Unhealthy
        (251, 350, 301, 400), // Hazardous
        (351, 500, 401, 500), // Very Hazardous
    ];

    // Find the appropriate breakpoint range
    for (pm_low, pm_high, aqi_low, aqi_high) in BREAKPOINTS {
        if pm25 >= pm_low && pm25 <= pm_high {
            // Linear interpolation formula:
            // AQI = ((AQIhigh - AQIlow) / (Conchigh - Conclow)) * (Conc - Conclow) + AQIlow
            // Translated from https://github.com/adafruit/Adafruit_PM25AQI/blob/master/Adafruit_PM25AQI.cpp
            // TODO: May want to find additional sources for this formula
            return ((aqi_high - aqi_low) * (pm25 - pm_low)) / (pm_high - pm_low) + aqi_low;
        }
    }

    // If PM2.5 is above 500, return the maximum AQI value
    500
}
