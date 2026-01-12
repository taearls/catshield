//! Global UI state management for Cat Shield
//!
//! This module organizes global state into logical groupings for easier maintenance:
//! - `ShieldState`: Core shield overlay state (window, close button, timer display)
//! - `SettingsState`: Settings window UI elements
//! - `AboutState`: About panel state
//! - `MenuBarState`: Menu bar and action handlers
//!
//! All atomic pointers store raw `*mut c_void` for FFI compatibility with objc2.

use std::cell::{Cell, RefCell};
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

    /// Timer field delegate for real-time validation
    pub static TIMER_FIELD_DELEGATE: AtomicPtr<c_void> = AtomicPtr::new(std::ptr::null_mut());

    /// Timer validation label
    pub static TIMER_VALIDATION: AtomicPtr<c_void> = AtomicPtr::new(std::ptr::null_mut());

    /// Opacity slider
    pub static OPACITY_SLIDER: AtomicPtr<c_void> = AtomicPtr::new(std::ptr::null_mut());

    /// Opacity value label
    pub static OPACITY_LABEL: AtomicPtr<c_void> = AtomicPtr::new(std::ptr::null_mut());

    /// Allowed keys text view (displays list of allowed keys)
    pub static ALLOWED_KEYS_VIEW: AtomicPtr<c_void> = AtomicPtr::new(std::ptr::null_mut());

    /// Allowed keys scroll view (wraps the text view)
    pub static ALLOWED_KEYS_SCROLL: AtomicPtr<c_void> = AtomicPtr::new(std::ptr::null_mut());

    /// Add key text field (for entering new keys)
    pub static ADD_KEY_FIELD: AtomicPtr<c_void> = AtomicPtr::new(std::ptr::null_mut());

    /// Add key validation label
    pub static ADD_KEY_VALIDATION: AtomicPtr<c_void> = AtomicPtr::new(std::ptr::null_mut());

    /// Action feedback label (auto-disappears after timeout)
    pub static ACTION_FEEDBACK_LABEL: AtomicPtr<c_void> = AtomicPtr::new(std::ptr::null_mut());

    /// Add key field delegate for real-time validation
    pub static ADD_KEY_FIELD_DELEGATE: AtomicPtr<c_void> = AtomicPtr::new(std::ptr::null_mut());

    /// Remove key button (enabled when a key is selected)
    pub static REMOVE_KEY_BUTTON: AtomicPtr<c_void> = AtomicPtr::new(std::ptr::null_mut());

    /// Add key button (enabled when input is valid)
    pub static ADD_KEY_BUTTON: AtomicPtr<c_void> = AtomicPtr::new(std::ptr::null_mut());

    // Thread-local state for pending allowed keys changes.
    // This allows Cancel to discard changes without affecting the global config.
    thread_local! {
        /// Pending allowed keys draft state (not yet saved to config).
        /// `None` means no draft is active (settings window closed).
        /// `Some(vec)` means a draft is active (initialized on window open, modified during editing).
        /// Access only through the helper functions below; this is intentionally private.
        static PENDING_ALLOWED_KEYS: RefCell<Option<Vec<String>>> = const { RefCell::new(None) };

        /// Currently selected allowed key index for deletion.
        /// `None` means no selection.
        /// `Some(index)` means the key at that index is selected.
        static SELECTED_ALLOWED_KEY_INDEX: Cell<Option<isize>> = const { Cell::new(None) };
    }

    /// Initialize pending allowed keys from current config.
    /// Called when settings window opens.
    pub fn init_pending_allowed_keys(keys: Option<Vec<String>>) {
        PENDING_ALLOWED_KEYS.with(|pending| {
            *pending.borrow_mut() = keys;
        });
    }

    /// Get the current pending allowed keys list.
    /// Returns a clone of the current pending state.
    pub fn get_pending_allowed_keys() -> Option<Vec<String>> {
        PENDING_ALLOWED_KEYS.with(|pending| pending.borrow().clone())
    }

    /// Set the pending allowed keys list.
    pub fn set_pending_allowed_keys(keys: Option<Vec<String>>) {
        PENDING_ALLOWED_KEYS.with(|pending| {
            *pending.borrow_mut() = keys;
        });
    }

    /// Clear pending allowed keys state (discard changes).
    /// Called when settings window is closed/cancelled.
    pub fn clear_pending_allowed_keys() {
        PENDING_ALLOWED_KEYS.with(|pending| {
            *pending.borrow_mut() = None;
        });
    }

    /// Get the currently selected allowed key index.
    pub fn get_selected_allowed_key_index() -> Option<isize> {
        SELECTED_ALLOWED_KEY_INDEX.with(|idx| idx.get())
    }

    /// Set the selected allowed key index.
    pub fn set_selected_allowed_key_index(index: Option<isize>) {
        SELECTED_ALLOWED_KEY_INDEX.with(|idx| idx.set(index));
    }

    /// Clear the selected allowed key index.
    /// Called when settings window is closed or after removing a key.
    pub fn clear_selected_allowed_key_index() {
        SELECTED_ALLOWED_KEY_INDEX.with(|idx| idx.set(None));
    }
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

    // ========================================================================
    // Hold Progress Function Tests
    // ========================================================================

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

    // ========================================================================
    // UI Constants Validation Tests
    // ========================================================================
    //
    // These tests use const blocks to validate UI constants at compile time.
    // If a constant violates its constraint, compilation will fail immediately.

    #[test]
    fn test_close_button_size_positive() {
        const { assert!(close_button::SIZE > 0.0) };
    }

    #[test]
    fn test_close_button_margin_positive() {
        const { assert!(close_button::MARGIN > 0.0) };
    }

    #[test]
    fn test_close_button_label_dimensions_positive() {
        const { assert!(close_button::LABEL_HEIGHT > 0.0) };
        const { assert!(close_button::LABEL_WIDTH > 0.0) };
    }

    #[test]
    fn test_close_button_hold_duration_reasonable() {
        // Hold duration should be between 1 and 10 seconds
        const { assert!(close_button::HOLD_DURATION_SECS >= 1.0) };
        const { assert!(close_button::HOLD_DURATION_SECS <= 10.0) };
    }

    #[test]
    fn test_timer_display_dimensions_positive() {
        const { assert!(timer_display::WIDTH > 0.0) };
        const { assert!(timer_display::HEIGHT > 0.0) };
    }

    #[test]
    fn test_timer_display_width_greater_than_margins() {
        const { assert!(timer_display::WIDTH > timer_display::MARGIN * 2.0) };
    }

    #[test]
    fn test_animation_interval_is_60fps() {
        // 60 FPS = ~0.0167 seconds per frame
        let expected_interval = 1.0 / 60.0;
        assert!((animation::INTERVAL_SECS - expected_interval).abs() < 0.001);
    }

    #[test]
    fn test_screen_saver_window_level_valid() {
        const { assert!(window_level::SCREEN_SAVER > 0) };
    }

    // ========================================================================
    // Pending Allowed Keys Tests
    // ========================================================================

    #[test]
    fn test_pending_allowed_keys_init_with_some() {
        settings::init_pending_allowed_keys(Some(vec!["Cmd+Space".to_string()]));
        let keys = settings::get_pending_allowed_keys();
        assert_eq!(keys, Some(vec!["Cmd+Space".to_string()]));
        settings::clear_pending_allowed_keys();
    }

    #[test]
    fn test_pending_allowed_keys_init_with_none() {
        settings::init_pending_allowed_keys(None);
        let keys = settings::get_pending_allowed_keys();
        assert!(keys.is_none());
        settings::clear_pending_allowed_keys();
    }

    #[test]
    fn test_pending_allowed_keys_set_updates_state() {
        settings::init_pending_allowed_keys(Some(vec!["F11".to_string()]));
        settings::set_pending_allowed_keys(Some(vec!["F11".to_string(), "F12".to_string()]));
        let keys = settings::get_pending_allowed_keys();
        assert_eq!(keys, Some(vec!["F11".to_string(), "F12".to_string()]));
        settings::clear_pending_allowed_keys();
    }

    #[test]
    fn test_pending_allowed_keys_clear_discards_state() {
        settings::init_pending_allowed_keys(Some(vec!["Cmd+Q".to_string()]));
        settings::clear_pending_allowed_keys();
        let keys = settings::get_pending_allowed_keys();
        assert!(keys.is_none());
    }

    #[test]
    fn test_pending_allowed_keys_set_empty_vec() {
        settings::init_pending_allowed_keys(Some(vec!["Key1".to_string()]));
        settings::set_pending_allowed_keys(Some(Vec::new()));
        let keys = settings::get_pending_allowed_keys();
        assert_eq!(keys, Some(Vec::new()));
        settings::clear_pending_allowed_keys();
    }

    #[test]
    fn test_pending_allowed_keys_multiple_operations() {
        // Simulate typical workflow: init -> add -> add -> clear all -> save workflow
        settings::init_pending_allowed_keys(Some(vec!["Existing".to_string()]));

        // Add another key
        let mut keys = settings::get_pending_allowed_keys().unwrap_or_default();
        keys.push("NewKey".to_string());
        settings::set_pending_allowed_keys(Some(keys));

        let result = settings::get_pending_allowed_keys();
        assert_eq!(
            result,
            Some(vec!["Existing".to_string(), "NewKey".to_string()])
        );

        // Clear all
        settings::set_pending_allowed_keys(Some(Vec::new()));
        assert_eq!(settings::get_pending_allowed_keys(), Some(Vec::new()));

        // Final cleanup
        settings::clear_pending_allowed_keys();
    }

    // ========================================================================
    // Selected Allowed Key Index Tests
    // ========================================================================

    #[test]
    fn test_selected_allowed_key_index_default_none() {
        // Start with no selection
        settings::clear_selected_allowed_key_index();
        assert_eq!(settings::get_selected_allowed_key_index(), None);
    }

    #[test]
    fn test_selected_allowed_key_index_set_and_get() {
        settings::set_selected_allowed_key_index(Some(2));
        assert_eq!(settings::get_selected_allowed_key_index(), Some(2));
        settings::clear_selected_allowed_key_index();
    }

    #[test]
    fn test_selected_allowed_key_index_clear() {
        settings::set_selected_allowed_key_index(Some(5));
        settings::clear_selected_allowed_key_index();
        assert_eq!(settings::get_selected_allowed_key_index(), None);
    }

    #[test]
    fn test_selected_allowed_key_index_toggle() {
        // Simulate toggle behavior: select -> deselect
        settings::set_selected_allowed_key_index(Some(3));
        assert_eq!(settings::get_selected_allowed_key_index(), Some(3));

        // Deselect by setting to None
        settings::set_selected_allowed_key_index(None);
        assert_eq!(settings::get_selected_allowed_key_index(), None);

        settings::clear_selected_allowed_key_index();
    }

    #[test]
    fn test_selected_allowed_key_index_change_selection() {
        // Select row 1
        settings::set_selected_allowed_key_index(Some(1));
        assert_eq!(settings::get_selected_allowed_key_index(), Some(1));

        // Select row 3 (should replace row 1)
        settings::set_selected_allowed_key_index(Some(3));
        assert_eq!(settings::get_selected_allowed_key_index(), Some(3));

        settings::clear_selected_allowed_key_index();
    }
}
