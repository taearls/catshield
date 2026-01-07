//! Shield overlay activation and deactivation for Cat Shield

use crate::input::get_exit_key;
use crate::platform::{
    allow_sleep, check_accessibility, check_accessibility_with_prompt, disable_event_tap,
    open_accessibility_settings, prevent_sleep, setup_event_tap,
};
use crate::platform::{CFRunLoopGetCurrent, CFRunLoopRunInMode};
use crate::timer::{AUTO_EXIT_ENABLED, WARNING_SHOWN};
use crate::ui::state::{
    CLOSE_BUTTON_LABEL_HEIGHT, CLOSE_BUTTON_LABEL_VIEW, CLOSE_BUTTON_LABEL_WIDTH,
    CLOSE_BUTTON_MARGIN, CLOSE_BUTTON_SIZE, CLOSE_BUTTON_TEXT_LABEL, CLOSE_BUTTON_VIEW,
    HAS_SLEEP_ASSERTION, IS_MOUSE_INSIDE, MENU_BAR_MODE, MOUSE_DOWN_TIME,
    NS_SCREEN_SAVER_WINDOW_LEVEL, SHIELD_ACTIVE, SHIELD_WINDOW, SLEEP_ASSERTION_ID,
    START_MENU_ITEM, TIMER_DISPLAY_VIEW, TIMER_HEADER_LABEL, TIMER_REF, TIMER_TIME_LABEL,
    TIMER_WARNING_LABEL,
};
use crate::ui::views::{CloseButtonLabelView, CloseButtonView};
use objc2::rc::Retained;
use objc2::MainThreadOnly;
use objc2_app_kit::{
    NSBackingStoreType, NSColor, NSMenuItem, NSScreen, NSWindow, NSWindowCollectionBehavior,
    NSWindowStyleMask,
};
use objc2_core_foundation::{
    kCFRunLoopCommonModes, kCFRunLoopDefaultMode, CFString, CGPoint, CGRect, CGSize,
};
use objc2_foundation::{ns_string, MainThreadMarker};
use std::ffi::c_void;
use std::sync::atomic::Ordering;

use crate::platform::{
    CFAbsoluteTimeGetCurrent, CFRunLoopAddTimer, CFRunLoopTimerCreate, CFRunLoopTimerInvalidate,
};
use crate::timer::{get_remaining_seconds, WARNING_SECONDS};
use crate::ui::state::TIMER_INTERVAL_SECS;

/// Timer callback to update progress, check for exit condition, and trigger redraw.
///
/// This callback handles both menu bar mode (returns to menu bar) and immediate mode
/// (terminates the app) based on the MENU_BAR_MODE flag.
///
/// # Safety
/// This function is called from the CFRunLoop timer and must be `unsafe extern "C"`.
pub unsafe extern "C" fn timer_callback(_timer: *mut c_void, _info: *mut c_void) {
    use crate::ui::state::{is_hold_complete, HOLD_DURATION_SECS};
    use objc2_app_kit::{NSApplication, NSView};

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
        // In menu bar mode, deactivate shield and return to menu bar
        // In immediate mode, terminate the app
        if MENU_BAR_MODE.load(Ordering::SeqCst) {
            deactivate_shield();
        } else if let Some(mtm) = MainThreadMarker::new() {
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
            // In menu bar mode, deactivate shield and return to menu bar
            // In immediate mode, terminate the app
            if MENU_BAR_MODE.load(Ordering::SeqCst) {
                deactivate_shield();
            } else if let Some(mtm) = MainThreadMarker::new() {
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
pub fn stop_close_button_timer() {
    unsafe {
        let timer = TIMER_REF.swap(std::ptr::null_mut(), Ordering::SeqCst);
        if !timer.is_null() {
            CFRunLoopTimerInvalidate(timer);
        }
    }
}

/// Deactivate the shield and return to menu bar mode
///
/// This function:
/// - Closes the shield window
/// - Stops the animation timer
/// - Disables the event tap
/// - Releases the sleep assertion
/// - Re-enables the "Start Protection" menu item
/// - Resets shield state
pub fn deactivate_shield() {
    // Only deactivate if shield is active
    if !SHIELD_ACTIVE.swap(false, Ordering::SeqCst) {
        return;
    }

    println!();
    println!("  🛡️  Deactivating Cat Shield...");

    // Stop the animation timer first
    stop_close_button_timer();

    // Disable the event tap
    disable_event_tap();

    // Release sleep assertion if we have one
    if HAS_SLEEP_ASSERTION.swap(false, Ordering::SeqCst) {
        let assertion_id = SLEEP_ASSERTION_ID.load(Ordering::SeqCst) as u32;
        allow_sleep(assertion_id);
    }

    // Close the shield window and properly release it
    // We use Retained::from_raw to reclaim ownership from the raw pointer,
    // which ensures the NSWindow is properly released when dropped
    let window_ptr = SHIELD_WINDOW.swap(std::ptr::null_mut(), Ordering::SeqCst);
    if !window_ptr.is_null() {
        unsafe {
            // Reconstruct Retained to take ownership and properly release
            let window: Retained<NSWindow> =
                Retained::from_raw(window_ptr as *mut NSWindow).expect("SHIELD_WINDOW was valid");
            window.close();
            // window is dropped here, calling release() on the NSWindow
        }
        println!("  ✓ Shield window closed");
    }

    // Release the close button view properly
    // The window's content view also holds a reference, but we need to release our ownership
    let close_button_ptr = CLOSE_BUTTON_VIEW.swap(std::ptr::null_mut(), Ordering::SeqCst);
    if !close_button_ptr.is_null() {
        unsafe {
            // Reconstruct Retained to take ownership and properly release
            let _close_button: Retained<CloseButtonView> =
                Retained::from_raw(close_button_ptr as *mut CloseButtonView)
                    .expect("CLOSE_BUTTON_VIEW was valid");
            // Dropped here, calling release()
        }
    }

    // Clear close button label view reference
    let close_button_label_ptr =
        CLOSE_BUTTON_LABEL_VIEW.swap(std::ptr::null_mut(), Ordering::SeqCst);
    if !close_button_label_ptr.is_null() {
        unsafe {
            let _label: Retained<CloseButtonLabelView> =
                Retained::from_raw(close_button_label_ptr as *mut CloseButtonLabelView)
                    .expect("CLOSE_BUTTON_LABEL_VIEW was valid");
        }
    }

    // Clear timer display view reference (only set in immediate mode, but clear for safety)
    TIMER_DISPLAY_VIEW.store(std::ptr::null_mut(), Ordering::SeqCst);

    // Clear NSTextField label references
    TIMER_HEADER_LABEL.store(std::ptr::null_mut(), Ordering::SeqCst);
    TIMER_TIME_LABEL.store(std::ptr::null_mut(), Ordering::SeqCst);
    TIMER_WARNING_LABEL.store(std::ptr::null_mut(), Ordering::SeqCst);
    CLOSE_BUTTON_TEXT_LABEL.store(std::ptr::null_mut(), Ordering::SeqCst);

    // Reset auto-exit timer state
    AUTO_EXIT_ENABLED.store(false, Ordering::SeqCst);
    WARNING_SHOWN.store(false, Ordering::SeqCst);

    // Reset close button state
    MOUSE_DOWN_TIME.with(|time| time.set(None));
    IS_MOUSE_INSIDE.with(|inside| inside.set(false));

    // Re-enable the "Start Protection" menu item
    let menu_item_ptr = START_MENU_ITEM.load(Ordering::SeqCst);
    if !menu_item_ptr.is_null() {
        unsafe {
            let menu_item: &NSMenuItem = &*(menu_item_ptr as *const NSMenuItem);
            menu_item.setEnabled(true);
        }
    }

    println!();
    println!("  ✓ Cat Shield deactivated");
    println!("  Click the 🐱 icon to activate protection again.");
    println!();
}

/// Activate the shield protection
///
/// This function creates the fullscreen overlay window, sets up input blocking,
/// and prevents sleep. It's called either from the menu item action or from
/// the CLI immediate mode.
pub fn activate_shield(mtm: MainThreadMarker) {
    // Prevent double-activation
    if SHIELD_ACTIVE.swap(true, Ordering::SeqCst) {
        return;
    }

    // Disable the "Start Protection" menu item while shield is active
    let menu_item_ptr = START_MENU_ITEM.load(Ordering::SeqCst);
    if !menu_item_ptr.is_null() {
        unsafe {
            let menu_item: &NSMenuItem = &*(menu_item_ptr as *const NSMenuItem);
            menu_item.setEnabled(false);
        }
    }

    // Get the exit key configuration
    let exit_key = get_exit_key();

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
            SHIELD_ACTIVE.store(false, Ordering::SeqCst);
            // Re-enable menu item
            if !menu_item_ptr.is_null() {
                unsafe {
                    let menu_item: &NSMenuItem = &*(menu_item_ptr as *const NSMenuItem);
                    menu_item.setEnabled(true);
                }
            }
            return;
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

    // Store window reference for cleanup
    SHIELD_WINDOW.store(Retained::as_ptr(&window) as *mut c_void, Ordering::SeqCst);

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

    // Prevent sleep
    if let Some(assertion_id) = prevent_sleep() {
        SLEEP_ASSERTION_ID.store(assertion_id as u64, Ordering::SeqCst);
        HAS_SLEEP_ASSERTION.store(true, Ordering::SeqCst);
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
        // Deactivate and return to menu bar mode
        deactivate_shield();
        return;
    }
    println!("  ✓ Input blocking active");

    println!();
    println!("  ═══════════════════════════════════════");
    println!("  🛡️  CAT SHIELD IS NOW ACTIVE!");
    println!("  ═══════════════════════════════════════");
    println!();
    println!("  Exit: Hold X button (top-right) for 3 seconds");
    println!("        Or press {}", exit_key.display_name);
    println!();

    // Keep the window and views retained so they don't get deallocated
    std::mem::forget(window);
    std::mem::forget(close_button);
    std::mem::forget(close_button_label);
}
