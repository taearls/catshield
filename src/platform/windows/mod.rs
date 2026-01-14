//! Windows-specific platform implementations for Cat Shield
//!
//! This module contains Windows-specific code:
//! - Low-level keyboard hook for input blocking
//! - Power management to prevent system sleep
//! - Platform trait implementations
//!
//! # Implementation Status
//!
//! - [x] InputBlocker via keyboard hook (WH_KEYBOARD_LL)
//! - [x] PowerManager (SetThreadExecutionState)
//! - [ ] PermissionChecker (placeholder) - future issue
//! - [ ] SystemTray (Shell_NotifyIcon) - future issue
//! - [ ] OverlayWindow (CreateWindowEx) - future issue

mod keyboard_hook;
mod power;

pub use keyboard_hook::{
    clear_allowed_keys, set_allowed_keys, set_exit_key_config, AllowedKeyConfig,
    WindowsInputBlocker,
};
pub use power::WindowsPowerManager;
