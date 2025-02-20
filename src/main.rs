#![no_std]
#![no_main]

use cortex_m_rt::entry;
use embassy_stm32::gpio::{Input, Level, Output, Pull, Speed};
use {defmt_rtt as _, panic_probe as _};

use embassy_stm32::i2c::{Config, I2c};
use embassy_stm32::time::Hertz;

use cortex_m_semihosting::hprintln;

static SENSOR_I2C_ADDR: u8 = 0x12;

#[entry]
fn main() -> ! {
    let p = embassy_stm32::init(Default::default());
    let mut led = Output::new(p.PE9, Level::High, Speed::Low);
    let button = Input::new(p.PA0, Pull::Down);

    // Assign I2C pins
    let scl = p.PA9;
    let sda = p.PA10;

    // Initialize I2C2 with 400kHz speed
    let mut i2c = I2c::new_blocking(p.I2C2, scl, sda, Hertz(400_000), Config::default());

    // Blocking read
    let mut buffer = [0u8; 1];
    // Read 1 byte from register 0xD0 (Device ID register)
    let register = [0xD0];
    i2c.blocking_write(SENSOR_I2C_ADDR, &register).unwrap();
    i2c.blocking_read(SENSOR_I2C_ADDR, &mut buffer).unwrap();

    hprintln!("register: {:?}", register);

    for addr in 0x00..=0x7F {
        let mut buffer = [0u8; 1];
        if i2c.blocking_read(addr, &mut buffer).is_ok() {
            hprintln!("Found device at address: 0x{:02X}", addr);
        }
    }

    loop {
        if button.is_high() {
            led.set_high();
        } else {
            led.set_low();
        }
    }
}
