//! Shield overlay window using iced
//!
//! Provides a fullscreen, semi-transparent overlay that displays:
//! - Cat icon with status indicator
//! - Timer countdown (when active) - large, centered, high contrast
//! - Optional progress bar showing time remaining
//! - Close button (hold to exit)
//! - Exit key hint
//!
//! This module handles only the visual rendering. Input blocking is handled
//! by the platform-specific implementations in `src/platform/`.
//!
//! # Timer Display
//!
//! When the `--timer` flag is used, the overlay shows a prominent countdown.
//! The format adapts based on duration:
//! - Under 1 hour: `MM:SS` (e.g., "05:30")
//! - 1 hour or more: `HH:MM:SS` (e.g., "01:30:00")
//!
//! The timer can be hidden with the `--hide-timer` flag while still maintaining
//! the auto-exit functionality.
//!
//! # Multi-Monitor Support
//!
//! iced's `Mode::Fullscreen` covers only the current monitor. For multi-monitor
//! setups, the application should spawn additional overlay windows on each display.
//! This is handled at the platform integration level (see main.rs).

use iced::event::Event;
use iced::keyboard;
use iced::widget::{button, center, column, container, row, text, Space};
use iced::window::{self, Level, Position, Settings as WindowSettings};
use iced::{time, Color, Element, Length, Size, Subscription, Task, Theme};
use std::time::{Duration, Instant};

use crate::ui_iced::theme::{borders, colors, spacing, typography, CatShieldTheme};

/// Create a horizontal progress bar widget with polished styling
///
/// # Arguments
/// * `progress` - Value from 0.0 (empty) to 1.0 (full)
/// * `is_warning` - If true, use warning color for the fill
fn progress_bar<'a>(progress: f32, is_warning: bool) -> Element<'a, OverlayMessage> {
    let bar_width: f32 = 320.0;
    let bar_height: f32 = 6.0;

    let fill_color = if is_warning {
        colors::PROGRESS_WARNING
    } else {
        colors::PROGRESS_FILL
    };

    // Create the background bar with subtle styling
    let background = container(
        Space::new()
            .width(Length::Fixed(bar_width))
            .height(Length::Fixed(bar_height)),
    )
    .style(move |_theme| container::Style {
        background: Some(iced::Background::Color(colors::PROGRESS_BACKGROUND)),
        border: iced::Border {
            radius: (bar_height / 2.0).into(),
            ..Default::default()
        },
        ..Default::default()
    });

    // Create the fill bar with smooth transition
    let fill_width = bar_width * progress.clamp(0.0, 1.0);
    let fill = container(
        Space::new()
            .width(Length::Fixed(fill_width))
            .height(Length::Fixed(bar_height)),
    )
    .style(move |_theme| container::Style {
        background: Some(iced::Background::Color(fill_color)),
        border: iced::Border {
            radius: (bar_height / 2.0).into(),
            ..Default::default()
        },
        ..Default::default()
    });

    // Stack fill on top of background using iced's stack widget
    use iced::widget::stack;
    stack![background, fill].into()
}

/// Create a cat icon with status indicator
fn cat_status_icon<'a>(is_warning: bool) -> Element<'a, OverlayMessage> {
    let icon_color = if is_warning {
        colors::WARNING
    } else {
        colors::ACCENT
    };

    container(text("🐱").size(72.0).color(icon_color)).into()
}

/// Create a status badge showing protection state
fn status_badge<'a>() -> Element<'a, OverlayMessage> {
    let badge_background = Color::from_rgba(0.3, 0.72, 0.4, 0.25);
    let badge_border = colors::SUCCESS;

    container(
        row![
            text("●")
                .size(typography::SIZE_CAPTION)
                .color(colors::SUCCESS),
            Space::new().width(Length::Fixed(spacing::XS)),
            text("Protected")
                .size(typography::SIZE_CAPTION)
                .color(colors::SUCCESS),
        ]
        .align_y(iced::Alignment::Center),
    )
    .padding([spacing::XS, spacing::MD])
    .style(move |_theme| container::Style {
        background: Some(iced::Background::Color(badge_background)),
        border: iced::Border {
            radius: borders::RADIUS_PILL.into(),
            width: borders::WIDTH_DEFAULT,
            color: badge_border,
        },
        ..Default::default()
    })
    .into()
}

/// Messages that can be sent to the overlay application
#[derive(Debug, Clone)]
pub enum OverlayMessage {
    /// Timer tick for updating elapsed/remaining time
    Tick(Instant),
    /// Request to close the overlay (from close button or exit key)
    RequestClose,
    /// Update the remaining time (set externally by timer system)
    SetRemainingSeconds(u64),
    /// Shield was deactivated externally
    ShieldDeactivated,
    /// Keyboard event received
    KeyPressed(keyboard::Key, keyboard::Modifiers),
    /// Window event received (for lifecycle management)
    WindowEvent(window::Event),
}

/// Exit key configuration for keyboard shortcut detection
#[derive(Debug, Clone)]
pub struct ExitKeyConfig {
    /// The key that triggers exit (e.g., 'U')
    pub key: keyboard::Key,
    /// Required modifiers (e.g., Cmd+Option on macOS)
    pub modifiers: keyboard::Modifiers,
    /// Display string for UI (e.g., "Cmd+Option+U")
    pub display: String,
}

impl Default for ExitKeyConfig {
    fn default() -> Self {
        Self {
            // Default: U key
            key: keyboard::Key::Character("u".into()),
            // Default modifiers: Logo (Cmd on macOS, Win on Windows) + Alt
            modifiers: keyboard::Modifiers::LOGO.union(keyboard::Modifiers::ALT),
            display: "Cmd+Option+U".to_string(),
        }
    }
}

/// State for the shield overlay application
pub struct OverlayApp {
    /// When the overlay was activated
    start_time: Instant,
    /// Elapsed seconds since activation
    elapsed_seconds: u64,
    /// Remaining seconds (if timer is set), None means no timer
    remaining_seconds: Option<u64>,
    /// Initial timer duration (for progress bar calculation)
    initial_duration: Option<u64>,
    /// Whether the shield is currently active
    is_active: bool,
    /// Exit key display string (e.g., "Cmd+Option+U")
    exit_key_display: String,
    /// Overlay opacity (0.0 - 1.0)
    opacity: f64,
    /// Overlay background color RGB (each component 0.0 - 1.0)
    color: (f32, f32, f32),
    /// Exit key configuration for detecting unlock shortcut
    exit_key_config: Option<ExitKeyConfig>,
    /// Whether to hide the timer display (--hide-timer flag)
    hide_timer: bool,
    /// Whether to show progress bar
    show_progress: bool,
}

/// Default overlay color (dark gray)
const DEFAULT_OVERLAY_COLOR: (f32, f32, f32) = (0.1, 0.1, 0.1);

impl Default for OverlayApp {
    fn default() -> Self {
        Self {
            start_time: Instant::now(),
            elapsed_seconds: 0,
            remaining_seconds: None,
            initial_duration: None,
            is_active: true,
            exit_key_display: String::new(),
            opacity: crate::config::DEFAULT_OVERLAY_OPACITY,
            color: DEFAULT_OVERLAY_COLOR,
            exit_key_config: None,
            hide_timer: false,
            show_progress: true,
        }
    }
}

/// Configuration for the overlay display
#[derive(Debug, Clone)]
pub struct OverlayConfig {
    /// Timer duration in seconds (None = no timer, show elapsed time)
    pub duration: Option<u64>,
    /// Whether to hide the timer display
    pub hide_timer: bool,
    /// Whether to show the progress bar
    pub show_progress: bool,
    /// Overlay opacity (0.0 - 1.0)
    pub opacity: f64,
    /// Overlay background color RGB (each component 0.0 - 1.0)
    pub color: (f32, f32, f32),
}

impl Default for OverlayConfig {
    fn default() -> Self {
        Self {
            duration: None,
            hide_timer: false,
            show_progress: true,
            opacity: crate::config::DEFAULT_OVERLAY_OPACITY,
            color: DEFAULT_OVERLAY_COLOR,
        }
    }
}

/// Configuration for the overlay timer display (legacy, for backward compatibility)
#[derive(Debug, Clone, Default)]
pub struct TimerConfig {
    /// Timer duration in seconds (None = no timer, show elapsed time)
    pub duration: Option<u64>,
    /// Whether to hide the timer display
    pub hide_timer: bool,
    /// Whether to show the progress bar
    pub show_progress: bool,
}

impl OverlayApp {
    /// Create a new overlay app with configuration
    pub fn new(exit_key_display: String, timer_seconds: Option<u64>, opacity: f64) -> Self {
        Self {
            start_time: Instant::now(),
            elapsed_seconds: 0,
            remaining_seconds: timer_seconds,
            initial_duration: timer_seconds,
            is_active: true,
            exit_key_display,
            opacity,
            color: DEFAULT_OVERLAY_COLOR,
            exit_key_config: None,
            hide_timer: false,
            show_progress: true,
        }
    }

    /// Create a new overlay app with full configuration including exit key
    pub fn with_exit_key(
        exit_key_config: ExitKeyConfig,
        timer_seconds: Option<u64>,
        opacity: f64,
    ) -> Self {
        Self {
            start_time: Instant::now(),
            elapsed_seconds: 0,
            remaining_seconds: timer_seconds,
            initial_duration: timer_seconds,
            is_active: true,
            exit_key_display: exit_key_config.display.clone(),
            opacity,
            color: DEFAULT_OVERLAY_COLOR,
            exit_key_config: Some(exit_key_config),
            hide_timer: false,
            show_progress: true,
        }
    }

    /// Create a new overlay app with full timer configuration (legacy)
    pub fn with_timer_config(
        exit_key_config: ExitKeyConfig,
        timer_config: TimerConfig,
        opacity: f64,
    ) -> Self {
        Self {
            start_time: Instant::now(),
            elapsed_seconds: 0,
            remaining_seconds: timer_config.duration,
            initial_duration: timer_config.duration,
            is_active: true,
            exit_key_display: exit_key_config.display.clone(),
            opacity,
            color: DEFAULT_OVERLAY_COLOR,
            exit_key_config: Some(exit_key_config),
            hide_timer: timer_config.hide_timer,
            show_progress: timer_config.show_progress,
        }
    }

    /// Create a new overlay app with full overlay configuration including color
    pub fn with_overlay_config(exit_key_config: ExitKeyConfig, config: OverlayConfig) -> Self {
        Self {
            start_time: Instant::now(),
            elapsed_seconds: 0,
            remaining_seconds: config.duration,
            initial_duration: config.duration,
            is_active: true,
            exit_key_display: exit_key_config.display.clone(),
            opacity: config.opacity,
            color: config.color,
            exit_key_config: Some(exit_key_config),
            hide_timer: config.hide_timer,
            show_progress: config.show_progress,
        }
    }

    /// Check if a key press matches the exit key configuration
    fn is_exit_key(&self, key: &keyboard::Key, modifiers: keyboard::Modifiers) -> bool {
        if let Some(ref config) = self.exit_key_config {
            // Check if modifiers match
            let modifiers_match = modifiers.contains(config.modifiers);

            // Check if key matches (case-insensitive for characters)
            let key_match = match (&config.key, key) {
                (keyboard::Key::Character(a), keyboard::Key::Character(b)) => {
                    a.to_lowercase() == b.to_lowercase()
                }
                (a, b) => a == b,
            };

            modifiers_match && key_match
        } else {
            false
        }
    }

    /// Update the application state in response to a message
    pub fn update(&mut self, message: OverlayMessage) -> Task<OverlayMessage> {
        match message {
            OverlayMessage::Tick(now) => {
                self.elapsed_seconds = now.duration_since(self.start_time).as_secs();

                // Update remaining time if timer is set
                if let Some(ref mut remaining) = self.remaining_seconds {
                    if *remaining > 0 {
                        *remaining = remaining.saturating_sub(1);
                    }
                    // Note: Auto-exit when remaining reaches 0 is handled externally
                    // by the timer system, not here
                }

                Task::none()
            }
            OverlayMessage::RequestClose => {
                // Signal that we want to close
                // The actual closing is handled by the platform integration
                self.is_active = false;
                iced::exit()
            }
            OverlayMessage::SetRemainingSeconds(seconds) => {
                self.remaining_seconds = Some(seconds);
                Task::none()
            }
            OverlayMessage::ShieldDeactivated => {
                self.is_active = false;
                iced::exit()
            }
            OverlayMessage::KeyPressed(key, modifiers) => {
                // Check if this is the exit key combination
                if self.is_exit_key(&key, modifiers) {
                    log::info!("Exit key detected, closing overlay");
                    self.is_active = false;
                    return iced::exit();
                }
                Task::none()
            }
            OverlayMessage::WindowEvent(_event) => {
                // Handle window events if needed (e.g., focus changes)
                // Currently just pass through
                Task::none()
            }
        }
    }

    /// Format duration as MM:SS or HH:MM:SS based on magnitude
    ///
    /// Returns (formatted_string, is_warning) where is_warning is true
    /// when remaining time is <= 60 seconds (for visual feedback)
    fn format_duration(seconds: u64) -> (String, bool) {
        let hours = seconds / 3600;
        let minutes = (seconds % 3600) / 60;
        let secs = seconds % 60;

        let formatted = if hours > 0 {
            format!("{hours:02}:{minutes:02}:{secs:02}")
        } else {
            format!("{minutes:02}:{secs:02}")
        };

        let is_warning = seconds <= 60 && seconds > 0;
        (formatted, is_warning)
    }

    /// Calculate progress as a fraction (0.0 to 1.0)
    fn calculate_progress(&self) -> Option<f32> {
        match (self.remaining_seconds, self.initial_duration) {
            (Some(remaining), Some(initial)) if initial > 0 => {
                Some(remaining as f32 / initial as f32)
            }
            _ => None,
        }
    }

    /// Render the overlay view with polished visual design
    pub fn view(&self) -> Element<'_, OverlayMessage> {
        // Determine if we're in warning state
        let is_warning = self.remaining_seconds.is_some_and(|r| r <= 60 && r > 0);

        // Build the main content column with improved spacing
        let mut content = column![]
            .spacing(spacing::LG)
            .align_x(iced::Alignment::Center);

        // Add cat icon with status
        content = content.push(cat_status_icon(is_warning));

        // Add title with status badge
        content = content.push(
            column![
                text("Cat Shield")
                    .size(typography::SIZE_HEADER_LARGE)
                    .color(colors::TEXT_PRIMARY),
                Space::new().height(Length::Fixed(spacing::SM)),
                status_badge(),
            ]
            .align_x(iced::Alignment::Center)
            .spacing(spacing::XS),
        );

        // Add timer display unless hidden
        if !self.hide_timer {
            // Format the timer display
            let (timer_text_str, timer_is_warning) = if let Some(remaining) = self.remaining_seconds
            {
                Self::format_duration(remaining)
            } else {
                // Elapsed time mode
                Self::format_duration(self.elapsed_seconds)
            };

            // Timer label with subtle styling
            let timer_label = if self.remaining_seconds.is_some() {
                "Time Remaining"
            } else {
                "Elapsed Time"
            };

            content = content.push(Space::new().height(Length::Fixed(spacing::MD)));

            content = content.push(
                text(timer_label)
                    .size(typography::SIZE_BODY)
                    .color(colors::TEXT_SECONDARY),
            );

            // Use warning color when time is running out
            let timer_color = if timer_is_warning && self.remaining_seconds.is_some() {
                colors::TIMER_WARNING
            } else {
                colors::TEXT_PRIMARY
            };

            // Large timer display with monospace-like appearance
            content = content.push(
                text(timer_text_str)
                    .size(typography::SIZE_HERO)
                    .color(timer_color),
            );

            // Add progress bar if we have timer duration and progress is enabled
            if self.show_progress {
                if let Some(progress) = self.calculate_progress() {
                    content = content.push(Space::new().height(Length::Fixed(spacing::SM)));
                    content = content.push(progress_bar(progress, timer_is_warning));
                }
            }
        }

        // Add some vertical space before hints section
        content = content.push(Space::new().height(Length::Fixed(spacing::XXL)));

        // Hint section with subtle card styling
        let mut hints = column![]
            .spacing(spacing::SM)
            .align_x(iced::Alignment::Center);

        // Add exit key hint if available
        if !self.exit_key_display.is_empty() {
            hints = hints.push(
                row![
                    text("Press")
                        .size(typography::SIZE_BODY)
                        .color(colors::TEXT_MUTED),
                    Space::new().width(Length::Fixed(spacing::XS)),
                    container(
                        text(&self.exit_key_display)
                            .size(typography::SIZE_BODY)
                            .color(colors::TEXT_PRIMARY)
                    )
                    .padding([spacing::XS, spacing::SM])
                    .style(|_theme| container::Style {
                        background: Some(iced::Background::Color(Color::from_rgba(
                            1.0, 1.0, 1.0, 0.1
                        ))),
                        border: iced::Border {
                            radius: borders::RADIUS_SM.into(),
                            width: borders::WIDTH_DEFAULT,
                            color: colors::BORDER_SUBTLE,
                        },
                        ..Default::default()
                    }),
                    Space::new().width(Length::Fixed(spacing::XS)),
                    text("to unlock")
                        .size(typography::SIZE_BODY)
                        .color(colors::TEXT_MUTED),
                ]
                .align_y(iced::Alignment::Center),
            );
        }

        content = content.push(hints);

        content = content.push(Space::new().height(Length::Fixed(spacing::LG)));

        // Close button with improved styling
        content = content.push(
            button(
                text("Close Shield")
                    .size(typography::SIZE_BODY)
                    .color(Color::WHITE),
            )
            .padding([spacing::MD, spacing::XL])
            .style(CatShieldTheme::close_button)
            .on_press(OverlayMessage::RequestClose),
        );

        // Wrap in centered container with semi-transparent background using configured color
        let (r, g, b) = self.color;
        let background_color = Color::from_rgba(r, g, b, self.opacity as f32);

        container(center(content))
            .width(Length::Fill)
            .height(Length::Fill)
            .style(move |_theme| container::Style {
                background: Some(iced::Background::Color(background_color)),
                ..Default::default()
            })
            .into()
    }

    /// Subscription for timer ticks and keyboard events
    pub fn subscription(&self) -> Subscription<OverlayMessage> {
        if self.is_active {
            // Combine timer subscription with keyboard event subscription
            Subscription::batch([
                // Update every second for timer display
                time::every(Duration::from_secs(1)).map(OverlayMessage::Tick),
                // Listen for keyboard events to detect exit key
                iced::event::listen_with(|event, _status, _id| match event {
                    Event::Keyboard(keyboard::Event::KeyPressed { key, modifiers, .. }) => {
                        Some(OverlayMessage::KeyPressed(key, modifiers))
                    }
                    Event::Window(window_event) => Some(OverlayMessage::WindowEvent(window_event)),
                    _ => None,
                }),
            ])
        } else {
            Subscription::none()
        }
    }

    /// Get the theme for the overlay
    pub fn theme(&self) -> Theme {
        CatShieldTheme::base()
    }

    /// Get window settings for the overlay
    ///
    /// Returns settings configured for a fullscreen, borderless, transparent,
    /// always-on-top overlay window.
    ///
    /// # Multi-Monitor Note
    ///
    /// The `fullscreen` setting only covers the current/primary monitor.
    /// For true multi-monitor coverage, the platform integration layer should
    /// spawn additional overlay windows on each connected display.
    pub fn window_settings() -> WindowSettings {
        WindowSettings {
            size: Size::new(1920.0, 1080.0), // Default size, overridden by fullscreen
            position: Position::Centered,
            fullscreen: true,          // Fullscreen mode
            decorations: false,        // Borderless
            transparent: true,         // Enable transparency
            level: Level::AlwaysOnTop, // Stay above other windows
            exit_on_close_request: true,
            resizable: false, // No resizing for overlay
            ..WindowSettings::default()
        }
    }

    /// Run the overlay application
    ///
    /// This is the main entry point for running the iced overlay.
    /// Call this after setting up input blocking.
    pub fn run(exit_key_display: String, timer_seconds: Option<u64>, opacity: f64) -> iced::Result {
        iced::application(
            move || Self::new(exit_key_display.clone(), timer_seconds, opacity),
            Self::update,
            Self::view,
        )
        .title("Cat Shield")
        .subscription(Self::subscription)
        .window(Self::window_settings())
        .theme(Self::theme)
        .run()
    }

    /// Run the overlay application with full exit key configuration
    ///
    /// This variant allows specifying the exit key configuration for keyboard
    /// shortcut detection within the iced application itself.
    ///
    /// # Arguments
    ///
    /// * `exit_key_config` - Configuration for the exit key shortcut
    /// * `timer_seconds` - Optional countdown timer duration
    /// * `opacity` - Overlay opacity (0.0 - 1.0)
    pub fn run_with_exit_key(
        exit_key_config: ExitKeyConfig,
        timer_seconds: Option<u64>,
        opacity: f64,
    ) -> iced::Result {
        iced::application(
            move || Self::with_exit_key(exit_key_config.clone(), timer_seconds, opacity),
            Self::update,
            Self::view,
        )
        .title("Cat Shield")
        .subscription(Self::subscription)
        .window(Self::window_settings())
        .theme(Self::theme)
        .run()
    }

    /// Run the overlay application with full timer configuration (legacy)
    ///
    /// This variant allows full control over timer display including:
    /// - Timer duration
    /// - Whether to hide the timer display (`--hide-timer`)
    /// - Whether to show the progress bar
    ///
    /// # Arguments
    ///
    /// * `exit_key_config` - Configuration for the exit key shortcut
    /// * `timer_config` - Timer display configuration
    /// * `opacity` - Overlay opacity (0.0 - 1.0)
    pub fn run_with_timer_config(
        exit_key_config: ExitKeyConfig,
        timer_config: TimerConfig,
        opacity: f64,
    ) -> iced::Result {
        iced::application(
            move || Self::with_timer_config(exit_key_config.clone(), timer_config.clone(), opacity),
            Self::update,
            Self::view,
        )
        .title("Cat Shield")
        .subscription(Self::subscription)
        .window(Self::window_settings())
        .theme(Self::theme)
        .run()
    }

    /// Run the overlay application with full overlay configuration
    ///
    /// This variant allows full control over all overlay settings including:
    /// - Timer duration
    /// - Whether to hide the timer display (`--hide-timer`)
    /// - Whether to show the progress bar
    /// - Overlay opacity
    /// - Overlay background color
    ///
    /// # Arguments
    ///
    /// * `exit_key_config` - Configuration for the exit key shortcut
    /// * `config` - Full overlay display configuration
    pub fn run_with_overlay_config(
        exit_key_config: ExitKeyConfig,
        config: OverlayConfig,
    ) -> iced::Result {
        iced::application(
            move || Self::with_overlay_config(exit_key_config.clone(), config.clone()),
            Self::update,
            Self::view,
        )
        .title("Cat Shield")
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
    fn test_overlay_default() {
        let app = OverlayApp::default();
        assert!(app.is_active);
        assert!(app.remaining_seconds.is_none());
        assert_eq!(app.elapsed_seconds, 0);
        assert!(app.exit_key_config.is_none());
        // Verify default opacity matches config constant
        assert_eq!(app.opacity, crate::config::DEFAULT_OVERLAY_OPACITY);
    }

    #[test]
    fn test_overlay_with_timer() {
        let app = OverlayApp::new("Cmd+Option+U".to_string(), Some(300), 0.85);
        assert!(app.is_active);
        assert_eq!(app.remaining_seconds, Some(300));
        assert_eq!(app.exit_key_display, "Cmd+Option+U");
        assert!(app.exit_key_config.is_none());
    }

    #[test]
    fn test_overlay_with_exit_key() {
        let exit_config = ExitKeyConfig {
            key: keyboard::Key::Character("x".into()),
            modifiers: keyboard::Modifiers::CTRL.union(keyboard::Modifiers::ALT),
            display: "Ctrl+Alt+X".to_string(),
        };
        let app = OverlayApp::with_exit_key(exit_config, Some(600), 0.5);
        assert!(app.is_active);
        assert_eq!(app.remaining_seconds, Some(600));
        assert_eq!(app.exit_key_display, "Ctrl+Alt+X");
        assert!(app.exit_key_config.is_some());
        assert_eq!(app.opacity, 0.5);
    }

    #[test]
    fn test_exit_key_detection() {
        let exit_config = ExitKeyConfig {
            key: keyboard::Key::Character("u".into()),
            modifiers: keyboard::Modifiers::LOGO.union(keyboard::Modifiers::ALT),
            display: "Cmd+Option+U".to_string(),
        };
        let app = OverlayApp::with_exit_key(exit_config, None, 0.5);

        // Matching key and modifiers
        assert!(app.is_exit_key(
            &keyboard::Key::Character("u".into()),
            keyboard::Modifiers::LOGO.union(keyboard::Modifiers::ALT)
        ));

        // Case-insensitive matching
        assert!(app.is_exit_key(
            &keyboard::Key::Character("U".into()),
            keyboard::Modifiers::LOGO.union(keyboard::Modifiers::ALT)
        ));

        // Wrong key
        assert!(!app.is_exit_key(
            &keyboard::Key::Character("x".into()),
            keyboard::Modifiers::LOGO.union(keyboard::Modifiers::ALT)
        ));

        // Missing modifiers
        assert!(!app.is_exit_key(
            &keyboard::Key::Character("u".into()),
            keyboard::Modifiers::LOGO
        ));

        // Extra modifiers (should still match - contains check)
        assert!(app.is_exit_key(
            &keyboard::Key::Character("u".into()),
            keyboard::Modifiers::LOGO
                .union(keyboard::Modifiers::ALT)
                .union(keyboard::Modifiers::SHIFT)
        ));
    }

    #[test]
    fn test_exit_key_detection_no_config() {
        let app = OverlayApp::default();
        // Should return false when no exit key is configured
        assert!(!app.is_exit_key(
            &keyboard::Key::Character("u".into()),
            keyboard::Modifiers::LOGO.union(keyboard::Modifiers::ALT)
        ));
    }

    #[test]
    fn test_window_settings() {
        let settings = OverlayApp::window_settings();
        assert!(!settings.decorations);
        assert!(settings.transparent);
        assert_eq!(settings.level, Level::AlwaysOnTop);
    }

    #[test]
    fn test_window_settings_is_fullscreen() {
        let settings = OverlayApp::window_settings();
        assert!(settings.fullscreen);
    }

    #[test]
    fn test_exit_key_config_default() {
        let config = ExitKeyConfig::default();
        assert_eq!(config.display, "Cmd+Option+U");
        assert!(config.modifiers.contains(keyboard::Modifiers::LOGO));
        assert!(config.modifiers.contains(keyboard::Modifiers::ALT));
    }

    // ============================================================
    // Timer display tests
    // ============================================================

    #[test]
    fn test_format_duration_seconds_only() {
        let (formatted, is_warning) = OverlayApp::format_duration(45);
        assert_eq!(formatted, "00:45");
        assert!(is_warning); // <= 60 seconds
    }

    #[test]
    fn test_format_duration_minutes_and_seconds() {
        let (formatted, is_warning) = OverlayApp::format_duration(90);
        assert_eq!(formatted, "01:30");
        assert!(!is_warning); // > 60 seconds
    }

    #[test]
    fn test_format_duration_hours() {
        let (formatted, is_warning) = OverlayApp::format_duration(3661);
        assert_eq!(formatted, "01:01:01");
        assert!(!is_warning);
    }

    #[test]
    fn test_format_duration_exactly_one_hour() {
        let (formatted, is_warning) = OverlayApp::format_duration(3600);
        assert_eq!(formatted, "01:00:00");
        assert!(!is_warning);
    }

    #[test]
    fn test_format_duration_warning_threshold() {
        // 60 seconds - should be warning
        let (_, is_warning) = OverlayApp::format_duration(60);
        assert!(is_warning);

        // 61 seconds - should not be warning
        let (_, is_warning) = OverlayApp::format_duration(61);
        assert!(!is_warning);

        // 0 seconds - should not be warning (expired)
        let (_, is_warning) = OverlayApp::format_duration(0);
        assert!(!is_warning);
    }

    #[test]
    fn test_format_duration_zero() {
        let (formatted, _) = OverlayApp::format_duration(0);
        assert_eq!(formatted, "00:00");
    }

    // ============================================================
    // Progress bar tests
    // ============================================================

    #[test]
    fn test_calculate_progress_full() {
        let app = OverlayApp {
            remaining_seconds: Some(100),
            initial_duration: Some(100),
            ..Default::default()
        };
        assert_eq!(app.calculate_progress(), Some(1.0));
    }

    #[test]
    fn test_calculate_progress_half() {
        let app = OverlayApp {
            remaining_seconds: Some(50),
            initial_duration: Some(100),
            ..Default::default()
        };
        assert_eq!(app.calculate_progress(), Some(0.5));
    }

    #[test]
    fn test_calculate_progress_empty() {
        let app = OverlayApp {
            remaining_seconds: Some(0),
            initial_duration: Some(100),
            ..Default::default()
        };
        assert_eq!(app.calculate_progress(), Some(0.0));
    }

    #[test]
    fn test_calculate_progress_no_timer() {
        let app = OverlayApp::default();
        assert!(app.calculate_progress().is_none());
    }

    #[test]
    fn test_calculate_progress_zero_duration() {
        let app = OverlayApp {
            remaining_seconds: Some(0),
            initial_duration: Some(0),
            ..Default::default()
        };
        assert!(app.calculate_progress().is_none());
    }

    // ============================================================
    // Timer config tests
    // ============================================================

    #[test]
    fn test_overlay_with_timer_config() {
        let exit_config = ExitKeyConfig::default();
        let timer_config = TimerConfig {
            duration: Some(300),
            hide_timer: true,
            show_progress: false,
        };
        let app = OverlayApp::with_timer_config(exit_config, timer_config, 0.7);

        assert!(app.is_active);
        assert_eq!(app.remaining_seconds, Some(300));
        assert_eq!(app.initial_duration, Some(300));
        assert!(app.hide_timer);
        assert!(!app.show_progress);
        assert_eq!(app.opacity, 0.7);
    }

    #[test]
    fn test_timer_config_default() {
        let config = TimerConfig::default();
        assert!(config.duration.is_none());
        assert!(!config.hide_timer);
        assert!(!config.show_progress);
    }

    #[test]
    fn test_overlay_stores_initial_duration() {
        let app = OverlayApp::new("Test".to_string(), Some(600), 0.5);
        assert_eq!(app.initial_duration, Some(600));
        assert_eq!(app.remaining_seconds, Some(600));
    }

    #[test]
    fn test_overlay_default_shows_timer() {
        let app = OverlayApp::default();
        assert!(!app.hide_timer);
        assert!(app.show_progress);
    }
}
