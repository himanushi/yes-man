//! Application layer for Yes Man
//!
//! This module contains the main application logic.

pub mod yes_man;
pub mod scheduler;

pub use yes_man::YesManApp;
pub use scheduler::{AppScheduler, BacklightController, PeriodicTask};
