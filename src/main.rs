//! Yes Man - M5Stack Core S3
//!
//! Entry point for the Yes Man application.

use esp_idf_hal::delay::FreeRtos;
use esp_idf_hal::prelude::*;
use esp_idf_svc::log::EspLogger;

use yes_man::app::{AppScheduler, BacklightController};
use yes_man::hardware::{Board, Ltr553};

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
    let mut board = Board::init(peripherals).map_err(|e| anyhow::anyhow!("{}", e))?;

    log::info!("Yes Man is ready!");
    log::info!("========================================");

    // Initialize scheduler and backlight controller
    let mut scheduler = AppScheduler::new();
    let mut backlight_ctrl = BacklightController::new();

    // Main loop with cooperative scheduling
    log::info!("Starting main loop (backlight interval: {}ms, tick: {}ms)",
        scheduler.backlight.interval_ms(), scheduler.tick_interval_ms());

    loop {
        // Task: Ambient light-based backlight adjustment
        if scheduler.backlight.should_run() {
            match board.read_ambient_light() {
                Ok(lux) => {
                    let target = Ltr553::lux_to_brightness(lux);
                    backlight_ctrl.set_target(target);
                    log::info!("Ambient: {} lux -> target: {}%, current: {}%",
                        lux, target, backlight_ctrl.current());
                }
                Err(e) => {
                    log::warn!("Failed to read ambient light: {}", e);
                }
            }

            // Apply smoothed brightness update
            if let Some(new_brightness) = backlight_ctrl.update() {
                if let Err(e) = board.set_backlight(new_brightness) {
                    log::warn!("Failed to set backlight: {}", e);
                } else {
                    log::info!("Backlight updated to {}%", new_brightness);
                }
            }
        }

        // Sleep for the tick interval
        FreeRtos::delay_ms(scheduler.tick_interval_ms());
    }
}
