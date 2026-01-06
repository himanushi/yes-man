//! GPIO Pin definitions for M5Stack Core S3

/// I2C Bus pins (internal)
pub mod i2c_pins {
    pub const SDA: u8 = 12;
    pub const SCL: u8 = 11;
}

/// SPI Display pins
pub mod spi_pins {
    pub const MOSI: u8 = 37;
    pub const SCLK: u8 = 36;
    pub const CS: u8 = 3;
    pub const DC: u8 = 35;
}

/// Touch panel pins
pub mod touch_pins {
    pub const INT: u8 = 21;
}

/// Audio pins
pub mod audio_pins {
    pub const I2S_MCLK: u8 = 0;
    pub const I2S_BCLK: u8 = 34;
    pub const I2S_LRCK: u8 = 33;
    pub const I2S_DATA_OUT: u8 = 13;
    pub const I2S_DATA_IN: u8 = 14;
}

/// SD Card pins (shares SPI bus with LCD)
pub mod sd_pins {
    pub const CS: u8 = 4;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_i2c_pins() {
        assert_eq!(i2c_pins::SDA, 12);
        assert_eq!(i2c_pins::SCL, 11);
    }

    #[test]
    fn test_spi_pins() {
        assert_eq!(spi_pins::MOSI, 37);
        assert_eq!(spi_pins::SCLK, 36);
        assert_eq!(spi_pins::CS, 3);
        assert_eq!(spi_pins::DC, 35);
    }
}
