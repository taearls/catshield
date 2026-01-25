//! Custom theming for Cat Shield iced UI
//!
//! Provides consistent styling across the overlay and settings windows.
//! This module defines the Cat Shield visual identity including:
//! - Color palette (dark and light modes)
//! - Typography constants
//! - Spacing constants
//! - Custom widget styling

use iced::widget::{button, checkbox, container, pick_list, rule, slider, text, text_input};
use iced::{Background, Border, Color, Shadow, Theme};

// ============================================================
// Color Palette
// ============================================================

/// Cat Shield color palette - Dark mode (default)
pub mod colors {
    use iced::Color;

    // === Brand Colors ===

    /// Primary accent color - Cat Shield brand blue
    /// Used for primary buttons, links, and highlights
    pub const ACCENT: Color = Color::from_rgb(0.35, 0.55, 0.95);

    /// Accent hover state
    pub const ACCENT_HOVER: Color = Color::from_rgb(0.45, 0.65, 1.0);

    /// Accent pressed state
    pub const ACCENT_PRESSED: Color = Color::from_rgb(0.25, 0.45, 0.85);

    /// Secondary accent - warmer tone for variety
    pub const ACCENT_WARM: Color = Color::from_rgb(0.95, 0.65, 0.35);

    // === Background Colors ===

    /// Deep background for main containers
    pub const BACKGROUND_DEEP: Color = Color::from_rgb(0.08, 0.08, 0.10);

    /// Primary background for windows
    pub const BACKGROUND_PRIMARY: Color = Color::from_rgb(0.12, 0.12, 0.14);

    /// Secondary background for cards and sections
    pub const BACKGROUND_SECONDARY: Color = Color::from_rgb(0.16, 0.16, 0.18);

    /// Elevated background for hover states and inputs
    pub const BACKGROUND_ELEVATED: Color = Color::from_rgb(0.20, 0.20, 0.22);

    /// Semi-transparent dark background for overlay
    pub const OVERLAY_BACKGROUND: Color = Color::from_rgba(0.08, 0.08, 0.10, 0.92);

    /// Fully opaque dark background for settings
    pub const SETTINGS_BACKGROUND: Color = Color::from_rgb(0.12, 0.12, 0.14);

    // === Text Colors ===

    /// Primary text - high contrast white
    pub const TEXT_PRIMARY: Color = Color::from_rgba(1.0, 1.0, 1.0, 0.95);

    /// Secondary text - slightly muted
    pub const TEXT_SECONDARY: Color = Color::from_rgba(1.0, 1.0, 1.0, 0.72);

    /// Muted text - for hints and disabled states
    pub const TEXT_MUTED: Color = Color::from_rgba(1.0, 1.0, 1.0, 0.45);

    /// Text on accent backgrounds
    pub const TEXT_ON_ACCENT: Color = Color::WHITE;

    // === Status Colors ===

    /// Close/danger button red
    pub const DANGER: Color = Color::from_rgb(0.92, 0.28, 0.28);

    /// Close button hover
    pub const DANGER_HOVER: Color = Color::from_rgb(1.0, 0.38, 0.38);

    /// Close button pressed
    pub const DANGER_PRESSED: Color = Color::from_rgb(0.72, 0.18, 0.18);

    /// Success/positive green
    pub const SUCCESS: Color = Color::from_rgb(0.30, 0.72, 0.40);

    /// Success hover
    pub const SUCCESS_HOVER: Color = Color::from_rgb(0.40, 0.82, 0.50);

    /// Warning/caution orange
    pub const WARNING: Color = Color::from_rgb(1.0, 0.62, 0.22);

    /// Warning hover
    pub const WARNING_HOVER: Color = Color::from_rgb(1.0, 0.72, 0.32);

    // === Legacy aliases (for backward compatibility) ===

    /// Close button red (alias for DANGER)
    pub const CLOSE_BUTTON: Color = DANGER;

    /// Close button hover (alias for DANGER_HOVER)
    pub const CLOSE_BUTTON_HOVER: Color = DANGER_HOVER;

    /// Close button pressed (alias for DANGER_PRESSED)
    pub const CLOSE_BUTTON_PRESSED: Color = DANGER_PRESSED;

    /// Timer warning color (alias for WARNING)
    pub const TIMER_WARNING: Color = WARNING;

    // === Progress Bar Colors ===

    /// Progress bar background (subtle)
    pub const PROGRESS_BACKGROUND: Color = Color::from_rgba(1.0, 1.0, 1.0, 0.15);

    /// Progress bar fill (accent-derived green)
    pub const PROGRESS_FILL: Color = Color::from_rgb(0.35, 0.75, 0.45);

    /// Progress bar fill when in warning state
    pub const PROGRESS_WARNING: Color = WARNING;

    // === Border Colors ===

    /// Subtle border for containers
    pub const BORDER_SUBTLE: Color = Color::from_rgba(1.0, 1.0, 1.0, 0.08);

    /// Medium border for inputs
    pub const BORDER_MEDIUM: Color = Color::from_rgba(1.0, 1.0, 1.0, 0.15);

    /// Strong border for focused elements
    pub const BORDER_FOCUS: Color = ACCENT;

    // === Slider Colors ===

    /// Slider track background
    pub const SLIDER_TRACK: Color = Color::from_rgba(1.0, 1.0, 1.0, 0.12);

    /// Slider track fill (active portion)
    pub const SLIDER_FILL: Color = ACCENT;

    /// Slider handle/thumb
    pub const SLIDER_HANDLE: Color = Color::WHITE;

    /// Slider handle hovered
    pub const SLIDER_HANDLE_HOVER: Color = Color::from_rgb(0.9, 0.9, 0.95);
}

/// Light mode color palette (for system theme support)
pub mod colors_light {
    use iced::Color;

    // === Brand Colors (same as dark, adjusted for light bg) ===

    pub const ACCENT: Color = Color::from_rgb(0.20, 0.45, 0.88);
    pub const ACCENT_HOVER: Color = Color::from_rgb(0.25, 0.50, 0.95);
    pub const ACCENT_PRESSED: Color = Color::from_rgb(0.15, 0.38, 0.78);

    // === Background Colors ===

    pub const BACKGROUND_DEEP: Color = Color::from_rgb(0.92, 0.92, 0.94);
    pub const BACKGROUND_PRIMARY: Color = Color::from_rgb(0.96, 0.96, 0.98);
    pub const BACKGROUND_SECONDARY: Color = Color::from_rgb(0.98, 0.98, 1.0);
    pub const BACKGROUND_ELEVATED: Color = Color::WHITE;
    pub const OVERLAY_BACKGROUND: Color = Color::from_rgba(0.96, 0.96, 0.98, 0.95);
    pub const SETTINGS_BACKGROUND: Color = Color::from_rgb(0.96, 0.96, 0.98);

    // === Text Colors ===

    pub const TEXT_PRIMARY: Color = Color::from_rgba(0.0, 0.0, 0.0, 0.90);
    pub const TEXT_SECONDARY: Color = Color::from_rgba(0.0, 0.0, 0.0, 0.65);
    pub const TEXT_MUTED: Color = Color::from_rgba(0.0, 0.0, 0.0, 0.40);
    pub const TEXT_ON_ACCENT: Color = Color::WHITE;

    // === Status Colors ===

    pub const DANGER: Color = Color::from_rgb(0.85, 0.22, 0.22);
    pub const DANGER_HOVER: Color = Color::from_rgb(0.95, 0.32, 0.32);
    pub const DANGER_PRESSED: Color = Color::from_rgb(0.68, 0.15, 0.15);
    pub const SUCCESS: Color = Color::from_rgb(0.22, 0.62, 0.32);
    pub const WARNING: Color = Color::from_rgb(0.92, 0.52, 0.12);

    // === Border Colors ===

    pub const BORDER_SUBTLE: Color = Color::from_rgba(0.0, 0.0, 0.0, 0.08);
    pub const BORDER_MEDIUM: Color = Color::from_rgba(0.0, 0.0, 0.0, 0.15);
    pub const BORDER_FOCUS: Color = ACCENT;
}

// ============================================================
// Typography Constants
// ============================================================

/// Typography constants for consistent text sizing
pub mod typography {
    /// Extra large text for hero elements
    pub const SIZE_HERO: f32 = 96.0;

    /// Large header text
    pub const SIZE_HEADER_LARGE: f32 = 32.0;

    /// Medium header text
    pub const SIZE_HEADER: f32 = 24.0;

    /// Small header text
    pub const SIZE_HEADER_SMALL: f32 = 20.0;

    /// Title text
    pub const SIZE_TITLE: f32 = 18.0;

    /// Body text (default)
    pub const SIZE_BODY: f32 = 14.0;

    /// Small body text
    pub const SIZE_BODY_SMALL: f32 = 13.0;

    /// Caption/label text
    pub const SIZE_CAPTION: f32 = 12.0;

    /// Micro text for badges, etc
    pub const SIZE_MICRO: f32 = 11.0;
}

// ============================================================
// Spacing Constants
// ============================================================

/// Spacing constants for consistent layouts
pub mod spacing {
    /// Extra small spacing (4px)
    pub const XS: f32 = 4.0;

    /// Small spacing (8px)
    pub const SM: f32 = 8.0;

    /// Medium spacing (12px)
    pub const MD: f32 = 12.0;

    /// Large spacing (16px)
    pub const LG: f32 = 16.0;

    /// Extra large spacing (24px)
    pub const XL: f32 = 24.0;

    /// XXL spacing (32px)
    pub const XXL: f32 = 32.0;

    /// Section padding for content areas
    pub const SECTION_PADDING: f32 = 20.0;

    /// Window padding
    pub const WINDOW_PADDING: f32 = 30.0;

    /// Card/group padding
    pub const CARD_PADDING: f32 = 15.0;
}

// ============================================================
// Border Constants
// ============================================================

/// Border radius constants
pub mod borders {
    /// Small radius for buttons, inputs
    pub const RADIUS_SM: f32 = 4.0;

    /// Medium radius for cards, containers
    pub const RADIUS_MD: f32 = 8.0;

    /// Large radius for modals, dialogs
    pub const RADIUS_LG: f32 = 12.0;

    /// Pill/full radius
    pub const RADIUS_PILL: f32 = 999.0;

    /// Default border width
    pub const WIDTH_DEFAULT: f32 = 1.0;

    /// Thick border for focus states
    pub const WIDTH_FOCUS: f32 = 2.0;
}

// ============================================================
// Shadow Constants
// ============================================================

/// Shadow presets for depth
pub mod shadows {
    use iced::{Color, Shadow, Vector};

    /// Subtle shadow for cards
    pub fn subtle() -> Shadow {
        Shadow {
            color: Color::from_rgba(0.0, 0.0, 0.0, 0.15),
            offset: Vector::new(0.0, 2.0),
            blur_radius: 4.0,
        }
    }

    /// Medium shadow for elevated elements
    pub fn medium() -> Shadow {
        Shadow {
            color: Color::from_rgba(0.0, 0.0, 0.0, 0.20),
            offset: Vector::new(0.0, 4.0),
            blur_radius: 8.0,
        }
    }

    /// Strong shadow for modals
    pub fn strong() -> Shadow {
        Shadow {
            color: Color::from_rgba(0.0, 0.0, 0.0, 0.30),
            offset: Vector::new(0.0, 8.0),
            blur_radius: 16.0,
        }
    }
}

// ============================================================
// System Theme Detection
// ============================================================

/// Represents the user's preferred color scheme
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ColorScheme {
    /// Dark mode (default)
    #[default]
    Dark,
    /// Light mode
    Light,
    /// Follow system preference
    System,
}

impl ColorScheme {
    /// Detect the current system color scheme
    pub fn detect_system() -> Self {
        match dark_light::detect() {
            Ok(dark_light::Mode::Dark) => ColorScheme::Dark,
            Ok(dark_light::Mode::Light) => ColorScheme::Light,
            Ok(dark_light::Mode::Unspecified) | Err(_) => ColorScheme::Dark, // Default to dark
        }
    }

    /// Get the effective color scheme (resolving System to actual mode)
    pub fn effective(&self) -> ColorScheme {
        match self {
            ColorScheme::System => Self::detect_system(),
            other => *other,
        }
    }

    /// Check if this is effectively dark mode
    pub fn is_dark(&self) -> bool {
        matches!(self.effective(), ColorScheme::Dark)
    }

    /// Get the iced Theme for this color scheme
    pub fn to_iced_theme(&self) -> Theme {
        if self.is_dark() {
            Theme::Dark
        } else {
            Theme::Light
        }
    }
}

// ============================================================
// Theme Configuration
// ============================================================

/// Custom theme configuration for Cat Shield
#[derive(Debug, Clone, Copy, Default)]
pub struct CatShieldTheme;

impl CatShieldTheme {
    /// Get the base iced theme (Dark)
    pub fn base() -> Theme {
        Theme::Dark
    }

    /// Get the light theme
    pub fn light() -> Theme {
        Theme::Light
    }

    /// Get theme based on system preference
    pub fn system() -> Theme {
        ColorScheme::detect_system().to_iced_theme()
    }

    /// Get theme for a specific color scheme
    pub fn for_scheme(scheme: ColorScheme) -> Theme {
        scheme.to_iced_theme()
    }

    /// Check if we should use dark mode colors
    pub fn is_dark_mode(theme: &Theme) -> bool {
        matches!(theme, Theme::Dark)
    }

    // ============================================================
    // Container Styles
    // ============================================================

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

    /// Style for card/section containers
    pub fn card_container(_theme: &Theme) -> container::Style {
        container::Style {
            background: Some(Background::Color(colors::BACKGROUND_SECONDARY)),
            border: Border {
                radius: borders::RADIUS_MD.into(),
                width: borders::WIDTH_DEFAULT,
                color: colors::BORDER_SUBTLE,
            },
            ..Default::default()
        }
    }

    /// Style for elevated card containers (with shadow)
    pub fn elevated_container(_theme: &Theme) -> container::Style {
        container::Style {
            background: Some(Background::Color(colors::BACKGROUND_ELEVATED)),
            border: Border {
                radius: borders::RADIUS_MD.into(),
                width: borders::WIDTH_DEFAULT,
                color: colors::BORDER_SUBTLE,
            },
            shadow: shadows::subtle(),
            ..Default::default()
        }
    }

    /// Style for input containers
    pub fn input_container(_theme: &Theme) -> container::Style {
        container::Style {
            background: Some(Background::Color(colors::BACKGROUND_DEEP)),
            border: Border {
                radius: borders::RADIUS_SM.into(),
                width: borders::WIDTH_DEFAULT,
                color: colors::BORDER_MEDIUM,
            },
            ..Default::default()
        }
    }

    // ============================================================
    // Button Styles
    // ============================================================

    /// Style for close button
    pub fn close_button(theme: &Theme, status: button::Status) -> button::Style {
        let base = button::primary(theme, status);

        let (background_color, text_color) = match status {
            button::Status::Active => (colors::DANGER, colors::TEXT_ON_ACCENT),
            button::Status::Hovered => (colors::DANGER_HOVER, colors::TEXT_ON_ACCENT),
            button::Status::Pressed => (colors::DANGER_PRESSED, colors::TEXT_ON_ACCENT),
            button::Status::Disabled => {
                (Color::from_rgba(0.5, 0.5, 0.5, 0.4), colors::TEXT_MUTED)
            }
        };

        button::Style {
            background: Some(Background::Color(background_color)),
            text_color,
            border: Border {
                radius: borders::RADIUS_MD.into(),
                ..base.border
            },
            shadow: if matches!(status, button::Status::Hovered) {
                shadows::subtle()
            } else {
                Shadow::default()
            },
            snap: base.snap,
        }
    }

    /// Style for primary action button
    pub fn primary_button(theme: &Theme, status: button::Status) -> button::Style {
        let base = button::primary(theme, status);

        let (background_color, text_color) = match status {
            button::Status::Active => (colors::ACCENT, colors::TEXT_ON_ACCENT),
            button::Status::Hovered => (colors::ACCENT_HOVER, colors::TEXT_ON_ACCENT),
            button::Status::Pressed => (colors::ACCENT_PRESSED, colors::TEXT_ON_ACCENT),
            button::Status::Disabled => {
                (Color::from_rgba(0.4, 0.6, 0.9, 0.4), colors::TEXT_MUTED)
            }
        };

        button::Style {
            background: Some(Background::Color(background_color)),
            text_color,
            border: Border {
                radius: borders::RADIUS_SM.into(),
                ..base.border
            },
            shadow: if matches!(status, button::Status::Hovered) {
                shadows::subtle()
            } else {
                Shadow::default()
            },
            snap: base.snap,
        }
    }

    /// Style for secondary/cancel button
    pub fn secondary_button(theme: &Theme, status: button::Status) -> button::Style {
        let base = button::secondary(theme, status);

        let (background_color, text_color, border_color) = match status {
            button::Status::Active => (
                Color::from_rgba(1.0, 1.0, 1.0, 0.08),
                colors::TEXT_SECONDARY,
                colors::BORDER_MEDIUM,
            ),
            button::Status::Hovered => (
                Color::from_rgba(1.0, 1.0, 1.0, 0.12),
                colors::TEXT_PRIMARY,
                colors::BORDER_MEDIUM,
            ),
            button::Status::Pressed => (
                Color::from_rgba(1.0, 1.0, 1.0, 0.05),
                colors::TEXT_SECONDARY,
                colors::BORDER_MEDIUM,
            ),
            button::Status::Disabled => (
                Color::from_rgba(0.5, 0.5, 0.5, 0.05),
                colors::TEXT_MUTED,
                colors::BORDER_SUBTLE,
            ),
        };

        button::Style {
            background: Some(Background::Color(background_color)),
            text_color,
            border: Border {
                radius: borders::RADIUS_SM.into(),
                width: borders::WIDTH_DEFAULT,
                color: border_color,
            },
            ..base
        }
    }

    /// Style for ghost/text-only button
    pub fn ghost_button(theme: &Theme, status: button::Status) -> button::Style {
        let base = button::secondary(theme, status);
        let (background_color, text_color) = match status {
            button::Status::Active => (Color::TRANSPARENT, colors::TEXT_SECONDARY),
            button::Status::Hovered => (Color::from_rgba(1.0, 1.0, 1.0, 0.08), colors::TEXT_PRIMARY),
            button::Status::Pressed => {
                (Color::from_rgba(1.0, 1.0, 1.0, 0.04), colors::TEXT_SECONDARY)
            }
            button::Status::Disabled => (Color::TRANSPARENT, colors::TEXT_MUTED),
        };

        button::Style {
            background: Some(Background::Color(background_color)),
            text_color,
            border: Border {
                radius: borders::RADIUS_SM.into(),
                ..Default::default()
            },
            shadow: Shadow::default(),
            snap: base.snap,
        }
    }

    /// Style for danger/destructive button
    pub fn danger_button(theme: &Theme, status: button::Status) -> button::Style {
        let base = button::primary(theme, status);

        let (background_color, text_color, border_color) = match status {
            button::Status::Active => (
                Color::from_rgba(0.92, 0.28, 0.28, 0.15),
                colors::DANGER,
                colors::DANGER,
            ),
            button::Status::Hovered => (
                Color::from_rgba(0.92, 0.28, 0.28, 0.25),
                colors::DANGER_HOVER,
                colors::DANGER_HOVER,
            ),
            button::Status::Pressed => (
                Color::from_rgba(0.92, 0.28, 0.28, 0.10),
                colors::DANGER_PRESSED,
                colors::DANGER_PRESSED,
            ),
            button::Status::Disabled => (
                Color::from_rgba(0.5, 0.5, 0.5, 0.1),
                colors::TEXT_MUTED,
                colors::BORDER_SUBTLE,
            ),
        };

        button::Style {
            background: Some(Background::Color(background_color)),
            text_color,
            border: Border {
                radius: borders::RADIUS_SM.into(),
                width: borders::WIDTH_DEFAULT,
                color: border_color,
            },
            ..base
        }
    }

    /// Style for success button
    pub fn success_button(theme: &Theme, status: button::Status) -> button::Style {
        let base = button::primary(theme, status);

        let (background_color, text_color) = match status {
            button::Status::Active => (colors::SUCCESS, colors::TEXT_ON_ACCENT),
            button::Status::Hovered => (colors::SUCCESS_HOVER, colors::TEXT_ON_ACCENT),
            button::Status::Pressed => {
                (Color::from_rgb(0.22, 0.60, 0.32), colors::TEXT_ON_ACCENT)
            }
            button::Status::Disabled => (Color::from_rgba(0.3, 0.7, 0.4, 0.4), colors::TEXT_MUTED),
        };

        button::Style {
            background: Some(Background::Color(background_color)),
            text_color,
            border: Border {
                radius: borders::RADIUS_SM.into(),
                ..base.border
            },
            shadow: if matches!(status, button::Status::Hovered) {
                shadows::subtle()
            } else {
                Shadow::default()
            },
            snap: base.snap,
        }
    }

    /// Style for tab/navigation button
    pub fn tab_button(_theme: &Theme, status: button::Status, is_selected: bool) -> button::Style {
        let (background_color, text_color, border_color) = if is_selected {
            match status {
                button::Status::Active | button::Status::Hovered | button::Status::Pressed => (
                    colors::ACCENT,
                    colors::TEXT_ON_ACCENT,
                    colors::ACCENT,
                ),
                button::Status::Disabled => (
                    Color::from_rgba(0.4, 0.6, 0.9, 0.4),
                    colors::TEXT_MUTED,
                    colors::BORDER_SUBTLE,
                ),
            }
        } else {
            match status {
                button::Status::Active => (
                    Color::TRANSPARENT,
                    colors::TEXT_SECONDARY,
                    Color::TRANSPARENT,
                ),
                button::Status::Hovered => (
                    Color::from_rgba(1.0, 1.0, 1.0, 0.08),
                    colors::TEXT_PRIMARY,
                    Color::TRANSPARENT,
                ),
                button::Status::Pressed => (
                    Color::from_rgba(1.0, 1.0, 1.0, 0.04),
                    colors::TEXT_SECONDARY,
                    Color::TRANSPARENT,
                ),
                button::Status::Disabled => (
                    Color::TRANSPARENT,
                    colors::TEXT_MUTED,
                    Color::TRANSPARENT,
                ),
            }
        };

        button::Style {
            background: Some(Background::Color(background_color)),
            text_color,
            border: Border {
                radius: borders::RADIUS_SM.into(),
                width: if is_selected {
                    borders::WIDTH_DEFAULT
                } else {
                    0.0
                },
                color: border_color,
            },
            ..Default::default()
        }
    }

    // ============================================================
    // Text Styles
    // ============================================================

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

    /// Style for success text
    pub fn success_text() -> text::Style {
        text::Style {
            color: Some(colors::SUCCESS),
        }
    }

    /// Style for warning text
    pub fn warning_text() -> text::Style {
        text::Style {
            color: Some(colors::WARNING),
        }
    }

    /// Style for danger/error text
    pub fn danger_text() -> text::Style {
        text::Style {
            color: Some(colors::DANGER),
        }
    }

    /// Style for accent/link text
    pub fn accent_text() -> text::Style {
        text::Style {
            color: Some(colors::ACCENT),
        }
    }

    // ============================================================
    // Input Styles
    // ============================================================

    /// Style for text input fields
    pub fn text_input_style(_theme: &Theme, status: text_input::Status) -> text_input::Style {
        let (background, border_color, icon_color, placeholder_color) = match status {
            text_input::Status::Active => (
                colors::BACKGROUND_DEEP,
                colors::BORDER_MEDIUM,
                colors::TEXT_MUTED,
                colors::TEXT_MUTED,
            ),
            text_input::Status::Hovered => (
                colors::BACKGROUND_DEEP,
                colors::BORDER_MEDIUM,
                colors::TEXT_SECONDARY,
                colors::TEXT_MUTED,
            ),
            text_input::Status::Focused { is_hovered: _ } => (
                colors::BACKGROUND_DEEP,
                colors::BORDER_FOCUS,
                colors::TEXT_PRIMARY,
                colors::TEXT_MUTED,
            ),
            text_input::Status::Disabled => (
                Color::from_rgba(0.08, 0.08, 0.1, 0.5),
                colors::BORDER_SUBTLE,
                colors::TEXT_MUTED,
                colors::TEXT_MUTED,
            ),
        };

        text_input::Style {
            background: Background::Color(background),
            border: Border {
                radius: borders::RADIUS_SM.into(),
                width: borders::WIDTH_DEFAULT,
                color: border_color,
            },
            icon: icon_color,
            placeholder: placeholder_color,
            value: colors::TEXT_PRIMARY,
            selection: colors::ACCENT,
        }
    }

    // ============================================================
    // Slider Styles
    // ============================================================

    /// Style for slider widgets
    pub fn slider_style(_theme: &Theme, status: slider::Status) -> slider::Style {
        let (fill_color, track_color, handle_bg, handle_border_color) = match status {
            slider::Status::Active => (
                colors::SLIDER_FILL,
                colors::SLIDER_TRACK,
                colors::SLIDER_HANDLE,
                colors::SLIDER_FILL,
            ),
            slider::Status::Hovered => (
                colors::ACCENT_HOVER,
                colors::SLIDER_TRACK,
                colors::SLIDER_HANDLE_HOVER,
                colors::ACCENT_HOVER,
            ),
            slider::Status::Dragged => (
                colors::ACCENT_PRESSED,
                colors::SLIDER_TRACK,
                colors::SLIDER_HANDLE,
                colors::ACCENT_PRESSED,
            ),
        };

        slider::Style {
            rail: slider::Rail {
                backgrounds: (
                    Background::Color(fill_color),
                    Background::Color(track_color),
                ),
                width: 4.0,
                border: Border {
                    radius: 2.0.into(),
                    ..Default::default()
                },
            },
            handle: slider::Handle {
                shape: slider::HandleShape::Circle { radius: 8.0 },
                background: Background::Color(handle_bg),
                border_width: 2.0,
                border_color: handle_border_color,
            },
        }
    }

    // ============================================================
    // Checkbox Styles
    // ============================================================

    /// Style for checkbox widgets
    pub fn checkbox_style(_theme: &Theme, status: checkbox::Status) -> checkbox::Style {
        let (background, icon_color, border_color, text_color) = match status {
            checkbox::Status::Active { is_checked } => {
                if is_checked {
                    (
                        Background::Color(colors::ACCENT),
                        colors::TEXT_ON_ACCENT,
                        colors::ACCENT,
                        colors::TEXT_PRIMARY,
                    )
                } else {
                    (
                        Background::Color(colors::BACKGROUND_DEEP),
                        colors::TEXT_MUTED,
                        colors::BORDER_MEDIUM,
                        colors::TEXT_PRIMARY,
                    )
                }
            }
            checkbox::Status::Hovered { is_checked } => {
                if is_checked {
                    (
                        Background::Color(colors::ACCENT_HOVER),
                        colors::TEXT_ON_ACCENT,
                        colors::ACCENT_HOVER,
                        colors::TEXT_PRIMARY,
                    )
                } else {
                    (
                        Background::Color(colors::BACKGROUND_ELEVATED),
                        colors::TEXT_SECONDARY,
                        colors::BORDER_MEDIUM,
                        colors::TEXT_PRIMARY,
                    )
                }
            }
            checkbox::Status::Disabled { is_checked } => {
                if is_checked {
                    (
                        Background::Color(Color::from_rgba(0.4, 0.6, 0.9, 0.4)),
                        colors::TEXT_MUTED,
                        colors::BORDER_SUBTLE,
                        colors::TEXT_MUTED,
                    )
                } else {
                    (
                        Background::Color(Color::from_rgba(0.08, 0.08, 0.1, 0.5)),
                        colors::TEXT_MUTED,
                        colors::BORDER_SUBTLE,
                        colors::TEXT_MUTED,
                    )
                }
            }
        };

        checkbox::Style {
            background,
            icon_color,
            border: Border {
                radius: borders::RADIUS_SM.into(),
                width: borders::WIDTH_DEFAULT,
                color: border_color,
            },
            text_color: Some(text_color),
        }
    }

    // ============================================================
    // Pick List (Dropdown) Styles
    // ============================================================

    /// Style for pick list (dropdown) widgets
    pub fn pick_list_style(
        _theme: &Theme,
        status: pick_list::Status,
    ) -> pick_list::Style {
        let (background, text_color, border_color, handle_color, placeholder_color) = match status {
            pick_list::Status::Active => (
                colors::BACKGROUND_DEEP,
                colors::TEXT_PRIMARY,
                colors::BORDER_MEDIUM,
                colors::TEXT_SECONDARY,
                colors::TEXT_MUTED,
            ),
            pick_list::Status::Hovered => (
                colors::BACKGROUND_ELEVATED,
                colors::TEXT_PRIMARY,
                colors::BORDER_MEDIUM,
                colors::TEXT_PRIMARY,
                colors::TEXT_MUTED,
            ),
            pick_list::Status::Opened { is_hovered: _ } => (
                colors::BACKGROUND_DEEP,
                colors::TEXT_PRIMARY,
                colors::BORDER_FOCUS,
                colors::ACCENT,
                colors::TEXT_MUTED,
            ),
        };

        pick_list::Style {
            background: Background::Color(background),
            text_color,
            placeholder_color,
            handle_color,
            border: Border {
                radius: borders::RADIUS_SM.into(),
                width: borders::WIDTH_DEFAULT,
                color: border_color,
            },
        }
    }

    // ============================================================
    // Rule (Divider) Styles
    // ============================================================

    /// Style for horizontal rule (divider)
    pub fn rule_style(_theme: &Theme) -> rule::Style {
        rule::Style {
            color: colors::BORDER_SUBTLE,
            radius: 0.0.into(),
            fill_mode: rule::FillMode::Full,
            snap: true,
        }
    }
}

// ============================================================
// Tests
// ============================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_color_constants_exist() {
        // Verify key colors are defined and have expected ranges
        let accent = colors::ACCENT;
        assert!(accent.r > 0.0 && accent.r < 1.0);
        assert!(accent.g > 0.0 && accent.g < 1.0);
        assert!(accent.b > 0.0 && accent.b < 1.0);

        // Text primary should be near-white
        let text = colors::TEXT_PRIMARY;
        assert!(text.r > 0.9);
        assert!(text.g > 0.9);
        assert!(text.b > 0.9);
    }

    #[test]
    fn test_typography_constants() {
        assert!(typography::SIZE_HERO > typography::SIZE_HEADER_LARGE);
        assert!(typography::SIZE_HEADER_LARGE > typography::SIZE_HEADER);
        assert!(typography::SIZE_HEADER > typography::SIZE_BODY);
        assert!(typography::SIZE_BODY > typography::SIZE_CAPTION);
        assert!(typography::SIZE_CAPTION > typography::SIZE_MICRO);
    }

    #[test]
    fn test_spacing_constants() {
        assert!(spacing::XS < spacing::SM);
        assert!(spacing::SM < spacing::MD);
        assert!(spacing::MD < spacing::LG);
        assert!(spacing::LG < spacing::XL);
        assert!(spacing::XL < spacing::XXL);
    }

    #[test]
    fn test_border_constants() {
        assert!(borders::RADIUS_SM < borders::RADIUS_MD);
        assert!(borders::RADIUS_MD < borders::RADIUS_LG);
        assert!(borders::RADIUS_LG < borders::RADIUS_PILL);
    }

    #[test]
    fn test_theme_base() {
        let theme = CatShieldTheme::base();
        // Just verify we get the Dark theme
        assert!(matches!(theme, Theme::Dark));
    }

    #[test]
    fn test_theme_light() {
        let theme = CatShieldTheme::light();
        assert!(matches!(theme, Theme::Light));
    }

    #[test]
    fn test_shadow_functions() {
        let subtle = shadows::subtle();
        let medium = shadows::medium();
        let strong = shadows::strong();

        // Verify increasing blur radius
        assert!(subtle.blur_radius < medium.blur_radius);
        assert!(medium.blur_radius < strong.blur_radius);

        // Verify increasing offset
        assert!(subtle.offset.y < medium.offset.y);
        assert!(medium.offset.y < strong.offset.y);
    }

    #[test]
    fn test_legacy_color_aliases() {
        // Ensure legacy names still work
        assert_eq!(colors::CLOSE_BUTTON.r, colors::DANGER.r);
        assert_eq!(colors::CLOSE_BUTTON_HOVER.r, colors::DANGER_HOVER.r);
        assert_eq!(colors::TIMER_WARNING.r, colors::WARNING.r);
    }

    #[test]
    fn test_light_mode_colors_differ_from_dark() {
        // Light mode backgrounds should be lighter than dark mode
        assert!(colors_light::BACKGROUND_PRIMARY.r > colors::BACKGROUND_PRIMARY.r);
        assert!(colors_light::BACKGROUND_PRIMARY.g > colors::BACKGROUND_PRIMARY.g);
        assert!(colors_light::BACKGROUND_PRIMARY.b > colors::BACKGROUND_PRIMARY.b);

        // Light mode text should be darker
        assert!(colors_light::TEXT_PRIMARY.r < colors::TEXT_PRIMARY.r);
    }
}
