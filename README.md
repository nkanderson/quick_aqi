# Quick AQI

Application to perform a quick measurement of local air quality. This implementation has been tested using Rust running on a STM32F303VCT6 MCU (on the STM32F3DISCOVERY board) with a PMSA003I sensor.

## Run

### Install Dependencies

**Note**: Installation and basic hardware config uses content from the [Embedded Rust Book](https://docs.rust-embedded.org/book/) as a starting point.

Add the Cortex-M4F hardware with floating point target:
```sh
$ rustup target add thumbv7em-none-eabihf
```

On MacOS, install GDB nad OpenOCD using the following:
```sh
$ brew install arm-none-eabi-gdb
$ brew install openocd
```
