//! Configuration module for Cat Shield
//!
//! This module handles:
//! - Config struct definition and serialization
//! - Config file I/O (loading and saving)
//! - CLI argument parsing (macOS-only, depends on ExitKey)

// CLI arguments module is macOS-only because it depends on ExitKey
#[cfg(target_os = "macos")]
mod args;
mod file;
mod types;

#[cfg(target_os = "macos")]
pub use args::{has_immediate_start_args, Args};
pub use file::{get_current_config, set_current_config};
pub use types::{Config, DEFAULT_OVERLAY_OPACITY, MAX_OVERLAY_OPACITY, MIN_OVERLAY_OPACITY};
