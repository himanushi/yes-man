//! Hardware abstraction traits
//!
//! These traits allow for dependency injection and testing with mocks.

use crate::error::YesManError;

/// Power management trait
pub trait PowerManagement {
    /// Enable the display backlight
    fn enable_backlight(&mut self) -> Result<(), YesManError>;

    /// Set backlight brightness (0-100)
    fn set_brightness(&mut self, level: u8) -> Result<(), YesManError>;

    /// Disable the display backlight
    fn disable_backlight(&mut self) -> Result<(), YesManError>;
}

/// I/O Expander trait
pub trait IoExpander {
    /// Reset the LCD
    fn reset_lcd(&mut self) -> Result<(), YesManError>;

    /// Reset the touch controller
    fn reset_touch(&mut self) -> Result<(), YesManError>;

    /// Set a pin high
    fn set_pin_high(&mut self, pin: u8) -> Result<(), YesManError>;

    /// Set a pin low
    fn set_pin_low(&mut self, pin: u8) -> Result<(), YesManError>;
}

/// Display trait for drawing operations
pub trait DisplayDriver {
    /// Clear the display with a color
    fn clear(&mut self, color: embedded_graphics::pixelcolor::Rgb565) -> Result<(), YesManError>;

    /// Get display width
    fn width(&self) -> u16;

    /// Get display height
    fn height(&self) -> u16;
}

/// Ambient light sensor trait
pub trait AmbientLightSensor {
    /// Read ambient light level in lux
    fn read_lux(&mut self) -> Result<u32, YesManError>;

    /// Convert lux to recommended brightness (0-100)
    fn lux_to_brightness(&self, lux: u32) -> u8;
}

#[cfg(test)]
pub mod mocks {
    //! Mock implementations for testing

    use super::*;

    /// Mock power management for testing
    pub struct MockPower {
        pub backlight_enabled: bool,
        pub brightness: u8,
    }

    impl Default for MockPower {
        fn default() -> Self {
            Self {
                backlight_enabled: false,
                brightness: 0,
            }
        }
    }

    impl PowerManagement for MockPower {
        fn enable_backlight(&mut self) -> Result<(), YesManError> {
            self.backlight_enabled = true;
            self.brightness = 100;
            Ok(())
        }

        fn set_brightness(&mut self, level: u8) -> Result<(), YesManError> {
            self.brightness = level.min(100);
            Ok(())
        }

        fn disable_backlight(&mut self) -> Result<(), YesManError> {
            self.backlight_enabled = false;
            self.brightness = 0;
            Ok(())
        }
    }

    /// Mock I/O expander for testing
    pub struct MockIoExpander {
        pub lcd_reset_count: u32,
        pub touch_reset_count: u32,
        pub pin_states: [bool; 16],
    }

    impl Default for MockIoExpander {
        fn default() -> Self {
            Self {
                lcd_reset_count: 0,
                touch_reset_count: 0,
                pin_states: [false; 16],
            }
        }
    }

    impl IoExpander for MockIoExpander {
        fn reset_lcd(&mut self) -> Result<(), YesManError> {
            self.lcd_reset_count += 1;
            Ok(())
        }

        fn reset_touch(&mut self) -> Result<(), YesManError> {
            self.touch_reset_count += 1;
            Ok(())
        }

        fn set_pin_high(&mut self, pin: u8) -> Result<(), YesManError> {
            if (pin as usize) < self.pin_states.len() {
                self.pin_states[pin as usize] = true;
            }
            Ok(())
        }

        fn set_pin_low(&mut self, pin: u8) -> Result<(), YesManError> {
            if (pin as usize) < self.pin_states.len() {
                self.pin_states[pin as usize] = false;
            }
            Ok(())
        }
    }

    /// Mock ambient light sensor for testing
    pub struct MockAmbientLight {
        pub lux_value: u32,
    }

    impl Default for MockAmbientLight {
        fn default() -> Self {
            Self { lux_value: 100 }
        }
    }

    impl AmbientLightSensor for MockAmbientLight {
        fn read_lux(&mut self) -> Result<u32, YesManError> {
            Ok(self.lux_value)
        }

        fn lux_to_brightness(&self, lux: u32) -> u8 {
            // Simple linear mapping for testing
            ((lux.min(1000) * 100) / 1000) as u8
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn test_mock_power() {
            let mut power = MockPower::default();
            assert!(!power.backlight_enabled);

            power.enable_backlight().unwrap();
            assert!(power.backlight_enabled);
            assert_eq!(power.brightness, 100);

            power.set_brightness(50).unwrap();
            assert_eq!(power.brightness, 50);

            power.disable_backlight().unwrap();
            assert!(!power.backlight_enabled);
        }

        #[test]
        fn test_mock_io_expander() {
            let mut io = MockIoExpander::default();
            assert_eq!(io.lcd_reset_count, 0);

            io.reset_lcd().unwrap();
            assert_eq!(io.lcd_reset_count, 1);

            io.set_pin_high(5).unwrap();
            assert!(io.pin_states[5]);

            io.set_pin_low(5).unwrap();
            assert!(!io.pin_states[5]);
        }

        #[test]
        fn test_mock_ambient_light() {
            let mut als = MockAmbientLight::default();
            assert_eq!(als.read_lux().unwrap(), 100);

            als.lux_value = 500;
            assert_eq!(als.read_lux().unwrap(), 500);
            assert_eq!(als.lux_to_brightness(500), 50);
        }
    }
}
