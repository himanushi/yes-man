//! Error types for Yes Man

use core::fmt;

/// Main error type for Yes Man
#[derive(Debug)]
pub enum YesManError {
    /// I2C communication error
    I2c(String),
    /// SPI communication error
    Spi(String),
    /// GPIO error
    Gpio(String),
    /// Display error
    Display(String),
    /// Configuration error
    Config(String),
    /// General hardware error
    Hardware(String),
}

impl fmt::Display for YesManError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            YesManError::I2c(msg) => write!(f, "I2C error: {}", msg),
            YesManError::Spi(msg) => write!(f, "SPI error: {}", msg),
            YesManError::Gpio(msg) => write!(f, "GPIO error: {}", msg),
            YesManError::Display(msg) => write!(f, "Display error: {}", msg),
            YesManError::Config(msg) => write!(f, "Config error: {}", msg),
            YesManError::Hardware(msg) => write!(f, "Hardware error: {}", msg),
        }
    }
}

impl std::error::Error for YesManError {}

/// Result type alias for Yes Man operations
pub type Result<T> = core::result::Result<T, YesManError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = YesManError::I2c("test error".to_string());
        assert_eq!(format!("{}", err), "I2C error: test error");
    }

    #[test]
    fn test_error_debug() {
        let err = YesManError::Display("display issue".to_string());
        let debug_str = format!("{:?}", err);
        assert!(debug_str.contains("Display"));
    }
}
