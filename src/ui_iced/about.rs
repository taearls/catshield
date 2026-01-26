//! About window UI using iced
//!
//! This module provides the About window for Cat Shield.
//! It displays version information, credits, and links.

use std::time::{Duration, Instant};

use iced::widget::{button, column, container, row, text, Space};
use iced::window::{Position, Settings as WindowSettings};
use iced::{time, Alignment, Element, Length, Padding, Size, Subscription, Task, Theme};

use crate::ui_iced::cat_animation::CatCompanion;
use crate::ui_iced::theme::{borders, colors, spacing, typography, CatShieldTheme};

/// Messages for the about window
#[derive(Debug, Clone)]
pub enum AboutMessage {
    /// Timer tick for updating cat animation
    Tick(Instant),
    /// Close the about window (via Close button)
    Close,
}

/// State for the about window
pub struct AboutWindow {
    /// Application version
    version: String,
    /// Animated cat companion
    cat: CatCompanion,
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
            cat: CatCompanion::new(),
        }
    }

    /// Update about window state
    pub fn update(&mut self, message: AboutMessage) -> Task<AboutMessage> {
        match message {
            AboutMessage::Tick(now) => {
                self.cat.tick(now);
                Task::none()
            }
            AboutMessage::Close => iced::exit(),
        }
    }

    /// Render the about view with polished styling
    pub fn view(&self) -> Element<'_, AboutMessage> {
        let content = column![
            // Animated cat companion
            container(self.cat.view::<AboutMessage>(false))
                .width(Length::Fill)
                .padding(Padding::from([spacing::WINDOW_PADDING, 0.0]).bottom(spacing::MD))
                .align_x(iced::alignment::Horizontal::Center),
            // App name
            text("Cat Shield")
                .size(typography::SIZE_HEADER)
                .color(colors::TEXT_PRIMARY),
            // Version badge
            container(
                text(format!("Version {}", self.version))
                    .size(typography::SIZE_CAPTION)
                    .color(colors::TEXT_SECONDARY)
            )
            .padding([spacing::XS, spacing::SM])
            .style(|_theme| container::Style {
                background: Some(iced::Background::Color(colors::BACKGROUND_ELEVATED)),
                border: iced::Border {
                    radius: borders::RADIUS_PILL.into(),
                    ..Default::default()
                },
                ..Default::default()
            }),
            // Spacer
            Space::new().height(Length::Fixed(spacing::XL)),
            // Description card
            container(
                text(
                    "A cross-platform application that creates a\n\
                     cat-proof screen overlay to keep your machine\n\
                     awake and protect your work from curious cats."
                )
                .size(typography::SIZE_BODY)
                .color(colors::TEXT_SECONDARY)
            )
            .width(Length::Fill)
            .padding(spacing::MD)
            .align_x(iced::alignment::Horizontal::Center)
            .style(CatShieldTheme::card_container),
            // Spacer
            Space::new().height(Length::Fixed(spacing::XL)),
            // Credits section with badges
            container(
                column![
                    text("Built with")
                        .size(typography::SIZE_CAPTION)
                        .color(colors::TEXT_MUTED),
                    Space::new().height(Length::Fixed(spacing::SM)),
                    row![
                        container(
                            text("🦀 Rust")
                                .size(typography::SIZE_CAPTION)
                                .color(colors::TEXT_SECONDARY)
                        )
                        .padding([spacing::XS, spacing::SM])
                        .style(|_theme| container::Style {
                            background: Some(iced::Background::Color(colors::BACKGROUND_ELEVATED)),
                            border: iced::Border {
                                radius: borders::RADIUS_SM.into(),
                                ..Default::default()
                            },
                            ..Default::default()
                        }),
                        container(
                            text("❄️ iced")
                                .size(typography::SIZE_CAPTION)
                                .color(colors::TEXT_SECONDARY)
                        )
                        .padding([spacing::XS, spacing::SM])
                        .style(|_theme| container::Style {
                            background: Some(iced::Background::Color(colors::BACKGROUND_ELEVATED)),
                            border: iced::Border {
                                radius: borders::RADIUS_SM.into(),
                                ..Default::default()
                            },
                            ..Default::default()
                        }),
                    ]
                    .spacing(spacing::SM),
                ]
                .spacing(spacing::XS)
                .align_x(Alignment::Center)
            )
            .width(Length::Fill)
            .align_x(iced::alignment::Horizontal::Center),
            // Spacer
            Space::new().height(Length::Fixed(spacing::MD)),
            // Links
            container(
                column![
                    text("github.com/taearls/catshield")
                        .size(typography::SIZE_CAPTION)
                        .color(colors::ACCENT),
                    text("MIT License")
                        .size(typography::SIZE_MICRO)
                        .color(colors::TEXT_MUTED),
                ]
                .spacing(spacing::XS)
                .align_x(Alignment::Center)
            )
            .width(Length::Fill)
            .align_x(iced::alignment::Horizontal::Center),
            // Spacer to push button to bottom
            Space::new().height(Length::Fill),
            // Close button
            container(
                button(
                    text("Close")
                        .size(typography::SIZE_BODY)
                        .color(colors::TEXT_ON_ACCENT)
                )
                .padding([spacing::SM, spacing::XL])
                .style(CatShieldTheme::primary_button)
                .on_press(AboutMessage::Close)
            )
            .width(Length::Fill)
            .align_x(iced::alignment::Horizontal::Center)
            .padding(Padding::from([0.0, 0.0]).bottom(spacing::SECTION_PADDING)),
        ]
        .spacing(spacing::SM)
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
        CatShieldTheme::system()
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

    /// Subscription for animation ticks
    ///
    /// Provides periodic ticks at ~30 FPS for smooth cat animation.
    pub fn subscription(&self) -> Subscription<AboutMessage> {
        time::every(Duration::from_millis(33)).map(AboutMessage::Tick)
    }

    /// Run the about window application
    pub fn run() -> iced::Result {
        iced::application(Self::new, Self::update, Self::view)
            .title("About Cat Shield")
            .subscription(Self::subscription)
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
