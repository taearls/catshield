//! Windows virtual key mappings
//!
//! This module provides conversion between the canonical `Key` enum and
//! Windows-specific virtual key codes.
//!
//! See: <https://docs.microsoft.com/en-us/windows/win32/inputdev/virtual-key-codes>

use super::Key;

/// Convert a canonical `Key` to its Windows virtual key code.
///
/// Returns `None` if the key is not supported on Windows (all currently supported keys are).
///
/// # Virtual Key Code Reference
///
/// - Letters: 0x41-0x5A (A-Z)
/// - Numbers: 0x30-0x39 (0-9)
/// - Function keys: 0x70-0x7B (F1-F12)
/// - Navigation: VK_LEFT, VK_RIGHT, VK_UP, VK_DOWN, etc.
///
/// # Examples
///
/// ```ignore
/// use cat_shield::input::keycodes::{Key, key_to_keycode};
///
/// assert_eq!(key_to_keycode(Key::A), Some(0x41));
/// assert_eq!(key_to_keycode(Key::Escape), Some(0x1B));
/// ```
#[must_use]
pub const fn key_to_keycode(key: Key) -> Option<u32> {
    match key {
        // Letters (VK_A through VK_Z: 0x41-0x5A)
        Key::A => Some(0x41),
        Key::B => Some(0x42),
        Key::C => Some(0x43),
        Key::D => Some(0x44),
        Key::E => Some(0x45),
        Key::F => Some(0x46),
        Key::G => Some(0x47),
        Key::H => Some(0x48),
        Key::I => Some(0x49),
        Key::J => Some(0x4A),
        Key::K => Some(0x4B),
        Key::L => Some(0x4C),
        Key::M => Some(0x4D),
        Key::N => Some(0x4E),
        Key::O => Some(0x4F),
        Key::P => Some(0x50),
        Key::Q => Some(0x51),
        Key::R => Some(0x52),
        Key::S => Some(0x53),
        Key::T => Some(0x54),
        Key::U => Some(0x55),
        Key::V => Some(0x56),
        Key::W => Some(0x57),
        Key::X => Some(0x58),
        Key::Y => Some(0x59),
        Key::Z => Some(0x5A),

        // Numbers (VK_0 through VK_9: 0x30-0x39)
        Key::Num0 => Some(0x30),
        Key::Num1 => Some(0x31),
        Key::Num2 => Some(0x32),
        Key::Num3 => Some(0x33),
        Key::Num4 => Some(0x34),
        Key::Num5 => Some(0x35),
        Key::Num6 => Some(0x36),
        Key::Num7 => Some(0x37),
        Key::Num8 => Some(0x38),
        Key::Num9 => Some(0x39),

        // Function keys (VK_F1 through VK_F12: 0x70-0x7B)
        Key::F1 => Some(0x70),
        Key::F2 => Some(0x71),
        Key::F3 => Some(0x72),
        Key::F4 => Some(0x73),
        Key::F5 => Some(0x74),
        Key::F6 => Some(0x75),
        Key::F7 => Some(0x76),
        Key::F8 => Some(0x77),
        Key::F9 => Some(0x78),
        Key::F10 => Some(0x79),
        Key::F11 => Some(0x7A),
        Key::F12 => Some(0x7B),

        // Special keys
        Key::Escape => Some(0x1B), // VK_ESCAPE
        Key::Tab => Some(0x09),    // VK_TAB
        Key::Space => Some(0x20),  // VK_SPACE
        Key::Return => Some(0x0D), // VK_RETURN
        Key::Delete => Some(0x08), // VK_BACK (Backspace)

        // Navigation keys
        Key::Left => Some(0x25),     // VK_LEFT
        Key::Up => Some(0x26),       // VK_UP
        Key::Right => Some(0x27),    // VK_RIGHT
        Key::Down => Some(0x28),     // VK_DOWN
        Key::Home => Some(0x24),     // VK_HOME
        Key::End => Some(0x23),      // VK_END
        Key::PageUp => Some(0x21),   // VK_PRIOR
        Key::PageDown => Some(0x22), // VK_NEXT

        // Punctuation and symbols
        // Note: These use OEM key codes which may vary by keyboard layout
        Key::Minus => Some(0xBD),        // VK_OEM_MINUS
        Key::Equal => Some(0xBB),        // VK_OEM_PLUS (equals sign)
        Key::LeftBracket => Some(0xDB),  // VK_OEM_4
        Key::RightBracket => Some(0xDD), // VK_OEM_6
        Key::Backslash => Some(0xDC),    // VK_OEM_5
        Key::Semicolon => Some(0xBA),    // VK_OEM_1
        Key::Quote => Some(0xDE),        // VK_OEM_7
        Key::Grave => Some(0xC0),        // VK_OEM_3 (backtick/tilde)
        Key::Comma => Some(0xBC),        // VK_OEM_COMMA
        Key::Period => Some(0xBE),       // VK_OEM_PERIOD
        Key::Slash => Some(0xBF),        // VK_OEM_2
    }
}

/// Convert a Windows virtual key code to the canonical `Key` enum.
///
/// Returns `None` if the keycode doesn't map to a supported key.
///
/// # Examples
///
/// ```ignore
/// use cat_shield::input::keycodes::{Key, keycode_to_key};
///
/// assert_eq!(keycode_to_key(0x41), Some(Key::A));
/// assert_eq!(keycode_to_key(0x1B), Some(Key::Escape));
/// assert_eq!(keycode_to_key(0xFF), None);
/// ```
#[must_use]
pub const fn keycode_to_key(keycode: u32) -> Option<Key> {
    match keycode {
        // Letters
        0x41 => Some(Key::A),
        0x42 => Some(Key::B),
        0x43 => Some(Key::C),
        0x44 => Some(Key::D),
        0x45 => Some(Key::E),
        0x46 => Some(Key::F),
        0x47 => Some(Key::G),
        0x48 => Some(Key::H),
        0x49 => Some(Key::I),
        0x4A => Some(Key::J),
        0x4B => Some(Key::K),
        0x4C => Some(Key::L),
        0x4D => Some(Key::M),
        0x4E => Some(Key::N),
        0x4F => Some(Key::O),
        0x50 => Some(Key::P),
        0x51 => Some(Key::Q),
        0x52 => Some(Key::R),
        0x53 => Some(Key::S),
        0x54 => Some(Key::T),
        0x55 => Some(Key::U),
        0x56 => Some(Key::V),
        0x57 => Some(Key::W),
        0x58 => Some(Key::X),
        0x59 => Some(Key::Y),
        0x5A => Some(Key::Z),

        // Numbers
        0x30 => Some(Key::Num0),
        0x31 => Some(Key::Num1),
        0x32 => Some(Key::Num2),
        0x33 => Some(Key::Num3),
        0x34 => Some(Key::Num4),
        0x35 => Some(Key::Num5),
        0x36 => Some(Key::Num6),
        0x37 => Some(Key::Num7),
        0x38 => Some(Key::Num8),
        0x39 => Some(Key::Num9),

        // Function keys
        0x70 => Some(Key::F1),
        0x71 => Some(Key::F2),
        0x72 => Some(Key::F3),
        0x73 => Some(Key::F4),
        0x74 => Some(Key::F5),
        0x75 => Some(Key::F6),
        0x76 => Some(Key::F7),
        0x77 => Some(Key::F8),
        0x78 => Some(Key::F9),
        0x79 => Some(Key::F10),
        0x7A => Some(Key::F11),
        0x7B => Some(Key::F12),

        // Special keys
        0x1B => Some(Key::Escape),
        0x09 => Some(Key::Tab),
        0x20 => Some(Key::Space),
        0x0D => Some(Key::Return),
        0x08 => Some(Key::Delete), // VK_BACK (Backspace)

        // Navigation keys
        0x25 => Some(Key::Left),
        0x26 => Some(Key::Up),
        0x27 => Some(Key::Right),
        0x28 => Some(Key::Down),
        0x24 => Some(Key::Home),
        0x23 => Some(Key::End),
        0x21 => Some(Key::PageUp),
        0x22 => Some(Key::PageDown),

        // Punctuation and symbols (OEM keys)
        0xBD => Some(Key::Minus),
        0xBB => Some(Key::Equal),
        0xDB => Some(Key::LeftBracket),
        0xDD => Some(Key::RightBracket),
        0xDC => Some(Key::Backslash),
        0xBA => Some(Key::Semicolon),
        0xDE => Some(Key::Quote),
        0xC0 => Some(Key::Grave),
        0xBC => Some(Key::Comma),
        0xBE => Some(Key::Period),
        0xBF => Some(Key::Slash),

        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_key_to_keycode_letters() {
        assert_eq!(key_to_keycode(Key::A), Some(0x41));
        assert_eq!(key_to_keycode(Key::B), Some(0x42));
        assert_eq!(key_to_keycode(Key::Q), Some(0x51));
        assert_eq!(key_to_keycode(Key::Z), Some(0x5A));
    }

    #[test]
    fn test_key_to_keycode_numbers() {
        assert_eq!(key_to_keycode(Key::Num0), Some(0x30));
        assert_eq!(key_to_keycode(Key::Num1), Some(0x31));
        assert_eq!(key_to_keycode(Key::Num9), Some(0x39));
    }

    #[test]
    fn test_key_to_keycode_function_keys() {
        assert_eq!(key_to_keycode(Key::F1), Some(0x70));
        assert_eq!(key_to_keycode(Key::F10), Some(0x79));
        assert_eq!(key_to_keycode(Key::F11), Some(0x7A));
        assert_eq!(key_to_keycode(Key::F12), Some(0x7B));
    }

    #[test]
    fn test_key_to_keycode_special() {
        assert_eq!(key_to_keycode(Key::Escape), Some(0x1B));
        assert_eq!(key_to_keycode(Key::Return), Some(0x0D));
        assert_eq!(key_to_keycode(Key::Space), Some(0x20));
        assert_eq!(key_to_keycode(Key::Tab), Some(0x09));
        assert_eq!(key_to_keycode(Key::Delete), Some(0x08));
    }

    #[test]
    fn test_key_to_keycode_navigation() {
        assert_eq!(key_to_keycode(Key::Left), Some(0x25));
        assert_eq!(key_to_keycode(Key::Right), Some(0x27));
        assert_eq!(key_to_keycode(Key::Up), Some(0x26));
        assert_eq!(key_to_keycode(Key::Down), Some(0x28));
        assert_eq!(key_to_keycode(Key::Home), Some(0x24));
        assert_eq!(key_to_keycode(Key::End), Some(0x23));
        assert_eq!(key_to_keycode(Key::PageUp), Some(0x21));
        assert_eq!(key_to_keycode(Key::PageDown), Some(0x22));
    }

    #[test]
    fn test_key_to_keycode_punctuation() {
        assert_eq!(key_to_keycode(Key::Minus), Some(0xBD));
        assert_eq!(key_to_keycode(Key::Equal), Some(0xBB));
        assert_eq!(key_to_keycode(Key::Semicolon), Some(0xBA));
        assert_eq!(key_to_keycode(Key::Comma), Some(0xBC));
        assert_eq!(key_to_keycode(Key::Period), Some(0xBE));
        assert_eq!(key_to_keycode(Key::Slash), Some(0xBF));
    }

    #[test]
    fn test_keycode_to_key_letters() {
        assert_eq!(keycode_to_key(0x41), Some(Key::A));
        assert_eq!(keycode_to_key(0x42), Some(Key::B));
        assert_eq!(keycode_to_key(0x51), Some(Key::Q));
        assert_eq!(keycode_to_key(0x5A), Some(Key::Z));
    }

    #[test]
    fn test_keycode_to_key_numbers() {
        assert_eq!(keycode_to_key(0x30), Some(Key::Num0));
        assert_eq!(keycode_to_key(0x31), Some(Key::Num1));
        assert_eq!(keycode_to_key(0x39), Some(Key::Num9));
    }

    #[test]
    fn test_keycode_to_key_function_keys() {
        assert_eq!(keycode_to_key(0x70), Some(Key::F1));
        assert_eq!(keycode_to_key(0x79), Some(Key::F10));
        assert_eq!(keycode_to_key(0x7A), Some(Key::F11));
        assert_eq!(keycode_to_key(0x7B), Some(Key::F12));
    }

    #[test]
    fn test_keycode_to_key_special() {
        assert_eq!(keycode_to_key(0x1B), Some(Key::Escape));
        assert_eq!(keycode_to_key(0x0D), Some(Key::Return));
        assert_eq!(keycode_to_key(0x20), Some(Key::Space));
        assert_eq!(keycode_to_key(0x09), Some(Key::Tab));
        assert_eq!(keycode_to_key(0x08), Some(Key::Delete));
    }

    #[test]
    fn test_keycode_to_key_unknown() {
        assert_eq!(keycode_to_key(0xFF), None);
        assert_eq!(keycode_to_key(0x00), None);
        assert_eq!(keycode_to_key(0x10), None); // VK_SHIFT (not in our Key enum)
    }

    #[test]
    fn test_roundtrip_key_to_keycode_to_key() {
        // Test that all keys can roundtrip through keycodes
        for &key in Key::all() {
            if let Some(keycode) = key_to_keycode(key) {
                let roundtrip = keycode_to_key(keycode);
                assert_eq!(
                    roundtrip,
                    Some(key),
                    "Failed roundtrip for key {:?} with keycode 0x{:X}",
                    key,
                    keycode
                );
            }
        }
    }

    #[test]
    fn test_all_keys_have_keycodes() {
        // Verify that all keys in the enum have Windows keycodes
        for &key in Key::all() {
            assert!(
                key_to_keycode(key).is_some(),
                "Key {:?} has no Windows keycode mapping",
                key
            );
        }
    }
}
