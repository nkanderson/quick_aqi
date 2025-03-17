#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_stm32::exti::ExtiInput;
use embassy_stm32::gpio::{Level, Output, Pull, Speed};
use embassy_stm32::peripherals::{PE10, PE11, PE12, PE13, PE14, PE15, PE8, PE9};
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

// Color enum to map AQI value to LED color
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Color {
    Green,
    Yellow,
    Orange,
    Red,
    Purple,
    DarkPurple,
}

// LED struct used to map names to pins
pub struct LedController {
    // The STM32F303 Discovery has LEDs on these pins:
    // PE8 (blue), PE9 (red), PE10 (orange), PE11 (green),
    // PE12 (blue), PE13 (red), PE14 (orange), PE15 (green)
    led_blue1: Output<'static>,
    led_red1: Output<'static>,
    led_orange1: Output<'static>,
    led_green1: Output<'static>,
    led_blue2: Output<'static>,
    led_red2: Output<'static>,
    led_orange2: Output<'static>,
    led_green2: Output<'static>,
}

impl LedController {
    pub fn new(
        pe8: PE8,
        pe9: PE9,
        pe10: PE10,
        pe11: PE11,
        pe12: PE12,
        pe13: PE13,
        pe14: PE14,
        pe15: PE15,
    ) -> Self {
        Self {
            led_blue1: Output::new(pe8, Level::Low, Speed::Low),
            led_red1: Output::new(pe9, Level::Low, Speed::Low),
            led_orange1: Output::new(pe10, Level::Low, Speed::Low),
            led_green1: Output::new(pe11, Level::Low, Speed::Low),
            led_blue2: Output::new(pe12, Level::Low, Speed::Low),
            led_red2: Output::new(pe13, Level::Low, Speed::Low),
            led_orange2: Output::new(pe14, Level::Low, Speed::Low),
            led_green2: Output::new(pe15, Level::Low, Speed::Low),
        }
    }

    pub fn set_color(&mut self, color: Color) {
        // Turn off all LEDs first
        self.all_off();

        // Set LEDs matching on color.
        // This is an approximation, since the Discovery board
        // doesn't not have individual LEDs with the exact colors
        // needed.
        match color {
            Color::Green => {
                self.led_green1.set_high();
                self.led_green2.set_high();
            }
            Color::Yellow => {
                self.led_green1.set_high();
                self.led_orange1.set_high();
            }
            Color::Orange => {
                self.led_orange1.set_high();
                self.led_orange2.set_high();
            }
            Color::Red => {
                self.led_red1.set_high();
                self.led_red2.set_high();
            }
            Color::Purple => {
                self.led_red1.set_high();
                self.led_blue1.set_high();
            }
            Color::DarkPurple => {
                self.led_red2.set_high();
                self.led_blue1.set_high();
                self.led_blue2.set_high();
            }
        }
    }

    fn all_off(&mut self) {
        self.led_orange1.set_low();
        self.led_green1.set_low();
        self.led_red1.set_low();
        self.led_blue1.set_low();
        self.led_orange2.set_low();
        self.led_green2.set_low();
        self.led_red2.set_low();
        self.led_blue2.set_low();
    }
}

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_stm32::init(Default::default());
    let mut button = ExtiInput::new(p.PA0, p.EXTI0, Pull::Down);

    // Assign I2C pins
    let scl = p.PA9;
    let sda = p.PA10;

    // Initialize I2C2 with 100kHz speed
    // TODO? May want to use 400kHz
    let mut i2c = I2c::new_blocking(p.I2C2, scl, sda, Hertz(100_000), Config::default());

    // Create our LED controller
    let mut led_controller =
        LedController::new(p.PE8, p.PE9, p.PE10, p.PE11, p.PE12, p.PE13, p.PE14, p.PE15);

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
    let mut aqi = 0;
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
                        aqi = calculate_aqi(pm25_concentration as f32);
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

    loop {
        button.wait_for_any_edge().await;
        if button.is_high() {
            match i2c.blocking_write_read(SENSOR_I2C_ADDR, &[0x00], &mut buffer) {
                Ok(()) => {
                    // Validate data from addresses 0x0 and 0x1 match 0x42 and 0x4d, the hardcoded header
                    match validate_header(&buffer[0..2]) {
                        Ok(_) => {
                            // Calculate sum of first 30 bytes as checksum
                            let mut calculated_sum: u16 = 0;
                            for i in 0..30 {
                                calculated_sum = calculated_sum.wrapping_add(buffer[i] as u16);
                            }
                            let received_sum = u16::from_be_bytes([buffer[30], buffer[31]]);

                            if calculated_sum != received_sum {
                                hprintln!("Warning: Checksum mismatch!");
                            } else {
                                // Parse big endian data
                                let data = parse_data(&mut buffer).unwrap_or_else(|err| {
                                    hprintln!("Error parsing data: {}", err);
                                    Pmsa003iData::default()
                                });

                                // Extract PM2.5 concentration
                                let pm25_concentration = data.pm2_5_env;

                                // Convert concentration to AQI
                                aqi = calculate_aqi(pm25_concentration as f32);
                                hprintln!("PM2.5 concentration: {} µg/m³", pm25_concentration);
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
            // Get color name from AQI value
            let color = get_aqi_color(aqi);

            // Set the LED color
            led_controller.set_color(color);

            hprintln!("Calculated AQI: {}, Color: {:?}", aqi, color);
            // Newline to separate output between readings
            hprintln!("");
        } else {
            led_controller.all_off();
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

// AQI mapping to LED color
fn get_aqi_color(value: u16) -> Color {
    match value {
        0..=50 => Color::Green,
        51..=100 => Color::Yellow,
        101..=150 => Color::Orange,
        151..=200 => Color::Red,
        201..=300 => Color::Purple,
        _ => Color::DarkPurple,
    }
}
