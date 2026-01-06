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
const AW9523_REG_OUTPUT0: u8 = 0x02; // P0 output
const AW9523_REG_OUTPUT1: u8 = 0x03; // P1 output
const AW9523_REG_CONFIG0: u8 = 0x04; // P0 direction
const AW9523_REG_CONFIG1: u8 = 0x05; // P1 direction
const AW9523_REG_CTL: u8 = 0x11;     // Control register
const AW9523_REG_SOFTRESET: u8 = 0x7F;

// AW9523 pin assignments for M5Stack Core S3 (on P1 = register 0x03)
// LCD_RST is bit 5 of P1
const AW9523_LCD_RST_BIT: u8 = 1 << 5; // 0x20

// AXP2101 for backlight control
const AXP2101_ADDR: u8 = 0x34;
const AXP2101_REG_DLDO_EN: u8 = 0x90;  // DLDO enable register
const AXP2101_REG_DLDO1_VOL: u8 = 0x99; // DLDO1 voltage register

fn init_aw9523(i2c: &mut I2cDriver) -> anyhow::Result<()> {
    log::info!("Initializing AW9523...");

    // Soft reset
    let _ = i2c.write(AW9523_ADDR, &[AW9523_REG_SOFTRESET, 0x00], 100);
    FreeRtos::delay_ms(10);

    // Set control register - push-pull mode
    i2c.write(AW9523_ADDR, &[AW9523_REG_CTL, 0x10], 100)?;

    // Configure P0 and P1 as output (0 = output)
    i2c.write(AW9523_ADDR, &[AW9523_REG_CONFIG0, 0x00], 100)?;
    i2c.write(AW9523_ADDR, &[AW9523_REG_CONFIG1, 0x00], 100)?;

    // LCD Reset sequence: LOW -> delay -> HIGH
    // Set LCD_RST LOW
    i2c.write(AW9523_ADDR, &[AW9523_REG_OUTPUT1, 0x00], 100)?;
    FreeRtos::delay_ms(20);

    // Set LCD_RST HIGH
    i2c.write(AW9523_ADDR, &[AW9523_REG_OUTPUT1, AW9523_LCD_RST_BIT], 100)?;
    FreeRtos::delay_ms(20);

    log::info!("AW9523 initialized, LCD reset complete");
    Ok(())
}

fn init_axp2101_backlight(i2c: &mut I2cDriver) -> anyhow::Result<()> {
    log::info!("Initializing AXP2101 backlight...");

    // Read current DLDO enable state
    let mut buf = [0u8; 1];
    i2c.write_read(AXP2101_ADDR, &[AXP2101_REG_DLDO_EN], &mut buf, 100)?;
    log::info!("AXP2101 DLDO enable reg before: 0x{:02x}", buf[0]);

    // Enable DLDO1 (bit 7)
    let new_val = buf[0] | 0x80;
    i2c.write(AXP2101_ADDR, &[AXP2101_REG_DLDO_EN, new_val], 100)?;
    log::info!("AXP2101 DLDO enable reg after: 0x{:02x}", new_val);

    // Set DLDO1 voltage for backlight brightness
    // Value range: 0x00-0x1C (500mV to 3300mV in 100mV steps)
    // 0x1C = 3.3V (max brightness)
    let brightness = 0x1C; // Max brightness
    i2c.write(AXP2101_ADDR, &[AXP2101_REG_DLDO1_VOL, brightness], 100)?;
    log::info!("AXP2101 backlight set to brightness: 0x{:02x}", brightness);

    log::info!("AXP2101 backlight enabled");
    Ok(())
}

fn main() -> anyhow::Result<()> {
    // Initialize ESP-IDF
    esp_idf_svc::sys::link_patches();
    EspLogger::initialize_default();

    log::info!("=== Yes Man Starting ===");
    log::info!("Initializing M5Stack Core S3...");

    let peripherals = Peripherals::take()?;

    // M5Stack Core S3 I2C pins (internal bus)
    // SDA: GPIO 12
    // SCL: GPIO 11
    let i2c = peripherals.i2c0;
    let sda = peripherals.pins.gpio12;
    let scl = peripherals.pins.gpio11;

    let config = I2cConfig::new().baudrate(400.kHz().into());
    let mut i2c_driver = I2cDriver::new(i2c, sda, scl, &config)?;
    log::info!("I2C initialized");

    // Initialize AXP2101 backlight FIRST (power on LCD)
    init_axp2101_backlight(&mut i2c_driver)?;

    // Small delay after power on
    FreeRtos::delay_ms(50);

    // Initialize AW9523 (LCD reset)
    init_aw9523(&mut i2c_driver)?;

    // M5Stack Core S3 Display SPI pins
    // LCD_MOSI: GPIO 37
    // LCD_SCK: GPIO 36
    // LCD_CS: GPIO 3
    // LCD_DC: GPIO 35

    let spi = peripherals.spi2;
    let sclk = peripherals.pins.gpio36;
    let mosi = peripherals.pins.gpio37;
    let cs = peripherals.pins.gpio3;
    let dc = PinDriver::output(peripherals.pins.gpio35)?;

    log::info!("Configuring SPI...");

    // Configure SPI
    let spi_config = SpiConfig::new().baudrate(40.MHz().into());
    let spi_driver = SpiDeviceDriver::new_single(
        spi,
        sclk,
        mosi,
        Option::<esp_idf_hal::gpio::Gpio0>::None, // MISO not used
        Some(cs),
        &SpiDriverConfig::default(),
        &spi_config,
    )?;

    // Create display interface
    let di = SPIInterface::new(spi_driver, dc);

    // Initialize display (ILI9342C, 320x240) without reset pin (handled by AW9523)
    log::info!("Initializing display...");
    let mut delay = FreeRtos;
    let mut display = Builder::new(mipidsi::models::ILI9342CRgb565, di)
        .orientation(Orientation::new().rotate(Rotation::Deg180))
        .color_order(ColorOrder::Bgr)
        .display_size(320, 240)
        .init(&mut delay)
        .map_err(|e| anyhow::anyhow!("Display init error: {:?}", e))?;

    log::info!("Display initialized!");

    // Clear display with blue to verify it's working
    log::info!("Clearing display...");
    display
        .clear(Rgb565::BLUE)
        .map_err(|e| anyhow::anyhow!("Clear error: {:?}", e))?;

    // Draw "Hello World!" text
    log::info!("Drawing text...");
    let style = MonoTextStyle::new(&FONT_10X20, Rgb565::WHITE);
    Text::with_alignment(
        "Hello World!",
        Point::new(160, 120),
        style,
        Alignment::Center,
    )
    .draw(&mut display)
    .map_err(|e| anyhow::anyhow!("Draw error: {:?}", e))?;

    log::info!("=== Display Ready! ===");

    // Keep running
    loop {
        FreeRtos::delay_ms(1000);
    }
}
