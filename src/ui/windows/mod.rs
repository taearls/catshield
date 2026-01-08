//! Window creation for Cat Shield

mod about;
mod settings;

pub use about::{show_about_window, AboutActionHandler, AboutWindowDelegate};
pub use settings::{show_settings_window, SettingsActionHandler, SettingsWindowDelegate};
