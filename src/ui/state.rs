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

// Legacy constant aliases for backwards compatibility
// These will be deprecated in favor of the module-based constants
pub const CLOSE_BUTTON_SIZE: f64 = close_button::SIZE;
pub const CLOSE_BUTTON_MARGIN: f64 = close_button::MARGIN;
pub const CLOSE_BUTTON_LABEL_HEIGHT: f64 = close_button::LABEL_HEIGHT;
pub const CLOSE_BUTTON_LABEL_WIDTH: f64 = close_button::LABEL_WIDTH;
pub const HOLD_DURATION_SECS: f64 = close_button::HOLD_DURATION_SECS;
pub const TIMER_INTERVAL_SECS: f64 = animation::INTERVAL_SECS;
pub const NS_SCREEN_SAVER_WINDOW_LEVEL: isize = window_level::SCREEN_SAVER;
pub const TIMER_DISPLAY_HEIGHT: f64 = timer_display::HEIGHT;
pub const TIMER_DISPLAY_WIDTH: f64 = timer_display::WIDTH;
pub const TIMER_DISPLAY_MARGIN: f64 = timer_display::MARGIN;

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

// Legacy aliases for backwards compatibility
pub static MENU_BAR_MODE: &AtomicBool = &shield::MODE_MENU_BAR;
pub static SHIELD_ACTIVE: &AtomicBool = &shield::IS_ACTIVE;
pub static SHIELD_WINDOW: &AtomicPtr<c_void> = &shield::WINDOW;
pub static CLOSE_BUTTON_VIEW: &AtomicPtr<c_void> = &shield::CLOSE_BUTTON;
pub static CLOSE_BUTTON_LABEL_VIEW: &AtomicPtr<c_void> = &shield::CLOSE_BUTTON_LABEL;
pub static CLOSE_BUTTON_TEXT_LABEL: &AtomicPtr<c_void> = &shield::CLOSE_BUTTON_TEXT;
pub static TIMER_DISPLAY_VIEW: &AtomicPtr<c_void> = &shield::TIMER_VIEW;
pub static TIMER_REF: &AtomicPtr<c_void> = &shield::TIMER_REF;
pub static TIMER_HEADER_LABEL: &AtomicPtr<c_void> = &shield::TIMER_HEADER;
pub static TIMER_TIME_LABEL: &AtomicPtr<c_void> = &shield::TIMER_TIME;
pub static TIMER_WARNING_LABEL: &AtomicPtr<c_void> = &shield::TIMER_WARNING;
pub static SLEEP_ASSERTION_ID: &AtomicU64 = &shield::SLEEP_ASSERTION_ID;
pub static HAS_SLEEP_ASSERTION: &AtomicBool = &shield::HAS_SLEEP_ASSERTION;

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

// Legacy aliases for backwards compatibility
pub static START_MENU_ITEM: &AtomicPtr<c_void> = &menu_bar::START_ITEM;
pub static SETTINGS_MENU_ITEM: &AtomicPtr<c_void> = &menu_bar::SETTINGS_ITEM;
pub static ABOUT_MENU_ITEM: &AtomicPtr<c_void> = &menu_bar::ABOUT_ITEM;
pub static MENU_ACTION_HANDLER: &AtomicPtr<c_void> = &menu_bar::ACTION_HANDLER;
pub static HELP_ACTION_HANDLER: &AtomicPtr<c_void> = &menu_bar::HELP_HANDLER;

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

// Legacy aliases for backwards compatibility
pub static SETTINGS_WINDOW: &AtomicPtr<c_void> = &settings::WINDOW;
pub static SETTINGS_WINDOW_DELEGATE: &AtomicPtr<c_void> = &settings::WINDOW_DELEGATE;
pub static SETTINGS_ACTION_HANDLER: &AtomicPtr<c_void> = &settings::ACTION_HANDLER;
pub static SETTINGS_EXIT_KEY_FIELD: &AtomicPtr<c_void> = &settings::EXIT_KEY_FIELD;
pub static SETTINGS_EXIT_KEY_FIELD_DELEGATE: &AtomicPtr<c_void> =
    &settings::EXIT_KEY_FIELD_DELEGATE;
pub static SETTINGS_EXIT_KEY_VALIDATION_LABEL: &AtomicPtr<c_void> = &settings::EXIT_KEY_VALIDATION;
pub static SETTINGS_TIMER_VALUE_FIELD: &AtomicPtr<c_void> = &settings::TIMER_VALUE_FIELD;
pub static SETTINGS_TIMER_UNIT_DROPDOWN: &AtomicPtr<c_void> = &settings::TIMER_UNIT_DROPDOWN;
pub static SETTINGS_TIMER_CHECKBOX: &AtomicPtr<c_void> = &settings::TIMER_CHECKBOX;
pub static SETTINGS_TIMER_VALIDATION_LABEL: &AtomicPtr<c_void> = &settings::TIMER_VALIDATION;
pub static SETTINGS_OPACITY_SLIDER: &AtomicPtr<c_void> = &settings::OPACITY_SLIDER;
pub static SETTINGS_OPACITY_LABEL: &AtomicPtr<c_void> = &settings::OPACITY_LABEL;

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

// Legacy aliases for backwards compatibility
pub static ABOUT_WINDOW: &AtomicPtr<c_void> = &about::WINDOW;
pub static ABOUT_WINDOW_DELEGATE: &AtomicPtr<c_void> = &about::WINDOW_DELEGATE;
pub static ABOUT_ACTION_HANDLER: &AtomicPtr<c_void> = &about::ACTION_HANDLER;

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

    #[test]
    fn test_constants_consistency() {
        // Verify legacy aliases match new module constants
        assert_eq!(CLOSE_BUTTON_SIZE, close_button::SIZE);
        assert_eq!(CLOSE_BUTTON_MARGIN, close_button::MARGIN);
        assert_eq!(CLOSE_BUTTON_LABEL_HEIGHT, close_button::LABEL_HEIGHT);
        assert_eq!(CLOSE_BUTTON_LABEL_WIDTH, close_button::LABEL_WIDTH);
        assert_eq!(HOLD_DURATION_SECS, close_button::HOLD_DURATION_SECS);
        assert_eq!(TIMER_INTERVAL_SECS, animation::INTERVAL_SECS);
        assert_eq!(NS_SCREEN_SAVER_WINDOW_LEVEL, window_level::SCREEN_SAVER);
        assert_eq!(TIMER_DISPLAY_HEIGHT, timer_display::HEIGHT);
        assert_eq!(TIMER_DISPLAY_WIDTH, timer_display::WIDTH);
        assert_eq!(TIMER_DISPLAY_MARGIN, timer_display::MARGIN);
    }
}
