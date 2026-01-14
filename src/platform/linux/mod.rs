//! Linux-specific platform implementations for Cat Shield
//!
//! This module contains Linux-specific code:
//! - X11 keyboard grab for input blocking
//! - Platform trait implementations
//!
//! # Implementation Status
//!
//! - [x] InputBlocker via X11 keyboard grab (XGrabKeyboard)
//! - [ ] InputBlocker via Wayland (keyboard-shortcuts-inhibit) - future issue #131
//! - [ ] PowerManager via D-Bus - future issue #108
//! - [ ] PermissionChecker (placeholder) - future issue
//! - [ ] SystemTray via libappindicator - future issue #109
//! - [ ] OverlayWindow (X11 + Wayland) - future issue #110

mod x11_keyboard;

pub use x11_keyboard::{
    allow_keyboard_event, block_keyboard_event, clear_allowed_keys, set_allowed_keys,
    set_exit_key_config, AllowedKeyConfig, ProcessResult, X11InputBlocker,
};
