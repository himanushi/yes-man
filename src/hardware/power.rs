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
        log::info!("Backlight: {}% -> voltage reg 0x{:02X}", level, voltage);
        Ok(())
    }

    /// Disable backlight
    pub fn disable_backlight(i2c: &mut I2cDriver) -> Result<(), YesManError> {
        let current = Self::read_register(i2c, axp2101::reg::DLDO_ENABLE)?;
        Self::write_register(i2c, axp2101::reg::DLDO_ENABLE, current & !axp2101::DLDO1_ENABLE_BIT)?;
        log::info!("Backlight disabled");
        Ok(())
    }

    /// Enable ALDO3 (3.3V for camera/LTR553)
    pub fn enable_aldo3(i2c: &mut I2cDriver) -> Result<(), YesManError> {
        log::info!("Enabling ALDO3 (camera/LTR553 power)...");

        // Set ALDO3 voltage to 3.3V
        Self::write_register(i2c, axp2101::ALDO3_VOLTAGE, axp2101::voltage::V3_3)?;

        // Enable ALDO3
        let current = Self::read_register(i2c, axp2101::LDO_ONOFF)?;
        Self::write_register(i2c, axp2101::LDO_ONOFF, current | axp2101::ALDO3_ENABLE_BIT)?;

        log::info!("ALDO3 enabled (3.3V)");
        Ok(())
    }

    /// 周辺機器の電源レール有効化 (BLDO1, BLDO2 バス電源用)
    pub fn enable_peripheral_power(i2c: &mut I2cDriver) -> Result<(), YesManError> {
        log::info!("周辺機器電源レール (BLDO1, BLDO2) を有効化中...");

        // BLDO1 有効ビット = 0x10 (bit 4)
        // BLDO2 有効ビット = 0x20 (bit 5)
        let current = Self::read_register(i2c, axp2101::LDO_ONOFF)?;
        Self::write_register(i2c, axp2101::LDO_ONOFF, current | 0x30)?; // BLDO1 + BLDO2 有効化

        log::info!("BLDO1/BLDO2 有効化完了");
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

    /// 明るさ (0-100) を電圧レジスタ値に変換
    fn brightness_to_voltage(brightness: u8) -> u8 {
        // DLDO1 電圧: 500mV + 値 * 100mV
        // 0x00 = 500mV, 0x1C = 3300mV (28段階)
        //
        // LED バックライトは点灯に最低 ~2.5V 必要
        // 0x14 = 2500mV (最小可視)
        // 0x1C = 3300mV (最大)
        //
        // 明るさ 0-100 を電圧範囲 0x14-0x1C (2.5V-3.3V) にマッピング
        const MIN_REG: u8 = 0x14; // 2.5V - LED 点灯の最小値
        const MAX_REG: u8 = 0x1C; // 3.3V - 最大輝度

        if brightness == 0 {
            return 0x00; // オフ
        }

        let clamped = brightness.min(100) as u32;
        let range = (MAX_REG - MIN_REG) as u32;
        MIN_REG + ((clamped * range) / 100) as u8
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_brightness_to_voltage() {
        // 0% = オフ
        assert_eq!(Axp2101::brightness_to_voltage(0), 0x00);
        // 100% = 最大 (3.3V)
        assert_eq!(Axp2101::brightness_to_voltage(100), 0x1C);
        // 1% = 最小可視 (2.5V)
        assert_eq!(Axp2101::brightness_to_voltage(1), 0x14);
        // 50% = 0x14 と 0x1C の中間
        assert_eq!(Axp2101::brightness_to_voltage(50), 0x18); // 0x14 + 4
        // クランプのテスト
        assert_eq!(Axp2101::brightness_to_voltage(255), 0x1C);
    }
}
