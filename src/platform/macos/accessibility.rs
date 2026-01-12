//! Accessibility permission handling for Cat Shield

use super::bindings::{
    kAXTrustedCheckOptionPrompt, kCFBooleanTrue, AXIsProcessTrusted, AXIsProcessTrustedWithOptions,
    CFDictionaryCreate, CFRelease,
};
use objc2_app_kit::NSWorkspace;
use objc2_foundation::{ns_string, NSURL};

/// Check if we have accessibility permissions
pub fn check_accessibility() -> bool {
    unsafe { AXIsProcessTrusted() }
}

/// Check accessibility permissions and prompt user with native dialog if not granted
pub fn check_accessibility_with_prompt() -> bool {
    unsafe {
        let keys = [kAXTrustedCheckOptionPrompt];
        let values = [kCFBooleanTrue];

        let dict = CFDictionaryCreate(
            std::ptr::null(),
            keys.as_ptr(),
            values.as_ptr(),
            1,
            std::ptr::null(),
            std::ptr::null(),
        );

        let result = AXIsProcessTrustedWithOptions(dict);

        if !dict.is_null() {
            CFRelease(dict);
        }

        result
    }
}

/// Open System Settings to the Accessibility privacy pane
pub fn open_accessibility_settings() -> bool {
    let url_string =
        ns_string!("x-apple.systempreferences:com.apple.preference.security?Privacy_Accessibility");

    if let Some(url) = NSURL::URLWithString(url_string) {
        let workspace = NSWorkspace::sharedWorkspace();
        return workspace.openURL(&url);
    }
    false
}
