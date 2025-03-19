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

With the board connected over USB and the sensor connected to the appropriate pins on the board, the provided configuration in `.cargo/config.toml` should allow for simply running `cargo build` and `cargo run` (if running a debug build is acceptable):
```sh
$ cargo build && cargo run
```

The required commands may instead be run with the target specified for the build, and the executable run using `probe-rs`:

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

Some helpful GDB commands are shown below.

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

The sections below contain output from end-user testing of the application functionality. In these cases, the baseline measurements were taken from a workstation in a home office. The elevated AQI readings were triggered using a blown-out candle, which emitted smoke that was captured by the sensor.

### Standard Output

The following shows program execution with the current standard output. It includes an initial ping check to ensure the device is connected. The output following shows individual AQI results using on-demand data collection using the Discovery board's user hardware button. In addition, the on-board LEDs are lit to reflect the current AQI range.

```sh
➜  quick_aqi git:(main) ✗ cargo build && cargo run
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.16s
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.08s
     Running `probe-rs run --chip STM32F303VCTx target/thumbv7em-none-eabihf/debug/quick_aqi`
      Erasing ✔ 100% [####################] 116.00 KiB @  46.17 KiB/s (took 3s)
  Programming ✔ 100% [####################] 116.00 KiB @  19.12 KiB/s (took 6s)                                                                                          Finished in 8.58s
Attempting to ping device at address 0x12
Device responded to ping
PM2.5 concentration: 41 µg/m³
Calculated AQI: 115, Color: Orange

PM2.5 concentration: 33 µg/m³
Calculated AQI: 96, Color: Yellow

PM2.5 concentration: 18 µg/m³
Calculated AQI: 68, Color: Yellow

PM2.5 concentration: 12 µg/m³
Calculated AQI: 56, Color: Yellow

PM2.5 concentration: 7 µg/m³
Calculated AQI: 39, Color: Green

PM2.5 concentration: 5 µg/m³
Calculated AQI: 28, Color: Green
```

### Full Debug Output

The following shows program execution with additional debugging output using the `pmsa003i::_print_all_regs` function, which includes a printout of the contents of each register. The calculated AQI and LED color are output while the user hardware button is pressed.

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
```

## Tests

A number of testing challenges arose, which are covered in more detail below. It is possible to run some basic tests for the AQI library using the following command (replace the target triple with the current host machine):

```sh
cargo test -p aqi --target aarch64-apple-darwin --lib
```

This runs the tests for only the `aqi` package, and only runs the library tests, which excludes Rustdoc examples. The target triple above works on an Apple silicon device. An alternative value for a 64-bit Linux machine would be `x86_64-unknown-linux-gnu`.


## Challenges and Successes

As with many embedded projects, the biggest challenges were at the beginning, in basic project setup and initial I2C communication with the sensor. I attempted to start by using blocking functions in order to keep the code within my current understanding of Rust. It's clear that Embassy is built for use with `async..await` though, so the initial project setup was maybe more difficult than when using non-blocking functions, but probably more understandable.

The sensor datasheet is somewhat sparse on details, so it took a few attempts to get I2C communication configured correctly using Embassy's blocking I2C functions. Once I determined it was preferable to read all registers at once, it became straightforward.

Related to the above, I did attempt a blocking version of the button-triggered data collection, but abandoned it as it was overly complex and not fully functional. Additionally, it was more difficult than expected to create a proper function signature for a data-fetching function that accepted an I2C instance. I wasn't able to come up with a working function using a blocking I2C instance, but was successful using a non-blocking instance.

On the plus side, the changes necessary to move from blocking to non-blocking were much simpler than expected. Changes were fairly minimal, and are seen in [PR #8](https://github.com/nkanderson/quick_aqi/pull/8). The provided Embassy documentation and examples were essential in understanding how to make these small changes.

The last challenge worth mentioning is the embedded testing environment. Since this is a `no_std` context, it's not possible to use the simple `#[test]` macro for unit tests. I've tried to get something set up using `defmt-test`, but have not yet had success. Ongoing work is present in the `add_testing` branch.

## Limitations and Future Improvements

### LED Output

The current application uses on-board LEDs from the Discovery board to indicate AQI range. These LEDs have limited output, and there is not a one-to-one mapping between the colors available and the EPA-specified AQI range colors.

Instead of using the on-board LEDs, it would be preferable to use a single RGB LED and drive it with the necessary combination of RGB values required to produce a specific color.

### Text Output

The application output the capture PM2.5 value from the sensor along with the calculated AQI to serial output on the host machine. It would be preferable for the application to output this data to an OLED screen connected over I2C.

It seems likely that connecting the OLED along with the sensor over I2C would make it necessary to use synchronization patterns from Embassy. Specifically, the Embassy book contains a section on ["sharing peripherals between tasks"](https://embassy.dev/book/#_sharing_peripherals_between_tasks) which would likely be helpful.

### Hardware Button

If the eventual goal is to move away from using a prototyping board like the Discovery, it will be necessary to use a separate hardware button as well. This change would probably be done within a larger set of changes moving away from using the board. In switching from using the Discovery board to a more custom board with the same (or similar) MCU, it will also become necessary to re-map the pins for the I2C configuration (assuming the pin re-mapping for LEDs has taken place with the above switch to a single RGB LED).

### Test Environment

As noted above, creating a testing environment in the `no_std` context is non-trivial. A future improvement would be to have a fully functional environment for running basic unit tests.

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
