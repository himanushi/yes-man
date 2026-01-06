//! Yes Man application

use embedded_graphics::prelude::*;
use embedded_graphics::pixelcolor::Rgb565;

use crate::error::YesManError;
use crate::graphics::{colors, Renderer};

/// Yes Man application state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum State {
    /// Idle state - showing default face
    Idle,
    /// Talking state - mouth animation
    Talking,
    /// Happy state - extra happy expression
    Happy,
    /// Thinking state - processing
    Thinking,
}

impl Default for State {
    fn default() -> Self {
        Self::Idle
    }
}

/// Yes Man Application
pub struct YesManApp {
    state: State,
}

impl YesManApp {
    /// Create a new Yes Man application
    pub fn new() -> Self {
        Self {
            state: State::default(),
        }
    }

    /// Get current state
    pub fn state(&self) -> State {
        self.state
    }

    /// Set state
    pub fn set_state(&mut self, state: State) {
        self.state = state;
        log::info!("Yes Man state changed to: {:?}", state);
    }

    /// Initialize the display with welcome message
    pub fn init_display<D>(&self, display: &mut D) -> Result<(), YesManError>
    where
        D: DrawTarget<Color = Rgb565>,
        D::Error: core::fmt::Debug,
    {
        log::info!("Initializing Yes Man display...");

        // Clear with Yes Man screen color
        Renderer::clear(display, colors::yes_man::SCREEN_BG)?;

        // Draw welcome message
        Renderer::draw_centered_text(display, "Hello World!", colors::standard::WHITE)?;

        log::info!("Yes Man display initialized");
        Ok(())
    }

    /// Update the display based on current state
    pub fn update<D>(&mut self, display: &mut D) -> Result<(), YesManError>
    where
        D: DrawTarget<Color = Rgb565>,
        D::Error: core::fmt::Debug,
    {
        match self.state {
            State::Idle => self.draw_idle(display),
            State::Talking => self.draw_talking(display),
            State::Happy => self.draw_happy(display),
            State::Thinking => self.draw_thinking(display),
        }
    }

    fn draw_idle<D>(&self, display: &mut D) -> Result<(), YesManError>
    where
        D: DrawTarget<Color = Rgb565>,
        D::Error: core::fmt::Debug,
    {
        Renderer::clear(display, colors::yes_man::SCREEN_BG)?;
        Renderer::draw_centered_text(display, "Yes Man", colors::yes_man::TEXT_COLOR)?;
        Ok(())
    }

    fn draw_talking<D>(&self, display: &mut D) -> Result<(), YesManError>
    where
        D: DrawTarget<Color = Rgb565>,
        D::Error: core::fmt::Debug,
    {
        Renderer::clear(display, colors::yes_man::SCREEN_BG)?;
        Renderer::draw_centered_text(display, "Sure thing!", colors::yes_man::TEXT_COLOR)?;
        Ok(())
    }

    fn draw_happy<D>(&self, display: &mut D) -> Result<(), YesManError>
    where
        D: DrawTarget<Color = Rgb565>,
        D::Error: core::fmt::Debug,
    {
        Renderer::clear(display, colors::yes_man::SCREEN_BG)?;
        Renderer::draw_centered_text(display, "Absolutely!", colors::yes_man::TEXT_COLOR)?;
        Ok(())
    }

    fn draw_thinking<D>(&self, display: &mut D) -> Result<(), YesManError>
    where
        D: DrawTarget<Color = Rgb565>,
        D::Error: core::fmt::Debug,
    {
        Renderer::clear(display, colors::yes_man::SCREEN_BG)?;
        Renderer::draw_centered_text(display, "Hmm...", colors::yes_man::TEXT_COLOR)?;
        Ok(())
    }
}

impl Default for YesManApp {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_state() {
        let app = YesManApp::new();
        assert_eq!(app.state(), State::Idle);
    }

    #[test]
    fn test_state_change() {
        let mut app = YesManApp::new();
        app.set_state(State::Talking);
        assert_eq!(app.state(), State::Talking);

        app.set_state(State::Happy);
        assert_eq!(app.state(), State::Happy);
    }
}
