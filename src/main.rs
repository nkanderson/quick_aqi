//! Quick AQI
//!
//! Application to perform a quick measurement of local air quality.
//! This implementation has been tested on a STM32F303VCT6 MCU
//! (on the STM32F3DISCOVERY board) with a PMSA003I sensor.
//!
//! The application provides an Air Quality Index (AQI) calculation based
//! on particulate matter of size 2.5 microns diameter or smaller, referred
//! to as PM2.5.
//!
//! It is intended to allow for basic data gathering on local AQI conditions,
//! which may or may not be well reported in a given area. It provides user
//! feedback in the form of LED output indicating the AQI range of the current
//! measurement, as defined by the EPA, in addition to an exact AQI calculation
//! printed to a serial debug output. Individual AQI measurements may be
//! triggered by pressing the onboard user button on the Discovery board.
//!
//!
//! # Examples
//!
//! ```sh
//! $ cargo build && cargo run
//! Attempting to ping device at address 0x12
//! Device responded to ping
//! PM2.5 concentration: 41 µg/m³
//! Calculated AQI: 115, Color: Orange
//!
//! PM2.5 concentration: 33 µg/m³
//! Calculated AQI: 96, Color: Yellow
//! ```

#![no_std]
#![no_main]

mod aqi;
mod pmsa003i;

use crate::aqi::Color;
use crate::pmsa003i::Pmsa003iData;
use cortex_m_semihosting::hprintln;
use embassy_executor::Spawner;
use embassy_stm32::bind_interrupts;
use embassy_stm32::exti::ExtiInput;
use embassy_stm32::gpio::{Level, Output, Pull, Speed};
use embassy_stm32::i2c::{Config, I2c};
use embassy_stm32::peripherals::{I2C2, PE10, PE11, PE12, PE13, PE14, PE15, PE8, PE9};
use embassy_stm32::time::Hertz;
use {defmt_rtt as _, panic_probe as _};

/// The LedController struct maps human-readable LED
/// names to their corresponding pin name for the
/// STM32F303 Discovery board.
pub struct LedController {
    // STM32F303 Discovery LED pins and their colors:
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
    /// Initialize the target board LEDs as GPIO output.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut led_controller = LedController::new(p.PE8, p.PE9, p.PE10, p.PE11, p.PE12, p.PE13, p.PE14, p.PE15);
    /// led_controller.set_color(color);
    /// ```
    #[allow(clippy::too_many_arguments)]
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

    /// Turn on desired LEDs based on the specified Color value.
    ///
    /// The Discovery board does not have LEDs with colors directly
    /// matching the EPA AQI ranges, so some approximations are made.
    /// For example, the represent a dark purple color, both blue LEDS
    /// along with a red LED are set high.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut led_controller = LedController::new(p.PE8, p.PE9, p.PE10, p.PE11, p.PE12, p.PE13, p.PE14, p.PE15);
    /// led_controller.set_color(Color::Orange);
    /// ```
    pub fn set_color(&mut self, color: Color) {
        // Turn off all LEDs first
        self.all_off();

        // Set LEDs matching on color
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

    /// Turn off all LEDs. This is used as a reset prior
    /// to setting desired LEDs high in set_color above.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut led_controller = LedController::new(p.PE8, p.PE9, p.PE10, p.PE11, p.PE12, p.PE13, p.PE14, p.PE15);
    /// led_controller.all_off();
    /// ```
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

// Embassy macro to bind interrupts to handlers.
// In this case, we're binding both event and error interrupts.
bind_interrupts!(struct Irqs {
    I2C2_EV => embassy_stm32::i2c::EventInterruptHandler<I2C2>;
    I2C2_ER => embassy_stm32::i2c::ErrorInterruptHandler<I2C2>;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_stm32::init(Default::default());
    let mut button = ExtiInput::new(p.PA0, p.EXTI0, Pull::Down);

    // Assign I2C pins
    let scl = p.PA9;
    let sda = p.PA10;

    // Initialize I2C2 with 100kHz speed
    let mut i2c = I2c::new(
        p.I2C2,
        scl,
        sda,
        Irqs,
        p.DMA1_CH4,
        p.DMA1_CH5,
        Hertz(100_000),
        Config::default(),
    );

    // Create our LED controller
    let mut led_controller =
        LedController::new(p.PE8, p.PE9, p.PE10, p.PE11, p.PE12, p.PE13, p.PE14, p.PE15);

    // Ping check the device
    hprintln!(
        "Attempting to ping device at address 0x{:02X}",
        pmsa003i::SENSOR_I2C_ADDR
    );

    match i2c.write(pmsa003i::SENSOR_I2C_ADDR, &[0x00]).await {
        Ok(()) => hprintln!("Device responded to ping"),
        Err(e) => hprintln!("Device did not respond to ping: {:?}", e),
    }

    let mut aqi = 0;

    loop {
        button.wait_for_any_edge().await;
        if button.is_high() {
            match pmsa003i::fetch_data(&mut i2c).await {
                Ok(sensor_data) => {
                    // If validations fail, skip data parsing and try again on the next iteration
                    if let Err(e) = pmsa003i::validate_header(&sensor_data[0..2]) {
                        hprintln!("Error validating header: {}", e);
                        continue;
                    }
                    if let Err(e) = pmsa003i::validate_checksum(&sensor_data[0..=31]) {
                        hprintln!("Error validating checksum: {}", e);
                        continue;
                    }

                    // Parse data
                    let data = pmsa003i::parse_data(&sensor_data).unwrap_or_else(|err| {
                        hprintln!("Error parsing data: {}", err);
                        Pmsa003iData::default()
                    });

                    // Get PM2.5 concentration
                    let pm25_concentration = data.pm2_5_env;

                    // Convert concentration to AQI
                    aqi = aqi::calculate_aqi(pm25_concentration as f32);
                    hprintln!("PM2.5 concentration: {} µg/m³", pm25_concentration);
                }
                Err(e) => hprintln!("Error reading registers: {:?}", e),
            }
            // Get color name from AQI value
            let color = aqi::get_aqi_color(aqi);

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
