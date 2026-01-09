//! Simple cooperative scheduler for periodic tasks
//!
//! This scheduler uses a time-slice based approach where each task
//! has a configurable interval. The main loop calls `tick()` which
//! checks if any tasks need to run.
//!
//! Design rationale:
//! - Cooperative (non-preemptive): Tasks must complete before others run
//! - Time-based: Uses system time to track intervals
//! - Low overhead: No dynamic allocation, no OS primitives
//! - Suitable for periodic sensor reading, animation updates, etc.

/// Get current time in milliseconds since boot
pub fn now_ms() -> u64 {
    // esp_timer_get_time() returns microseconds since boot
    (unsafe { esp_idf_svc::sys::esp_timer_get_time() } / 1000) as u64
}

/// A periodic task tracker
#[derive(Debug)]
pub struct PeriodicTask {
    /// Interval between executions in milliseconds
    interval_ms: u32,
    /// Last execution time in milliseconds
    last_run_ms: u64,
    /// Whether the task is enabled
    enabled: bool,
}

impl PeriodicTask {
    /// Create a new periodic task with the given interval
    pub fn new(interval_ms: u32) -> Self {
        Self {
            interval_ms,
            last_run_ms: 0,
            enabled: true,
        }
    }

    /// Check if the task should run and update last_run if so
    /// Returns true if the task should execute
    pub fn should_run(&mut self) -> bool {
        if !self.enabled {
            return false;
        }

        let now = now_ms();
        if now.wrapping_sub(self.last_run_ms) >= self.interval_ms as u64 {
            self.last_run_ms = now;
            true
        } else {
            false
        }
    }

    /// Force the task to run on the next check
    pub fn force_next(&mut self) {
        self.last_run_ms = 0;
    }

    /// Enable the task
    pub fn enable(&mut self) {
        self.enabled = true;
    }

    /// Disable the task
    pub fn disable(&mut self) {
        self.enabled = false;
    }

    /// Check if the task is enabled
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Get the interval in milliseconds
    pub fn interval_ms(&self) -> u32 {
        self.interval_ms
    }

    /// Set a new interval
    pub fn set_interval_ms(&mut self, interval_ms: u32) {
        self.interval_ms = interval_ms;
    }
}

/// Application scheduler managing all periodic tasks
pub struct AppScheduler {
    /// Backlight adjustment task (reads ambient light, updates backlight)
    pub backlight: PeriodicTask,
    /// Main loop tick interval (for FreeRtos delay)
    tick_interval_ms: u32,
}

impl AppScheduler {
    /// Create a new scheduler with default intervals
    ///
    /// Default intervals:
    /// - Backlight: 500ms (ambient light doesn't change rapidly)
    /// - Tick: 50ms (main loop sleep time)
    pub fn new() -> Self {
        Self {
            backlight: PeriodicTask::new(500),
            tick_interval_ms: 50,
        }
    }

    /// Create a scheduler with custom intervals
    pub fn with_intervals(backlight_interval_ms: u32, tick_interval_ms: u32) -> Self {
        Self {
            backlight: PeriodicTask::new(backlight_interval_ms),
            tick_interval_ms,
        }
    }

    /// Get the main loop tick interval
    pub fn tick_interval_ms(&self) -> u32 {
        self.tick_interval_ms
    }

    /// Set the main loop tick interval
    pub fn set_tick_interval_ms(&mut self, interval_ms: u32) {
        self.tick_interval_ms = interval_ms;
    }
}

impl Default for AppScheduler {
    fn default() -> Self {
        Self::new()
    }
}

/// Backlight controller with smoothing
///
/// Smooths brightness transitions to avoid jarring changes.
/// Uses exponential moving average for natural-feeling transitions.
pub struct BacklightController {
    /// Current brightness level (0-100)
    current_brightness: u8,
    /// Target brightness level (0-100)
    target_brightness: u8,
    /// Smoothing factor (0.0 = no change, 1.0 = instant change)
    /// Default: 0.3 (moderate smoothing)
    smoothing_factor: f32,
    /// Minimum change threshold to update hardware
    min_change_threshold: u8,
}

impl BacklightController {
    /// Create a new backlight controller
    pub fn new() -> Self {
        Self {
            current_brightness: 100, // Start at max
            target_brightness: 100,
            smoothing_factor: 0.3,
            min_change_threshold: 2,
        }
    }

    /// Set the target brightness (from ambient light calculation)
    pub fn set_target(&mut self, brightness: u8) {
        self.target_brightness = brightness.min(100);
    }

    /// Update the current brightness towards target using smoothing
    /// Returns Some(new_level) if hardware should be updated, None otherwise
    pub fn update(&mut self) -> Option<u8> {
        if self.current_brightness == self.target_brightness {
            return None;
        }

        // Exponential moving average
        let current = self.current_brightness as f32;
        let target = self.target_brightness as f32;
        let new_brightness = current + (target - current) * self.smoothing_factor;

        let new_level = new_brightness.round() as u8;

        // Only update if change exceeds threshold
        let diff = (new_level as i16 - self.current_brightness as i16).unsigned_abs() as u8;
        if diff >= self.min_change_threshold {
            self.current_brightness = new_level;
            Some(new_level)
        } else if diff > 0 && self.current_brightness != self.target_brightness {
            // Snap to target when very close
            self.current_brightness = self.target_brightness;
            Some(self.current_brightness)
        } else {
            None
        }
    }

    /// Get current brightness level
    pub fn current(&self) -> u8 {
        self.current_brightness
    }

    /// Get target brightness level
    pub fn target(&self) -> u8 {
        self.target_brightness
    }

    /// Set smoothing factor (0.0-1.0)
    pub fn set_smoothing(&mut self, factor: f32) {
        self.smoothing_factor = factor.clamp(0.0, 1.0);
    }
}

impl Default for BacklightController {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_periodic_task_initial_run() {
        let _task = PeriodicTask::new(100);
        // First call should always run (last_run is 0)
        // Note: This test won't work correctly without mocking time
        // In real usage, the first check after boot will run
    }

    #[test]
    fn test_periodic_task_disable() {
        let mut task = PeriodicTask::new(100);
        task.disable();
        assert!(!task.is_enabled());
        assert!(!task.should_run());
    }

    #[test]
    fn test_backlight_controller_smoothing() {
        let mut ctrl = BacklightController::new();
        ctrl.set_target(50);

        // First update should move towards target
        let update1 = ctrl.update();
        assert!(update1.is_some());
        assert!(ctrl.current() < 100);
        assert!(ctrl.current() > 50);
    }

    #[test]
    fn test_backlight_controller_no_change() {
        let mut ctrl = BacklightController::new();
        // Target equals current (100)
        ctrl.set_target(100);
        assert!(ctrl.update().is_none());
    }

    #[test]
    fn test_backlight_controller_convergence() {
        let mut ctrl = BacklightController::new();
        ctrl.set_smoothing(0.5);
        ctrl.set_target(0);

        // Should converge to target
        for _ in 0..20 {
            ctrl.update();
        }
        assert_eq!(ctrl.current(), 0);
    }
}
