//! Power management for Cat Shield
//!
//! Provides functions to prevent and allow system sleep.

use super::bindings::{
    IOPMAssertionCreateWithName, IOPMAssertionRelease, K_IOPM_ASSERTION_LEVEL_ON,
};
use objc2_core_foundation::{CFRetained, CFString};
use std::ffi::c_void;

/// Creates an IOKit assertion to prevent the system from sleeping
pub fn prevent_sleep() -> Option<u32> {
    let assertion_type = CFString::from_static_str("PreventUserIdleDisplaySleep");
    let reason =
        CFString::from_static_str("Cat Shield is active - protecting your work from cats!");

    let mut assertion_id: u32 = 0;

    let result = unsafe {
        IOPMAssertionCreateWithName(
            CFRetained::as_ptr(&assertion_type).as_ptr() as *const c_void,
            K_IOPM_ASSERTION_LEVEL_ON,
            CFRetained::as_ptr(&reason).as_ptr() as *const c_void,
            &mut assertion_id,
        )
    };

    if result == 0 {
        log::info!("✓ Sleep prevention enabled");
        Some(assertion_id)
    } else {
        log::error!("Failed to create power assertion: {}", result);
        None
    }
}

/// Releases the sleep prevention assertion
pub fn allow_sleep(assertion_id: u32) {
    let result = unsafe { IOPMAssertionRelease(assertion_id) };
    if result == 0 {
        log::info!("✓ Sleep prevention disabled");
    }
}
