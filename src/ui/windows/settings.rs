//! Settings window for Cat Shield

use crate::config::{
    get_current_config, set_current_config, DEFAULT_OVERLAY_OPACITY, MAX_OVERLAY_OPACITY,
    MIN_OVERLAY_OPACITY,
};
use crate::input::{set_exit_key, ExitKey, DEFAULT_EXIT_KEY};
use crate::timer::{parse_duration, parse_timer_value_and_unit};
use crate::ui::helpers::create_label;
use crate::ui::ptr_helper::{with_ptr, with_ptr_void};
use crate::ui::state::{menu_bar, settings};
use objc2::rc::Retained;
use objc2::runtime::ProtocolObject;
use objc2::{define_class, msg_send, MainThreadOnly};
use objc2_app_kit::{
    NSApplication, NSBackingStoreType, NSButton, NSButtonType, NSColor, NSControlSize,
    NSControlStateValueOff, NSControlStateValueOn, NSControlTextEditingDelegate, NSFont,
    NSMenuItem, NSPanel, NSPopUpButton, NSScreen, NSSlider, NSTextAlignment, NSTextField,
    NSTextFieldDelegate, NSWindowDelegate, NSWindowStyleMask,
};
use objc2_core_foundation::{CGFloat, CGPoint, CGRect, CGSize};
use objc2_foundation::{
    ns_string, MainThreadMarker, NSNotification, NSObject, NSObjectProtocol, NSString,
};
use std::ffi::c_void;
use std::sync::atomic::{AtomicPtr, Ordering};

/// Empty ivars for the SettingsActionHandler
pub struct SettingsActionHandlerIvars {}

/// Empty ivars for the SettingsWindowDelegate
pub struct SettingsWindowDelegateIvars {}

/// Empty ivars for the ExitKeyFieldDelegate
pub struct ExitKeyFieldDelegateIvars {}

/// Empty ivars for the TimerFieldDelegate
pub struct TimerFieldDelegateIvars {}

// Delegate for exit key text field to handle real-time text changes
define_class!(
    #[unsafe(super(NSObject))]
    #[thread_kind = MainThreadOnly]
    #[name = "ExitKeyFieldDelegate"]
    #[ivars = ExitKeyFieldDelegateIvars]
    pub struct ExitKeyFieldDelegate;

    unsafe impl NSObjectProtocol for ExitKeyFieldDelegate {}

    // NSTextFieldDelegate is required for setDelegate on NSTextField
    unsafe impl NSTextFieldDelegate for ExitKeyFieldDelegate {}

    unsafe impl NSControlTextEditingDelegate for ExitKeyFieldDelegate {
        /// Called when text changes in the control (real-time, on every keystroke)
        #[unsafe(method(controlTextDidChange:))]
        fn control_text_did_change(&self, _notification: &NSNotification) {
            // Read the current value from the exit key field and validate
            unsafe {
                with_ptr_void::<NSTextField, _>(&settings::EXIT_KEY_FIELD, |field| {
                    let value = field.stringValue().to_string();
                    validate_exit_key_realtime(&value);
                });
            }
        }
    }
);

impl ExitKeyFieldDelegate {
    pub fn new(mtm: MainThreadMarker) -> Retained<Self> {
        let this = mtm.alloc::<ExitKeyFieldDelegate>();
        let this = this.set_ivars(ExitKeyFieldDelegateIvars {});
        unsafe { msg_send![super(this), init] }
    }
}

// Delegate for timer value text field to handle real-time text changes
define_class!(
    #[unsafe(super(NSObject))]
    #[thread_kind = MainThreadOnly]
    #[name = "TimerFieldDelegate"]
    #[ivars = TimerFieldDelegateIvars]
    pub struct TimerFieldDelegate;

    unsafe impl NSObjectProtocol for TimerFieldDelegate {}

    // NSTextFieldDelegate is required for setDelegate on NSTextField
    unsafe impl NSTextFieldDelegate for TimerFieldDelegate {}

    unsafe impl NSControlTextEditingDelegate for TimerFieldDelegate {
        /// Called when text changes in the control (real-time, on every keystroke)
        #[unsafe(method(controlTextDidChange:))]
        fn control_text_did_change(&self, _notification: &NSNotification) {
            // Only validate if the timer checkbox is enabled
            let should_validate = unsafe {
                with_ptr::<NSButton, _, _>(&settings::TIMER_CHECKBOX, |checkbox| {
                    checkbox.state() == NSControlStateValueOn
                })
            };

            if should_validate != Some(true) {
                return; // Don't validate when checkbox is unchecked or not found
            }

            // Read the current value from the timer field and validate
            unsafe {
                with_ptr_void::<NSTextField, _>(&settings::TIMER_VALUE_FIELD, |field| {
                    let value = field.stringValue().to_string();
                    validate_timer_realtime(&value);
                });
            }
        }
    }
);

impl TimerFieldDelegate {
    pub fn new(mtm: MainThreadMarker) -> Retained<Self> {
        let this = mtm.alloc::<TimerFieldDelegate>();
        let this = this.set_ivars(TimerFieldDelegateIvars {});
        unsafe { msg_send![super(this), init] }
    }
}

// Window delegate to handle window close events
define_class!(
    #[unsafe(super(NSObject))]
    #[thread_kind = MainThreadOnly]
    #[name = "SettingsWindowDelegate"]
    #[ivars = SettingsWindowDelegateIvars]
    pub struct SettingsWindowDelegate;

    unsafe impl NSObjectProtocol for SettingsWindowDelegate {}

    unsafe impl NSWindowDelegate for SettingsWindowDelegate {
        /// Called when the window is about to close (including via X button)
        #[unsafe(method(windowWillClose:))]
        fn window_will_close(&self, _notification: &NSNotification) {
            cleanup_settings_window_references();
        }
    }
);

impl SettingsWindowDelegate {
    pub fn new(mtm: MainThreadMarker) -> Retained<Self> {
        let this = mtm.alloc::<SettingsWindowDelegate>();
        let this = this.set_ivars(SettingsWindowDelegateIvars {});
        unsafe { msg_send![super(this), init] }
    }
}

// Settings action handler for menu items and buttons
define_class!(
    #[unsafe(super(NSObject))]
    #[name = "SettingsActionHandler"]
    #[ivars = SettingsActionHandlerIvars]
    pub struct SettingsActionHandler;

    impl SettingsActionHandler {
        /// Action method called when "Settings..." menu item is clicked
        #[unsafe(method(showSettings:))]
        unsafe fn show_settings(&self, _sender: Option<&NSMenuItem>) {
            if let Some(mtm) = MainThreadMarker::new() {
                show_settings_window(mtm);
            }
        }

        /// Action method called when "Save" button is clicked
        #[unsafe(method(saveSettings:))]
        unsafe fn save_settings(&self, _sender: Option<&NSButton>) {
            save_settings_from_window();
        }

        /// Action method called when "Cancel" button is clicked
        #[unsafe(method(cancelSettings:))]
        unsafe fn cancel_settings(&self, _sender: Option<&NSButton>) {
            close_settings_window();
        }

        /// Action method called when opacity slider value changes
        #[unsafe(method(opacityChanged:))]
        unsafe fn opacity_changed(&self, sender: Option<&NSSlider>) {
            if let Some(slider) = sender {
                update_opacity_label(slider.doubleValue());
            }
        }

        /// Action method called when timer checkbox state changes
        #[unsafe(method(timerCheckboxChanged:))]
        unsafe fn timer_checkbox_changed(&self, sender: Option<&NSButton>) {
            if let Some(checkbox) = sender {
                let is_enabled = checkbox.state() == NSControlStateValueOn;
                update_timer_field_enabled(is_enabled);
            }
        }

        /// Action method called when "Reset to Default" button is clicked
        #[unsafe(method(resetDefaults:))]
        unsafe fn reset_defaults(&self, _sender: Option<&NSButton>) {
            reset_settings_to_defaults();
        }
    }
);

impl SettingsActionHandler {
    pub fn new(mtm: MainThreadMarker) -> Retained<Self> {
        let this = mtm.alloc::<SettingsActionHandler>();
        let this = this.set_ivars(SettingsActionHandlerIvars {});
        unsafe { msg_send![super(this), init] }
    }
}

/// Update the opacity percentage label
fn update_opacity_label(value: f64) {
    unsafe {
        with_ptr_void::<NSTextField, _>(&settings::OPACITY_LABEL, |label| {
            let percentage = (value * 100.0) as i32;
            label.setStringValue(&NSString::from_str(&format!("{}%", percentage)));
        });
    }
}

/// Update the timer field and dropdown enabled state based on checkbox
fn update_timer_field_enabled(enabled: bool) {
    // Update number field
    unsafe {
        with_ptr_void::<NSTextField, _>(&settings::TIMER_VALUE_FIELD, |field| {
            field.setEnabled(enabled);
            if enabled {
                field.setTextColor(Some(&NSColor::labelColor()));
            } else {
                field.setTextColor(Some(&NSColor::disabledControlTextColor()));
            }
        });
    }

    // Update unit dropdown
    unsafe {
        with_ptr_void::<NSPopUpButton, _>(&settings::TIMER_UNIT_DROPDOWN, |dropdown| {
            dropdown.setEnabled(enabled);
        });
    }

    // Clear validation label when checkbox is unchecked (disabled state)
    if !enabled {
        unsafe {
            with_ptr_void::<NSTextField, _>(&settings::TIMER_VALIDATION, |label| {
                label.setStringValue(&NSString::from_str(""));
                label.setTextColor(Some(&NSColor::systemGreenColor()));
            });
        }
    }
}

/// Reset all settings fields to their default values
/// Does NOT auto-save; user must click Save to persist
fn reset_settings_to_defaults() {
    // Reset Exit Key field to default
    unsafe {
        with_ptr_void::<NSTextField, _>(&settings::EXIT_KEY_FIELD, |field| {
            field.setStringValue(&NSString::from_str(DEFAULT_EXIT_KEY));
        });
    }
    // Update validation label
    validate_exit_key_realtime(DEFAULT_EXIT_KEY);

    // Reset Timer checkbox to disabled (unchecked)
    unsafe {
        with_ptr_void::<NSButton, _>(&settings::TIMER_CHECKBOX, |checkbox| {
            checkbox.setState(NSControlStateValueOff);
        });
    }

    // Reset Timer value field to empty
    unsafe {
        with_ptr_void::<NSTextField, _>(&settings::TIMER_VALUE_FIELD, |field| {
            field.setStringValue(ns_string!(""));
        });
    }

    // Reset Timer unit dropdown to Minutes (index 0)
    unsafe {
        with_ptr_void::<NSPopUpButton, _>(&settings::TIMER_UNIT_DROPDOWN, |dropdown| {
            dropdown.selectItemAtIndex(0);
        });
    }

    // Update timer field enabled state (disabled since checkbox is unchecked)
    update_timer_field_enabled(false);

    // Reset Opacity slider to 50% (0.5)
    unsafe {
        with_ptr_void::<NSSlider, _>(&settings::OPACITY_SLIDER, |slider| {
            slider.setDoubleValue(DEFAULT_OVERLAY_OPACITY);
        });
    }

    // Update opacity label to 50%
    update_opacity_label(DEFAULT_OVERLAY_OPACITY);

    println!("  Settings reset to defaults (not saved)");
}

/// Clean up settings window UI element references
/// Called both when window is closed programmatically and via X button
fn cleanup_settings_window_references() {
    // Clear window reference (swap to null to avoid double-cleanup)
    settings::WINDOW.store(std::ptr::null_mut(), Ordering::Release);

    // Clear UI element references
    settings::EXIT_KEY_FIELD.store(std::ptr::null_mut(), Ordering::Release);
    settings::TIMER_VALUE_FIELD.store(std::ptr::null_mut(), Ordering::Release);
    settings::TIMER_UNIT_DROPDOWN.store(std::ptr::null_mut(), Ordering::Release);
    settings::TIMER_CHECKBOX.store(std::ptr::null_mut(), Ordering::Release);
    settings::OPACITY_SLIDER.store(std::ptr::null_mut(), Ordering::Release);
    settings::OPACITY_LABEL.store(std::ptr::null_mut(), Ordering::Release);
    settings::EXIT_KEY_VALIDATION.store(std::ptr::null_mut(), Ordering::Release);
    settings::TIMER_VALIDATION.store(std::ptr::null_mut(), Ordering::Release);

    // Re-enable the settings menu item
    unsafe {
        with_ptr_void::<NSMenuItem, _>(&menu_bar::SETTINGS_ITEM, |menu_item| {
            menu_item.setEnabled(true);
        });
    }

    println!("  Settings window closed");
}

/// Close the settings window (called by Cancel/Save buttons)
fn close_settings_window() {
    unsafe {
        with_ptr_void::<NSPanel, _>(&settings::WINDOW, |window| {
            window.close();
        });
    }
    // Note: cleanup_settings_window_references() will be called by the window delegate
}

/// Save settings from the window to config file
fn save_settings_from_window() {
    let mut config = get_current_config();
    let mut has_errors = false;

    // Get exit key value and validate
    if let Some(value) = unsafe {
        with_ptr::<NSTextField, _, _>(&settings::EXIT_KEY_FIELD, |field| {
            field.stringValue().to_string()
        })
    } {
        let result = validate_exit_key_input(&value);
        update_exit_key_validation_label(&result);

        match result {
            ExitKeyValidation::Valid(key) => {
                config.exit_key = key;
            }
            ExitKeyValidation::Invalid(_) => {
                has_errors = true;
            }
        }
    }

    // Get timer value and validate (if checkbox is checked)
    // Read checkbox state
    let is_timer_enabled = unsafe {
        with_ptr::<NSButton, _, _>(&settings::TIMER_CHECKBOX, |checkbox| {
            checkbox.state() == NSControlStateValueOn
        })
    };
    // Read timer value
    let timer_value = unsafe {
        with_ptr::<NSTextField, _, _>(&settings::TIMER_VALUE_FIELD, |field| {
            field.stringValue().to_string()
        })
    };
    // Read unit dropdown index
    let unit_index = unsafe {
        with_ptr::<NSPopUpButton, _, _>(&settings::TIMER_UNIT_DROPDOWN, |dropdown| {
            dropdown.indexOfSelectedItem()
        })
    };

    // Process timer settings if all fields are available
    if let (Some(is_enabled), Some(value_str), Some(unit_idx)) =
        (is_timer_enabled, timer_value, unit_index)
    {
        if is_enabled {
            let trimmed = value_str.trim();

            if !trimmed.is_empty() {
                // Parse the number
                match trimmed.parse::<u64>() {
                    Ok(num) if num > 0 => {
                        // Get the selected unit suffix
                        let unit_suffix = match unit_idx {
                            0 => "m", // Minutes
                            1 => "h", // Hours
                            2 => "s", // Seconds
                            _ => "m", // Default to minutes
                        };

                        // Construct the duration string
                        let duration_str = format!("{}{}", num, unit_suffix);

                        // Validate using parse_duration
                        match parse_duration(&duration_str) {
                            Ok(_) => {
                                config.default_timer = Some(duration_str);
                                set_validation_label(&settings::TIMER_VALIDATION, true, "✓ Valid");
                            }
                            Err(e) => {
                                set_validation_label(&settings::TIMER_VALIDATION, false, &e);
                                has_errors = true;
                            }
                        }
                    }
                    Ok(_) => {
                        set_validation_label(
                            &settings::TIMER_VALIDATION,
                            false,
                            "Must be greater than 0",
                        );
                        has_errors = true;
                    }
                    Err(_) => {
                        set_validation_label(&settings::TIMER_VALIDATION, false, "Enter a number");
                        has_errors = true;
                    }
                }
            } else {
                set_validation_label(&settings::TIMER_VALIDATION, false, "Duration required");
                has_errors = true;
            }
        } else {
            config.default_timer = None;
            set_validation_label(&settings::TIMER_VALIDATION, true, "");
        }
    }

    // Get opacity value
    if let Some(opacity) = unsafe {
        with_ptr::<NSSlider, _, _>(&settings::OPACITY_SLIDER, |slider| slider.doubleValue())
    } {
        config.overlay_opacity = Some(opacity);
    }

    if has_errors {
        println!("  ⚠️  Settings have validation errors, please fix them");
        return;
    }

    // Save config to file
    match config.save() {
        Ok(()) => {
            println!("  ✓ Settings saved to config file");
            set_current_config(config.clone());

            // Update the global exit key if changed
            if let Some(ref key_str) = config.exit_key {
                if let Ok(key) = ExitKey::parse(key_str) {
                    set_exit_key(&key);
                    println!("  ✓ Exit key updated to: {}", key.display_name);
                }
            }

            close_settings_window();
        }
        Err(e) => {
            eprintln!("  ✗ Failed to save settings: {}", e);
        }
    }
}

/// Set a validation label with success or error state
fn set_validation_label(ptr: &AtomicPtr<c_void>, is_valid: bool, message: &str) {
    unsafe {
        with_ptr_void::<NSTextField, _>(ptr, |label| {
            label.setStringValue(&NSString::from_str(message));
            if is_valid {
                label.setTextColor(Some(&NSColor::systemGreenColor()));
            } else {
                label.setTextColor(Some(&NSColor::systemRedColor()));
            }
        });
    }
}

/// Result of validating an exit key input string
#[derive(Debug, PartialEq)]
pub(crate) enum ExitKeyValidation {
    /// Valid input: Some(key_string) for custom key, None for empty (use default)
    Valid(Option<String>),
    /// Invalid input with error message
    Invalid(String),
}

/// Validate an exit key input string
pub(crate) fn validate_exit_key_input(value: &str) -> ExitKeyValidation {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        ExitKeyValidation::Valid(None)
    } else {
        match ExitKey::parse(trimmed) {
            Ok(_) => ExitKeyValidation::Valid(Some(trimmed.to_string())),
            Err(e) => ExitKeyValidation::Invalid(e),
        }
    }
}

/// Update the exit key validation label based on validation result
fn update_exit_key_validation_label(result: &ExitKeyValidation) {
    match result {
        ExitKeyValidation::Valid(None) => {
            set_validation_label(
                &settings::EXIT_KEY_VALIDATION,
                true,
                &format!("Using default: {}", DEFAULT_EXIT_KEY),
            );
        }
        ExitKeyValidation::Valid(Some(_)) => {
            set_validation_label(&settings::EXIT_KEY_VALIDATION, true, "✓ Valid");
        }
        ExitKeyValidation::Invalid(e) => {
            set_validation_label(&settings::EXIT_KEY_VALIDATION, false, e);
        }
    }
}

/// Validate exit key field in real-time as user types
fn validate_exit_key_realtime(value: &str) {
    let result = validate_exit_key_input(value);
    update_exit_key_validation_label(&result);
}

/// Result of validating a timer duration input
#[derive(Debug, PartialEq)]
enum TimerValidation {
    /// Valid input with the parsed number
    Valid(u64),
    /// Empty input (no validation message shown)
    Empty,
    /// Invalid: negative number
    Negative,
    /// Invalid: zero value
    Zero,
    /// Invalid: not a number
    NotANumber,
}

/// Validate a timer duration string
fn validate_timer_input(value: &str) -> TimerValidation {
    let trimmed = value.trim();

    if trimmed.is_empty() {
        return TimerValidation::Empty;
    }

    if trimmed.starts_with('-') {
        return TimerValidation::Negative;
    }

    match trimmed.parse::<u64>() {
        Ok(num) if num > 0 => TimerValidation::Valid(num),
        Ok(_) => TimerValidation::Zero,
        Err(_) => TimerValidation::NotANumber,
    }
}

/// Validate timer duration field in real-time as user types
fn validate_timer_realtime(value: &str) {
    match validate_timer_input(value) {
        TimerValidation::Valid(_) => {
            set_validation_label(&settings::TIMER_VALIDATION, true, "✓ Valid");
        }
        TimerValidation::Empty => {
            // Don't show validation (let Save button handle "Duration required")
            set_validation_label(&settings::TIMER_VALIDATION, true, "");
        }
        TimerValidation::Negative => {
            set_validation_label(
                &settings::TIMER_VALIDATION,
                false,
                "Must be a positive number",
            );
        }
        TimerValidation::Zero => {
            set_validation_label(&settings::TIMER_VALIDATION, false, "Must be greater than 0");
        }
        TimerValidation::NotANumber => {
            set_validation_label(&settings::TIMER_VALIDATION, false, "Enter a number");
        }
    }
}

// ============================================================================
// Settings Window Layout Constants
// ============================================================================

/// Window dimensions
const WINDOW_WIDTH: CGFloat = 400.0;
const WINDOW_HEIGHT: CGFloat = 370.0;

/// Layout constants
const MARGIN: CGFloat = 20.0;
const LABEL_HEIGHT: CGFloat = 20.0;
const FIELD_HEIGHT: CGFloat = 24.0;
const ROW_SPACING: CGFloat = 8.0;
const SECTION_SPACING: CGFloat = 20.0;

/// Button constants
const BUTTON_HEIGHT: CGFloat = 28.0;
const BUTTON_WIDTH: CGFloat = 80.0;
const RESET_BUTTON_WIDTH: CGFloat = 120.0;
const BUTTON_SPACING: CGFloat = 12.0;

// ============================================================================
// Settings Window Setup Functions
// ============================================================================

/// Check if settings window is already open, bring to front if so
/// Returns true if we should continue creating the window, false if already open
fn prepare_settings_window() -> bool {
    // Try to bring existing window to front
    let window_exists = unsafe {
        with_ptr::<NSPanel, _, _>(&settings::WINDOW, |window| {
            window.makeKeyAndOrderFront(None);
        })
    };
    if window_exists.is_some() {
        return false;
    }

    // Disable the settings menu item while window is open
    unsafe {
        with_ptr_void::<NSMenuItem, _>(&menu_bar::SETTINGS_ITEM, |menu_item| {
            menu_item.setEnabled(false);
        });
    }

    true
}

/// Create the settings panel window centered on screen
fn create_settings_panel(mtm: MainThreadMarker) -> Retained<NSPanel> {
    // Calculate center position on screen
    let screen_frame = NSScreen::mainScreen(mtm)
        .map(|s| s.frame())
        .unwrap_or(CGRect {
            origin: CGPoint { x: 0.0, y: 0.0 },
            size: CGSize {
                width: 1920.0,
                height: 1080.0,
            },
        });

    let window_x = (screen_frame.size.width - WINDOW_WIDTH) / 2.0;
    let window_y = (screen_frame.size.height - WINDOW_HEIGHT) / 2.0;

    let window_frame = CGRect {
        origin: CGPoint {
            x: window_x,
            y: window_y,
        },
        size: CGSize {
            width: WINDOW_WIDTH,
            height: WINDOW_HEIGHT,
        },
    };

    // Create the settings panel (utility window style)
    let panel = {
        let panel = NSPanel::alloc(mtm);
        NSPanel::initWithContentRect_styleMask_backing_defer(
            panel,
            window_frame,
            NSWindowStyleMask::Titled | NSWindowStyleMask::Closable,
            NSBackingStoreType::Buffered,
            false,
        )
    };

    panel.setTitle(ns_string!("Cat Shield Settings"));
    panel.setLevel(3); // NSFloatingWindowLevel
    unsafe {
        panel.setReleasedWhenClosed(false);
    }

    panel
}

/// Get or create the settings window delegate
fn get_or_create_window_delegate(mtm: MainThreadMarker) -> &'static SettingsWindowDelegate {
    unsafe {
        let delegate_ptr = settings::WINDOW_DELEGATE.load(Ordering::Acquire);
        if delegate_ptr.is_null() {
            let new_delegate = SettingsWindowDelegate::new(mtm);
            settings::WINDOW_DELEGATE.store(
                Retained::as_ptr(&new_delegate) as *mut c_void,
                Ordering::Release,
            );
            // SAFETY: Transferring ownership to global AtomicPtr storage.
            // The delegate must outlive the settings window for the entire app lifetime.
            // Preventing drop here is correct because:
            // - The pointer is stored in settings::WINDOW_DELEGATE (a 'static AtomicPtr)
            // - The delegate is reused across multiple window opens/closes
            // - The Objective-C runtime retains it via setDelegate()
            // Cleanup: Never explicitly released; lives for app duration.
            std::mem::forget(new_delegate);
            &*(settings::WINDOW_DELEGATE.load(Ordering::Acquire) as *const SettingsWindowDelegate)
        } else {
            &*(delegate_ptr as *const SettingsWindowDelegate)
        }
    }
}

/// Get or create the settings action handler
fn get_or_create_action_handler(mtm: MainThreadMarker) -> &'static SettingsActionHandler {
    unsafe {
        let handler_ptr = settings::ACTION_HANDLER.load(Ordering::Acquire);
        if handler_ptr.is_null() {
            let new_handler = SettingsActionHandler::new(mtm);
            settings::ACTION_HANDLER.store(
                Retained::as_ptr(&new_handler) as *mut c_void,
                Ordering::Release,
            );
            // SAFETY: Transferring ownership to global AtomicPtr storage.
            // The handler must outlive all settings UI interactions for the app lifetime.
            // Preventing drop here is correct because:
            // - The pointer is stored in settings::ACTION_HANDLER (a 'static AtomicPtr)
            // - The handler is target for menu items and buttons (setTarget())
            // - Reused across multiple settings window opens/closes
            // Cleanup: Never explicitly released; lives for app duration.
            std::mem::forget(new_handler);
            &*(settings::ACTION_HANDLER.load(Ordering::Acquire) as *const SettingsActionHandler)
        } else {
            &*(handler_ptr as *const SettingsActionHandler)
        }
    }
}

/// Get or create the exit key field delegate
fn get_or_create_exit_key_delegate(mtm: MainThreadMarker) -> &'static ExitKeyFieldDelegate {
    unsafe {
        let delegate_ptr = settings::EXIT_KEY_FIELD_DELEGATE.load(Ordering::Acquire);
        if delegate_ptr.is_null() {
            let new_delegate = ExitKeyFieldDelegate::new(mtm);
            settings::EXIT_KEY_FIELD_DELEGATE.store(
                Retained::as_ptr(&new_delegate) as *mut c_void,
                Ordering::Release,
            );
            // SAFETY: Transferring ownership to global AtomicPtr storage.
            // The delegate handles real-time text validation for the exit key field.
            // Preventing drop here is correct because:
            // - The pointer is stored in settings::EXIT_KEY_FIELD_DELEGATE (a 'static AtomicPtr)
            // - NSTextField retains the delegate via setDelegate()
            // - Reused across multiple settings window opens/closes
            // Cleanup: Never explicitly released; lives for app duration.
            std::mem::forget(new_delegate);
            &*(settings::EXIT_KEY_FIELD_DELEGATE.load(Ordering::Acquire)
                as *const ExitKeyFieldDelegate)
        } else {
            &*(delegate_ptr as *const ExitKeyFieldDelegate)
        }
    }
}

/// Get or create the timer field delegate
fn get_or_create_timer_field_delegate(mtm: MainThreadMarker) -> &'static TimerFieldDelegate {
    unsafe {
        let delegate_ptr = settings::TIMER_FIELD_DELEGATE.load(Ordering::Acquire);
        if delegate_ptr.is_null() {
            let new_delegate = TimerFieldDelegate::new(mtm);
            settings::TIMER_FIELD_DELEGATE.store(
                Retained::as_ptr(&new_delegate) as *mut c_void,
                Ordering::Release,
            );
            // SAFETY: Transferring ownership to global AtomicPtr storage.
            // The delegate handles real-time text validation for the timer value field.
            // Preventing drop here is correct because:
            // - The pointer is stored in settings::TIMER_FIELD_DELEGATE (a 'static AtomicPtr)
            // - NSTextField retains the delegate via setDelegate()
            // - Reused across multiple settings window opens/closes
            // Cleanup: Never explicitly released; lives for app duration.
            std::mem::forget(new_delegate);
            &*(settings::TIMER_FIELD_DELEGATE.load(Ordering::Acquire) as *const TimerFieldDelegate)
        } else {
            &*(delegate_ptr as *const TimerFieldDelegate)
        }
    }
}

// ============================================================================
// Settings Section Setup Functions
// ============================================================================

/// Set up the Exit Key section in the settings window
/// Returns the new y_offset after adding this section
fn setup_exit_key_section(
    mtm: MainThreadMarker,
    content_view: &objc2_app_kit::NSView,
    config: &crate::config::Config,
    mut y_offset: CGFloat,
) -> CGFloat {
    let field_width = WINDOW_WIDTH - (MARGIN * 2.0);

    // Section label
    y_offset -= LABEL_HEIGHT;
    let exit_key_label = create_label(
        mtm,
        "Exit Key Shortcut:",
        CGRect {
            origin: CGPoint {
                x: MARGIN,
                y: y_offset,
            },
            size: CGSize {
                width: field_width,
                height: LABEL_HEIGHT,
            },
        },
        13.0,
        &NSColor::labelColor(),
        true,
    );
    content_view.addSubview(&exit_key_label);

    // Text field
    y_offset -= FIELD_HEIGHT + ROW_SPACING;
    let exit_key_field = NSTextField::new(mtm);
    exit_key_field.setFrame(CGRect {
        origin: CGPoint {
            x: MARGIN,
            y: y_offset,
        },
        size: CGSize {
            width: field_width,
            height: FIELD_HEIGHT,
        },
    });
    exit_key_field.setStringValue(&NSString::from_str(
        config.exit_key.as_deref().unwrap_or(DEFAULT_EXIT_KEY),
    ));
    exit_key_field.setPlaceholderString(Some(ns_string!("e.g., Cmd+Option+U")));

    // Set up delegate for real-time validation
    let exit_key_delegate = get_or_create_exit_key_delegate(mtm);
    unsafe {
        exit_key_field.setDelegate(Some(ProtocolObject::from_ref(exit_key_delegate)));
    }
    content_view.addSubview(&exit_key_field);
    settings::EXIT_KEY_FIELD.store(
        Retained::as_ptr(&exit_key_field) as *mut c_void,
        Ordering::Release,
    );
    // SAFETY: Transferring ownership to Objective-C runtime and global AtomicPtr.
    // Preventing drop here is correct because:
    // - The view is retained by NSWindow's content view hierarchy (addSubview)
    // - The pointer is stored in settings::EXIT_KEY_FIELD for later access
    // Cleanup: cleanup_settings_window_references() sets the AtomicPtr to null,
    // and NSWindow releases the view when the window closes.
    std::mem::forget(exit_key_field);

    // Validation label
    y_offset -= LABEL_HEIGHT + 2.0;
    let exit_key_validation = NSTextField::new(mtm);
    exit_key_validation.setFrame(CGRect {
        origin: CGPoint {
            x: MARGIN,
            y: y_offset,
        },
        size: CGSize {
            width: field_width,
            height: LABEL_HEIGHT,
        },
    });
    exit_key_validation.setEditable(false);
    exit_key_validation.setSelectable(false);
    exit_key_validation.setBordered(false);
    exit_key_validation.setDrawsBackground(false);
    exit_key_validation.setStringValue(ns_string!(""));
    exit_key_validation.setFont(Some(&NSFont::systemFontOfSize(11.0)));
    content_view.addSubview(&exit_key_validation);
    settings::EXIT_KEY_VALIDATION.store(
        Retained::as_ptr(&exit_key_validation) as *mut c_void,
        Ordering::Release,
    );
    // SAFETY: Transferring ownership to Objective-C runtime and global AtomicPtr.
    // Preventing drop here is correct because:
    // - The view is retained by NSWindow's content view hierarchy (addSubview)
    // - The pointer is stored in settings::EXIT_KEY_VALIDATION for validation updates
    // Cleanup: cleanup_settings_window_references() sets the AtomicPtr to null,
    // and NSWindow releases the view when the window closes.
    std::mem::forget(exit_key_validation);

    // Note: Do NOT call validate_exit_key_realtime() here.
    // Validation should only appear after user modifies the field.

    y_offset
}

/// Set up the Timer section in the settings window
/// Returns the new y_offset after adding this section
fn setup_timer_section(
    mtm: MainThreadMarker,
    content_view: &objc2_app_kit::NSView,
    config: &crate::config::Config,
    handler: &SettingsActionHandler,
    mut y_offset: CGFloat,
) -> CGFloat {
    let field_width = WINDOW_WIDTH - (MARGIN * 2.0);

    // Section spacing and label
    y_offset -= SECTION_SPACING;
    y_offset -= LABEL_HEIGHT;
    let timer_label = create_label(
        mtm,
        "Default Timer:",
        CGRect {
            origin: CGPoint {
                x: MARGIN,
                y: y_offset,
            },
            size: CGSize {
                width: field_width,
                height: LABEL_HEIGHT,
            },
        },
        13.0,
        &NSColor::labelColor(),
        true,
    );
    content_view.addSubview(&timer_label);

    // Checkbox for enabling default timer
    y_offset -= FIELD_HEIGHT + ROW_SPACING;
    let timer_checkbox = unsafe {
        let checkbox = NSButton::checkboxWithTitle_target_action(
            ns_string!("Enable auto-exit timer"),
            Some(handler),
            Some(objc2::sel!(timerCheckboxChanged:)),
            mtm,
        );
        checkbox.setFrame(CGRect {
            origin: CGPoint {
                x: MARGIN,
                y: y_offset,
            },
            size: CGSize {
                width: 200.0,
                height: FIELD_HEIGHT,
            },
        });
        checkbox.setControlSize(NSControlSize::Regular);
        if config.default_timer.is_some() {
            checkbox.setState(NSControlStateValueOn);
        } else {
            checkbox.setState(NSControlStateValueOff);
        }
        checkbox
    };
    content_view.addSubview(&timer_checkbox);
    settings::TIMER_CHECKBOX.store(
        Retained::as_ptr(&timer_checkbox) as *mut c_void,
        Ordering::Release,
    );
    // SAFETY: Transferring ownership to Objective-C runtime and global AtomicPtr.
    // Preventing drop here is correct because:
    // - The view is retained by NSWindow's content view hierarchy (addSubview)
    // - The pointer is stored in settings::TIMER_CHECKBOX for state access
    // Cleanup: cleanup_settings_window_references() sets the AtomicPtr to null,
    // and NSWindow releases the view when the window closes.
    std::mem::forget(timer_checkbox);

    // Parse existing timer value to extract number and unit
    // Use parse_duration() to properly handle compound durations like "1h30m"
    // and canonicalize to a single unit for the UI (hours, minutes, or seconds)
    y_offset -= FIELD_HEIGHT + ROW_SPACING;
    let (timer_value, timer_unit_index) = if let Some(ref timer_str) = config.default_timer {
        match parse_duration(timer_str) {
            Ok(total_secs) => {
                // Canonicalize to single-unit UI representation
                // Prefer hours if evenly divisible, then minutes, then seconds
                if total_secs % 3600 == 0 {
                    ((total_secs / 3600).to_string(), 1) // hours
                } else if total_secs % 60 == 0 {
                    ((total_secs / 60).to_string(), 0) // minutes
                } else {
                    (total_secs.to_string(), 2) // seconds
                }
            }
            // Fallback to old parsing for invalid strings (shouldn't happen with valid config)
            Err(_) => parse_timer_value_and_unit(timer_str),
        }
    } else {
        ("".to_string(), 0) // Default to minutes
    };

    // Number input field (narrower, on the left)
    let number_field_width: CGFloat = 80.0;
    let dropdown_width: CGFloat = 100.0;
    let spacing: CGFloat = 10.0;

    let timer_value_field = NSTextField::new(mtm);
    timer_value_field.setFrame(CGRect {
        origin: CGPoint {
            x: MARGIN,
            y: y_offset,
        },
        size: CGSize {
            width: number_field_width,
            height: FIELD_HEIGHT,
        },
    });
    timer_value_field.setStringValue(&NSString::from_str(&timer_value));
    timer_value_field.setPlaceholderString(Some(ns_string!("30")));
    timer_value_field.setEnabled(config.default_timer.is_some());
    if config.default_timer.is_none() {
        timer_value_field.setTextColor(Some(&NSColor::disabledControlTextColor()));
    }

    // Set up delegate for real-time validation
    let timer_field_delegate = get_or_create_timer_field_delegate(mtm);
    unsafe {
        timer_value_field.setDelegate(Some(ProtocolObject::from_ref(timer_field_delegate)));
    }
    content_view.addSubview(&timer_value_field);
    settings::TIMER_VALUE_FIELD.store(
        Retained::as_ptr(&timer_value_field) as *mut c_void,
        Ordering::Release,
    );
    // SAFETY: Transferring ownership to Objective-C runtime and global AtomicPtr.
    // Preventing drop here is correct because:
    // - The view is retained by NSWindow's content view hierarchy (addSubview)
    // - The pointer is stored in settings::TIMER_VALUE_FIELD for value access
    // Cleanup: cleanup_settings_window_references() sets the AtomicPtr to null,
    // and NSWindow releases the view when the window closes.
    std::mem::forget(timer_value_field);

    // Unit dropdown (to the right of the number field)
    let timer_unit_dropdown = NSPopUpButton::new(mtm);
    timer_unit_dropdown.setFrame(CGRect {
        origin: CGPoint {
            x: MARGIN + number_field_width + spacing,
            y: y_offset,
        },
        size: CGSize {
            width: dropdown_width,
            height: FIELD_HEIGHT,
        },
    });
    timer_unit_dropdown.addItemWithTitle(ns_string!("Minutes"));
    timer_unit_dropdown.addItemWithTitle(ns_string!("Hours"));
    timer_unit_dropdown.addItemWithTitle(ns_string!("Seconds"));
    timer_unit_dropdown.selectItemAtIndex(timer_unit_index);
    timer_unit_dropdown.setEnabled(config.default_timer.is_some());
    content_view.addSubview(&timer_unit_dropdown);
    settings::TIMER_UNIT_DROPDOWN.store(
        Retained::as_ptr(&timer_unit_dropdown) as *mut c_void,
        Ordering::Release,
    );
    // SAFETY: Transferring ownership to Objective-C runtime and global AtomicPtr.
    // Preventing drop here is correct because:
    // - The view is retained by NSWindow's content view hierarchy (addSubview)
    // - The pointer is stored in settings::TIMER_UNIT_DROPDOWN for selection access
    // Cleanup: cleanup_settings_window_references() sets the AtomicPtr to null,
    // and NSWindow releases the view when the window closes.
    std::mem::forget(timer_unit_dropdown);

    // Validation label
    y_offset -= LABEL_HEIGHT + 2.0;
    let timer_validation = NSTextField::new(mtm);
    timer_validation.setFrame(CGRect {
        origin: CGPoint {
            x: MARGIN,
            y: y_offset,
        },
        size: CGSize {
            width: field_width,
            height: LABEL_HEIGHT,
        },
    });
    timer_validation.setEditable(false);
    timer_validation.setSelectable(false);
    timer_validation.setBordered(false);
    timer_validation.setDrawsBackground(false);
    timer_validation.setStringValue(ns_string!(""));
    timer_validation.setFont(Some(&NSFont::systemFontOfSize(11.0)));
    content_view.addSubview(&timer_validation);
    settings::TIMER_VALIDATION.store(
        Retained::as_ptr(&timer_validation) as *mut c_void,
        Ordering::Release,
    );
    // SAFETY: Transferring ownership to Objective-C runtime and global AtomicPtr.
    // Preventing drop here is correct because:
    // - The view is retained by NSWindow's content view hierarchy (addSubview)
    // - The pointer is stored in settings::TIMER_VALIDATION for validation updates
    // Cleanup: cleanup_settings_window_references() sets the AtomicPtr to null,
    // and NSWindow releases the view when the window closes.
    std::mem::forget(timer_validation);

    y_offset
}

/// Set up the Opacity section in the settings window
/// Returns the new y_offset after adding this section
fn setup_opacity_section(
    mtm: MainThreadMarker,
    content_view: &objc2_app_kit::NSView,
    config: &crate::config::Config,
    handler: &SettingsActionHandler,
    mut y_offset: CGFloat,
) -> CGFloat {
    let field_width = WINDOW_WIDTH - (MARGIN * 2.0);

    // Section spacing and label
    y_offset -= SECTION_SPACING;
    y_offset -= LABEL_HEIGHT;
    let opacity_label = create_label(
        mtm,
        "Overlay Opacity:",
        CGRect {
            origin: CGPoint {
                x: MARGIN,
                y: y_offset,
            },
            size: CGSize {
                width: 120.0,
                height: LABEL_HEIGHT,
            },
        },
        13.0,
        &NSColor::labelColor(),
        true,
    );
    content_view.addSubview(&opacity_label);

    // Current percentage label (right side)
    let percentage_label = NSTextField::new(mtm);
    percentage_label.setFrame(CGRect {
        origin: CGPoint {
            x: WINDOW_WIDTH - MARGIN - 50.0,
            y: y_offset,
        },
        size: CGSize {
            width: 50.0,
            height: LABEL_HEIGHT,
        },
    });
    percentage_label.setEditable(false);
    percentage_label.setSelectable(false);
    percentage_label.setBordered(false);
    percentage_label.setDrawsBackground(false);
    percentage_label.setAlignment(NSTextAlignment::Right);
    let current_opacity = config.opacity();
    percentage_label.setStringValue(&NSString::from_str(&format!(
        "{}%",
        (current_opacity * 100.0) as i32
    )));
    percentage_label.setFont(Some(&NSFont::boldSystemFontOfSize(13.0)));
    content_view.addSubview(&percentage_label);
    settings::OPACITY_LABEL.store(
        Retained::as_ptr(&percentage_label) as *mut c_void,
        Ordering::Release,
    );
    // SAFETY: Transferring ownership to Objective-C runtime and global AtomicPtr.
    // Preventing drop here is correct because:
    // - The view is retained by NSWindow's content view hierarchy (addSubview)
    // - The pointer is stored in settings::OPACITY_LABEL for display updates
    // Cleanup: cleanup_settings_window_references() sets the AtomicPtr to null,
    // and NSWindow releases the view when the window closes.
    std::mem::forget(percentage_label);

    // Slider row
    y_offset -= FIELD_HEIGHT + ROW_SPACING;

    // Slider with min/max labels
    let slider_margin = 35.0;
    let slider_width = field_width - (slider_margin * 2.0);

    // Min label (20%)
    let min_label = create_label(
        mtm,
        "20%",
        CGRect {
            origin: CGPoint {
                x: MARGIN,
                y: y_offset,
            },
            size: CGSize {
                width: 30.0,
                height: FIELD_HEIGHT,
            },
        },
        11.0,
        &NSColor::secondaryLabelColor(),
        false,
    );
    content_view.addSubview(&min_label);

    // Max label (80%)
    let max_label = create_label(
        mtm,
        "80%",
        CGRect {
            origin: CGPoint {
                x: WINDOW_WIDTH - MARGIN - 30.0,
                y: y_offset,
            },
            size: CGSize {
                width: 30.0,
                height: FIELD_HEIGHT,
            },
        },
        11.0,
        &NSColor::secondaryLabelColor(),
        false,
    );
    max_label.setAlignment(NSTextAlignment::Right);
    content_view.addSubview(&max_label);

    // Opacity slider
    let opacity_slider = {
        let slider = NSSlider::new(mtm);
        slider.setFrame(CGRect {
            origin: CGPoint {
                x: MARGIN + slider_margin,
                y: y_offset,
            },
            size: CGSize {
                width: slider_width,
                height: FIELD_HEIGHT,
            },
        });
        slider.setMinValue(MIN_OVERLAY_OPACITY);
        slider.setMaxValue(MAX_OVERLAY_OPACITY);
        slider.setDoubleValue(current_opacity);
        unsafe {
            slider.setTarget(Some(handler));
            slider.setAction(Some(objc2::sel!(opacityChanged:)));
        }
        slider
    };
    content_view.addSubview(&opacity_slider);
    settings::OPACITY_SLIDER.store(
        Retained::as_ptr(&opacity_slider) as *mut c_void,
        Ordering::Release,
    );
    // SAFETY: Transferring ownership to Objective-C runtime and global AtomicPtr.
    // Preventing drop here is correct because:
    // - The view is retained by NSWindow's content view hierarchy (addSubview)
    // - The pointer is stored in settings::OPACITY_SLIDER for value access
    // Cleanup: cleanup_settings_window_references() sets the AtomicPtr to null,
    // and NSWindow releases the view when the window closes.
    std::mem::forget(opacity_slider);

    y_offset
}

/// Set up the Buttons section at the bottom of the settings window
fn setup_button_section(
    mtm: MainThreadMarker,
    content_view: &objc2_app_kit::NSView,
    handler: &SettingsActionHandler,
) {
    let button_y: CGFloat = MARGIN;

    // Reset to Default button (left side)
    let reset_button = unsafe {
        let button = NSButton::buttonWithTitle_target_action(
            ns_string!("Reset to Default"),
            Some(handler),
            Some(objc2::sel!(resetDefaults:)),
            mtm,
        );
        button.setFrame(CGRect {
            origin: CGPoint {
                x: MARGIN,
                y: button_y,
            },
            size: CGSize {
                width: RESET_BUTTON_WIDTH,
                height: BUTTON_HEIGHT,
            },
        });
        button.setButtonType(NSButtonType::MomentaryPushIn);
        button
    };
    content_view.addSubview(&reset_button);
    // SAFETY: Transferring ownership to Objective-C runtime.
    // Preventing drop here is correct because:
    // - The view is retained by NSWindow's content view hierarchy (addSubview)
    // - The button action is handled by the SettingsActionHandler
    // Cleanup: NSWindow releases all subviews when the window closes.
    std::mem::forget(reset_button);

    // Cancel button (right side, before Save)
    let cancel_button = unsafe {
        let button = NSButton::buttonWithTitle_target_action(
            ns_string!("Cancel"),
            Some(handler),
            Some(objc2::sel!(cancelSettings:)),
            mtm,
        );
        button.setFrame(CGRect {
            origin: CGPoint {
                x: WINDOW_WIDTH - MARGIN - BUTTON_WIDTH - BUTTON_SPACING - BUTTON_WIDTH,
                y: button_y,
            },
            size: CGSize {
                width: BUTTON_WIDTH,
                height: BUTTON_HEIGHT,
            },
        });
        button.setButtonType(NSButtonType::MomentaryPushIn);
        button.setKeyEquivalent(ns_string!("\u{1b}")); // Escape key
        button
    };
    content_view.addSubview(&cancel_button);
    // SAFETY: Transferring ownership to Objective-C runtime.
    // Preventing drop here is correct because:
    // - The view is retained by NSWindow's content view hierarchy (addSubview)
    // - The button action is handled by the SettingsActionHandler
    // Cleanup: NSWindow releases all subviews when the window closes.
    std::mem::forget(cancel_button);

    // Save button (right)
    let save_button = unsafe {
        let button = NSButton::buttonWithTitle_target_action(
            ns_string!("Save"),
            Some(handler),
            Some(objc2::sel!(saveSettings:)),
            mtm,
        );
        button.setFrame(CGRect {
            origin: CGPoint {
                x: WINDOW_WIDTH - MARGIN - BUTTON_WIDTH,
                y: button_y,
            },
            size: CGSize {
                width: BUTTON_WIDTH,
                height: BUTTON_HEIGHT,
            },
        });
        button.setButtonType(NSButtonType::MomentaryPushIn);
        button.setKeyEquivalent(ns_string!("\r")); // Return key
        button
    };
    content_view.addSubview(&save_button);
    // SAFETY: Transferring ownership to Objective-C runtime.
    // Preventing drop here is correct because:
    // - The view is retained by NSWindow's content view hierarchy (addSubview)
    // - The button action is handled by the SettingsActionHandler
    // Cleanup: NSWindow releases all subviews when the window closes.
    std::mem::forget(save_button);
}

/// Finalize window setup and display it
fn finalize_and_show_window(mtm: MainThreadMarker, panel: Retained<NSPanel>) {
    // Activate the application so the window can receive focus
    // This is needed because the app runs in accessory mode (no dock icon)
    let app = NSApplication::sharedApplication(mtm);
    #[allow(deprecated)]
    app.activateIgnoringOtherApps(true);

    // Show the window
    panel.makeKeyAndOrderFront(None);

    // SAFETY: Transferring ownership to global AtomicPtr storage.
    // Preventing drop here is correct because:
    // - The pointer was already stored in settings::WINDOW (in show_settings_window)
    // - The window must remain valid while displayed
    // - The Objective-C runtime also holds references
    // Cleanup: The panel intentionally lives for the app's lifetime and is reused
    // across multiple open/close cycles. When the window closes, only the pointer
    // is cleared (not released) so the panel can be reshown. The subviews are
    // released by NSWindow when closed, but the panel itself persists.
    std::mem::forget(panel);

    println!("  Settings window opened");
}

// ============================================================================
// Main Entry Point
// ============================================================================

/// Show the settings window
pub fn show_settings_window(mtm: MainThreadMarker) {
    // Check if already open, disable menu item
    if !prepare_settings_window() {
        return;
    }

    // Create panel and set up delegates
    let panel = create_settings_panel(mtm);
    let delegate = get_or_create_window_delegate(mtm);
    panel.setDelegate(Some(ProtocolObject::from_ref(delegate)));

    // Store window reference
    settings::WINDOW.store(Retained::as_ptr(&panel) as *mut c_void, Ordering::Release);

    // Get action handler
    let handler = get_or_create_action_handler(mtm);

    // Load current config
    let config = get_current_config();

    // Create content view with controls
    if let Some(content_view) = panel.contentView() {
        let mut y_offset = WINDOW_HEIGHT - MARGIN - 10.0;

        // Set up each section
        y_offset = setup_exit_key_section(mtm, &content_view, &config, y_offset);
        y_offset = setup_timer_section(mtm, &content_view, &config, handler, y_offset);
        let _ = setup_opacity_section(mtm, &content_view, &config, handler, y_offset);
        setup_button_section(mtm, &content_view, handler);
    }

    // Show the window
    finalize_and_show_window(mtm, panel);
}

#[cfg(test)]
mod tests {
    use super::*;

    // ========================================================================
    // Timer Validation Tests
    // ========================================================================

    #[test]
    fn test_validate_timer_input_valid() {
        assert_eq!(TimerValidation::Valid(30), validate_timer_input("30"));
        assert_eq!(TimerValidation::Valid(1), validate_timer_input("1"));
        assert_eq!(TimerValidation::Valid(999), validate_timer_input("999"));
    }

    #[test]
    fn test_validate_timer_input_valid_with_whitespace() {
        assert_eq!(TimerValidation::Valid(30), validate_timer_input("  30  "));
        assert_eq!(TimerValidation::Valid(5), validate_timer_input("\t5\n"));
    }

    #[test]
    fn test_validate_timer_input_empty() {
        assert_eq!(TimerValidation::Empty, validate_timer_input(""));
        assert_eq!(TimerValidation::Empty, validate_timer_input("   "));
        assert_eq!(TimerValidation::Empty, validate_timer_input("\t\n"));
    }

    #[test]
    fn test_validate_timer_input_zero() {
        assert_eq!(TimerValidation::Zero, validate_timer_input("0"));
        assert_eq!(TimerValidation::Zero, validate_timer_input("  0  "));
    }

    #[test]
    fn test_validate_timer_input_negative() {
        assert_eq!(TimerValidation::Negative, validate_timer_input("-5"));
        assert_eq!(TimerValidation::Negative, validate_timer_input("-1"));
        assert_eq!(TimerValidation::Negative, validate_timer_input("  -10  "));
    }

    #[test]
    fn test_validate_timer_input_not_a_number() {
        assert_eq!(TimerValidation::NotANumber, validate_timer_input("abc"));
        assert_eq!(TimerValidation::NotANumber, validate_timer_input("12.5"));
        assert_eq!(TimerValidation::NotANumber, validate_timer_input("30m"));
        assert_eq!(TimerValidation::NotANumber, validate_timer_input("hello"));
    }

    // ========================================================================
    // Exit Key Validation Tests
    // ========================================================================

    #[test]
    fn test_validate_exit_key_input_valid_key() {
        match validate_exit_key_input("Cmd+Q") {
            ExitKeyValidation::Valid(Some(_)) => {}
            _ => panic!("Expected valid key"),
        }
    }

    #[test]
    fn test_validate_exit_key_input_valid_with_multiple_modifiers() {
        match validate_exit_key_input("Cmd+Option+U") {
            ExitKeyValidation::Valid(Some(key)) => {
                assert_eq!(key, "Cmd+Option+U");
            }
            _ => panic!("Expected valid key with multiple modifiers"),
        }
    }

    #[test]
    fn test_validate_exit_key_input_empty_returns_none() {
        match validate_exit_key_input("") {
            ExitKeyValidation::Valid(None) => {}
            _ => panic!("Expected Valid(None) for empty input"),
        }
    }

    #[test]
    fn test_validate_exit_key_input_whitespace_only() {
        match validate_exit_key_input("   ") {
            ExitKeyValidation::Valid(None) => {}
            _ => panic!("Expected Valid(None) for whitespace"),
        }
    }

    #[test]
    fn test_validate_exit_key_input_whitespace_tabs_newlines() {
        match validate_exit_key_input("\t\n  ") {
            ExitKeyValidation::Valid(None) => {}
            _ => panic!("Expected Valid(None) for tabs/newlines"),
        }
    }

    #[test]
    fn test_validate_exit_key_input_invalid_key() {
        match validate_exit_key_input("InvalidKey") {
            ExitKeyValidation::Invalid(_) => {}
            _ => panic!("Expected Invalid for bad key"),
        }
    }

    #[test]
    fn test_validate_exit_key_input_missing_modifier() {
        match validate_exit_key_input("Q") {
            ExitKeyValidation::Invalid(_) => {}
            _ => panic!("Expected Invalid for key without modifier"),
        }
    }

    #[test]
    fn test_validate_exit_key_input_invalid_modifier() {
        match validate_exit_key_input("Super+Q") {
            ExitKeyValidation::Invalid(_) => {}
            _ => panic!("Expected Invalid for unknown modifier"),
        }
    }
}
