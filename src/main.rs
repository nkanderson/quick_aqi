#![no_std]
#![no_main]

use cortex_m_rt::entry;
use embassy_stm32::gpio::{Input, Level, Output, Pull, Speed};
use {defmt_rtt as _, panic_probe as _};

use cortex_m_semihosting::hprintln;
use embassy_stm32::i2c::{Config, I2c};
use embassy_stm32::time::Hertz;
use {defmt_rtt as _, panic_probe as _};

static SENSOR_I2C_ADDR: u8 = 0x12;

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

    // Testing blocking reads
    // Read from registers 0x00 and 0x01, confirm they match expected
    // output of 0x42 and 0x4d
    let mut buffer = [0u8; 2];
    hprintln!("Reading registers 0x00 and 0x01");
    match i2c.blocking_write_read(SENSOR_I2C_ADDR, &[0x00], &mut buffer) {
        Ok(()) => {
            hprintln!("Register 0x00 value: 0x{:02X} (expected 0x42)", buffer[0]);
            if buffer[0] != 0x42 {
                hprintln!("Warning: Unexpected value for register 0x00");
            }

            hprintln!("Register 0x01 value: 0x{:02X} (expected 0x4D)", buffer[1]);
            if buffer[1] != 0x4D {
                hprintln!("Warning: Unexpected value for register 0x01");
            }
        }
        Err(e) => hprintln!("Error reading registers: {:?}", e),
    }

    // Reset I2C bus with a stop condition
    // i2c.blocking_write(SENSOR_I2C_ADDR, &[]).ok();

    // Steps to get and process data
    // Get all data from sensor (addresses 0x00 through 0x1f, 32 total)
    // Validate data from addresses 0x0 and 0x1 match 0x42 and 0x4d, respectively
    // - This is a hardcoded header
    // Calculate checksum
    // - I think just the sum of the first 30 addresses
    // - Maybe compare this to last 2 addresses (the Adafruit lib is a little confusing on this...)
    // Data is in big endian, make sure it's parsed and stored correctly
    // Convert concentration to AQI

    loop {
        if button.is_high() {
            led.set_high();
        } else {
            led.set_low();
        }
    }
}
