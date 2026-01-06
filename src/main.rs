//! Yes Man - M5Stack Core S3
//!
//! Entry point for the Yes Man application.

use esp_idf_hal::delay::FreeRtos;
use esp_idf_hal::prelude::*;
use esp_idf_svc::log::EspLogger;

use yes_man::hardware::Board;

fn main() -> anyhow::Result<()> {
    // Initialize ESP-IDF
    esp_idf_svc::sys::link_patches();
    EspLogger::initialize_default();

    log::info!("========================================");
    log::info!("=== Yes Man - Fallout: New Vegas ===");
    log::info!("=== M5Stack Core S3 Edition ===");
    log::info!("========================================");

    // Initialize hardware and display
    let peripherals = Peripherals::take()?;
    Board::init(peripherals).map_err(|e| anyhow::anyhow!("{}", e))?;

    log::info!("Yes Man is ready!");
    log::info!("========================================");

    // Main loop
    loop {
        FreeRtos::delay_ms(1000);
    }
}
