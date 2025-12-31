//! Cat Shield - A cat-proof screen overlay for macOS
//!
//! Creates a semi-transparent overlay that:
//! - Blocks all keyboard and mouse input
//! - Keeps the machine awake
//! - Unlocks with Cmd+Option+U
//!
//! Usage: Run the application, and it will immediately activate the shield.
//! Press Cmd+Option+U to unlock and exit.
//!
//! IMPORTANT: This app requires Accessibility permissions to block input.
//! Go to System Preferences → Security & Privacy → Privacy → Accessibility
//! and add this application.

use cocoa::appkit::{
    NSApp, NSApplication, NSApplicationActivationPolicy, NSBackingStoreType, NSColor, NSScreen,
    NSWindow, NSWindowCollectionBehavior, NSWindowStyleMask,
};
use cocoa::base::{nil, NO};
use cocoa::foundation::{NSAutoreleasePool, NSRect, NSString};
use core_foundation::base::TCFType;
use core_foundation::runloop::{kCFRunLoopCommonModes, CFRunLoop};
use core_foundation::string::CFString;
use core_graphics::event::{CGEventTapLocation, CGEventTapOptions, CGEventTapPlacement, CGEventType};
use objc::{msg_send, sel, sel_impl};
use std::ffi::c_void;
use std::process;
use std::sync::atomic::Ordering;

// IOKit power management bindings
#[link(name = "IOKit", kind = "framework")]
extern "C" {
    fn IOPMAssertionCreateWithName(
        assertion_type: *const c_void,
        level: u32,
        reason_for_activity: *const c_void,
        assertion_id: *mut u32,
    ) -> i32;
    fn IOPMAssertionRelease(assertion_id: u32) -> i32;
}

// CoreGraphics event tap bindings
#[link(name = "CoreGraphics", kind = "framework")]
extern "C" {
    fn CGEventTapCreate(
        tap: CGEventTapLocation,
        place: CGEventTapPlacement,
        options: CGEventTapOptions,
        events_of_interest: u64,
        callback: extern "C" fn(
            proxy: *mut c_void,
            event_type: CGEventType,
            event: *mut c_void,
            user_info: *mut c_void,
        ) -> *mut c_void,
        user_info: *mut c_void,
    ) -> *mut c_void;

    fn CFMachPortCreateRunLoopSource(
        allocator: *const c_void,
        port: *mut c_void,
        order: i64,
    ) -> *mut c_void;

    fn CGEventTapEnable(tap: *mut c_void, enable: bool);

    fn AXIsProcessTrusted() -> bool;
}

const K_IOPM_ASSERTION_LEVEL_ON: u32 = 255;

// Keycode for 'U' on macOS
const KEY_U: i64 = 32;

// CoreGraphics event constants
const K_CG_KEYBOARD_EVENT_KEYCODE: u32 = 9;
const K_CG_EVENT_FLAG_MASK_COMMAND: u64 = 0x100000;
const K_CG_EVENT_FLAG_MASK_ALTERNATE: u64 = 0x80000;

// Window levels from NSWindow.h
const NS_SCREEN_SAVER_WINDOW_LEVEL: i64 = 1000;

// Global pointer to the event tap for re-enabling from callback
static EVENT_TAP: std::sync::atomic::AtomicPtr<c_void> =
    std::sync::atomic::AtomicPtr::new(std::ptr::null_mut());

/// Creates an IOKit assertion to prevent the system from sleeping
fn prevent_sleep() -> Option<u32> {
    let assertion_type = CFString::new("PreventUserIdleDisplaySleep");
    let reason = CFString::new("Cat Shield is active - protecting your work from cats!");

    let mut assertion_id: u32 = 0;

    let result = unsafe {
        IOPMAssertionCreateWithName(
            assertion_type.as_concrete_TypeRef() as *const c_void,
            K_IOPM_ASSERTION_LEVEL_ON,
            reason.as_concrete_TypeRef() as *const c_void,
            &mut assertion_id,
        )
    };

    if result == 0 {
        println!("  ✓ Sleep prevention enabled");
        Some(assertion_id)
    } else {
        eprintln!("  ✗ Failed to create power assertion: {}", result);
        None
    }
}

/// Releases the sleep prevention assertion
fn allow_sleep(assertion_id: u32) {
    let result = unsafe { IOPMAssertionRelease(assertion_id) };
    if result == 0 {
        println!("  ✓ Sleep prevention disabled");
    }
}

/// Callback for the CGEventTap - intercepts and blocks events
extern "C" fn event_tap_callback(
    _proxy: *mut c_void,
    event_type: CGEventType,
    event: *mut c_void,
    _user_info: *mut c_void,
) -> *mut c_void {
    // Handle tap disabled event (system can disable taps if they're too slow)
    if matches!(
        event_type,
        CGEventType::TapDisabledByTimeout | CGEventType::TapDisabledByUserInput
    ) {
        eprintln!("  ⚠️  Event tap was disabled, re-enabling...");
        // Re-enable the tap using the stored pointer
        let tap = EVENT_TAP.load(Ordering::SeqCst);
        if !tap.is_null() {
            unsafe {
                CGEventTapEnable(tap, true);
            }
        }
        return event;
    }

    // Check for unlock combination: Cmd+Option+U
    if matches!(event_type, CGEventType::KeyDown) {
        unsafe {
            // Get event flags and keycode using CoreGraphics functions
            #[link(name = "CoreGraphics", kind = "framework")]
            extern "C" {
                fn CGEventGetFlags(event: *mut c_void) -> u64;
                fn CGEventGetIntegerValueField(event: *mut c_void, field: u32) -> i64;
            }

            let flags = CGEventGetFlags(event);
            let keycode = CGEventGetIntegerValueField(event, K_CG_KEYBOARD_EVENT_KEYCODE);

            // Check for Cmd + Option + U key
            let cmd_pressed = (flags & K_CG_EVENT_FLAG_MASK_COMMAND) != 0;
            let option_pressed = (flags & K_CG_EVENT_FLAG_MASK_ALTERNATE) != 0;

            if cmd_pressed && option_pressed && keycode == KEY_U {
                println!("\n  🔓 Unlock combination detected (Cmd+Option+U)!");

                // Stop the run loop to allow clean exit
                CFRunLoop::get_current().stop();

                // Let this event through
                return event;
            }
        }
    }

    // Block all keyboard and mouse events by returning NULL
    match event_type {
        CGEventType::KeyDown
        | CGEventType::KeyUp
        | CGEventType::FlagsChanged
        | CGEventType::LeftMouseDown
        | CGEventType::LeftMouseUp
        | CGEventType::RightMouseDown
        | CGEventType::RightMouseUp
        | CGEventType::MouseMoved
        | CGEventType::LeftMouseDragged
        | CGEventType::RightMouseDragged
        | CGEventType::ScrollWheel
        | CGEventType::OtherMouseDown
        | CGEventType::OtherMouseUp
        | CGEventType::OtherMouseDragged => {
            // Return NULL to block the event
            std::ptr::null_mut()
        }
        _ => event,
    }
}

/// Check if we have accessibility permissions
fn check_accessibility() -> bool {
    unsafe { AXIsProcessTrusted() }
}

/// Create and enable the event tap
fn setup_event_tap() -> bool {
    // Define event mask for all keyboard and mouse events
    let event_mask: u64 = (1 << CGEventType::KeyDown as u64)
        | (1 << CGEventType::KeyUp as u64)
        | (1 << CGEventType::FlagsChanged as u64)
        | (1 << CGEventType::LeftMouseDown as u64)
        | (1 << CGEventType::LeftMouseUp as u64)
        | (1 << CGEventType::RightMouseDown as u64)
        | (1 << CGEventType::RightMouseUp as u64)
        | (1 << CGEventType::MouseMoved as u64)
        | (1 << CGEventType::LeftMouseDragged as u64)
        | (1 << CGEventType::RightMouseDragged as u64)
        | (1 << CGEventType::ScrollWheel as u64)
        | (1 << CGEventType::OtherMouseDown as u64)
        | (1 << CGEventType::OtherMouseUp as u64)
        | (1 << CGEventType::OtherMouseDragged as u64);

    unsafe {
        // Create the event tap
        let tap = CGEventTapCreate(
            CGEventTapLocation::HID, // Intercept at the HID level (earliest)
            CGEventTapPlacement::HeadInsertEventTap,
            CGEventTapOptions::Default, // Active tap that can modify/block events
            event_mask,
            event_tap_callback,
            std::ptr::null_mut(),
        );

        if tap.is_null() {
            return false;
        }

        // Store the tap pointer globally so we can re-enable it from the callback
        EVENT_TAP.store(tap, Ordering::SeqCst);

        // Create a run loop source and add it to the current run loop
        let run_loop_source = CFMachPortCreateRunLoopSource(std::ptr::null(), tap, 0);

        if run_loop_source.is_null() {
            // Clean up the tap to avoid resource leak
            #[link(name = "CoreFoundation", kind = "framework")]
            extern "C" {
                fn CFMachPortInvalidate(port: *mut c_void);
                fn CFRelease(cf: *mut c_void);
            }
            CFMachPortInvalidate(tap);
            CFRelease(tap);
            EVENT_TAP.store(std::ptr::null_mut(), Ordering::SeqCst);
            return false;
        }

        // Add to run loop
        #[link(name = "CoreFoundation", kind = "framework")]
        extern "C" {
            fn CFRunLoopAddSource(rl: *mut c_void, source: *mut c_void, mode: *const c_void);
            fn CFRunLoopGetCurrent() -> *mut c_void;
        }

        let current_run_loop = CFRunLoopGetCurrent();
        CFRunLoopAddSource(
            current_run_loop,
            run_loop_source,
            kCFRunLoopCommonModes as *const _ as *const c_void,
        );

        // Enable the tap
        CGEventTapEnable(tap, true);

        true
    }
}

fn main() {
    println!();
    println!("  🐱 CAT SHIELD 🛡️");
    println!("  ════════════════════════════════════════");
    println!("  Protecting your work from curious cats!");
    println!();

    // Check accessibility permissions first
    if !check_accessibility() {
        eprintln!("  ⚠️  ACCESSIBILITY PERMISSION REQUIRED");
        eprintln!();
        eprintln!("  To block keyboard/mouse input, this app needs");
        eprintln!("  Accessibility permissions:");
        eprintln!();
        eprintln!("  1. Open System Settings");
        eprintln!("  2. Go to Privacy & Security → Accessibility");
        eprintln!("  3. Click '+' and add this application");
        eprintln!("  4. Restart Cat Shield");
        eprintln!();
        eprintln!("  The app will now run in LIMITED MODE");
        eprintln!("  (overlay + sleep prevention only)");
        eprintln!();
    }

    unsafe {
        // Create autorelease pool
        let _pool = NSAutoreleasePool::new(nil);

        // Initialize the application
        let app = NSApp();
        app.setActivationPolicy_(
            NSApplicationActivationPolicy::NSApplicationActivationPolicyAccessory,
        );

        // Get the main screen dimensions
        let screen = NSScreen::mainScreen(nil);
        if screen == nil {
            eprintln!("  ✗ Failed to get main screen");
            process::exit(1);
        }
        let screen_frame: NSRect = msg_send![screen, frame];

        // Create a fullscreen, borderless window
        let window = NSWindow::alloc(nil).initWithContentRect_styleMask_backing_defer_(
            screen_frame,
            NSWindowStyleMask::NSBorderlessWindowMask,
            NSBackingStoreType::NSBackingStoreBuffered,
            NO,
        );

        // Configure window to be topmost
        let _: () = msg_send![window, setLevel: NS_SCREEN_SAVER_WINDOW_LEVEL];

        // Set window to appear on all spaces and stay visible
        window.setCollectionBehavior_(
            NSWindowCollectionBehavior::NSWindowCollectionBehaviorCanJoinAllSpaces
                | NSWindowCollectionBehavior::NSWindowCollectionBehaviorStationary
                | NSWindowCollectionBehavior::NSWindowCollectionBehaviorIgnoresCycle,
        );

        // Make window semi-transparent (30% opacity)
        window.setOpaque_(NO);
        window.setAlphaValue_(0.3);

        // Set a dark background color
        let bg_color = NSColor::colorWithRed_green_blue_alpha_(nil, 0.1, 0.1, 0.15, 1.0);
        window.setBackgroundColor_(bg_color);

        // Keep window visible
        window.setHidesOnDeactivate_(NO);

        // Accept mouse events (needed for blocking)
        window.setIgnoresMouseEvents_(NO);

        // Set title
        let title = NSString::alloc(nil).init_str("Cat Shield");
        window.setTitle_(title);

        // Show the window
        window.makeKeyAndOrderFront_(nil);

        println!("  ✓ Overlay window active");

        // Prevent sleep
        let assertion_id = prevent_sleep();

        // Set up event tap if we have permissions
        if check_accessibility() {
            if setup_event_tap() {
                println!("  ✓ Input blocking active");
            } else {
                eprintln!("  ✗ Failed to create event tap");
            }
        }

        println!();
        println!("  ═══════════════════════════════════════");
        println!("  🛡️  CAT SHIELD IS NOW ACTIVE!");
        println!("  ═══════════════════════════════════════");
        println!();
        println!("  Press Cmd+Option+U to unlock and exit");
        println!();

        // Run the event loop
        CFRunLoop::run_current();

        // Cleanup
        if let Some(id) = assertion_id {
            allow_sleep(id);
        }
    }

    println!();
    println!("  👋 Cat Shield deactivated. Goodbye!");
    println!();
}
