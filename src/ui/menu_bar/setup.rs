//! Menu bar setup for Cat Shield

use super::handlers::MenuActionHandler;
use crate::ui::state::{
    ABOUT_ACTION_HANDLER, ABOUT_MENU_ITEM, MENU_ACTION_HANDLER, SETTINGS_ACTION_HANDLER,
    SETTINGS_MENU_ITEM, START_MENU_ITEM,
};
use crate::ui::windows::{AboutActionHandler, SettingsActionHandler};
use objc2::rc::Retained;
use objc2_app_kit::{NSMenu, NSMenuItem, NSStatusBar, NSStatusItem};
use objc2_foundation::{ns_string, MainThreadMarker};
use std::ffi::c_void;
use std::sync::atomic::Ordering;

/// Set up the menu bar status item with cat emoji icon
///
/// Creates an NSStatusItem in the system menu bar with:
/// - Cat emoji (🐱) as the icon
/// - "Cat Shield" tooltip on hover
/// - Comprehensive dropdown menu with all application features
///
/// Menu Structure:
/// - Header: "🐱 Cat Shield" (branding)
/// - Protection: Start/Stop Protection (for Issue #17)
/// - Configuration: Settings (for Issue #16)
/// - Information: About and Help (About for Issue #19)
/// - Exit: Quit with Cmd+Q
///
/// Returns the Retained<NSStatusItem> which must be kept alive for the duration
/// of the app to prevent the status item from being deallocated.
pub fn setup_menu_bar(mtm: MainThreadMarker) -> Retained<NSStatusItem> {
    // Get the system status bar
    let status_bar = NSStatusBar::systemStatusBar();

    // Create a status item with variable length (adjusts to content)
    // NSVariableStatusItemLength = -1.0
    let status_item = status_bar.statusItemWithLength(-1.0);

    // Configure the button (the clickable part of the status item)
    if let Some(button) = status_item.button(mtm) {
        // Set the cat emoji as the title
        button.setTitle(ns_string!("🐱"));

        // Set tooltip for accessibility
        button.setToolTip(Some(ns_string!(
            "Cat Shield - Protect your work from curious cats"
        )));
    }

    // Create the main dropdown menu
    let menu = NSMenu::new(mtm);

    // ============================================
    // HEADER SECTION
    // ============================================

    // Add "Cat Shield" title (disabled, just for branding)
    let title_item = NSMenuItem::new(mtm);
    title_item.setTitle(ns_string!("🐱 Cat Shield"));
    title_item.setEnabled(false);
    menu.addItem(&title_item);

    menu.addItem(&NSMenuItem::separatorItem(mtm));

    // ============================================
    // PROTECTION SECTION
    // ============================================

    // Create menu action handler and wire it to the Start Protection item
    let action_handler = MenuActionHandler::new(mtm);

    // Store the handler globally to keep it alive
    MENU_ACTION_HANDLER.store(
        Retained::as_ptr(&action_handler) as *mut c_void,
        Ordering::SeqCst,
    );

    // Add "Start Protection" item - activates the shield overlay on-demand
    let start_item = NSMenuItem::new(mtm);
    start_item.setTitle(ns_string!("Start Protection"));
    start_item.setToolTip(Some(ns_string!("Activate cat shield overlay")));

    // Set the target and action for the menu item
    unsafe {
        start_item.setTarget(Some(&action_handler));
        start_item.setAction(Some(objc2::sel!(startProtection:)));
    }

    // Store the start menu item reference for enabling/disabling
    START_MENU_ITEM.store(
        Retained::as_ptr(&start_item) as *mut c_void,
        Ordering::SeqCst,
    );

    menu.addItem(&start_item);

    // Keep handler alive
    std::mem::forget(action_handler);

    // Add "Stop Protection" item - deactivates the shield overlay when active
    // Initially hidden, will be shown when protection is active
    let stop_item = NSMenuItem::new(mtm);
    stop_item.setTitle(ns_string!("Stop Protection"));
    stop_item.setToolTip(Some(ns_string!("Deactivate cat shield overlay")));
    stop_item.setEnabled(false); // Will be enabled when shield is active
    stop_item.setHidden(true); // Hidden until protection is active
    menu.addItem(&stop_item);

    menu.addItem(&NSMenuItem::separatorItem(mtm));

    // ============================================
    // CONFIGURATION SECTION
    // ============================================

    // Add "Settings..." item - opens settings window for configuring preferences
    let settings_item = NSMenuItem::new(mtm);
    settings_item.setTitle(ns_string!("Settings..."));
    settings_item.setToolTip(Some(ns_string!(
        "Configure exit key, timer, and overlay opacity"
    )));
    settings_item.setKeyEquivalent(ns_string!(",")); // Standard Cmd+, for settings

    // Create settings action handler and wire it to the Settings item
    let settings_handler = SettingsActionHandler::new(mtm);

    // Set the target and action for the settings menu item
    unsafe {
        settings_item.setTarget(Some(&settings_handler));
        settings_item.setAction(Some(objc2::sel!(showSettings:)));
    }

    // Store the settings menu item reference for enabling/disabling
    SETTINGS_MENU_ITEM.store(
        Retained::as_ptr(&settings_item) as *mut c_void,
        Ordering::SeqCst,
    );

    // Store handler globally to keep it alive
    SETTINGS_ACTION_HANDLER.store(
        Retained::as_ptr(&settings_handler) as *mut c_void,
        Ordering::SeqCst,
    );

    menu.addItem(&settings_item);

    // Keep handler alive
    std::mem::forget(settings_handler);

    menu.addItem(&NSMenuItem::separatorItem(mtm));

    // ============================================
    // INFORMATION SECTION
    // ============================================

    // Create about action handler and wire it to the About item
    let about_handler = AboutActionHandler::new(mtm);

    // Add "About Cat Shield" item
    // Shows version, credits, and app information
    let about_item = NSMenuItem::new(mtm);
    about_item.setTitle(ns_string!("About Cat Shield"));
    about_item.setToolTip(Some(ns_string!("View application information and version")));

    // Set the target and action for the about menu item
    unsafe {
        about_item.setTarget(Some(&about_handler));
        about_item.setAction(Some(objc2::sel!(showAbout:)));
    }

    // Store the about menu item reference for enabling/disabling
    ABOUT_MENU_ITEM.store(
        Retained::as_ptr(&about_item) as *mut c_void,
        Ordering::SeqCst,
    );

    // Store handler globally to keep it alive
    ABOUT_ACTION_HANDLER.store(
        Retained::as_ptr(&about_handler) as *mut c_void,
        Ordering::SeqCst,
    );

    menu.addItem(&about_item);

    // Keep handler and menu item alive
    std::mem::forget(about_handler);
    std::mem::forget(about_item);

    // Add "Help" submenu
    // Contains links to documentation, GitHub, and support resources
    let help_item = NSMenuItem::new(mtm);
    help_item.setTitle(ns_string!("Help"));

    // Create Help submenu
    let help_submenu = NSMenu::new(mtm);

    // Help -> View Documentation
    let docs_item = NSMenuItem::new(mtm);
    docs_item.setTitle(ns_string!("View Documentation"));
    docs_item.setToolTip(Some(ns_string!("Open README on GitHub")));
    docs_item.setEnabled(false); // Will need custom action handler to open URL
    help_submenu.addItem(&docs_item);

    // Help -> Report Issue
    let issue_item = NSMenuItem::new(mtm);
    issue_item.setTitle(ns_string!("Report Issue"));
    issue_item.setToolTip(Some(ns_string!("Report a bug on GitHub")));
    issue_item.setEnabled(false); // Will need custom action handler to open URL
    help_submenu.addItem(&issue_item);

    // Help -> Release Notes
    let release_item = NSMenuItem::new(mtm);
    release_item.setTitle(ns_string!("Release Notes"));
    release_item.setToolTip(Some(ns_string!("View ROADMAP and release notes")));
    release_item.setEnabled(false); // Will need custom action handler to open URL
    help_submenu.addItem(&release_item);

    help_item.setSubmenu(Some(&help_submenu));
    menu.addItem(&help_item);

    menu.addItem(&NSMenuItem::separatorItem(mtm));

    // ============================================
    // EXIT SECTION
    // ============================================

    // Add "Quit Cat Shield" item
    // Note: This uses the standard terminate: action which NSApplication handles
    let quit_item = NSMenuItem::new(mtm);
    quit_item.setTitle(ns_string!("Quit Cat Shield"));
    quit_item.setToolTip(Some(ns_string!("Quit the application")));
    unsafe {
        quit_item.setAction(Some(objc2::sel!(terminate:)));
    }
    // Set keyboard shortcut Cmd+Q
    quit_item.setKeyEquivalent(ns_string!("q"));
    menu.addItem(&quit_item);

    // Attach menu to status item
    status_item.setMenu(Some(&menu));

    println!("  ✓ Menu bar icon active (🐱) with comprehensive dropdown menu");

    status_item
}
