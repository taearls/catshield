//! Platform-agnostic types for Cat Shield
//!
//! These types are used across all platform implementations to ensure
//! a consistent interface regardless of the underlying operating system.

use std::fmt;

/// Represents keyboard modifier keys in a platform-agnostic way.
///
/// Each platform will map its native modifier flags to this structure.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Modifiers {
    /// Command key (macOS) / Windows key (Windows) / Super key (Linux)
    pub command: bool,
    /// Option key (macOS) / Alt key (Windows/Linux)
    pub option: bool,
    /// Control key (all platforms)
    pub control: bool,
    /// Shift key (all platforms)
    pub shift: bool,
}

impl Modifiers {
    /// Creates a new Modifiers instance with no modifiers pressed.
    #[must_use]
    pub const fn none() -> Self {
        Self {
            command: false,
            option: false,
            control: false,
            shift: false,
        }
    }

    /// Returns true if no modifiers are pressed.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        !self.command && !self.option && !self.control && !self.shift
    }

    /// Returns true if any modifier is pressed.
    #[must_use]
    pub const fn any(&self) -> bool {
        self.command || self.option || self.control || self.shift
    }
}

impl fmt::Display for Modifiers {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut parts = Vec::new();
        if self.command {
            parts.push("Cmd");
        }
        if self.option {
            parts.push("Option");
        }
        if self.control {
            parts.push("Ctrl");
        }
        if self.shift {
            parts.push("Shift");
        }
        if parts.is_empty() {
            write!(f, "None")
        } else {
            write!(f, "{}", parts.join("+"))
        }
    }
}

/// Represents a keyboard event in a platform-agnostic way.
#[derive(Debug, Clone, Copy)]
pub struct KeyEvent {
    /// The virtual keycode (platform-specific value)
    pub keycode: i64,
    /// The modifier keys pressed during this event
    pub modifiers: Modifiers,
    /// Whether this is a key press (true) or key release (false)
    pub is_key_down: bool,
}

impl KeyEvent {
    /// Creates a new KeyEvent.
    #[must_use]
    pub const fn new(keycode: i64, modifiers: Modifiers, is_key_down: bool) -> Self {
        Self {
            keycode,
            modifiers,
            is_key_down,
        }
    }
}

/// Represents a sleep prevention assertion.
///
/// This is an opaque handle returned by `PowerManager::prevent_sleep()`.
/// The exact meaning of the ID is platform-specific:
/// - macOS: IOKit assertion ID
/// - Windows: Power request handle
/// - Linux: D-Bus inhibit cookie
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SleepAssertion {
    /// Platform-specific assertion identifier
    id: u64,
}

impl SleepAssertion {
    /// Creates a new SleepAssertion with the given platform-specific ID.
    #[must_use]
    pub const fn new(id: u64) -> Self {
        Self { id }
    }

    /// Returns the platform-specific assertion ID.
    #[must_use]
    pub const fn id(&self) -> u64 {
        self.id
    }
}

/// Represents a rectangle for window positioning and sizing.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Rect {
    /// X coordinate of the top-left corner
    pub x: f64,
    /// Y coordinate of the top-left corner
    pub y: f64,
    /// Width of the rectangle
    pub width: f64,
    /// Height of the rectangle
    pub height: f64,
}

impl Rect {
    /// Creates a new Rect with the given position and size.
    #[must_use]
    pub const fn new(x: f64, y: f64, width: f64, height: f64) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    /// Creates a zero-sized Rect at the origin.
    #[must_use]
    pub const fn zero() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            width: 0.0,
            height: 0.0,
        }
    }

    /// Returns true if the rectangle has zero area.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.width <= 0.0 || self.height <= 0.0
    }

    /// Returns the area of the rectangle.
    ///
    /// Returns 0.0 for empty or invalid rectangles (zero or negative dimensions).
    #[must_use]
    pub fn area(&self) -> f64 {
        if self.is_empty() {
            0.0
        } else {
            self.width * self.height
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_modifiers_none() {
        let mods = Modifiers::none();
        assert!(!mods.command);
        assert!(!mods.option);
        assert!(!mods.control);
        assert!(!mods.shift);
        assert!(mods.is_empty());
        assert!(!mods.any());
    }

    #[test]
    fn test_modifiers_with_command() {
        let mods = Modifiers {
            command: true,
            ..Modifiers::none()
        };
        assert!(!mods.is_empty());
        assert!(mods.any());
    }

    #[test]
    fn test_modifiers_display() {
        let mods = Modifiers {
            command: true,
            shift: true,
            ..Modifiers::none()
        };
        let display = format!("{mods}");
        assert!(display.contains("Cmd"));
        assert!(display.contains("Shift"));
    }

    #[test]
    fn test_modifiers_display_none() {
        let mods = Modifiers::none();
        assert_eq!(format!("{mods}"), "None");
    }

    #[test]
    fn test_key_event_new() {
        let event = KeyEvent::new(42, Modifiers::none(), true);
        assert_eq!(event.keycode, 42);
        assert!(event.is_key_down);
        assert!(event.modifiers.is_empty());
    }

    #[test]
    fn test_sleep_assertion() {
        let assertion = SleepAssertion::new(12345);
        assert_eq!(assertion.id(), 12345);
    }

    #[test]
    fn test_rect_new() {
        let rect = Rect::new(10.0, 20.0, 100.0, 50.0);
        assert_eq!(rect.x, 10.0);
        assert_eq!(rect.y, 20.0);
        assert_eq!(rect.width, 100.0);
        assert_eq!(rect.height, 50.0);
    }

    #[test]
    fn test_rect_zero() {
        let rect = Rect::zero();
        assert_eq!(rect.x, 0.0);
        assert_eq!(rect.y, 0.0);
        assert_eq!(rect.width, 0.0);
        assert_eq!(rect.height, 0.0);
        assert!(rect.is_empty());
    }

    #[test]
    fn test_rect_area() {
        let rect = Rect::new(0.0, 0.0, 10.0, 5.0);
        assert_eq!(rect.area(), 50.0);
    }

    #[test]
    fn test_rect_area_empty_returns_zero() {
        let zero_width = Rect::new(0.0, 0.0, 0.0, 10.0);
        let zero_height = Rect::new(0.0, 0.0, 10.0, 0.0);
        let negative_width = Rect::new(0.0, 0.0, -5.0, 10.0);
        let negative_height = Rect::new(0.0, 0.0, 10.0, -5.0);

        assert_eq!(zero_width.area(), 0.0);
        assert_eq!(zero_height.area(), 0.0);
        assert_eq!(negative_width.area(), 0.0);
        assert_eq!(negative_height.area(), 0.0);
    }

    #[test]
    fn test_rect_is_empty() {
        let zero_width = Rect::new(0.0, 0.0, 0.0, 10.0);
        let zero_height = Rect::new(0.0, 0.0, 10.0, 0.0);
        let negative = Rect::new(0.0, 0.0, -5.0, 10.0);
        let valid = Rect::new(0.0, 0.0, 10.0, 10.0);

        assert!(zero_width.is_empty());
        assert!(zero_height.is_empty());
        assert!(negative.is_empty());
        assert!(!valid.is_empty());
    }
}
