#![no_std]
#![no_main]

use defmt_rtt as _;
use defmt_test as _;
use embassy_stm32 as _;
use panic_probe as _;

use quick_aqi as _;
use quick_aqi::pmsa003i::validate_checksum;

#[defmt_test::tests]
mod tests {
    use super::*;
    use defmt::assert_eq;

    #[test]
    fn test_validate_checksum() {
        //
        // Test validation success
        //
        let mut data = [0u8; 32];
        // Fill first 30 bytes with 1s
        data[..30].copy_from_slice(&[1; 30]);
        // Calculate correct checksum
        let checksum: u16 = data[..30].iter().map(|&b| b as u16).sum();
        // Store it in big-endian format
        data[30..32].copy_from_slice(&checksum.to_be_bytes());

        assert_eq!(validate_checksum(&data), Ok(()));

        //
        // Test validation failure
        //
        // Incorrect checksum
        data[30..32].copy_from_slice(&[0x00, 0x00]);

        assert_eq!(validate_checksum(&data), Err("Checksum validation failed"));

        //
        // Incorrect number of bytes
        //
        // Less than 32 bytes
        let data = [0u8; 31];

        assert_eq!(
            validate_checksum(&data),
            Err("Could not validate checksum, incorrect number of bytes received")
        );
    }
}
