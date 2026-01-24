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

use iced::widget::{
    button, checkbox, column, container, pick_list, row, rule, scrollable, slider, text,
    text_input, Space,
};
use iced::window::{Position, Settings as WindowSettings};
use iced::{Alignment, Color, Element, Length, Size, Task, Theme};

use crate::config::{
    Config, DEFAULT_OVERLAY_OPACITY, MAX_OVERLAY_OPACITY, MIN_OVERLAY_OPACITY,
};
use crate::ui_iced::theme::{colors, CatShieldTheme};

/// Preset overlay colors for quick selection
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OverlayColor {
    /// Dark gray (default)
    Dark,
    /// Deep blue
    Blue,
    /// Dark green
    Green,
    /// Dark purple
    Purple,
    /// Custom color (use color picker)
    Custom,
}

impl OverlayColor {
    /// Get the RGB values for this color preset
    pub fn to_rgb(&self) -> (f32, f32, f32) {
        match self {
            OverlayColor::Dark => (0.1, 0.1, 0.1),
            OverlayColor::Blue => (0.05, 0.1, 0.2),
            OverlayColor::Green => (0.05, 0.15, 0.1),
            OverlayColor::Purple => (0.12, 0.08, 0.18),
            OverlayColor::Custom => (0.1, 0.1, 0.1), // Default for custom
        }
    }

    /// All available color presets
    pub const ALL: [OverlayColor; 5] = [
        OverlayColor::Dark,
        OverlayColor::Blue,
        OverlayColor::Green,
        OverlayColor::Purple,
        OverlayColor::Custom,
    ];
}

impl std::fmt::Display for OverlayColor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OverlayColor::Dark => write!(f, "Dark Gray"),
            OverlayColor::Blue => write!(f, "Deep Blue"),
            OverlayColor::Green => write!(f, "Forest Green"),
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
    // Overlay section
    /// Opacity slider changed
    OpacityChanged(f64),
    /// Color preset selected
    ColorPresetSelected(OverlayColor),

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
    /// Overlay opacity (0.2 - 0.8)
    opacity: f64,
    /// Selected color preset
    color_preset: OverlayColor,

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

        Self {
            opacity,
            color_preset: OverlayColor::Dark, // Will add color to config later

            timer_preset,
            custom_timer,
            exit_key: config.exit_key.clone().unwrap_or_default(),

            allowed_keys: config.allowed_keys.clone().unwrap_or_default(),
            new_allowed_key_input: String::new(),

            launch_at_login: config.launch_at_login.unwrap_or(false),
            trace_logging: config.enable_trace_logging.unwrap_or(false),

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

        Config {
            exit_key: if self.exit_key.is_empty() {
                None
            } else {
                Some(self.exit_key.clone())
            },
            default_timer,
            overlay_opacity: Some(self.opacity),
            allowed_keys: if self.allowed_keys.is_empty() {
                None
            } else {
                Some(self.allowed_keys.clone())
            },
            launch_at_login: Some(self.launch_at_login),
            enable_trace_logging: Some(self.trace_logging),
        }
    }

    /// Check if settings have changed from original
    fn check_for_changes(&mut self) {
        let current = self.build_config();
        self.has_changes = current.exit_key != self.original_config.exit_key
            || current.default_timer != self.original_config.default_timer
            || current.overlay_opacity != self.original_config.overlay_opacity
            || current.allowed_keys != self.original_config.allowed_keys
            || current.launch_at_login != self.original_config.launch_at_login
            || current.enable_trace_logging != self.original_config.enable_trace_logging;
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
        self.color_preset = OverlayColor::Dark;
        self.timer_preset = TimerPreset::None;
        self.custom_timer = String::new();
        self.exit_key = String::new();
        self.allowed_keys = Vec::new();
        self.new_allowed_key_input = String::new();
        self.launch_at_login = false;
        self.trace_logging = false;
        self.check_for_changes();
    }

    /// Update settings state
    pub fn update(&mut self, message: SettingsMessage) -> Task<SettingsMessage> {
        // Clear messages on any action
        self.error_message = None;
        self.success_message = None;

        match message {
            // Overlay section
            SettingsMessage::OpacityChanged(value) => {
                self.opacity = value.clamp(MIN_OVERLAY_OPACITY, MAX_OVERLAY_OPACITY);
                self.check_for_changes();
            }
            SettingsMessage::ColorPresetSelected(color) => {
                self.color_preset = color;
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

            // Navigation
            SettingsMessage::SwitchSection(section) => {
                self.current_section = section;
            }

            // Actions
            SettingsMessage::Save => {
                match self.save_settings() {
                    Ok(()) => {
                        self.success_message = Some("Settings saved".to_string());
                        return iced::exit();
                    }
                    Err(e) => {
                        self.error_message = Some(format!("Failed to save: {e}"));
                    }
                }
            }
            SettingsMessage::Apply => {
                match self.save_settings() {
                    Ok(()) => {
                        self.success_message = Some("Settings applied".to_string());
                    }
                    Err(e) => {
                        self.error_message = Some(format!("Failed to apply: {e}"));
                    }
                }
            }
            SettingsMessage::Cancel | SettingsMessage::CloseRequested => {
                return iced::exit();
            }
            SettingsMessage::ResetDefaults => {
                self.reset_to_defaults();
            }
        }

        Task::none()
    }

    /// Render the settings view
    pub fn view(&self) -> Element<'_, SettingsMessage> {
        let content = column![
            // Header
            self.view_header(),
            rule::horizontal(1),
            // Navigation tabs
            self.view_tabs(),
            rule::horizontal(1),
            // Main content area (scrollable)
            scrollable(self.view_current_section())
                .height(Length::Fill)
                .width(Length::Fill),
            // Footer with action buttons
            rule::horizontal(1),
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

    /// Render the header
    fn view_header(&self) -> Element<'_, SettingsMessage> {
        let title = text("Cat Shield Settings")
            .size(24)
            .color(colors::TEXT_PRIMARY);

        let subtitle = if self.has_changes {
            text("Unsaved changes")
                .size(12)
                .color(colors::TIMER_WARNING)
        } else {
            text("Configure your protection settings")
                .size(12)
                .color(colors::TEXT_SECONDARY)
        };

        container(
            column![title, subtitle]
                .spacing(4)
                .align_x(Alignment::Center),
        )
        .width(Length::Fill)
        .padding([20, 30])
        .into()
    }

    /// Render the navigation tabs
    fn view_tabs(&self) -> Element<'_, SettingsMessage> {
        let make_tab = |section: SettingsSection, label: &'static str, current: SettingsSection| {
            let is_selected = current == section;
            let style = if is_selected {
                CatShieldTheme::primary_button
            } else {
                CatShieldTheme::secondary_button
            };

            button(
                text(label)
                    .size(14)
                    .color(if is_selected {
                        colors::TEXT_PRIMARY
                    } else {
                        colors::TEXT_SECONDARY
                    }),
            )
            .padding([8, 16])
            .style(style)
            .on_press(SettingsMessage::SwitchSection(section))
        };

        let current = self.current_section;

        container(
            row![
                make_tab(SettingsSection::Overlay, "Overlay", current),
                make_tab(SettingsSection::Behavior, "Behavior", current),
                make_tab(SettingsSection::Advanced, "Advanced", current),
                make_tab(SettingsSection::About, "About", current),
            ]
            .spacing(8)
            .align_y(Alignment::Center),
        )
        .width(Length::Fill)
        .padding([12, 30])
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
            .padding([20, 30])
            .into()
    }

    /// Render the overlay settings section
    fn view_overlay_section(&self) -> Element<'_, SettingsMessage> {
        let opacity_percent = (self.opacity * 100.0).round() as i32;

        // Opacity preview box
        let preview_color = Color::from_rgba(
            0.1,
            0.1,
            0.1,
            self.opacity as f32,
        );

        column![
            // Opacity setting
            self.view_setting_group(
                "Opacity",
                "How dark the overlay appears",
                column![
                    row![
                        text("Transparency")
                            .size(14)
                            .color(colors::TEXT_SECONDARY),
                        Space::new().width(Length::Fill),
                        text(format!("{}%", opacity_percent))
                            .size(14)
                            .color(colors::TEXT_PRIMARY),
                    ]
                    .align_y(Alignment::Center),
                    slider(
                        MIN_OVERLAY_OPACITY..=MAX_OVERLAY_OPACITY,
                        self.opacity,
                        SettingsMessage::OpacityChanged
                    )
                    .step(0.05),
                    row![
                        text("More transparent")
                            .size(11)
                            .color(colors::TEXT_MUTED),
                        Space::new().width(Length::Fill),
                        text("More opaque")
                            .size(11)
                            .color(colors::TEXT_MUTED),
                    ],
                    // Preview
                    Space::new().height(Length::Fixed(15.0)),
                    text("Preview").size(12).color(colors::TEXT_SECONDARY),
                    container(
                        text("Cat Shield Active")
                            .size(16)
                            .color(colors::TEXT_PRIMARY)
                    )
                    .width(Length::Fill)
                    .height(Length::Fixed(80.0))
                    .padding(20)
                    .style(move |_theme| container::Style {
                        background: Some(iced::Background::Color(preview_color)),
                        border: iced::Border {
                            radius: 8.0.into(),
                            width: 1.0,
                            color: colors::TEXT_MUTED,
                        },
                        ..Default::default()
                    }),
                ]
                .spacing(8),
            ),
            Space::new().height(Length::Fixed(20.0)),
            // Color preset setting
            self.view_setting_group(
                "Color Theme",
                "Choose a color for the overlay background",
                pick_list(
                    OverlayColor::ALL.as_slice(),
                    Some(self.color_preset),
                    SettingsMessage::ColorPresetSelected
                )
                .width(Length::Fixed(200.0))
                .padding([8, 12]),
            ),
        ]
        .spacing(10)
        .into()
    }

    /// Render the behavior settings section
    fn view_behavior_section(&self) -> Element<'_, SettingsMessage> {
        column![
            // Exit key setting
            self.view_setting_group(
                "Exit Key",
                "Key combination to unlock the screen (e.g., Cmd+Option+U)",
                text_input("Cmd+Option+U", &self.exit_key)
                    .on_input(SettingsMessage::ExitKeyChanged)
                    .padding([10, 12])
                    .width(Length::Fixed(250.0)),
            ),
            Space::new().height(Length::Fixed(20.0)),
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
                    .padding([8, 12]),
                    if self.timer_preset == TimerPreset::Custom {
                        row![
                            text("Custom duration:")
                                .size(13)
                                .color(colors::TEXT_SECONDARY),
                            text_input("e.g., 45m or 1h30m", &self.custom_timer)
                                .on_input(SettingsMessage::CustomTimerChanged)
                                .padding([8, 10])
                                .width(Length::Fixed(150.0)),
                        ]
                        .spacing(10)
                        .align_y(Alignment::Center)
                    } else {
                        row![]
                    },
                ]
                .spacing(10),
            ),
            Space::new().height(Length::Fixed(20.0)),
            // Allowed keys setting
            self.view_setting_group(
                "Allowed Keys",
                "Keyboard shortcuts that bypass the shield (e.g., for volume control)",
                column![
                    // List of current allowed keys
                    column(
                        self.allowed_keys
                            .iter()
                            .enumerate()
                            .map(|(i, key)| {
                                row![
                                    text(key)
                                        .size(14)
                                        .color(colors::TEXT_PRIMARY),
                                    Space::new().width(Length::Fill),
                                    button(
                                        text("Remove")
                                            .size(12)
                                            .color(colors::CLOSE_BUTTON)
                                    )
                                    .padding([4, 8])
                                    .style(CatShieldTheme::ghost_button)
                                    .on_press(SettingsMessage::RemoveAllowedKey(i)),
                                ]
                                .spacing(8)
                                .align_y(Alignment::Center)
                                .padding([4, 8])
                                .into()
                            })
                            .collect::<Vec<_>>()
                    )
                    .spacing(4),
                    // Add new key input
                    row![
                        text_input("Add key (e.g., Cmd+Space)", &self.new_allowed_key_input)
                            .on_input(SettingsMessage::AllowedKeyInputChanged)
                            .on_submit(SettingsMessage::AddAllowedKey)
                            .padding([8, 10])
                            .width(Length::Fixed(200.0)),
                        button(text("Add").size(13).color(colors::TEXT_PRIMARY))
                            .padding([8, 12])
                            .style(CatShieldTheme::primary_button)
                            .on_press(SettingsMessage::AddAllowedKey),
                    ]
                    .spacing(8)
                    .align_y(Alignment::Center),
                    // Preset buttons
                    row![
                        text("Presets:").size(12).color(colors::TEXT_MUTED),
                        button(text("Media Keys").size(12).color(colors::ACCENT))
                            .padding([4, 8])
                            .style(CatShieldTheme::ghost_button)
                            .on_press(SettingsMessage::AddMediaKeysPreset),
                        button(text("Spotlight").size(12).color(colors::ACCENT))
                            .padding([4, 8])
                            .style(CatShieldTheme::ghost_button)
                            .on_press(SettingsMessage::AddSpotlightPreset),
                    ]
                    .spacing(8)
                    .align_y(Alignment::Center),
                ]
                .spacing(12),
            ),
        ]
        .spacing(10)
        .into()
    }

    /// Render the advanced settings section
    fn view_advanced_section(&self) -> Element<'_, SettingsMessage> {
        column![
            // Launch at login
            self.view_setting_group(
                "Startup",
                "System startup behavior",
                checkbox(self.launch_at_login)
                    .label("Launch Cat Shield at login")
                    .on_toggle(SettingsMessage::LaunchAtLoginToggled)
                    .text_size(14),
            ),
            Space::new().height(Length::Fixed(20.0)),
            // Trace logging
            self.view_setting_group(
                "Debug Logging",
                "Write detailed event logs to ~/.config/catshield/logs/",
                column![
                    checkbox(self.trace_logging)
                        .label("Enable trace logging")
                        .on_toggle(SettingsMessage::TraceLoggingToggled)
                        .text_size(14),
                    text("Warning: Logs may contain sensitive information")
                        .size(11)
                        .color(colors::TIMER_WARNING),
                ]
                .spacing(6),
            ),
            Space::new().height(Length::Fixed(20.0)),
            // Reset to defaults
            self.view_setting_group(
                "Reset",
                "Restore all settings to their defaults",
                button(
                    text("Reset to Defaults")
                        .size(14)
                        .color(colors::CLOSE_BUTTON)
                )
                .padding([10, 16])
                .style(CatShieldTheme::danger_button)
                .on_press(SettingsMessage::ResetDefaults),
            ),
        ]
        .spacing(10)
        .into()
    }

    /// Render the about section
    fn view_about_section(&self) -> Element<'_, SettingsMessage> {
        column![
            // Version info
            container(
                column![
                    text("Cat Shield")
                        .size(28)
                        .color(colors::TEXT_PRIMARY),
                    text(format!("Version {}", env!("CARGO_PKG_VERSION")))
                        .size(14)
                        .color(colors::TEXT_SECONDARY),
                ]
                .spacing(4)
                .align_x(Alignment::Center)
            )
            .width(Length::Fill)
            .padding([20, 0]),
            Space::new().height(Length::Fixed(20.0)),
            // Description
            container(
                text("A cross-platform application that creates a cat-proof \
                      screen overlay to keep your machine awake and block \
                      all input while protecting your work from curious cats.")
                    .size(14)
                    .color(colors::TEXT_SECONDARY)
            )
            .width(Length::Fill)
            .padding([0, 20]),
            Space::new().height(Length::Fixed(30.0)),
            // Links section
            self.view_setting_group(
                "Links",
                "Get help and contribute",
                column![
                    text("GitHub: github.com/your-username/catshield")
                        .size(13)
                        .color(colors::ACCENT),
                    text("Documentation: catshield.dev")
                        .size(13)
                        .color(colors::ACCENT),
                    text("Report Issues: github.com/your-username/catshield/issues")
                        .size(13)
                        .color(colors::ACCENT),
                ]
                .spacing(8),
            ),
            Space::new().height(Length::Fixed(20.0)),
            // Credits
            self.view_setting_group(
                "Credits",
                "Built with",
                column![
                    text("Rust + iced framework")
                        .size(13)
                        .color(colors::TEXT_SECONDARY),
                    text("MIT License")
                        .size(13)
                        .color(colors::TEXT_SECONDARY),
                ]
                .spacing(4),
            ),
        ]
        .spacing(10)
        .into()
    }

    /// Helper to create a consistent setting group layout
    fn view_setting_group<'a>(
        &self,
        title: &'a str,
        description: &'a str,
        content: impl Into<Element<'a, SettingsMessage>>,
    ) -> Element<'a, SettingsMessage> {
        container(
            column![
                text(title).size(16).color(colors::TEXT_PRIMARY),
                text(description).size(12).color(colors::TEXT_MUTED),
                Space::new().height(Length::Fixed(10.0)),
                content.into(),
            ]
            .spacing(4),
        )
        .width(Length::Fill)
        .padding([15, 20])
        .style(|_theme| container::Style {
            background: Some(iced::Background::Color(Color::from_rgba(1.0, 1.0, 1.0, 0.03))),
            border: iced::Border {
                radius: 8.0.into(),
                ..Default::default()
            },
            ..Default::default()
        })
        .into()
    }

    /// Render the footer with action buttons
    fn view_footer(&self) -> Element<'_, SettingsMessage> {
        let mut footer_row = row![].spacing(12).align_y(Alignment::Center);

        // Error/success message
        if let Some(ref error) = self.error_message {
            footer_row = footer_row.push(
                text(error)
                    .size(13)
                    .color(colors::CLOSE_BUTTON),
            );
        } else if let Some(ref success) = self.success_message {
            footer_row = footer_row.push(
                text(success)
                    .size(13)
                    .color(colors::PROGRESS_FILL),
            );
        }

        footer_row = footer_row.push(Space::new().width(Length::Fill));

        // Cancel button
        footer_row = footer_row.push(
            button(text("Cancel").size(14).color(colors::TEXT_SECONDARY))
                .padding([10, 20])
                .style(CatShieldTheme::secondary_button)
                .on_press(SettingsMessage::Cancel),
        );

        // Apply button (only if there are changes)
        if self.has_changes {
            footer_row = footer_row.push(
                button(text("Apply").size(14).color(colors::TEXT_PRIMARY))
                    .padding([10, 20])
                    .style(CatShieldTheme::primary_button)
                    .on_press(SettingsMessage::Apply),
            );
        }

        // Save button
        footer_row = footer_row.push(
            button(
                text(if self.has_changes { "Save" } else { "OK" })
                    .size(14)
                    .color(colors::TEXT_PRIMARY),
            )
            .padding([10, 20])
            .style(CatShieldTheme::primary_button)
            .on_press(SettingsMessage::Save),
        );

        container(footer_row)
            .width(Length::Fill)
            .padding([15, 30])
            .into()
    }

    /// Get the theme for the settings window
    pub fn theme(&self) -> Theme {
        CatShieldTheme::base()
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

    /// Run the settings window application
    pub fn run() -> iced::Result {
        iced::application(Self::new, Self::update, Self::view)
            .title("Cat Shield Settings")
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

    // ============================================================
    // SettingsWindow creation and initialization tests
    // ============================================================

    #[test]
    fn test_settings_window_default() {
        // Note: This loads from the actual config file, so we test with blank_settings instead
        let window = blank_settings();
        assert_eq!(window.opacity, DEFAULT_OVERLAY_OPACITY);
        assert_eq!(window.color_preset, OverlayColor::Dark);
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
            allowed_keys: Some(vec!["F11".to_string(), "F12".to_string()]),
            launch_at_login: Some(true),
            enable_trace_logging: Some(false),
        };

        let window = SettingsWindow::from_config(config);
        assert_eq!(window.exit_key, "Cmd+Shift+X");
        assert_eq!(window.timer_preset, TimerPreset::ThirtyMinutes);
        assert_eq!(window.opacity, 0.7);
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
        let (r, g, b) = OverlayColor::Dark.to_rgb();
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
        assert_eq!(format!("{}", OverlayColor::Dark), "Dark Gray");
        assert_eq!(format!("{}", OverlayColor::Blue), "Deep Blue");
        assert_eq!(format!("{}", OverlayColor::Green), "Forest Green");
        assert_eq!(format!("{}", OverlayColor::Purple), "Dark Purple");
        assert_eq!(format!("{}", OverlayColor::Custom), "Custom...");
    }

    // ============================================================
    // Timer preset tests
    // ============================================================

    #[test]
    fn test_timer_preset_to_duration_string() {
        assert_eq!(TimerPreset::FiveMinutes.to_duration_string(), Some("5m".to_string()));
        assert_eq!(TimerPreset::FifteenMinutes.to_duration_string(), Some("15m".to_string()));
        assert_eq!(TimerPreset::ThirtyMinutes.to_duration_string(), Some("30m".to_string()));
        assert_eq!(TimerPreset::OneHour.to_duration_string(), Some("1h".to_string()));
        assert_eq!(TimerPreset::TwoHours.to_duration_string(), Some("2h".to_string()));
        assert_eq!(TimerPreset::Custom.to_duration_string(), None);
        assert_eq!(TimerPreset::None.to_duration_string(), None);
    }

    #[test]
    fn test_timer_preset_from_duration_string() {
        assert_eq!(TimerPreset::from_duration_string("5m"), TimerPreset::FiveMinutes);
        assert_eq!(TimerPreset::from_duration_string("15m"), TimerPreset::FifteenMinutes);
        assert_eq!(TimerPreset::from_duration_string("30m"), TimerPreset::ThirtyMinutes);
        assert_eq!(TimerPreset::from_duration_string("1h"), TimerPreset::OneHour);
        assert_eq!(TimerPreset::from_duration_string("2h"), TimerPreset::TwoHours);
        assert_eq!(TimerPreset::from_duration_string("45m"), TimerPreset::Custom);
        assert_eq!(TimerPreset::from_duration_string("weird"), TimerPreset::Custom);
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
        window.update(SettingsMessage::OpacityChanged(0.7));

        assert_eq!(window.opacity, 0.7);
        assert!(window.has_changes);
    }

    #[test]
    fn test_update_opacity_clamped() {
        let mut window = blank_settings();

        // Below minimum
        window.update(SettingsMessage::OpacityChanged(0.1));
        assert_eq!(window.opacity, MIN_OVERLAY_OPACITY);

        // Above maximum
        window.update(SettingsMessage::OpacityChanged(0.95));
        assert_eq!(window.opacity, MAX_OVERLAY_OPACITY);
    }

    #[test]
    fn test_update_color_preset_selected() {
        let mut window = blank_settings();
        window.update(SettingsMessage::ColorPresetSelected(OverlayColor::Blue));

        assert_eq!(window.color_preset, OverlayColor::Blue);
    }

    #[test]
    fn test_update_timer_preset_selected() {
        let mut window = blank_settings();
        window.update(SettingsMessage::TimerPresetSelected(TimerPreset::OneHour));

        assert_eq!(window.timer_preset, TimerPreset::OneHour);
        assert_eq!(window.custom_timer, "1h");
        assert!(window.has_changes);
    }

    #[test]
    fn test_update_custom_timer_changed() {
        let mut window = blank_settings();
        window.update(SettingsMessage::CustomTimerChanged("45m".to_string()));

        assert_eq!(window.custom_timer, "45m");
        assert_eq!(window.timer_preset, TimerPreset::Custom);
        assert!(window.has_changes);
    }

    #[test]
    fn test_update_exit_key_changed() {
        let mut window = blank_settings();
        window.update(SettingsMessage::ExitKeyChanged("Cmd+Shift+Q".to_string()));

        assert_eq!(window.exit_key, "Cmd+Shift+Q");
        assert!(window.has_changes);
    }

    #[test]
    fn test_update_add_allowed_key() {
        let mut window = blank_settings();
        window.new_allowed_key_input = "F11".to_string();
        window.update(SettingsMessage::AddAllowedKey);

        assert_eq!(window.allowed_keys, vec!["F11".to_string()]);
        assert!(window.new_allowed_key_input.is_empty());
        assert!(window.has_changes);
    }

    #[test]
    fn test_update_add_allowed_key_duplicate() {
        let mut window = blank_settings();
        window.allowed_keys = vec!["F11".to_string()];
        window.new_allowed_key_input = "F11".to_string();
        window.update(SettingsMessage::AddAllowedKey);

        // Should not add duplicate
        assert_eq!(window.allowed_keys.len(), 1);
    }

    #[test]
    fn test_update_add_allowed_key_empty() {
        let mut window = blank_settings();
        window.new_allowed_key_input = "   ".to_string();
        window.update(SettingsMessage::AddAllowedKey);

        // Should not add empty/whitespace-only key
        assert!(window.allowed_keys.is_empty());
    }

    #[test]
    fn test_update_remove_allowed_key() {
        let mut window = blank_settings();
        window.allowed_keys = vec!["F11".to_string(), "F12".to_string()];
        window.update(SettingsMessage::RemoveAllowedKey(0));

        assert_eq!(window.allowed_keys, vec!["F12".to_string()]);
        assert!(window.has_changes);
    }

    #[test]
    fn test_update_remove_allowed_key_invalid_index() {
        let mut window = blank_settings();
        window.allowed_keys = vec!["F11".to_string()];
        window.update(SettingsMessage::RemoveAllowedKey(5));

        // Should not panic, list unchanged
        assert_eq!(window.allowed_keys.len(), 1);
    }

    #[test]
    fn test_update_add_media_keys_preset() {
        let mut window = blank_settings();
        window.update(SettingsMessage::AddMediaKeysPreset);

        assert!(window.allowed_keys.contains(&"F11".to_string()));
        assert!(window.allowed_keys.contains(&"F12".to_string()));
        assert!(window.has_changes);
    }

    #[test]
    fn test_update_add_spotlight_preset() {
        let mut window = blank_settings();
        window.update(SettingsMessage::AddSpotlightPreset);

        assert!(window.allowed_keys.contains(&"Cmd+Space".to_string()));
        assert!(window.has_changes);
    }

    #[test]
    fn test_update_launch_at_login_toggled() {
        let mut window = blank_settings();
        window.update(SettingsMessage::LaunchAtLoginToggled(true));

        assert!(window.launch_at_login);
        assert!(window.has_changes);
    }

    #[test]
    fn test_update_trace_logging_toggled() {
        let mut window = blank_settings();
        window.update(SettingsMessage::TraceLoggingToggled(true));

        assert!(window.trace_logging);
        assert!(window.has_changes);
    }

    #[test]
    fn test_update_switch_section() {
        let mut window = blank_settings();
        window.update(SettingsMessage::SwitchSection(SettingsSection::Behavior));

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

        window.update(SettingsMessage::ResetDefaults);

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

        window.update(SettingsMessage::OpacityChanged(0.6));

        assert!(window.error_message.is_none());
        assert!(window.success_message.is_none());
    }
}
