//! Board abstraction for M5Stack Core S3 (Facade pattern)
//!
//! This module provides a high-level interface for initializing
//! and accessing all M5Stack Core S3 hardware components.

use esp_idf_hal::delay::FreeRtos;
use esp_idf_hal::gpio::PinDriver;
use esp_idf_hal::i2c::{I2cConfig, I2cDriver};
use esp_idf_hal::prelude::*;
use esp_idf_hal::spi::{config::Config as SpiConfig, SpiDeviceDriver, SpiDriverConfig};

use display_interface_spi::SPIInterface;
use embedded_graphics::{
    mono_font::{ascii::FONT_10X20, MonoTextStyle},
    pixelcolor::Rgb565,
    prelude::*,
    text::{Alignment, Text},
};
use mipidsi::{options::*, Builder, models::ILI9342CRgb565};

use crate::config::{pins, display as display_config};
use crate::error::YesManError;
use crate::hardware::{Axp2101, Aw9523, Ltr553};

/// M5Stack Core S3 Board (Facade)
///
/// Provides unified access to all board hardware.
pub struct Board;

/// Runtime context for board hardware access
///
/// This struct holds the I2C driver and provides methods
/// for runtime hardware operations (e.g., sensor reading, backlight control).
pub struct BoardRuntime<'d> {
    i2c: I2cDriver<'d>,
}

impl<'d> BoardRuntime<'d> {
    /// Read ambient light level in lux
    pub fn read_ambient_light(&mut self) -> Result<u32, YesManError> {
        Ltr553::read_lux(&mut self.i2c)
    }

    /// Set backlight brightness (0-100)
    pub fn set_backlight(&mut self, level: u8) -> Result<(), YesManError> {
        Axp2101::set_brightness(&mut self.i2c, level)
    }

    /// Get mutable reference to I2C driver for direct access
    pub fn i2c(&mut self) -> &mut I2cDriver<'d> {
        &mut self.i2c
    }
}

impl Board {
    /// Initialize all board hardware and display Hello World
    ///
    /// This is the main entry point for hardware initialization.
    /// It follows the correct initialization sequence:
    /// 1. I2C bus
    /// 2. Power management (backlight)
    /// 3. I/O expander (LCD reset)
    /// 4. Ambient light sensor
    /// 5. SPI display
    ///
    /// Returns a `BoardRuntime` that provides runtime access to hardware.
    pub fn init(peripherals: Peripherals) -> Result<BoardRuntime<'static>, YesManError> {
        // Wait for serial monitor to connect
        FreeRtos::delay_ms(500);

        log::info!("========================================");
        log::info!("Initializing M5Stack Core S3 Board");
        log::info!("========================================");

        // Step 1: Initialize I2C (400kHz like M5Unified)
        log::info!("Step 1: Initializing I2C bus...");
        let i2c_config = I2cConfig::new().baudrate(400.kHz().into());
        let mut i2c = I2cDriver::new(
            peripherals.i2c0,
            peripherals.pins.gpio12,
            peripherals.pins.gpio11,
            &i2c_config,
        ).map_err(|e| YesManError::I2c(format!("I2C init error: {:?}", e)))?;

        log::info!("I2C initialized (SDA={}, SCL={})", pins::i2c_pins::SDA, pins::i2c_pins::SCL);

        // Step 2: Initialize power management
        log::info!("Step 2: Initializing power management...");
        Axp2101::init(&mut i2c)?;
        Axp2101::enable_backlight(&mut i2c)?;
        FreeRtos::delay_ms(50);

        // Step 3: Initialize I/O expander
        log::info!("Step 3: Initializing I/O expander...");
        Aw9523::init(&mut i2c)?;
        Aw9523::reset_lcd(&mut i2c)?;
        FreeRtos::delay_ms(100);

        // Step 4: Enable power for camera/LTR553
        log::info!("Step 4: Enabling power for camera/LTR553...");
        Axp2101::enable_aldo3(&mut i2c)?;
        Axp2101::enable_peripheral_power(&mut i2c)?;
        FreeRtos::delay_ms(100); // Longer delay for power stabilization

        // Step 4a: Reset camera and touch (LTR-553 shares ribbon cable)
        log::info!("Step 4a: Resetting camera and touch...");
        Aw9523::reset_camera(&mut i2c)?;
        Aw9523::reset_touch(&mut i2c)?;
        FreeRtos::delay_ms(100); // Wait for peripherals to stabilize

        // Debug: Read AW9523 output registers to verify pin states
        {
            let mut buf = [0u8; 1];
            if i2c.write_read(0x58, &[0x02], &mut buf, 100).is_ok() {
                log::info!("AW9523 P0 output: 0x{:02X} (expect BUS_OUT_EN=bit1, TOUCH_RST=bit0)", buf[0]);
            }
            if i2c.write_read(0x58, &[0x03], &mut buf, 100).is_ok() {
                log::info!("AW9523 P1 output: 0x{:02X} (expect CAM_RST=bit0, LCD_RST=bit1)", buf[0]);
            }
        }

        // Debug: Read AXP2101 power status registers
        {
            let mut buf = [0u8; 1];
            // LDO enable status
            if i2c.write_read(0x34, &[0x90], &mut buf, 100).is_ok() {
                log::info!("AXP2101 LDO_ONOFF (0x90): 0x{:02X}", buf[0]);
                log::info!("  DLDO1(bit7)={} DLDO2(bit6)={} BLDO2(bit5)={} BLDO1(bit4)={}",
                    (buf[0] >> 7) & 1, (buf[0] >> 6) & 1, (buf[0] >> 5) & 1, (buf[0] >> 4) & 1);
                log::info!("  ALDO4(bit3)={} ALDO3(bit2)={} ALDO2(bit1)={} ALDO1(bit0)={}",
                    (buf[0] >> 3) & 1, (buf[0] >> 2) & 1, (buf[0] >> 1) & 1, buf[0] & 1);
            }
            // DCDC enable status
            if i2c.write_read(0x34, &[0x80], &mut buf, 100).is_ok() {
                log::info!("AXP2101 DCDC_ONOFF (0x80): 0x{:02X}", buf[0]);
            }
            // Read all LDO voltages
            for (name, reg) in [("ALDO1", 0x92u8), ("ALDO2", 0x93), ("ALDO3", 0x94), ("ALDO4", 0x95),
                                ("BLDO1", 0x96), ("BLDO2", 0x97), ("DLDO1", 0x99), ("DLDO2", 0x9A)] {
                if i2c.write_read(0x34, &[reg], &mut buf, 100).is_ok() {
                    let mv = 500 + (buf[0] as u32) * 100;
                    log::info!("  {} (0x{:02X}): 0x{:02X} = {}mV", name, reg, buf[0], mv);
                }
            }
        }

        // Step 4b: Scan I2C bus to find devices (using write probe)
        log::info!("========================================");
        log::info!("I2C Bus Scan Results:");
        let mut found_count = 0;
        for addr in 0x08u8..0x78 {
            // Try write with empty data (address probe)
            if i2c.write(addr, &[], 10).is_ok() {
                found_count += 1;
                let name = match addr {
                    0x10 => "BMM150 (magnetometer)",
                    0x21 => "GC0308 (camera)",
                    0x23 => "LTR-553 (ambient light)",
                    0x34 => "AXP2101 (power)",
                    0x36 => "AW88298 (audio amp)",
                    0x38 => "FT6336U (touch)",
                    0x40 => "ES7210 (audio ADC)",
                    0x51 => "BM8563 (RTC)",
                    0x58 => "AW9523B (IO expander)",
                    0x68 | 0x69 => "BMI270 (IMU)",
                    _ => "unknown",
                };
                log::info!("  0x{:02X} - {}", addr, name);
            }
        }
        log::info!("Total: {} devices found", found_count);
        log::info!("========================================");

        // Wait longer and re-scan to catch slow devices
        FreeRtos::delay_ms(200);
        log::info!("Second I2C scan after delay:");
        for addr in [0x10u8, 0x21, 0x23, 0x36] {
            if i2c.write(addr, &[], 10).is_ok() {
                log::info!("  Found: 0x{:02X}", addr);
            } else {
                log::info!("  Missing: 0x{:02X}", addr);
            }
        }

        // Step 4c: Initialize ambient light sensor
        log::info!("Step 4c: Initializing ambient light sensor...");
        match Ltr553::init(&mut i2c) {
            Ok(_) => log::info!("Ambient light sensor initialized"),
            Err(e) => log::warn!("Ambient light sensor init failed (non-fatal): {}", e),
        }
        FreeRtos::delay_ms(50);

        // Step 5: Initialize SPI display
        log::info!("Step 5: Initializing SPI display...");
        let spi_config = SpiConfig::new().baudrate(display_config::spi::FREQ_HZ_SAFE.Hz().into());
        let spi = SpiDeviceDriver::new_single(
            peripherals.spi2,
            peripherals.pins.gpio36,
            peripherals.pins.gpio37,
            Option::<esp_idf_hal::gpio::Gpio0>::None,
            Some(peripherals.pins.gpio3),
            &SpiDriverConfig::default(),
            &spi_config,
        ).map_err(|e| YesManError::Spi(format!("SPI init error: {:?}", e)))?;

        let dc = PinDriver::output(peripherals.pins.gpio35)
            .map_err(|e| YesManError::Gpio(format!("DC pin error: {:?}", e)))?;

        // Create display interface
        let di = SPIInterface::new(spi, dc);

        // Initialize display
        log::info!("Initializing ILI9342C display...");
        let mut delay = FreeRtos;
        let mut display = Builder::new(ILI9342CRgb565, di)
            .orientation(Orientation::new().rotate(Rotation::Deg0))
            .color_order(ColorOrder::Bgr)
            .display_size(display_config::WIDTH, display_config::HEIGHT)
            .init(&mut delay)
            .map_err(|e| YesManError::Display(format!("Init error: {:?}", e)))?;

        log::info!("Display initialized ({}x{})", display_config::WIDTH, display_config::HEIGHT);

        // Draw Hello World
        log::info!("Drawing Hello World...");
        display.clear(Rgb565::new(5, 20, 5)) // Dark green Yes Man background
            .map_err(|e| YesManError::Display(format!("Clear error: {:?}", e)))?;

        let style = MonoTextStyle::new(&FONT_10X20, Rgb565::WHITE);
        Text::with_alignment(
            "Hello World!",
            Point::new(display_config::CENTER_X, display_config::CENTER_Y),
            style,
            Alignment::Center,
        )
        .draw(&mut display)
        .map_err(|e| YesManError::Display(format!("Draw error: {:?}", e)))?;

        log::info!("========================================");
        log::info!("Board initialization complete!");
        log::info!("========================================");

        // Keep display alive (don't drop it)
        core::mem::forget(display);

        Ok(BoardRuntime { i2c })
    }
}

/// I2C bus scanner utility
pub fn scan_i2c(i2c: &mut I2cDriver) {
    log::info!("=== Scanning I2C bus ===");
    for addr in 0x08..0x78 {
        let mut buf = [0u8; 1];
        if i2c.read(addr, &mut buf, 10).is_ok() {
            log::info!("Found I2C device at 0x{:02X}", addr);
        }
    }
    log::info!("=== I2C scan complete ===");
}
