//! Platform-specific bindings for Cat Shield
//!
//! This module contains:
//! - Platform abstraction traits for cross-platform support
//! - Platform-agnostic types (KeyEvent, Modifiers, SleepAssertion, Rect)
//! - Error types for platform operations
//! - Platform-specific implementations (in subdirectories):
//!   - macOS: FFI bindings, accessibility, power management, event tap
//!   - Windows: keyboard hook, input blocking
//!   - Linux: X11 keyboard grab, input blocking

// Platform abstraction layer
pub mod errors;
pub mod traits;
pub mod types;

// Integration tests
#[cfg(test)]
mod integration_tests;

// Re-export commonly used items from the abstraction layer
pub use errors::{InputBlockError, PermissionError, PowerError, TrayError, WindowError};
pub use traits::{InputBlocker, OverlayWindow, PermissionChecker, PowerManager, SystemTray};
pub use types::{KeyEvent, Modifiers, Rect, SleepAssertion};

// Platform-specific implementations
#[cfg(target_os = "macos")]
pub mod macos;

#[cfg(target_os = "windows")]
pub mod windows;

#[cfg(target_os = "linux")]
pub mod linux;

// Re-export platform-specific items for backward compatibility
// High-level APIs (accessibility, power, event tap)
#[cfg(target_os = "macos")]
pub use macos::{
    allow_sleep, check_accessibility, check_accessibility_with_prompt, disable_event_tap,
    open_accessibility_settings, prevent_sleep, setup_event_tap, EVENT_TAP,
    EVENT_TAP_RUN_LOOP_SOURCE,
};

// Platform trait implementations
#[cfg(target_os = "macos")]
pub use macos::{MacOSInputBlocker, MacOSPermissionChecker, MacOSPlatform, MacOSPowerManager};

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

// Windows platform trait implementations and helpers
#[cfg(target_os = "windows")]
pub use windows::{
    clear_allowed_keys, set_allowed_keys, set_exit_key_config, AllowedKeyConfig, CloseCallback,
    MenuCallback, TrayMenuAction, WindowsInputBlocker, WindowsOverlayWindow, WindowsPowerManager,
    WindowsSystemTray,
};

// Linux platform trait implementations and helpers
#[cfg(target_os = "linux")]
pub use linux::{
    allow_keyboard_event, block_keyboard_event, clear_allowed_keys, detect_display_server,
    display_info_summary, is_keyboard_shortcuts_inhibit_available,
    is_native_wayland_overlay_available, is_wayland_session, is_wlr_layer_shell_available,
    is_x11_session, print_wayland_warning, set_allowed_keys, set_exit_key_config, should_proceed,
    AllowedKeyConfig, DisplayRecommendation, DisplayServer, DisplayServerInfo, LinuxOverlayWindow,
    LinuxPowerManager, LinuxSystemTray, MenuCallback, ProcessResult, TrayMenuAction,
    WaylandCapabilities, WaylandMode, WaylandOverlayWindow, X11InputBlocker,
};
