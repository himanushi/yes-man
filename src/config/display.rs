//! Display configuration for M5Stack Core S3

/// Display dimensions
pub const WIDTH: u16 = 320;
pub const HEIGHT: u16 = 240;

/// Display center coordinates
pub const CENTER_X: i32 = (WIDTH / 2) as i32;
pub const CENTER_Y: i32 = (HEIGHT / 2) as i32;

/// SPI configuration
pub mod spi {
    /// SPI clock frequency in Hz
    pub const FREQ_HZ: u32 = 40_000_000; // 40 MHz

    /// Safe SPI frequency for debugging
    pub const FREQ_HZ_SAFE: u32 = 20_000_000; // 20 MHz
}

/// ILI9342C display controller settings (ESP32 only)
#[cfg(target_arch = "xtensa")]
pub mod ili9342c {
    use mipidsi::options::{ColorOrder, Orientation, Rotation};

    pub fn orientation() -> Orientation {
        Orientation::new().rotate(Rotation::Deg180)
    }

    pub const COLOR_ORDER: ColorOrder = ColorOrder::Bgr;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_display_dimensions() {
        assert_eq!(WIDTH, 320);
        assert_eq!(HEIGHT, 240);
    }

    #[test]
    fn test_center_coordinates() {
        assert_eq!(CENTER_X, 160);
        assert_eq!(CENTER_Y, 120);
    }

    #[test]
    fn test_spi_frequency() {
        assert!(spi::FREQ_HZ <= 80_000_000, "SPI freq too high");
        assert!(spi::FREQ_HZ_SAFE <= spi::FREQ_HZ);
    }
}
