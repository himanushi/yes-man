//! LTR-553ALS-WA Ambient Light Sensor driver

use esp_idf_hal::i2c::I2cDriver;
use crate::config::i2c::ltr553;
use crate::error::YesManError;

/// LTR-553ALS-WA Ambient Light Sensor operations
pub struct Ltr553;

impl Ltr553 {
    /// Initialize the ambient light sensor
    pub fn init(i2c: &mut I2cDriver) -> Result<(), YesManError> {
        log::info!("Initializing LTR-553 at 0x{:02X}...", ltr553::ADDR);

        // Verify manufacturer ID
        let manufac_id = Self::read_register(i2c, ltr553::reg::MANUFAC_ID)?;
        if manufac_id != ltr553::EXPECTED_MANUFAC_ID {
            log::warn!(
                "LTR-553 unexpected manufacturer ID: 0x{:02X} (expected: 0x{:02X})",
                manufac_id,
                ltr553::EXPECTED_MANUFAC_ID
            );
        }

        // Enable ALS with 1x gain (wide range: 1 ~ 64k lux)
        Self::write_register(i2c, ltr553::reg::ALS_CONTR, ltr553::als_ctrl::GAIN_1X | ltr553::als_ctrl::ACTIVE)?;

        // Set measurement rate: 100ms integration, 500ms repeat rate
        // Bits [2:0] = integration time, Bits [5:3] = measurement rate
        // 0x03 = 100ms integration, 500ms measurement rate
        Self::write_register(i2c, ltr553::reg::ALS_MEAS_RATE, 0x03)?;

        log::info!("LTR-553 initialized (manufacturer ID: 0x{:02X})", manufac_id);
        Ok(())
    }

    /// Read ambient light level (raw ADC value)
    /// Returns CH0 (visible + IR) and CH1 (IR only) values
    pub fn read_als_raw(i2c: &mut I2cDriver) -> Result<(u16, u16), YesManError> {
        // Read all 4 bytes at once (CH1_0, CH1_1, CH0_0, CH0_1)
        let mut buf = [0u8; 4];
        i2c.write_read(ltr553::ADDR, &[ltr553::reg::ALS_DATA_CH1_0], &mut buf, 100)
            .map_err(|e| YesManError::I2c(format!("LTR-553 read error: {:?}", e)))?;

        let ch1 = u16::from_le_bytes([buf[0], buf[1]]);
        let ch0 = u16::from_le_bytes([buf[2], buf[3]]);

        Ok((ch0, ch1))
    }

    /// Read ambient light and calculate approximate lux value
    /// Uses simplified calculation suitable for backlight control
    pub fn read_lux(i2c: &mut I2cDriver) -> Result<u32, YesManError> {
        let (ch0, ch1) = Self::read_als_raw(i2c)?;

        // Simplified lux calculation
        // For accurate lux, need to account for gain, integration time, and ratio
        // This approximation is sufficient for backlight control
        let lux = if ch0 == 0 {
            0
        } else {
            let ratio = (ch1 as f32) / (ch0 as f32);
            let lux_f = if ratio < 0.45 {
                1.7743 * (ch0 as f32) + 1.1059 * (ch1 as f32)
            } else if ratio < 0.64 {
                4.2785 * (ch0 as f32) - 1.9548 * (ch1 as f32)
            } else if ratio < 0.85 {
                0.5926 * (ch0 as f32) + 0.1185 * (ch1 as f32)
            } else {
                0.0
            };
            lux_f as u32
        };

        Ok(lux)
    }

    /// Convert lux to backlight brightness percentage (0-100)
    /// Uses logarithmic mapping for natural perception
    pub fn lux_to_brightness(lux: u32) -> u8 {
        // Human eye perceives light logarithmically
        // Map lux ranges to brightness levels:
        // 0-10 lux: 10-30% (dark room)
        // 10-100 lux: 30-50% (dim indoor)
        // 100-1000 lux: 50-70% (indoor)
        // 1000-10000 lux: 70-90% (bright indoor / shade)
        // 10000+ lux: 90-100% (direct sunlight)

        const MIN_BRIGHTNESS: u8 = 10;
        const MAX_BRIGHTNESS: u8 = 100;

        if lux == 0 {
            return MIN_BRIGHTNESS;
        }

        // Logarithmic mapping
        // log10(1) = 0, log10(10) = 1, log10(100) = 2, log10(10000) = 4
        let log_lux = (lux as f32).log10();

        // Map log10 range [0, 4] to brightness [10, 100]
        let normalized = (log_lux / 4.0).clamp(0.0, 1.0);
        let brightness = MIN_BRIGHTNESS as f32 + normalized * (MAX_BRIGHTNESS - MIN_BRIGHTNESS) as f32;

        brightness as u8
    }

    fn read_register(i2c: &mut I2cDriver, reg: u8) -> Result<u8, YesManError> {
        let mut buf = [0u8; 1];
        i2c.write_read(ltr553::ADDR, &[reg], &mut buf, 100)
            .map_err(|e| YesManError::I2c(format!("LTR-553 read error: {:?}", e)))?;
        Ok(buf[0])
    }

    fn write_register(i2c: &mut I2cDriver, reg: u8, value: u8) -> Result<(), YesManError> {
        i2c.write(ltr553::ADDR, &[reg, value], 100)
            .map_err(|e| YesManError::I2c(format!("LTR-553 write error: {:?}", e)))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lux_to_brightness_dark() {
        assert_eq!(Ltr553::lux_to_brightness(0), 10);
        assert!(Ltr553::lux_to_brightness(1) >= 10);
        assert!(Ltr553::lux_to_brightness(10) <= 50);
    }

    #[test]
    fn test_lux_to_brightness_bright() {
        assert!(Ltr553::lux_to_brightness(10000) >= 90);
        assert_eq!(Ltr553::lux_to_brightness(100000), 100);
    }

    #[test]
    fn test_lux_to_brightness_monotonic() {
        let b1 = Ltr553::lux_to_brightness(10);
        let b2 = Ltr553::lux_to_brightness(100);
        let b3 = Ltr553::lux_to_brightness(1000);
        assert!(b1 <= b2);
        assert!(b2 <= b3);
    }
}
