//! Cat Shield - A cat-proof screen overlay for macOS, Windows, and Linux
//!
//! This library provides all the functionality for Cat Shield,
//! organized into the following modules:
//!
//! - [`cat_animation`]: Animated cat companion for the shield overlay
//! - [`config`]: Configuration and CLI argument handling
//! - [`input`]: Keyboard input handling and exit key parsing
//! - [`lock`]: Single-instance lock mechanism
//! - [`platform`]: Platform-specific bindings (macOS, Windows, Linux)
//! - [`shield_core`]: Shared shield activation logic (macOS-only)
//! - [`timer`]: Timer parsing, formatting, and auto-exit state
//! - [`ui`]: User interface components (views, windows, menu bar)
//!
//! # Platform Support
//!
//! - **macOS**: Full support with global keyboard/mouse blocking
//! - **Windows**: Full support with keyboard hook-based blocking
//! - **Linux (X11)**: Full support via XGrabKeyboard
//! - **Linux (Wayland)**: Keyboard capture when focused (pointer can escape)

pub mod cat_animation;
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
#[cfg(target_os = "linux")]
pub use config::LinuxArgs;
#[cfg(target_os = "macos")]
pub use input::ExitKey;
pub use lock::{acquire_instance_lock, release_instance_lock, LockResult};
pub use timer::{format_duration, parse_duration};
#[cfg(target_os = "macos")]
pub use ui::{activate_shield, deactivate_shield};
