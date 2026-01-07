//! AW9523B I/O Expander driver

use esp_idf_hal::delay::FreeRtos;
use esp_idf_hal::i2c::I2cDriver;
use crate::config::i2c::aw9523;
use crate::error::YesManError;

/// AW9523B I/O Expander operations
pub struct Aw9523;

impl Aw9523 {
    /// Initialize the AW9523
    pub fn init(i2c: &mut I2cDriver) -> Result<(), YesManError> {
        log::info!("Initializing AW9523 at 0x{:02X}...", aw9523::ADDR);

        // Soft reset
        let _ = Self::write_register(i2c, aw9523::reg::SOFT_RESET, 0x00);
        FreeRtos::delay_ms(10);

        // Verify chip ID
        let chip_id = Self::read_register(i2c, aw9523::reg::CHIP_ID)?;
        log::info!("AW9523 chip ID: 0x{:02X} (expected: 0x{:02X})", chip_id, aw9523::EXPECTED_CHIP_ID);

        // Set control register - push-pull mode
        Self::write_register(i2c, aw9523::reg::CTL, aw9523::ctl::PUSH_PULL)?;

        // Configure P0 and P1 as output (0 = output)
        Self::write_register(i2c, aw9523::reg::CONFIG0, 0x00)?;
        Self::write_register(i2c, aw9523::reg::CONFIG1, 0x00)?;

        // Set initial output state:
        // P0: TOUCH_RST(0)=HIGH, BUS_OUT_EN(1)=HIGH
        // P1: CAM_RST(0)=HIGH, LCD_RST(1)=HIGH
        Self::write_register(i2c, aw9523::reg::OUTPUT0, 0x03)?; // P0.0, P0.1 HIGH
        Self::write_register(i2c, aw9523::reg::OUTPUT1, 0x03)?; // P1.0, P1.1 HIGH

        log::info!("AW9523 initialized (BUS_OUT_EN enabled)");
        Ok(())
    }

    /// Enable peripheral bus output
    pub fn enable_bus_output(i2c: &mut I2cDriver) -> Result<(), YesManError> {
        log::info!("Enabling peripheral bus output (P0.1)...");
        const BUS_OUT_EN: u8 = 1 << 1; // P0.1

        let mut p0_state = Self::read_register(i2c, aw9523::reg::OUTPUT0).unwrap_or(0);
        p0_state |= BUS_OUT_EN;
        Self::write_register(i2c, aw9523::reg::OUTPUT0, p0_state)?;

        log::info!("Peripheral bus enabled");
        Ok(())
    }

    /// Reset LCD via IO expander pin
    pub fn reset_lcd(i2c: &mut I2cDriver) -> Result<(), YesManError> {
        log::info!("Resetting LCD...");

        // Read current P1 state
        let mut p1_state = Self::read_register(i2c, aw9523::reg::OUTPUT1).unwrap_or(0);

        // LCD_RST LOW
        p1_state &= !aw9523::pins::LCD_RST;
        Self::write_register(i2c, aw9523::reg::OUTPUT1, p1_state)?;
        FreeRtos::delay_ms(50);

        // LCD_RST HIGH
        p1_state |= aw9523::pins::LCD_RST;
        Self::write_register(i2c, aw9523::reg::OUTPUT1, p1_state)?;
        FreeRtos::delay_ms(150);

        log::info!("LCD reset complete");
        Ok(())
    }

    /// Reset touch controller via IO expander pin (P0.0)
    pub fn reset_touch(i2c: &mut I2cDriver) -> Result<(), YesManError> {
        log::info!("Resetting touch controller...");

        // TOUCH_RST is P0.0
        const TOUCH_RST: u8 = 1 << 0;

        // Read current P0 state
        let mut p0_state = Self::read_register(i2c, aw9523::reg::OUTPUT0).unwrap_or(0);

        // TOUCH_RST LOW
        p0_state &= !TOUCH_RST;
        Self::write_register(i2c, aw9523::reg::OUTPUT0, p0_state)?;
        FreeRtos::delay_ms(10);

        // TOUCH_RST HIGH
        p0_state |= TOUCH_RST;
        Self::write_register(i2c, aw9523::reg::OUTPUT0, p0_state)?;
        FreeRtos::delay_ms(50);

        log::info!("Touch reset complete");
        Ok(())
    }

    /// Reset camera via IO expander pin (P1.0)
    /// Also needed for LTR-553 which shares the same ribbon cable
    pub fn reset_camera(i2c: &mut I2cDriver) -> Result<(), YesManError> {
        log::info!("Resetting camera (for LTR-553 access)...");

        // CAM_RST is P1.0
        const CAM_RST: u8 = 1 << 0;

        // Read current P1 state
        let mut p1_state = Self::read_register(i2c, aw9523::reg::OUTPUT1).unwrap_or(0);

        // CAM_RST LOW
        p1_state &= !CAM_RST;
        Self::write_register(i2c, aw9523::reg::OUTPUT1, p1_state)?;
        FreeRtos::delay_ms(10);

        // CAM_RST HIGH
        p1_state |= CAM_RST;
        Self::write_register(i2c, aw9523::reg::OUTPUT1, p1_state)?;
        FreeRtos::delay_ms(50);

        log::info!("Camera reset complete");
        Ok(())
    }

    /// Set a pin high
    pub fn set_pin_high(i2c: &mut I2cDriver, pin: u8) -> Result<(), YesManError> {
        if pin < 8 {
            // P0 pins
            let current = Self::read_register(i2c, aw9523::reg::OUTPUT0)?;
            Self::write_register(i2c, aw9523::reg::OUTPUT0, current | (1 << pin))?;
        } else {
            // P1 pins
            let current = Self::read_register(i2c, aw9523::reg::OUTPUT1)?;
            Self::write_register(i2c, aw9523::reg::OUTPUT1, current | (1 << (pin - 8)))?;
        }
        Ok(())
    }

    /// Set a pin low
    pub fn set_pin_low(i2c: &mut I2cDriver, pin: u8) -> Result<(), YesManError> {
        if pin < 8 {
            // P0 pins
            let current = Self::read_register(i2c, aw9523::reg::OUTPUT0)?;
            Self::write_register(i2c, aw9523::reg::OUTPUT0, current & !(1 << pin))?;
        } else {
            // P1 pins
            let current = Self::read_register(i2c, aw9523::reg::OUTPUT1)?;
            Self::write_register(i2c, aw9523::reg::OUTPUT1, current & !(1 << (pin - 8)))?;
        }
        Ok(())
    }

    /// Read a register
    fn read_register(i2c: &mut I2cDriver, reg: u8) -> Result<u8, YesManError> {
        let mut buf = [0u8; 1];
        i2c.write_read(aw9523::ADDR, &[reg], &mut buf, 100)
            .map_err(|e| YesManError::I2c(format!("AW9523 read error: {:?}", e)))?;
        Ok(buf[0])
    }

    /// Write a register
    fn write_register(i2c: &mut I2cDriver, reg: u8, value: u8) -> Result<(), YesManError> {
        i2c.write(aw9523::ADDR, &[reg, value], 100)
            .map_err(|e| YesManError::I2c(format!("AW9523 write error: {:?}", e)))?;
        Ok(())
    }
}
