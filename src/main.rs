use esp_idf_hal::{
    delay::FreeRtos,
    gpio::PinDriver,
    i2c::{I2cConfig, I2cDriver},
    prelude::*,
    spi::{config::Config as SpiConfig, SpiDeviceDriver, SpiDriverConfig},
};
use esp_idf_svc::log::EspLogger;

use display_interface_spi::SPIInterface;
use embedded_graphics::{
    mono_font::{ascii::FONT_10X20, MonoTextStyle},
    pixelcolor::Rgb565,
    prelude::*,
    text::{Alignment, Text},
};
use mipidsi::{options::*, Builder};

// AW9523 I2C address
const AW9523_ADDR: u8 = 0x58;

// AW9523 registers
const AW9523_REG_OUTPUT1: u8 = 0x03; // P1 output
const AW9523_REG_CONFIG0: u8 = 0x04; // P0 direction
const AW9523_REG_CONFIG1: u8 = 0x05; // P1 direction
const AW9523_REG_CTL: u8 = 0x11;     // Control register
const AW9523_REG_SOFTRESET: u8 = 0x7F;

// AW9523 pin assignments for M5Stack Core S3
// Pin 9 = P1 bit 1 (since P1 is pins 8-15, pin 9 = bit 1)
const AW9523_LCD_RST_BIT: u8 = 1 << 1; // 0x02

// AXP2101 for backlight control
const AXP2101_ADDR: u8 = 0x34;

fn i2c_scan(i2c: &mut I2cDriver) {
    log::info!("=== Scanning I2C bus ===");
    for addr in 0x08..0x78 {
        let mut buf = [0u8; 1];
        if i2c.read(addr, &mut buf, 10).is_ok() {
            log::info!("Found I2C device at 0x{:02X}", addr);
        }
    }
    log::info!("=== I2C scan complete ===");
}

fn init_aw9523(i2c: &mut I2cDriver) -> anyhow::Result<()> {
    log::info!("Initializing AW9523 at 0x{:02X}...", AW9523_ADDR);

    // Read chip ID to verify communication
    let mut buf = [0u8; 1];
    match i2c.write_read(AW9523_ADDR, &[0x10], &mut buf, 100) {
        Ok(_) => log::info!("AW9523 chip ID: 0x{:02X} (should be 0x23)", buf[0]),
        Err(e) => {
            log::error!("Failed to read AW9523: {:?}", e);
            return Err(anyhow::anyhow!("AW9523 communication failed"));
        }
    }

    // Soft reset
    let _ = i2c.write(AW9523_ADDR, &[AW9523_REG_SOFTRESET, 0x00], 100);
    FreeRtos::delay_ms(10);

    // Set control register - push-pull mode
    i2c.write(AW9523_ADDR, &[AW9523_REG_CTL, 0x10], 100)?;

    // Configure P0 and P1 as output (0 = output)
    i2c.write(AW9523_ADDR, &[AW9523_REG_CONFIG0, 0x00], 100)?;
    i2c.write(AW9523_ADDR, &[AW9523_REG_CONFIG1, 0x00], 100)?;

    // LCD Reset sequence: LOW -> delay -> HIGH
    log::info!("Performing LCD reset (pin 9 = P1 bit 1)...");
    i2c.write(AW9523_ADDR, &[AW9523_REG_OUTPUT1, 0x00], 100)?;
    FreeRtos::delay_ms(50);
    i2c.write(AW9523_ADDR, &[AW9523_REG_OUTPUT1, AW9523_LCD_RST_BIT], 100)?;
    FreeRtos::delay_ms(150);

    log::info!("AW9523 initialized");
    Ok(())
}

fn init_axp2101_backlight(i2c: &mut I2cDriver) -> anyhow::Result<()> {
    log::info!("Initializing AXP2101 at 0x{:02X}...", AXP2101_ADDR);

    // Read chip ID
    let mut buf = [0u8; 1];
    match i2c.write_read(AXP2101_ADDR, &[0x03], &mut buf, 100) {
        Ok(_) => log::info!("AXP2101 chip ID: 0x{:02X} (should be 0x4A for AXP2101)", buf[0]),
        Err(e) => {
            log::error!("Failed to read AXP2101: {:?}", e);
            return Err(anyhow::anyhow!("AXP2101 communication failed"));
        }
    }

    // Read and modify DLDO enable register (0x90)
    i2c.write_read(AXP2101_ADDR, &[0x90], &mut buf, 100)?;
    log::info!("AXP2101 reg 0x90 before: 0x{:02X}", buf[0]);

    // Enable DLDO1 (bit 7)
    i2c.write(AXP2101_ADDR, &[0x90, buf[0] | 0x80], 100)?;
    log::info!("AXP2101 DLDO1 enabled");

    // Set DLDO1 voltage to max (3.3V = 0x1C)
    i2c.write(AXP2101_ADDR, &[0x99, 0x1C], 100)?;
    log::info!("AXP2101 DLDO1 voltage set to 3.3V");

    log::info!("AXP2101 backlight initialized");
    Ok(())
}

fn main() -> anyhow::Result<()> {
    esp_idf_svc::sys::link_patches();
    EspLogger::initialize_default();

    log::info!("========================================");
    log::info!("=== Yes Man - M5Stack Core S3 ===");
    log::info!("========================================");

    let peripherals = Peripherals::take()?;

    // I2C setup (internal bus: SDA=GPIO12, SCL=GPIO11)
    let i2c = peripherals.i2c0;
    let sda = peripherals.pins.gpio12;
    let scl = peripherals.pins.gpio11;

    log::info!("Setting up I2C (SDA=12, SCL=11)...");
    let config = I2cConfig::new().baudrate(100.kHz().into());
    let mut i2c_driver = I2cDriver::new(i2c, sda, scl, &config)?;
    log::info!("I2C ready");

    // Scan to verify I2C is working
    i2c_scan(&mut i2c_driver);

    // Initialize power (backlight)
    match init_axp2101_backlight(&mut i2c_driver) {
        Ok(_) => log::info!("AXP2101 OK"),
        Err(e) => log::error!("AXP2101 FAILED: {:?}", e),
    }

    FreeRtos::delay_ms(100);

    // Initialize IO expander (LCD reset)
    match init_aw9523(&mut i2c_driver) {
        Ok(_) => log::info!("AW9523 OK"),
        Err(e) => log::error!("AW9523 FAILED: {:?}", e),
    }

    FreeRtos::delay_ms(100);

    // SPI Display setup
    log::info!("Setting up SPI display...");
    let spi = peripherals.spi2;
    let sclk = peripherals.pins.gpio36;
    let mosi = peripherals.pins.gpio37;
    let cs = peripherals.pins.gpio3;
    let dc = PinDriver::output(peripherals.pins.gpio35)?;

    let spi_config = SpiConfig::new().baudrate(20.MHz().into());
    let spi_driver = SpiDeviceDriver::new_single(
        spi,
        sclk,
        mosi,
        Option::<esp_idf_hal::gpio::Gpio0>::None,
        Some(cs),
        &SpiDriverConfig::default(),
        &spi_config,
    )?;
    log::info!("SPI ready");

    let di = SPIInterface::new(spi_driver, dc);

    log::info!("Initializing ILI9342C display...");
    let mut delay = FreeRtos;
    let mut display = Builder::new(mipidsi::models::ILI9342CRgb565, di)
        .orientation(Orientation::new().rotate(Rotation::Deg180))
        .color_order(ColorOrder::Bgr)
        .display_size(320, 240)
        .init(&mut delay)
        .map_err(|e| anyhow::anyhow!("Display init error: {:?}", e))?;

    log::info!("Display initialized! Clearing to BLUE...");
    display.clear(Rgb565::BLUE).map_err(|e| anyhow::anyhow!("Clear error: {:?}", e))?;

    log::info!("Drawing Hello World...");
    let style = MonoTextStyle::new(&FONT_10X20, Rgb565::WHITE);
    Text::with_alignment("Hello World!", Point::new(160, 120), style, Alignment::Center)
        .draw(&mut display)
        .map_err(|e| anyhow::anyhow!("Draw error: {:?}", e))?;

    log::info!("========================================");
    log::info!("=== DONE - Check the display! ===");
    log::info!("========================================");

    loop {
        FreeRtos::delay_ms(5000);
        log::info!("Running...");
    }
}
