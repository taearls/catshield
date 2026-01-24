//! Settings window UI using iced
//!
//! This module will provide the settings window for configuring Cat Shield.
//! Currently a placeholder for future implementation (Issue #157).
//!
//! Planned features:
//! - Exit key configuration
//! - Timer presets
//! - Overlay opacity slider
//! - Launch at login toggle
//! - Allowed keys list management

use iced::widget::{column, container, text};
use iced::{Element, Length, Theme};

use crate::ui_iced::theme::{colors, CatShieldTheme};

/// Messages for the settings window
#[derive(Debug, Clone)]
pub enum SettingsMessage {
    /// Placeholder - no messages yet
    NoOp,
}

/// State for the settings window
#[derive(Default)]
pub struct SettingsWindow {
    // Settings state will be added in Issue #157
}

impl SettingsWindow {
    /// Create a new settings window
    pub fn new() -> Self {
        Self::default()
    }

    /// Update settings state
    pub fn update(&mut self, message: SettingsMessage) {
        match message {
            SettingsMessage::NoOp => {}
        }
    }

    /// Render the settings view
    pub fn view(&self) -> Element<'_, SettingsMessage> {
        let content = column![
            text("Cat Shield Settings")
                .size(24)
                .color(colors::TEXT_PRIMARY),
            text("Settings UI coming in Issue #157")
                .size(14)
                .color(colors::TEXT_SECONDARY),
        ]
        .spacing(20)
        .padding(30);

        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .style(CatShieldTheme::settings_container)
            .into()
    }

    /// Get the theme for the settings window
    pub fn theme(&self) -> Theme {
        CatShieldTheme::base()
    }
}
