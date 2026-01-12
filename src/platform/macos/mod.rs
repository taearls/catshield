//! macOS-specific platform implementations for Cat Shield
//!
//! This module contains macOS-specific code:
//! - FFI bindings for IOKit, CoreGraphics, ApplicationServices, CoreFoundation
//! - Accessibility permission handling
//! - Power management (sleep prevention)
//! - Event tap for input blocking

mod accessibility;
mod bindings;
mod event_tap;
mod power;

pub use accessibility::{
    check_accessibility, check_accessibility_with_prompt, open_accessibility_settings,
};
pub use bindings::*;
pub use event_tap::{disable_event_tap, setup_event_tap, EVENT_TAP, EVENT_TAP_RUN_LOOP_SOURCE};
pub use power::{allow_sleep, prevent_sleep};
