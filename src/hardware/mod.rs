//! Hardware Abstraction Layer for M5Stack Core S3
//!
//! This module provides abstracted access to hardware components.

pub mod traits;
pub mod power;
pub mod io_expander;
pub mod board;

pub use power::Axp2101;
pub use io_expander::Aw9523;
pub use board::Board;
