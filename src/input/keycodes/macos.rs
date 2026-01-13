//! macOS virtual keycode mappings
//!
//! This module provides conversion between the canonical `Key` enum and
//! macOS-specific virtual keycodes (CGKeyCode).
//!
//! See: <https://developer.apple.com/documentation/coregraphics/cgkeycode>

use super::Key;

/// Convert a canonical `Key` to its macOS virtual keycode.
///
/// Returns `None` if the key is not supported on macOS (all currently supported keys are).
///
/// # Examples
///
/// ```ignore
/// use cat_shield::input::keycodes::{Key, key_to_keycode};
///
/// assert_eq!(key_to_keycode(Key::A), Some(0));
/// assert_eq!(key_to_keycode(Key::Escape), Some(53));
/// ```
#[must_use]
pub const fn key_to_keycode(key: Key) -> Option<u32> {
    match key {
        // Letters (macOS uses a non-QWERTY order internally)
        Key::A => Some(0),
        Key::S => Some(1),
        Key::D => Some(2),
        Key::F => Some(3),
        Key::H => Some(4),
        Key::G => Some(5),
        Key::Z => Some(6),
        Key::X => Some(7),
        Key::C => Some(8),
        Key::V => Some(9),
        Key::B => Some(11),
        Key::Q => Some(12),
        Key::W => Some(13),
        Key::E => Some(14),
        Key::R => Some(15),
        Key::Y => Some(16),
        Key::T => Some(17),
        Key::O => Some(31),
        Key::U => Some(32),
        Key::I => Some(34),
        Key::P => Some(35),
        Key::L => Some(37),
        Key::J => Some(38),
        Key::K => Some(40),
        Key::N => Some(45),
        Key::M => Some(46),

        // Numbers
        Key::Num1 => Some(18),
        Key::Num2 => Some(19),
        Key::Num3 => Some(20),
        Key::Num4 => Some(21),
        Key::Num5 => Some(23),
        Key::Num6 => Some(22),
        Key::Num7 => Some(26),
        Key::Num8 => Some(28),
        Key::Num9 => Some(25),
        Key::Num0 => Some(29),

        // Punctuation and symbols
        Key::Equal => Some(24),
        Key::Minus => Some(27),
        Key::RightBracket => Some(30),
        Key::LeftBracket => Some(33),
        Key::Quote => Some(39),
        Key::Semicolon => Some(41),
        Key::Backslash => Some(42),
        Key::Comma => Some(43),
        Key::Slash => Some(44),
        Key::Period => Some(47),
        Key::Grave => Some(50),

        // Special keys
        Key::Return => Some(36),
        Key::Tab => Some(48),
        Key::Space => Some(49),
        Key::Delete => Some(51),
        Key::Escape => Some(53),

        // Function keys
        Key::F1 => Some(122),
        Key::F2 => Some(120),
        Key::F3 => Some(99),
        Key::F4 => Some(118),
        Key::F5 => Some(96),
        Key::F6 => Some(97),
        Key::F7 => Some(98),
        Key::F8 => Some(100),
        Key::F9 => Some(101),
        Key::F10 => Some(109),
        Key::F11 => Some(103),
        Key::F12 => Some(111),

        // Navigation keys
        Key::Home => Some(115),
        Key::End => Some(119),
        Key::PageUp => Some(116),
        Key::PageDown => Some(121),
        Key::Left => Some(123),
        Key::Right => Some(124),
        Key::Down => Some(125),
        Key::Up => Some(126),
    }
}

/// Convert a macOS virtual keycode to the canonical `Key` enum.
///
/// Returns `None` if the keycode doesn't map to a supported key.
///
/// # Examples
///
/// ```ignore
/// use cat_shield::input::keycodes::{Key, keycode_to_key};
///
/// assert_eq!(keycode_to_key(0), Some(Key::A));
/// assert_eq!(keycode_to_key(53), Some(Key::Escape));
/// assert_eq!(keycode_to_key(999), None);
/// ```
#[must_use]
pub const fn keycode_to_key(keycode: u32) -> Option<Key> {
    match keycode {
        // Letters
        0 => Some(Key::A),
        1 => Some(Key::S),
        2 => Some(Key::D),
        3 => Some(Key::F),
        4 => Some(Key::H),
        5 => Some(Key::G),
        6 => Some(Key::Z),
        7 => Some(Key::X),
        8 => Some(Key::C),
        9 => Some(Key::V),
        11 => Some(Key::B),
        12 => Some(Key::Q),
        13 => Some(Key::W),
        14 => Some(Key::E),
        15 => Some(Key::R),
        16 => Some(Key::Y),
        17 => Some(Key::T),
        31 => Some(Key::O),
        32 => Some(Key::U),
        34 => Some(Key::I),
        35 => Some(Key::P),
        37 => Some(Key::L),
        38 => Some(Key::J),
        40 => Some(Key::K),
        45 => Some(Key::N),
        46 => Some(Key::M),

        // Numbers
        18 => Some(Key::Num1),
        19 => Some(Key::Num2),
        20 => Some(Key::Num3),
        21 => Some(Key::Num4),
        22 => Some(Key::Num6),
        23 => Some(Key::Num5),
        25 => Some(Key::Num9),
        26 => Some(Key::Num7),
        28 => Some(Key::Num8),
        29 => Some(Key::Num0),

        // Punctuation and symbols
        24 => Some(Key::Equal),
        27 => Some(Key::Minus),
        30 => Some(Key::RightBracket),
        33 => Some(Key::LeftBracket),
        39 => Some(Key::Quote),
        41 => Some(Key::Semicolon),
        42 => Some(Key::Backslash),
        43 => Some(Key::Comma),
        44 => Some(Key::Slash),
        47 => Some(Key::Period),
        50 => Some(Key::Grave),

        // Special keys
        36 => Some(Key::Return),
        48 => Some(Key::Tab),
        49 => Some(Key::Space),
        51 => Some(Key::Delete),
        53 => Some(Key::Escape),

        // Function keys
        122 => Some(Key::F1),
        120 => Some(Key::F2),
        99 => Some(Key::F3),
        118 => Some(Key::F4),
        96 => Some(Key::F5),
        97 => Some(Key::F6),
        98 => Some(Key::F7),
        100 => Some(Key::F8),
        101 => Some(Key::F9),
        109 => Some(Key::F10),
        103 => Some(Key::F11),
        111 => Some(Key::F12),

        // Navigation keys
        115 => Some(Key::Home),
        119 => Some(Key::End),
        116 => Some(Key::PageUp),
        121 => Some(Key::PageDown),
        123 => Some(Key::Left),
        124 => Some(Key::Right),
        125 => Some(Key::Down),
        126 => Some(Key::Up),

        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_key_to_keycode_letters() {
        assert_eq!(key_to_keycode(Key::A), Some(0));
        assert_eq!(key_to_keycode(Key::S), Some(1));
        assert_eq!(key_to_keycode(Key::Q), Some(12));
        assert_eq!(key_to_keycode(Key::U), Some(32));
        assert_eq!(key_to_keycode(Key::Z), Some(6));
    }

    #[test]
    fn test_key_to_keycode_numbers() {
        assert_eq!(key_to_keycode(Key::Num0), Some(29));
        assert_eq!(key_to_keycode(Key::Num1), Some(18));
        assert_eq!(key_to_keycode(Key::Num9), Some(25));
    }

    #[test]
    fn test_key_to_keycode_function_keys() {
        assert_eq!(key_to_keycode(Key::F1), Some(122));
        assert_eq!(key_to_keycode(Key::F10), Some(109));
        assert_eq!(key_to_keycode(Key::F11), Some(103));
        assert_eq!(key_to_keycode(Key::F12), Some(111));
    }

    #[test]
    fn test_key_to_keycode_special() {
        assert_eq!(key_to_keycode(Key::Escape), Some(53));
        assert_eq!(key_to_keycode(Key::Return), Some(36));
        assert_eq!(key_to_keycode(Key::Space), Some(49));
        assert_eq!(key_to_keycode(Key::Tab), Some(48));
        assert_eq!(key_to_keycode(Key::Delete), Some(51));
    }

    #[test]
    fn test_key_to_keycode_navigation() {
        assert_eq!(key_to_keycode(Key::Left), Some(123));
        assert_eq!(key_to_keycode(Key::Right), Some(124));
        assert_eq!(key_to_keycode(Key::Up), Some(126));
        assert_eq!(key_to_keycode(Key::Down), Some(125));
        assert_eq!(key_to_keycode(Key::Home), Some(115));
        assert_eq!(key_to_keycode(Key::End), Some(119));
        assert_eq!(key_to_keycode(Key::PageUp), Some(116));
        assert_eq!(key_to_keycode(Key::PageDown), Some(121));
    }

    #[test]
    fn test_keycode_to_key_letters() {
        assert_eq!(keycode_to_key(0), Some(Key::A));
        assert_eq!(keycode_to_key(1), Some(Key::S));
        assert_eq!(keycode_to_key(12), Some(Key::Q));
        assert_eq!(keycode_to_key(32), Some(Key::U));
        assert_eq!(keycode_to_key(6), Some(Key::Z));
    }

    #[test]
    fn test_keycode_to_key_numbers() {
        assert_eq!(keycode_to_key(29), Some(Key::Num0));
        assert_eq!(keycode_to_key(18), Some(Key::Num1));
        assert_eq!(keycode_to_key(25), Some(Key::Num9));
    }

    #[test]
    fn test_keycode_to_key_function_keys() {
        assert_eq!(keycode_to_key(122), Some(Key::F1));
        assert_eq!(keycode_to_key(109), Some(Key::F10));
        assert_eq!(keycode_to_key(103), Some(Key::F11));
        assert_eq!(keycode_to_key(111), Some(Key::F12));
    }

    #[test]
    fn test_keycode_to_key_special() {
        assert_eq!(keycode_to_key(53), Some(Key::Escape));
        assert_eq!(keycode_to_key(36), Some(Key::Return));
        assert_eq!(keycode_to_key(49), Some(Key::Space));
        assert_eq!(keycode_to_key(48), Some(Key::Tab));
        assert_eq!(keycode_to_key(51), Some(Key::Delete));
    }

    #[test]
    fn test_keycode_to_key_unknown() {
        assert_eq!(keycode_to_key(999), None);
        assert_eq!(keycode_to_key(10), None); // Unused keycode
        assert_eq!(keycode_to_key(200), None);
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
                    "Failed roundtrip for key {:?} with keycode {}",
                    key,
                    keycode
                );
            }
        }
    }

    #[test]
    fn test_all_keys_have_keycodes() {
        // Verify that all keys in the enum have macOS keycodes
        for &key in Key::all() {
            assert!(
                key_to_keycode(key).is_some(),
                "Key {:?} has no macOS keycode mapping",
                key
            );
        }
    }
}
