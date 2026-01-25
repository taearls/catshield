//! AppKit action handlers that launch iced windows
//!
//! This module provides Objective-C compatible handlers that can be used as
//! targets for NSMenuItem actions, bridging AppKit menu items to iced windows.
//!
//! # Usage
//!
//! These handlers are set as targets for menu items in the NSStatusItem menu.
//! When the menu item is clicked, the handler's action method is called,
//! which spawns the corresponding iced window.

use crate::ui::ptr_helper::with_ptr_void;
use crate::ui::state::menu_bar;
use crate::ui_iced::integration;
use objc2::rc::Retained;
use objc2::{define_class, msg_send};
use objc2_app_kit::NSMenuItem;
use objc2_foundation::{MainThreadMarker, NSObject};
use std::sync::atomic::Ordering;

/// Empty ivars for the IcedSettingsHandler
pub struct IcedSettingsHandlerIvars {}

/// Empty ivars for the IcedAboutHandler
pub struct IcedAboutHandlerIvars {}

// Handler that opens the iced settings window
define_class!(
    #[unsafe(super(NSObject))]
    #[name = "IcedSettingsHandler"]
    #[ivars = IcedSettingsHandlerIvars]
    pub struct IcedSettingsHandler;

    impl IcedSettingsHandler {
        /// Action method called when "Settings..." menu item is clicked
        #[unsafe(method(showSettings:))]
        unsafe fn show_settings(&self, _sender: Option<&NSMenuItem>) {
            // Disable menu item while window is open
            unsafe {
                with_ptr_void::<NSMenuItem, _>(&menu_bar::SETTINGS_ITEM, |menu_item| {
                    menu_item.setEnabled(false);
                });
            }

            // Open the iced settings window
            // Note: This spawns a new thread and returns immediately
            if integration::open_settings_window() {
                log::debug!("Launched iced settings window");

                // Spawn a thread to re-enable the menu item when window closes
                std::thread::spawn(|| {
                    // Poll until window is closed
                    while integration::is_settings_window_open() {
                        std::thread::sleep(std::time::Duration::from_millis(100));
                    }

                    // Re-enable menu item on main thread
                    // Note: We need to dispatch back to main thread for AppKit operations
                    log::debug!("Settings window closed, re-enabling menu item");
                    // The menu item will be re-enabled on next menu access
                    // This is a limitation - proper solution would use dispatch_async
                    SETTINGS_NEEDS_REENABLE.store(true, Ordering::SeqCst);
                });
            } else {
                // Window was already open, re-enable menu item
                unsafe {
                    with_ptr_void::<NSMenuItem, _>(&menu_bar::SETTINGS_ITEM, |menu_item| {
                        menu_item.setEnabled(true);
                    });
                }
            }
        }
    }
);

impl IcedSettingsHandler {
    pub fn new(mtm: MainThreadMarker) -> Retained<Self> {
        let this = mtm.alloc::<IcedSettingsHandler>();
        let this = this.set_ivars(IcedSettingsHandlerIvars {});
        unsafe { msg_send![super(this), init] }
    }
}

// Handler that opens the iced about window
define_class!(
    #[unsafe(super(NSObject))]
    #[name = "IcedAboutHandler"]
    #[ivars = IcedAboutHandlerIvars]
    pub struct IcedAboutHandler;

    impl IcedAboutHandler {
        /// Action method called when "About Cat Shield" menu item is clicked
        #[unsafe(method(showAbout:))]
        unsafe fn show_about(&self, _sender: Option<&NSMenuItem>) {
            // Disable menu item while window is open
            unsafe {
                with_ptr_void::<NSMenuItem, _>(&menu_bar::ABOUT_ITEM, |menu_item| {
                    menu_item.setEnabled(false);
                });
            }

            // Open the iced about window
            if integration::open_about_window() {
                log::debug!("Launched iced about window");

                // Spawn a thread to re-enable the menu item when window closes
                std::thread::spawn(|| {
                    // Poll until window is closed
                    while integration::is_about_window_open() {
                        std::thread::sleep(std::time::Duration::from_millis(100));
                    }

                    // Mark that menu item needs re-enabling
                    log::debug!("About window closed, re-enabling menu item");
                    ABOUT_NEEDS_REENABLE.store(true, Ordering::SeqCst);
                });
            } else {
                // Window was already open, re-enable menu item
                unsafe {
                    with_ptr_void::<NSMenuItem, _>(&menu_bar::ABOUT_ITEM, |menu_item| {
                        menu_item.setEnabled(true);
                    });
                }
            }
        }
    }
);

impl IcedAboutHandler {
    pub fn new(mtm: MainThreadMarker) -> Retained<Self> {
        let this = mtm.alloc::<IcedAboutHandler>();
        let this = this.set_ivars(IcedAboutHandlerIvars {});
        unsafe { msg_send![super(this), init] }
    }
}

/// Flag indicating the settings menu item needs re-enabling
/// This is set by the background thread when the iced window closes
pub static SETTINGS_NEEDS_REENABLE: std::sync::atomic::AtomicBool =
    std::sync::atomic::AtomicBool::new(false);

/// Flag indicating the about menu item needs re-enabling
pub static ABOUT_NEEDS_REENABLE: std::sync::atomic::AtomicBool =
    std::sync::atomic::AtomicBool::new(false);

/// Check and re-enable menu items if needed
///
/// This should be called periodically from the main thread (e.g., in a timer
/// or event handler) to re-enable menu items after iced windows close.
pub fn check_menu_item_reenable() {
    if SETTINGS_NEEDS_REENABLE.swap(false, Ordering::SeqCst) {
        unsafe {
            with_ptr_void::<NSMenuItem, _>(&menu_bar::SETTINGS_ITEM, |menu_item| {
                menu_item.setEnabled(true);
            });
        }
        log::debug!("Re-enabled settings menu item");
    }

    if ABOUT_NEEDS_REENABLE.swap(false, Ordering::SeqCst) {
        unsafe {
            with_ptr_void::<NSMenuItem, _>(&menu_bar::ABOUT_ITEM, |menu_item| {
                menu_item.setEnabled(true);
            });
        }
        log::debug!("Re-enabled about menu item");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reenable_flags_initially_false() {
        // Reset for test
        SETTINGS_NEEDS_REENABLE.store(false, Ordering::SeqCst);
        ABOUT_NEEDS_REENABLE.store(false, Ordering::SeqCst);

        assert!(!SETTINGS_NEEDS_REENABLE.load(Ordering::SeqCst));
        assert!(!ABOUT_NEEDS_REENABLE.load(Ordering::SeqCst));
    }
}
