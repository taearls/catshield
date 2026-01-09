//! Event tap for input blocking in Cat Shield
//!
//! Creates and manages a CGEventTap to intercept and block keyboard input.

use super::bindings::{
    CFMachPortCreateRunLoopSource, CFRelease, CFRunLoopAddSource, CFRunLoopGetCurrent,
    CFRunLoopRemoveSource, CGEventTapEnable,
};
use crate::input::check_exit_key;
use crate::ui::state::shield;
use objc2_app_kit::NSApplication;
use objc2_core_foundation::{kCFRunLoopCommonModes, CFMachPort, CFRetained, CFString};
use objc2_core_graphics::{
    CGEvent, CGEventField, CGEventMask, CGEventTapLocation, CGEventTapOptions, CGEventTapPlacement,
    CGEventTapProxy, CGEventType,
};
use objc2_foundation::MainThreadMarker;
use std::ffi::c_void;
use std::ptr::NonNull;
use std::sync::atomic::{AtomicPtr, Ordering};

/// Global pointer to the event tap for re-enabling from callback
pub static EVENT_TAP: AtomicPtr<c_void> = AtomicPtr::new(std::ptr::null_mut());
/// Global pointer to the event tap's run loop source for cleanup
pub static EVENT_TAP_RUN_LOOP_SOURCE: AtomicPtr<c_void> = AtomicPtr::new(std::ptr::null_mut());

/// Callback for the CGEventTap - intercepts and blocks events
unsafe extern "C-unwind" fn event_tap_callback(
    _proxy: CGEventTapProxy,
    event_type: CGEventType,
    event: NonNull<CGEvent>,
    _user_info: *mut c_void,
) -> *mut CGEvent {
    // Handle tap disabled event (system can disable taps if they're too slow)
    if event_type == CGEventType::TapDisabledByTimeout
        || event_type == CGEventType::TapDisabledByUserInput
    {
        eprintln!("  ⚠️  Event tap was disabled, re-enabling...");
        // Re-enable the tap using the stored pointer
        let tap = EVENT_TAP.load(Ordering::SeqCst);
        if !tap.is_null() {
            CGEventTapEnable(tap, true);
        }
        return event.as_ptr();
    }

    // Check for configured exit key combination
    if event_type == CGEventType::KeyDown {
        let cg_event = event.as_ref();

        let flags = CGEvent::flags(Some(cg_event));
        let keycode =
            CGEvent::integer_value_field(Some(cg_event), CGEventField::KeyboardEventKeycode);

        // Check if the key combination matches the configured exit key
        if check_exit_key(keycode, flags) {
            println!("\n  🔓 Exit key combination detected!");

            // In menu bar mode, deactivate shield and return to menu bar
            // In immediate mode, terminate the app
            if shield::MODE_MENU_BAR.load(Ordering::SeqCst) {
                crate::ui::shield::deactivate_shield();
            } else if let Some(mtm) = MainThreadMarker::new() {
                let app = NSApplication::sharedApplication(mtm);
                app.terminate(None);
            }

            // Let this event through
            return event.as_ptr();
        }
    }

    // Block keyboard events by returning NULL
    // Mouse events are allowed through so our close button can work
    // (our topmost window captures all mouse events anyway)
    if event_type == CGEventType::KeyDown
        || event_type == CGEventType::KeyUp
        || event_type == CGEventType::FlagsChanged
    {
        // Return NULL to block the event
        return std::ptr::null_mut();
    }

    event.as_ptr()
}

/// Create and enable the event tap
pub fn setup_event_tap() -> bool {
    // Define event mask for keyboard events only
    // Mouse events are NOT blocked - our topmost fullscreen window captures them,
    // and we need mouse events to reach our close button
    let event_mask: CGEventMask = (1u64 << CGEventType::KeyDown.0)
        | (1u64 << CGEventType::KeyUp.0)
        | (1u64 << CGEventType::FlagsChanged.0);

    unsafe {
        // Create the event tap using CGEvent::tap_create
        let tap_opt = CGEvent::tap_create(
            CGEventTapLocation::HIDEventTap, // Intercept at the HID level (earliest)
            CGEventTapPlacement::HeadInsertEventTap,
            CGEventTapOptions::Default, // Active tap that can modify/block events
            event_mask,
            Some(event_tap_callback),
            std::ptr::null_mut(),
        );

        let tap: CFRetained<CFMachPort> = match tap_opt {
            Some(t) => t,
            None => return false,
        };

        // Get raw pointer for storing and run loop source creation
        let tap_ptr = CFRetained::as_ptr(&tap).as_ptr() as *mut c_void;

        // Store the tap pointer globally so we can re-enable it from the callback
        EVENT_TAP.store(tap_ptr, Ordering::SeqCst);

        // Create a run loop source and add it to the current run loop
        let run_loop_source = CFMachPortCreateRunLoopSource(std::ptr::null(), tap_ptr, 0);

        if run_loop_source.is_null() {
            EVENT_TAP.store(std::ptr::null_mut(), Ordering::SeqCst);
            return false;
        }

        // Store run loop source for cleanup in disable_event_tap()
        EVENT_TAP_RUN_LOOP_SOURCE.store(run_loop_source, Ordering::SeqCst);

        // Add to run loop
        let current_run_loop = CFRunLoopGetCurrent();
        let run_loop_mode = kCFRunLoopCommonModes.expect("kCFRunLoopCommonModes should exist");
        CFRunLoopAddSource(
            current_run_loop,
            run_loop_source,
            (run_loop_mode as *const CFString) as *const c_void,
        );

        // Enable the tap
        CGEventTapEnable(tap_ptr, true);

        // Transfer ownership of CFMachPort to raw pointer stored in EVENT_TAP.
        // We call std::mem::forget to prevent CFRetained from releasing it here;
        // instead, we'll release it manually via CFRelease in disable_event_tap().
        std::mem::forget(tap);

        true
    }
}

/// Disable and clean up the event tap
pub fn disable_event_tap() {
    let tap_ptr = EVENT_TAP.swap(std::ptr::null_mut(), Ordering::SeqCst);
    let source_ptr = EVENT_TAP_RUN_LOOP_SOURCE.swap(std::ptr::null_mut(), Ordering::SeqCst);

    if !tap_ptr.is_null() {
        unsafe {
            // Disable the tap
            CGEventTapEnable(tap_ptr, false);

            // Remove the run loop source from the run loop before releasing
            if !source_ptr.is_null() {
                let current_run_loop = CFRunLoopGetCurrent();
                let run_loop_mode =
                    kCFRunLoopCommonModes.expect("kCFRunLoopCommonModes should exist");
                CFRunLoopRemoveSource(
                    current_run_loop,
                    source_ptr,
                    (run_loop_mode as *const CFString) as *const c_void,
                );
                // Release the run loop source
                CFRelease(source_ptr);
            }

            // Release the CFMachPort
            CFRelease(tap_ptr);
        }
        println!("  ✓ Input blocking disabled");
    }
}
