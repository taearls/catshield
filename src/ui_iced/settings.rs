//! Settings window UI using iced
//!
//! This module provides the settings window for configuring Cat Shield preferences.
//! It follows the Elm architecture (State -> Message -> View) consistent with the overlay.
//!
//! # Sections
//!
//! - **Overlay**: Visual settings (opacity, color)
//! - **Behavior**: Functional settings (default timer, exit key)
//! - **Advanced**: Developer options (trace logging)
//! - **About**: Version and links
//!
//! # State Management
//!
//! The settings window maintains a working copy of the configuration that can be
//! saved (Apply/OK), cancelled (Cancel), or reset to defaults (Reset).

use std::time::{Duration, Instant};

use iced::widget::{
    button, checkbox, column, container, pick_list, row, rule, scrollable, slider, text,
    text_input, Space,
};
use iced::window::{Position, Settings as WindowSettings};
use iced::{time, Alignment, Color, Element, Length, Size, Subscription, Task, Theme};

use crate::config::{Config, DEFAULT_OVERLAY_OPACITY, MAX_OVERLAY_OPACITY, MIN_OVERLAY_OPACITY};
use crate::ui_iced::cat_animation::{CatCompanion, CatPosition};
use crate::ui_iced::theme::{borders, colors, spacing, typography, CatShieldTheme, ColorScheme};

/// Preset overlay colors for quick selection
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OverlayColor {
    /// Dark gray (default)
    Gray,
    /// Deep blue
    Blue,
    /// Dark green
    Green,
    /// Dark red
    Red,
    /// Dark purple
    Purple,
    /// Custom color (use hex input)
    Custom,
}

impl OverlayColor {
    /// Get the RGB values for this color preset
    pub fn to_rgb(&self) -> (f32, f32, f32) {
        match self {
            OverlayColor::Gray => (0.1, 0.1, 0.1),
            OverlayColor::Blue => (0.05, 0.1, 0.2),
            OverlayColor::Green => (0.05, 0.15, 0.1),
            OverlayColor::Red => (0.15, 0.05, 0.05),
            OverlayColor::Purple => (0.12, 0.08, 0.18),
            OverlayColor::Custom => (0.1, 0.1, 0.1), // Default for custom, actual value from hex input
        }
    }

    /// Get the config string representation for this color
    pub fn to_config_string(&self) -> Option<String> {
        match self {
            OverlayColor::Gray => Some("gray".to_string()),
            OverlayColor::Blue => Some("blue".to_string()),
            OverlayColor::Green => Some("green".to_string()),
            OverlayColor::Red => Some("red".to_string()),
            OverlayColor::Purple => Some("purple".to_string()),
            OverlayColor::Custom => None, // Custom uses hex_color_input
        }
    }

    /// Parse a config string to an OverlayColor
    pub fn from_config_string(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "gray" | "grey" | "dark" => OverlayColor::Gray,
            "blue" => OverlayColor::Blue,
            "green" => OverlayColor::Green,
            "red" => OverlayColor::Red,
            "purple" => OverlayColor::Purple,
            _ if s.starts_with('#') => OverlayColor::Custom,
            _ => OverlayColor::Gray, // Default fallback
        }
    }

    /// All available color presets
    pub const ALL: [OverlayColor; 6] = [
        OverlayColor::Gray,
        OverlayColor::Blue,
        OverlayColor::Green,
        OverlayColor::Red,
        OverlayColor::Purple,
        OverlayColor::Custom,
    ];
}

impl std::fmt::Display for OverlayColor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OverlayColor::Gray => write!(f, "Dark Gray"),
            OverlayColor::Blue => write!(f, "Deep Blue"),
            OverlayColor::Green => write!(f, "Forest Green"),
            OverlayColor::Red => write!(f, "Dark Red"),
            OverlayColor::Purple => write!(f, "Dark Purple"),
            OverlayColor::Custom => write!(f, "Custom..."),
        }
    }
}

/// Timer duration presets
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimerPreset {
    /// 5 minutes
    FiveMinutes,
    /// 15 minutes
    FifteenMinutes,
    /// 30 minutes
    ThirtyMinutes,
    /// 1 hour
    OneHour,
    /// 2 hours
    TwoHours,
    /// Custom duration
    Custom,
    /// No default timer
    None,
}

impl TimerPreset {
    /// Get the duration string for this preset
    pub fn to_duration_string(&self) -> Option<String> {
        match self {
            TimerPreset::FiveMinutes => Some("5m".to_string()),
            TimerPreset::FifteenMinutes => Some("15m".to_string()),
            TimerPreset::ThirtyMinutes => Some("30m".to_string()),
            TimerPreset::OneHour => Some("1h".to_string()),
            TimerPreset::TwoHours => Some("2h".to_string()),
            TimerPreset::Custom => None, // Use custom input
            TimerPreset::None => None,
        }
    }

    /// All available timer presets
    pub const ALL: [TimerPreset; 7] = [
        TimerPreset::None,
        TimerPreset::FiveMinutes,
        TimerPreset::FifteenMinutes,
        TimerPreset::ThirtyMinutes,
        TimerPreset::OneHour,
        TimerPreset::TwoHours,
        TimerPreset::Custom,
    ];

    /// Parse a duration string to find matching preset
    pub fn from_duration_string(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "5m" => TimerPreset::FiveMinutes,
            "15m" => TimerPreset::FifteenMinutes,
            "30m" => TimerPreset::ThirtyMinutes,
            "1h" => TimerPreset::OneHour,
            "2h" => TimerPreset::TwoHours,
            _ => TimerPreset::Custom,
        }
    }
}

impl std::fmt::Display for TimerPreset {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TimerPreset::FiveMinutes => write!(f, "5 minutes"),
            TimerPreset::FifteenMinutes => write!(f, "15 minutes"),
            TimerPreset::ThirtyMinutes => write!(f, "30 minutes"),
            TimerPreset::OneHour => write!(f, "1 hour"),
            TimerPreset::TwoHours => write!(f, "2 hours"),
            TimerPreset::Custom => write!(f, "Custom..."),
            TimerPreset::None => write!(f, "No default"),
        }
    }
}

/// Color scheme preference for the application
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ColorSchemePreference {
    /// Follow system dark/light mode
    #[default]
    System,
    /// Always use dark mode
    Dark,
    /// Always use light mode
    Light,
}

impl ColorSchemePreference {
    /// Convert to config string
    pub fn to_config_string(&self) -> String {
        match self {
            ColorSchemePreference::System => "system".to_string(),
            ColorSchemePreference::Dark => "dark".to_string(),
            ColorSchemePreference::Light => "light".to_string(),
        }
    }

    /// Parse from config string
    pub fn from_config_string(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "dark" => ColorSchemePreference::Dark,
            "light" => ColorSchemePreference::Light,
            _ => ColorSchemePreference::System,
        }
    }

    /// All available preferences
    pub const ALL: [ColorSchemePreference; 3] = [
        ColorSchemePreference::System,
        ColorSchemePreference::Dark,
        ColorSchemePreference::Light,
    ];

    /// Convert to ColorScheme for the theme system
    pub fn to_color_scheme(&self) -> ColorScheme {
        match self {
            ColorSchemePreference::System => ColorScheme::System,
            ColorSchemePreference::Dark => ColorScheme::Dark,
            ColorSchemePreference::Light => ColorScheme::Light,
        }
    }
}

impl std::fmt::Display for ColorSchemePreference {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ColorSchemePreference::System => write!(f, "Follow System"),
            ColorSchemePreference::Dark => write!(f, "Dark Mode"),
            ColorSchemePreference::Light => write!(f, "Light Mode"),
        }
    }
}

/// Which section of settings is currently expanded/focused
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SettingsSection {
    #[default]
    Overlay,
    Behavior,
    Advanced,
    About,
}

/// Messages for the settings window
#[derive(Debug, Clone)]
pub enum SettingsMessage {
    // Animation
    /// Timer tick for updating cat animation
    Tick(Instant),

    // Overlay section
    /// Opacity slider changed
    OpacityChanged(f64),
    /// Color preset selected
    ColorPresetSelected(OverlayColor),
    /// Custom hex color input changed
    HexColorChanged(String),

    // Behavior section
    /// Timer preset selected
    TimerPresetSelected(TimerPreset),
    /// Custom timer duration input changed
    CustomTimerChanged(String),
    /// Exit key input changed
    ExitKeyChanged(String),

    // Allowed keys
    /// Add new allowed key
    AddAllowedKey,
    /// Remove allowed key at index
    RemoveAllowedKey(usize),
    /// Allowed key input changed (for the new key being added)
    AllowedKeyInputChanged(String),
    /// Add media keys preset (F11, F12)
    AddMediaKeysPreset,
    /// Add Spotlight preset (Cmd+Space)
    AddSpotlightPreset,

    // Advanced section
    /// Launch at login toggled
    LaunchAtLoginToggled(bool),
    /// Trace logging toggled
    TraceLoggingToggled(bool),
    /// Color scheme preference changed
    ColorSchemeChanged(ColorSchemePreference),

    // Cat companion section
    /// Show cat toggled
    ShowCatToggled(bool),
    /// Cat position changed
    CatPositionChanged(CatPosition),

    // Navigation
    /// Switch to a different section
    SwitchSection(SettingsSection),

    // Actions
    /// Save settings and close
    Save,
    /// Apply settings without closing
    Apply,
    /// Cancel and close without saving
    Cancel,
    /// Reset to defaults
    ResetDefaults,

    // Window events
    /// Window close requested
    CloseRequested,
}

/// State for the settings window
pub struct SettingsWindow {
    // Working copy of config
    /// Overlay opacity (0.1 - 0.9)
    opacity: f64,
    /// Selected color preset
    color_preset: OverlayColor,
    /// Custom hex color input (when color_preset is Custom)
    hex_color_input: String,

    /// Selected timer preset
    timer_preset: TimerPreset,
    /// Custom timer duration string
    custom_timer: String,
    /// Exit key string
    exit_key: String,

    /// Allowed keys list
    allowed_keys: Vec<String>,
    /// Input for new allowed key
    new_allowed_key_input: String,

    /// Launch at login setting
    launch_at_login: bool,
    /// Trace logging enabled
    trace_logging: bool,
    /// Color scheme preference
    color_scheme: ColorSchemePreference,

    /// Whether to show animated cat companion
    show_cat: bool,
    /// Position of the cat companion
    cat_position: CatPosition,
    /// Animated cat companion for preview
    cat_preview: CatCompanion,

    // UI state
    /// Currently selected section
    current_section: SettingsSection,
    /// Whether there are unsaved changes
    has_changes: bool,
    /// Original config for comparison
    original_config: Config,
    /// Error message to display (if any)
    error_message: Option<String>,
    /// Success message to display (if any)
    success_message: Option<String>,
}

impl Default for SettingsWindow {
    fn default() -> Self {
        Self::new()
    }
}

impl SettingsWindow {
    /// Create a new settings window, loading current config
    pub fn new() -> Self {
        let config = Config::load();
        Self::from_config(config)
    }

    /// Create settings window from a specific config
    pub fn from_config(config: Config) -> Self {
        let opacity = config.opacity();
        let timer_preset = config
            .default_timer
            .as_ref()
            .map(|t| TimerPreset::from_duration_string(t))
            .unwrap_or(TimerPreset::None);
        let custom_timer = config.default_timer.clone().unwrap_or_default();

        // Parse color from config
        let color_str = config.color();
        let color_preset = OverlayColor::from_config_string(color_str);
        let hex_color_input = if color_preset == OverlayColor::Custom {
            color_str.to_string()
        } else {
            String::new()
        };

        Self {
            opacity,
            color_preset,
            hex_color_input,

            timer_preset,
            custom_timer,
            exit_key: config.exit_key.clone().unwrap_or_default(),

            allowed_keys: config.allowed_keys.clone().unwrap_or_default(),
            new_allowed_key_input: String::new(),

            launch_at_login: config.launch_at_login.unwrap_or(false),
            trace_logging: config.enable_trace_logging.unwrap_or(false),
            color_scheme: ColorSchemePreference::from_config_string(config.color_scheme()),

            show_cat: config.show_cat(),
            cat_position: CatPosition::from_config_string(config.cat_position()),
            cat_preview: CatCompanion::with_settings(config.show_cat(), CatPosition::from_config_string(config.cat_position())),

            current_section: SettingsSection::Overlay,
            has_changes: false,
            original_config: config,
            error_message: None,
            success_message: None,
        }
    }

    /// Build a Config from current settings
    fn build_config(&self) -> Config {
        let default_timer = match self.timer_preset {
            TimerPreset::None => None,
            TimerPreset::Custom => {
                if self.custom_timer.is_empty() {
                    None
                } else {
                    Some(self.custom_timer.clone())
                }
            }
            preset => preset.to_duration_string(),
        };

        // Build overlay color - use hex input for Custom, otherwise use preset name
        let overlay_color = match self.color_preset {
            OverlayColor::Custom => {
                if self.hex_color_input.is_empty() {
                    None
                } else {
                    Some(self.hex_color_input.clone())
                }
            }
            preset => preset.to_config_string(),
        };

        Config {
            exit_key: if self.exit_key.is_empty() {
                None
            } else {
                Some(self.exit_key.clone())
            },
            default_timer,
            overlay_opacity: Some(self.opacity),
            overlay_color,
            allowed_keys: if self.allowed_keys.is_empty() {
                None
            } else {
                Some(self.allowed_keys.clone())
            },
            launch_at_login: Some(self.launch_at_login),
            enable_trace_logging: Some(self.trace_logging),
            color_scheme: Some(self.color_scheme.to_config_string()),
            show_cat: Some(self.show_cat),
            cat_position: Some(self.cat_position.to_config_string().to_string()),
        }
    }

    /// Check if settings have changed from original
    fn check_for_changes(&mut self) {
        let current = self.build_config();
        // Compare effective opacity and color values rather than Option types to handle
        // the case where original_config values are None (defaults)
        self.has_changes = current.exit_key != self.original_config.exit_key
            || current.default_timer != self.original_config.default_timer
            || current.opacity() != self.original_config.opacity()
            || current.color() != self.original_config.color()
            || current.allowed_keys != self.original_config.allowed_keys
            || current.launch_at_login != self.original_config.launch_at_login
            || current.enable_trace_logging != self.original_config.enable_trace_logging
            || current.color_scheme() != self.original_config.color_scheme()
            || current.show_cat() != self.original_config.show_cat()
            || current.cat_position() != self.original_config.cat_position();
    }

    /// Save settings to config file
    fn save_settings(&mut self) -> Result<(), String> {
        let config = self.build_config();
        config.save()?;
        self.original_config = config;
        self.has_changes = false;
        Ok(())
    }

    /// Reset all settings to defaults
    fn reset_to_defaults(&mut self) {
        self.opacity = DEFAULT_OVERLAY_OPACITY;
        self.color_preset = OverlayColor::Gray;
        self.hex_color_input = String::new();
        self.timer_preset = TimerPreset::None;
        self.custom_timer = String::new();
        self.exit_key = String::new();
        self.allowed_keys = Vec::new();
        self.new_allowed_key_input = String::new();
        self.launch_at_login = false;
        self.trace_logging = false;
        self.color_scheme = ColorSchemePreference::System;
        self.show_cat = true;
        self.cat_position = CatPosition::default();
        self.cat_preview = CatCompanion::with_settings(true, CatPosition::default());
        self.check_for_changes();
    }

    /// Update settings state
    pub fn update(&mut self, message: SettingsMessage) -> Task<SettingsMessage> {
        // Clear messages on any action
        self.error_message = None;
        self.success_message = None;

        match message {
            // Animation tick
            SettingsMessage::Tick(now) => {
                // Update cat animation for preview
                self.cat_preview.tick(now);
                // Don't clear messages for ticks
                return Task::none();
            }

            // Overlay section
            SettingsMessage::OpacityChanged(value) => {
                self.opacity = value.clamp(MIN_OVERLAY_OPACITY, MAX_OVERLAY_OPACITY);
                self.check_for_changes();
            }
            SettingsMessage::ColorPresetSelected(color) => {
                self.color_preset = color;
                // Clear hex input when switching away from Custom
                if color != OverlayColor::Custom {
                    self.hex_color_input.clear();
                }
                self.check_for_changes();
            }
            SettingsMessage::HexColorChanged(value) => {
                self.hex_color_input = value;
                self.color_preset = OverlayColor::Custom;
                self.check_for_changes();
            }

            // Behavior section
            SettingsMessage::TimerPresetSelected(preset) => {
                self.timer_preset = preset;
                if let Some(duration) = preset.to_duration_string() {
                    self.custom_timer = duration;
                }
                self.check_for_changes();
            }
            SettingsMessage::CustomTimerChanged(value) => {
                self.custom_timer = value;
                self.timer_preset = TimerPreset::Custom;
                self.check_for_changes();
            }
            SettingsMessage::ExitKeyChanged(value) => {
                self.exit_key = value;
                self.check_for_changes();
            }

            // Allowed keys
            SettingsMessage::AddAllowedKey => {
                let key = self.new_allowed_key_input.trim().to_string();
                if !key.is_empty() && !self.allowed_keys.contains(&key) {
                    self.allowed_keys.push(key);
                    self.new_allowed_key_input.clear();
                    self.check_for_changes();
                }
            }
            SettingsMessage::RemoveAllowedKey(index) => {
                if index < self.allowed_keys.len() {
                    self.allowed_keys.remove(index);
                    self.check_for_changes();
                }
            }
            SettingsMessage::AllowedKeyInputChanged(value) => {
                self.new_allowed_key_input = value;
            }
            SettingsMessage::AddMediaKeysPreset => {
                for key in ["F11", "F12"] {
                    if !self.allowed_keys.contains(&key.to_string()) {
                        self.allowed_keys.push(key.to_string());
                    }
                }
                self.check_for_changes();
            }
            SettingsMessage::AddSpotlightPreset => {
                let spotlight = "Cmd+Space".to_string();
                if !self.allowed_keys.contains(&spotlight) {
                    self.allowed_keys.push(spotlight);
                }
                self.check_for_changes();
            }

            // Advanced section
            SettingsMessage::LaunchAtLoginToggled(enabled) => {
                self.launch_at_login = enabled;
                self.check_for_changes();
            }
            SettingsMessage::TraceLoggingToggled(enabled) => {
                self.trace_logging = enabled;
                self.check_for_changes();
            }
            SettingsMessage::ColorSchemeChanged(scheme) => {
                self.color_scheme = scheme;
                self.check_for_changes();
            }

            // Cat companion section
            SettingsMessage::ShowCatToggled(enabled) => {
                self.show_cat = enabled;
                self.cat_preview.visible = enabled;
                self.check_for_changes();
            }
            SettingsMessage::CatPositionChanged(position) => {
                self.cat_position = position;
                self.cat_preview.position = position;
                self.check_for_changes();
            }

            // Navigation
            SettingsMessage::SwitchSection(section) => {
                self.current_section = section;
            }

            // Actions
            SettingsMessage::Save => match self.save_settings() {
                Ok(()) => {
                    self.success_message = Some("Settings saved".to_string());
                    return iced::exit();
                }
                Err(e) => {
                    self.error_message = Some(format!("Failed to save: {e}"));
                }
            },
            SettingsMessage::Apply => match self.save_settings() {
                Ok(()) => {
                    self.success_message = Some("Settings applied".to_string());
                }
                Err(e) => {
                    self.error_message = Some(format!("Failed to apply: {e}"));
                }
            },
            SettingsMessage::Cancel | SettingsMessage::CloseRequested => {
                return iced::exit();
            }
            SettingsMessage::ResetDefaults => {
                self.reset_to_defaults();
            }
        }

        Task::none()
    }

    /// Render the settings view with polished layout
    pub fn view(&self) -> Element<'_, SettingsMessage> {
        let content = column![
            // Header
            self.view_header(),
            rule::horizontal(1).style(CatShieldTheme::rule_style),
            // Navigation tabs
            self.view_tabs(),
            rule::horizontal(1).style(CatShieldTheme::rule_style),
            // Main content area (scrollable)
            scrollable(self.view_current_section())
                .height(Length::Fill)
                .width(Length::Fill),
            // Footer with action buttons
            rule::horizontal(1).style(CatShieldTheme::rule_style),
            self.view_footer(),
        ]
        .spacing(0)
        .width(Length::Fill)
        .height(Length::Fill);

        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .style(CatShieldTheme::settings_container)
            .into()
    }

    /// Render the header with polished styling
    fn view_header(&self) -> Element<'_, SettingsMessage> {
        let title = row![
            text("🐱").size(typography::SIZE_HEADER),
            Space::new().width(Length::Fixed(spacing::SM)),
            text("Cat Shield Settings")
                .size(typography::SIZE_HEADER)
                .color(colors::TEXT_PRIMARY),
        ]
        .align_y(Alignment::Center);

        let subtitle = if self.has_changes {
            row![
                text("●")
                    .size(typography::SIZE_CAPTION)
                    .color(colors::WARNING),
                Space::new().width(Length::Fixed(spacing::XS)),
                text("Unsaved changes")
                    .size(typography::SIZE_CAPTION)
                    .color(colors::WARNING),
            ]
            .align_y(Alignment::Center)
        } else {
            row![text("Configure your protection settings")
                .size(typography::SIZE_CAPTION)
                .color(colors::TEXT_SECONDARY),]
        };

        container(
            column![title, subtitle]
                .spacing(spacing::SM)
                .align_x(Alignment::Center),
        )
        .width(Length::Fill)
        .padding([spacing::SECTION_PADDING, spacing::WINDOW_PADDING])
        .into()
    }

    /// Render the navigation tabs with polished styling
    fn view_tabs(&self) -> Element<'_, SettingsMessage> {
        let make_tab = |section: SettingsSection,
                        label: &'static str,
                        icon: &'static str,
                        current: SettingsSection| {
            let is_selected = current == section;

            button(
                row![
                    text(icon).size(typography::SIZE_BODY),
                    Space::new().width(Length::Fixed(spacing::XS)),
                    text(label)
                        .size(typography::SIZE_BODY)
                        .color(if is_selected {
                            colors::TEXT_ON_ACCENT
                        } else {
                            colors::TEXT_SECONDARY
                        }),
                ]
                .align_y(Alignment::Center),
            )
            .padding([spacing::SM, spacing::LG])
            .style(move |theme, status| CatShieldTheme::tab_button(theme, status, is_selected))
            .on_press(SettingsMessage::SwitchSection(section))
        };

        let current = self.current_section;

        container(
            row![
                make_tab(SettingsSection::Overlay, "Overlay", "🎨", current),
                make_tab(SettingsSection::Behavior, "Behavior", "⚙️", current),
                make_tab(SettingsSection::Advanced, "Advanced", "🔧", current),
                make_tab(SettingsSection::About, "About", "ℹ️", current),
            ]
            .spacing(spacing::SM)
            .align_y(Alignment::Center),
        )
        .width(Length::Fill)
        .padding([spacing::MD, spacing::WINDOW_PADDING])
        .into()
    }

    /// Render the current section content
    fn view_current_section(&self) -> Element<'_, SettingsMessage> {
        let content = match self.current_section {
            SettingsSection::Overlay => self.view_overlay_section(),
            SettingsSection::Behavior => self.view_behavior_section(),
            SettingsSection::Advanced => self.view_advanced_section(),
            SettingsSection::About => self.view_about_section(),
        };

        container(content)
            .width(Length::Fill)
            .padding([spacing::SECTION_PADDING, spacing::WINDOW_PADDING])
            .into()
    }

    /// Get the current effective RGB color for preview
    fn get_preview_rgb(&self) -> (f32, f32, f32) {
        if self.color_preset == OverlayColor::Custom && !self.hex_color_input.is_empty() {
            // Try to parse hex color, fall back to gray on invalid input
            Config::parse_color_to_rgb(&self.hex_color_input).unwrap_or((0.1, 0.1, 0.1))
        } else {
            self.color_preset.to_rgb()
        }
    }

    /// Render the overlay settings section with polished styling
    fn view_overlay_section(&self) -> Element<'_, SettingsMessage> {
        let opacity_percent = (self.opacity * 100.0).round() as i32;

        // Opacity preview box using selected color (preset or custom hex)
        let (r, g, b) = self.get_preview_rgb();
        let preview_color = Color::from_rgba(r, g, b, self.opacity as f32);

        // Check if hex color is valid (for showing error state)
        let hex_is_valid = self.hex_color_input.is_empty()
            || Config::parse_color_to_rgb(&self.hex_color_input).is_some();

        column![
            // Opacity setting
            self.view_setting_group(
                "Opacity",
                "How dark the overlay appears (10% - 90%)",
                column![
                    row![
                        text("Transparency")
                            .size(typography::SIZE_BODY)
                            .color(colors::TEXT_SECONDARY),
                        Space::new().width(Length::Fill),
                        container(
                            text(format!("{opacity_percent}%"))
                                .size(typography::SIZE_BODY)
                                .color(colors::TEXT_PRIMARY)
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
                    .align_y(Alignment::Center),
                    slider(
                        MIN_OVERLAY_OPACITY..=MAX_OVERLAY_OPACITY,
                        self.opacity,
                        SettingsMessage::OpacityChanged
                    )
                    .step(0.05)
                    .style(CatShieldTheme::slider_style),
                    row![
                        text("More transparent")
                            .size(typography::SIZE_MICRO)
                            .color(colors::TEXT_MUTED),
                        Space::new().width(Length::Fill),
                        text("More opaque")
                            .size(typography::SIZE_MICRO)
                            .color(colors::TEXT_MUTED),
                    ],
                    // Preview section
                    Space::new().height(Length::Fixed(spacing::LG)),
                    text("Preview")
                        .size(typography::SIZE_CAPTION)
                        .color(colors::TEXT_SECONDARY),
                    container(
                        column![
                            text("🐱").size(32.0),
                            text("Cat Shield Active")
                                .size(typography::SIZE_TITLE)
                                .color(colors::TEXT_PRIMARY),
                        ]
                        .align_x(Alignment::Center)
                        .spacing(spacing::SM)
                    )
                    .width(Length::Fill)
                    .height(Length::Fixed(100.0))
                    .padding(spacing::SECTION_PADDING)
                    .align_x(iced::alignment::Horizontal::Center)
                    .align_y(iced::alignment::Vertical::Center)
                    .style(move |_theme| container::Style {
                        background: Some(iced::Background::Color(preview_color)),
                        border: iced::Border {
                            radius: borders::RADIUS_MD.into(),
                            width: borders::WIDTH_DEFAULT,
                            color: colors::BORDER_SUBTLE,
                        },
                        ..Default::default()
                    }),
                ]
                .spacing(spacing::SM),
            ),
            Space::new().height(Length::Fixed(spacing::XL)),
            // Color preset setting
            self.view_setting_group(
                "Overlay Color",
                "Choose a color for the overlay background",
                column![
                    pick_list(
                        OverlayColor::ALL.as_slice(),
                        Some(self.color_preset),
                        SettingsMessage::ColorPresetSelected
                    )
                    .width(Length::Fixed(200.0))
                    .padding([spacing::SM, spacing::MD])
                    .style(CatShieldTheme::pick_list_style),
                    // Show hex color input when Custom is selected
                    if self.color_preset == OverlayColor::Custom {
                        column![
                            Space::new().height(Length::Fixed(spacing::MD)),
                            row![
                                text("Hex color:")
                                    .size(typography::SIZE_BODY_SMALL)
                                    .color(colors::TEXT_SECONDARY),
                                text_input("#RRGGBB", &self.hex_color_input)
                                    .on_input(SettingsMessage::HexColorChanged)
                                    .padding([spacing::SM, spacing::MD])
                                    .width(Length::Fixed(120.0))
                                    .style(CatShieldTheme::text_input_style),
                            ]
                            .spacing(spacing::MD)
                            .align_y(Alignment::Center),
                            if !hex_is_valid {
                                text("Invalid hex color format")
                                    .size(typography::SIZE_MICRO)
                                    .color(colors::WARNING)
                            } else {
                                text("").size(typography::SIZE_MICRO)
                            },
                        ]
                        .spacing(spacing::XS)
                    } else {
                        column![]
                    },
                ]
                .spacing(spacing::XS),
            ),
            Space::new().height(Length::Fixed(spacing::XL)),
            // Color scheme / dark mode setting
            self.view_setting_group(
                "Appearance",
                "Choose dark mode, light mode, or follow system preference",
                column![
                    pick_list(
                        ColorSchemePreference::ALL.as_slice(),
                        Some(self.color_scheme),
                        SettingsMessage::ColorSchemeChanged
                    )
                    .width(Length::Fixed(200.0))
                    .padding([spacing::SM, spacing::MD])
                    .style(CatShieldTheme::pick_list_style),
                    Space::new().height(Length::Fixed(spacing::SM)),
                    self.view_current_scheme_indicator(),
                ]
                .spacing(spacing::XS),
            ),
            Space::new().height(Length::Fixed(spacing::XL)),
            // Cat companion settings
            self.view_setting_group(
                "Cat Companion",
                "Show an animated cat on the protection overlay",
                column![
                    checkbox(self.show_cat)
                        .label("Show animated cat")
                        .on_toggle(SettingsMessage::ShowCatToggled)
                        .text_size(typography::SIZE_BODY)
                        .style(CatShieldTheme::checkbox_style),
                    if self.show_cat {
                        column![
                            Space::new().height(Length::Fixed(spacing::MD)),
                            row![
                                text("Position:")
                                    .size(typography::SIZE_BODY_SMALL)
                                    .color(colors::TEXT_SECONDARY),
                                pick_list(
                                    CatPosition::ALL.as_slice(),
                                    Some(self.cat_position),
                                    SettingsMessage::CatPositionChanged
                                )
                                .width(Length::Fixed(150.0))
                                .padding([spacing::SM, spacing::MD])
                                .style(CatShieldTheme::pick_list_style),
                            ]
                            .spacing(spacing::MD)
                            .align_y(iced::Alignment::Center),
                            Space::new().height(Length::Fixed(spacing::SM)),
                            container(
                                row![
                                    self.cat_preview.view::<SettingsMessage>(false),
                                    Space::new().width(Length::Fixed(spacing::SM)),
                                    text("The cat will animate with bobbing and blinking")
                                        .size(typography::SIZE_CAPTION)
                                        .color(colors::TEXT_MUTED),
                                ]
                                .align_y(iced::Alignment::Center)
                            )
                            .padding([spacing::SM, spacing::MD])
                            .style(|_theme| container::Style {
                                background: Some(iced::Background::Color(
                                    colors::BACKGROUND_ELEVATED
                                )),
                                border: iced::Border {
                                    radius: borders::RADIUS_SM.into(),
                                    ..Default::default()
                                },
                                ..Default::default()
                            }),
                        ]
                    } else {
                        column![]
                    },
                ]
                .spacing(spacing::XS),
            ),
        ]
        .spacing(spacing::MD)
        .into()
    }

    /// Show indicator of current effective color scheme
    fn view_current_scheme_indicator(&self) -> Element<'_, SettingsMessage> {
        let (icon, label) = if self.color_scheme.to_color_scheme().is_dark() {
            ("🌙", "Dark mode active")
        } else {
            ("☀️", "Light mode active")
        };

        container(
            row![
                text(icon).size(typography::SIZE_BODY),
                Space::new().width(Length::Fixed(spacing::XS)),
                text(label)
                    .size(typography::SIZE_CAPTION)
                    .color(colors::TEXT_MUTED),
            ]
            .align_y(Alignment::Center),
        )
        .padding([spacing::XS, spacing::SM])
        .style(|_theme| container::Style {
            background: Some(iced::Background::Color(colors::BACKGROUND_ELEVATED)),
            border: iced::Border {
                radius: borders::RADIUS_SM.into(),
                ..Default::default()
            },
            ..Default::default()
        })
        .into()
    }

    /// Helper to render the allowed keys list
    fn view_allowed_keys_list(&self) -> Element<'_, SettingsMessage> {
        if self.allowed_keys.is_empty() {
            return container(
                text("No allowed keys configured")
                    .size(typography::SIZE_BODY_SMALL)
                    .color(colors::TEXT_MUTED),
            )
            .padding(spacing::SM)
            .into();
        }

        let items: Vec<Element<'_, SettingsMessage>> = self
            .allowed_keys
            .iter()
            .enumerate()
            .map(|(i, key)| {
                container(
                    row![
                        container(
                            text(key)
                                .size(typography::SIZE_BODY)
                                .color(colors::TEXT_PRIMARY)
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
                        Space::new().width(Length::Fill),
                        button(text("×").size(typography::SIZE_TITLE).color(colors::DANGER))
                            .padding([spacing::XS, spacing::SM])
                            .style(CatShieldTheme::ghost_button)
                            .on_press(SettingsMessage::RemoveAllowedKey(i)),
                    ]
                    .align_y(Alignment::Center),
                )
                .padding([spacing::XS, 0.0])
                .into()
            })
            .collect();

        container(column(items).spacing(spacing::XS)).into()
    }

    /// Render the behavior settings section with polished styling
    fn view_behavior_section(&self) -> Element<'_, SettingsMessage> {
        column![
            // Exit key setting
            self.view_setting_group(
                "Exit Key",
                "Key combination to unlock the screen (e.g., Cmd+Option+U)",
                text_input("Cmd+Option+U", &self.exit_key)
                    .on_input(SettingsMessage::ExitKeyChanged)
                    .padding([spacing::SM, spacing::MD])
                    .width(Length::Fixed(250.0))
                    .style(CatShieldTheme::text_input_style),
            ),
            Space::new().height(Length::Fixed(spacing::XL)),
            // Default timer setting
            self.view_setting_group(
                "Default Timer",
                "Auto-unlock after this duration when starting protection",
                column![
                    pick_list(
                        TimerPreset::ALL.as_slice(),
                        Some(self.timer_preset),
                        SettingsMessage::TimerPresetSelected
                    )
                    .width(Length::Fixed(200.0))
                    .padding([spacing::SM, spacing::MD])
                    .style(CatShieldTheme::pick_list_style),
                    if self.timer_preset == TimerPreset::Custom {
                        row![
                            text("Custom duration:")
                                .size(typography::SIZE_BODY_SMALL)
                                .color(colors::TEXT_SECONDARY),
                            text_input("e.g., 45m or 1h30m", &self.custom_timer)
                                .on_input(SettingsMessage::CustomTimerChanged)
                                .padding([spacing::SM, spacing::MD])
                                .width(Length::Fixed(150.0))
                                .style(CatShieldTheme::text_input_style),
                        ]
                        .spacing(spacing::MD)
                        .align_y(Alignment::Center)
                    } else {
                        row![]
                    },
                ]
                .spacing(spacing::MD),
            ),
            Space::new().height(Length::Fixed(spacing::XL)),
            // Allowed keys setting
            self.view_setting_group(
                "Allowed Keys",
                "Keyboard shortcuts that bypass the shield (e.g., for volume control)",
                column![
                    // List of current allowed keys with improved styling
                    self.view_allowed_keys_list(),
                    Space::new().height(Length::Fixed(spacing::SM)),
                    // Add new key input
                    row![
                        text_input("Add key (e.g., Cmd+Space)", &self.new_allowed_key_input)
                            .on_input(SettingsMessage::AllowedKeyInputChanged)
                            .on_submit(SettingsMessage::AddAllowedKey)
                            .padding([spacing::SM, spacing::MD])
                            .width(Length::Fixed(200.0))
                            .style(CatShieldTheme::text_input_style),
                        button(
                            text("Add")
                                .size(typography::SIZE_BODY_SMALL)
                                .color(colors::TEXT_ON_ACCENT)
                        )
                        .padding([spacing::SM, spacing::MD])
                        .style(CatShieldTheme::primary_button)
                        .on_press(SettingsMessage::AddAllowedKey),
                    ]
                    .spacing(spacing::SM)
                    .align_y(Alignment::Center),
                    // Preset buttons with improved styling
                    Space::new().height(Length::Fixed(spacing::SM)),
                    row![
                        text("Quick add:")
                            .size(typography::SIZE_CAPTION)
                            .color(colors::TEXT_MUTED),
                        button(
                            text("🔊 Media Keys")
                                .size(typography::SIZE_CAPTION)
                                .color(colors::ACCENT)
                        )
                        .padding([spacing::XS, spacing::SM])
                        .style(CatShieldTheme::ghost_button)
                        .on_press(SettingsMessage::AddMediaKeysPreset),
                        button(
                            text("🔍 Spotlight")
                                .size(typography::SIZE_CAPTION)
                                .color(colors::ACCENT)
                        )
                        .padding([spacing::XS, spacing::SM])
                        .style(CatShieldTheme::ghost_button)
                        .on_press(SettingsMessage::AddSpotlightPreset),
                    ]
                    .spacing(spacing::SM)
                    .align_y(Alignment::Center),
                ]
                .spacing(spacing::SM),
            ),
        ]
        .spacing(spacing::MD)
        .into()
    }

    /// Render the advanced settings section with polished styling
    fn view_advanced_section(&self) -> Element<'_, SettingsMessage> {
        column![
            // Launch at login
            self.view_setting_group(
                "Startup",
                "System startup behavior",
                checkbox(self.launch_at_login)
                    .label("Launch Cat Shield at login")
                    .on_toggle(SettingsMessage::LaunchAtLoginToggled)
                    .text_size(typography::SIZE_BODY)
                    .style(CatShieldTheme::checkbox_style),
            ),
            Space::new().height(Length::Fixed(spacing::XL)),
            // Trace logging
            self.view_setting_group(
                "Debug Logging",
                "Write detailed event logs to ~/.config/catshield/logs/",
                column![
                    checkbox(self.trace_logging)
                        .label("Enable trace logging")
                        .on_toggle(SettingsMessage::TraceLoggingToggled)
                        .text_size(typography::SIZE_BODY)
                        .style(CatShieldTheme::checkbox_style),
                    Space::new().height(Length::Fixed(spacing::SM)),
                    container(
                        row![
                            text("⚠️").size(typography::SIZE_CAPTION),
                            Space::new().width(Length::Fixed(spacing::XS)),
                            text("Logs may contain sensitive information")
                                .size(typography::SIZE_MICRO)
                                .color(colors::WARNING),
                        ]
                        .align_y(Alignment::Center)
                    )
                    .padding([spacing::XS, spacing::SM])
                    .style(|_theme| container::Style {
                        background: Some(iced::Background::Color(Color::from_rgba(
                            1.0, 0.62, 0.22, 0.1,
                        ))),
                        border: iced::Border {
                            radius: borders::RADIUS_SM.into(),
                            width: borders::WIDTH_DEFAULT,
                            color: Color::from_rgba(1.0, 0.62, 0.22, 0.3),
                        },
                        ..Default::default()
                    }),
                ]
                .spacing(spacing::XS),
            ),
            Space::new().height(Length::Fixed(spacing::XL)),
            // Reset to defaults
            self.view_setting_group(
                "Reset",
                "Restore all settings to their defaults",
                button(
                    row![
                        text("↺").size(typography::SIZE_BODY),
                        Space::new().width(Length::Fixed(spacing::XS)),
                        text("Reset to Defaults")
                            .size(typography::SIZE_BODY)
                            .color(colors::DANGER),
                    ]
                    .align_y(Alignment::Center),
                )
                .padding([spacing::SM, spacing::LG])
                .style(CatShieldTheme::danger_button)
                .on_press(SettingsMessage::ResetDefaults),
            ),
        ]
        .spacing(spacing::MD)
        .into()
    }

    /// Render the about section with polished styling
    fn view_about_section(&self) -> Element<'_, SettingsMessage> {
        column![
            // Version info with cat icon
            container(
                column![
                    text("🐱").size(48.0),
                    Space::new().height(Length::Fixed(spacing::SM)),
                    text("Cat Shield")
                        .size(typography::SIZE_HEADER)
                        .color(colors::TEXT_PRIMARY),
                    container(
                        text(format!("Version {}", env!("CARGO_PKG_VERSION")))
                            .size(typography::SIZE_CAPTION)
                            .color(colors::TEXT_SECONDARY)
                    )
                    .padding([spacing::XS, spacing::MD])
                    .style(|_theme| container::Style {
                        background: Some(iced::Background::Color(colors::BACKGROUND_ELEVATED)),
                        border: iced::Border {
                            radius: borders::RADIUS_PILL.into(),
                            ..Default::default()
                        },
                        ..Default::default()
                    }),
                ]
                .spacing(spacing::XS)
                .align_x(Alignment::Center)
            )
            .width(Length::Fill)
            .padding([spacing::SECTION_PADDING, 0.0]),
            Space::new().height(Length::Fixed(spacing::LG)),
            // Description in a subtle card
            container(
                text(
                    "A cross-platform application that creates a cat-proof \
                      screen overlay to keep your machine awake and block \
                      all input while protecting your work from curious cats."
                )
                .size(typography::SIZE_BODY)
                .color(colors::TEXT_SECONDARY)
            )
            .width(Length::Fill)
            .padding(spacing::SECTION_PADDING)
            .style(CatShieldTheme::card_container),
            Space::new().height(Length::Fixed(spacing::XL)),
            // Links section with icons
            self.view_setting_group(
                "Links",
                "Get help and contribute",
                column![
                    row![
                        text("📦").size(typography::SIZE_BODY),
                        Space::new().width(Length::Fixed(spacing::SM)),
                        text("github.com/taearls/catshield")
                            .size(typography::SIZE_BODY_SMALL)
                            .color(colors::ACCENT),
                    ]
                    .align_y(Alignment::Center),
                    row![
                        text("🐛").size(typography::SIZE_BODY),
                        Space::new().width(Length::Fixed(spacing::SM)),
                        text("github.com/taearls/catshield/issues")
                            .size(typography::SIZE_BODY_SMALL)
                            .color(colors::ACCENT),
                    ]
                    .align_y(Alignment::Center),
                ]
                .spacing(spacing::SM),
            ),
            Space::new().height(Length::Fixed(spacing::XL)),
            // Credits with tech stack badges
            self.view_setting_group(
                "Credits",
                "Built with",
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
                    container(
                        text("📜 MIT")
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
            ),
        ]
        .spacing(spacing::MD)
        .into()
    }

    /// Helper to create a consistent setting group layout with polished styling
    fn view_setting_group<'a>(
        &self,
        title: &'a str,
        description: &'a str,
        content: impl Into<Element<'a, SettingsMessage>>,
    ) -> Element<'a, SettingsMessage> {
        container(
            column![
                text(title)
                    .size(typography::SIZE_TITLE)
                    .color(colors::TEXT_PRIMARY),
                text(description)
                    .size(typography::SIZE_CAPTION)
                    .color(colors::TEXT_MUTED),
                Space::new().height(Length::Fixed(spacing::MD)),
                content.into(),
            ]
            .spacing(spacing::XS),
        )
        .width(Length::Fill)
        .padding([spacing::CARD_PADDING, spacing::SECTION_PADDING])
        .style(|_theme| container::Style {
            background: Some(iced::Background::Color(colors::BACKGROUND_SECONDARY)),
            border: iced::Border {
                radius: borders::RADIUS_MD.into(),
                width: borders::WIDTH_DEFAULT,
                color: colors::BORDER_SUBTLE,
            },
            ..Default::default()
        })
        .into()
    }

    /// Render the footer with action buttons and polished styling
    fn view_footer(&self) -> Element<'_, SettingsMessage> {
        let mut footer_row = row![].spacing(spacing::MD).align_y(Alignment::Center);

        // Error/success message with improved styling
        if let Some(ref error) = self.error_message {
            footer_row = footer_row.push(
                container(
                    row![
                        text("✕").size(typography::SIZE_BODY).color(colors::DANGER),
                        Space::new().width(Length::Fixed(spacing::XS)),
                        text(error)
                            .size(typography::SIZE_BODY_SMALL)
                            .color(colors::DANGER),
                    ]
                    .align_y(Alignment::Center),
                )
                .padding([spacing::XS, spacing::SM])
                .style(|_theme| container::Style {
                    background: Some(iced::Background::Color(Color::from_rgba(
                        0.92, 0.28, 0.28, 0.1,
                    ))),
                    border: iced::Border {
                        radius: borders::RADIUS_SM.into(),
                        ..Default::default()
                    },
                    ..Default::default()
                }),
            );
        } else if let Some(ref success) = self.success_message {
            footer_row = footer_row.push(
                container(
                    row![
                        text("✓").size(typography::SIZE_BODY).color(colors::SUCCESS),
                        Space::new().width(Length::Fixed(spacing::XS)),
                        text(success)
                            .size(typography::SIZE_BODY_SMALL)
                            .color(colors::SUCCESS),
                    ]
                    .align_y(Alignment::Center),
                )
                .padding([spacing::XS, spacing::SM])
                .style(|_theme| container::Style {
                    background: Some(iced::Background::Color(Color::from_rgba(
                        0.30, 0.72, 0.40, 0.1,
                    ))),
                    border: iced::Border {
                        radius: borders::RADIUS_SM.into(),
                        ..Default::default()
                    },
                    ..Default::default()
                }),
            );
        }

        footer_row = footer_row.push(Space::new().width(Length::Fill));

        // Cancel button
        footer_row = footer_row.push(
            button(
                text("Cancel")
                    .size(typography::SIZE_BODY)
                    .color(colors::TEXT_SECONDARY),
            )
            .padding([spacing::SM, spacing::XL])
            .style(CatShieldTheme::secondary_button)
            .on_press(SettingsMessage::Cancel),
        );

        // Apply button (only if there are changes)
        if self.has_changes {
            footer_row = footer_row.push(
                button(
                    text("Apply")
                        .size(typography::SIZE_BODY)
                        .color(colors::TEXT_ON_ACCENT),
                )
                .padding([spacing::SM, spacing::XL])
                .style(CatShieldTheme::secondary_button)
                .on_press(SettingsMessage::Apply),
            );
        }

        // Save button with appropriate label
        let save_label = if self.has_changes { "Save" } else { "OK" };
        footer_row = footer_row.push(
            button(
                text(save_label)
                    .size(typography::SIZE_BODY)
                    .color(colors::TEXT_ON_ACCENT),
            )
            .padding([spacing::SM, spacing::XL])
            .style(CatShieldTheme::primary_button)
            .on_press(SettingsMessage::Save),
        );

        container(footer_row)
            .width(Length::Fill)
            .padding([spacing::CARD_PADDING, spacing::WINDOW_PADDING])
            .into()
    }

    /// Get the theme for the settings window
    pub fn theme(&self) -> Theme {
        CatShieldTheme::for_scheme(self.color_scheme.to_color_scheme())
    }

    /// Get window settings for the settings window
    pub fn window_settings() -> WindowSettings {
        WindowSettings {
            size: Size::new(550.0, 650.0),
            position: Position::Centered,
            resizable: true,
            decorations: true,
            transparent: false,
            min_size: Some(Size::new(450.0, 500.0)),
            max_size: Some(Size::new(800.0, 900.0)),
            ..WindowSettings::default()
        }
    }

    /// Subscription for animation ticks
    ///
    /// Provides periodic ticks at ~30 FPS for smooth cat animation in the preview.
    pub fn subscription(&self) -> Subscription<SettingsMessage> {
        // Only tick when on the Overlay section where the cat preview is visible
        if self.current_section == SettingsSection::Overlay && self.show_cat {
            time::every(Duration::from_millis(33)).map(SettingsMessage::Tick)
        } else {
            Subscription::none()
        }
    }

    /// Run the settings window application
    pub fn run() -> iced::Result {
        iced::application(Self::new, Self::update, Self::view)
            .title("Cat Shield Settings")
            .subscription(Self::subscription)
            .window(Self::window_settings())
            .theme(Self::theme)
            .run()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Create a blank settings window for testing (doesn't load real config)
    fn blank_settings() -> SettingsWindow {
        SettingsWindow::from_config(Config::default())
    }

    /// Helper to apply a message in tests (discards the Task result)
    fn apply(window: &mut SettingsWindow, message: SettingsMessage) {
        let _ = window.update(message);
    }

    // ============================================================
    // SettingsWindow creation and initialization tests
    // ============================================================

    #[test]
    fn test_settings_window_default() {
        // Note: This loads from the actual config file, so we test with blank_settings instead
        let window = blank_settings();
        assert_eq!(window.opacity, DEFAULT_OVERLAY_OPACITY);
        assert_eq!(window.color_preset, OverlayColor::Gray);
        assert_eq!(window.timer_preset, TimerPreset::None);
        assert!(!window.has_changes);
        assert_eq!(window.current_section, SettingsSection::Overlay);
    }

    #[test]
    fn test_settings_window_from_config() {
        let config = Config {
            exit_key: Some("Cmd+Shift+X".to_string()),
            default_timer: Some("30m".to_string()),
            overlay_opacity: Some(0.7),
            overlay_color: Some("blue".to_string()),
            allowed_keys: Some(vec!["F11".to_string(), "F12".to_string()]),
            launch_at_login: Some(true),
            enable_trace_logging: Some(false),
            color_scheme: None,
            show_cat: None,
            cat_position: None,
        };

        let window = SettingsWindow::from_config(config);
        assert_eq!(window.exit_key, "Cmd+Shift+X");
        assert_eq!(window.timer_preset, TimerPreset::ThirtyMinutes);
        assert_eq!(window.opacity, 0.7);
        assert_eq!(window.color_preset, OverlayColor::Blue);
        assert_eq!(window.allowed_keys.len(), 2);
        assert!(window.launch_at_login);
        assert!(!window.trace_logging);
    }

    #[test]
    fn test_settings_window_empty_config() {
        let config = Config::default();
        let window = SettingsWindow::from_config(config);

        assert!(window.exit_key.is_empty());
        assert_eq!(window.timer_preset, TimerPreset::None);
        assert_eq!(window.opacity, DEFAULT_OVERLAY_OPACITY);
        assert!(window.allowed_keys.is_empty());
        assert!(!window.launch_at_login);
        assert!(!window.trace_logging);
    }

    // ============================================================
    // Overlay color tests
    // ============================================================

    #[test]
    fn test_overlay_color_to_rgb() {
        let (r, g, b) = OverlayColor::Gray.to_rgb();
        assert!((r - 0.1).abs() < f32::EPSILON);
        assert!((g - 0.1).abs() < f32::EPSILON);
        assert!((b - 0.1).abs() < f32::EPSILON);

        let (r, g, b) = OverlayColor::Blue.to_rgb();
        assert!((r - 0.05).abs() < f32::EPSILON);
        assert!((g - 0.1).abs() < f32::EPSILON);
        assert!((b - 0.2).abs() < f32::EPSILON);
    }

    #[test]
    fn test_overlay_color_display() {
        assert_eq!(format!("{}", OverlayColor::Gray), "Dark Gray");
        assert_eq!(format!("{}", OverlayColor::Blue), "Deep Blue");
        assert_eq!(format!("{}", OverlayColor::Green), "Forest Green");
        assert_eq!(format!("{}", OverlayColor::Red), "Dark Red");
        assert_eq!(format!("{}", OverlayColor::Purple), "Dark Purple");
        assert_eq!(format!("{}", OverlayColor::Custom), "Custom...");
    }

    #[test]
    fn test_overlay_color_to_config_string() {
        assert_eq!(
            OverlayColor::Gray.to_config_string(),
            Some("gray".to_string())
        );
        assert_eq!(
            OverlayColor::Blue.to_config_string(),
            Some("blue".to_string())
        );
        assert_eq!(
            OverlayColor::Green.to_config_string(),
            Some("green".to_string())
        );
        assert_eq!(
            OverlayColor::Red.to_config_string(),
            Some("red".to_string())
        );
        assert_eq!(
            OverlayColor::Purple.to_config_string(),
            Some("purple".to_string())
        );
        assert_eq!(OverlayColor::Custom.to_config_string(), None);
    }

    #[test]
    fn test_overlay_color_from_config_string() {
        assert_eq!(OverlayColor::from_config_string("gray"), OverlayColor::Gray);
        assert_eq!(OverlayColor::from_config_string("grey"), OverlayColor::Gray);
        assert_eq!(OverlayColor::from_config_string("dark"), OverlayColor::Gray);
        assert_eq!(OverlayColor::from_config_string("blue"), OverlayColor::Blue);
        assert_eq!(
            OverlayColor::from_config_string("green"),
            OverlayColor::Green
        );
        assert_eq!(OverlayColor::from_config_string("red"), OverlayColor::Red);
        assert_eq!(
            OverlayColor::from_config_string("purple"),
            OverlayColor::Purple
        );
        assert_eq!(
            OverlayColor::from_config_string("#FF0000"),
            OverlayColor::Custom
        );
    }

    #[test]
    fn test_overlay_color_from_config_string_case_insensitive() {
        assert_eq!(OverlayColor::from_config_string("BLUE"), OverlayColor::Blue);
        assert_eq!(OverlayColor::from_config_string("Blue"), OverlayColor::Blue);
    }

    #[test]
    fn test_overlay_color_all_presets() {
        // Verify ALL constant contains all expected colors
        assert_eq!(OverlayColor::ALL.len(), 6);
        assert!(OverlayColor::ALL.contains(&OverlayColor::Gray));
        assert!(OverlayColor::ALL.contains(&OverlayColor::Blue));
        assert!(OverlayColor::ALL.contains(&OverlayColor::Green));
        assert!(OverlayColor::ALL.contains(&OverlayColor::Red));
        assert!(OverlayColor::ALL.contains(&OverlayColor::Purple));
        assert!(OverlayColor::ALL.contains(&OverlayColor::Custom));
    }

    // ============================================================
    // Timer preset tests
    // ============================================================

    #[test]
    fn test_timer_preset_to_duration_string() {
        assert_eq!(
            TimerPreset::FiveMinutes.to_duration_string(),
            Some("5m".to_string())
        );
        assert_eq!(
            TimerPreset::FifteenMinutes.to_duration_string(),
            Some("15m".to_string())
        );
        assert_eq!(
            TimerPreset::ThirtyMinutes.to_duration_string(),
            Some("30m".to_string())
        );
        assert_eq!(
            TimerPreset::OneHour.to_duration_string(),
            Some("1h".to_string())
        );
        assert_eq!(
            TimerPreset::TwoHours.to_duration_string(),
            Some("2h".to_string())
        );
        assert_eq!(TimerPreset::Custom.to_duration_string(), None);
        assert_eq!(TimerPreset::None.to_duration_string(), None);
    }

    #[test]
    fn test_timer_preset_from_duration_string() {
        assert_eq!(
            TimerPreset::from_duration_string("5m"),
            TimerPreset::FiveMinutes
        );
        assert_eq!(
            TimerPreset::from_duration_string("15m"),
            TimerPreset::FifteenMinutes
        );
        assert_eq!(
            TimerPreset::from_duration_string("30m"),
            TimerPreset::ThirtyMinutes
        );
        assert_eq!(
            TimerPreset::from_duration_string("1h"),
            TimerPreset::OneHour
        );
        assert_eq!(
            TimerPreset::from_duration_string("2h"),
            TimerPreset::TwoHours
        );
        assert_eq!(
            TimerPreset::from_duration_string("45m"),
            TimerPreset::Custom
        );
        assert_eq!(
            TimerPreset::from_duration_string("weird"),
            TimerPreset::Custom
        );
    }

    #[test]
    fn test_timer_preset_display() {
        assert_eq!(format!("{}", TimerPreset::FiveMinutes), "5 minutes");
        assert_eq!(format!("{}", TimerPreset::OneHour), "1 hour");
        assert_eq!(format!("{}", TimerPreset::None), "No default");
        assert_eq!(format!("{}", TimerPreset::Custom), "Custom...");
    }

    // ============================================================
    // Config building tests
    // ============================================================

    #[test]
    fn test_build_config_basic() {
        let mut window = blank_settings();
        window.exit_key = "Cmd+Q".to_string();
        window.opacity = 0.6;
        window.timer_preset = TimerPreset::ThirtyMinutes;

        let config = window.build_config();
        assert_eq!(config.exit_key, Some("Cmd+Q".to_string()));
        assert_eq!(config.overlay_opacity, Some(0.6));
        assert_eq!(config.default_timer, Some("30m".to_string()));
    }

    #[test]
    fn test_build_config_empty_exit_key() {
        let window = blank_settings();
        let config = window.build_config();
        assert!(config.exit_key.is_none());
    }

    #[test]
    fn test_build_config_custom_timer() {
        let mut window = blank_settings();
        window.timer_preset = TimerPreset::Custom;
        window.custom_timer = "45m".to_string();

        let config = window.build_config();
        assert_eq!(config.default_timer, Some("45m".to_string()));
    }

    #[test]
    fn test_build_config_no_timer() {
        let mut window = blank_settings();
        window.timer_preset = TimerPreset::None;

        let config = window.build_config();
        assert!(config.default_timer.is_none());
    }

    #[test]
    fn test_build_config_allowed_keys() {
        let mut window = blank_settings();
        window.allowed_keys = vec!["F11".to_string(), "Cmd+Space".to_string()];

        let config = window.build_config();
        assert_eq!(
            config.allowed_keys,
            Some(vec!["F11".to_string(), "Cmd+Space".to_string()])
        );
    }

    #[test]
    fn test_build_config_empty_allowed_keys() {
        let window = blank_settings();
        let config = window.build_config();
        assert!(config.allowed_keys.is_none());
    }

    // ============================================================
    // Update message handling tests
    // ============================================================

    #[test]
    fn test_update_opacity_changed() {
        let mut window = blank_settings();
        apply(&mut window, SettingsMessage::OpacityChanged(0.7));

        assert_eq!(window.opacity, 0.7);
        assert!(window.has_changes);
    }

    #[test]
    fn test_update_opacity_clamped() {
        let mut window = blank_settings();

        // Below minimum
        apply(&mut window, SettingsMessage::OpacityChanged(0.1));
        assert_eq!(window.opacity, MIN_OVERLAY_OPACITY);

        // Above maximum
        apply(&mut window, SettingsMessage::OpacityChanged(0.95));
        assert_eq!(window.opacity, MAX_OVERLAY_OPACITY);
    }

    #[test]
    fn test_update_color_preset_selected() {
        let mut window = blank_settings();
        apply(
            &mut window,
            SettingsMessage::ColorPresetSelected(OverlayColor::Blue),
        );

        assert_eq!(window.color_preset, OverlayColor::Blue);
        assert!(window.has_changes);
    }

    #[test]
    fn test_update_color_preset_clears_hex_input() {
        let mut window = blank_settings();
        window.hex_color_input = "#FF0000".to_string();
        window.color_preset = OverlayColor::Custom;

        apply(
            &mut window,
            SettingsMessage::ColorPresetSelected(OverlayColor::Blue),
        );

        assert_eq!(window.color_preset, OverlayColor::Blue);
        assert!(window.hex_color_input.is_empty());
    }

    #[test]
    fn test_update_hex_color_changed() {
        let mut window = blank_settings();
        apply(
            &mut window,
            SettingsMessage::HexColorChanged("#FF5500".to_string()),
        );

        assert_eq!(window.hex_color_input, "#FF5500");
        assert_eq!(window.color_preset, OverlayColor::Custom);
        assert!(window.has_changes);
    }

    #[test]
    fn test_build_config_with_color_preset() {
        let mut window = blank_settings();
        window.color_preset = OverlayColor::Blue;

        let config = window.build_config();
        assert_eq!(config.overlay_color, Some("blue".to_string()));
    }

    #[test]
    fn test_build_config_with_custom_hex_color() {
        let mut window = blank_settings();
        window.color_preset = OverlayColor::Custom;
        window.hex_color_input = "#1a2b3c".to_string();

        let config = window.build_config();
        assert_eq!(config.overlay_color, Some("#1a2b3c".to_string()));
    }

    #[test]
    fn test_build_config_custom_empty_hex() {
        let mut window = blank_settings();
        window.color_preset = OverlayColor::Custom;
        window.hex_color_input = String::new();

        let config = window.build_config();
        assert!(config.overlay_color.is_none());
    }

    #[test]
    fn test_settings_loads_custom_hex_color() {
        let config = Config {
            overlay_color: Some("#ABC123".to_string()),
            ..Default::default()
        };

        let window = SettingsWindow::from_config(config);
        assert_eq!(window.color_preset, OverlayColor::Custom);
        assert_eq!(window.hex_color_input, "#ABC123");
    }

    #[test]
    fn test_update_timer_preset_selected() {
        let mut window = blank_settings();
        apply(
            &mut window,
            SettingsMessage::TimerPresetSelected(TimerPreset::OneHour),
        );

        assert_eq!(window.timer_preset, TimerPreset::OneHour);
        assert_eq!(window.custom_timer, "1h");
        assert!(window.has_changes);
    }

    #[test]
    fn test_update_custom_timer_changed() {
        let mut window = blank_settings();
        apply(
            &mut window,
            SettingsMessage::CustomTimerChanged("45m".to_string()),
        );

        assert_eq!(window.custom_timer, "45m");
        assert_eq!(window.timer_preset, TimerPreset::Custom);
        assert!(window.has_changes);
    }

    #[test]
    fn test_update_exit_key_changed() {
        let mut window = blank_settings();
        apply(
            &mut window,
            SettingsMessage::ExitKeyChanged("Cmd+Shift+Q".to_string()),
        );

        assert_eq!(window.exit_key, "Cmd+Shift+Q");
        assert!(window.has_changes);
    }

    #[test]
    fn test_update_add_allowed_key() {
        let mut window = blank_settings();
        window.new_allowed_key_input = "F11".to_string();
        apply(&mut window, SettingsMessage::AddAllowedKey);

        assert_eq!(window.allowed_keys, vec!["F11".to_string()]);
        assert!(window.new_allowed_key_input.is_empty());
        assert!(window.has_changes);
    }

    #[test]
    fn test_update_add_allowed_key_duplicate() {
        let mut window = blank_settings();
        window.allowed_keys = vec!["F11".to_string()];
        window.new_allowed_key_input = "F11".to_string();
        apply(&mut window, SettingsMessage::AddAllowedKey);

        // Should not add duplicate
        assert_eq!(window.allowed_keys.len(), 1);
    }

    #[test]
    fn test_update_add_allowed_key_empty() {
        let mut window = blank_settings();
        window.new_allowed_key_input = "   ".to_string();
        apply(&mut window, SettingsMessage::AddAllowedKey);

        // Should not add empty/whitespace-only key
        assert!(window.allowed_keys.is_empty());
    }

    #[test]
    fn test_update_remove_allowed_key() {
        let mut window = blank_settings();
        window.allowed_keys = vec!["F11".to_string(), "F12".to_string()];
        apply(&mut window, SettingsMessage::RemoveAllowedKey(0));

        assert_eq!(window.allowed_keys, vec!["F12".to_string()]);
        assert!(window.has_changes);
    }

    #[test]
    fn test_update_remove_allowed_key_invalid_index() {
        let mut window = blank_settings();
        window.allowed_keys = vec!["F11".to_string()];
        apply(&mut window, SettingsMessage::RemoveAllowedKey(5));

        // Should not panic, list unchanged
        assert_eq!(window.allowed_keys.len(), 1);
    }

    #[test]
    fn test_update_add_media_keys_preset() {
        let mut window = blank_settings();
        apply(&mut window, SettingsMessage::AddMediaKeysPreset);

        assert!(window.allowed_keys.contains(&"F11".to_string()));
        assert!(window.allowed_keys.contains(&"F12".to_string()));
        assert!(window.has_changes);
    }

    #[test]
    fn test_update_add_spotlight_preset() {
        let mut window = blank_settings();
        apply(&mut window, SettingsMessage::AddSpotlightPreset);

        assert!(window.allowed_keys.contains(&"Cmd+Space".to_string()));
        assert!(window.has_changes);
    }

    #[test]
    fn test_update_launch_at_login_toggled() {
        let mut window = blank_settings();
        apply(&mut window, SettingsMessage::LaunchAtLoginToggled(true));

        assert!(window.launch_at_login);
        assert!(window.has_changes);
    }

    #[test]
    fn test_update_trace_logging_toggled() {
        let mut window = blank_settings();
        apply(&mut window, SettingsMessage::TraceLoggingToggled(true));

        assert!(window.trace_logging);
        assert!(window.has_changes);
    }

    #[test]
    fn test_update_switch_section() {
        let mut window = blank_settings();
        apply(
            &mut window,
            SettingsMessage::SwitchSection(SettingsSection::Behavior),
        );

        assert_eq!(window.current_section, SettingsSection::Behavior);
        // Section switch doesn't create changes
        assert!(!window.has_changes);
    }

    #[test]
    fn test_update_reset_defaults() {
        let mut window = blank_settings();
        window.opacity = 0.8;
        window.exit_key = "Cmd+Q".to_string();
        window.allowed_keys = vec!["F11".to_string()];
        window.launch_at_login = true;

        apply(&mut window, SettingsMessage::ResetDefaults);

        assert_eq!(window.opacity, DEFAULT_OVERLAY_OPACITY);
        assert!(window.exit_key.is_empty());
        assert!(window.allowed_keys.is_empty());
        assert!(!window.launch_at_login);
    }

    // ============================================================
    // Window settings tests
    // ============================================================

    #[test]
    fn test_window_settings() {
        let settings = SettingsWindow::window_settings();
        assert!(settings.decorations);
        assert!(!settings.transparent);
        assert!(settings.resizable);
        assert_eq!(settings.size, Size::new(550.0, 650.0));
    }

    #[test]
    fn test_window_settings_min_size() {
        let settings = SettingsWindow::window_settings();
        assert!(settings.min_size.is_some());
        let min = settings.min_size.unwrap();
        assert_eq!(min, Size::new(450.0, 500.0));
    }

    #[test]
    fn test_window_settings_max_size() {
        let settings = SettingsWindow::window_settings();
        assert!(settings.max_size.is_some());
        let max = settings.max_size.unwrap();
        assert_eq!(max, Size::new(800.0, 900.0));
    }

    // ============================================================
    // Settings section tests
    // ============================================================

    #[test]
    fn test_settings_section_default() {
        let section = SettingsSection::default();
        assert_eq!(section, SettingsSection::Overlay);
    }

    // ============================================================
    // Message clearing tests
    // ============================================================

    #[test]
    fn test_messages_cleared_on_action() {
        let mut window = blank_settings();
        window.error_message = Some("Old error".to_string());
        window.success_message = Some("Old success".to_string());

        apply(&mut window, SettingsMessage::OpacityChanged(0.6));

        assert!(window.error_message.is_none());
        assert!(window.success_message.is_none());
    }
}
