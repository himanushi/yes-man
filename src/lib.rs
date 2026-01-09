//! Yes Man - M5Stack Core S3
//!
//! A Rust implementation of Yes Man from Fallout: New Vegas
//! running on M5Stack Core S3.
//!
//! # Architecture
//!
//! ```text
//! ┌─────────────────────────────────────┐
//! │         Application Layer           │  ← Yes Man logic
//! │              (app/)                 │
//! ├─────────────────────────────────────┤
//! │          Graphics Layer             │  ← Rendering
//! │           (graphics/)               │
//! ├─────────────────────────────────────┤
//! │      Hardware Abstraction Layer     │  ← Traits & drivers
//! │           (hardware/)               │
//! ├─────────────────────────────────────┤
//! │       Configuration Layer           │  ← Constants & config
//! │            (config/)                │
//! └─────────────────────────────────────┘
//! ```
//!
//! # Design Patterns
//!
//! - **Facade**: `Board` simplifies hardware initialization
//! - **Strategy**: State-based rendering (future: face animations)
//! - **Dependency Injection**: Traits enable testing with mocks

pub mod config;
pub mod error;
pub mod graphics;

// ESP32 専用モジュール
#[cfg(target_arch = "xtensa")]
pub mod hardware;
#[cfg(target_arch = "xtensa")]
pub mod app;

pub use error::{YesManError, Result};

#[cfg(target_arch = "xtensa")]
pub use app::YesManApp;
