//! Custom theming for Cat Shield iced UI
//!
//! Provides consistent styling across the overlay and settings windows.

use iced::widget::{button, container, text};
use iced::{Background, Border, Color, Theme};

/// Cat Shield color palette
pub mod colors {
    use iced::Color;

    /// Semi-transparent dark background for overlay
    pub const OVERLAY_BACKGROUND: Color = Color::from_rgba(0.1, 0.1, 0.1, 0.85);

    /// Fully opaque dark background for settings
    pub const SETTINGS_BACKGROUND: Color = Color::from_rgb(0.15, 0.15, 0.15);

    /// Primary accent color (cat shield brand)
    pub const ACCENT: Color = Color::from_rgb(0.4, 0.6, 0.9);

    /// Close button red
    pub const CLOSE_BUTTON: Color = Color::from_rgb(0.9, 0.3, 0.3);

    /// Close button hover
    pub const CLOSE_BUTTON_HOVER: Color = Color::from_rgb(1.0, 0.4, 0.4);

    /// Close button pressed
    pub const CLOSE_BUTTON_PRESSED: Color = Color::from_rgb(0.7, 0.2, 0.2);

    /// Text primary (white)
    pub const TEXT_PRIMARY: Color = Color::WHITE;

    /// Text secondary (muted white)
    pub const TEXT_SECONDARY: Color = Color::from_rgba(1.0, 1.0, 1.0, 0.7);

    /// Text muted (dim white)
    pub const TEXT_MUTED: Color = Color::from_rgba(1.0, 1.0, 1.0, 0.4);
}

/// Custom theme configuration for Cat Shield
#[derive(Debug, Clone, Copy, Default)]
pub struct CatShieldTheme;

impl CatShieldTheme {
    /// Get the base iced theme (Dark)
    pub fn base() -> Theme {
        Theme::Dark
    }

    /// Style for the overlay background container
    pub fn overlay_container(_theme: &Theme) -> container::Style {
        container::Style {
            background: Some(Background::Color(colors::OVERLAY_BACKGROUND)),
            ..Default::default()
        }
    }

    /// Style for the settings window background container
    pub fn settings_container(_theme: &Theme) -> container::Style {
        container::Style {
            background: Some(Background::Color(colors::SETTINGS_BACKGROUND)),
            ..Default::default()
        }
    }

    /// Style for close button
    pub fn close_button(theme: &Theme, status: button::Status) -> button::Style {
        let base = button::primary(theme, status);

        let background_color = match status {
            button::Status::Active => colors::CLOSE_BUTTON,
            button::Status::Hovered => colors::CLOSE_BUTTON_HOVER,
            button::Status::Pressed => colors::CLOSE_BUTTON_PRESSED,
            button::Status::Disabled => Color::from_rgba(0.5, 0.5, 0.5, 0.5),
        };

        button::Style {
            background: Some(Background::Color(background_color)),
            text_color: colors::TEXT_PRIMARY,
            border: Border {
                radius: 8.0.into(),
                ..base.border
            },
            ..base
        }
    }

    /// Style for primary action button
    pub fn primary_button(theme: &Theme, status: button::Status) -> button::Style {
        let base = button::primary(theme, status);

        let background_color = match status {
            button::Status::Active => colors::ACCENT,
            button::Status::Hovered => Color::from_rgb(0.5, 0.7, 1.0),
            button::Status::Pressed => Color::from_rgb(0.3, 0.5, 0.8),
            button::Status::Disabled => Color::from_rgba(0.4, 0.6, 0.9, 0.5),
        };

        button::Style {
            background: Some(Background::Color(background_color)),
            text_color: colors::TEXT_PRIMARY,
            border: Border {
                radius: 6.0.into(),
                ..base.border
            },
            ..base
        }
    }

    /// Style for large timer text
    pub fn timer_text() -> text::Style {
        text::Style {
            color: Some(colors::TEXT_PRIMARY),
        }
    }

    /// Style for secondary/instructional text
    pub fn secondary_text() -> text::Style {
        text::Style {
            color: Some(colors::TEXT_SECONDARY),
        }
    }

    /// Style for muted/hint text
    pub fn muted_text() -> text::Style {
        text::Style {
            color: Some(colors::TEXT_MUTED),
        }
    }
}
