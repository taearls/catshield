//! Shield overlay window using iced
//!
//! Provides a fullscreen, semi-transparent overlay that displays:
//! - Timer countdown (when active)
//! - Close button (hold to exit)
//! - Exit key hint
//!
//! This module handles only the visual rendering. Input blocking is handled
//! by the platform-specific implementations in `src/platform/`.
//!
//! # Multi-Monitor Support
//!
//! iced's `Mode::Fullscreen` covers only the current monitor. For multi-monitor
//! setups, the application should spawn additional overlay windows on each display.
//! This is handled at the platform integration level (see main.rs).

use iced::event::Event;
use iced::keyboard;
use iced::widget::{button, center, column, container, text};
use iced::window::{self, Level, Position, Settings as WindowSettings};
use iced::{time, Color, Element, Length, Size, Subscription, Task, Theme};
use std::time::{Duration, Instant};

use crate::ui_iced::theme::{colors, CatShieldTheme};

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
    /// Whether the shield is currently active
    is_active: bool,
    /// Exit key display string (e.g., "Cmd+Option+U")
    exit_key_display: String,
    /// Overlay opacity (0.0 - 1.0)
    opacity: f64,
    /// Exit key configuration for detecting unlock shortcut
    exit_key_config: Option<ExitKeyConfig>,
}

impl Default for OverlayApp {
    fn default() -> Self {
        Self {
            start_time: Instant::now(),
            elapsed_seconds: 0,
            remaining_seconds: None,
            is_active: true,
            exit_key_display: String::new(),
            opacity: crate::config::DEFAULT_OVERLAY_OPACITY,
            exit_key_config: None,
        }
    }
}

impl OverlayApp {
    /// Create a new overlay app with configuration
    pub fn new(exit_key_display: String, timer_seconds: Option<u64>, opacity: f64) -> Self {
        Self {
            start_time: Instant::now(),
            elapsed_seconds: 0,
            remaining_seconds: timer_seconds,
            is_active: true,
            exit_key_display,
            opacity,
            exit_key_config: None,
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
            is_active: true,
            exit_key_display: exit_key_config.display.clone(),
            opacity,
            exit_key_config: Some(exit_key_config),
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

    /// Render the overlay view
    pub fn view(&self) -> Element<'_, OverlayMessage> {
        // Format the timer display
        let timer_text = if let Some(remaining) = self.remaining_seconds {
            // Countdown mode
            format!("{:02}:{:02}", remaining / 60, remaining % 60)
        } else {
            // Elapsed time mode
            format!(
                "{:02}:{:02}",
                self.elapsed_seconds / 60,
                self.elapsed_seconds % 60
            )
        };

        // Timer label
        let timer_label = if self.remaining_seconds.is_some() {
            "Time Remaining"
        } else {
            "Elapsed Time"
        };

        // Build the main content column
        let mut content = column![
            text("Cat Shield Active")
                .size(32)
                .color(colors::TEXT_PRIMARY),
            text(timer_label).size(16).color(colors::TEXT_SECONDARY),
            text(timer_text).size(72).color(colors::TEXT_PRIMARY),
        ]
        .spacing(10)
        .align_x(iced::Alignment::Center);

        // Add exit key hint if available
        if !self.exit_key_display.is_empty() {
            content = content.push(
                text(format!("Press {} to unlock", self.exit_key_display))
                    .size(14)
                    .color(colors::TEXT_MUTED),
            );
        }

        // Add close button hint
        // TODO(#155): Implement hold-to-close behavior, then update this text
        content = content.push(
            text("or click the close button")
                .size(12)
                .color(colors::TEXT_MUTED),
        );

        content = content.push(
            button(text("Close").size(16).color(Color::WHITE))
                .padding([12, 24])
                .style(CatShieldTheme::close_button)
                .on_press(OverlayMessage::RequestClose),
        );

        // Wrap in centered container with semi-transparent background
        let background_color = Color::from_rgba(0.1, 0.1, 0.1, self.opacity as f32);

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
            resizable: false,          // No resizing for overlay
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
}
