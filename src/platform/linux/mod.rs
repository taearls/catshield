//! Linux-specific platform implementations for Cat Shield
//!
//! This module contains Linux-specific code:
//! - X11 keyboard grab for input blocking
//! - D-Bus power management for sleep prevention
//! - Platform trait implementations
//!
//! # Implementation Status
//!
//! - [x] InputBlocker via X11 keyboard grab (XGrabKeyboard)
//! - [ ] InputBlocker via Wayland (keyboard-shortcuts-inhibit) - future issue #131
//! - [x] PowerManager via D-Bus (FreeDesktop ScreenSaver, GNOME SessionManager)
//! - [ ] PermissionChecker (placeholder) - future issue
//! - [ ] SystemTray via libappindicator - future issue #109
//! - [ ] OverlayWindow (X11 + Wayland) - future issue #110

mod power;
mod x11_keyboard;

pub use power::LinuxPowerManager;
pub use x11_keyboard::{
    allow_keyboard_event, block_keyboard_event, clear_allowed_keys, set_allowed_keys,
    set_exit_key_config, AllowedKeyConfig, ProcessResult, X11InputBlocker,
};
