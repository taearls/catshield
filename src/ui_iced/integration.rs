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
//! - Linux: ksni StatusNotifierItem for system tray integration
//! - They're already working and well-tested
//! - iced doesn't provide native menu bar/tray support
//!
//! # macOS Subprocess Approach
//!
//! On macOS, iced (which uses winit for windowing) requires the EventLoop to be
//! created on the main thread. Since the main thread is already running the
//! NSApplication event loop, we cannot spawn iced windows in background threads.
//!
//! Instead, we launch settings and about windows as separate subprocess binaries.
//! Each subprocess has its own main thread where iced can create its EventLoop.
//! This approach is transparent to the user - windows appear normally when
//! clicking menu items.
//!
//! # Windows and Linux Thread Approach
//!
//! On Windows and Linux, iced/winit can create event loops on background threads,
//! so we continue to use the thread-based approach for simplicity.
//!
//! # Thread/Process Safety
//!
//! Window state is tracked using atomic booleans to prevent multiple instances
//! of the same window from being opened simultaneously.

use std::sync::atomic::{AtomicBool, Ordering};

/// Whether the iced settings window is currently open
static SETTINGS_WINDOW_OPEN: AtomicBool = AtomicBool::new(false);

/// Whether the iced about window is currently open
static ABOUT_WINDOW_OPEN: AtomicBool = AtomicBool::new(false);

/// Get the path to the helper binary for launching iced windows.
///
/// On macOS, iced windows must run in a separate process because winit
/// requires the EventLoop to be on the main thread.
///
/// This function finds the helper binary by looking in the same directory
/// as the main executable.
#[cfg(target_os = "macos")]
fn get_helper_binary_path(binary_name: &str) -> Option<std::path::PathBuf> {
    // Get the path to the current executable
    let current_exe = std::env::current_exe().ok()?;
    let exe_dir = current_exe.parent()?;

    // Look for the helper binary in the same directory
    let helper_path = exe_dir.join(binary_name);
    if helper_path.exists() {
        return Some(helper_path);
    }

    // Also check in common development locations (target/debug, target/release)
    // This handles the case where we're running from `cargo run`
    if let Some(target_dir) = exe_dir.parent() {
        // Check debug directory
        let debug_path = target_dir.join("debug").join(binary_name);
        if debug_path.exists() {
            return Some(debug_path);
        }

        // Check release directory
        let release_path = target_dir.join("release").join(binary_name);
        if release_path.exists() {
            return Some(release_path);
        }
    }

    None
}

/// Open the iced settings window
///
/// On macOS, this launches a subprocess because winit requires the EventLoop
/// to be on the main thread. On Windows and Linux, this spawns a new thread.
///
/// If the settings window is already open, this function does nothing.
///
/// # Returns
///
/// `true` if the window was opened, `false` if it was already open.
#[cfg(target_os = "macos")]
pub fn open_settings_window() -> bool {
    use std::process::Command;
    use std::thread;

    // Check if already open
    if SETTINGS_WINDOW_OPEN.swap(true, Ordering::SeqCst) {
        log::debug!("Settings window already open");
        return false;
    }

    log::info!("Opening iced settings window (subprocess)");

    // Find the settings_window binary
    let binary_path = match get_helper_binary_path("settings_window") {
        Some(path) => path,
        None => {
            log::error!("Could not find settings_window binary");
            SETTINGS_WINDOW_OPEN.store(false, Ordering::SeqCst);
            return false;
        }
    };

    // Spawn the subprocess in a monitoring thread
    thread::Builder::new()
        .name("settings-monitor".to_string())
        .spawn(move || {
            log::debug!("Launching settings window: {:?}", binary_path);

            // Spawn the subprocess and wait for it to exit
            match Command::new(&binary_path).spawn() {
                Ok(mut child) => {
                    // Wait for the subprocess to exit
                    match child.wait() {
                        Ok(status) => {
                            if !status.success() {
                                log::warn!("Settings window exited with status: {status}");
                            }
                        }
                        Err(e) => {
                            log::error!("Failed to wait for settings window: {e}");
                        }
                    }
                }
                Err(e) => {
                    log::error!("Failed to launch settings window subprocess: {e}");
                }
            }

            // Mark window as closed when subprocess exits
            SETTINGS_WINDOW_OPEN.store(false, Ordering::SeqCst);
            log::debug!("Settings window closed");
        })
        .expect("Failed to spawn settings monitor thread");

    true
}

/// Open the iced about window
///
/// On macOS, this launches a subprocess because winit requires the EventLoop
/// to be on the main thread. On Windows and Linux, this spawns a new thread.
///
/// If the about window is already open, this function does nothing.
///
/// # Returns
///
/// `true` if the window was opened, `false` if it was already open.
#[cfg(target_os = "macos")]
pub fn open_about_window() -> bool {
    use std::process::Command;
    use std::thread;

    // Check if already open
    if ABOUT_WINDOW_OPEN.swap(true, Ordering::SeqCst) {
        log::debug!("About window already open");
        return false;
    }

    log::info!("Opening iced about window (subprocess)");

    // Find the about_window binary
    let binary_path = match get_helper_binary_path("about_window") {
        Some(path) => path,
        None => {
            log::error!("Could not find about_window binary");
            ABOUT_WINDOW_OPEN.store(false, Ordering::SeqCst);
            return false;
        }
    };

    // Spawn the subprocess in a monitoring thread
    thread::Builder::new()
        .name("about-monitor".to_string())
        .spawn(move || {
            log::debug!("Launching about window: {:?}", binary_path);

            // Spawn the subprocess and wait for it to exit
            match Command::new(&binary_path).spawn() {
                Ok(mut child) => {
                    // Wait for the subprocess to exit
                    match child.wait() {
                        Ok(status) => {
                            if !status.success() {
                                log::warn!("About window exited with status: {status}");
                            }
                        }
                        Err(e) => {
                            log::error!("Failed to wait for about window: {e}");
                        }
                    }
                }
                Err(e) => {
                    log::error!("Failed to launch about window subprocess: {e}");
                }
            }

            // Mark window as closed when subprocess exits
            ABOUT_WINDOW_OPEN.store(false, Ordering::SeqCst);
            log::debug!("About window closed");
        })
        .expect("Failed to spawn about monitor thread");

    true
}

/// Open the iced settings window (Windows/Linux version)
///
/// This function spawns a new thread to run the iced settings window.
/// If the settings window is already open, this function does nothing
/// and brings the existing window to front (when supported).
///
/// # Returns
///
/// `true` if the window was opened, `false` if it was already open.
#[cfg(not(target_os = "macos"))]
pub fn open_settings_window() -> bool {
    use std::thread;

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

/// Open the iced about window (Windows/Linux version)
///
/// This function spawns a new thread to run the iced about window.
/// If the about window is already open, this function does nothing.
///
/// # Returns
///
/// `true` if the window was opened, `false` if it was already open.
#[cfg(not(target_os = "macos"))]
pub fn open_about_window() -> bool {
    use std::thread;

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

    #[cfg(target_os = "macos")]
    #[test]
    fn test_get_helper_binary_path_returns_none_for_missing_binary() {
        // Test with a binary that definitely doesn't exist
        let path = get_helper_binary_path("nonexistent_binary_12345");
        assert!(path.is_none());
    }
}
