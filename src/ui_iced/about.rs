//! About window UI using iced
//!
//! This module provides the About window for Cat Shield.
//! It displays version information, credits, and links.

use iced::widget::{button, column, container, row, text, Space};
use iced::window::{Position, Settings as WindowSettings};
use iced::{Alignment, Element, Length, Padding, Size, Task, Theme};

use crate::ui_iced::theme::{colors, CatShieldTheme};

/// Messages for the about window
#[derive(Debug, Clone)]
pub enum AboutMessage {
    /// Close the about window
    Close,
    /// Window close requested (via title bar X button)
    CloseRequested,
}

/// State for the about window
pub struct AboutWindow {
    /// Application version
    version: String,
}

impl Default for AboutWindow {
    fn default() -> Self {
        Self::new()
    }
}

impl AboutWindow {
    /// Create a new about window
    pub fn new() -> Self {
        Self {
            version: env!("CARGO_PKG_VERSION").to_string(),
        }
    }

    /// Update about window state
    pub fn update(&mut self, message: AboutMessage) -> Task<AboutMessage> {
        match message {
            AboutMessage::Close | AboutMessage::CloseRequested => iced::exit(),
        }
    }

    /// Render the about view
    pub fn view(&self) -> Element<'_, AboutMessage> {
        let content = column![
            // Cat emoji
            container(text("🐱").size(64))
                .width(Length::Fill)
                .padding(Padding::from([30.0, 0.0]).bottom(10.0))
                .align_x(iced::alignment::Horizontal::Center),
            // App name
            text("Cat Shield").size(28).color(colors::TEXT_PRIMARY),
            // Version
            text(format!("Version {}", self.version))
                .size(14)
                .color(colors::TEXT_SECONDARY),
            // Spacer
            Space::new().height(Length::Fixed(20.0)),
            // Description
            container(
                text(
                    "A cross-platform application that creates a\n\
                     cat-proof screen overlay to keep your machine\n\
                     awake and protect your work from curious cats."
                )
                .size(14)
                .color(colors::TEXT_SECONDARY)
            )
            .width(Length::Fill)
            .align_x(iced::alignment::Horizontal::Center),
            // Spacer
            Space::new().height(Length::Fixed(20.0)),
            // Credits section
            container(
                column![
                    text("Built with").size(12).color(colors::TEXT_MUTED),
                    text("Rust + iced framework")
                        .size(13)
                        .color(colors::TEXT_SECONDARY),
                ]
                .spacing(4)
                .align_x(Alignment::Center)
            )
            .width(Length::Fill)
            .align_x(iced::alignment::Horizontal::Center),
            // Spacer
            Space::new().height(Length::Fixed(10.0)),
            // Links
            container(
                column![
                    text("github.com/taearls/catshield")
                        .size(12)
                        .color(colors::ACCENT),
                    text("MIT License").size(11).color(colors::TEXT_MUTED),
                ]
                .spacing(4)
                .align_x(Alignment::Center)
            )
            .width(Length::Fill)
            .align_x(iced::alignment::Horizontal::Center),
            // Spacer to push button to bottom
            Space::new().height(Length::Fill),
            // Close button
            container(
                row![button(text("Close").size(14).color(colors::TEXT_PRIMARY))
                    .padding([10, 24])
                    .style(CatShieldTheme::primary_button)
                    .on_press(AboutMessage::Close),]
                .align_y(Alignment::Center)
            )
            .width(Length::Fill)
            .align_x(iced::alignment::Horizontal::Center)
            .padding(Padding::from([0.0, 0.0]).bottom(20.0)),
        ]
        .spacing(6)
        .align_x(Alignment::Center)
        .width(Length::Fill)
        .height(Length::Fill);

        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .style(CatShieldTheme::settings_container)
            .into()
    }

    /// Get the theme for the about window
    pub fn theme(&self) -> Theme {
        CatShieldTheme::base()
    }

    /// Get window settings for the about window
    pub fn window_settings() -> WindowSettings {
        WindowSettings {
            size: Size::new(300.0, 380.0),
            position: Position::Centered,
            resizable: false,
            decorations: true,
            transparent: false,
            min_size: None,
            max_size: None,
            ..WindowSettings::default()
        }
    }

    /// Run the about window application
    pub fn run() -> iced::Result {
        iced::application(Self::new, Self::update, Self::view)
            .title("About Cat Shield")
            .window(Self::window_settings())
            .theme(Self::theme)
            .run()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_about_window_default() {
        let window = AboutWindow::default();
        assert!(!window.version.is_empty());
    }

    #[test]
    fn test_about_window_version() {
        let window = AboutWindow::new();
        assert_eq!(window.version, env!("CARGO_PKG_VERSION"));
    }

    #[test]
    fn test_window_settings() {
        let settings = AboutWindow::window_settings();
        assert!(!settings.resizable);
        assert!(settings.decorations);
        assert_eq!(settings.size, Size::new(300.0, 380.0));
    }
}
