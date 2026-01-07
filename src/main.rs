//! Cat Shield - A cat-proof screen overlay for macOS
//!
//! Creates a semi-transparent overlay that:
//! - Blocks all keyboard and mouse input
//! - Keeps the machine awake
//! - Click and hold close button (3 seconds) to exit
//! - Or unlock with configurable keyboard shortcut (default: Cmd+Option+U)
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
//! Exit Key: Use --exit-key or -e to set custom exit shortcut:
//!   cat_shield --exit-key "Cmd+Shift+Q"
//!   cat_shield --exit-key "Ctrl+Option+Escape"
//!   cat_shield -e "Cmd+Shift+X"
//!
//! Config File: Persistent settings can be stored in ~/.config/catshield/config.toml:
//!   exit_key = "Cmd+Option+U"
//!
//! Note: Keyboard shortcuts require Accessibility permissions.
//! Go to System Preferences → Security & Privacy → Privacy → Accessibility
//! and add this application.

use cat_shield::config::{has_immediate_start_args, Args, Config};
use cat_shield::input::{set_exit_key, ExitKey, DEFAULT_EXIT_KEY};
use cat_shield::lock::{acquire_instance_lock, release_instance_lock, LockResult};
use cat_shield::platform::CFRunLoopRunInMode;
use cat_shield::platform::{
    allow_sleep, check_accessibility, check_accessibility_with_prompt, open_accessibility_settings,
    prevent_sleep, setup_event_tap,
};
use cat_shield::timer::{format_duration, get_remaining_seconds, init_auto_exit_timer};
use cat_shield::ui::menu_bar::setup_menu_bar;
use cat_shield::ui::shield::stop_close_button_timer;
use cat_shield::ui::state::{
    CLOSE_BUTTON_LABEL_HEIGHT, CLOSE_BUTTON_LABEL_VIEW, CLOSE_BUTTON_LABEL_WIDTH,
    CLOSE_BUTTON_MARGIN, CLOSE_BUTTON_SIZE, CLOSE_BUTTON_VIEW, MENU_BAR_MODE,
    NS_SCREEN_SAVER_WINDOW_LEVEL, TIMER_DISPLAY_HEIGHT, TIMER_DISPLAY_MARGIN, TIMER_DISPLAY_VIEW,
    TIMER_DISPLAY_WIDTH, TIMER_REF,
};
use cat_shield::ui::views::{CloseButtonLabelView, CloseButtonView, TimerDisplayView};

use clap::Parser;
use objc2::rc::Retained;
use objc2::MainThreadOnly;
use objc2_app_kit::{
    NSApplication, NSApplicationActivationPolicy, NSBackingStoreType, NSColor, NSScreen, NSWindow,
    NSWindowCollectionBehavior, NSWindowStyleMask,
};
use objc2_core_foundation::{
    kCFRunLoopCommonModes, kCFRunLoopDefaultMode, CFString, CGPoint, CGRect, CGSize,
};
use objc2_foundation::{ns_string, MainThreadMarker};
use std::ffi::c_void;
use std::process;
use std::sync::atomic::Ordering;

use cat_shield::platform::{
    CFAbsoluteTimeGetCurrent, CFRunLoopAddTimer, CFRunLoopGetCurrent, CFRunLoopTimerCreate,
};
use cat_shield::timer::{AUTO_EXIT_ENABLED, WARNING_SECONDS, WARNING_SHOWN};
use cat_shield::ui::state::{
    is_hold_complete, HOLD_DURATION_SECS, IS_MOUSE_INSIDE, MOUSE_DOWN_TIME, TIMER_INTERVAL_SECS,
};

// Timer callback to update progress, check for exit condition, and trigger redraw
// This is duplicated from shield.rs for the immediate mode path
unsafe extern "C" fn timer_callback(_timer: *mut c_void, _info: *mut c_void) {
    use objc2_app_kit::NSView;

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

    // Trigger redraw of close button label (for countdown during hold)
    let label_view_ptr = CLOSE_BUTTON_LABEL_VIEW.load(Ordering::SeqCst);
    if !label_view_ptr.is_null() {
        let view: &NSView = &*(label_view_ptr as *const NSView);
        view.setNeedsDisplay(true);
    }

    // Trigger redraw of timer display
    let timer_view_ptr = TIMER_DISPLAY_VIEW.load(Ordering::SeqCst);
    if !timer_view_ptr.is_null() {
        let view: &NSView = &*(timer_view_ptr as *const NSView);
        view.setNeedsDisplay(true);
    }
}

/// Start the animation timer for immediate mode
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

fn main() {
    // Parse command line arguments
    let args = Args::parse();

    // Check for existing instance (single-instance enforcement)
    match acquire_instance_lock() {
        LockResult::Acquired => {
            // Successfully acquired lock, continue startup
        }
        LockResult::AlreadyRunning(pid) => {
            eprintln!();
            eprintln!("  🐱 Cat Shield is already running (PID: {})", pid);
            eprintln!("  Look for the 🐱 icon in your menu bar.");
            eprintln!();
            process::exit(0);
        }
        LockResult::Error(e) => {
            eprintln!(
                "  ⚠️  Warning: Could not check for existing instance: {}",
                e
            );
            // Continue anyway - lock check is best-effort
        }
    }

    // Load config file
    let config = Config::load();

    // Determine exit key: CLI arg > config file > default
    let exit_key = if let Some(ref key) = args.exit_key {
        key.clone()
    } else if let Some(ref key_str) = config.exit_key {
        match ExitKey::parse(key_str) {
            Ok(key) => key,
            Err(e) => {
                eprintln!("  ⚠️  Invalid exit_key in config file: {}", e);
                eprintln!("      Using default: {}", DEFAULT_EXIT_KEY);
                ExitKey::default()
            }
        }
    } else {
        ExitKey::default()
    };

    // Set the global exit key configuration
    set_exit_key(&exit_key);

    // Get main thread marker - required for AppKit operations
    let mtm = MainThreadMarker::new().expect("Must run on main thread");

    // Initialize the application
    let app = NSApplication::sharedApplication(mtm);
    app.setActivationPolicy(NSApplicationActivationPolicy::Accessory);

    // Check if we should enter menu bar mode (no CLI args that trigger immediate start)
    if !has_immediate_start_args(&args) {
        // Menu bar mode: show icon in menu bar and wait for user interaction
        MENU_BAR_MODE.store(true, Ordering::SeqCst);

        println!();
        println!("  🐱 CAT SHIELD 🛡️");
        println!("  ════════════════════════════════════════");
        println!("  Menu bar mode active");
        println!();

        // Set up menu bar icon
        let _status_item = setup_menu_bar(mtm);

        println!();
        println!("  Click the 🐱 icon in your menu bar to access Cat Shield.");
        println!("  Use 'Start Protection' to activate the shield.");
        println!("  Or run with --timer or --exit-key to start immediately.");
        println!();

        // Finish launching the application (required for menu bar apps)
        app.finishLaunching();

        // Run the NSApplication event loop
        // The status item keeps the app alive in the menu bar
        app.run();

        // Release single-instance lock before exiting
        release_instance_lock();

        println!();
        println!("  👋 Cat Shield closed. Goodbye!");
        println!();
        return;
    }

    // Immediate shield mode: CLI args provided, start protection now
    // Check accessibility permissions FIRST, before any UI
    let mut has_accessibility = check_accessibility();

    if !has_accessibility {
        println!();
        println!("  🐱 CAT SHIELD 🛡️");
        println!("  ════════════════════════════════════════");
        println!();
        eprintln!("  ⚠️  ACCESSIBILITY PERMISSION REQUIRED");
        eprintln!();
        eprintln!("  To block keyboard/mouse input and use the exit");
        eprintln!(
            "  shortcut ({}), this app needs Accessibility permissions.",
            exit_key.display_name
        );
        eprintln!();

        // Try to prompt user with native dialog
        println!("  Requesting accessibility permissions...");
        has_accessibility = check_accessibility_with_prompt();

        if has_accessibility {
            println!("  ✓ Permissions granted!");
            println!();
        } else {
            eprintln!();
            eprintln!("  Opening System Settings → Accessibility...");

            if open_accessibility_settings() {
                eprintln!("  ✓ System Settings opened");
            }
            eprintln!();
            eprintln!("  Please add Cat Shield to the Accessibility list.");
            eprintln!("  Waiting for permissions...");
            eprintln!();

            // Poll for permissions every 1 second using CFRunLoopRunInMode
            // This allows the run loop to process events while waiting,
            // which is necessary for macOS to update accessibility permission state
            const POLL_INTERVAL_SECS: f64 = 1.0;
            loop {
                unsafe {
                    let mode = kCFRunLoopDefaultMode.expect("kCFRunLoopDefaultMode should exist");
                    CFRunLoopRunInMode((mode as *const CFString).cast(), POLL_INTERVAL_SECS, false);
                }
                if check_accessibility() {
                    println!("  ✓ Permissions granted! Starting Cat Shield...");
                    println!();
                    break;
                }
            }
        }
    }

    println!();
    println!("  🐱 CAT SHIELD 🛡️");
    println!("  ════════════════════════════════════════");
    println!("  Protecting your work from curious cats!");
    println!();

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

    // Window must be non-opaque to allow transparent background, but keep alphaValue at 1.0
    // so that subviews (like label containers) can render fully opaque
    window.setOpaque(false);
    window.setAlphaValue(1.0);

    // Set a semi-transparent dark background color (the overlay effect)
    let bg_color = NSColor::colorWithRed_green_blue_alpha(0.1, 0.1, 0.15, 0.5);
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

    // Create the close button label view (positioned below the button)
    let label_x = screen_frame.size.width
        - CLOSE_BUTTON_MARGIN
        - CLOSE_BUTTON_SIZE / 2.0
        - CLOSE_BUTTON_LABEL_WIDTH / 2.0;
    let label_y = screen_frame.size.height
        - CLOSE_BUTTON_SIZE
        - CLOSE_BUTTON_MARGIN
        - CLOSE_BUTTON_LABEL_HEIGHT
        - 5.0; // 5px gap between button and label

    let close_button_label_frame = CGRect {
        origin: CGPoint {
            x: label_x,
            y: label_y,
        },
        size: CGSize {
            width: CLOSE_BUTTON_LABEL_WIDTH,
            height: CLOSE_BUTTON_LABEL_HEIGHT,
        },
    };
    let close_button_label = CloseButtonLabelView::new(mtm, close_button_label_frame);

    // Store label view reference for timer callback updates
    CLOSE_BUTTON_LABEL_VIEW.store(
        Retained::as_ptr(&close_button_label) as *mut c_void,
        Ordering::SeqCst,
    );

    // Add close button and label to the window's content view
    if let Some(content_view) = window.contentView() {
        content_view.addSubview(&close_button);
        content_view.addSubview(&close_button_label);
    }

    // Start the animation timer
    start_close_button_timer();

    println!("  ✓ Close button active (hold 3s to exit)");
    println!("  ✓ Exit key: {}", exit_key.display_name);

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

    // Set up event tap - this is the core security feature
    // Without input blocking, the shield is just a visual overlay
    if !setup_event_tap() {
        eprintln!("  ✗ Failed to create event tap");
        eprintln!();
        eprintln!("  ════════════════════════════════════════");
        eprintln!("  ⚠️  SHIELD ACTIVATION FAILED");
        eprintln!("  ════════════════════════════════════════");
        eprintln!();
        eprintln!("  Input blocking could not be enabled.");
        eprintln!("  The shield cannot protect without this feature.");
        eprintln!();
        eprintln!("  Please check:");
        eprintln!("  - Accessibility permissions are granted");
        eprintln!("  - No other apps are blocking event taps");
        eprintln!();
        std::process::exit(1);
    }
    println!("  ✓ Input blocking active");

    println!();
    println!("  ═══════════════════════════════════════");
    println!("  🛡️  CAT SHIELD IS NOW ACTIVE!");
    println!("  ═══════════════════════════════════════");
    println!();
    println!("  Exit: Hold X button (top-right) for 3 seconds");
    println!("        Or press {}", exit_key.display_name);
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

    // Release single-instance lock before exiting
    release_instance_lock();

    println!();
    println!("  👋 Cat Shield deactivated. Goodbye!");
    println!();
}
