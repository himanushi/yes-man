//! Color definitions for Yes Man

use embedded_graphics::pixelcolor::Rgb565;
use embedded_graphics::prelude::RgbColor;

/// Yes Man themed colors
pub mod yes_man {
    use super::*;

    /// Securitron screen green/yellow
    pub const SCREEN_BG: Rgb565 = Rgb565::new(20, 50, 5);  // Dark amber/green

    /// Bright screen color for face
    pub const FACE_COLOR: Rgb565 = Rgb565::new(31, 63, 10);  // Bright yellow-green

    /// Eye color
    pub const EYE_COLOR: Rgb565 = Rgb565::BLACK;

    /// Mouth color
    pub const MOUTH_COLOR: Rgb565 = Rgb565::BLACK;

    /// Text color
    pub const TEXT_COLOR: Rgb565 = Rgb565::new(31, 63, 20);  // Bright green
}

/// Standard colors
pub mod standard {
    use super::*;

    pub const BLACK: Rgb565 = Rgb565::BLACK;
    pub const WHITE: Rgb565 = Rgb565::WHITE;
    pub const RED: Rgb565 = Rgb565::RED;
    pub const GREEN: Rgb565 = Rgb565::GREEN;
    pub const BLUE: Rgb565 = Rgb565::BLUE;
    pub const YELLOW: Rgb565 = Rgb565::YELLOW;
    pub const CYAN: Rgb565 = Rgb565::CYAN;
    pub const MAGENTA: Rgb565 = Rgb565::MAGENTA;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_colors_are_different() {
        assert_ne!(yes_man::SCREEN_BG, yes_man::FACE_COLOR);
        assert_ne!(yes_man::EYE_COLOR, yes_man::FACE_COLOR);
    }

    #[test]
    fn test_standard_colors() {
        assert_eq!(standard::BLACK, Rgb565::BLACK);
        assert_eq!(standard::WHITE, Rgb565::WHITE);
    }
}
