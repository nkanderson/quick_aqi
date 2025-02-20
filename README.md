# Quick AQI

Application to perform a quick measurement of local air quality. This implementation has been tested using Rust running on a STM32F303VCT6 MCU (on the STM32F3DISCOVERY board) with a PMSA003I sensor.

## Run

### Install Dependencies

**Note**: Installation and basic hardware config uses content from the [Embedded Rust Book](https://docs.rust-embedded.org/book/) as a starting point, along with documentation for the [embassy-stm32](https://crates.io/crates/embassy-stm32) crate.

Add the Cortex-M4F hardware with floating point target:
```sh
$ rustup target add thumbv7em-none-eabihf
```

On MacOS, install GDB and OpenOCD using the following:
```sh
$ brew install arm-none-eabi-gdb
$ brew install openocd
```

### Debugging
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

## Resources and References

[Embassy book - starting a new project](https://embassy.dev/book/#_starting_a_new_project)
Examples of configuration and project initialization tooling
