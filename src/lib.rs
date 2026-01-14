//! Cat Shield - A cat-proof screen overlay for macOS
//!
//! This library provides all the functionality for Cat Shield,
//! organized into the following modules:
//!
//! - [`config`]: Configuration and CLI argument handling
//! - [`input`]: Keyboard input handling and exit key parsing
//! - [`lock`]: Single-instance lock mechanism
//! - [`platform`]: macOS platform bindings (IOKit, CoreGraphics, etc.)
//! - [`shield_core`]: Shared shield activation logic (macOS-only)
//! - [`timer`]: Timer parsing, formatting, and auto-exit state
//! - [`ui`]: User interface components (views, windows, menu bar, macOS-only)
//!
//! Note: This application is currently macOS-only. Windows and Linux support
//! is planned for future releases.

pub mod config;
pub mod input;
pub mod lock;
pub mod logging;
pub mod platform;
#[cfg(target_os = "macos")]
pub mod shield_core;
pub mod timer;
pub mod ui;

// Re-export commonly used items for convenience
#[cfg(target_os = "macos")]
pub use config::Args;
pub use config::Config;
#[cfg(target_os = "macos")]
pub use input::ExitKey;
pub use lock::{acquire_instance_lock, release_instance_lock, LockResult};
pub use timer::{format_duration, parse_duration};
#[cfg(target_os = "macos")]
pub use ui::{activate_shield, deactivate_shield};
