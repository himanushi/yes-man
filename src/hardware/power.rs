//! AXP2101 Power Management IC driver

use esp_idf_hal::i2c::I2cDriver;
use crate::config::i2c::axp2101;
use crate::error::YesManError;

/// AXP2101 Power Management IC operations
pub struct Axp2101;

impl Axp2101 {
    /// Initialize the AXP2101
    pub fn init(i2c: &mut I2cDriver) -> Result<(), YesManError> {
        log::info!("Initializing AXP2101 at 0x{:02X}...", axp2101::ADDR);

        // Verify chip ID
        let chip_id = Self::read_register(i2c, axp2101::reg::CHIP_ID)?;
        log::info!("AXP2101 chip ID: 0x{:02X} (expected: 0x{:02X})", chip_id, axp2101::EXPECTED_CHIP_ID);

        Ok(())
    }

    /// Enable backlight
    pub fn enable_backlight(i2c: &mut I2cDriver) -> Result<(), YesManError> {
        log::info!("Enabling backlight...");

        // Read current DLDO enable state
        let current = Self::read_register(i2c, axp2101::reg::DLDO_ENABLE)?;

        // Enable DLDO1 (bit 7)
        Self::write_register(i2c, axp2101::reg::DLDO_ENABLE, current | axp2101::DLDO1_ENABLE_BIT)?;

        // Set voltage to max (3.3V)
        Self::write_register(i2c, axp2101::reg::DLDO1_VOLTAGE, axp2101::voltage::V3_3)?;

        log::info!("Backlight enabled");
        Ok(())
    }

    /// Set brightness level (0-100)
    pub fn set_brightness(i2c: &mut I2cDriver, level: u8) -> Result<(), YesManError> {
        let voltage = Self::brightness_to_voltage(level);
        Self::write_register(i2c, axp2101::reg::DLDO1_VOLTAGE, voltage)?;
        log::debug!("Backlight brightness set to {}% (0x{:02X})", level, voltage);
        Ok(())
    }

    /// Disable backlight
    pub fn disable_backlight(i2c: &mut I2cDriver) -> Result<(), YesManError> {
        let current = Self::read_register(i2c, axp2101::reg::DLDO_ENABLE)?;
        Self::write_register(i2c, axp2101::reg::DLDO_ENABLE, current & !axp2101::DLDO1_ENABLE_BIT)?;
        log::info!("Backlight disabled");
        Ok(())
    }

    /// Read a register
    fn read_register(i2c: &mut I2cDriver, reg: u8) -> Result<u8, YesManError> {
        let mut buf = [0u8; 1];
        i2c.write_read(axp2101::ADDR, &[reg], &mut buf, 100)
            .map_err(|e| YesManError::I2c(format!("AXP2101 read error: {:?}", e)))?;
        Ok(buf[0])
    }

    /// Write a register
    fn write_register(i2c: &mut I2cDriver, reg: u8, value: u8) -> Result<(), YesManError> {
        i2c.write(axp2101::ADDR, &[reg, value], 100)
            .map_err(|e| YesManError::I2c(format!("AXP2101 write error: {:?}", e)))?;
        Ok(())
    }

    /// Convert brightness (0-100) to voltage register value
    fn brightness_to_voltage(brightness: u8) -> u8 {
        // Map 0-100 to voltage range
        // 0x00 = 500mV, 0x1C = 3300mV
        let clamped = brightness.min(100) as u32;
        ((clamped * 0x1C) / 100) as u8
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_brightness_to_voltage() {
        assert_eq!(Axp2101::brightness_to_voltage(0), 0x00);
        assert_eq!(Axp2101::brightness_to_voltage(100), 0x1C);
        assert_eq!(Axp2101::brightness_to_voltage(50), 0x0E);
        // Test clamping
        assert_eq!(Axp2101::brightness_to_voltage(255), 0x1C);
    }
}
