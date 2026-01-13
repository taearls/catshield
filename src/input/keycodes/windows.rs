//! Windows virtual key mappings (stub)
//!
//! This module will provide conversion between the canonical `Key` enum and
//! Windows-specific virtual key codes.
//!
//! See: <https://docs.microsoft.com/en-us/windows/win32/inputdev/virtual-key-codes>
//!
//! # Status
//!
//! This is a stub module. Full implementation will be added as part of Windows support.

use super::Key;

/// Convert a canonical `Key` to its Windows virtual key code.
///
/// # Status
///
/// This is a stub implementation. Returns `None` for all keys until Windows support is implemented.
#[must_use]
pub const fn key_to_keycode(_key: Key) -> Option<u32> {
    // TODO: Implement Windows virtual key mappings
    // Reference: https://docs.microsoft.com/en-us/windows/win32/inputdev/virtual-key-codes
    //
    // Example mappings (to be implemented):
    // Key::A => Some(0x41),  // VK_A
    // Key::Escape => Some(0x1B),  // VK_ESCAPE
    // Key::F1 => Some(0x70),  // VK_F1
    // etc.
    None
}

/// Convert a Windows virtual key code to the canonical `Key` enum.
///
/// # Status
///
/// This is a stub implementation. Returns `None` for all keycodes until Windows support is implemented.
#[must_use]
pub const fn keycode_to_key(_keycode: u32) -> Option<Key> {
    // TODO: Implement Windows virtual key mappings
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
        assert_eq!(keycode_to_key(0x41), None);
        assert_eq!(keycode_to_key(0x1B), None);
    }
}
