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
use cat_shield::platform::{allow_sleep, prevent_sleep, setup_event_tap};
use cat_shield::shield_core::{
    create_shield_window, ensure_accessibility, print_activation_banner, print_shield_active,
    setup_close_button,
};
use cat_shield::timer::{format_duration, get_remaining_seconds, init_auto_exit_timer};
use cat_shield::ui::menu_bar::setup_menu_bar;
use cat_shield::ui::shield::{stop_close_button_timer, timer_callback};
use cat_shield::ui::state::{animation, shield, timer_display};
use cat_shield::ui::views::TimerDisplayView;

use clap::Parser;
use objc2::rc::Retained;
use objc2_app_kit::{NSApplication, NSApplicationActivationPolicy, NSScreen};
use objc2_core_foundation::{kCFRunLoopCommonModes, CFString, CGPoint, CGRect, CGSize};
use objc2_foundation::MainThreadMarker;
use std::ffi::c_void;
use std::process;
use std::sync::atomic::Ordering;

use cat_shield::platform::{
    CFAbsoluteTimeGetCurrent, CFRunLoopAddTimer, CFRunLoopGetCurrent, CFRunLoopTimerCreate,
};

/// Start the animation timer for immediate mode
fn start_close_button_timer() {
    // SAFETY: CFRunLoopTimerCreate and related calls are safe because:
    // - All arguments are valid (null allocator uses default, valid time values)
    // - timer_callback is a valid extern "C" function
    // - We check for null return before using the timer
    // - CFRunLoopAddTimer is called on the current thread's run loop
    // - The timer pointer is stored in shield::TIMER_REF for later cleanup
    unsafe {
        let timer = CFRunLoopTimerCreate(
            std::ptr::null(),
            CFAbsoluteTimeGetCurrent() + animation::INTERVAL_SECS,
            animation::INTERVAL_SECS,
            0,
            0,
            timer_callback,
            std::ptr::null(),
        );

        if !timer.is_null() {
            let run_loop = CFRunLoopGetCurrent();
            let mode = kCFRunLoopCommonModes.expect("kCFRunLoopCommonModes should exist");
            CFRunLoopAddTimer(run_loop, timer, (mode as *const CFString) as *const c_void);
            shield::TIMER_REF.store(timer, Ordering::Release);
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
        shield::MODE_MENU_BAR.store(true, Ordering::Release);

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
    // Ensure accessibility permissions (uses shared logic from shield_core)
    ensure_accessibility(&exit_key);

    // Print activation banner (shared)
    print_activation_banner();

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

    // Create the shield window using shared logic
    let window = create_shield_window(mtm, screen_frame);

    // Show the window
    window.makeKeyAndOrderFront(None);

    println!("  ✓ Overlay window active");

    // Create close button and label using shared logic
    let (close_button, close_button_label) = setup_close_button(mtm, screen_frame);

    // Store view references for timer callback.
    // Safety: The view remains valid because contentView retains it and
    // app.run() blocks until we're ready to exit. The timer is stopped
    // before cleanup begins.
    shield::CLOSE_BUTTON.store(
        Retained::as_ptr(&close_button) as *mut c_void,
        Ordering::Release,
    );
    shield::CLOSE_BUTTON_LABEL.store(
        Retained::as_ptr(&close_button_label) as *mut c_void,
        Ordering::Release,
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
                    x: timer_display::MARGIN,
                    y: screen_frame.size.height - timer_display::HEIGHT - timer_display::MARGIN,
                },
                size: CGSize {
                    width: timer_display::WIDTH,
                    height: timer_display::HEIGHT,
                },
            };

            let timer_display = TimerDisplayView::new(mtm, timer_display_frame);

            // Store view reference for timer callback
            shield::TIMER_VIEW.store(
                Retained::as_ptr(&timer_display) as *mut c_void,
                Ordering::Release,
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

    // Print shield active status (shared)
    let timer_info = if args.timer.is_some() {
        Some(format!(
            "{} remaining",
            format_duration(get_remaining_seconds())
        ))
    } else {
        None
    };
    print_shield_active(&exit_key, timer_info.as_deref());

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
