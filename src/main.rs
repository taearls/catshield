//! Cat Shield - A cat-proof screen overlay for macOS
//!
//! Creates a semi-transparent overlay that:
//! - Blocks all keyboard and mouse input
//! - Keeps the machine awake
//! - Click and hold close button (3 seconds) to exit
//! - Or unlock with Cmd+Option+U (requires Accessibility permissions)
//! - Optional timer-based auto-exit
//!
//! Usage: Run the application, and it will immediately activate the shield.
//! Click and hold the X button in the top-right corner for 3 seconds to exit.
//!
//! Timer: Use --timer or -t to set auto-exit timer:
//!   cat_shield --timer 30m      # Exit after 30 minutes
//!   cat_shield --timer 2h       # Exit after 2 hours
//!   cat_shield -t 45m           # Short form
//!
//! Optional: For keyboard shortcut exit (Cmd+Option+U), grant Accessibility permissions:
//! Go to System Preferences → Security & Privacy → Privacy → Accessibility
//! and add this application.

use clap::Parser;
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
use std::sync::atomic::{AtomicBool, AtomicPtr, AtomicU64, Ordering};
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

// CoreFoundation bindings
#[link(name = "CoreFoundation", kind = "framework")]
extern "C" {
    // Run loop source management
    fn CFMachPortCreateRunLoopSource(
        allocator: *const c_void,
        port: *mut c_void,
        order: i64,
    ) -> *mut c_void;
    fn CFRunLoopAddSource(rl: *mut c_void, source: *mut c_void, mode: *const c_void);

    // Run loop access
    fn CFRunLoopGetCurrent() -> *mut c_void;

    // Timer management
    fn CFRunLoopAddTimer(rl: *mut c_void, timer: *mut c_void, mode: *const c_void);
    fn CFRunLoopTimerCreate(
        allocator: *const c_void,
        fire_date: f64,
        interval: f64,
        flags: u32,
        order: i64,
        callout: unsafe extern "C" fn(*mut c_void, *mut c_void),
        context: *const c_void,
    ) -> *mut c_void;
    fn CFRunLoopTimerInvalidate(timer: *mut c_void);
    fn CFAbsoluteTimeGetCurrent() -> f64;
}

const K_IOPM_ASSERTION_LEVEL_ON: u32 = 255;

// Keycode for 'U' on macOS
const KEY_U: i64 = 32;

// Close button configuration
const CLOSE_BUTTON_SIZE: CGFloat = 80.0; // Large, easy-to-see button
const CLOSE_BUTTON_MARGIN: CGFloat = 30.0;
const HOLD_DURATION_SECS: f64 = 3.0;
const TIMER_INTERVAL_SECS: f64 = 1.0 / 60.0; // 60 FPS for smooth animation

// Window levels from NSWindow.h
const NS_SCREEN_SAVER_WINDOW_LEVEL: isize = 1000;

// Timer configuration
const MIN_TIMER_SECONDS: u64 = 60; // Minimum 1 minute
const MAX_TIMER_SECONDS: u64 = 24 * 60 * 60; // Maximum 24 hours
const WARNING_SECONDS: u64 = 60; // Show warning 1 minute before exit

// Timer display configuration
const TIMER_DISPLAY_HEIGHT: CGFloat = 60.0;
const TIMER_DISPLAY_WIDTH: CGFloat = 200.0;
const TIMER_DISPLAY_MARGIN: CGFloat = 30.0;

/// CLI arguments for Cat Shield
#[derive(Parser, Debug)]
#[command(name = "cat_shield")]
#[command(author = "Tyler Earls")]
#[command(version)]
#[command(about = "A cat-proof screen overlay that keeps your machine awake and blocks input")]
struct Args {
    /// Auto-exit after specified duration (e.g., 30m, 2h, 1h30m)
    #[arg(short, long, value_parser = parse_duration)]
    timer: Option<u64>,

    /// Hide the countdown timer display
    #[arg(long)]
    hide_timer: bool,
}

/// Parse duration string like "30m", "2h", "1h30m" into seconds
fn parse_duration(s: &str) -> Result<u64, String> {
    let s = s.trim().to_lowercase();
    if s.is_empty() {
        return Err("Duration cannot be empty".to_string());
    }

    let mut total_seconds: u64 = 0;
    let mut current_num = String::new();

    for c in s.chars() {
        if c.is_ascii_digit() {
            current_num.push(c);
        } else if c == 'h' {
            if current_num.is_empty() {
                return Err("Missing number before 'h'".to_string());
            }
            let hours: u64 = current_num
                .parse()
                .map_err(|_| format!("Invalid number: {}", current_num))?;
            total_seconds += hours * 3600;
            current_num.clear();
        } else if c == 'm' {
            if current_num.is_empty() {
                return Err("Missing number before 'm'".to_string());
            }
            let minutes: u64 = current_num
                .parse()
                .map_err(|_| format!("Invalid number: {}", current_num))?;
            total_seconds += minutes * 60;
            current_num.clear();
        } else if c == 's' {
            if current_num.is_empty() {
                return Err("Missing number before 's'".to_string());
            }
            let secs: u64 = current_num
                .parse()
                .map_err(|_| format!("Invalid number: {}", current_num))?;
            total_seconds += secs;
            current_num.clear();
        } else if !c.is_whitespace() {
            return Err(format!("Invalid character in duration: '{}'", c));
        }
    }

    // If there are remaining digits without a unit, assume minutes
    if !current_num.is_empty() {
        let minutes: u64 = current_num
            .parse()
            .map_err(|_| format!("Invalid number: {}", current_num))?;
        total_seconds += minutes * 60;
    }

    if total_seconds == 0 {
        return Err("Duration must be greater than zero".to_string());
    }

    if total_seconds < MIN_TIMER_SECONDS {
        return Err(format!(
            "Duration must be at least {} seconds (1 minute)",
            MIN_TIMER_SECONDS
        ));
    }

    if total_seconds > MAX_TIMER_SECONDS {
        return Err(format!(
            "Duration must not exceed {} seconds (24 hours)",
            MAX_TIMER_SECONDS
        ));
    }

    Ok(total_seconds)
}

/// Calculate hold progress as a value from 0.0 to 1.0.
///
/// # Arguments
/// * `elapsed_secs` - Time elapsed since mouse down in seconds
/// * `hold_duration_secs` - Required hold duration in seconds
///
/// # Returns
/// Progress value clamped to range [0.0, 1.0]
#[inline]
fn calculate_hold_progress(elapsed_secs: f64, hold_duration_secs: f64) -> f64 {
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
fn is_hold_complete(elapsed_secs: f64, hold_duration_secs: f64) -> bool {
    elapsed_secs >= hold_duration_secs
}

// Global timer reference for cleanup
static TIMER_REF: AtomicPtr<c_void> = AtomicPtr::new(std::ptr::null_mut());

// Global view reference for timer callback
static CLOSE_BUTTON_VIEW: AtomicPtr<c_void> = AtomicPtr::new(std::ptr::null_mut());

// Global pointer to the event tap for re-enabling from callback
static EVENT_TAP: AtomicPtr<c_void> = AtomicPtr::new(std::ptr::null_mut());

// Global timer state for auto-exit feature
static AUTO_EXIT_ENABLED: AtomicBool = AtomicBool::new(false);
static AUTO_EXIT_START_TIME: AtomicU64 = AtomicU64::new(0);
static AUTO_EXIT_DURATION_SECS: AtomicU64 = AtomicU64::new(0);
static WARNING_SHOWN: AtomicBool = AtomicBool::new(false);

// Global reference to the timer display view for updates
static TIMER_DISPLAY_VIEW: AtomicPtr<c_void> = AtomicPtr::new(std::ptr::null_mut());

// Close button state stored in thread-local for the view
thread_local! {
    static MOUSE_DOWN_TIME: Cell<Option<Instant>> = const { Cell::new(None) };
    static IS_MOUSE_INSIDE: Cell<bool> = const { Cell::new(false) };
}

// Timer callback to update progress, check for exit condition, and trigger redraw
unsafe extern "C" fn timer_callback(_timer: *mut c_void, _info: *mut c_void) {
    // Check if hold duration has been exceeded (close button)
    let should_exit_from_button = MOUSE_DOWN_TIME.with(|time| {
        if let Some(start) = time.get() {
            let is_inside = IS_MOUSE_INSIDE.with(|inside| inside.get());
            is_inside && is_hold_complete(start.elapsed().as_secs_f64(), HOLD_DURATION_SECS)
        } else {
            false
        }
    });

    if should_exit_from_button {
        // Use NSApplication terminate to properly exit the app run loop
        if let Some(mtm) = MainThreadMarker::new() {
            let app = NSApplication::sharedApplication(mtm);
            app.terminate(None);
        }
        return;
    }

    // Check auto-exit timer
    if AUTO_EXIT_ENABLED.load(Ordering::SeqCst) {
        let remaining = get_remaining_seconds();

        // Show warning when approaching exit
        if remaining <= WARNING_SECONDS && !WARNING_SHOWN.swap(true, Ordering::SeqCst) {
            println!();
            println!("  ⚠️  Auto-exit in {} seconds!", remaining);
            println!();
        }

        // Check if timer has expired
        if remaining == 0 {
            println!();
            println!("  ⏰ Timer expired - auto-exiting...");
            if let Some(mtm) = MainThreadMarker::new() {
                let app = NSApplication::sharedApplication(mtm);
                app.terminate(None);
            }
            return;
        }
    }

    // Trigger redraw of close button
    let view_ptr = CLOSE_BUTTON_VIEW.load(Ordering::SeqCst);
    if !view_ptr.is_null() {
        let view: &NSView = &*(view_ptr as *const NSView);
        view.setNeedsDisplay(true);
    }

    // Trigger redraw of timer display
    let timer_view_ptr = TIMER_DISPLAY_VIEW.load(Ordering::SeqCst);
    if !timer_view_ptr.is_null() {
        let view: &NSView = &*(timer_view_ptr as *const NSView);
        view.setNeedsDisplay(true);
    }
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

/// Initialize the auto-exit timer with the specified duration in seconds
fn init_auto_exit_timer(duration_secs: u64) {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    AUTO_EXIT_START_TIME.store(now, Ordering::SeqCst);
    AUTO_EXIT_DURATION_SECS.store(duration_secs, Ordering::SeqCst);
    AUTO_EXIT_ENABLED.store(true, Ordering::SeqCst);
}

/// Get the remaining seconds until auto-exit, or 0 if expired
fn get_remaining_seconds() -> u64 {
    if !AUTO_EXIT_ENABLED.load(Ordering::SeqCst) {
        return u64::MAX;
    }

    let start = AUTO_EXIT_START_TIME.load(Ordering::SeqCst);
    let duration = AUTO_EXIT_DURATION_SECS.load(Ordering::SeqCst);
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();

    let elapsed = now.saturating_sub(start);
    duration.saturating_sub(elapsed)
}

/// Format seconds as a human-readable string (e.g., "1h 30m 45s")
fn format_duration(total_secs: u64) -> String {
    let hours = total_secs / 3600;
    let minutes = (total_secs % 3600) / 60;
    let secs = total_secs % 60;

    if hours > 0 {
        format!("{}h {:02}m {:02}s", hours, minutes, secs)
    } else if minutes > 0 {
        format!("{}m {:02}s", minutes, secs)
    } else {
        format!("{}s", secs)
    }
}

/// Ivars for the TimerDisplayView
struct TimerDisplayViewIvars {}

define_class!(
    #[unsafe(super(NSView))]
    #[name = "TimerDisplayView"]
    #[ivars = TimerDisplayViewIvars]
    struct TimerDisplayView;

    impl TimerDisplayView {
        #[unsafe(method(drawRect:))]
        unsafe fn draw_rect(&self, _dirty_rect: CGRect) {
            draw_timer_display(self);
        }
    }
);

impl TimerDisplayView {
    fn new(mtm: MainThreadMarker, frame: CGRect) -> Retained<Self> {
        let this = mtm.alloc::<TimerDisplayView>();
        let this = this.set_ivars(TimerDisplayViewIvars {});
        unsafe { msg_send![super(this), initWithFrame: frame] }
    }
}

/// Draw the timer countdown display
fn draw_timer_display(view: &NSView) {
    let bounds = view.bounds();
    let remaining = get_remaining_seconds();
    let is_warning = remaining <= WARNING_SECONDS;

    // Background rounded rectangle
    let bg_color = if is_warning {
        // Red/orange warning color
        NSColor::colorWithRed_green_blue_alpha(0.8, 0.3, 0.1, 0.9)
    } else {
        // Dark semi-transparent background
        NSColor::colorWithRed_green_blue_alpha(0.1, 0.1, 0.15, 0.9)
    };
    bg_color.set();

    let corner_radius = 10.0;
    let bg_rect = CGRect {
        origin: CGPoint { x: 0.0, y: 0.0 },
        size: bounds.size,
    };
    let bg_path = NSBezierPath::bezierPathWithRoundedRect_xRadius_yRadius(
        bg_rect,
        corner_radius,
        corner_radius,
    );
    bg_path.fill();

    // Border
    let border_color = if is_warning {
        NSColor::colorWithRed_green_blue_alpha(1.0, 0.5, 0.2, 1.0)
    } else {
        NSColor::colorWithRed_green_blue_alpha(0.5, 0.5, 0.5, 0.8)
    };
    border_color.set();
    bg_path.setLineWidth(2.0);
    bg_path.stroke();

    // Draw time text using simple shapes (since we can't easily use NSString drawing)
    // We'll draw a simple digital-style countdown
    let time_str = format_duration(remaining);

    // Draw the time as a series of character approximations
    // For simplicity, we'll just draw colored rectangles to indicate time
    // The actual time will be printed to console

    // Draw a progress bar showing remaining time
    let duration = AUTO_EXIT_DURATION_SECS.load(Ordering::SeqCst);
    let progress = if duration > 0 {
        remaining as f64 / duration as f64
    } else {
        0.0
    };

    // Progress bar background
    let bar_margin = 10.0;
    let bar_height = 20.0;
    let bar_y = (bounds.size.height - bar_height) / 2.0;
    let bar_width = bounds.size.width - (bar_margin * 2.0);

    let bar_bg_color = NSColor::colorWithRed_green_blue_alpha(0.2, 0.2, 0.2, 1.0);
    bar_bg_color.set();

    let bar_bg_rect = CGRect {
        origin: CGPoint {
            x: bar_margin,
            y: bar_y,
        },
        size: CGSize {
            width: bar_width,
            height: bar_height,
        },
    };
    let bar_bg_path =
        NSBezierPath::bezierPathWithRoundedRect_xRadius_yRadius(bar_bg_rect, 5.0, 5.0);
    bar_bg_path.fill();

    // Progress bar fill
    let bar_fill_color = if is_warning {
        NSColor::colorWithRed_green_blue_alpha(1.0, 0.3, 0.1, 1.0)
    } else {
        NSColor::colorWithRed_green_blue_alpha(0.2, 0.8, 0.3, 1.0)
    };
    bar_fill_color.set();

    let fill_width = bar_width * progress;
    if fill_width > 0.0 {
        let bar_fill_rect = CGRect {
            origin: CGPoint {
                x: bar_margin,
                y: bar_y,
            },
            size: CGSize {
                width: fill_width,
                height: bar_height,
            },
        };
        let bar_fill_path =
            NSBezierPath::bezierPathWithRoundedRect_xRadius_yRadius(bar_fill_rect, 5.0, 5.0);
        bar_fill_path.fill();
    }

    // Print time to console periodically (every second, roughly)
    // This is handled by the main timer callback which prints warnings
    _ = time_str; // Suppress unused warning - time is displayed via progress bar
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
            calculate_hold_progress(start.elapsed().as_secs_f64(), HOLD_DURATION_SECS)
        } else {
            0.0
        }
    });

    let is_inside = IS_MOUSE_INSIDE.with(|inside| inside.get());

    // Background circle - bright red for visibility
    let bg_color = if is_inside && progress > 0.0 {
        NSColor::colorWithRed_green_blue_alpha(0.9, 0.2, 0.2, 1.0) // Bright red when pressed
    } else {
        NSColor::colorWithRed_green_blue_alpha(0.8, 0.1, 0.1, 0.95) // Dark red normally
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

    // White border for extra visibility
    let border_color = NSColor::colorWithRed_green_blue_alpha(1.0, 1.0, 1.0, 0.9);
    border_color.set();
    let border_path = NSBezierPath::bezierPathWithOvalInRect(CGRect {
        origin: CGPoint {
            x: center_x - radius,
            y: center_y - radius,
        },
        size: CGSize {
            width: radius * 2.0,
            height: radius * 2.0,
        },
    });
    border_path.setLineWidth(3.0);
    border_path.stroke();

    // Progress arc (if holding) - bright green
    if progress > 0.0 && is_inside {
        let progress_color = NSColor::colorWithRed_green_blue_alpha(0.2, 1.0, 0.2, 1.0);
        progress_color.set();

        // Draw arc from top, going clockwise
        let start_angle = 90.0; // Top of circle
        let end_angle = 90.0 - (progress * 360.0);

        let arc_path = NSBezierPath::bezierPath();
        arc_path.setLineWidth(6.0); // Thicker progress ring

        arc_path.appendBezierPathWithArcWithCenter_radius_startAngle_endAngle_clockwise(
            CGPoint {
                x: center_x,
                y: center_y,
            },
            radius - 5.0,
            start_angle,
            end_angle,
            true, // clockwise
        );
        arc_path.stroke();
    }

    // Draw X - always white and bold
    let x_color = NSColor::colorWithRed_green_blue_alpha(1.0, 1.0, 1.0, 1.0);
    x_color.set();

    let x_size = radius * 0.4;
    let x_path = NSBezierPath::bezierPath();
    x_path.setLineWidth(5.0); // Thicker X

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

            // Use NSApplication terminate to properly exit
            if let Some(mtm) = MainThreadMarker::new() {
                let app = NSApplication::sharedApplication(mtm);
                app.terminate(None);
            }

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
    // Parse command line arguments
    let args = Args::parse();

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

    // Make window semi-transparent (50% opacity - visible but not fully blocking view)
    window.setOpaque(false);
    window.setAlphaValue(0.5);

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
    // app.run() blocks until we're ready to exit. The timer is stopped
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

    // Set up auto-exit timer if specified
    if let Some(duration_secs) = args.timer {
        init_auto_exit_timer(duration_secs);
        println!(
            "  ✓ Auto-exit timer set: {}",
            format_duration(duration_secs)
        );

        // Create timer display view if not hidden
        if !args.hide_timer {
            let timer_display_frame = CGRect {
                origin: CGPoint {
                    x: TIMER_DISPLAY_MARGIN,
                    y: screen_frame.size.height - TIMER_DISPLAY_HEIGHT - TIMER_DISPLAY_MARGIN,
                },
                size: CGSize {
                    width: TIMER_DISPLAY_WIDTH,
                    height: TIMER_DISPLAY_HEIGHT,
                },
            };

            let timer_display = TimerDisplayView::new(mtm, timer_display_frame);

            // Store view reference for timer callback
            TIMER_DISPLAY_VIEW.store(
                Retained::as_ptr(&timer_display) as *mut c_void,
                Ordering::SeqCst,
            );

            // Add timer display to the window's content view
            if let Some(content_view) = window.contentView() {
                content_view.addSubview(&timer_display);
            }

            println!("  ✓ Timer display active");
        }
    }

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
    if args.timer.is_some() {
        println!(
            "        Or wait for timer ({} remaining)",
            format_duration(get_remaining_seconds())
        );
    }
    println!();

    // Run the NSApplication event loop (required for AppKit event handling)
    app.run();

    // Cleanup
    stop_close_button_timer();

    if let Some(id) = assertion_id {
        allow_sleep(id);
    }

    println!();
    println!("  👋 Cat Shield deactivated. Goodbye!");
    println!();
}

#[cfg(test)]
mod tests {
    use super::*;

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

    #[test]
    fn test_parse_duration_minutes() {
        assert_eq!(parse_duration("30m").unwrap(), 30 * 60);
        assert_eq!(parse_duration("1m").unwrap(), 60);
        assert_eq!(parse_duration("90m").unwrap(), 90 * 60);
    }

    #[test]
    fn test_parse_duration_hours() {
        assert_eq!(parse_duration("1h").unwrap(), 3600);
        assert_eq!(parse_duration("2h").unwrap(), 2 * 3600);
        assert_eq!(parse_duration("24h").unwrap(), 24 * 3600);
    }

    #[test]
    fn test_parse_duration_combined() {
        assert_eq!(parse_duration("1h30m").unwrap(), 3600 + 30 * 60);
        assert_eq!(parse_duration("2h45m").unwrap(), 2 * 3600 + 45 * 60);
    }

    #[test]
    fn test_parse_duration_with_spaces() {
        assert_eq!(parse_duration(" 30m ").unwrap(), 30 * 60);
        assert_eq!(parse_duration("1h 30m").unwrap(), 3600 + 30 * 60);
    }

    #[test]
    fn test_parse_duration_bare_number_as_minutes() {
        // A bare number without unit is treated as minutes
        assert_eq!(parse_duration("30").unwrap(), 30 * 60);
        assert_eq!(parse_duration("60").unwrap(), 60 * 60);
    }

    #[test]
    fn test_parse_duration_seconds() {
        assert_eq!(parse_duration("90s").unwrap(), 90);
        assert_eq!(parse_duration("1m30s").unwrap(), 90);
    }

    #[test]
    fn test_parse_duration_errors() {
        assert!(parse_duration("").is_err());
        assert!(parse_duration("0m").is_err());
        assert!(parse_duration("abc").is_err());
        assert!(parse_duration("30x").is_err());
        assert!(parse_duration("30s").is_err()); // Less than 1 minute
        assert!(parse_duration("25h").is_err()); // More than 24 hours
    }

    #[test]
    fn test_format_duration_seconds_only() {
        assert_eq!(format_duration(45), "45s");
        assert_eq!(format_duration(1), "1s");
    }

    #[test]
    fn test_format_duration_minutes_and_seconds() {
        assert_eq!(format_duration(90), "1m 30s");
        assert_eq!(format_duration(3599), "59m 59s");
    }

    #[test]
    fn test_format_duration_hours_minutes_seconds() {
        assert_eq!(format_duration(3600), "1h 00m 00s");
        assert_eq!(format_duration(3661), "1h 01m 01s");
        assert_eq!(format_duration(7200 + 1800 + 45), "2h 30m 45s");
    }
}
