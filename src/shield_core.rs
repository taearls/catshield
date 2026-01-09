//! Shared shield activation logic for Cat Shield
//!
//! This module contains shared functions used by both immediate mode (main.rs)
//! and menu bar mode (ui/shield.rs) for:
//! - Accessibility permission checking and polling
//! - Shield window creation and configuration
//! - Close button setup
//!
//! By extracting this shared logic, we avoid duplication and ensure
//! consistent behavior between the two modes.

use crate::config::get_current_config;
use crate::input::ExitKey;
use crate::platform::{
    check_accessibility, check_accessibility_with_prompt, open_accessibility_settings,
    CFRunLoopRunInMode,
};
use crate::ui::state::{close_button, window_level};
use crate::ui::views::{CloseButtonLabelView, CloseButtonView};
use objc2::rc::Retained;
use objc2::MainThreadOnly;
use objc2_app_kit::{
    NSBackingStoreType, NSColor, NSWindow, NSWindowCollectionBehavior, NSWindowStyleMask,
};
use objc2_core_foundation::{kCFRunLoopDefaultMode, CFString, CGPoint, CGRect, CGSize};
use objc2_foundation::{ns_string, MainThreadMarker};

/// UI theme constants for the shield overlay
pub mod theme {
    /// Background color RGB values (dark theme)
    pub const BG_RED: f64 = 0.1;
    pub const BG_GREEN: f64 = 0.1;
    pub const BG_BLUE: f64 = 0.15;

    /// Gap between close button and its label
    pub const BUTTON_LABEL_GAP: f64 = 5.0;
}

/// Ensure accessibility permissions are granted, prompting the user if necessary.
///
/// This function blocks until accessibility permissions are granted:
/// 1. Checks if accessibility is already granted (returns immediately if so)
/// 2. If not, prompts the user with the system dialog
/// 3. If still not granted, opens System Settings and polls until granted
///
/// # Arguments
/// * `exit_key` - The configured exit key for display in messages
///
/// # Note
/// This function will loop indefinitely until permissions are granted.
/// It does not return until the user grants accessibility access.
pub fn ensure_accessibility(exit_key: &ExitKey) {
    if check_accessibility() {
        return;
    }

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
    if check_accessibility_with_prompt() {
        println!("  ✓ Permissions granted!");
        println!();
        return;
    }

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
            return;
        }
    }
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
pub fn create_shield_window(mtm: MainThreadMarker, frame: CGRect) -> Retained<NSWindow> {
    // Create a fullscreen, borderless window
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

    // Required when creating NSWindow outside a window controller
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
    println!();
    println!("  🐱 CAT SHIELD 🛡️");
    println!("  ════════════════════════════════════════");
    println!("  Protecting your work from curious cats!");
    println!();
}

/// Print the shield active status with exit instructions.
///
/// # Arguments
/// * `exit_key` - The configured exit key for display
/// * `timer_info` - Optional timer remaining info string
pub fn print_shield_active(exit_key: &ExitKey, timer_info: Option<&str>) {
    println!();
    println!("  ═══════════════════════════════════════");
    println!("  🛡️  CAT SHIELD IS NOW ACTIVE!");
    println!("  ═══════════════════════════════════════");
    println!();
    println!("  Exit: Hold X button (top-right) for 3 seconds");
    println!("        Or press {}", exit_key.display_name);
    if let Some(info) = timer_info {
        println!("        Or wait for timer ({})", info);
    }
    println!();
}

// Compile-time validation of theme constants using const assertions
const _: () = {
    assert!(theme::BG_RED >= 0.0 && theme::BG_RED <= 1.0);
    assert!(theme::BG_GREEN >= 0.0 && theme::BG_GREEN <= 1.0);
    assert!(theme::BG_BLUE >= 0.0 && theme::BG_BLUE <= 1.0);
    assert!(theme::BUTTON_LABEL_GAP > 0.0);
};
