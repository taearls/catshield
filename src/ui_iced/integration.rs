//! Platform integration for iced windows
//!
//! This module provides the bridge between platform-specific UI (menu bar, system tray)
//! and iced-based windows (Settings, About). It handles:
//!
//! - Launching iced windows from menu/tray actions
//! - Window lifecycle management (open/close)
//! - Coordination between platform UI and iced event loops
//!
//! # Architecture
//!
//! Platform-specific menus remain in native code because:
//! - macOS: NSStatusItem integrates naturally with the system menu bar
//! - Windows: Win32 Shell_NotifyIcon for system tray integration
//! - They're already working and well-tested
//! - iced doesn't provide native menu bar/tray support
//!
//! When a menu item is clicked (e.g., "Settings" or "About"), this module spawns
//! the corresponding iced window in a new thread. The iced application runs
//! its own event loop until the window is closed.
//!
//! # Thread Safety
//!
//! Window state is tracked using atomic booleans to prevent multiple instances
//! of the same window from being opened simultaneously.

use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;

/// Whether the iced settings window is currently open
static SETTINGS_WINDOW_OPEN: AtomicBool = AtomicBool::new(false);

/// Whether the iced about window is currently open
static ABOUT_WINDOW_OPEN: AtomicBool = AtomicBool::new(false);

/// Open the iced settings window
///
/// This function spawns a new thread to run the iced settings window.
/// If the settings window is already open, this function does nothing
/// and brings the existing window to front (when supported).
///
/// # Returns
///
/// `true` if the window was opened, `false` if it was already open.
pub fn open_settings_window() -> bool {
    // Check if already open
    if SETTINGS_WINDOW_OPEN.swap(true, Ordering::SeqCst) {
        log::debug!("Settings window already open");
        // TODO: Bring existing window to front
        return false;
    }

    log::info!("Opening iced settings window");

    // Spawn the iced settings window in a new thread
    thread::Builder::new()
        .name("iced-settings".to_string())
        .spawn(move || {
            // Run the iced settings application
            if let Err(e) = crate::ui_iced::SettingsWindow::run() {
                log::error!("Settings window error: {e}");
            }

            // Mark window as closed when iced exits
            SETTINGS_WINDOW_OPEN.store(false, Ordering::SeqCst);
            log::debug!("Settings window closed");
        })
        .expect("Failed to spawn settings window thread");

    true
}

/// Open the iced about window
///
/// This function spawns a new thread to run the iced about window.
/// If the about window is already open, this function does nothing.
///
/// # Returns
///
/// `true` if the window was opened, `false` if it was already open.
pub fn open_about_window() -> bool {
    // Check if already open
    if ABOUT_WINDOW_OPEN.swap(true, Ordering::SeqCst) {
        log::debug!("About window already open");
        return false;
    }

    log::info!("Opening iced about window");

    // Spawn the iced about window in a new thread
    thread::Builder::new()
        .name("iced-about".to_string())
        .spawn(move || {
            // Run the iced about application
            if let Err(e) = crate::ui_iced::about::AboutWindow::run() {
                log::error!("About window error: {e}");
            }

            // Mark window as closed when iced exits
            ABOUT_WINDOW_OPEN.store(false, Ordering::SeqCst);
            log::debug!("About window closed");
        })
        .expect("Failed to spawn about window thread");

    true
}

/// Check if the settings window is currently open
pub fn is_settings_window_open() -> bool {
    SETTINGS_WINDOW_OPEN.load(Ordering::SeqCst)
}

/// Check if the about window is currently open
pub fn is_about_window_open() -> bool {
    ABOUT_WINDOW_OPEN.load(Ordering::SeqCst)
}

/// Close all iced windows
///
/// This is called during application shutdown to ensure clean cleanup.
/// Note: This only resets the tracking state; the actual iced windows
/// will close when their event loops exit.
pub fn close_all_windows() {
    SETTINGS_WINDOW_OPEN.store(false, Ordering::SeqCst);
    ABOUT_WINDOW_OPEN.store(false, Ordering::SeqCst);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_window_state_initially_closed() {
        // Reset state for test
        SETTINGS_WINDOW_OPEN.store(false, Ordering::SeqCst);
        ABOUT_WINDOW_OPEN.store(false, Ordering::SeqCst);

        assert!(!is_settings_window_open());
        assert!(!is_about_window_open());
    }

    #[test]
    fn test_close_all_windows() {
        SETTINGS_WINDOW_OPEN.store(true, Ordering::SeqCst);
        ABOUT_WINDOW_OPEN.store(true, Ordering::SeqCst);

        close_all_windows();

        assert!(!is_settings_window_open());
        assert!(!is_about_window_open());
    }
}
