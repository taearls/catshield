//! macOS-specific platform implementations for Cat Shield
//!
//! This module contains macOS-specific code:
//! - FFI bindings for IOKit, CoreGraphics, ApplicationServices, CoreFoundation
//! - Accessibility permission handling
//! - Power management (sleep prevention)
//! - Event tap for input blocking
//! - Login item management (launch at login)
//! - Platform trait implementations

mod accessibility;
mod bindings;
mod event_tap;
mod impls;
mod login_item;
mod power;

pub use accessibility::{
    check_accessibility, check_accessibility_with_prompt, open_accessibility_settings,
};
pub use bindings::*;
pub use event_tap::{disable_event_tap, setup_event_tap, EVENT_TAP, EVENT_TAP_RUN_LOOP_SOURCE};
pub use impls::{MacOSInputBlocker, MacOSPermissionChecker, MacOSPlatform, MacOSPowerManager};
pub use login_item::{
    is_login_item_enabled, register_login_item, set_login_item_enabled, unregister_login_item,
};
pub use power::{allow_sleep, prevent_sleep};
