//! Platform-specific bindings for Cat Shield
//!
//! This module contains:
//! - Platform abstraction traits for cross-platform support
//! - Platform-agnostic types (KeyEvent, Modifiers, SleepAssertion, Rect)
//! - Error types for platform operations
//! - Platform-specific implementations (in subdirectories):
//!   - macOS: FFI bindings, accessibility, power management, event tap

// Platform abstraction layer
pub mod errors;
pub mod traits;
pub mod types;

// Re-export commonly used items from the abstraction layer
pub use errors::{InputBlockError, PermissionError, PowerError, TrayError, WindowError};
pub use traits::{InputBlocker, OverlayWindow, PermissionChecker, PowerManager, SystemTray};
pub use types::{KeyEvent, Modifiers, Rect, SleepAssertion};

// Platform-specific implementations
#[cfg(target_os = "macos")]
pub mod macos;

// Re-export platform-specific items for backward compatibility
#[cfg(target_os = "macos")]
pub use macos::{
    check_accessibility, check_accessibility_with_prompt, disable_event_tap, open_accessibility_settings,
    setup_event_tap, allow_sleep, prevent_sleep, EVENT_TAP, EVENT_TAP_RUN_LOOP_SOURCE,
};
#[cfg(target_os = "macos")]
pub use macos::{
    // IOKit power management
    IOPMAssertionCreateWithName, IOPMAssertionRelease, K_IOPM_ASSERTION_LEVEL_ON,
    // CoreGraphics
    CGEventTapEnable, AXIsProcessTrusted,
    // ApplicationServices
    AXIsProcessTrustedWithOptions, kAXTrustedCheckOptionPrompt,
    // CoreFoundation
    CFMachPortCreateRunLoopSource, CFRunLoopAddSource, CFRunLoopRemoveSource, CFRunLoopGetCurrent,
    CFRunLoopAddTimer, CFRunLoopTimerCreate, CFRunLoopTimerInvalidate, CFAbsoluteTimeGetCurrent,
    CFRunLoopRunInMode, kCFBooleanTrue, CFDictionaryCreate, CFRelease,
    // Process management
    kill,
};
