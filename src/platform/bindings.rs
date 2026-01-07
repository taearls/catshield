//! FFI bindings for macOS frameworks
//!
//! Contains extern "C" declarations for IOKit, CoreGraphics,
//! ApplicationServices, and CoreFoundation functions.

use std::ffi::c_void;

// IOKit power management bindings
#[link(name = "IOKit", kind = "framework")]
extern "C" {
    pub fn IOPMAssertionCreateWithName(
        assertion_type: *const c_void,
        level: u32,
        reason_for_activity: *const c_void,
        assertion_id: *mut u32,
    ) -> i32;
    pub fn IOPMAssertionRelease(assertion_id: u32) -> i32;
}

// Additional CoreGraphics functions not in objc2-core-graphics
#[link(name = "CoreGraphics", kind = "framework")]
extern "C" {
    pub fn CGEventTapEnable(tap: *mut c_void, enable: bool);
    pub fn AXIsProcessTrusted() -> bool;
}

// ApplicationServices framework for accessibility permission prompting
#[link(name = "ApplicationServices", kind = "framework")]
extern "C" {
    pub fn AXIsProcessTrustedWithOptions(options: *const c_void) -> bool;
}

// Accessibility options key
#[link(name = "ApplicationServices", kind = "framework")]
extern "C" {
    pub static kAXTrustedCheckOptionPrompt: *const c_void;
}

// CoreFoundation bindings
#[link(name = "CoreFoundation", kind = "framework")]
extern "C" {
    // Run loop source management
    pub fn CFMachPortCreateRunLoopSource(
        allocator: *const c_void,
        port: *mut c_void,
        order: i64,
    ) -> *mut c_void;
    pub fn CFRunLoopAddSource(rl: *mut c_void, source: *mut c_void, mode: *const c_void);
    pub fn CFRunLoopRemoveSource(rl: *mut c_void, source: *mut c_void, mode: *const c_void);

    // Run loop access
    pub fn CFRunLoopGetCurrent() -> *mut c_void;

    // Timer management
    pub fn CFRunLoopAddTimer(rl: *mut c_void, timer: *mut c_void, mode: *const c_void);
    pub fn CFRunLoopTimerCreate(
        allocator: *const c_void,
        fire_date: f64,
        interval: f64,
        flags: u32,
        order: i64,
        callout: unsafe extern "C" fn(*mut c_void, *mut c_void),
        context: *const c_void,
    ) -> *mut c_void;
    pub fn CFRunLoopTimerInvalidate(timer: *mut c_void);
    pub fn CFAbsoluteTimeGetCurrent() -> f64;

    // Run loop execution (for polling with event processing)
    pub fn CFRunLoopRunInMode(
        mode: *const c_void,
        seconds: f64,
        return_after_source_handled: bool,
    ) -> i32;

    // Dictionary creation for accessibility options
    pub static kCFBooleanTrue: *const c_void;
    pub fn CFDictionaryCreate(
        allocator: *const c_void,
        keys: *const *const c_void,
        values: *const *const c_void,
        num_values: isize,
        key_callbacks: *const c_void,
        value_callbacks: *const c_void,
    ) -> *mut c_void;
    pub fn CFRelease(cf: *const c_void);
}

// Declare kill function from libc for process existence check
extern "C" {
    pub fn kill(pid: i32, sig: i32) -> i32;
}

/// IOKit power management assertion level constant
pub const K_IOPM_ASSERTION_LEVEL_ON: u32 = 255;
