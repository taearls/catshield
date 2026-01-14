//! Shared shield activation logic for Cat Shield
//!
//! This module contains shared functions used by both immediate mode (main.rs)
//! and menu bar mode (ui/shield.rs) for:
//! - Accessibility permission checking and polling
//! - Shield window creation and configuration
//! - Close button setup
//! - Timer display setup
//!
//! By extracting this shared logic, we avoid duplication and ensure
//! consistent behavior between the two modes.
//!
//! # Platform Abstraction
//!
//! The permission checking functions use the `PermissionChecker` trait,
//! allowing different implementations for macOS, Windows, and Linux.

use crate::config::get_current_config;
use crate::input::ExitKey;
#[cfg(target_os = "macos")]
use crate::platform::CFRunLoopRunInMode;
use crate::platform::PermissionChecker;
use crate::timer::{format_duration, init_auto_exit_timer};
use crate::ui::state::{close_button, shield, timer_display, window_level};
use crate::ui::views::{CloseButtonLabelView, CloseButtonView, TimerDisplayView};
#[cfg(target_os = "macos")]
use objc2::rc::Retained;
#[cfg(target_os = "macos")]
use objc2::MainThreadOnly;
#[cfg(target_os = "macos")]
use objc2_app_kit::{
    NSBackingStoreType, NSColor, NSWindow, NSWindowCollectionBehavior, NSWindowStyleMask,
};
#[cfg(target_os = "macos")]
use objc2_core_foundation::{CGPoint, CGRect, CGSize};
#[cfg(target_os = "macos")]
use objc2_foundation::{ns_string, MainThreadMarker};
#[cfg(target_os = "macos")]
use std::ffi::c_void;
#[cfg(target_os = "macos")]
use std::sync::atomic::Ordering;

/// UI theme constants for the shield overlay
pub mod theme {
    /// Background color RGB values (dark theme)
    pub const BG_RED: f64 = 0.1;
    pub const BG_GREEN: f64 = 0.1;
    pub const BG_BLUE: f64 = 0.15;

    /// Gap between close button and its label
    pub const BUTTON_LABEL_GAP: f64 = 5.0;
}

/// Ensure platform permissions are granted, prompting the user if necessary.
///
/// This function blocks until required permissions are granted:
/// 1. Checks if permissions are already granted (returns immediately if so)
/// 2. If not, prompts the user with the system dialog
/// 3. If still not granted, opens system settings and polls until granted
///
/// # Arguments
/// * `permission_checker` - Platform-specific permission checker implementation
/// * `exit_key` - The configured exit key for display in messages
///
/// # Type Parameters
/// * `P` - A type implementing the `PermissionChecker` trait
///
/// # Note
/// This function will loop indefinitely until permissions are granted.
/// It does not return until the user grants the required access.
pub fn ensure_permissions<P: PermissionChecker>(permission_checker: &P, exit_key: &ExitKey) {
    if permission_checker.check_permissions() {
        return;
    }

    log::info!("🐱 CAT SHIELD 🛡️");
    log::info!("════════════════════════════════════════");
    log::warn!("ACCESSIBILITY PERMISSION REQUIRED");
    log::warn!(
        "To block keyboard/mouse input and use the exit shortcut ({}), this app needs Accessibility permissions.",
        exit_key.display_name
    );

    // Try to prompt user with native dialog
    log::info!("Requesting accessibility permissions...");
    if permission_checker.check_permissions_with_prompt() {
        log::info!("✓ Permissions granted!");
        return;
    }

    log::info!("Opening System Settings → Accessibility...");

    if permission_checker.open_permissions_settings().is_ok() {
        log::info!("✓ System Settings opened");
    }
    log::info!("Please add Cat Shield to the Accessibility list.");
    log::info!("Waiting for permissions...");

    // Poll for permissions - platform-specific sleep mechanism
    poll_for_permissions(permission_checker);
}

/// Poll for permissions until granted.
///
/// This function implements platform-specific polling:
/// - **macOS**: Uses CFRunLoopRunInMode to allow UI updates during polling
/// - **Other platforms**: Uses standard thread sleep
#[cfg(target_os = "macos")]
fn poll_for_permissions<P: PermissionChecker>(permission_checker: &P) {
    use objc2_core_foundation::{kCFRunLoopDefaultMode, CFString};

    const POLL_INTERVAL_SECS: f64 = 1.0;
    loop {
        // SAFETY: CFRunLoopRunInMode is safe to call because:
        // - kCFRunLoopDefaultMode is a valid, non-null constant from Core Foundation
        // - The pointer cast is valid for the expected type
        // - We're calling this on the main thread which owns this run loop
        unsafe {
            let mode = kCFRunLoopDefaultMode.expect("kCFRunLoopDefaultMode should exist");
            CFRunLoopRunInMode((mode as *const CFString).cast(), POLL_INTERVAL_SECS, false);
        }
        if permission_checker.check_permissions() {
            log::info!("✓ Permissions granted! Starting Cat Shield...");
            return;
        }
    }
}

/// Poll for permissions until granted (non-macOS platforms).
#[cfg(not(target_os = "macos"))]
fn poll_for_permissions<P: PermissionChecker>(permission_checker: &P) {
    use std::thread;
    use std::time::Duration;

    const POLL_INTERVAL_MS: u64 = 1000;
    loop {
        thread::sleep(Duration::from_millis(POLL_INTERVAL_MS));
        if permission_checker.check_permissions() {
            log::info!("✓ Permissions granted! Starting Cat Shield...");
            return;
        }
    }
}

/// Legacy wrapper for backward compatibility.
///
/// This function creates a macOS permission checker and calls `ensure_permissions`.
/// New code should use `ensure_permissions` directly with the appropriate
/// platform-specific permission checker.
#[cfg(target_os = "macos")]
pub fn ensure_accessibility(exit_key: &ExitKey) {
    use crate::platform::MacOSPermissionChecker;
    let checker = MacOSPermissionChecker::new();
    ensure_permissions(&checker, exit_key);
}

/// Create and configure the shield overlay window.
///
/// Creates a fullscreen, borderless, topmost window with:
/// - Screen saver window level (appears above everything)
/// - Appears on all spaces (Spaces/Mission Control)
/// - Semi-transparent dark background using config opacity
/// - Accepts mouse events (for input blocking)
///
/// # Arguments
/// * `mtm` - MainThreadMarker proving we're on the main thread
/// * `frame` - The frame rectangle (typically full screen)
///
/// # Returns
/// A retained reference to the configured NSWindow
#[cfg(target_os = "macos")]
pub fn create_shield_window(mtm: MainThreadMarker, frame: CGRect) -> Retained<NSWindow> {
    // SAFETY: NSWindow initialization is safe because:
    // - mtm (MainThreadMarker) guarantees we're on the main thread
    // - The frame rect is a valid CGRect with reasonable dimensions
    // - NSWindowStyleMask::Borderless is a valid style mask
    // - NSBackingStoreType::Buffered is a valid backing store type
    let window = unsafe {
        let window = NSWindow::alloc(mtm);
        NSWindow::initWithContentRect_styleMask_backing_defer(
            window,
            frame,
            NSWindowStyleMask::Borderless,
            NSBackingStoreType::Buffered,
            false,
        )
    };

    // Configure window to be topmost
    window.setLevel(window_level::SCREEN_SAVER);

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

    // Get opacity from config (uses default if not set)
    let config = get_current_config();
    let opacity = config.opacity();

    // Set a semi-transparent dark background color (the overlay effect)
    let bg_color = NSColor::colorWithRed_green_blue_alpha(
        theme::BG_RED,
        theme::BG_GREEN,
        theme::BG_BLUE,
        opacity,
    );
    window.setBackgroundColor(Some(&bg_color));

    // Keep window visible
    window.setHidesOnDeactivate(false);

    // Accept mouse events (needed for blocking)
    window.setIgnoresMouseEvents(false);

    // Set title
    window.setTitle(ns_string!("Cat Shield"));

    // SAFETY: setReleasedWhenClosed is safe because:
    // - The window reference is valid (just created and configured)
    // - We're on the main thread (guaranteed by mtm)
    // - Setting to false prevents automatic deallocation, which is correct
    //   since we manage the window lifetime via Retained<NSWindow>
    unsafe {
        window.setReleasedWhenClosed(false);
    }

    window
}

/// Create the close button view and its label for the shield overlay.
///
/// Positions the close button in the top-right corner with a label below it
/// showing "Hold 3s to exit" instructions.
///
/// # Arguments
/// * `mtm` - MainThreadMarker proving we're on the main thread
/// * `screen_frame` - The screen frame used for positioning
///
/// # Returns
/// A tuple of (close_button_view, close_button_label_view)
#[cfg(target_os = "macos")]
pub fn setup_close_button(
    mtm: MainThreadMarker,
    screen_frame: CGRect,
) -> (Retained<CloseButtonView>, Retained<CloseButtonLabelView>) {
    // Create close button in top-right corner
    let close_button_frame = CGRect {
        origin: CGPoint {
            x: screen_frame.size.width - close_button::SIZE - close_button::MARGIN,
            y: screen_frame.size.height - close_button::SIZE - close_button::MARGIN,
        },
        size: CGSize {
            width: close_button::SIZE,
            height: close_button::SIZE,
        },
    };

    let close_button = CloseButtonView::new(mtm, close_button_frame);

    // Create the close button label view (positioned below the button)
    let label_x = screen_frame.size.width
        - close_button::MARGIN
        - close_button::SIZE / 2.0
        - close_button::LABEL_WIDTH / 2.0;
    let label_y = screen_frame.size.height
        - close_button::SIZE
        - close_button::MARGIN
        - close_button::LABEL_HEIGHT
        - theme::BUTTON_LABEL_GAP;

    let close_button_label_frame = CGRect {
        origin: CGPoint {
            x: label_x,
            y: label_y,
        },
        size: CGSize {
            width: close_button::LABEL_WIDTH,
            height: close_button::LABEL_HEIGHT,
        },
    };
    let close_button_label = CloseButtonLabelView::new(mtm, close_button_label_frame);

    (close_button, close_button_label)
}

/// Print the shield activation banner.
pub fn print_activation_banner() {
    log::info!("🐱 CAT SHIELD 🛡️");
    log::info!("════════════════════════════════════════");
    log::info!("Protecting your work from curious cats!");
}

/// Print the shield active status with exit instructions.
///
/// # Arguments
/// * `exit_key` - The configured exit key for display
/// * `timer_info` - Optional timer remaining info string
pub fn print_shield_active(exit_key: &ExitKey, timer_info: Option<&str>) {
    log::info!("═══════════════════════════════════════");
    log::info!("🛡️  CAT SHIELD IS NOW ACTIVE!");
    log::info!("═══════════════════════════════════════");
    log::info!("Exit: Hold X button (top-right) for 3 seconds");
    log::info!("      Or press {}", exit_key.display_name);
    if let Some(info) = timer_info {
        log::info!("      Or wait for timer ({})", info);
    }
}

/// Set up the auto-exit timer and create the timer display view.
///
/// This function:
/// 1. Initializes the auto-exit timer with the specified duration
/// 2. Creates and configures the timer display view
/// 3. Stores the view reference in global state for the timer callback
/// 4. Adds the view to the window's content view
/// 5. Transfers ownership to prevent deallocation
///
/// # Arguments
/// * `mtm` - MainThreadMarker proving we're on the main thread
/// * `window` - The shield window to add the timer display to
/// * `screen_frame` - The screen frame for positioning the timer display
/// * `duration_secs` - The timer duration in seconds
///
/// # Note
/// The timer view ownership is transferred via `std::mem::forget` and the raw pointer
/// is stored in `shield::TIMER_VIEW`. It will be reclaimed in `deactivate_shield()`.
#[cfg(target_os = "macos")]
pub fn setup_timer_display(
    mtm: MainThreadMarker,
    window: &NSWindow,
    screen_frame: CGRect,
    duration_secs: u64,
) {
    // Initialize the auto-exit timer
    init_auto_exit_timer(duration_secs);
    log::info!("✓ Auto-exit timer set: {}", format_duration(duration_secs));

    // Create timer display view
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

    let timer_view = TimerDisplayView::new(mtm, timer_display_frame);

    // Store view reference for timer callback
    shield::TIMER_VIEW.store(
        Retained::as_ptr(&timer_view) as *mut c_void,
        Ordering::Release,
    );

    // Add timer display to the window's content view
    if let Some(content_view) = window.contentView() {
        content_view.addSubview(&timer_view);
    }

    log::info!("✓ Timer display active");

    // SAFETY: Transfer ownership to prevent deallocation while shield is active.
    // The timer view is retained by the window's content view and the raw pointer
    // is stored in shield::TIMER_VIEW. It will be reclaimed in deactivate_shield().
    std::mem::forget(timer_view);
}

// Compile-time validation of theme constants using const assertions
const _: () = {
    assert!(theme::BG_RED >= 0.0 && theme::BG_RED <= 1.0);
    assert!(theme::BG_GREEN >= 0.0 && theme::BG_GREEN <= 1.0);
    assert!(theme::BG_BLUE >= 0.0 && theme::BG_BLUE <= 1.0);
    assert!(theme::BUTTON_LABEL_GAP > 0.0);
};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::platform::errors::PermissionError;

    /// Mock permission checker that always has permissions granted
    struct GrantedPermissionChecker;

    impl PermissionChecker for GrantedPermissionChecker {
        fn check_permissions(&self) -> bool {
            true
        }

        fn check_permissions_with_prompt(&self) -> bool {
            true
        }

        fn open_permissions_settings(&self) -> Result<(), PermissionError> {
            Ok(())
        }
    }

    /// Mock permission checker that tracks method calls
    struct TrackingPermissionChecker {
        check_count: std::cell::Cell<u32>,
        permissions_granted: bool,
    }

    impl TrackingPermissionChecker {
        fn new(granted: bool) -> Self {
            Self {
                check_count: std::cell::Cell::new(0),
                permissions_granted: granted,
            }
        }

        fn check_count(&self) -> u32 {
            self.check_count.get()
        }
    }

    impl PermissionChecker for TrackingPermissionChecker {
        fn check_permissions(&self) -> bool {
            self.check_count.set(self.check_count.get() + 1);
            self.permissions_granted
        }

        fn check_permissions_with_prompt(&self) -> bool {
            self.permissions_granted
        }

        fn open_permissions_settings(&self) -> Result<(), PermissionError> {
            Ok(())
        }
    }

    #[test]
    fn test_ensure_permissions_returns_immediately_when_granted() {
        let exit_key = ExitKey::default();
        let checker = GrantedPermissionChecker;

        // Should return immediately without blocking
        ensure_permissions(&checker, &exit_key);
        // If we get here, permissions were checked and granted
    }

    #[test]
    fn test_ensure_permissions_checks_permissions_first() {
        let exit_key = ExitKey::default();
        let checker = TrackingPermissionChecker::new(true);

        ensure_permissions(&checker, &exit_key);

        // Should have checked permissions at least once
        assert!(checker.check_count() >= 1);
    }

    #[test]
    fn test_print_activation_banner_does_not_panic() {
        // Just verify it doesn't panic
        print_activation_banner();
    }

    #[test]
    fn test_print_shield_active_without_timer() {
        let exit_key = ExitKey::default();
        // Should not panic
        print_shield_active(&exit_key, None);
    }

    #[test]
    fn test_print_shield_active_with_timer() {
        let exit_key = ExitKey::default();
        // Should not panic
        print_shield_active(&exit_key, Some("30m 0s"));
    }
}
