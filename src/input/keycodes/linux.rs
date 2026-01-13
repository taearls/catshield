//! Linux keycode mappings (stub)
//!
//! This module will provide conversion between the canonical `Key` enum and
//! Linux-specific keycodes (for both X11 and Wayland).
//!
//! See:
//! - X11: <https://cgit.freedesktop.org/xorg/proto/x11proto/tree/keysymdef.h>
//! - evdev: <https://github.com/torvalds/linux/blob/master/include/uapi/linux/input-event-codes.h>
//!
//! # Status
//!
//! This is a stub module. Full implementation will be added as part of Linux support.

use super::Key;

/// Convert a canonical `Key` to its Linux keycode.
///
/// # Status
///
/// This is a stub implementation. Returns `None` for all keys until Linux support is implemented.
#[must_use]
pub const fn key_to_keycode(_key: Key) -> Option<u32> {
    // TODO: Implement Linux keycode mappings
    // This will likely need to support both X11 keysyms and evdev keycodes.
    //
    // Example evdev mappings (to be implemented):
    // Key::A => Some(30),  // KEY_A
    // Key::Escape => Some(1),  // KEY_ESC
    // Key::F1 => Some(59),  // KEY_F1
    // etc.
    None
}

/// Convert a Linux keycode to the canonical `Key` enum.
///
/// # Status
///
/// This is a stub implementation. Returns `None` for all keycodes until Linux support is implemented.
#[must_use]
pub const fn keycode_to_key(_keycode: u32) -> Option<Key> {
    // TODO: Implement Linux keycode mappings
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stub_key_to_keycode() {
        // Stub always returns None
        assert_eq!(key_to_keycode(Key::A), None);
        assert_eq!(key_to_keycode(Key::Escape), None);
    }

    #[test]
    fn test_stub_keycode_to_key() {
        // Stub always returns None
        assert_eq!(keycode_to_key(30), None);
        assert_eq!(keycode_to_key(1), None);
    }
}
