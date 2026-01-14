//! Windows-specific platform implementations for Cat Shield
//!
//! This module contains Windows-specific code:
//! - Low-level keyboard hook for input blocking
//! - Platform trait implementations
//!
//! # Implementation Status
//!
//! - [x] InputBlocker via keyboard hook (WH_KEYBOARD_LL)
//! - [ ] PowerManager (SetThreadExecutionState) - future issue
//! - [ ] PermissionChecker (placeholder) - future issue
//! - [ ] SystemTray (Shell_NotifyIcon) - future issue
//! - [ ] OverlayWindow (CreateWindowEx) - future issue

mod keyboard_hook;

pub use keyboard_hook::{
    clear_allowed_keys, set_allowed_keys, set_exit_key_config, AllowedKeyConfig,
    WindowsInputBlocker,
};
