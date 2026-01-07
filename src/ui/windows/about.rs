//! About window for Cat Shield

use crate::ui::helpers::create_label;
use crate::ui::state::{ABOUT_MENU_ITEM, ABOUT_WINDOW, ABOUT_WINDOW_DELEGATE};
use objc2::rc::Retained;
use objc2::runtime::ProtocolObject;
use objc2::{define_class, msg_send, MainThreadOnly};
use objc2_app_kit::{
    NSApplication, NSBackingStoreType, NSButton, NSButtonType, NSColor, NSFont, NSMenuItem,
    NSPanel, NSScreen, NSTextAlignment, NSTextField, NSWindowDelegate, NSWindowStyleMask,
};
use objc2_core_foundation::{CGFloat, CGPoint, CGRect, CGSize};
use objc2_foundation::{ns_string, MainThreadMarker, NSNotification, NSObject, NSObjectProtocol};
use std::ffi::c_void;
use std::sync::atomic::Ordering;

/// Empty ivars for the AboutWindowDelegate
pub struct AboutWindowDelegateIvars {}

/// Empty ivars for the AboutActionHandler
pub struct AboutActionHandlerIvars {}

// Window delegate to handle window close events
define_class!(
    #[unsafe(super(NSObject))]
    #[thread_kind = MainThreadOnly]
    #[name = "AboutWindowDelegate"]
    #[ivars = AboutWindowDelegateIvars]
    pub struct AboutWindowDelegate;

    unsafe impl NSObjectProtocol for AboutWindowDelegate {}

    unsafe impl NSWindowDelegate for AboutWindowDelegate {
        /// Called when the window is about to close (including via X button)
        #[unsafe(method(windowWillClose:))]
        fn window_will_close(&self, _notification: &NSNotification) {
            cleanup_about_window_references();
        }
    }
);

impl AboutWindowDelegate {
    pub fn new(mtm: MainThreadMarker) -> Retained<Self> {
        let this = mtm.alloc::<AboutWindowDelegate>();
        let this = this.set_ivars(AboutWindowDelegateIvars {});
        unsafe { msg_send![super(this), init] }
    }
}

// About action handler for menu items and buttons
define_class!(
    #[unsafe(super(NSObject))]
    #[name = "AboutActionHandler"]
    #[ivars = AboutActionHandlerIvars]
    pub struct AboutActionHandler;

    impl AboutActionHandler {
        /// Action method called when "About Cat Shield" menu item is clicked
        #[unsafe(method(showAbout:))]
        unsafe fn show_about(&self, _sender: Option<&NSMenuItem>) {
            if let Some(mtm) = MainThreadMarker::new() {
                show_about_window(mtm);
            }
        }

        /// Action method called when "Close" button is clicked
        #[unsafe(method(closeAbout:))]
        unsafe fn close_about(&self, _sender: Option<&NSButton>) {
            close_about_window();
        }
    }
);

impl AboutActionHandler {
    pub fn new(mtm: MainThreadMarker) -> Retained<Self> {
        let this = mtm.alloc::<AboutActionHandler>();
        let this = this.set_ivars(AboutActionHandlerIvars {});
        unsafe { msg_send![super(this), init] }
    }
}

/// Clean up about window UI element references
/// Called both when window is closed programmatically and via X button
fn cleanup_about_window_references() {
    // Clear window reference (swap to null to avoid double-cleanup)
    ABOUT_WINDOW.store(std::ptr::null_mut(), Ordering::SeqCst);

    // Re-enable the about menu item
    let menu_item_ptr = ABOUT_MENU_ITEM.load(Ordering::SeqCst);
    if !menu_item_ptr.is_null() {
        unsafe {
            let menu_item: &NSMenuItem = &*(menu_item_ptr as *const NSMenuItem);
            menu_item.setEnabled(true);
        }
    }

    println!("  About window closed");
}

/// Close the about window (called by Close button)
fn close_about_window() {
    let window_ptr = ABOUT_WINDOW.load(Ordering::SeqCst);
    if !window_ptr.is_null() {
        unsafe {
            let window: &NSPanel = &*(window_ptr as *const NSPanel);
            window.close();
        }
        // Note: cleanup_about_window_references() will be called by the window delegate
    }
}

/// Show the about window
pub fn show_about_window(mtm: MainThreadMarker) {
    // Check if about window is already open
    let existing = ABOUT_WINDOW.load(Ordering::SeqCst);
    if !existing.is_null() {
        // Bring existing window to front
        unsafe {
            let window: &NSPanel = &*(existing as *const NSPanel);
            window.makeKeyAndOrderFront(None);
        }
        return;
    }

    // Disable the about menu item while window is open
    let menu_item_ptr = ABOUT_MENU_ITEM.load(Ordering::SeqCst);
    if !menu_item_ptr.is_null() {
        unsafe {
            let menu_item: &NSMenuItem = &*(menu_item_ptr as *const NSMenuItem);
            menu_item.setEnabled(false);
        }
    }

    // Window dimensions
    let window_width: CGFloat = 300.0;
    let window_height: CGFloat = 260.0;

    // Calculate center position on screen
    let screen_frame = NSScreen::mainScreen(mtm)
        .map(|s| s.frame())
        .unwrap_or(CGRect {
            origin: CGPoint { x: 0.0, y: 0.0 },
            size: CGSize {
                width: 1920.0,
                height: 1080.0,
            },
        });

    let window_x = (screen_frame.size.width - window_width) / 2.0;
    let window_y = (screen_frame.size.height - window_height) / 2.0;

    let window_frame = CGRect {
        origin: CGPoint {
            x: window_x,
            y: window_y,
        },
        size: CGSize {
            width: window_width,
            height: window_height,
        },
    };

    // Create the about panel (utility window style)
    let panel = {
        let panel = NSPanel::alloc(mtm);
        NSPanel::initWithContentRect_styleMask_backing_defer(
            panel,
            window_frame,
            NSWindowStyleMask::Titled | NSWindowStyleMask::Closable,
            NSBackingStoreType::Buffered,
            false,
        )
    };

    panel.setTitle(ns_string!("About Cat Shield"));
    panel.setLevel(3); // NSFloatingWindowLevel
    unsafe {
        panel.setReleasedWhenClosed(false);
    }

    // Create and set window delegate to handle close button (X) cleanup
    let delegate = unsafe {
        let delegate_ptr = ABOUT_WINDOW_DELEGATE.load(Ordering::SeqCst);
        if delegate_ptr.is_null() {
            let new_delegate = AboutWindowDelegate::new(mtm);
            ABOUT_WINDOW_DELEGATE.store(
                Retained::as_ptr(&new_delegate) as *mut c_void,
                Ordering::SeqCst,
            );
            std::mem::forget(new_delegate);
            &*(ABOUT_WINDOW_DELEGATE.load(Ordering::SeqCst) as *const AboutWindowDelegate)
        } else {
            &*(delegate_ptr as *const AboutWindowDelegate)
        }
    };
    panel.setDelegate(Some(ProtocolObject::from_ref(delegate)));

    // Store window reference
    ABOUT_WINDOW.store(Retained::as_ptr(&panel) as *mut c_void, Ordering::SeqCst);

    // Get or create about action handler
    let handler = unsafe {
        let handler_ptr = crate::ui::state::ABOUT_ACTION_HANDLER.load(Ordering::SeqCst);
        if handler_ptr.is_null() {
            let new_handler = AboutActionHandler::new(mtm);
            crate::ui::state::ABOUT_ACTION_HANDLER.store(
                Retained::as_ptr(&new_handler) as *mut c_void,
                Ordering::SeqCst,
            );
            std::mem::forget(new_handler);
            &*(crate::ui::state::ABOUT_ACTION_HANDLER.load(Ordering::SeqCst)
                as *const AboutActionHandler)
        } else {
            &*(handler_ptr as *const AboutActionHandler)
        }
    };

    // Get version from Cargo.toml at compile time
    let version = env!("CARGO_PKG_VERSION");

    // Create content view with about info
    if let Some(content_view) = panel.contentView() {
        let margin: CGFloat = 20.0;

        // ========================================
        // Cat Emoji (large)
        // ========================================
        let emoji_size: CGFloat = 60.0;
        let emoji_y = window_height - margin - emoji_size - 10.0;
        let emoji_label = NSTextField::new(mtm);
        emoji_label.setFrame(CGRect {
            origin: CGPoint {
                x: (window_width - emoji_size) / 2.0,
                y: emoji_y,
            },
            size: CGSize {
                width: emoji_size,
                height: emoji_size,
            },
        });
        emoji_label.setEditable(false);
        emoji_label.setSelectable(false);
        emoji_label.setBordered(false);
        emoji_label.setDrawsBackground(false);
        emoji_label.setAlignment(NSTextAlignment::Center);
        emoji_label.setStringValue(ns_string!("\u{1F431}")); // 🐱
        emoji_label.setFont(Some(&NSFont::systemFontOfSize(48.0)));
        content_view.addSubview(&emoji_label);
        std::mem::forget(emoji_label);

        // ========================================
        // App Name
        // ========================================
        let name_height: CGFloat = 28.0;
        let name_y = emoji_y - name_height - 5.0;
        let name_label = NSTextField::new(mtm);
        name_label.setFrame(CGRect {
            origin: CGPoint {
                x: margin,
                y: name_y,
            },
            size: CGSize {
                width: window_width - (margin * 2.0),
                height: name_height,
            },
        });
        name_label.setEditable(false);
        name_label.setSelectable(false);
        name_label.setBordered(false);
        name_label.setDrawsBackground(false);
        name_label.setAlignment(NSTextAlignment::Center);
        name_label.setStringValue(ns_string!("Cat Shield"));
        name_label.setFont(Some(&NSFont::boldSystemFontOfSize(20.0)));
        name_label.setTextColor(Some(&NSColor::labelColor()));
        content_view.addSubview(&name_label);
        std::mem::forget(name_label);

        // ========================================
        // Version
        // ========================================
        let version_height: CGFloat = 20.0;
        let version_y = name_y - version_height - 2.0;
        let version_label = NSTextField::new(mtm);
        version_label.setFrame(CGRect {
            origin: CGPoint {
                x: margin,
                y: version_y,
            },
            size: CGSize {
                width: window_width - (margin * 2.0),
                height: version_height,
            },
        });
        version_label.setEditable(false);
        version_label.setSelectable(false);
        version_label.setBordered(false);
        version_label.setDrawsBackground(false);
        version_label.setAlignment(NSTextAlignment::Center);
        version_label.setStringValue(&objc2_foundation::NSString::from_str(&format!(
            "Version {}",
            version
        )));
        version_label.setFont(Some(&NSFont::systemFontOfSize(12.0)));
        version_label.setTextColor(Some(&NSColor::secondaryLabelColor()));
        content_view.addSubview(&version_label);
        std::mem::forget(version_label);

        // ========================================
        // Description
        // ========================================
        let description_height: CGFloat = 40.0;
        let description_y = version_y - description_height - 15.0;
        let description_label = create_label(
            mtm,
            "Protect your work from curious\ncats and keyboard-walking pets.",
            CGRect {
                origin: CGPoint {
                    x: margin,
                    y: description_y,
                },
                size: CGSize {
                    width: window_width - (margin * 2.0),
                    height: description_height,
                },
            },
            13.0,
            &NSColor::labelColor(),
            false,
        );
        description_label.setAlignment(NSTextAlignment::Center);
        content_view.addSubview(&description_label);

        // ========================================
        // Close Button
        // ========================================
        let button_height: CGFloat = 28.0;
        let button_width: CGFloat = 80.0;
        let button_y: CGFloat = margin;

        let close_button = unsafe {
            let button = NSButton::buttonWithTitle_target_action(
                ns_string!("Close"),
                Some(handler),
                Some(objc2::sel!(closeAbout:)),
                mtm,
            );
            button.setFrame(CGRect {
                origin: CGPoint {
                    x: (window_width - button_width) / 2.0,
                    y: button_y,
                },
                size: CGSize {
                    width: button_width,
                    height: button_height,
                },
            });
            button.setButtonType(NSButtonType::MomentaryPushIn);
            button.setKeyEquivalent(ns_string!("\r")); // Return key to close
            button
        };
        content_view.addSubview(&close_button);
        std::mem::forget(close_button);
    }

    // Activate the application so the window can receive focus
    // This is needed because the app runs in accessory mode (no dock icon)
    let app = NSApplication::sharedApplication(mtm);
    #[allow(deprecated)]
    app.activateIgnoringOtherApps(true);

    // Show the window
    panel.makeKeyAndOrderFront(None);

    // Keep panel alive
    std::mem::forget(panel);

    println!("  About window opened");
}
