//! UI module for Cat Shield
//!
//! This module contains all UI-related functionality:
//! - Global UI state management
//! - Custom NSView classes (close button, timer display)
//! - Window creation (shield overlay, settings)
//! - Menu bar setup and handlers
//! - iced window handlers (for launching iced-based Settings/About windows)
//!
//! Note: Most UI functionality is macOS-only. Windows and Linux support
//! is planned for future releases.

#[cfg(target_os = "macos")]
pub mod helpers;
#[cfg(target_os = "macos")]
pub mod iced_handlers;
#[cfg(target_os = "macos")]
pub mod menu_bar;
#[cfg(target_os = "macos")]
pub mod ptr_helper;
#[cfg(target_os = "macos")]
pub mod shield;
#[cfg(target_os = "macos")]
pub mod state;
#[cfg(target_os = "macos")]
pub mod views;
#[cfg(target_os = "macos")]
pub mod windows;

#[cfg(target_os = "macos")]
pub use helpers::create_label;
#[cfg(target_os = "macos")]
pub use iced_handlers::{IcedAboutHandler, IcedSettingsHandler};
#[cfg(target_os = "macos")]
pub use ptr_helper::{with_ptr, with_ptr_mut, with_ptr_void, with_raw_ptr};
#[cfg(target_os = "macos")]
pub use shield::{activate_shield, deactivate_shield, timer_callback};
#[cfg(target_os = "macos")]
pub use state::*;
