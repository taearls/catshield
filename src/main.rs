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

// This application is currently macOS-only
#[cfg(not(target_os = "macos"))]
fn main() {
    eprintln!("Cat Shield is currently only supported on macOS.");
    eprintln!("Windows and Linux support is planned for future releases.");
    std::process::exit(1);
}

#[cfg(target_os = "macos")]
use cat_shield::config::{has_immediate_start_args, Args, Config};
#[cfg(target_os = "macos")]
use cat_shield::input::{set_exit_key, ExitKey, DEFAULT_EXIT_KEY};
#[cfg(target_os = "macos")]
use cat_shield::lock::{acquire_instance_lock, release_instance_lock, LockResult};
#[cfg(target_os = "macos")]
use cat_shield::platform::{allow_sleep, prevent_sleep, setup_event_tap};
#[cfg(target_os = "macos")]
use cat_shield::shield_core::{
    create_shield_window, ensure_accessibility, print_activation_banner, print_shield_active,
    setup_close_button, setup_timer_display,
};
#[cfg(target_os = "macos")]
use cat_shield::timer::{format_duration, get_remaining_seconds};
#[cfg(target_os = "macos")]
use cat_shield::ui::menu_bar::setup_menu_bar;
#[cfg(target_os = "macos")]
use cat_shield::ui::shield::{stop_close_button_timer, timer_callback};
#[cfg(target_os = "macos")]
use cat_shield::ui::state::{animation, shield};

#[cfg(target_os = "macos")]
use clap::Parser;
#[cfg(target_os = "macos")]
use objc2::rc::Retained;
#[cfg(target_os = "macos")]
use objc2_app_kit::{NSApplication, NSApplicationActivationPolicy, NSScreen};
#[cfg(target_os = "macos")]
use objc2_core_foundation::{kCFRunLoopCommonModes, CFString};
#[cfg(target_os = "macos")]
use objc2_foundation::MainThreadMarker;
#[cfg(target_os = "macos")]
use std::ffi::c_void;
#[cfg(target_os = "macos")]
use std::process;
#[cfg(target_os = "macos")]
use std::sync::atomic::Ordering;

#[cfg(target_os = "macos")]
use cat_shield::platform::{
    CFAbsoluteTimeGetCurrent, CFRunLoopAddTimer, CFRunLoopGetCurrent, CFRunLoopTimerCreate,
};

/// Start the animation timer for immediate mode
#[cfg(target_os = "macos")]
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

#[cfg(target_os = "macos")]
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

    // Always set up menu bar mode - shield deactivation returns here
    shield::MODE_MENU_BAR.store(true, Ordering::Release);

    println!();
    println!("  🐱 CAT SHIELD 🛡️");
    println!("  ════════════════════════════════════════");

    // Set up menu bar icon (always, so we can return here after shield deactivation)
    let _status_item = setup_menu_bar(mtm);

    // Check if we should start shield immediately (CLI args provided)
    if has_immediate_start_args(&args) {
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

        // Store window reference for cleanup (needed for deactivate_shield)
        shield::WINDOW.store(Retained::as_ptr(&window) as *mut c_void, Ordering::Release);

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

        // Set up auto-exit timer if specified (uses shared helper)
        if let Some(duration_secs) = args.timer {
            if !args.hide_timer {
                setup_timer_display(mtm, &window, screen_frame, duration_secs);
            } else {
                // Timer without display - just initialize the timer
                cat_shield::timer::init_auto_exit_timer(duration_secs);
                println!(
                    "  ✓ Auto-exit timer set: {}",
                    format_duration(duration_secs)
                );
            }
        }

        // Prevent sleep
        if let Some(assertion_id) = prevent_sleep() {
            shield::SLEEP_ASSERTION_ID.store(assertion_id as u64, Ordering::Release);
            shield::HAS_SLEEP_ASSERTION.store(true, Ordering::Release);
        }

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

        // Mark shield as active
        shield::IS_ACTIVE.store(true, Ordering::Release);

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

        // Transfer ownership to prevent deallocation while shield is active.
        // The raw pointers stored in global state are reclaimed in deactivate_shield()
        // using Retained::from_raw, which properly releases the objects.
        std::mem::forget(window);
        std::mem::forget(close_button);
        std::mem::forget(close_button_label);
    } else {
        // Menu bar only mode (no CLI args)
        println!("  Menu bar mode active");
        println!();
        println!("  Click the 🐱 icon in your menu bar to access Cat Shield.");
        println!("  Use 'Start Protection' to activate the shield.");
        println!("  Or run with --timer or --exit-key to start immediately.");
        println!();
    }

    // Finish launching the application (required for menu bar apps)
    app.finishLaunching();

    // Run the NSApplication event loop
    // The status item keeps the app alive in the menu bar
    app.run();

    // Cleanup (when app.terminate() is called)
    stop_close_button_timer();

    if shield::HAS_SLEEP_ASSERTION.load(Ordering::Acquire) {
        let id = shield::SLEEP_ASSERTION_ID.load(Ordering::Acquire) as u32;
        allow_sleep(id);
    }

    // Release single-instance lock before exiting
    release_instance_lock();

    println!();
    println!("  👋 Cat Shield closed. Goodbye!");
    println!();
}
