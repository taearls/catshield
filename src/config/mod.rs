//! Configuration module for Cat Shield
//!
//! This module handles:
//! - Config struct definition and serialization
//! - Config file I/O (loading and saving)
//! - CLI argument parsing (platform-specific)

// CLI arguments module for macOS (depends on ExitKey)
#[cfg(target_os = "macos")]
mod args;

// CLI arguments module for Linux (includes wayland-mode flag)
#[cfg(target_os = "linux")]
mod args_linux;

mod file;
mod types;

#[cfg(target_os = "macos")]
pub use args::{has_immediate_start_args, Args};

#[cfg(target_os = "linux")]
pub use args_linux::{has_immediate_start_args_linux, LinuxArgs};

pub use file::{get_current_config, set_current_config};
pub use types::{Config, DEFAULT_OVERLAY_OPACITY, MAX_OVERLAY_OPACITY, MIN_OVERLAY_OPACITY};
