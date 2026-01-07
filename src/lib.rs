//! Cat Shield - A cat-proof screen overlay for macOS
//!
//! This library provides all the functionality for Cat Shield,
//! organized into the following modules:
//!
//! - [`config`]: Configuration and CLI argument handling
//! - [`input`]: Keyboard input handling and exit key parsing
//! - [`lock`]: Single-instance lock mechanism
//! - [`platform`]: macOS platform bindings (IOKit, CoreGraphics, etc.)
//! - [`timer`]: Timer parsing, formatting, and auto-exit state
//! - [`ui`]: User interface components (views, windows, menu bar)

pub mod config;
pub mod input;
pub mod lock;
pub mod platform;
pub mod timer;
pub mod ui;

// Re-export commonly used items for convenience
pub use config::{Args, Config};
pub use input::ExitKey;
pub use lock::{acquire_instance_lock, release_instance_lock, LockResult};
pub use timer::{format_duration, parse_duration};
pub use ui::{activate_shield, deactivate_shield};
