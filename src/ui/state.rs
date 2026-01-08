//! Global UI state management for Cat Shield

use std::cell::Cell;
use std::ffi::c_void;
use std::sync::atomic::{AtomicBool, AtomicPtr, AtomicU64};
use std::time::Instant;

// Close button configuration constants
pub const CLOSE_BUTTON_SIZE: f64 = 80.0; // Large, easy-to-see button
pub const CLOSE_BUTTON_MARGIN: f64 = 30.0;
pub const CLOSE_BUTTON_LABEL_HEIGHT: f64 = 30.0;
pub const CLOSE_BUTTON_LABEL_WIDTH: f64 = 120.0;
pub const HOLD_DURATION_SECS: f64 = 3.0;
pub const TIMER_INTERVAL_SECS: f64 = 1.0 / 60.0; // 60 FPS for smooth animation

// Window levels from NSWindow.h
pub const NS_SCREEN_SAVER_WINDOW_LEVEL: isize = 1000;

// Timer display configuration
pub const TIMER_DISPLAY_HEIGHT: f64 = 70.0;
pub const TIMER_DISPLAY_WIDTH: f64 = 260.0;
pub const TIMER_DISPLAY_MARGIN: f64 = 30.0;

// Global timer reference for cleanup
pub static TIMER_REF: AtomicPtr<c_void> = AtomicPtr::new(std::ptr::null_mut());

// Global view reference for timer callback
pub static CLOSE_BUTTON_VIEW: AtomicPtr<c_void> = AtomicPtr::new(std::ptr::null_mut());

// Global reference to the timer display view for updates
pub static TIMER_DISPLAY_VIEW: AtomicPtr<c_void> = AtomicPtr::new(std::ptr::null_mut());

// Global references to timer display labels (NSTextField)
pub static TIMER_HEADER_LABEL: AtomicPtr<c_void> = AtomicPtr::new(std::ptr::null_mut());
pub static TIMER_TIME_LABEL: AtomicPtr<c_void> = AtomicPtr::new(std::ptr::null_mut());
pub static TIMER_WARNING_LABEL: AtomicPtr<c_void> = AtomicPtr::new(std::ptr::null_mut());

// Global reference to close button label (NSTextField)
pub static CLOSE_BUTTON_TEXT_LABEL: AtomicPtr<c_void> = AtomicPtr::new(std::ptr::null_mut());

// Global reference to the close button label view for updating during hold
pub static CLOSE_BUTTON_LABEL_VIEW: AtomicPtr<c_void> = AtomicPtr::new(std::ptr::null_mut());

// Shield state for on-demand activation (Issue #17)
// Track whether we're in menu bar mode (app stays running after shield deactivates)
pub static MENU_BAR_MODE: AtomicBool = AtomicBool::new(false);
// Track whether the shield is currently active
pub static SHIELD_ACTIVE: AtomicBool = AtomicBool::new(false);
// Global reference to the shield window for cleanup
pub static SHIELD_WINDOW: AtomicPtr<c_void> = AtomicPtr::new(std::ptr::null_mut());
// Global reference to the "Start Protection" menu item for enabling/disabling
pub static START_MENU_ITEM: AtomicPtr<c_void> = AtomicPtr::new(std::ptr::null_mut());
// Global storage for sleep assertion ID so we can release it on shield deactivate
pub static SLEEP_ASSERTION_ID: AtomicU64 = AtomicU64::new(0);
pub static HAS_SLEEP_ASSERTION: AtomicBool = AtomicBool::new(false);

// Global reference to the menu action handler to keep it alive
pub static MENU_ACTION_HANDLER: AtomicPtr<c_void> = AtomicPtr::new(std::ptr::null_mut());

// Global reference to the settings action handler to keep it alive
pub static SETTINGS_ACTION_HANDLER: AtomicPtr<c_void> = AtomicPtr::new(std::ptr::null_mut());

// Global reference to the settings window to prevent multiple instances
pub static SETTINGS_WINDOW: AtomicPtr<c_void> = AtomicPtr::new(std::ptr::null_mut());

// Global reference to the settings menu item for enabling/disabling
pub static SETTINGS_MENU_ITEM: AtomicPtr<c_void> = AtomicPtr::new(std::ptr::null_mut());

// Settings window UI element references for reading values on save
pub static SETTINGS_EXIT_KEY_FIELD: AtomicPtr<c_void> = AtomicPtr::new(std::ptr::null_mut());
pub static SETTINGS_TIMER_VALUE_FIELD: AtomicPtr<c_void> = AtomicPtr::new(std::ptr::null_mut());
pub static SETTINGS_TIMER_UNIT_DROPDOWN: AtomicPtr<c_void> = AtomicPtr::new(std::ptr::null_mut());
pub static SETTINGS_TIMER_CHECKBOX: AtomicPtr<c_void> = AtomicPtr::new(std::ptr::null_mut());
pub static SETTINGS_OPACITY_SLIDER: AtomicPtr<c_void> = AtomicPtr::new(std::ptr::null_mut());
pub static SETTINGS_OPACITY_LABEL: AtomicPtr<c_void> = AtomicPtr::new(std::ptr::null_mut());
pub static SETTINGS_EXIT_KEY_VALIDATION_LABEL: AtomicPtr<c_void> =
    AtomicPtr::new(std::ptr::null_mut());
pub static SETTINGS_TIMER_VALIDATION_LABEL: AtomicPtr<c_void> =
    AtomicPtr::new(std::ptr::null_mut());
pub static SETTINGS_WINDOW_DELEGATE: AtomicPtr<c_void> = AtomicPtr::new(std::ptr::null_mut());

// Global reference to the about action handler to keep it alive
pub static ABOUT_ACTION_HANDLER: AtomicPtr<c_void> = AtomicPtr::new(std::ptr::null_mut());

// Global reference to the about window to prevent multiple instances
pub static ABOUT_WINDOW: AtomicPtr<c_void> = AtomicPtr::new(std::ptr::null_mut());

// Global reference to the about menu item for enabling/disabling
pub static ABOUT_MENU_ITEM: AtomicPtr<c_void> = AtomicPtr::new(std::ptr::null_mut());

// Global reference to the about window delegate
pub static ABOUT_WINDOW_DELEGATE: AtomicPtr<c_void> = AtomicPtr::new(std::ptr::null_mut());

// Close button state stored in thread-local for the view
thread_local! {
    pub static MOUSE_DOWN_TIME: Cell<Option<Instant>> = const { Cell::new(None) };
    pub static IS_MOUSE_INSIDE: Cell<bool> = const { Cell::new(false) };
}

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
