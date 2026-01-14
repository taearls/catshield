//! Linux X11 keysym mappings
//!
//! This module provides conversion between the canonical `Key` enum and
//! Linux-specific X11 keysyms.
//!
//! See: <https://cgit.freedesktop.org/xorg/proto/x11proto/tree/keysymdef.h>
//!
//! # X11 Keysym Ranges
//!
//! - Latin letters (lowercase): XK_a (0x61) - XK_z (0x7a)
//! - Latin letters (uppercase): XK_A (0x41) - XK_Z (0x5a)
//! - Digits: XK_0 (0x30) - XK_9 (0x39)
//! - Function keys: XK_F1 (0xffbe) - XK_F12 (0xffc9)
//! - Special keys: XK_Escape (0xff1b), XK_Return (0xff0d), etc.
//! - Navigation: XK_Left (0xff51) - XK_Down (0xff54), etc.
//!
//! # Note on Case Sensitivity
//!
//! X11 keysyms distinguish between uppercase and lowercase letters.
//! This implementation maps both cases to the same canonical Key variant.
//! When converting from Key to keysym, lowercase keysyms are used by default.

use super::Key;

// =============================================================================
// X11 Keysym Constants
// =============================================================================
// These constants are defined in X11/keysymdef.h

// TTY function keys (0xFF00-0xFFFF)
const XK_BACKSPACE: u32 = 0xff08;
const XK_TAB: u32 = 0xff09;
const XK_RETURN: u32 = 0xff0d;
const XK_ESCAPE: u32 = 0xff1b;
const XK_DELETE: u32 = 0xffff;

// Cursor control & motion (0xFF50-0xFF58)
const XK_HOME: u32 = 0xff50;
const XK_LEFT: u32 = 0xff51;
const XK_UP: u32 = 0xff52;
const XK_RIGHT: u32 = 0xff53;
const XK_DOWN: u32 = 0xff54;
const XK_PAGE_UP: u32 = 0xff55;
const XK_PAGE_DOWN: u32 = 0xff56;
const XK_END: u32 = 0xff57;

// Function keys (0xFFBE-0xFFCC)
const XK_F1: u32 = 0xffbe;
const XK_F2: u32 = 0xffbf;
const XK_F3: u32 = 0xffc0;
const XK_F4: u32 = 0xffc1;
const XK_F5: u32 = 0xffc2;
const XK_F6: u32 = 0xffc3;
const XK_F7: u32 = 0xffc4;
const XK_F8: u32 = 0xffc5;
const XK_F9: u32 = 0xffc6;
const XK_F10: u32 = 0xffc7;
const XK_F11: u32 = 0xffc8;
const XK_F12: u32 = 0xffc9;

// Latin 1 (0x0020-0x007E) - ASCII printable range
const XK_SPACE: u32 = 0x0020;

// Punctuation and symbols (ASCII values)
const XK_APOSTROPHE: u32 = 0x0027; // ' (single quote)
const XK_COMMA: u32 = 0x002c; // ,
const XK_MINUS: u32 = 0x002d; // -
const XK_PERIOD: u32 = 0x002e; // .
const XK_SLASH: u32 = 0x002f; // /
const XK_SEMICOLON: u32 = 0x003b; // ;
const XK_EQUAL: u32 = 0x003d; // =
const XK_BRACKETLEFT: u32 = 0x005b; // [
const XK_BACKSLASH: u32 = 0x005c; // \
const XK_BRACKETRIGHT: u32 = 0x005d; // ]
const XK_GRAVE: u32 = 0x0060; // ` (backtick)

// Digits (ASCII values)
const XK_0: u32 = 0x0030;
const XK_1: u32 = 0x0031;
const XK_2: u32 = 0x0032;
const XK_3: u32 = 0x0033;
const XK_4: u32 = 0x0034;
const XK_5: u32 = 0x0035;
const XK_6: u32 = 0x0036;
const XK_7: u32 = 0x0037;
const XK_8: u32 = 0x0038;
const XK_9: u32 = 0x0039;

// Lowercase letters (ASCII values)
const XK_A_LOWER: u32 = 0x0061;
const XK_B_LOWER: u32 = 0x0062;
const XK_C_LOWER: u32 = 0x0063;
const XK_D_LOWER: u32 = 0x0064;
const XK_E_LOWER: u32 = 0x0065;
const XK_F_LOWER: u32 = 0x0066;
const XK_G_LOWER: u32 = 0x0067;
const XK_H_LOWER: u32 = 0x0068;
const XK_I_LOWER: u32 = 0x0069;
const XK_J_LOWER: u32 = 0x006a;
const XK_K_LOWER: u32 = 0x006b;
const XK_L_LOWER: u32 = 0x006c;
const XK_M_LOWER: u32 = 0x006d;
const XK_N_LOWER: u32 = 0x006e;
const XK_O_LOWER: u32 = 0x006f;
const XK_P_LOWER: u32 = 0x0070;
const XK_Q_LOWER: u32 = 0x0071;
const XK_R_LOWER: u32 = 0x0072;
const XK_S_LOWER: u32 = 0x0073;
const XK_T_LOWER: u32 = 0x0074;
const XK_U_LOWER: u32 = 0x0075;
const XK_V_LOWER: u32 = 0x0076;
const XK_W_LOWER: u32 = 0x0077;
const XK_X_LOWER: u32 = 0x0078;
const XK_Y_LOWER: u32 = 0x0079;
const XK_Z_LOWER: u32 = 0x007a;

// Uppercase letters (ASCII values)
const XK_A_UPPER: u32 = 0x0041;
const XK_B_UPPER: u32 = 0x0042;
const XK_C_UPPER: u32 = 0x0043;
const XK_D_UPPER: u32 = 0x0044;
const XK_E_UPPER: u32 = 0x0045;
const XK_F_UPPER: u32 = 0x0046;
const XK_G_UPPER: u32 = 0x0047;
const XK_H_UPPER: u32 = 0x0048;
const XK_I_UPPER: u32 = 0x0049;
const XK_J_UPPER: u32 = 0x004a;
const XK_K_UPPER: u32 = 0x004b;
const XK_L_UPPER: u32 = 0x004c;
const XK_M_UPPER: u32 = 0x004d;
const XK_N_UPPER: u32 = 0x004e;
const XK_O_UPPER: u32 = 0x004f;
const XK_P_UPPER: u32 = 0x0050;
const XK_Q_UPPER: u32 = 0x0051;
const XK_R_UPPER: u32 = 0x0052;
const XK_S_UPPER: u32 = 0x0053;
const XK_T_UPPER: u32 = 0x0054;
const XK_U_UPPER: u32 = 0x0055;
const XK_V_UPPER: u32 = 0x0056;
const XK_W_UPPER: u32 = 0x0057;
const XK_X_UPPER: u32 = 0x0058;
const XK_Y_UPPER: u32 = 0x0059;
const XK_Z_UPPER: u32 = 0x005a;

/// Convert a canonical `Key` to its X11 keysym.
///
/// Returns the lowercase keysym for letter keys.
///
/// # Examples
///
/// ```ignore
/// use cat_shield::input::keycodes::{Key, key_to_keycode};
///
/// assert_eq!(key_to_keycode(Key::A), Some(0x61)); // XK_a (lowercase)
/// assert_eq!(key_to_keycode(Key::Escape), Some(0xff1b)); // XK_Escape
/// ```
#[must_use]
pub const fn key_to_keycode(key: Key) -> Option<u32> {
    match key {
        // Letters (using lowercase keysyms)
        Key::A => Some(XK_A_LOWER),
        Key::B => Some(XK_B_LOWER),
        Key::C => Some(XK_C_LOWER),
        Key::D => Some(XK_D_LOWER),
        Key::E => Some(XK_E_LOWER),
        Key::F => Some(XK_F_LOWER),
        Key::G => Some(XK_G_LOWER),
        Key::H => Some(XK_H_LOWER),
        Key::I => Some(XK_I_LOWER),
        Key::J => Some(XK_J_LOWER),
        Key::K => Some(XK_K_LOWER),
        Key::L => Some(XK_L_LOWER),
        Key::M => Some(XK_M_LOWER),
        Key::N => Some(XK_N_LOWER),
        Key::O => Some(XK_O_LOWER),
        Key::P => Some(XK_P_LOWER),
        Key::Q => Some(XK_Q_LOWER),
        Key::R => Some(XK_R_LOWER),
        Key::S => Some(XK_S_LOWER),
        Key::T => Some(XK_T_LOWER),
        Key::U => Some(XK_U_LOWER),
        Key::V => Some(XK_V_LOWER),
        Key::W => Some(XK_W_LOWER),
        Key::X => Some(XK_X_LOWER),
        Key::Y => Some(XK_Y_LOWER),
        Key::Z => Some(XK_Z_LOWER),

        // Numbers
        Key::Num0 => Some(XK_0),
        Key::Num1 => Some(XK_1),
        Key::Num2 => Some(XK_2),
        Key::Num3 => Some(XK_3),
        Key::Num4 => Some(XK_4),
        Key::Num5 => Some(XK_5),
        Key::Num6 => Some(XK_6),
        Key::Num7 => Some(XK_7),
        Key::Num8 => Some(XK_8),
        Key::Num9 => Some(XK_9),

        // Function keys
        Key::F1 => Some(XK_F1),
        Key::F2 => Some(XK_F2),
        Key::F3 => Some(XK_F3),
        Key::F4 => Some(XK_F4),
        Key::F5 => Some(XK_F5),
        Key::F6 => Some(XK_F6),
        Key::F7 => Some(XK_F7),
        Key::F8 => Some(XK_F8),
        Key::F9 => Some(XK_F9),
        Key::F10 => Some(XK_F10),
        Key::F11 => Some(XK_F11),
        Key::F12 => Some(XK_F12),

        // Special keys
        Key::Escape => Some(XK_ESCAPE),
        Key::Tab => Some(XK_TAB),
        Key::Space => Some(XK_SPACE),
        Key::Return => Some(XK_RETURN),
        Key::Delete => Some(XK_BACKSPACE), // Map Delete to Backspace (common behavior)

        // Navigation keys
        Key::Left => Some(XK_LEFT),
        Key::Right => Some(XK_RIGHT),
        Key::Up => Some(XK_UP),
        Key::Down => Some(XK_DOWN),
        Key::Home => Some(XK_HOME),
        Key::End => Some(XK_END),
        Key::PageUp => Some(XK_PAGE_UP),
        Key::PageDown => Some(XK_PAGE_DOWN),

        // Punctuation and symbols
        Key::Minus => Some(XK_MINUS),
        Key::Equal => Some(XK_EQUAL),
        Key::LeftBracket => Some(XK_BRACKETLEFT),
        Key::RightBracket => Some(XK_BRACKETRIGHT),
        Key::Backslash => Some(XK_BACKSLASH),
        Key::Semicolon => Some(XK_SEMICOLON),
        Key::Quote => Some(XK_APOSTROPHE),
        Key::Grave => Some(XK_GRAVE),
        Key::Comma => Some(XK_COMMA),
        Key::Period => Some(XK_PERIOD),
        Key::Slash => Some(XK_SLASH),
    }
}

/// Convert an X11 keysym to the canonical `Key` enum.
///
/// Handles both uppercase and lowercase letter keysyms, mapping them
/// to the same Key variant.
///
/// # Examples
///
/// ```ignore
/// use cat_shield::input::keycodes::{Key, keycode_to_key};
///
/// assert_eq!(keycode_to_key(0x61), Some(Key::A)); // XK_a (lowercase)
/// assert_eq!(keycode_to_key(0x41), Some(Key::A)); // XK_A (uppercase)
/// assert_eq!(keycode_to_key(0xff1b), Some(Key::Escape)); // XK_Escape
/// assert_eq!(keycode_to_key(0xffff), None); // Unknown keysym
/// ```
#[must_use]
pub const fn keycode_to_key(keycode: u32) -> Option<Key> {
    match keycode {
        // Lowercase letters
        XK_A_LOWER => Some(Key::A),
        XK_B_LOWER => Some(Key::B),
        XK_C_LOWER => Some(Key::C),
        XK_D_LOWER => Some(Key::D),
        XK_E_LOWER => Some(Key::E),
        XK_F_LOWER => Some(Key::F),
        XK_G_LOWER => Some(Key::G),
        XK_H_LOWER => Some(Key::H),
        XK_I_LOWER => Some(Key::I),
        XK_J_LOWER => Some(Key::J),
        XK_K_LOWER => Some(Key::K),
        XK_L_LOWER => Some(Key::L),
        XK_M_LOWER => Some(Key::M),
        XK_N_LOWER => Some(Key::N),
        XK_O_LOWER => Some(Key::O),
        XK_P_LOWER => Some(Key::P),
        XK_Q_LOWER => Some(Key::Q),
        XK_R_LOWER => Some(Key::R),
        XK_S_LOWER => Some(Key::S),
        XK_T_LOWER => Some(Key::T),
        XK_U_LOWER => Some(Key::U),
        XK_V_LOWER => Some(Key::V),
        XK_W_LOWER => Some(Key::W),
        XK_X_LOWER => Some(Key::X),
        XK_Y_LOWER => Some(Key::Y),
        XK_Z_LOWER => Some(Key::Z),

        // Uppercase letters (map to same Key as lowercase)
        XK_A_UPPER => Some(Key::A),
        XK_B_UPPER => Some(Key::B),
        XK_C_UPPER => Some(Key::C),
        XK_D_UPPER => Some(Key::D),
        XK_E_UPPER => Some(Key::E),
        XK_F_UPPER => Some(Key::F),
        XK_G_UPPER => Some(Key::G),
        XK_H_UPPER => Some(Key::H),
        XK_I_UPPER => Some(Key::I),
        XK_J_UPPER => Some(Key::J),
        XK_K_UPPER => Some(Key::K),
        XK_L_UPPER => Some(Key::L),
        XK_M_UPPER => Some(Key::M),
        XK_N_UPPER => Some(Key::N),
        XK_O_UPPER => Some(Key::O),
        XK_P_UPPER => Some(Key::P),
        XK_Q_UPPER => Some(Key::Q),
        XK_R_UPPER => Some(Key::R),
        XK_S_UPPER => Some(Key::S),
        XK_T_UPPER => Some(Key::T),
        XK_U_UPPER => Some(Key::U),
        XK_V_UPPER => Some(Key::V),
        XK_W_UPPER => Some(Key::W),
        XK_X_UPPER => Some(Key::X),
        XK_Y_UPPER => Some(Key::Y),
        XK_Z_UPPER => Some(Key::Z),

        // Numbers
        XK_0 => Some(Key::Num0),
        XK_1 => Some(Key::Num1),
        XK_2 => Some(Key::Num2),
        XK_3 => Some(Key::Num3),
        XK_4 => Some(Key::Num4),
        XK_5 => Some(Key::Num5),
        XK_6 => Some(Key::Num6),
        XK_7 => Some(Key::Num7),
        XK_8 => Some(Key::Num8),
        XK_9 => Some(Key::Num9),

        // Function keys
        XK_F1 => Some(Key::F1),
        XK_F2 => Some(Key::F2),
        XK_F3 => Some(Key::F3),
        XK_F4 => Some(Key::F4),
        XK_F5 => Some(Key::F5),
        XK_F6 => Some(Key::F6),
        XK_F7 => Some(Key::F7),
        XK_F8 => Some(Key::F8),
        XK_F9 => Some(Key::F9),
        XK_F10 => Some(Key::F10),
        XK_F11 => Some(Key::F11),
        XK_F12 => Some(Key::F12),

        // Special keys
        XK_ESCAPE => Some(Key::Escape),
        XK_TAB => Some(Key::Tab),
        XK_SPACE => Some(Key::Space),
        XK_RETURN => Some(Key::Return),
        XK_BACKSPACE => Some(Key::Delete),
        XK_DELETE => Some(Key::Delete), // Both XK_BackSpace and XK_Delete map to Delete

        // Navigation keys
        XK_LEFT => Some(Key::Left),
        XK_RIGHT => Some(Key::Right),
        XK_UP => Some(Key::Up),
        XK_DOWN => Some(Key::Down),
        XK_HOME => Some(Key::Home),
        XK_END => Some(Key::End),
        XK_PAGE_UP => Some(Key::PageUp),
        XK_PAGE_DOWN => Some(Key::PageDown),

        // Punctuation and symbols
        XK_MINUS => Some(Key::Minus),
        XK_EQUAL => Some(Key::Equal),
        XK_BRACKETLEFT => Some(Key::LeftBracket),
        XK_BRACKETRIGHT => Some(Key::RightBracket),
        XK_BACKSLASH => Some(Key::Backslash),
        XK_SEMICOLON => Some(Key::Semicolon),
        XK_APOSTROPHE => Some(Key::Quote),
        XK_GRAVE => Some(Key::Grave),
        XK_COMMA => Some(Key::Comma),
        XK_PERIOD => Some(Key::Period),
        XK_SLASH => Some(Key::Slash),

        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_key_to_keycode_letters() {
        assert_eq!(key_to_keycode(Key::A), Some(0x61)); // XK_a
        assert_eq!(key_to_keycode(Key::B), Some(0x62)); // XK_b
        assert_eq!(key_to_keycode(Key::Q), Some(0x71)); // XK_q
        assert_eq!(key_to_keycode(Key::Z), Some(0x7a)); // XK_z
    }

    #[test]
    fn test_key_to_keycode_numbers() {
        assert_eq!(key_to_keycode(Key::Num0), Some(0x30)); // XK_0
        assert_eq!(key_to_keycode(Key::Num1), Some(0x31)); // XK_1
        assert_eq!(key_to_keycode(Key::Num9), Some(0x39)); // XK_9
    }

    #[test]
    fn test_key_to_keycode_function_keys() {
        assert_eq!(key_to_keycode(Key::F1), Some(0xffbe)); // XK_F1
        assert_eq!(key_to_keycode(Key::F10), Some(0xffc7)); // XK_F10
        assert_eq!(key_to_keycode(Key::F11), Some(0xffc8)); // XK_F11
        assert_eq!(key_to_keycode(Key::F12), Some(0xffc9)); // XK_F12
    }

    #[test]
    fn test_key_to_keycode_special() {
        assert_eq!(key_to_keycode(Key::Escape), Some(0xff1b)); // XK_Escape
        assert_eq!(key_to_keycode(Key::Return), Some(0xff0d)); // XK_Return
        assert_eq!(key_to_keycode(Key::Space), Some(0x20)); // XK_space
        assert_eq!(key_to_keycode(Key::Tab), Some(0xff09)); // XK_Tab
        assert_eq!(key_to_keycode(Key::Delete), Some(0xff08)); // XK_BackSpace
    }

    #[test]
    fn test_key_to_keycode_navigation() {
        assert_eq!(key_to_keycode(Key::Left), Some(0xff51)); // XK_Left
        assert_eq!(key_to_keycode(Key::Right), Some(0xff53)); // XK_Right
        assert_eq!(key_to_keycode(Key::Up), Some(0xff52)); // XK_Up
        assert_eq!(key_to_keycode(Key::Down), Some(0xff54)); // XK_Down
        assert_eq!(key_to_keycode(Key::Home), Some(0xff50)); // XK_Home
        assert_eq!(key_to_keycode(Key::End), Some(0xff57)); // XK_End
        assert_eq!(key_to_keycode(Key::PageUp), Some(0xff55)); // XK_Page_Up
        assert_eq!(key_to_keycode(Key::PageDown), Some(0xff56)); // XK_Page_Down
    }

    #[test]
    fn test_key_to_keycode_punctuation() {
        assert_eq!(key_to_keycode(Key::Minus), Some(0x2d)); // XK_minus
        assert_eq!(key_to_keycode(Key::Equal), Some(0x3d)); // XK_equal
        assert_eq!(key_to_keycode(Key::Semicolon), Some(0x3b)); // XK_semicolon
        assert_eq!(key_to_keycode(Key::Comma), Some(0x2c)); // XK_comma
        assert_eq!(key_to_keycode(Key::Period), Some(0x2e)); // XK_period
        assert_eq!(key_to_keycode(Key::Slash), Some(0x2f)); // XK_slash
        assert_eq!(key_to_keycode(Key::Grave), Some(0x60)); // XK_grave
        assert_eq!(key_to_keycode(Key::Quote), Some(0x27)); // XK_apostrophe
        assert_eq!(key_to_keycode(Key::LeftBracket), Some(0x5b)); // XK_bracketleft
        assert_eq!(key_to_keycode(Key::RightBracket), Some(0x5d)); // XK_bracketright
        assert_eq!(key_to_keycode(Key::Backslash), Some(0x5c)); // XK_backslash
    }

    #[test]
    fn test_keycode_to_key_lowercase_letters() {
        assert_eq!(keycode_to_key(0x61), Some(Key::A)); // XK_a
        assert_eq!(keycode_to_key(0x62), Some(Key::B)); // XK_b
        assert_eq!(keycode_to_key(0x71), Some(Key::Q)); // XK_q
        assert_eq!(keycode_to_key(0x7a), Some(Key::Z)); // XK_z
    }

    #[test]
    fn test_keycode_to_key_uppercase_letters() {
        // Uppercase letters should map to the same Key as lowercase
        assert_eq!(keycode_to_key(0x41), Some(Key::A)); // XK_A
        assert_eq!(keycode_to_key(0x42), Some(Key::B)); // XK_B
        assert_eq!(keycode_to_key(0x51), Some(Key::Q)); // XK_Q
        assert_eq!(keycode_to_key(0x5a), Some(Key::Z)); // XK_Z
    }

    #[test]
    fn test_keycode_to_key_numbers() {
        assert_eq!(keycode_to_key(0x30), Some(Key::Num0)); // XK_0
        assert_eq!(keycode_to_key(0x31), Some(Key::Num1)); // XK_1
        assert_eq!(keycode_to_key(0x39), Some(Key::Num9)); // XK_9
    }

    #[test]
    fn test_keycode_to_key_function_keys() {
        assert_eq!(keycode_to_key(0xffbe), Some(Key::F1)); // XK_F1
        assert_eq!(keycode_to_key(0xffc7), Some(Key::F10)); // XK_F10
        assert_eq!(keycode_to_key(0xffc8), Some(Key::F11)); // XK_F11
        assert_eq!(keycode_to_key(0xffc9), Some(Key::F12)); // XK_F12
    }

    #[test]
    fn test_keycode_to_key_special() {
        assert_eq!(keycode_to_key(0xff1b), Some(Key::Escape)); // XK_Escape
        assert_eq!(keycode_to_key(0xff0d), Some(Key::Return)); // XK_Return
        assert_eq!(keycode_to_key(0x20), Some(Key::Space)); // XK_space
        assert_eq!(keycode_to_key(0xff09), Some(Key::Tab)); // XK_Tab
        assert_eq!(keycode_to_key(0xff08), Some(Key::Delete)); // XK_BackSpace
        assert_eq!(keycode_to_key(0xffff), Some(Key::Delete)); // XK_Delete
    }

    #[test]
    fn test_keycode_to_key_unknown() {
        assert_eq!(keycode_to_key(0x0000), None);
        assert_eq!(keycode_to_key(0xff00), None); // Undefined in TTY range
        assert_eq!(keycode_to_key(0xfffe), None); // Not a valid keysym
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
                    "Failed roundtrip for key {:?} with keycode 0x{:04x}",
                    key,
                    keycode
                );
            }
        }
    }

    #[test]
    fn test_all_keys_have_keycodes() {
        // Verify that all keys in the enum have X11 keysym mappings
        for &key in Key::all() {
            assert!(
                key_to_keycode(key).is_some(),
                "Key {:?} has no X11 keysym mapping",
                key
            );
        }
    }

    #[test]
    fn test_case_insensitive_letter_mapping() {
        // Both uppercase and lowercase keysyms should map to the same Key
        for (lower, upper, key) in [
            (0x61, 0x41, Key::A),
            (0x62, 0x42, Key::B),
            (0x63, 0x43, Key::C),
            (0x7a, 0x5a, Key::Z),
        ] {
            assert_eq!(
                keycode_to_key(lower),
                Some(key),
                "Lowercase keysym 0x{:02x} should map to {:?}",
                lower,
                key
            );
            assert_eq!(
                keycode_to_key(upper),
                Some(key),
                "Uppercase keysym 0x{:02x} should map to {:?}",
                upper,
                key
            );
        }
    }
}
