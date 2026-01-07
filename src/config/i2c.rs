//! I2C device addresses and register definitions for M5Stack Core S3

/// AXP2101 Power Management IC
pub mod axp2101 {
    /// I2C address
    pub const ADDR: u8 = 0x34;

    /// Registers
    pub mod reg {
        pub const CHIP_ID: u8 = 0x03;
        pub const DLDO_ENABLE: u8 = 0x90;
        pub const BLDO_ENABLE: u8 = 0x91;
        pub const DLDO1_VOLTAGE: u8 = 0x99;
    }

    /// Expected chip ID
    pub const EXPECTED_CHIP_ID: u8 = 0x4A;

    /// DLDO1 enable bit (for backlight)
    pub const DLDO1_ENABLE_BIT: u8 = 0x80;

    /// Voltage settings (500mV + value * 100mV)
    pub mod voltage {
        pub const V3_3: u8 = 0x1C; // 3.3V
        pub const V2_8: u8 = 0x17; // 2.8V
    }
}

/// AW9523B I/O Expander
pub mod aw9523 {
    /// I2C address
    pub const ADDR: u8 = 0x58;

    /// Registers
    pub mod reg {
        pub const INPUT0: u8 = 0x00;
        pub const INPUT1: u8 = 0x01;
        pub const OUTPUT0: u8 = 0x02;
        pub const OUTPUT1: u8 = 0x03;
        pub const CONFIG0: u8 = 0x04;
        pub const CONFIG1: u8 = 0x05;
        pub const INT0: u8 = 0x06;
        pub const INT1: u8 = 0x07;
        pub const CHIP_ID: u8 = 0x10;
        pub const CTL: u8 = 0x11;
        pub const SOFT_RESET: u8 = 0x7F;
    }

    /// Expected chip ID
    pub const EXPECTED_CHIP_ID: u8 = 0x23;

    /// Pin assignments on P1 (pins 8-15)
    pub mod pins {
        /// LCD Reset (P1_1 = pin 9)
        pub const LCD_RST: u8 = 1 << 1;
        /// Touch Reset (P1_0 = pin 8)
        pub const TOUCH_RST: u8 = 1 << 0;
        /// Bus output enable (P1_2 = pin 10)
        pub const BUS_OUT_EN: u8 = 1 << 2;
    }

    /// Control register values
    pub mod ctl {
        /// Push-pull mode for P0
        pub const PUSH_PULL: u8 = 0x10;
    }
}

/// FT6336U Touch Controller
pub mod ft6336 {
    /// I2C address
    pub const ADDR: u8 = 0x38;
}

/// ES7210 Audio ADC
pub mod es7210 {
    /// I2C address
    pub const ADDR: u8 = 0x40;
}

/// AW88298 Audio Amplifier
pub mod aw88298 {
    /// I2C address
    pub const ADDR: u8 = 0x36;
}

/// LTR-553ALS-WA Ambient Light / Proximity Sensor
pub mod ltr553 {
    /// I2C address
    pub const ADDR: u8 = 0x23;

    /// Registers
    pub mod reg {
        /// ALS control (gain, mode)
        pub const ALS_CONTR: u8 = 0x80;
        /// Proximity sensor control
        pub const PS_CONTR: u8 = 0x81;
        /// ALS measurement rate
        pub const ALS_MEAS_RATE: u8 = 0x85;
        /// Manufacturer ID (should be 0x05)
        pub const MANUFAC_ID: u8 = 0x86;
        /// Part ID
        pub const PART_ID: u8 = 0x87;
        /// ALS data channel 1 low byte
        pub const ALS_DATA_CH1_0: u8 = 0x88;
        /// ALS data channel 1 high byte
        pub const ALS_DATA_CH1_1: u8 = 0x89;
        /// ALS data channel 0 low byte
        pub const ALS_DATA_CH0_0: u8 = 0x8A;
        /// ALS data channel 0 high byte
        pub const ALS_DATA_CH0_1: u8 = 0x8B;
        /// ALS/PS status
        pub const ALS_PS_STATUS: u8 = 0x8C;
    }

    /// ALS control values
    pub mod als_ctrl {
        /// ALS active mode
        pub const ACTIVE: u8 = 0x01;
        /// ALS standby mode
        pub const STANDBY: u8 = 0x00;
        /// Gain 1x (1 ~ 64k lux)
        pub const GAIN_1X: u8 = 0x00;
        /// Gain 2x
        pub const GAIN_2X: u8 = 0x04;
        /// Gain 4x
        pub const GAIN_4X: u8 = 0x08;
        /// Gain 8x
        pub const GAIN_8X: u8 = 0x0C;
        /// Gain 48x
        pub const GAIN_48X: u8 = 0x18;
        /// Gain 96x (0.01 ~ 600 lux)
        pub const GAIN_96X: u8 = 0x1C;
    }

    /// Expected manufacturer ID
    pub const EXPECTED_MANUFAC_ID: u8 = 0x05;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_axp2101_address() {
        assert_eq!(axp2101::ADDR, 0x34);
    }

    #[test]
    fn test_aw9523_address() {
        assert_eq!(aw9523::ADDR, 0x58);
    }

    #[test]
    fn test_aw9523_lcd_rst_pin() {
        assert_eq!(aw9523::pins::LCD_RST, 0x02);
    }

    #[test]
    fn test_i2c_addresses_in_valid_range() {
        // Valid 7-bit I2C addresses: 0x08 - 0x77
        let addresses = [
            axp2101::ADDR,
            aw9523::ADDR,
            ft6336::ADDR,
            es7210::ADDR,
            aw88298::ADDR,
            ltr553::ADDR,
        ];
        for addr in addresses {
            assert!(addr >= 0x08 && addr <= 0x77, "Invalid I2C address: 0x{:02X}", addr);
        }
    }

    #[test]
    fn test_ltr553_address() {
        assert_eq!(ltr553::ADDR, 0x23);
    }
}
