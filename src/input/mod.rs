//! Input handling module for Cat Shield
//!
//! This module contains keyboard input handling, including:
//! - Virtual keycode mappings for macOS
//! - Exit key parsing and handling

mod exit_key;
mod keycodes;

pub use exit_key::{
    check_exit_key, format_exit_key_display, get_exit_key, set_exit_key, ExitKey,
    DEFAULT_EXIT_KEY, EXIT_KEY_KEYCODE, EXIT_KEY_REQUIRES_CMD, EXIT_KEY_REQUIRES_CTRL,
    EXIT_KEY_REQUIRES_OPTION, EXIT_KEY_REQUIRES_SHIFT,
};
pub use keycodes::{keycode_from_name, keycode_to_name};
