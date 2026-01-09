//! Global UI state management for Cat Shield
//!
//! This module organizes global state into logical groupings for easier maintenance:
//! - `ShieldState`: Core shield overlay state (window, close button, timer display)
//! - `SettingsState`: Settings window UI elements
//! - `AboutState`: About panel state
//! - `MenuBarState`: Menu bar and action handlers
//!
//! All atomic pointers store raw `*mut c_void` for FFI compatibility with objc2.

use std::cell::Cell;
use std::ffi::c_void;
use std::sync::atomic::{AtomicBool, AtomicPtr, AtomicU64};
use std::time::Instant;

// ============================================================================
// UI CONSTANTS
// ============================================================================

/// Close button configuration constants
pub mod close_button {
    /// Button size in points (large, easy-to-see)
    pub const SIZE: f64 = 80.0;
    /// Margin from screen edge
    pub const MARGIN: f64 = 30.0;
    /// Label height below button
    pub const LABEL_HEIGHT: f64 = 30.0;
    /// Label width below button
    pub const LABEL_WIDTH: f64 = 120.0;
    /// Required hold duration in seconds
    pub const HOLD_DURATION_SECS: f64 = 3.0;
}

/// Timer display configuration constants
pub mod timer_display {
    /// Display height in points
    pub const HEIGHT: f64 = 70.0;
    /// Display width in points
    pub const WIDTH: f64 = 260.0;
    /// Margin from screen edge
    pub const MARGIN: f64 = 30.0;
}

/// Animation timing constants
pub mod animation {
    /// Timer interval for 60 FPS smooth animation
    pub const INTERVAL_SECS: f64 = 1.0 / 60.0;
}

/// Window level constants from NSWindow.h
pub mod window_level {
    /// Screen saver window level (appears above everything)
    pub const SCREEN_SAVER: isize = 1000;
}

// ============================================================================
// SHIELD STATE
// ============================================================================

/// State for the shield overlay window and its UI elements
pub mod shield {
    use super::*;

    /// Whether the app is in menu bar mode (stays running after shield deactivates)
    pub static MODE_MENU_BAR: AtomicBool = AtomicBool::new(false);

    /// Whether the shield is currently active
    pub static IS_ACTIVE: AtomicBool = AtomicBool::new(false);

    /// Reference to the shield window for cleanup
    pub static WINDOW: AtomicPtr<c_void> = AtomicPtr::new(std::ptr::null_mut());

    /// Reference to the close button view
    pub static CLOSE_BUTTON: AtomicPtr<c_void> = AtomicPtr::new(std::ptr::null_mut());

    /// Reference to the close button label view
    pub static CLOSE_BUTTON_LABEL: AtomicPtr<c_void> = AtomicPtr::new(std::ptr::null_mut());

    /// Reference to the close button text label (NSTextField)
    pub static CLOSE_BUTTON_TEXT: AtomicPtr<c_void> = AtomicPtr::new(std::ptr::null_mut());

    /// Reference to the timer display view
    pub static TIMER_VIEW: AtomicPtr<c_void> = AtomicPtr::new(std::ptr::null_mut());

    /// Reference to the CFRunLoop timer for animations
    pub static TIMER_REF: AtomicPtr<c_void> = AtomicPtr::new(std::ptr::null_mut());

    /// Timer display header label (NSTextField)
    pub static TIMER_HEADER: AtomicPtr<c_void> = AtomicPtr::new(std::ptr::null_mut());

    /// Timer display time label (NSTextField)
    pub static TIMER_TIME: AtomicPtr<c_void> = AtomicPtr::new(std::ptr::null_mut());

    /// Timer display warning label (NSTextField)
    pub static TIMER_WARNING: AtomicPtr<c_void> = AtomicPtr::new(std::ptr::null_mut());

    /// Sleep assertion ID for power management
    pub static SLEEP_ASSERTION_ID: AtomicU64 = AtomicU64::new(0);

    /// Whether we have an active sleep assertion
    pub static HAS_SLEEP_ASSERTION: AtomicBool = AtomicBool::new(false);
}

// ============================================================================
// MENU BAR STATE
// ============================================================================

/// State for the menu bar and its action handlers
pub mod menu_bar {
    use super::*;

    /// Reference to the "Start Protection" menu item for enabling/disabling
    pub static START_ITEM: AtomicPtr<c_void> = AtomicPtr::new(std::ptr::null_mut());

    /// Reference to the "Settings..." menu item
    pub static SETTINGS_ITEM: AtomicPtr<c_void> = AtomicPtr::new(std::ptr::null_mut());

    /// Reference to the "About Cat Shield" menu item
    pub static ABOUT_ITEM: AtomicPtr<c_void> = AtomicPtr::new(std::ptr::null_mut());

    /// Reference to the menu action handler (keeps it alive)
    pub static ACTION_HANDLER: AtomicPtr<c_void> = AtomicPtr::new(std::ptr::null_mut());

    /// Reference to the help action handler (keeps it alive)
    pub static HELP_HANDLER: AtomicPtr<c_void> = AtomicPtr::new(std::ptr::null_mut());
}

// ============================================================================
// SETTINGS WINDOW STATE
// ============================================================================

/// State for the settings window and its UI elements
pub mod settings {
    use super::*;

    /// Reference to the settings window (prevents multiple instances)
    pub static WINDOW: AtomicPtr<c_void> = AtomicPtr::new(std::ptr::null_mut());

    /// Reference to the settings window delegate
    pub static WINDOW_DELEGATE: AtomicPtr<c_void> = AtomicPtr::new(std::ptr::null_mut());

    /// Reference to the settings action handler (keeps it alive)
    pub static ACTION_HANDLER: AtomicPtr<c_void> = AtomicPtr::new(std::ptr::null_mut());

    /// Exit key text field
    pub static EXIT_KEY_FIELD: AtomicPtr<c_void> = AtomicPtr::new(std::ptr::null_mut());

    /// Exit key field delegate for real-time validation
    pub static EXIT_KEY_FIELD_DELEGATE: AtomicPtr<c_void> = AtomicPtr::new(std::ptr::null_mut());

    /// Exit key validation label
    pub static EXIT_KEY_VALIDATION: AtomicPtr<c_void> = AtomicPtr::new(std::ptr::null_mut());

    /// Timer value text field
    pub static TIMER_VALUE_FIELD: AtomicPtr<c_void> = AtomicPtr::new(std::ptr::null_mut());

    /// Timer unit dropdown
    pub static TIMER_UNIT_DROPDOWN: AtomicPtr<c_void> = AtomicPtr::new(std::ptr::null_mut());

    /// Timer enable checkbox
    pub static TIMER_CHECKBOX: AtomicPtr<c_void> = AtomicPtr::new(std::ptr::null_mut());

    /// Timer validation label
    pub static TIMER_VALIDATION: AtomicPtr<c_void> = AtomicPtr::new(std::ptr::null_mut());

    /// Opacity slider
    pub static OPACITY_SLIDER: AtomicPtr<c_void> = AtomicPtr::new(std::ptr::null_mut());

    /// Opacity value label
    pub static OPACITY_LABEL: AtomicPtr<c_void> = AtomicPtr::new(std::ptr::null_mut());
}

// ============================================================================
// ABOUT WINDOW STATE
// ============================================================================

/// State for the about window
pub mod about {
    use super::*;

    /// Reference to the about window (prevents multiple instances)
    pub static WINDOW: AtomicPtr<c_void> = AtomicPtr::new(std::ptr::null_mut());

    /// Reference to the about window delegate
    pub static WINDOW_DELEGATE: AtomicPtr<c_void> = AtomicPtr::new(std::ptr::null_mut());

    /// Reference to the about action handler (keeps it alive)
    pub static ACTION_HANDLER: AtomicPtr<c_void> = AtomicPtr::new(std::ptr::null_mut());
}

// ============================================================================
// CLOSE BUTTON INTERACTION STATE
// ============================================================================

// Thread-local state for close button mouse interaction
thread_local! {
    // Time when mouse button was pressed down on the close button
    pub static MOUSE_DOWN_TIME: Cell<Option<Instant>> = const { Cell::new(None) };

    // Whether the mouse is currently inside the close button bounds
    pub static IS_MOUSE_INSIDE: Cell<bool> = const { Cell::new(false) };
}

// ============================================================================
// UTILITY FUNCTIONS
// ============================================================================

/// Calculate hold progress as a value from 0.0 to 1.0.
///
/// # Arguments
/// * `elapsed_secs` - Time elapsed since mouse down in seconds
/// * `hold_duration_secs` - Required hold duration in seconds
///
/// # Returns
/// Progress value clamped to range [0.0, 1.0]
#[inline]
pub fn calculate_hold_progress(elapsed_secs: f64, hold_duration_secs: f64) -> f64 {
    (elapsed_secs / hold_duration_secs).min(1.0)
}

/// Check if the hold duration has been met.
///
/// # Arguments
/// * `elapsed_secs` - Time elapsed since mouse down in seconds
/// * `hold_duration_secs` - Required hold duration in seconds
///
/// # Returns
/// `true` if the hold duration has been met or exceeded
#[inline]
pub fn is_hold_complete(elapsed_secs: f64, hold_duration_secs: f64) -> bool {
    elapsed_secs >= hold_duration_secs
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_hold_progress_zero() {
        assert_eq!(calculate_hold_progress(0.0, 3.0), 0.0);
    }

    #[test]
    fn test_calculate_hold_progress_partial() {
        let progress = calculate_hold_progress(1.5, 3.0);
        assert!((progress - 0.5).abs() < f64::EPSILON);
    }

    #[test]
    fn test_calculate_hold_progress_complete() {
        assert_eq!(calculate_hold_progress(3.0, 3.0), 1.0);
    }

    #[test]
    fn test_calculate_hold_progress_exceeds() {
        // Should clamp to 1.0 when elapsed exceeds duration
        assert_eq!(calculate_hold_progress(5.0, 3.0), 1.0);
    }

    #[test]
    fn test_is_hold_complete_false() {
        assert!(!is_hold_complete(2.0, 3.0));
        assert!(!is_hold_complete(2.999, 3.0));
    }

    #[test]
    fn test_is_hold_complete_exact() {
        assert!(is_hold_complete(3.0, 3.0));
    }

    #[test]
    fn test_is_hold_complete_exceeds() {
        assert!(is_hold_complete(5.0, 3.0));
    }
}
