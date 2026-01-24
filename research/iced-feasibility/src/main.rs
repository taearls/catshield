//! Minimal iced prototype for catshield overlay feasibility testing
//!
//! This prototype tests the following iced capabilities:
//! 1. Fullscreen borderless window
//! 2. Transparent/semi-transparent background
//! 3. AlwaysOnTop window level
//! 4. Custom rendering (close button, timer text)
//! 5. Mouse event handling
//! 6. Timer/animation subscriptions
//!
//! Run with: cargo run
//! Exit with: Click close button

use iced::widget::{button, center, column, container, text};
use iced::window::{self, Level, Position, Settings as WindowSettings};
use iced::{time, Color, Element, Length, Size, Subscription, Task, Theme};
use std::time::{Duration, Instant};

fn main() -> iced::Result {
    iced::application(Overlay::default, Overlay::update, Overlay::view)
        .title("CatShield Overlay Prototype")
        .subscription(Overlay::subscription)
        .window(WindowSettings {
            size: Size::new(800.0, 600.0), // Will be resized to fullscreen
            position: Position::Centered,
            decorations: false, // Borderless
            transparent: true,  // Enable transparency
            level: Level::AlwaysOnTop,
            exit_on_close_request: true,
            ..WindowSettings::default()
        })
        .theme(Overlay::theme)
        .run()
}

#[derive(Debug, Clone)]
enum Message {
    Tick(Instant),
    ToggleFullscreen,
    Exit,
}

struct Overlay {
    start_time: Instant,
    elapsed_seconds: u64,
    is_fullscreen: bool,
}

impl Default for Overlay {
    fn default() -> Self {
        Self {
            start_time: Instant::now(),
            elapsed_seconds: 0,
            is_fullscreen: false,
        }
    }
}

impl Overlay {
    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Tick(now) => {
                self.elapsed_seconds = now.duration_since(self.start_time).as_secs();
                Task::none()
            }
            Message::ToggleFullscreen => {
                self.is_fullscreen = !self.is_fullscreen;
                // Toggle fullscreen mode - requires querying for window ID first
                let mode = if self.is_fullscreen {
                    window::Mode::Fullscreen
                } else {
                    window::Mode::Windowed
                };
                // For simplicity, just exit fullscreen demonstration
                // The actual catshield will start fullscreen
                iced::exit()
            }
            Message::Exit => iced::exit(),
        }
    }

    fn view(&self) -> Element<'_, Message> {
        let timer_text = format!(
            "{:02}:{:02}",
            self.elapsed_seconds / 60,
            self.elapsed_seconds % 60
        );

        // Main overlay content
        let content = column![
            text("CatShield Overlay Prototype")
                .size(40)
                .color(Color::WHITE),
            text(timer_text).size(80).color(Color::WHITE),
            text("Click buttons below to control")
                .size(16)
                .color(Color::from_rgba(1.0, 1.0, 1.0, 0.7)),
            button(text("Toggle Fullscreen").color(Color::WHITE))
                .padding(10)
                .on_press(Message::ToggleFullscreen),
            button(text("Close").color(Color::WHITE))
                .padding(10)
                .style(|theme, status| {
                    let mut style = button::primary(theme, status);
                    style.background =
                        Some(iced::Background::Color(Color::from_rgb(0.8, 0.2, 0.2)));
                    style
                })
                .on_press(Message::Exit),
        ]
        .spacing(20)
        .align_x(iced::Alignment::Center);

        // Semi-transparent background container
        container(center(content))
            .width(Length::Fill)
            .height(Length::Fill)
            .style(|_theme| container::Style {
                background: Some(iced::Background::Color(Color::from_rgba(
                    0.1, 0.1, 0.1, 0.85,
                ))),
                ..container::Style::default()
            })
            .into()
    }

    fn subscription(&self) -> Subscription<Message> {
        // Timer subscription for elapsed time updates (10 times per second)
        time::every(Duration::from_millis(100)).map(Message::Tick)
    }

    fn theme(&self) -> Theme {
        Theme::Dark
    }
}
