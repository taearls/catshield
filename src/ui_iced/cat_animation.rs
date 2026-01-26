//! Animated cat companion for the overlay
//!
//! This module provides an animated cat that appears on the overlay,
//! adding visual delight to the protection screen. The cat has various
//! animation states:
//!
//! - Idle: Gentle breathing/bobbing animation
//! - Blinking: Occasional eye blinks
//! - Alert: Reactive animation when input is blocked (future enhancement)
//!
//! The cat can be positioned in different corners of the screen and
//! can be disabled via settings or the `--no-cat` CLI flag.

use std::time::{Duration, Instant};

use iced::widget::{column, container, text, Space};
use iced::{Color, Element, Length};

use crate::ui_iced::theme::{colors, spacing, typography};

/// Position of the cat on the overlay
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CatPosition {
    /// Bottom right corner (default)
    #[default]
    BottomRight,
    /// Bottom left corner
    BottomLeft,
    /// Top right corner
    TopRight,
    /// Top left corner
    TopLeft,
}

impl CatPosition {
    /// All available positions
    pub const ALL: [CatPosition; 4] = [
        CatPosition::BottomRight,
        CatPosition::BottomLeft,
        CatPosition::TopRight,
        CatPosition::TopLeft,
    ];

    /// Get horizontal alignment for this position
    pub fn horizontal_alignment(&self) -> iced::alignment::Horizontal {
        match self {
            CatPosition::BottomRight | CatPosition::TopRight => iced::alignment::Horizontal::Right,
            CatPosition::BottomLeft | CatPosition::TopLeft => iced::alignment::Horizontal::Left,
        }
    }

    /// Get vertical alignment for this position
    pub fn vertical_alignment(&self) -> iced::alignment::Vertical {
        match self {
            CatPosition::BottomRight | CatPosition::BottomLeft => iced::alignment::Vertical::Bottom,
            CatPosition::TopRight | CatPosition::TopLeft => iced::alignment::Vertical::Top,
        }
    }

    /// Convert to config string
    pub fn to_config_string(&self) -> &'static str {
        match self {
            CatPosition::BottomRight => "bottom-right",
            CatPosition::BottomLeft => "bottom-left",
            CatPosition::TopRight => "top-right",
            CatPosition::TopLeft => "top-left",
        }
    }

    /// Parse from config string
    pub fn from_config_string(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "bottom-left" => CatPosition::BottomLeft,
            "top-right" => CatPosition::TopRight,
            "top-left" => CatPosition::TopLeft,
            _ => CatPosition::BottomRight,
        }
    }
}

impl std::fmt::Display for CatPosition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CatPosition::BottomRight => write!(f, "Bottom Right"),
            CatPosition::BottomLeft => write!(f, "Bottom Left"),
            CatPosition::TopRight => write!(f, "Top Right"),
            CatPosition::TopLeft => write!(f, "Top Left"),
        }
    }
}

/// Animation state for the cat
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CatAnimationState {
    /// Default idle animation with gentle bobbing
    #[default]
    Idle,
    /// Cat is blinking
    Blinking,
    /// Cat is sleeping (eyes closed, slower breathing)
    Sleeping,
}

/// Frame data for cat animation (ASCII art frames)
#[derive(Debug, Clone)]
pub struct CatFrame {
    /// The cat's face/body representation
    pub body: &'static str,
    /// Duration this frame should be displayed
    pub duration_ms: u64,
}

/// Predefined animation frames for the cat
pub mod frames {
    use super::CatFrame;

    /// Idle animation frames - cat with open eyes
    pub const IDLE: [CatFrame; 4] = [
        // Normal idle
        CatFrame {
            body: "  /\\_/\\  \n ( o.o ) \n  > ^ <  ",
            duration_ms: 2000,
        },
        // Slight head tilt
        CatFrame {
            body: "  /\\_/\\  \n ( o.o ) \n  > ^ <  ",
            duration_ms: 1500,
        },
        // Looking left
        CatFrame {
            body: "  /\\_/\\  \n ( o.o ) \n  > ^ <  ",
            duration_ms: 1000,
        },
        // Looking right
        CatFrame {
            body: "  /\\_/\\  \n ( o.o ) \n  > ^ <  ",
            duration_ms: 1000,
        },
    ];

    /// Blinking animation frames
    pub const BLINK: [CatFrame; 3] = [
        // Eyes closing
        CatFrame {
            body: "  /\\_/\\  \n ( -.o ) \n  > ^ <  ",
            duration_ms: 80,
        },
        // Eyes closed
        CatFrame {
            body: "  /\\_/\\  \n ( -.- ) \n  > ^ <  ",
            duration_ms: 120,
        },
        // Eyes opening
        CatFrame {
            body: "  /\\_/\\  \n ( o.- ) \n  > ^ <  ",
            duration_ms: 80,
        },
    ];

    /// Sleeping animation frames
    pub const SLEEPING: [CatFrame; 2] = [
        CatFrame {
            body: "  /\\_/\\  \n ( -.- ) \n  > ^ < z",
            duration_ms: 1500,
        },
        CatFrame {
            body: "  /\\_/\\  \n ( -.- ) \n  > ^ < Z",
            duration_ms: 1500,
        },
    ];
}

/// Cat companion animation state
#[derive(Debug, Clone)]
pub struct CatCompanion {
    /// Whether the cat is visible
    pub visible: bool,
    /// Current animation state
    pub state: CatAnimationState,
    /// Position on the screen
    pub position: CatPosition,
    /// When the current animation frame started
    pub frame_start: Instant,
    /// Current frame index within the animation
    pub frame_index: usize,
    /// Time since last blink
    pub last_blink: Instant,
    /// Interval between blinks (randomized)
    pub blink_interval: Duration,
    /// Current vertical offset for bobbing animation
    pub bob_offset: f32,
    /// Animation phase for bobbing (0.0 - 1.0)
    pub bob_phase: f32,
}

impl Default for CatCompanion {
    fn default() -> Self {
        Self::new()
    }
}

impl CatCompanion {
    /// Create a new cat companion with default settings
    pub fn new() -> Self {
        let now = Instant::now();
        Self {
            visible: true,
            state: CatAnimationState::Idle,
            position: CatPosition::default(),
            frame_start: now,
            frame_index: 0,
            last_blink: now,
            blink_interval: Duration::from_secs(4),
            bob_offset: 0.0,
            bob_phase: 0.0,
        }
    }

    /// Create a cat companion with specific settings
    pub fn with_settings(visible: bool, position: CatPosition) -> Self {
        let mut cat = Self::new();
        cat.visible = visible;
        cat.position = position;
        cat
    }

    /// Update the animation state based on elapsed time
    ///
    /// Call this on each tick to advance animations.
    /// Returns true if the visual state changed and a redraw is needed.
    pub fn tick(&mut self, now: Instant) -> bool {
        if !self.visible {
            return false;
        }

        let mut changed = false;

        // Update bobbing animation (continuous sine wave)
        let elapsed_secs = now.duration_since(self.frame_start).as_secs_f32();
        let new_bob_phase = (elapsed_secs * 0.5) % 1.0; // Complete cycle every 2 seconds
        if (new_bob_phase - self.bob_phase).abs() > 0.01 {
            self.bob_phase = new_bob_phase;
            // Sine wave for smooth bobbing: -3 to +3 pixels
            self.bob_offset = (self.bob_phase * std::f32::consts::TAU).sin() * 3.0;
            changed = true;
        }

        // Check if it's time to blink
        if self.state == CatAnimationState::Idle
            && now.duration_since(self.last_blink) >= self.blink_interval
        {
            self.state = CatAnimationState::Blinking;
            self.frame_index = 0;
            self.frame_start = now;
            self.last_blink = now;
            // Randomize next blink interval (3-6 seconds)
            let random_offset = (now.elapsed().as_nanos() % 3000) as u64;
            self.blink_interval = Duration::from_millis(3000 + random_offset);
            changed = true;
        }

        // Advance animation frames
        let frames = self.current_frames();
        if !frames.is_empty() {
            let current_frame = &frames[self.frame_index];
            let frame_duration = Duration::from_millis(current_frame.duration_ms);

            if now.duration_since(self.frame_start) >= frame_duration {
                self.frame_index = (self.frame_index + 1) % frames.len();
                self.frame_start = now;
                changed = true;

                // Return to idle after blink animation completes
                if self.state == CatAnimationState::Blinking && self.frame_index == 0 {
                    self.state = CatAnimationState::Idle;
                }
            }
        }

        changed
    }

    /// Get the current animation frames based on state
    fn current_frames(&self) -> &[CatFrame] {
        match self.state {
            CatAnimationState::Idle => &frames::IDLE,
            CatAnimationState::Blinking => &frames::BLINK,
            CatAnimationState::Sleeping => &frames::SLEEPING,
        }
    }

    /// Get the current frame to display
    pub fn current_frame(&self) -> &CatFrame {
        let frames = self.current_frames();
        &frames[self.frame_index.min(frames.len() - 1)]
    }

    /// Get the cat emoji representation for the overlay
    ///
    /// Returns different emoji based on animation state for visual variety.
    pub fn emoji(&self) -> &'static str {
        match self.state {
            CatAnimationState::Idle => "🐱",
            CatAnimationState::Blinking => "😺",
            CatAnimationState::Sleeping => "😸",
        }
    }

    /// Render the cat companion as an iced Element
    pub fn view<'a, M: 'a + Clone>(&self, is_warning: bool) -> Element<'a, M> {
        if !self.visible {
            return Space::new()
                .width(Length::Shrink)
                .height(Length::Shrink)
                .into();
        }

        let icon_color = if is_warning {
            colors::WARNING
        } else {
            colors::ACCENT
        };

        // Create the cat emoji with animation-based size variation
        let base_size = 64.0_f32;
        let size_variation = (self.bob_phase * std::f32::consts::TAU).sin() * 2.0;
        let icon_size = base_size + size_variation;

        let cat_emoji = text(self.emoji()).size(icon_size).color(icon_color);

        // Add a subtle caption based on state
        let caption = match self.state {
            CatAnimationState::Idle => "Guarding...",
            CatAnimationState::Blinking => "Watching...",
            CatAnimationState::Sleeping => "Zzz...",
        };

        let caption_color = Color::from_rgba(1.0, 1.0, 1.0, 0.5);

        container(
            column![
                cat_emoji,
                text(caption)
                    .size(typography::SIZE_MICRO)
                    .color(caption_color),
            ]
            .spacing(spacing::XS)
            .align_x(iced::Alignment::Center),
        )
        .padding(spacing::LG)
        .into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cat_position_default() {
        let pos = CatPosition::default();
        assert_eq!(pos, CatPosition::BottomRight);
    }

    #[test]
    fn test_cat_position_from_config_string() {
        assert_eq!(
            CatPosition::from_config_string("bottom-right"),
            CatPosition::BottomRight
        );
        assert_eq!(
            CatPosition::from_config_string("bottom-left"),
            CatPosition::BottomLeft
        );
        assert_eq!(
            CatPosition::from_config_string("top-right"),
            CatPosition::TopRight
        );
        assert_eq!(
            CatPosition::from_config_string("top-left"),
            CatPosition::TopLeft
        );
        // Invalid should default to bottom-right
        assert_eq!(
            CatPosition::from_config_string("invalid"),
            CatPosition::BottomRight
        );
    }

    #[test]
    fn test_cat_position_to_config_string() {
        assert_eq!(CatPosition::BottomRight.to_config_string(), "bottom-right");
        assert_eq!(CatPosition::BottomLeft.to_config_string(), "bottom-left");
        assert_eq!(CatPosition::TopRight.to_config_string(), "top-right");
        assert_eq!(CatPosition::TopLeft.to_config_string(), "top-left");
    }

    #[test]
    fn test_cat_position_display() {
        assert_eq!(format!("{}", CatPosition::BottomRight), "Bottom Right");
        assert_eq!(format!("{}", CatPosition::TopLeft), "Top Left");
    }

    #[test]
    fn test_cat_companion_default() {
        let cat = CatCompanion::new();
        assert!(cat.visible);
        assert_eq!(cat.state, CatAnimationState::Idle);
        assert_eq!(cat.position, CatPosition::BottomRight);
        assert_eq!(cat.frame_index, 0);
    }

    #[test]
    fn test_cat_companion_with_settings() {
        let cat = CatCompanion::with_settings(false, CatPosition::TopLeft);
        assert!(!cat.visible);
        assert_eq!(cat.position, CatPosition::TopLeft);
    }

    #[test]
    fn test_cat_emoji_states() {
        let mut cat = CatCompanion::new();

        cat.state = CatAnimationState::Idle;
        assert_eq!(cat.emoji(), "🐱");

        cat.state = CatAnimationState::Blinking;
        assert_eq!(cat.emoji(), "😺");

        cat.state = CatAnimationState::Sleeping;
        assert_eq!(cat.emoji(), "😸");
    }

    #[test]
    fn test_cat_tick_invisible() {
        let mut cat = CatCompanion::new();
        cat.visible = false;

        let changed = cat.tick(Instant::now());
        assert!(!changed);
    }

    #[test]
    fn test_animation_frames_exist() {
        // Verify all animation frame arrays are non-empty
        assert!(!frames::IDLE.is_empty());
        assert!(!frames::BLINK.is_empty());
        assert!(!frames::SLEEPING.is_empty());
    }

    #[test]
    fn test_current_frame_returns_valid() {
        let cat = CatCompanion::new();
        let frame = cat.current_frame();
        assert!(!frame.body.is_empty());
        assert!(frame.duration_ms > 0);
    }
}
