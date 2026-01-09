//! Settings window for Cat Shield

use crate::config::{
    get_current_config, set_current_config, DEFAULT_OVERLAY_OPACITY, MAX_OVERLAY_OPACITY,
    MIN_OVERLAY_OPACITY,
};
use crate::input::{set_exit_key, ExitKey, DEFAULT_EXIT_KEY};
use crate::timer::{parse_duration, parse_timer_value_and_unit};
use crate::ui::helpers::create_label;
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
use std::sync::atomic::Ordering;

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
            // Read the current value from the exit key field
            let field_ptr = settings::EXIT_KEY_FIELD.load(Ordering::SeqCst);
            if !field_ptr.is_null() {
                unsafe {
                    let field: &NSTextField = &*(field_ptr as *const NSTextField);
                    let value = field.stringValue().to_string();
                    validate_exit_key_realtime(&value);
                }
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
            let checkbox_ptr = settings::TIMER_CHECKBOX.load(Ordering::SeqCst);
            if !checkbox_ptr.is_null() {
                unsafe {
                    let checkbox: &NSButton = &*(checkbox_ptr as *const NSButton);
                    if checkbox.state() != NSControlStateValueOn {
                        return; // Don't validate when checkbox is unchecked
                    }
                }
            }

            // Read the current value from the timer field
            let field_ptr = settings::TIMER_VALUE_FIELD.load(Ordering::SeqCst);
            if !field_ptr.is_null() {
                unsafe {
                    let field: &NSTextField = &*(field_ptr as *const NSTextField);
                    let value = field.stringValue().to_string();
                    validate_timer_realtime(&value);
                }
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
    let label_ptr = settings::OPACITY_LABEL.load(Ordering::SeqCst);
    if !label_ptr.is_null() {
        unsafe {
            let label: &NSTextField = &*(label_ptr as *const NSTextField);
            let percentage = (value * 100.0) as i32;
            label.setStringValue(&NSString::from_str(&format!("{}%", percentage)));
        }
    }
}

/// Update the timer field and dropdown enabled state based on checkbox
fn update_timer_field_enabled(enabled: bool) {
    // Update number field
    let field_ptr = settings::TIMER_VALUE_FIELD.load(Ordering::SeqCst);
    if !field_ptr.is_null() {
        unsafe {
            let field: &NSTextField = &*(field_ptr as *const NSTextField);
            field.setEnabled(enabled);
            if enabled {
                field.setTextColor(Some(&NSColor::labelColor()));
            } else {
                field.setTextColor(Some(&NSColor::disabledControlTextColor()));
            }
        }
    }

    // Update unit dropdown
    let dropdown_ptr = settings::TIMER_UNIT_DROPDOWN.load(Ordering::SeqCst);
    if !dropdown_ptr.is_null() {
        unsafe {
            let dropdown: &NSPopUpButton = &*(dropdown_ptr as *const NSPopUpButton);
            dropdown.setEnabled(enabled);
        }
    }

    // Clear validation label when checkbox is unchecked (disabled state)
    if !enabled {
        update_validation_label(settings::TIMER_VALIDATION.load(Ordering::SeqCst), true, "");
    }
}

/// Reset all settings fields to their default values
/// Does NOT auto-save; user must click Save to persist
fn reset_settings_to_defaults() {
    // Reset Exit Key field to default
    let exit_key_ptr = settings::EXIT_KEY_FIELD.load(Ordering::SeqCst);
    if !exit_key_ptr.is_null() {
        unsafe {
            let field: &NSTextField = &*(exit_key_ptr as *const NSTextField);
            field.setStringValue(&NSString::from_str(DEFAULT_EXIT_KEY));
        }
        // Update validation label
        validate_exit_key_realtime(DEFAULT_EXIT_KEY);
    }

    // Reset Timer checkbox to disabled (unchecked)
    let timer_checkbox_ptr = settings::TIMER_CHECKBOX.load(Ordering::SeqCst);
    if !timer_checkbox_ptr.is_null() {
        unsafe {
            let checkbox: &NSButton = &*(timer_checkbox_ptr as *const NSButton);
            checkbox.setState(NSControlStateValueOff);
        }
    }

    // Reset Timer value field to empty
    let timer_value_ptr = settings::TIMER_VALUE_FIELD.load(Ordering::SeqCst);
    if !timer_value_ptr.is_null() {
        unsafe {
            let field: &NSTextField = &*(timer_value_ptr as *const NSTextField);
            field.setStringValue(ns_string!(""));
        }
    }

    // Reset Timer unit dropdown to Minutes (index 0)
    let timer_unit_ptr = settings::TIMER_UNIT_DROPDOWN.load(Ordering::SeqCst);
    if !timer_unit_ptr.is_null() {
        unsafe {
            let dropdown: &NSPopUpButton = &*(timer_unit_ptr as *const NSPopUpButton);
            dropdown.selectItemAtIndex(0);
        }
    }

    // Update timer field enabled state (disabled since checkbox is unchecked)
    update_timer_field_enabled(false);

    // Clear timer validation label
    update_validation_label(settings::TIMER_VALIDATION.load(Ordering::SeqCst), true, "");

    // Reset Opacity slider to 50% (0.5)
    let opacity_ptr = settings::OPACITY_SLIDER.load(Ordering::SeqCst);
    if !opacity_ptr.is_null() {
        unsafe {
            let slider: &NSSlider = &*(opacity_ptr as *const NSSlider);
            slider.setDoubleValue(DEFAULT_OVERLAY_OPACITY);
        }
    }

    // Update opacity label to 50%
    update_opacity_label(DEFAULT_OVERLAY_OPACITY);

    println!("  Settings reset to defaults (not saved)");
}

/// Clean up settings window UI element references
/// Called both when window is closed programmatically and via X button
fn cleanup_settings_window_references() {
    // Clear window reference (swap to null to avoid double-cleanup)
    settings::WINDOW.store(std::ptr::null_mut(), Ordering::SeqCst);

    // Clear UI element references
    settings::EXIT_KEY_FIELD.store(std::ptr::null_mut(), Ordering::SeqCst);
    settings::TIMER_VALUE_FIELD.store(std::ptr::null_mut(), Ordering::SeqCst);
    settings::TIMER_UNIT_DROPDOWN.store(std::ptr::null_mut(), Ordering::SeqCst);
    settings::TIMER_CHECKBOX.store(std::ptr::null_mut(), Ordering::SeqCst);
    settings::OPACITY_SLIDER.store(std::ptr::null_mut(), Ordering::SeqCst);
    settings::OPACITY_LABEL.store(std::ptr::null_mut(), Ordering::SeqCst);
    settings::EXIT_KEY_VALIDATION.store(std::ptr::null_mut(), Ordering::SeqCst);
    settings::TIMER_VALIDATION.store(std::ptr::null_mut(), Ordering::SeqCst);

    // Re-enable the settings menu item
    let menu_item_ptr = menu_bar::SETTINGS_ITEM.load(Ordering::SeqCst);
    if !menu_item_ptr.is_null() {
        unsafe {
            let menu_item: &NSMenuItem = &*(menu_item_ptr as *const NSMenuItem);
            menu_item.setEnabled(true);
        }
    }

    println!("  Settings window closed");
}

/// Close the settings window (called by Cancel/Save buttons)
fn close_settings_window() {
    let window_ptr = settings::WINDOW.load(Ordering::SeqCst);
    if !window_ptr.is_null() {
        unsafe {
            let window: &NSPanel = &*(window_ptr as *const NSPanel);
            window.close();
        }
        // Note: cleanup_settings_window_references() will be called by the window delegate
    }
}

/// Save settings from the window to config file
fn save_settings_from_window() {
    let mut config = get_current_config();
    let mut has_errors = false;

    // Get exit key value and validate
    let exit_key_ptr = settings::EXIT_KEY_FIELD.load(Ordering::SeqCst);
    if !exit_key_ptr.is_null() {
        let value = unsafe {
            let field: &NSTextField = &*(exit_key_ptr as *const NSTextField);
            field.stringValue().to_string()
        };
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
    let timer_checkbox_ptr = settings::TIMER_CHECKBOX.load(Ordering::SeqCst);
    let timer_value_ptr = settings::TIMER_VALUE_FIELD.load(Ordering::SeqCst);
    let timer_unit_ptr = settings::TIMER_UNIT_DROPDOWN.load(Ordering::SeqCst);
    if !timer_checkbox_ptr.is_null() && !timer_value_ptr.is_null() && !timer_unit_ptr.is_null() {
        unsafe {
            let checkbox: &NSButton = &*(timer_checkbox_ptr as *const NSButton);
            let value_field: &NSTextField = &*(timer_value_ptr as *const NSTextField);
            let unit_dropdown: &NSPopUpButton = &*(timer_unit_ptr as *const NSPopUpButton);
            let is_enabled = checkbox.state() == NSControlStateValueOn;

            if is_enabled {
                let value_str = value_field.stringValue().to_string();
                let trimmed = value_str.trim();

                if !trimmed.is_empty() {
                    // Parse the number
                    match trimmed.parse::<u64>() {
                        Ok(num) if num > 0 => {
                            // Get the selected unit suffix
                            let unit_index = unit_dropdown.indexOfSelectedItem();
                            let unit_suffix = match unit_index {
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
                                    update_validation_label(
                                        settings::TIMER_VALIDATION.load(Ordering::SeqCst),
                                        true,
                                        "✓ Valid",
                                    );
                                }
                                Err(e) => {
                                    update_validation_label(
                                        settings::TIMER_VALIDATION.load(Ordering::SeqCst),
                                        false,
                                        &e,
                                    );
                                    has_errors = true;
                                }
                            }
                        }
                        Ok(_) => {
                            update_validation_label(
                                settings::TIMER_VALIDATION.load(Ordering::SeqCst),
                                false,
                                "Must be greater than 0",
                            );
                            has_errors = true;
                        }
                        Err(_) => {
                            update_validation_label(
                                settings::TIMER_VALIDATION.load(Ordering::SeqCst),
                                false,
                                "Enter a number",
                            );
                            has_errors = true;
                        }
                    }
                } else {
                    update_validation_label(
                        settings::TIMER_VALIDATION.load(Ordering::SeqCst),
                        false,
                        "Duration required",
                    );
                    has_errors = true;
                }
            } else {
                config.default_timer = None;
                update_validation_label(
                    settings::TIMER_VALIDATION.load(Ordering::SeqCst),
                    true,
                    "",
                );
            }
        }
    }

    // Get opacity value
    let opacity_ptr = settings::OPACITY_SLIDER.load(Ordering::SeqCst);
    if !opacity_ptr.is_null() {
        unsafe {
            let slider: &NSSlider = &*(opacity_ptr as *const NSSlider);
            config.overlay_opacity = Some(slider.doubleValue());
        }
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

/// Update a validation label with success or error state
fn update_validation_label(label_ptr: *mut c_void, is_valid: bool, message: &str) {
    if !label_ptr.is_null() {
        unsafe {
            let label: &NSTextField = &*(label_ptr as *const NSTextField);
            label.setStringValue(&NSString::from_str(message));
            if is_valid {
                label.setTextColor(Some(&NSColor::systemGreenColor()));
            } else {
                label.setTextColor(Some(&NSColor::systemRedColor()));
            }
        }
    }
}

/// Result of validating an exit key input string
enum ExitKeyValidation {
    /// Valid input: Some(key_string) for custom key, None for empty (use default)
    Valid(Option<String>),
    /// Invalid input with error message
    Invalid(String),
}

/// Validate an exit key input string
fn validate_exit_key_input(value: &str) -> ExitKeyValidation {
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
    let label_ptr = settings::EXIT_KEY_VALIDATION.load(Ordering::SeqCst);
    match result {
        ExitKeyValidation::Valid(None) => {
            update_validation_label(
                label_ptr,
                true,
                &format!("Using default: {}", DEFAULT_EXIT_KEY),
            );
        }
        ExitKeyValidation::Valid(Some(_)) => {
            update_validation_label(label_ptr, true, "✓ Valid");
        }
        ExitKeyValidation::Invalid(e) => {
            update_validation_label(label_ptr, false, e);
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
    let label_ptr = settings::TIMER_VALIDATION.load(Ordering::SeqCst);

    match validate_timer_input(value) {
        TimerValidation::Valid(_) => {
            update_validation_label(label_ptr, true, "✓ Valid");
        }
        TimerValidation::Empty => {
            // Don't show validation (let Save button handle "Duration required")
            update_validation_label(label_ptr, true, "");
        }
        TimerValidation::Negative => {
            update_validation_label(label_ptr, false, "Must be a positive number");
        }
        TimerValidation::Zero => {
            update_validation_label(label_ptr, false, "Must be greater than 0");
        }
        TimerValidation::NotANumber => {
            update_validation_label(label_ptr, false, "Enter a number");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
}

/// Show the settings window
pub fn show_settings_window(mtm: MainThreadMarker) {
    // Check if settings window is already open
    let existing = settings::WINDOW.load(Ordering::SeqCst);
    if !existing.is_null() {
        // Bring existing window to front
        unsafe {
            let window: &NSPanel = &*(existing as *const NSPanel);
            window.makeKeyAndOrderFront(None);
        }
        return;
    }

    // Disable the settings menu item while window is open
    let menu_item_ptr = menu_bar::SETTINGS_ITEM.load(Ordering::SeqCst);
    if !menu_item_ptr.is_null() {
        unsafe {
            let menu_item: &NSMenuItem = &*(menu_item_ptr as *const NSMenuItem);
            menu_item.setEnabled(false);
        }
    }

    // Window dimensions
    let window_width: CGFloat = 400.0;
    let window_height: CGFloat = 370.0;

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

    let window_x = (screen_frame.size.width - window_width) / 2.0;
    let window_y = (screen_frame.size.height - window_height) / 2.0;

    let window_frame = CGRect {
        origin: CGPoint {
            x: window_x,
            y: window_y,
        },
        size: CGSize {
            width: window_width,
            height: window_height,
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

    // Create and set window delegate to handle close button (X) cleanup
    let delegate = unsafe {
        let delegate_ptr = settings::WINDOW_DELEGATE.load(Ordering::SeqCst);
        if delegate_ptr.is_null() {
            let new_delegate = SettingsWindowDelegate::new(mtm);
            settings::WINDOW_DELEGATE.store(
                Retained::as_ptr(&new_delegate) as *mut c_void,
                Ordering::SeqCst,
            );
            std::mem::forget(new_delegate);
            &*(settings::WINDOW_DELEGATE.load(Ordering::SeqCst) as *const SettingsWindowDelegate)
        } else {
            &*(delegate_ptr as *const SettingsWindowDelegate)
        }
    };
    panel.setDelegate(Some(ProtocolObject::from_ref(delegate)));

    // Store window reference
    settings::WINDOW.store(Retained::as_ptr(&panel) as *mut c_void, Ordering::SeqCst);

    // Get or create settings action handler
    let handler = unsafe {
        let handler_ptr = settings::ACTION_HANDLER.load(Ordering::SeqCst);
        if handler_ptr.is_null() {
            let new_handler = SettingsActionHandler::new(mtm);
            settings::ACTION_HANDLER.store(
                Retained::as_ptr(&new_handler) as *mut c_void,
                Ordering::SeqCst,
            );
            std::mem::forget(new_handler);
            &*(settings::ACTION_HANDLER.load(Ordering::SeqCst) as *const SettingsActionHandler)
        } else {
            &*(handler_ptr as *const SettingsActionHandler)
        }
    };

    // Load current config values
    let config = get_current_config();

    // Create content view with controls
    if let Some(content_view) = panel.contentView() {
        let margin: CGFloat = 20.0;
        let label_height: CGFloat = 20.0;
        let field_height: CGFloat = 24.0;
        let row_spacing: CGFloat = 8.0;
        let section_spacing: CGFloat = 20.0;
        let field_width = window_width - (margin * 2.0);

        let mut y_offset = window_height - margin - 10.0;

        // ========================================
        // Exit Key Section
        // ========================================
        y_offset -= label_height;
        let exit_key_label = create_label(
            mtm,
            "Exit Key Shortcut:",
            CGRect {
                origin: CGPoint {
                    x: margin,
                    y: y_offset,
                },
                size: CGSize {
                    width: field_width,
                    height: label_height,
                },
            },
            13.0,
            &NSColor::labelColor(),
            true,
        );
        content_view.addSubview(&exit_key_label);

        y_offset -= field_height + row_spacing;
        let exit_key_field = NSTextField::new(mtm);
        exit_key_field.setFrame(CGRect {
            origin: CGPoint {
                x: margin,
                y: y_offset,
            },
            size: CGSize {
                width: field_width,
                height: field_height,
            },
        });
        exit_key_field.setStringValue(&NSString::from_str(
            config.exit_key.as_deref().unwrap_or(DEFAULT_EXIT_KEY),
        ));
        exit_key_field.setPlaceholderString(Some(ns_string!("e.g., Cmd+Option+U")));
        // Set up delegate for real-time validation on text changes
        let exit_key_delegate = unsafe {
            let delegate_ptr = settings::EXIT_KEY_FIELD_DELEGATE.load(Ordering::SeqCst);
            if delegate_ptr.is_null() {
                let new_delegate = ExitKeyFieldDelegate::new(mtm);
                settings::EXIT_KEY_FIELD_DELEGATE.store(
                    Retained::as_ptr(&new_delegate) as *mut c_void,
                    Ordering::SeqCst,
                );
                std::mem::forget(new_delegate);
                &*(settings::EXIT_KEY_FIELD_DELEGATE.load(Ordering::SeqCst)
                    as *const ExitKeyFieldDelegate)
            } else {
                &*(delegate_ptr as *const ExitKeyFieldDelegate)
            }
        };
        unsafe {
            exit_key_field.setDelegate(Some(ProtocolObject::from_ref(exit_key_delegate)));
        }
        content_view.addSubview(&exit_key_field);
        settings::EXIT_KEY_FIELD.store(
            Retained::as_ptr(&exit_key_field) as *mut c_void,
            Ordering::SeqCst,
        );
        std::mem::forget(exit_key_field);

        y_offset -= label_height + 2.0;
        let exit_key_validation = NSTextField::new(mtm);
        exit_key_validation.setFrame(CGRect {
            origin: CGPoint {
                x: margin,
                y: y_offset,
            },
            size: CGSize {
                width: field_width,
                height: label_height,
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
            Ordering::SeqCst,
        );
        std::mem::forget(exit_key_validation);

        // Note: Do NOT call validate_exit_key_realtime() here.
        // Validation should only appear after user modifies the field.
        // The delegate's controlTextDidChange: handles real-time validation on edits.

        // ========================================
        // Default Timer Section
        // ========================================
        y_offset -= section_spacing;
        y_offset -= label_height;
        let timer_label = create_label(
            mtm,
            "Default Timer:",
            CGRect {
                origin: CGPoint {
                    x: margin,
                    y: y_offset,
                },
                size: CGSize {
                    width: field_width,
                    height: label_height,
                },
            },
            13.0,
            &NSColor::labelColor(),
            true,
        );
        content_view.addSubview(&timer_label);

        y_offset -= field_height + row_spacing;

        // Checkbox for enabling default timer
        let timer_checkbox = unsafe {
            let checkbox = NSButton::checkboxWithTitle_target_action(
                ns_string!("Enable auto-exit timer"),
                Some(handler),
                Some(objc2::sel!(timerCheckboxChanged:)),
                mtm,
            );
            checkbox.setFrame(CGRect {
                origin: CGPoint {
                    x: margin,
                    y: y_offset,
                },
                size: CGSize {
                    width: 200.0,
                    height: field_height,
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
            Ordering::SeqCst,
        );
        std::mem::forget(timer_checkbox);

        y_offset -= field_height + row_spacing;

        // Parse existing timer value to extract number and unit
        let (timer_value, timer_unit_index) = if let Some(ref timer_str) = config.default_timer {
            parse_timer_value_and_unit(timer_str)
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
                x: margin,
                y: y_offset,
            },
            size: CGSize {
                width: number_field_width,
                height: field_height,
            },
        });
        timer_value_field.setStringValue(&NSString::from_str(&timer_value));
        timer_value_field.setPlaceholderString(Some(ns_string!("30")));
        timer_value_field.setEnabled(config.default_timer.is_some());
        if config.default_timer.is_none() {
            timer_value_field.setTextColor(Some(&NSColor::disabledControlTextColor()));
        }
        // Set up delegate for real-time validation on text changes
        let timer_field_delegate = unsafe {
            let delegate_ptr = settings::TIMER_FIELD_DELEGATE.load(Ordering::SeqCst);
            if delegate_ptr.is_null() {
                let new_delegate = TimerFieldDelegate::new(mtm);
                settings::TIMER_FIELD_DELEGATE.store(
                    Retained::as_ptr(&new_delegate) as *mut c_void,
                    Ordering::SeqCst,
                );
                std::mem::forget(new_delegate);
                &*(settings::TIMER_FIELD_DELEGATE.load(Ordering::SeqCst)
                    as *const TimerFieldDelegate)
            } else {
                &*(delegate_ptr as *const TimerFieldDelegate)
            }
        };
        unsafe {
            timer_value_field.setDelegate(Some(ProtocolObject::from_ref(timer_field_delegate)));
        }
        content_view.addSubview(&timer_value_field);
        settings::TIMER_VALUE_FIELD.store(
            Retained::as_ptr(&timer_value_field) as *mut c_void,
            Ordering::SeqCst,
        );
        std::mem::forget(timer_value_field);

        // Unit dropdown (to the right of the number field)
        let timer_unit_dropdown = NSPopUpButton::new(mtm);
        timer_unit_dropdown.setFrame(CGRect {
            origin: CGPoint {
                x: margin + number_field_width + spacing,
                y: y_offset,
            },
            size: CGSize {
                width: dropdown_width,
                height: field_height,
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
            Ordering::SeqCst,
        );
        std::mem::forget(timer_unit_dropdown);

        y_offset -= label_height + 2.0;
        let timer_validation = NSTextField::new(mtm);
        timer_validation.setFrame(CGRect {
            origin: CGPoint {
                x: margin,
                y: y_offset,
            },
            size: CGSize {
                width: field_width,
                height: label_height,
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
            Ordering::SeqCst,
        );
        std::mem::forget(timer_validation);

        // ========================================
        // Overlay Opacity Section
        // ========================================
        y_offset -= section_spacing;
        y_offset -= label_height;
        let opacity_label = create_label(
            mtm,
            "Overlay Opacity:",
            CGRect {
                origin: CGPoint {
                    x: margin,
                    y: y_offset,
                },
                size: CGSize {
                    width: 120.0,
                    height: label_height,
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
                x: window_width - margin - 50.0,
                y: y_offset,
            },
            size: CGSize {
                width: 50.0,
                height: label_height,
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
            Ordering::SeqCst,
        );
        std::mem::forget(percentage_label);

        y_offset -= field_height + row_spacing;

        // Slider with min/max labels
        let slider_margin = 35.0;
        let slider_width = field_width - (slider_margin * 2.0);

        // Min label (20%)
        let min_label = create_label(
            mtm,
            "20%",
            CGRect {
                origin: CGPoint {
                    x: margin,
                    y: y_offset,
                },
                size: CGSize {
                    width: 30.0,
                    height: field_height,
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
                    x: window_width - margin - 30.0,
                    y: y_offset,
                },
                size: CGSize {
                    width: 30.0,
                    height: field_height,
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
                    x: margin + slider_margin,
                    y: y_offset,
                },
                size: CGSize {
                    width: slider_width,
                    height: field_height,
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
            Ordering::SeqCst,
        );
        std::mem::forget(opacity_slider);

        // ========================================
        // Buttons Section
        // ========================================
        let button_height: CGFloat = 28.0;
        let button_width: CGFloat = 80.0;
        let reset_button_width: CGFloat = 120.0;
        let button_spacing: CGFloat = 12.0;
        let button_y: CGFloat = margin;

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
                    x: margin,
                    y: button_y,
                },
                size: CGSize {
                    width: reset_button_width,
                    height: button_height,
                },
            });
            button.setButtonType(NSButtonType::MomentaryPushIn);
            button
        };
        content_view.addSubview(&reset_button);
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
                    x: window_width - margin - button_width - button_spacing - button_width,
                    y: button_y,
                },
                size: CGSize {
                    width: button_width,
                    height: button_height,
                },
            });
            button.setButtonType(NSButtonType::MomentaryPushIn);
            button.setKeyEquivalent(ns_string!("\u{1b}")); // Escape key
            button
        };
        content_view.addSubview(&cancel_button);
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
                    x: window_width - margin - button_width,
                    y: button_y,
                },
                size: CGSize {
                    width: button_width,
                    height: button_height,
                },
            });
            button.setButtonType(NSButtonType::MomentaryPushIn);
            button.setKeyEquivalent(ns_string!("\r")); // Return key
            button
        };
        content_view.addSubview(&save_button);
        std::mem::forget(save_button);
    }

    // Activate the application so the window can receive focus
    // This is needed because the app runs in accessory mode (no dock icon)
    let app = NSApplication::sharedApplication(mtm);
    #[allow(deprecated)]
    app.activateIgnoringOtherApps(true);

    // Show the window
    panel.makeKeyAndOrderFront(None);

    // Keep panel alive
    std::mem::forget(panel);

    println!("  Settings window opened");
}
