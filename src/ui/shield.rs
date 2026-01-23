//! Shield overlay activation and deactivation for Cat Shield
//!
//! This module is macOS-only. Windows and Linux support is planned for future releases.

use crate::cat_animation;
use crate::config::Config;
use crate::input::get_exit_key;
use crate::platform::{
    allow_sleep, disable_event_tap, prevent_sleep, setup_event_tap, CFRunLoopGetCurrent,
};
use crate::shield_core::{
    create_shield_window, ensure_accessibility, print_activation_banner, print_shield_active,
    setup_close_button, setup_timer_display,
};
use crate::timer::{
    format_duration, get_remaining_seconds, parse_duration, AUTO_EXIT_ENABLED, WARNING_SHOWN,
};
use crate::ui::ptr_helper::with_ptr_void;
use crate::ui::state::{menu_bar, shield, IS_MOUSE_INSIDE, MOUSE_DOWN_TIME};
use crate::ui::views::{CatAnimationView, CloseButtonLabelView, CloseButtonView, TimerDisplayView};
use objc2::rc::Retained;
use objc2_app_kit::{NSMenuItem, NSScreen, NSTextField, NSWindow};
use objc2_core_foundation::{kCFRunLoopCommonModes, CFString, CGRect};
use objc2_foundation::MainThreadMarker;
use std::ffi::c_void;
use std::mem::ManuallyDrop;
use std::sync::atomic::Ordering;

use crate::platform::{
    CFAbsoluteTimeGetCurrent, CFRunLoopAddTimer, CFRunLoopTimerCreate, CFRunLoopTimerInvalidate,
};
use crate::timer::WARNING_SECONDS;
use crate::ui::state::animation;

/// Timer callback to update progress, check for exit condition, and trigger redraw.
///
/// This callback handles both menu bar mode (returns to menu bar) and immediate mode
/// (terminates the app) based on the shield::MODE_MENU_BAR flag.
///
/// # Safety
/// This function is called from the CFRunLoop timer and must be `unsafe extern "C"`.
/// The caller (CFRunLoop) guarantees:
/// - The function is called on the thread that owns the run loop
/// - The timer has not been invalidated
/// - The callback is invoked at regular intervals as configured
pub unsafe extern "C" fn timer_callback(_timer: *mut c_void, _info: *mut c_void) {
    use crate::ui::state::{close_button, is_hold_complete};
    use objc2_app_kit::{NSApplication, NSView};

    // Cache all atomic loads upfront to avoid redundant loads in the hot path.
    // This callback runs at 60Hz, so minimizing atomic operations improves performance.
    let menu_bar_mode = shield::MODE_MENU_BAR.load(Ordering::Acquire);
    let close_btn_ptr = shield::CLOSE_BUTTON.load(Ordering::Acquire);
    let label_ptr = shield::CLOSE_BUTTON_LABEL.load(Ordering::Acquire);
    let timer_ptr = shield::TIMER_VIEW.load(Ordering::Acquire);
    let cat_ptr = shield::CAT_ANIMATION_VIEW.load(Ordering::Acquire);

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
        if menu_bar_mode {
            deactivate_shield();
        } else if let Some(mtm) = MainThreadMarker::new() {
            let app = NSApplication::sharedApplication(mtm);
            app.terminate(None);
        }
        return;
    }

    // Check auto-exit timer
    if AUTO_EXIT_ENABLED.load(Ordering::Acquire) {
        let remaining = get_remaining_seconds();

        // Show warning when approaching exit
        if remaining <= WARNING_SECONDS && !WARNING_SHOWN.swap(true, Ordering::AcqRel) {
            log::warn!("Auto-exit in {} seconds!", remaining);
        }

        // Check if timer has expired
        if remaining == 0 {
            log::info!("⏰ Timer expired - auto-exiting...");
            // In menu bar mode, deactivate shield and return to menu bar
            // In immediate mode, terminate the app
            if menu_bar_mode {
                deactivate_shield();
            } else if let Some(mtm) = MainThreadMarker::new() {
                let app = NSApplication::sharedApplication(mtm);
                app.terminate(None);
            }
            return;
        }
    }

    // Trigger redraw of close button using cached pointer
    if !close_btn_ptr.is_null() {
        // SAFETY: The pointer was loaded from shield::CLOSE_BUTTON which stores a valid
        // NSView pointer that was created in activate_shield. The pointer remains valid
        // for the lifetime of the shield window.
        let view: &NSView = &*(close_btn_ptr as *const NSView);
        view.setNeedsDisplay(true);
    }

    // Trigger redraw of close button label (for countdown during hold) using cached pointer
    if !label_ptr.is_null() {
        // SAFETY: The pointer was loaded from shield::CLOSE_BUTTON_LABEL which stores a valid
        // NSView pointer that was created in activate_shield. The pointer remains valid
        // for the lifetime of the shield window.
        let view: &NSView = &*(label_ptr as *const NSView);
        view.setNeedsDisplay(true);
    }

    // Trigger redraw of timer display using cached pointer
    if !timer_ptr.is_null() {
        // SAFETY: The pointer was loaded from shield::TIMER_VIEW which stores a valid
        // NSView pointer that was created in setup (immediate mode) or activate_shield.
        // The pointer remains valid for the lifetime of the shield window.
        let view: &NSView = &*(timer_ptr as *const NSView);
        view.setNeedsDisplay(true);
    }

    // Trigger redraw of cat animation view using cached pointer
    if !cat_ptr.is_null() {
        // SAFETY: The pointer was loaded from shield::CAT_ANIMATION_VIEW which stores a valid
        // NSView pointer that was created in activate_shield. The pointer remains valid
        // for the lifetime of the shield window.
        let view: &NSView = &*(cat_ptr as *const NSView);
        view.setNeedsDisplay(true);
    }
}

/// Start the animation timer for the close button
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

/// Stop the animation timer
pub fn stop_close_button_timer() {
    // SAFETY: CFRunLoopTimerInvalidate is safe because:
    // - The timer pointer was obtained from our own atomic swap
    // - We check for null before calling invalidate
    // - The timer was created by CFRunLoopTimerCreate in start_close_button_timer
    // - After invalidation, the timer will not fire again
    unsafe {
        let timer = shield::TIMER_REF.swap(std::ptr::null_mut(), Ordering::AcqRel);
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
    // Use AcqRel for swap: acquire to see previous writes, release for new value
    if !shield::IS_ACTIVE.swap(false, Ordering::AcqRel) {
        return;
    }

    log::info!("🛡️  Deactivating Cat Shield...");

    // Stop the animation timer first
    stop_close_button_timer();

    // Disable the event tap
    disable_event_tap();

    // Release sleep assertion if we have one
    if shield::HAS_SLEEP_ASSERTION.swap(false, Ordering::AcqRel) {
        let assertion_id = shield::SLEEP_ASSERTION_ID.load(Ordering::Acquire) as u32;
        allow_sleep(assertion_id);
    }

    // Close the shield window and properly release it
    // We use Retained::from_raw to reclaim ownership from the raw pointer,
    // which ensures the NSWindow is properly released when dropped
    let window_ptr = shield::WINDOW.swap(std::ptr::null_mut(), Ordering::AcqRel);
    if !window_ptr.is_null() {
        // SAFETY: Retained::from_raw is safe because:
        // - window_ptr was stored from a valid Retained<NSWindow> in activate_shield
        // - The atomic swap ensures we only reclaim ownership once
        // - The pointer type cast is correct (it was stored as *mut c_void from NSWindow)
        // - The window is valid until we drop it here
        unsafe {
            // Reconstruct Retained to take ownership and properly release
            let window: Retained<NSWindow> =
                Retained::from_raw(window_ptr as *mut NSWindow).expect("shield::WINDOW was valid");
            window.close();
            // window is dropped here, calling release() on the NSWindow
        }
        log::info!("✓ Shield window closed");
    }

    // Release the close button view properly
    // The window's content view also holds a reference, but we need to release our ownership
    let close_button_ptr = shield::CLOSE_BUTTON.swap(std::ptr::null_mut(), Ordering::AcqRel);
    if !close_button_ptr.is_null() {
        // SAFETY: Retained::from_raw is safe because:
        // - close_button_ptr was stored from a valid Retained<CloseButtonView> in activate_shield
        // - The atomic swap ensures we only reclaim ownership once
        // - The pointer type cast is correct (it was stored as *mut c_void from CloseButtonView)
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
        shield::CLOSE_BUTTON_LABEL.swap(std::ptr::null_mut(), Ordering::AcqRel);
    if !close_button_label_ptr.is_null() {
        // SAFETY: Retained::from_raw is safe because:
        // - close_button_label_ptr was stored from a valid Retained in activate_shield
        // - The atomic swap ensures we only reclaim ownership once
        // - The pointer type cast is correct
        unsafe {
            let _label: Retained<CloseButtonLabelView> =
                Retained::from_raw(close_button_label_ptr as *mut CloseButtonLabelView)
                    .expect("shield::CLOSE_BUTTON_LABEL was valid");
        }
    }

    // Release the timer display view properly
    // The window's content view also holds a reference, but we need to release our ownership
    let timer_view_ptr = shield::TIMER_VIEW.swap(std::ptr::null_mut(), Ordering::AcqRel);
    if !timer_view_ptr.is_null() {
        // SAFETY: Retained::from_raw is safe because:
        // - timer_view_ptr was stored from a valid Retained<TimerDisplayView> in setup_timer_display
        // - The atomic swap ensures we only reclaim ownership once
        // - The pointer type cast is correct (it was stored as *mut c_void from TimerDisplayView)
        unsafe {
            let _timer_view: Retained<TimerDisplayView> =
                Retained::from_raw(timer_view_ptr as *mut TimerDisplayView)
                    .expect("shield::TIMER_VIEW was valid");
            // Dropped here, calling release()
        }
    }

    // Release NSTextField label references properly to avoid memory leaks
    // Each label was created with Retained and forgotten, so we must reclaim ownership
    let timer_header_ptr = shield::TIMER_HEADER.swap(std::ptr::null_mut(), Ordering::AcqRel);
    if !timer_header_ptr.is_null() {
        // SAFETY: Retained::from_raw is safe because:
        // - timer_header_ptr was stored from a valid Retained<NSTextField> in timer_display.rs
        // - The atomic swap ensures we only reclaim ownership once
        // - The pointer type cast is correct (it was stored as *mut c_void from NSTextField)
        unsafe {
            let _label: Retained<NSTextField> =
                Retained::from_raw(timer_header_ptr as *mut NSTextField)
                    .expect("shield::TIMER_HEADER was valid");
            // Dropped here, calling release()
        }
    }

    let timer_time_ptr = shield::TIMER_TIME.swap(std::ptr::null_mut(), Ordering::AcqRel);
    if !timer_time_ptr.is_null() {
        // SAFETY: Retained::from_raw is safe because:
        // - timer_time_ptr was stored from a valid Retained<NSTextField> in timer_display.rs
        // - The atomic swap ensures we only reclaim ownership once
        // - The pointer type cast is correct (it was stored as *mut c_void from NSTextField)
        unsafe {
            let _label: Retained<NSTextField> =
                Retained::from_raw(timer_time_ptr as *mut NSTextField)
                    .expect("shield::TIMER_TIME was valid");
            // Dropped here, calling release()
        }
    }

    let timer_warning_ptr = shield::TIMER_WARNING.swap(std::ptr::null_mut(), Ordering::AcqRel);
    if !timer_warning_ptr.is_null() {
        // SAFETY: Retained::from_raw is safe because:
        // - timer_warning_ptr was stored from a valid Retained<NSTextField> in timer_display.rs
        // - The atomic swap ensures we only reclaim ownership once
        // - The pointer type cast is correct (it was stored as *mut c_void from NSTextField)
        unsafe {
            let _label: Retained<NSTextField> =
                Retained::from_raw(timer_warning_ptr as *mut NSTextField)
                    .expect("shield::TIMER_WARNING was valid");
            // Dropped here, calling release()
        }
    }

    let close_button_text_ptr =
        shield::CLOSE_BUTTON_TEXT.swap(std::ptr::null_mut(), Ordering::AcqRel);
    if !close_button_text_ptr.is_null() {
        // SAFETY: Retained::from_raw is safe because:
        // - close_button_text_ptr was stored from a valid Retained<NSTextField> in close_button_label.rs
        // - The atomic swap ensures we only reclaim ownership once
        // - The pointer type cast is correct (it was stored as *mut c_void from NSTextField)
        unsafe {
            let _label: Retained<NSTextField> =
                Retained::from_raw(close_button_text_ptr as *mut NSTextField)
                    .expect("shield::CLOSE_BUTTON_TEXT was valid");
            // Dropped here, calling release()
        }
    }

    // Release the cat animation view properly
    let cat_animation_ptr = shield::CAT_ANIMATION_VIEW.swap(std::ptr::null_mut(), Ordering::AcqRel);
    if !cat_animation_ptr.is_null() {
        // SAFETY: Retained::from_raw is safe because:
        // - cat_animation_ptr was stored from a valid Retained<CatAnimationView> in activate_shield
        // - The atomic swap ensures we only reclaim ownership once
        // - The pointer type cast is correct (it was stored as *mut c_void from CatAnimationView)
        unsafe {
            let _cat_view: Retained<CatAnimationView> =
                Retained::from_raw(cat_animation_ptr as *mut CatAnimationView)
                    .expect("shield::CAT_ANIMATION_VIEW was valid");
            // Dropped here, calling release()
        }
    }

    // Cleanup cat animation state
    cat_animation::cleanup();

    // Reset auto-exit timer state
    AUTO_EXIT_ENABLED.store(false, Ordering::Release);
    WARNING_SHOWN.store(false, Ordering::Release);

    // Reset close button state
    MOUSE_DOWN_TIME.with(|time| time.set(None));
    IS_MOUSE_INSIDE.with(|inside| inside.set(false));

    // Clear allowed keys
    use crate::input::clear_allowed_keys;
    clear_allowed_keys();

    // Re-enable the "Start Protection" menu item
    unsafe {
        with_ptr_void::<NSMenuItem, _>(&menu_bar::START_ITEM, |menu_item| {
            menu_item.setEnabled(true);
        });
    }

    log::info!("✓ Cat Shield deactivated");
    log::info!("Click the 🐱 icon to activate protection again.");
}

/// Activate the shield protection
///
/// This function creates the fullscreen overlay window, sets up input blocking,
/// and prevents sleep. It's called either from the menu item action or from
/// the CLI immediate mode.
pub fn activate_shield(mtm: MainThreadMarker) {
    // Prevent double-activation
    // Use AcqRel for swap to prevent double-activation
    if shield::IS_ACTIVE.swap(true, Ordering::AcqRel) {
        return;
    }

    // Disable the "Start Protection" menu item while shield is active
    unsafe {
        with_ptr_void::<NSMenuItem, _>(&menu_bar::START_ITEM, |menu_item| {
            menu_item.setEnabled(false);
        });
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
            log::error!("Failed to get main screen");
            shield::IS_ACTIVE.store(false, Ordering::Release);
            // Re-enable menu item
            unsafe {
                with_ptr_void::<NSMenuItem, _>(&menu_bar::START_ITEM, |menu_item| {
                    menu_item.setEnabled(true);
                });
            }
            return;
        }
    };
    let screen_frame = screen.frame();

    // Create the shield window using shared logic
    let window = create_shield_window(mtm, screen_frame);

    // Store window reference for cleanup
    shield::WINDOW.store(Retained::as_ptr(&window) as *mut c_void, Ordering::Release);

    // Show the window
    window.makeKeyAndOrderFront(None);

    log::info!("✓ Overlay window active");

    // Create close button and label using shared logic
    let (close_button, close_button_label) = setup_close_button(mtm, screen_frame);

    // Store view references for timer callback
    // Use Release ordering - timer callback will use Acquire to read
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

    log::info!("✓ Close button active (hold 3s to exit)");
    log::info!("✓ Exit key: {}", exit_key.display_name);

    // Check for default timer in config and set up auto-exit if configured (uses shared helper)
    let config = Config::load();
    let timer_duration_secs: Option<u64> = if let Some(ref timer_str) = config.default_timer {
        match parse_duration(timer_str) {
            Ok(duration_secs) => {
                setup_timer_display(mtm, &window, screen_frame, duration_secs);
                Some(duration_secs)
            }
            Err(e) => {
                log::warn!("Invalid default_timer in config: {}", e);
                None
            }
        }
    } else {
        None
    };

    // Set up cat animation if enabled in config
    if config.show_cat_animation() {
        // Initialize cat animation with screen dimensions
        cat_animation::init(screen_frame.size.width, screen_frame.size.height, &config);

        // Create cat animation view (fullscreen to allow cat to move anywhere)
        let cat_frame = CGRect {
            origin: objc2_core_foundation::CGPoint { x: 0.0, y: 0.0 },
            size: screen_frame.size,
        };
        let cat_animation_view = CatAnimationView::new(mtm, cat_frame);

        // Store reference for timer callback
        shield::CAT_ANIMATION_VIEW.store(
            Retained::as_ptr(&cat_animation_view) as *mut c_void,
            Ordering::Release,
        );

        // Add to window's content view
        if let Some(content_view) = window.contentView() {
            content_view.addSubview(&cat_animation_view);
        }

        log::info!("✓ Animated cat companion active");

        // Transfer ownership to ManuallyDrop
        let _ = ManuallyDrop::new(cat_animation_view);
    }

    // Load allowed keys from config
    if let Some(ref allowed_keys_strs) = config.allowed_keys {
        use crate::input::parse_and_set_allowed_keys;

        match parse_and_set_allowed_keys(allowed_keys_strs) {
            Ok(()) => {
                if !allowed_keys_strs.is_empty() {
                    log::info!("✓ Allowed keys: {}", allowed_keys_strs.join(", "));
                }
            }
            Err(errors) => {
                log::warn!("Invalid allowed_keys in config:");
                for error in errors {
                    log::warn!("  {}", error);
                }
            }
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
        log::error!("Failed to create event tap");
        log::error!("════════════════════════════════════════");
        log::error!("SHIELD ACTIVATION FAILED");
        log::error!("════════════════════════════════════════");
        log::error!("Input blocking could not be enabled.");
        log::error!("The shield cannot protect without this feature.");
        log::error!("Please check:");
        log::error!("  - Accessibility permissions are granted");
        log::error!("  - No other apps are blocking event taps");
        // Deactivate and return to menu bar mode
        deactivate_shield();
        return;
    }
    log::info!("✓ Input blocking active");

    // Print shield active status (shared)
    let timer_info = timer_duration_secs
        .map(|_| format!("{} remaining", format_duration(get_remaining_seconds())));
    print_shield_active(&exit_key, timer_info.as_deref());

    // Transfer ownership to ManuallyDrop to prevent deallocation while shield is active.
    // The raw pointers stored in global state are reclaimed in deactivate_shield()
    // using Retained::from_raw, which properly releases the objects.
    let _ = ManuallyDrop::new(window);
    let _ = ManuallyDrop::new(close_button);
    let _ = ManuallyDrop::new(close_button_label);
}
