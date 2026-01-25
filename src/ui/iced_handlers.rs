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
//!
//! # Menu Item Re-enabling
//!
//! When a window is opened, its corresponding menu item is disabled to prevent
//! multiple instances. When the window closes, the menu item is re-enabled
//! using `dispatch_async` to safely execute on the main thread.

use crate::ui::ptr_helper::with_ptr_void;
use crate::ui::state::menu_bar;
use crate::ui_iced::integration;
use objc2::rc::Retained;
use objc2::{define_class, msg_send};
use objc2_app_kit::NSMenuItem;
use objc2_foundation::{MainThreadMarker, NSObject};

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

                    // Re-enable menu item on main thread using dispatch_async
                    log::debug!("Settings window closed, re-enabling menu item");
                    dispatch_to_main_thread(reenable_settings_menu_item);
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

                    // Re-enable menu item on main thread using dispatch_async
                    log::debug!("About window closed, re-enabling menu item");
                    dispatch_to_main_thread(reenable_about_menu_item);
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

/// Dispatch a closure to run on the main thread
///
/// This uses Grand Central Dispatch to safely execute AppKit operations
/// from background threads.
fn dispatch_to_main_thread<F>(f: F)
where
    F: FnOnce() + Send + 'static,
{
    dispatch2::DispatchQueue::main().exec_async(f);
}

/// Re-enable the settings menu item (called on main thread via dispatch)
fn reenable_settings_menu_item() {
    unsafe {
        with_ptr_void::<NSMenuItem, _>(&menu_bar::SETTINGS_ITEM, |menu_item| {
            menu_item.setEnabled(true);
        });
    }
    log::debug!("Re-enabled settings menu item");
}

/// Re-enable the about menu item (called on main thread via dispatch)
fn reenable_about_menu_item() {
    unsafe {
        with_ptr_void::<NSMenuItem, _>(&menu_bar::ABOUT_ITEM, |menu_item| {
            menu_item.setEnabled(true);
        });
    }
    log::debug!("Re-enabled about menu item");
}
