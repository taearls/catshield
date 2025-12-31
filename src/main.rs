//! Cat Shield - A cat-proof screen overlay for macOS
//!
//! Creates a semi-transparent overlay that:
//! - Blocks all keyboard and mouse input
//! - Keeps the machine awake
//! - Click and hold close button (3 seconds) to exit
//! - Or unlock with Cmd+Option+U (requires Accessibility permissions)
//!
//! Usage: Run the application, and it will immediately activate the shield.
//! Click and hold the X button in the top-right corner for 3 seconds to exit.
//!
//! Optional: For keyboard shortcut exit (Cmd+Option+U), grant Accessibility permissions:
//! Go to System Preferences → Security & Privacy → Privacy → Accessibility
//! and add this application.

use objc2::rc::Retained;
use objc2::{define_class, msg_send, MainThreadOnly};
use objc2_app_kit::{
    NSApplication, NSApplicationActivationPolicy, NSBackingStoreType, NSBezierPath, NSColor,
    NSEvent, NSScreen, NSView, NSWindow, NSWindowCollectionBehavior, NSWindowStyleMask,
};
use objc2_core_foundation::{
    kCFRunLoopCommonModes, CFMachPort, CFRetained, CFString, CGFloat, CGPoint, CGRect, CGSize,
};
use objc2_core_graphics::{
    CGEvent, CGEventField, CGEventFlags, CGEventMask, CGEventTapLocation, CGEventTapOptions,
    CGEventTapPlacement, CGEventTapProxy, CGEventType,
};
use objc2_foundation::{ns_string, MainThreadMarker};
use std::cell::Cell;
use std::ffi::c_void;
use std::process;
use std::ptr::NonNull;
use std::sync::atomic::{AtomicPtr, Ordering};
use std::time::Instant;

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

// Additional CoreGraphics functions not in objc2-core-graphics
#[link(name = "CoreGraphics", kind = "framework")]
extern "C" {
    fn CGEventTapEnable(tap: *mut c_void, enable: bool);
    fn AXIsProcessTrusted() -> bool;
}

// CoreFoundation function for creating run loop source from mach port
#[link(name = "CoreFoundation", kind = "framework")]
extern "C" {
    fn CFMachPortCreateRunLoopSource(
        allocator: *const c_void,
        port: *mut c_void,
        order: i64,
    ) -> *mut c_void;
    fn CFRunLoopAddSource(rl: *mut c_void, source: *mut c_void, mode: *const c_void);
    fn CFRunLoopAddTimer(rl: *mut c_void, timer: *mut c_void, mode: *const c_void);
    fn CFRunLoopGetCurrent() -> *mut c_void;
    fn CFRunLoopStop(rl: *mut c_void);
    fn CFRunLoopRun();
}

const K_IOPM_ASSERTION_LEVEL_ON: u32 = 255;

// Keycode for 'U' on macOS
const KEY_U: i64 = 32;

// Close button configuration
const CLOSE_BUTTON_SIZE: CGFloat = 44.0;
const CLOSE_BUTTON_MARGIN: CGFloat = 20.0;
const HOLD_DURATION_SECS: f64 = 3.0;
const TIMER_INTERVAL_SECS: f64 = 1.0 / 60.0; // 60 FPS for smooth animation

// Window levels from NSWindow.h
const NS_SCREEN_SAVER_WINDOW_LEVEL: isize = 1000;

// Global timer reference for cleanup
static TIMER_REF: AtomicPtr<c_void> = AtomicPtr::new(std::ptr::null_mut());

// Global view reference for timer callback
static CLOSE_BUTTON_VIEW: AtomicPtr<c_void> = AtomicPtr::new(std::ptr::null_mut());

// Global pointer to the event tap for re-enabling from callback
static EVENT_TAP: AtomicPtr<c_void> = AtomicPtr::new(std::ptr::null_mut());

// Close button state stored in thread-local for the view
thread_local! {
    static MOUSE_DOWN_TIME: Cell<Option<Instant>> = const { Cell::new(None) };
    static IS_MOUSE_INSIDE: Cell<bool> = const { Cell::new(false) };
}

// Timer callback to update progress, check for exit condition, and trigger redraw
unsafe extern "C" fn timer_callback(_timer: *mut c_void, _info: *mut c_void) {
    // Check if hold duration has been exceeded
    let should_exit = MOUSE_DOWN_TIME.with(|time| {
        if let Some(start) = time.get() {
            let is_inside = IS_MOUSE_INSIDE.with(|inside| inside.get());
            is_inside && start.elapsed().as_secs_f64() >= HOLD_DURATION_SECS
        } else {
            false
        }
    });

    if should_exit {
        CFRunLoopStop(CFRunLoopGetCurrent());
        return;
    }

    // Trigger redraw
    let view_ptr = CLOSE_BUTTON_VIEW.load(Ordering::SeqCst);
    if !view_ptr.is_null() {
        let view: &NSView = &*(view_ptr as *const NSView);
        view.setNeedsDisplay(true);
    }
}

// CoreFoundation timer bindings
#[link(name = "CoreFoundation", kind = "framework")]
extern "C" {
    fn CFRunLoopTimerCreate(
        allocator: *const c_void,
        fire_date: f64,
        interval: f64,
        flags: u32,
        order: i64,
        callout: unsafe extern "C" fn(*mut c_void, *mut c_void),
        context: *const c_void,
    ) -> *mut c_void;
    fn CFAbsoluteTimeGetCurrent() -> f64;
    fn CFRunLoopTimerInvalidate(timer: *mut c_void);
}

/// Start the animation timer for the close button
fn start_close_button_timer() {
    unsafe {
        let timer = CFRunLoopTimerCreate(
            std::ptr::null(),
            CFAbsoluteTimeGetCurrent() + TIMER_INTERVAL_SECS,
            TIMER_INTERVAL_SECS,
            0,
            0,
            timer_callback,
            std::ptr::null(),
        );

        if !timer.is_null() {
            let run_loop = CFRunLoopGetCurrent();
            let mode = kCFRunLoopCommonModes.expect("kCFRunLoopCommonModes should exist");
            CFRunLoopAddTimer(run_loop, timer, (mode as *const CFString) as *const c_void);
            TIMER_REF.store(timer, Ordering::SeqCst);
        }
    }
}

/// Stop the animation timer
fn stop_close_button_timer() {
    unsafe {
        let timer = TIMER_REF.swap(std::ptr::null_mut(), Ordering::SeqCst);
        if !timer.is_null() {
            CFRunLoopTimerInvalidate(timer);
        }
    }
}

/// Ivars for the CloseButtonView
struct CloseButtonViewIvars {}

define_class!(
    #[unsafe(super(NSView))]
    #[name = "CloseButtonView"]
    #[ivars = CloseButtonViewIvars]
    struct CloseButtonView;

    impl CloseButtonView {
        #[unsafe(method(drawRect:))]
        unsafe fn draw_rect(&self, _dirty_rect: CGRect) {
            draw_close_button(self);
        }

        #[unsafe(method(mouseDown:))]
        unsafe fn mouse_down(&self, _event: &NSEvent) {
            MOUSE_DOWN_TIME.with(|time| {
                time.set(Some(Instant::now()));
            });
            IS_MOUSE_INSIDE.with(|inside| inside.set(true));
            self.setNeedsDisplay(true);
        }

        #[unsafe(method(mouseUp:))]
        unsafe fn mouse_up(&self, _event: &NSEvent) {
            MOUSE_DOWN_TIME.with(|time| {
                time.set(None);
            });
            self.setNeedsDisplay(true);
        }

        #[unsafe(method(mouseDragged:))]
        unsafe fn mouse_dragged(&self, event: &NSEvent) {
            // Check if mouse is still inside the button
            let location = event.locationInWindow();
            let bounds = self.bounds();

            // Convert to view coordinates
            let local_point = self.convertPoint_fromView(location, None);

            let is_inside = local_point.x >= 0.0
                && local_point.x <= bounds.size.width
                && local_point.y >= 0.0
                && local_point.y <= bounds.size.height;

            let was_inside = IS_MOUSE_INSIDE.with(|inside| inside.get());

            if is_inside != was_inside {
                IS_MOUSE_INSIDE.with(|inside| inside.set(is_inside));

                // Reset timer if mouse left the button
                if !is_inside {
                    MOUSE_DOWN_TIME.with(|time| {
                        time.set(None);
                    });
                } else {
                    // Restart timer if mouse re-entered
                    MOUSE_DOWN_TIME.with(|time| {
                        time.set(Some(Instant::now()));
                    });
                }
            }

            self.setNeedsDisplay(true);
        }
    }
);

impl CloseButtonView {
    fn new(mtm: MainThreadMarker, frame: CGRect) -> Retained<Self> {
        let this = mtm.alloc::<CloseButtonView>();
        let this = this.set_ivars(CloseButtonViewIvars {});
        unsafe { msg_send![super(this), initWithFrame: frame] }
    }
}

/// Draw the close button with progress indicator
fn draw_close_button(view: &NSView) {
    let bounds = view.bounds();
    let center_x = bounds.size.width / 2.0;
    let center_y = bounds.size.height / 2.0;
    let radius = (bounds.size.width.min(bounds.size.height) / 2.0) - 2.0;

    // Calculate progress (0.0 to 1.0)
    let progress = MOUSE_DOWN_TIME.with(|time| {
        if let Some(start) = time.get() {
            let elapsed = start.elapsed().as_secs_f64();
            (elapsed / HOLD_DURATION_SECS).min(1.0)
        } else {
            0.0
        }
    });

    let is_inside = IS_MOUSE_INSIDE.with(|inside| inside.get());

    // Background circle - semi-transparent
    let bg_color = if is_inside && progress > 0.0 {
        NSColor::colorWithRed_green_blue_alpha(0.3, 0.3, 0.3, 0.9)
    } else {
        NSColor::colorWithRed_green_blue_alpha(0.2, 0.2, 0.2, 0.7)
    };

    bg_color.set();

    let bg_path = NSBezierPath::bezierPathWithOvalInRect(CGRect {
        origin: CGPoint {
            x: center_x - radius,
            y: center_y - radius,
        },
        size: CGSize {
            width: radius * 2.0,
            height: radius * 2.0,
        },
    });
    bg_path.fill();

    // Progress arc (if holding)
    if progress > 0.0 && is_inside {
        let progress_color = NSColor::colorWithRed_green_blue_alpha(0.4, 0.8, 0.4, 1.0);
        progress_color.set();

        // Draw arc from top, going clockwise
        let start_angle = 90.0; // Top of circle
        let end_angle = 90.0 - (progress * 360.0);

        let arc_path = NSBezierPath::bezierPath();
        arc_path.setLineWidth(4.0);

        arc_path.appendBezierPathWithArcWithCenter_radius_startAngle_endAngle_clockwise(
            CGPoint {
                x: center_x,
                y: center_y,
            },
            radius - 3.0,
            start_angle,
            end_angle,
            true, // clockwise
        );
        arc_path.stroke();
    }

    // Draw X
    let x_color = if is_inside {
        NSColor::colorWithRed_green_blue_alpha(1.0, 1.0, 1.0, 1.0)
    } else {
        NSColor::colorWithRed_green_blue_alpha(0.8, 0.8, 0.8, 0.8)
    };

    x_color.set();

    let x_size = radius * 0.5;
    let x_path = NSBezierPath::bezierPath();
    x_path.setLineWidth(3.0);

    // First line of X (top-left to bottom-right)
    x_path.moveToPoint(CGPoint {
        x: center_x - x_size,
        y: center_y + x_size,
    });
    x_path.lineToPoint(CGPoint {
        x: center_x + x_size,
        y: center_y - x_size,
    });

    // Second line of X (top-right to bottom-left)
    x_path.moveToPoint(CGPoint {
        x: center_x + x_size,
        y: center_y + x_size,
    });
    x_path.lineToPoint(CGPoint {
        x: center_x - x_size,
        y: center_y - x_size,
    });

    x_path.stroke();
}

/// Creates an IOKit assertion to prevent the system from sleeping
fn prevent_sleep() -> Option<u32> {
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

    // Check for unlock combination: Cmd+Option+U
    if event_type == CGEventType::KeyDown {
        let cg_event = event.as_ref();

        let flags = CGEvent::flags(Some(cg_event));
        let keycode =
            CGEvent::integer_value_field(Some(cg_event), CGEventField::KeyboardEventKeycode);

        // Check for Cmd + Option + U key
        let cmd_pressed = flags.contains(CGEventFlags::MaskCommand);
        let option_pressed = flags.contains(CGEventFlags::MaskAlternate);

        if cmd_pressed && option_pressed && keycode == KEY_U {
            println!("\n  🔓 Unlock combination detected (Cmd+Option+U)!");

            // Stop the run loop to allow clean exit
            CFRunLoopStop(CFRunLoopGetCurrent());

            // Let this event through
            return event.as_ptr();
        }
    }

    // Block all keyboard and mouse events by returning NULL
    if event_type == CGEventType::KeyDown
        || event_type == CGEventType::KeyUp
        || event_type == CGEventType::FlagsChanged
        || event_type == CGEventType::LeftMouseDown
        || event_type == CGEventType::LeftMouseUp
        || event_type == CGEventType::RightMouseDown
        || event_type == CGEventType::RightMouseUp
        || event_type == CGEventType::MouseMoved
        || event_type == CGEventType::LeftMouseDragged
        || event_type == CGEventType::RightMouseDragged
        || event_type == CGEventType::ScrollWheel
        || event_type == CGEventType::OtherMouseDown
        || event_type == CGEventType::OtherMouseUp
        || event_type == CGEventType::OtherMouseDragged
    {
        // Return NULL to block the event
        return std::ptr::null_mut();
    }

    event.as_ptr()
}

/// Check if we have accessibility permissions
fn check_accessibility() -> bool {
    unsafe { AXIsProcessTrusted() }
}

/// Create and enable the event tap
fn setup_event_tap() -> bool {
    // Define event mask for all keyboard and mouse events
    let event_mask: CGEventMask = (1u64 << CGEventType::KeyDown.0)
        | (1u64 << CGEventType::KeyUp.0)
        | (1u64 << CGEventType::FlagsChanged.0)
        | (1u64 << CGEventType::LeftMouseDown.0)
        | (1u64 << CGEventType::LeftMouseUp.0)
        | (1u64 << CGEventType::RightMouseDown.0)
        | (1u64 << CGEventType::RightMouseUp.0)
        | (1u64 << CGEventType::MouseMoved.0)
        | (1u64 << CGEventType::LeftMouseDragged.0)
        | (1u64 << CGEventType::RightMouseDragged.0)
        | (1u64 << CGEventType::ScrollWheel.0)
        | (1u64 << CGEventType::OtherMouseDown.0)
        | (1u64 << CGEventType::OtherMouseUp.0)
        | (1u64 << CGEventType::OtherMouseDragged.0);

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

        // Intentionally leak the CFRetained<CFMachPort> to keep the event tap alive
        // for the entire program lifetime. The raw pointer in EVENT_TAP remains valid,
        // and cleanup happens automatically on process exit.
        std::mem::forget(tap);

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

    // Get main thread marker - required for AppKit operations
    let mtm = MainThreadMarker::new().expect("Must run on main thread");

    // Initialize the application
    let app = NSApplication::sharedApplication(mtm);
    app.setActivationPolicy(NSApplicationActivationPolicy::Accessory);

    // Get the main screen dimensions
    let screen = NSScreen::mainScreen(mtm);
    let screen = match screen {
        Some(s) => s,
        None => {
            eprintln!("  ✗ Failed to get main screen");
            process::exit(1);
        }
    };
    let screen_frame = screen.frame();

    // Create a fullscreen, borderless window
    let window = unsafe {
        let window = NSWindow::alloc(mtm);
        NSWindow::initWithContentRect_styleMask_backing_defer(
            window,
            screen_frame,
            NSWindowStyleMask::Borderless,
            NSBackingStoreType::Buffered,
            false,
        )
    };

    // Configure window to be topmost
    window.setLevel(NS_SCREEN_SAVER_WINDOW_LEVEL);

    // Set window to appear on all spaces and stay visible
    window.setCollectionBehavior(
        NSWindowCollectionBehavior::CanJoinAllSpaces
            | NSWindowCollectionBehavior::Stationary
            | NSWindowCollectionBehavior::IgnoresCycle,
    );

    // Make window semi-transparent (30% opacity)
    window.setOpaque(false);
    window.setAlphaValue(0.3);

    // Set a dark background color
    let bg_color = NSColor::colorWithRed_green_blue_alpha(0.1, 0.1, 0.15, 1.0);
    window.setBackgroundColor(Some(&bg_color));

    // Keep window visible
    window.setHidesOnDeactivate(false);

    // Accept mouse events (needed for blocking)
    window.setIgnoresMouseEvents(false);

    // Set title
    window.setTitle(ns_string!("Cat Shield"));

    // Required when creating NSWindow outside a window controller
    unsafe {
        window.setReleasedWhenClosed(false);
    }

    // Show the window
    window.makeKeyAndOrderFront(None);

    println!("  ✓ Overlay window active");

    // Create and add the close button in top-right corner
    let close_button_frame = CGRect {
        origin: CGPoint {
            x: screen_frame.size.width - CLOSE_BUTTON_SIZE - CLOSE_BUTTON_MARGIN,
            y: screen_frame.size.height - CLOSE_BUTTON_SIZE - CLOSE_BUTTON_MARGIN,
        },
        size: CGSize {
            width: CLOSE_BUTTON_SIZE,
            height: CLOSE_BUTTON_SIZE,
        },
    };

    let close_button = CloseButtonView::new(mtm, close_button_frame);

    // Store view reference for timer callback.
    // Safety: The view remains valid because contentView retains it and
    // CFRunLoopRun() blocks until we're ready to exit. The timer is stopped
    // before cleanup begins.
    CLOSE_BUTTON_VIEW.store(
        Retained::as_ptr(&close_button) as *mut c_void,
        Ordering::SeqCst,
    );

    // Add close button to the window's content view
    if let Some(content_view) = window.contentView() {
        content_view.addSubview(&close_button);
    }

    // Start the animation timer
    start_close_button_timer();

    println!("  ✓ Close button active (hold 3s to exit)");

    // Prevent sleep
    let assertion_id = prevent_sleep();

    // Set up event tap if we have permissions
    let has_accessibility = check_accessibility();
    if has_accessibility {
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
    println!("  Exit: Hold X button (top-right) for 3 seconds");
    if has_accessibility {
        println!("        Or press Cmd+Option+U");
    }
    println!();

    // Run the event loop using CoreFoundation run loop
    unsafe {
        CFRunLoopRun();
    }

    // Cleanup
    stop_close_button_timer();

    if let Some(id) = assertion_id {
        allow_sleep(id);
    }

    println!();
    println!("  👋 Cat Shield deactivated. Goodbye!");
    println!();
}
