//! Shield overlay window using iced
//!
//! Provides a fullscreen, semi-transparent overlay that displays:
//! - Timer countdown (when active)
//! - Close button (hold to exit)
//! - Exit key hint
//!
//! This module handles only the visual rendering. Input blocking is handled
//! by the platform-specific implementations in `src/platform/`.

use iced::widget::{button, center, column, container, text};
use iced::window::{Level, Position, Settings as WindowSettings};
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
}

impl Default for OverlayApp {
    fn default() -> Self {
        Self {
            start_time: Instant::now(),
            elapsed_seconds: 0,
            remaining_seconds: None,
            is_active: true,
            exit_key_display: String::new(),
            opacity: 0.85,
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

        // Add close button
        content = content.push(
            text("or hold close button for 3 seconds")
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

    /// Subscription for timer ticks
    pub fn subscription(&self) -> Subscription<OverlayMessage> {
        if self.is_active {
            // Update every second for timer display
            time::every(Duration::from_secs(1)).map(OverlayMessage::Tick)
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
    pub fn window_settings() -> WindowSettings {
        WindowSettings {
            size: Size::new(800.0, 600.0), // Will be overridden to fullscreen
            position: Position::Centered,
            decorations: false,        // Borderless
            transparent: true,         // Enable transparency
            level: Level::AlwaysOnTop, // Stay above other windows
            exit_on_close_request: true,
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
    }

    #[test]
    fn test_overlay_with_timer() {
        let app = OverlayApp::new("Cmd+Option+U".to_string(), Some(300), 0.85);
        assert!(app.is_active);
        assert_eq!(app.remaining_seconds, Some(300));
        assert_eq!(app.exit_key_display, "Cmd+Option+U");
    }

    #[test]
    fn test_window_settings() {
        let settings = OverlayApp::window_settings();
        assert!(!settings.decorations);
        assert!(settings.transparent);
        assert_eq!(settings.level, Level::AlwaysOnTop);
    }
}
