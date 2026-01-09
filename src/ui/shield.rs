//! Shield overlay activation and deactivation for Cat Shield

use crate::input::get_exit_key;
use crate::platform::{
    allow_sleep, disable_event_tap, prevent_sleep, setup_event_tap, CFRunLoopGetCurrent,
};
use crate::shield_core::{
    create_shield_window, ensure_accessibility, print_activation_banner, print_shield_active,
    setup_close_button,
};
use crate::timer::{AUTO_EXIT_ENABLED, WARNING_SHOWN};
use crate::ui::state::{menu_bar, shield, IS_MOUSE_INSIDE, MOUSE_DOWN_TIME};
use crate::ui::views::{CloseButtonLabelView, CloseButtonView};
use objc2::rc::Retained;
use objc2_app_kit::{NSMenuItem, NSScreen, NSWindow};
use objc2_core_foundation::{kCFRunLoopCommonModes, CFString};
use objc2_foundation::MainThreadMarker;
use std::ffi::c_void;
use std::mem::ManuallyDrop;
use std::sync::atomic::Ordering;

use crate::platform::{
    CFAbsoluteTimeGetCurrent, CFRunLoopAddTimer, CFRunLoopTimerCreate, CFRunLoopTimerInvalidate,
};
use crate::timer::{get_remaining_seconds, WARNING_SECONDS};
use crate::ui::state::animation;

/// Timer callback to update progress, check for exit condition, and trigger redraw.
///
/// This callback handles both menu bar mode (returns to menu bar) and immediate mode
/// (terminates the app) based on the shield::MODE_MENU_BAR flag.
///
/// # Safety
/// This function is called from the CFRunLoop timer and must be `unsafe extern "C"`.
pub unsafe extern "C" fn timer_callback(_timer: *mut c_void, _info: *mut c_void) {
    use crate::ui::state::{close_button, is_hold_complete};
    use objc2_app_kit::{NSApplication, NSView};

    // Check if hold duration has been exceeded (close button)
    let should_exit_from_button = MOUSE_DOWN_TIME.with(|time| {
        if let Some(start) = time.get() {
            let is_inside = IS_MOUSE_INSIDE.with(|inside| inside.get());
            is_inside
                && is_hold_complete(
                    start.elapsed().as_secs_f64(),
                    close_button::HOLD_DURATION_SECS,
                )
        } else {
            false
        }
    });

    if should_exit_from_button {
        // In menu bar mode, deactivate shield and return to menu bar
        // In immediate mode, terminate the app
        if shield::MODE_MENU_BAR.load(Ordering::SeqCst) {
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
            if shield::MODE_MENU_BAR.load(Ordering::SeqCst) {
                deactivate_shield();
            } else if let Some(mtm) = MainThreadMarker::new() {
                let app = NSApplication::sharedApplication(mtm);
                app.terminate(None);
            }
            return;
        }
    }

    // Trigger redraw of close button
    let view_ptr = shield::CLOSE_BUTTON.load(Ordering::SeqCst);
    if !view_ptr.is_null() {
        let view: &NSView = &*(view_ptr as *const NSView);
        view.setNeedsDisplay(true);
    }

    // Trigger redraw of close button label (for countdown during hold)
    let label_view_ptr = shield::CLOSE_BUTTON_LABEL.load(Ordering::SeqCst);
    if !label_view_ptr.is_null() {
        let view: &NSView = &*(label_view_ptr as *const NSView);
        view.setNeedsDisplay(true);
    }

    // Trigger redraw of timer display
    let timer_view_ptr = shield::TIMER_VIEW.load(Ordering::SeqCst);
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
            shield::TIMER_REF.store(timer, Ordering::SeqCst);
        }
    }
}

/// Stop the animation timer
pub fn stop_close_button_timer() {
    unsafe {
        let timer = shield::TIMER_REF.swap(std::ptr::null_mut(), Ordering::SeqCst);
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
    if !shield::IS_ACTIVE.swap(false, Ordering::SeqCst) {
        return;
    }

    println!();
    println!("  🛡️  Deactivating Cat Shield...");

    // Stop the animation timer first
    stop_close_button_timer();

    // Disable the event tap
    disable_event_tap();

    // Release sleep assertion if we have one
    if shield::HAS_SLEEP_ASSERTION.swap(false, Ordering::SeqCst) {
        let assertion_id = shield::SLEEP_ASSERTION_ID.load(Ordering::SeqCst) as u32;
        allow_sleep(assertion_id);
    }

    // Close the shield window and properly release it
    // We use Retained::from_raw to reclaim ownership from the raw pointer,
    // which ensures the NSWindow is properly released when dropped
    let window_ptr = shield::WINDOW.swap(std::ptr::null_mut(), Ordering::SeqCst);
    if !window_ptr.is_null() {
        unsafe {
            // Reconstruct Retained to take ownership and properly release
            let window: Retained<NSWindow> =
                Retained::from_raw(window_ptr as *mut NSWindow).expect("shield::WINDOW was valid");
            window.close();
            // window is dropped here, calling release() on the NSWindow
        }
        println!("  ✓ Shield window closed");
    }

    // Release the close button view properly
    // The window's content view also holds a reference, but we need to release our ownership
    let close_button_ptr = shield::CLOSE_BUTTON.swap(std::ptr::null_mut(), Ordering::SeqCst);
    if !close_button_ptr.is_null() {
        unsafe {
            // Reconstruct Retained to take ownership and properly release
            let _close_button: Retained<CloseButtonView> =
                Retained::from_raw(close_button_ptr as *mut CloseButtonView)
                    .expect("shield::CLOSE_BUTTON was valid");
            // Dropped here, calling release()
        }
    }

    // Clear close button label view reference
    let close_button_label_ptr =
        shield::CLOSE_BUTTON_LABEL.swap(std::ptr::null_mut(), Ordering::SeqCst);
    if !close_button_label_ptr.is_null() {
        unsafe {
            let _label: Retained<CloseButtonLabelView> =
                Retained::from_raw(close_button_label_ptr as *mut CloseButtonLabelView)
                    .expect("shield::CLOSE_BUTTON_LABEL was valid");
        }
    }

    // Clear timer display view reference (only set in immediate mode, but clear for safety)
    shield::TIMER_VIEW.store(std::ptr::null_mut(), Ordering::SeqCst);

    // Clear NSTextField label references
    shield::TIMER_HEADER.store(std::ptr::null_mut(), Ordering::SeqCst);
    shield::TIMER_TIME.store(std::ptr::null_mut(), Ordering::SeqCst);
    shield::TIMER_WARNING.store(std::ptr::null_mut(), Ordering::SeqCst);
    shield::CLOSE_BUTTON_TEXT.store(std::ptr::null_mut(), Ordering::SeqCst);

    // Reset auto-exit timer state
    AUTO_EXIT_ENABLED.store(false, Ordering::SeqCst);
    WARNING_SHOWN.store(false, Ordering::SeqCst);

    // Reset close button state
    MOUSE_DOWN_TIME.with(|time| time.set(None));
    IS_MOUSE_INSIDE.with(|inside| inside.set(false));

    // Re-enable the "Start Protection" menu item
    let menu_item_ptr = menu_bar::START_ITEM.load(Ordering::SeqCst);
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
    if shield::IS_ACTIVE.swap(true, Ordering::SeqCst) {
        return;
    }

    // Disable the "Start Protection" menu item while shield is active
    let menu_item_ptr = menu_bar::START_ITEM.load(Ordering::SeqCst);
    if !menu_item_ptr.is_null() {
        unsafe {
            let menu_item: &NSMenuItem = &*(menu_item_ptr as *const NSMenuItem);
            menu_item.setEnabled(false);
        }
    }

    // Get the exit key configuration
    let exit_key = get_exit_key();

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
            shield::IS_ACTIVE.store(false, Ordering::SeqCst);
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

    // Create the shield window using shared logic
    let window = create_shield_window(mtm, screen_frame);

    // Store window reference for cleanup
    shield::WINDOW.store(Retained::as_ptr(&window) as *mut c_void, Ordering::SeqCst);

    // Show the window
    window.makeKeyAndOrderFront(None);

    println!("  ✓ Overlay window active");

    // Create close button and label using shared logic
    let (close_button, close_button_label) = setup_close_button(mtm, screen_frame);

    // Store view references for timer callback
    shield::CLOSE_BUTTON.store(
        Retained::as_ptr(&close_button) as *mut c_void,
        Ordering::SeqCst,
    );
    shield::CLOSE_BUTTON_LABEL.store(
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
        shield::SLEEP_ASSERTION_ID.store(assertion_id as u64, Ordering::SeqCst);
        shield::HAS_SLEEP_ASSERTION.store(true, Ordering::SeqCst);
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

    // Print shield active status (shared)
    print_shield_active(&exit_key, None);

    // Transfer ownership to ManuallyDrop to prevent deallocation while shield is active.
    // The raw pointers stored in global state are reclaimed in deactivate_shield()
    // using Retained::from_raw, which properly releases the objects.
    let _ = ManuallyDrop::new(window);
    let _ = ManuallyDrop::new(close_button);
    let _ = ManuallyDrop::new(close_button_label);
}
