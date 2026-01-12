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
// High-level APIs (accessibility, power, event tap)
#[cfg(target_os = "macos")]
pub use macos::{
    allow_sleep, check_accessibility, check_accessibility_with_prompt, disable_event_tap,
    open_accessibility_settings, prevent_sleep, setup_event_tap, EVENT_TAP,
    EVENT_TAP_RUN_LOOP_SOURCE,
};

// Low-level FFI bindings (grouped by framework)
#[cfg(target_os = "macos")]
pub use macos::{
    // ApplicationServices - Accessibility APIs
    kAXTrustedCheckOptionPrompt,
    // CoreFoundation - Run loop and utility functions
    kCFBooleanTrue,
    // libc - Process management
    kill,
    AXIsProcessTrusted,
    AXIsProcessTrustedWithOptions,
    CFAbsoluteTimeGetCurrent,
    CFDictionaryCreate,
    CFMachPortCreateRunLoopSource,
    CFRelease,
    CFRunLoopAddSource,
    CFRunLoopAddTimer,
    CFRunLoopGetCurrent,
    CFRunLoopRemoveSource,
    CFRunLoopRunInMode,
    CFRunLoopTimerCreate,
    CFRunLoopTimerInvalidate,
    // CoreGraphics - Event tap control
    CGEventTapEnable,
    // IOKit - Power management
    IOPMAssertionCreateWithName,
    IOPMAssertionRelease,
    K_IOPM_ASSERTION_LEVEL_ON,
};
