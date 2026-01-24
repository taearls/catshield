//! Login item management for macOS
//!
//! Uses the ServiceManagement framework (SMAppService) to register/unregister
//! the app as a login item. This is the modern approach for macOS 13+.
//!
//! # Technical Details
//!
//! On macOS 13 (Ventura) and later, `SMAppService.mainApp` provides a simple API
//! to register the current app as a login item. This appears in System Settings
//! under General > Login Items.
//!
//! For non-sandboxed apps (like Cat Shield), this requires the app to be in
//! /Applications or a similar standard location for the system to recognize it.

use objc2::msg_send;
use objc2::runtime::{AnyClass, AnyObject, Bool};

/// Check if the app is currently registered as a login item
pub fn is_login_item_enabled() -> bool {
    unsafe {
        // Get the SMAppService class
        let class = match AnyClass::get(c"SMAppService") {
            Some(c) => c,
            None => {
                log::warn!("SMAppService class not available (requires macOS 13+)");
                return false;
            }
        };

        // Get the main app service: [SMAppService mainAppService]
        let service: *mut AnyObject = msg_send![class, mainAppService];
        if service.is_null() {
            log::warn!("Failed to get main app service");
            return false;
        }

        // Check status: [service status]
        // SMAppServiceStatus:
        //   0 = notRegistered
        //   1 = enabled
        //   2 = requiresApproval
        //   3 = notFound
        let status: isize = msg_send![service, status];
        log::debug!("Login item status: {status}");

        // Status 1 means enabled
        status == 1
    }
}

/// Register the app as a login item
///
/// Returns Ok(()) on success, or Err with an error message on failure.
pub fn register_login_item() -> Result<(), String> {
    unsafe {
        // Get the SMAppService class
        let class = match AnyClass::get(c"SMAppService") {
            Some(c) => c,
            None => {
                return Err("SMAppService not available (requires macOS 13+)".to_string());
            }
        };

        // Get the main app service
        let service: *mut AnyObject = msg_send![class, mainAppService];
        if service.is_null() {
            return Err("Failed to get main app service".to_string());
        }

        // Register: [service registerAndReturnError:]
        // We need to pass a pointer to an NSError* to receive any error
        let mut error: *mut AnyObject = std::ptr::null_mut();
        let success: Bool = msg_send![service, registerAndReturnError: &mut error];

        if success.as_bool() {
            log::info!("Successfully registered as login item");
            Ok(())
        } else {
            // Try to get error description
            let error_msg = if !error.is_null() {
                let desc: *mut AnyObject = msg_send![error, localizedDescription];
                if !desc.is_null() {
                    let utf8: *const std::ffi::c_char = msg_send![desc, UTF8String];
                    if !utf8.is_null() {
                        std::ffi::CStr::from_ptr(utf8)
                            .to_string_lossy()
                            .into_owned()
                    } else {
                        "Unknown error".to_string()
                    }
                } else {
                    "Unknown error".to_string()
                }
            } else {
                "Unknown error".to_string()
            };

            log::error!("Failed to register as login item: {error_msg}");
            Err(error_msg)
        }
    }
}

/// Unregister the app as a login item
///
/// Returns Ok(()) on success, or Err with an error message on failure.
pub fn unregister_login_item() -> Result<(), String> {
    unsafe {
        // Get the SMAppService class
        let class = match AnyClass::get(c"SMAppService") {
            Some(c) => c,
            None => {
                return Err("SMAppService not available (requires macOS 13+)".to_string());
            }
        };

        // Get the main app service
        let service: *mut AnyObject = msg_send![class, mainAppService];
        if service.is_null() {
            return Err("Failed to get main app service".to_string());
        }

        // Unregister: [service unregisterAndReturnError:]
        let mut error: *mut AnyObject = std::ptr::null_mut();
        let success: Bool = msg_send![service, unregisterAndReturnError: &mut error];

        if success.as_bool() {
            log::info!("Successfully unregistered as login item");
            Ok(())
        } else {
            // Try to get error description
            let error_msg = if !error.is_null() {
                let desc: *mut AnyObject = msg_send![error, localizedDescription];
                if !desc.is_null() {
                    let utf8: *const std::ffi::c_char = msg_send![desc, UTF8String];
                    if !utf8.is_null() {
                        std::ffi::CStr::from_ptr(utf8)
                            .to_string_lossy()
                            .into_owned()
                    } else {
                        "Unknown error".to_string()
                    }
                } else {
                    "Unknown error".to_string()
                }
            } else {
                "Unknown error".to_string()
            };

            log::error!("Failed to unregister as login item: {error_msg}");
            Err(error_msg)
        }
    }
}

/// Set the login item state (register or unregister based on the boolean)
///
/// This is a convenience function that calls either `register_login_item()` or
/// `unregister_login_item()` based on the `enabled` parameter.
pub fn set_login_item_enabled(enabled: bool) -> Result<(), String> {
    if enabled {
        register_login_item()
    } else {
        unregister_login_item()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Note: These tests are marked as #[ignore] because they require macOS 13+
    // and actually modify system login items. Run manually when needed.

    #[test]
    #[ignore = "Requires macOS 13+ and modifies system state"]
    fn test_is_login_item_enabled() {
        // Just check that the function doesn't panic
        let _ = is_login_item_enabled();
    }

    #[test]
    #[ignore = "Requires macOS 13+ and modifies system state"]
    fn test_register_and_unregister() {
        // Register
        let register_result = register_login_item();
        println!("Register result: {:?}", register_result);

        // Check status
        let is_enabled = is_login_item_enabled();
        println!("Is enabled after register: {}", is_enabled);

        // Unregister
        let unregister_result = unregister_login_item();
        println!("Unregister result: {:?}", unregister_result);

        // Check status again
        let is_enabled_after = is_login_item_enabled();
        println!("Is enabled after unregister: {}", is_enabled_after);
    }
}
