//! Platform-specific bindings for Cat Shield
//!
//! This module contains:
//! - Platform abstraction traits for cross-platform support
//! - Platform-agnostic types (KeyEvent, Modifiers, SleepAssertion, Rect)
//! - Error types for platform operations
//! - macOS-specific implementations:
//!   - FFI bindings for IOKit, CoreGraphics, etc.
//!   - Accessibility permission handling
//!   - Power management (sleep prevention)
//!   - Event tap for input blocking

// Platform abstraction layer
pub mod errors;
pub mod traits;
pub mod types;

// Re-export commonly used items from the abstraction layer
pub use errors::{InputBlockError, PermissionError, PowerError, TrayError, WindowError};
pub use traits::{InputBlocker, OverlayWindow, PermissionChecker, PowerManager, SystemTray};
pub use types::{KeyEvent, Modifiers, Rect, SleepAssertion};

// macOS-specific implementations (will be moved to platform/macos/ in issue #95)
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
