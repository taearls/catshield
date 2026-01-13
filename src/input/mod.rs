//! Input handling module for Cat Shield
//!
//! This module contains keyboard input handling, including:
//! - Platform-agnostic key representation (`Key` enum)
//! - Platform-specific virtual keycode mappings
//! - Exit key parsing and handling
//! - Allowed keys (keys that pass through the shield)

mod allowed_keys;
mod exit_key;
pub mod keycodes;

pub use allowed_keys::{
    add_preset_keys, clear_allowed_keys, get_allowed_keys, is_key_allowed,
    parse_and_set_allowed_keys, presets, set_allowed_keys, AllowedKey,
};
pub use exit_key::{
    check_exit_key, format_exit_key_display, get_exit_key, set_exit_key, ExitKey, DEFAULT_EXIT_KEY,
    EXIT_KEY_KEYCODE, EXIT_KEY_REQUIRES_CMD, EXIT_KEY_REQUIRES_CTRL, EXIT_KEY_REQUIRES_OPTION,
    EXIT_KEY_REQUIRES_SHIFT,
};
pub use keycodes::{key_from_name, key_to_name, keycode_from_name, keycode_to_name, Key};

// Re-export platform-specific keycode conversion functions
#[cfg(target_os = "macos")]
pub use keycodes::{key_to_keycode, keycode_to_key};
