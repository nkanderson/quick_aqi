# Quick AQI

Application to perform a quick measurement of local air quality. This implementation has been tested using Rust running on a STM32F303VCT6 MCU (on the STM32F3DISCOVERY board) with a PMSA003I sensor.

The application provides an Air Quality Index (AQI) calculation based on particulate matter of size 2.5 microns diameter or smaller, referred to as PM2.5. It is intended to allow for basic data gathering on local AQI conditions, which may or may not be well reported in a given area. It provides user feedback in the form of LED output indicating the AQI range of the current measurement, as defined by the EPA, in addition to an exact AQI calculation printed to a serial debug output.

Future iterations will include on-demand calculations using the user button control, as well as output of the exact AQI value to an OLED display. This may allow further extensions, including a display of particle count, which may be useful in a cleanroom context.

**Author**: Niklas Anderson

## Run

### Install Dependencies

**Note**: Installation and basic hardware config uses content from the [Embedded Rust Book](https://docs.rust-embedded.org/book/) as a starting point, along with documentation for the [embassy-stm32](https://crates.io/crates/embassy-stm32) crate. If the steps below do not cover the target installation environment, a good starting place would be the [Embedded Rust Book's "Installation" overview](https://docs.rust-embedded.org/book/intro/install.html).

Add the Cortex-M4F hardware with floating point target:
```sh
$ rustup target add thumbv7em-none-eabihf
```

If running using `probe-rs`, install `probe-rs`:
```sh
$ cargo binstall probe-rs-tool
```

For other install contexts and options, see the [`probe-rs` installation docs](https://probe.rs/docs/getting-started/installation/).

If running in debug context on MacOS, install GDB and OpenOCD using the following:
```sh
$ brew install arm-none-eabi-gdb
$ brew install openocd
```

### Build and Execute Program

With the board connected over USB and the sensor connected to the appropriate pins on the board, probably the simplest way to test the program is by using `probe-rs`.

Build the executable with the desired target:
```sh
$ cargo build --target thumbv7em-none-eabihf
```

Run using `probe-rs run`:
```sh
$ probe-rs run --chip STM32F303VC target/thumbv7em-none-eabihf/debug/quick_aqi
```

### Debugging

It's also possible to run in a more involved debug configuration using `openocd` and `gdb`.

First build the executable with the desired target:
```sh
$ cargo build --target thumbv7em-none-eabihf
```

Next run `openocd`:
```sh
$ openocd -f interface/stlink.cfg -f target/stm32f3x.cfg
```

In a second terminal, run gdb. The command below shows this for `arm-none-eabi-gdb` on macOS:
```sh
$ arm-none-eabi-gdb target/thumbv7em-none-eabihf/debug/quick_aqi
```

#### GDB commands

```gdb
# Connect to process on port 3333
(gdb) target extended-remote :3333
# Enable semihosting for debugging output on host
(gdb) monitor arm semihosting enable
# Set a breakpoint on the main function
(gdb) b main
# Set a breakpoint on line 55 in the main.rs file
(gdb) b main.rs:55
# Continue execution
(gdb) continue
# Go to next
(gdb) next
# Show contents of registers
(gdb) info registers
# Show backtrace
(gdb) bt
# Show specific register contents (in this case, the link register)
(gdb) info reg lr

```

## Example Output

The following shows program execution with additional debugging output, including printout of the contents of each register. The calculated AQI and LED color are output while the user hardware button is pressed.

```sh
➜  quick_aqi git:(main) ✗ probe-rs run --chip STM32F303VC target/thumbv7em-none-eabihf/debug/quick_aqi
      Erasing ✔ 100% [####################]  88.00 KiB @  48.31 KiB/s (took 2s)
  Programming ✔ 100% [####################]  87.00 KiB @  18.98 KiB/s (took 5s)                                                                                          Finished in 6.41s
Attempting to ping device at address 0x12
Device responded to ping
Reading all registers
Header validated successfully
Calculated checksum: 552
Received checksum: 552
Checksum validated successfully
PM2.5 concentration: 3 µg/m³
AQI: 17
Register 0x00: 0x42
Register 0x01: 0x4D
Register 0x02: 0x00
Register 0x03: 0x1C
Register 0x04: 0x00
Register 0x05: 0x00
Register 0x06: 0x00
Register 0x07: 0x03
Register 0x08: 0x00
Register 0x09: 0x03
Register 0x0A: 0x00
Register 0x0B: 0x00
Register 0x0C: 0x00
Register 0x0D: 0x03
Register 0x0E: 0x00
Register 0x0F: 0x03
Register 0x10: 0x00
Register 0x11: 0x84
Register 0x12: 0x00
Register 0x13: 0x2A
Register 0x14: 0x00
Register 0x15: 0x18
Register 0x16: 0x00
Register 0x17: 0x14
Register 0x18: 0x00
Register 0x19: 0x00
Register 0x1A: 0x00
Register 0x1B: 0x00
Register 0x1C: 0x97
Register 0x1D: 0x00
Register 0x1E: 0x02
Register 0x1F: 0x28
Calculated AQI: 17, Color: Green
Calculated AQI: 17, Color: Green
Calculated AQI: 17, Color: Green
Calculated AQI: 17, Color: Green
Calculated AQI: 17, Color: Green
```

The next set of output comes from running the program after extinguishing a candle and allowing the smoke to enter the sensor. This served as very basic end-user testing to ensure the output changed in the expected direction with real-world input.

```sh
➜  quick_aqi git:(main) ✗ probe-rs run --chip STM32F303VC target/thumbv7em-none-eabihf/debug/quick_aqi
      Erasing ✔ 100% [####################]  88.00 KiB @  48.95 KiB/s (took 2s)
  Programming ✔ 100% [####################]  87.00 KiB @  19.13 KiB/s (took 5s)                                                                                          Finished in 6.35s
Attempting to ping device at address 0x12
Device responded to ping
Reading all registers
Header validated successfully
Calculated checksum: 1926
Received checksum: 1926
Checksum validated successfully
PM2.5 concentration: 101 µg/m³
AQI: 183
Register 0x00: 0x42
Register 0x01: 0x4D
Register 0x02: 0x00
Register 0x03: 0x1C
Register 0x04: 0x00
Register 0x05: 0x29
Register 0x06: 0x00
Register 0x07: 0x98
Register 0x08: 0x01
Register 0x09: 0x63
Register 0x0A: 0x00
Register 0x0B: 0x1F
Register 0x0C: 0x00
Register 0x0D: 0x65
Register 0x0E: 0x00
Register 0x0F: 0xEC
Register 0x10: 0x27
Register 0x11: 0xBD
Register 0x12: 0x0C
Register 0x13: 0xF2
Register 0x14: 0x07
Register 0x15: 0xB5
Register 0x16: 0x04
Register 0x17: 0x44
Register 0x18: 0x02
Register 0x19: 0x19
Register 0x1A: 0x01
Register 0x1B: 0xAD
Register 0x1C: 0x97
Register 0x1D: 0x00
Register 0x1E: 0x07
Register 0x1F: 0x86
Calculated AQI: 183, Color: Red
Calculated AQI: 183, Color: Red
Calculated AQI: 183, Color: Red
```

## Resources and References

[Embassy book - starting a new project](https://embassy.dev/book/#_starting_a_new_project)  
Examples of configuration and project initialization tooling

[Adafruit PMSA003I Overview](https://learn.adafruit.com/pmsa003i/overview)  
Provides details on the sensor and the Adafruit breakout board.

[Adafruit forum - post by StanJ](https://forums.adafruit.com/viewtopic.php?f=48&p=767725&t=136528#p767725)  
Explains differences between the types of data provided by the sensor.

[EPA documentation - AQI technical details](https://document.airnow.gov/technical-assistance-document-for-the-reporting-of-daily-air-quailty.pdf)  
Provides a technical explanation of the Air Quality Index measurements, including equations for calculation.

[EPA AQI updates 2024](https://www.epa.gov/system/files/documents/2024-02/pm-naaqs-air-quality-index-fact-sheet.pdf)  
Includes updated ranges for calculating AQI using PM2.5.
