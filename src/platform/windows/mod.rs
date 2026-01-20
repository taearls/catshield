//! Windows-specific platform implementations for Cat Shield
//!
//! This module contains Windows-specific code:
//! - Low-level keyboard hook for input blocking
//! - Power management to prevent system sleep
//! - System tray icon and context menu
//! - Fullscreen overlay window
//! - Platform trait implementations
//!
//! # Implementation Status
//!
//! - [x] InputBlocker via keyboard hook (WH_KEYBOARD_LL)
//! - [x] PowerManager (SetThreadExecutionState)
//! - [x] SystemTray (Shell_NotifyIcon)
//! - [x] OverlayWindow (CreateWindowEx with layered windows)
//! - [ ] PermissionChecker (placeholder) - future issue

mod keyboard_hook;
mod overlay;
mod power;
mod tray;

pub use keyboard_hook::{
    clear_allowed_keys, set_allowed_keys, set_exit_key_config, AllowedKeyConfig,
    WindowsInputBlocker,
};
pub use overlay::{CloseCallback, WindowsOverlayWindow};
pub use power::WindowsPowerManager;
pub use tray::{MenuCallback, TrayMenuAction, WindowsSystemTray};
