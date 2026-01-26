//! Standalone settings window binary for Cat Shield
//!
//! This binary is launched as a subprocess by the main Cat Shield application
//! when the user clicks "Preferences..." in the menu bar. On macOS, iced/winit
//! requires the EventLoop to run on the main thread, so we can't spawn iced
//! windows in background threads. Instead, we launch them as separate processes.
//!
//! # Usage
//!
//! This binary is not intended to be run directly by users. It is invoked
//! automatically by Cat Shield when the settings window is requested.

use cat_shield::ui_iced::SettingsWindow;

fn main() {
    // Run the iced settings application on the main thread
    if let Err(e) = SettingsWindow::run() {
        eprintln!("Settings window error: {e}");
        std::process::exit(1);
    }
}
