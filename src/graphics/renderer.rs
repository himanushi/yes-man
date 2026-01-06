//! Renderer for drawing on the display

use embedded_graphics::{
    mono_font::{ascii::FONT_10X20, MonoTextStyle},
    pixelcolor::Rgb565,
    prelude::*,
    text::{Alignment, Text},
};

use crate::config::display::{CENTER_X, CENTER_Y};
use crate::error::YesManError;

/// Renderer for drawing operations
pub struct Renderer;

impl Renderer {
    /// Draw centered text on the display
    pub fn draw_centered_text<D>(
        display: &mut D,
        text: &str,
        color: Rgb565,
    ) -> Result<(), YesManError>
    where
        D: DrawTarget<Color = Rgb565>,
        D::Error: core::fmt::Debug,
    {
        let style = MonoTextStyle::new(&FONT_10X20, color);
        Text::with_alignment(
            text,
            Point::new(CENTER_X, CENTER_Y),
            style,
            Alignment::Center,
        )
        .draw(display)
        .map_err(|e| YesManError::Display(format!("Draw text error: {:?}", e)))?;

        Ok(())
    }

    /// Draw text at a specific position
    pub fn draw_text<D>(
        display: &mut D,
        text: &str,
        position: Point,
        color: Rgb565,
    ) -> Result<(), YesManError>
    where
        D: DrawTarget<Color = Rgb565>,
        D::Error: core::fmt::Debug,
    {
        let style = MonoTextStyle::new(&FONT_10X20, color);
        Text::new(text, position, style)
            .draw(display)
            .map_err(|e| YesManError::Display(format!("Draw text error: {:?}", e)))?;

        Ok(())
    }

    /// Clear the display with a color
    pub fn clear<D>(display: &mut D, color: Rgb565) -> Result<(), YesManError>
    where
        D: DrawTarget<Color = Rgb565>,
        D::Error: core::fmt::Debug,
    {
        display
            .clear(color)
            .map_err(|e| YesManError::Display(format!("Clear error: {:?}", e)))?;

        Ok(())
    }
}
