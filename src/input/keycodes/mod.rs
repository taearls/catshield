//! Platform-agnostic keycode system for Cat Shield
//!
//! This module provides a canonical `Key` enum that represents all supported keys
//! in a platform-independent way, along with platform-specific mapping functions.
//!
//! # Architecture
//!
//! The keycode system is split into:
//! - `Key` enum: Platform-agnostic key representation
//! - `macos.rs`: macOS-specific virtual keycode mappings
//! - `windows.rs`: Windows-specific virtual key mappings (stub)
//! - `linux.rs`: Linux-specific keycode mappings (stub)
//!
//! # Usage
//!
//! ```ignore
//! use cat_shield::input::keycodes::{Key, key_from_name, key_to_name};
//!
//! // Parse a key name to the canonical Key enum
//! let key = key_from_name("escape").unwrap();
//! assert_eq!(key, Key::Escape);
//!
//! // Convert back to display name
//! let name = key_to_name(key);
//! assert_eq!(name, "Escape");
//! ```

#[cfg(target_os = "macos")]
mod macos;

#[cfg(target_os = "windows")]
mod windows;

#[cfg(target_os = "linux")]
mod linux;

/// Platform-agnostic key representation.
///
/// This enum covers all keys that Cat Shield supports across platforms.
/// Each variant represents a physical key on the keyboard.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Key {
    // Letters (A-Z)
    A,
    B,
    C,
    D,
    E,
    F,
    G,
    H,
    I,
    J,
    K,
    L,
    M,
    N,
    O,
    P,
    Q,
    R,
    S,
    T,
    U,
    V,
    W,
    X,
    Y,
    Z,

    // Numbers (0-9)
    Num0,
    Num1,
    Num2,
    Num3,
    Num4,
    Num5,
    Num6,
    Num7,
    Num8,
    Num9,

    // Function keys (F1-F12)
    F1,
    F2,
    F3,
    F4,
    F5,
    F6,
    F7,
    F8,
    F9,
    F10,
    F11,
    F12,

    // Special keys
    Escape,
    Tab,
    Space,
    Return,
    Delete,

    // Navigation keys
    Left,
    Right,
    Up,
    Down,
    Home,
    End,
    PageUp,
    PageDown,

    // Punctuation and symbols
    Minus,
    Equal,
    LeftBracket,
    RightBracket,
    Backslash,
    Semicolon,
    Quote,
    Grave,
    Comma,
    Period,
    Slash,
}

impl Key {
    /// Returns all supported keys.
    #[must_use]
    pub fn all() -> &'static [Key] {
        use Key::*;
        &[
            A,
            B,
            C,
            D,
            E,
            F,
            G,
            H,
            I,
            J,
            K,
            L,
            M,
            N,
            O,
            P,
            Q,
            R,
            S,
            T,
            U,
            V,
            W,
            X,
            Y,
            Z,
            Num0,
            Num1,
            Num2,
            Num3,
            Num4,
            Num5,
            Num6,
            Num7,
            Num8,
            Num9,
            F1,
            F2,
            F3,
            F4,
            F5,
            F6,
            F7,
            F8,
            F9,
            F10,
            F11,
            F12,
            Escape,
            Tab,
            Space,
            Return,
            Delete,
            Left,
            Right,
            Up,
            Down,
            Home,
            End,
            PageUp,
            PageDown,
            Minus,
            Equal,
            LeftBracket,
            RightBracket,
            Backslash,
            Semicolon,
            Quote,
            Grave,
            Comma,
            Period,
            Slash,
        ]
    }

    /// Returns true if this key is a letter (A-Z).
    #[must_use]
    pub const fn is_letter(self) -> bool {
        matches!(
            self,
            Key::A
                | Key::B
                | Key::C
                | Key::D
                | Key::E
                | Key::F
                | Key::G
                | Key::H
                | Key::I
                | Key::J
                | Key::K
                | Key::L
                | Key::M
                | Key::N
                | Key::O
                | Key::P
                | Key::Q
                | Key::R
                | Key::S
                | Key::T
                | Key::U
                | Key::V
                | Key::W
                | Key::X
                | Key::Y
                | Key::Z
        )
    }

    /// Returns true if this key is a number (0-9).
    #[must_use]
    pub const fn is_number(self) -> bool {
        matches!(
            self,
            Key::Num0
                | Key::Num1
                | Key::Num2
                | Key::Num3
                | Key::Num4
                | Key::Num5
                | Key::Num6
                | Key::Num7
                | Key::Num8
                | Key::Num9
        )
    }

    /// Returns true if this key is a function key (F1-F12).
    #[must_use]
    pub const fn is_function_key(self) -> bool {
        matches!(
            self,
            Key::F1
                | Key::F2
                | Key::F3
                | Key::F4
                | Key::F5
                | Key::F6
                | Key::F7
                | Key::F8
                | Key::F9
                | Key::F10
                | Key::F11
                | Key::F12
        )
    }

    /// Returns true if this key is a navigation key (arrows, home, end, page up/down).
    #[must_use]
    pub const fn is_navigation(self) -> bool {
        matches!(
            self,
            Key::Left
                | Key::Right
                | Key::Up
                | Key::Down
                | Key::Home
                | Key::End
                | Key::PageUp
                | Key::PageDown
        )
    }
}

/// Parse a key name string to the canonical `Key` enum.
///
/// This function is case-insensitive and supports various aliases for keys.
///
/// # Examples
///
/// ```ignore
/// use cat_shield::input::keycodes::key_from_name;
///
/// assert_eq!(key_from_name("a"), Some(Key::A));
/// assert_eq!(key_from_name("Escape"), Some(Key::Escape));
/// assert_eq!(key_from_name("esc"), Some(Key::Escape)); // alias
/// assert_eq!(key_from_name("unknown"), None);
/// ```
#[must_use]
pub fn key_from_name(name: &str) -> Option<Key> {
    match name.to_lowercase().as_str() {
        // Letters
        "a" => Some(Key::A),
        "b" => Some(Key::B),
        "c" => Some(Key::C),
        "d" => Some(Key::D),
        "e" => Some(Key::E),
        "f" => Some(Key::F),
        "g" => Some(Key::G),
        "h" => Some(Key::H),
        "i" => Some(Key::I),
        "j" => Some(Key::J),
        "k" => Some(Key::K),
        "l" => Some(Key::L),
        "m" => Some(Key::M),
        "n" => Some(Key::N),
        "o" => Some(Key::O),
        "p" => Some(Key::P),
        "q" => Some(Key::Q),
        "r" => Some(Key::R),
        "s" => Some(Key::S),
        "t" => Some(Key::T),
        "u" => Some(Key::U),
        "v" => Some(Key::V),
        "w" => Some(Key::W),
        "x" => Some(Key::X),
        "y" => Some(Key::Y),
        "z" => Some(Key::Z),

        // Numbers (accept both digit and symbol aliases)
        "0" | ")" => Some(Key::Num0),
        "1" | "!" => Some(Key::Num1),
        "2" | "@" => Some(Key::Num2),
        "3" | "#" => Some(Key::Num3),
        "4" | "$" => Some(Key::Num4),
        "5" | "%" => Some(Key::Num5),
        "6" | "^" => Some(Key::Num6),
        "7" | "&" => Some(Key::Num7),
        "8" | "*" => Some(Key::Num8),
        "9" | "(" => Some(Key::Num9),

        // Function keys
        "f1" => Some(Key::F1),
        "f2" => Some(Key::F2),
        "f3" => Some(Key::F3),
        "f4" => Some(Key::F4),
        "f5" => Some(Key::F5),
        "f6" => Some(Key::F6),
        "f7" => Some(Key::F7),
        "f8" => Some(Key::F8),
        "f9" => Some(Key::F9),
        "f10" => Some(Key::F10),
        "f11" => Some(Key::F11),
        "f12" => Some(Key::F12),

        // Special keys (with aliases)
        "escape" | "esc" => Some(Key::Escape),
        "tab" => Some(Key::Tab),
        "space" => Some(Key::Space),
        "return" | "enter" => Some(Key::Return),
        "delete" | "backspace" => Some(Key::Delete),

        // Navigation keys (with aliases)
        "left" | "leftarrow" => Some(Key::Left),
        "right" | "rightarrow" => Some(Key::Right),
        "up" | "uparrow" => Some(Key::Up),
        "down" | "downarrow" => Some(Key::Down),
        "home" => Some(Key::Home),
        "end" => Some(Key::End),
        "pageup" => Some(Key::PageUp),
        "pagedown" => Some(Key::PageDown),

        // Punctuation and symbols (accept both normal and shifted versions)
        "-" | "_" => Some(Key::Minus),
        "=" | "+" => Some(Key::Equal),
        "[" | "{" => Some(Key::LeftBracket),
        "]" | "}" => Some(Key::RightBracket),
        "\\" | "|" => Some(Key::Backslash),
        ";" | ":" => Some(Key::Semicolon),
        "'" | "\"" => Some(Key::Quote),
        "`" | "~" => Some(Key::Grave),
        "," | "<" => Some(Key::Comma),
        "." | ">" => Some(Key::Period),
        "/" | "?" => Some(Key::Slash),

        _ => None,
    }
}

/// Convert a `Key` to its display name.
///
/// Returns a human-readable name suitable for display in the UI.
///
/// # Examples
///
/// ```ignore
/// use cat_shield::input::keycodes::{Key, key_to_name};
///
/// assert_eq!(key_to_name(Key::A), "A");
/// assert_eq!(key_to_name(Key::Escape), "Escape");
/// assert_eq!(key_to_name(Key::F1), "F1");
/// ```
#[must_use]
pub const fn key_to_name(key: Key) -> &'static str {
    match key {
        // Letters
        Key::A => "A",
        Key::B => "B",
        Key::C => "C",
        Key::D => "D",
        Key::E => "E",
        Key::F => "F",
        Key::G => "G",
        Key::H => "H",
        Key::I => "I",
        Key::J => "J",
        Key::K => "K",
        Key::L => "L",
        Key::M => "M",
        Key::N => "N",
        Key::O => "O",
        Key::P => "P",
        Key::Q => "Q",
        Key::R => "R",
        Key::S => "S",
        Key::T => "T",
        Key::U => "U",
        Key::V => "V",
        Key::W => "W",
        Key::X => "X",
        Key::Y => "Y",
        Key::Z => "Z",

        // Numbers
        Key::Num0 => "0",
        Key::Num1 => "1",
        Key::Num2 => "2",
        Key::Num3 => "3",
        Key::Num4 => "4",
        Key::Num5 => "5",
        Key::Num6 => "6",
        Key::Num7 => "7",
        Key::Num8 => "8",
        Key::Num9 => "9",

        // Function keys
        Key::F1 => "F1",
        Key::F2 => "F2",
        Key::F3 => "F3",
        Key::F4 => "F4",
        Key::F5 => "F5",
        Key::F6 => "F6",
        Key::F7 => "F7",
        Key::F8 => "F8",
        Key::F9 => "F9",
        Key::F10 => "F10",
        Key::F11 => "F11",
        Key::F12 => "F12",

        // Special keys
        Key::Escape => "Escape",
        Key::Tab => "Tab",
        Key::Space => "Space",
        Key::Return => "Return",
        Key::Delete => "Delete",

        // Navigation keys
        Key::Left => "Left",
        Key::Right => "Right",
        Key::Up => "Up",
        Key::Down => "Down",
        Key::Home => "Home",
        Key::End => "End",
        Key::PageUp => "PageUp",
        Key::PageDown => "PageDown",

        // Punctuation and symbols
        Key::Minus => "-",
        Key::Equal => "=",
        Key::LeftBracket => "[",
        Key::RightBracket => "]",
        Key::Backslash => "\\",
        Key::Semicolon => ";",
        Key::Quote => "'",
        Key::Grave => "`",
        Key::Comma => ",",
        Key::Period => ".",
        Key::Slash => "/",
    }
}

// Re-export platform-specific functions
#[cfg(target_os = "macos")]
pub use macos::{key_to_keycode, keycode_to_key};

#[cfg(target_os = "windows")]
pub use windows::{key_to_keycode, keycode_to_key};

#[cfg(target_os = "linux")]
pub use linux::{key_to_keycode, keycode_to_key};

// =============================================================================
// Legacy compatibility functions
// =============================================================================
// These functions maintain backward compatibility with the old keycodes.rs API.

/// Convert a key name to its platform-specific keycode (legacy API).
///
/// This function is deprecated in favor of using `key_from_name` + `key_to_keycode`.
/// It is maintained for backward compatibility.
#[cfg(target_os = "macos")]
#[must_use]
pub fn keycode_from_name(name: &str) -> Option<i64> {
    key_from_name(name).and_then(|key| key_to_keycode(key).map(i64::from))
}

/// Convert a platform-specific keycode to its display name (legacy API).
///
/// This function is deprecated in favor of using `keycode_to_key` + `key_to_name`.
/// It is maintained for backward compatibility.
#[cfg(target_os = "macos")]
#[must_use]
pub fn keycode_to_name(keycode: i64) -> Option<&'static str> {
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let keycode_u32 = keycode as u32;
    keycode_to_key(keycode_u32).map(key_to_name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_key_from_name_letters() {
        assert_eq!(key_from_name("a"), Some(Key::A));
        assert_eq!(key_from_name("A"), Some(Key::A));
        assert_eq!(key_from_name("z"), Some(Key::Z));
        assert_eq!(key_from_name("Z"), Some(Key::Z));
    }

    #[test]
    fn test_key_from_name_numbers() {
        assert_eq!(key_from_name("0"), Some(Key::Num0));
        assert_eq!(key_from_name("9"), Some(Key::Num9));
        assert_eq!(key_from_name(")"), Some(Key::Num0)); // Shifted alias
        assert_eq!(key_from_name("!"), Some(Key::Num1)); // Shifted alias
    }

    #[test]
    fn test_key_from_name_function_keys() {
        assert_eq!(key_from_name("f1"), Some(Key::F1));
        assert_eq!(key_from_name("F12"), Some(Key::F12));
    }

    #[test]
    fn test_key_from_name_special_keys() {
        assert_eq!(key_from_name("escape"), Some(Key::Escape));
        assert_eq!(key_from_name("Escape"), Some(Key::Escape));
        assert_eq!(key_from_name("esc"), Some(Key::Escape)); // Alias
        assert_eq!(key_from_name("return"), Some(Key::Return));
        assert_eq!(key_from_name("enter"), Some(Key::Return)); // Alias
        assert_eq!(key_from_name("space"), Some(Key::Space));
        assert_eq!(key_from_name("tab"), Some(Key::Tab));
        assert_eq!(key_from_name("delete"), Some(Key::Delete));
        assert_eq!(key_from_name("backspace"), Some(Key::Delete)); // Alias
    }

    #[test]
    fn test_key_from_name_navigation() {
        assert_eq!(key_from_name("left"), Some(Key::Left));
        assert_eq!(key_from_name("leftarrow"), Some(Key::Left)); // Alias
        assert_eq!(key_from_name("right"), Some(Key::Right));
        assert_eq!(key_from_name("up"), Some(Key::Up));
        assert_eq!(key_from_name("down"), Some(Key::Down));
        assert_eq!(key_from_name("home"), Some(Key::Home));
        assert_eq!(key_from_name("end"), Some(Key::End));
        assert_eq!(key_from_name("pageup"), Some(Key::PageUp));
        assert_eq!(key_from_name("pagedown"), Some(Key::PageDown));
    }

    #[test]
    fn test_key_from_name_punctuation() {
        assert_eq!(key_from_name("-"), Some(Key::Minus));
        assert_eq!(key_from_name("_"), Some(Key::Minus)); // Shifted
        assert_eq!(key_from_name("="), Some(Key::Equal));
        assert_eq!(key_from_name("+"), Some(Key::Equal)); // Shifted
        assert_eq!(key_from_name("["), Some(Key::LeftBracket));
        assert_eq!(key_from_name("]"), Some(Key::RightBracket));
        assert_eq!(key_from_name("\\"), Some(Key::Backslash));
        assert_eq!(key_from_name(";"), Some(Key::Semicolon));
        assert_eq!(key_from_name("'"), Some(Key::Quote));
        assert_eq!(key_from_name("`"), Some(Key::Grave));
        assert_eq!(key_from_name(","), Some(Key::Comma));
        assert_eq!(key_from_name("."), Some(Key::Period));
        assert_eq!(key_from_name("/"), Some(Key::Slash));
    }

    #[test]
    fn test_key_from_name_unknown() {
        assert_eq!(key_from_name("unknown"), None);
        assert_eq!(key_from_name(""), None);
        assert_eq!(key_from_name("f13"), None);
    }

    #[test]
    fn test_key_to_name() {
        assert_eq!(key_to_name(Key::A), "A");
        assert_eq!(key_to_name(Key::Z), "Z");
        assert_eq!(key_to_name(Key::Num0), "0");
        assert_eq!(key_to_name(Key::F1), "F1");
        assert_eq!(key_to_name(Key::Escape), "Escape");
        assert_eq!(key_to_name(Key::Space), "Space");
        assert_eq!(key_to_name(Key::Left), "Left");
    }

    #[test]
    fn test_key_all() {
        let all_keys = Key::all();
        assert!(!all_keys.is_empty());
        // Check a few known keys are in the list
        assert!(all_keys.contains(&Key::A));
        assert!(all_keys.contains(&Key::Escape));
        assert!(all_keys.contains(&Key::F1));
    }

    #[test]
    fn test_key_is_letter() {
        assert!(Key::A.is_letter());
        assert!(Key::Z.is_letter());
        assert!(!Key::Num0.is_letter());
        assert!(!Key::Escape.is_letter());
    }

    #[test]
    fn test_key_is_number() {
        assert!(Key::Num0.is_number());
        assert!(Key::Num9.is_number());
        assert!(!Key::A.is_number());
        assert!(!Key::F1.is_number());
    }

    #[test]
    fn test_key_is_function_key() {
        assert!(Key::F1.is_function_key());
        assert!(Key::F12.is_function_key());
        assert!(!Key::Escape.is_function_key());
        assert!(!Key::A.is_function_key());
    }

    #[test]
    fn test_key_is_navigation() {
        assert!(Key::Left.is_navigation());
        assert!(Key::Right.is_navigation());
        assert!(Key::Home.is_navigation());
        assert!(Key::PageDown.is_navigation());
        assert!(!Key::A.is_navigation());
        assert!(!Key::Escape.is_navigation());
    }

    #[test]
    fn test_key_roundtrip() {
        // Test that all keys can be converted to name and back
        for &key in Key::all() {
            let name = key_to_name(key);
            let parsed = key_from_name(name);
            assert_eq!(
                parsed,
                Some(key),
                "Failed roundtrip for key {:?} with name '{}'",
                key,
                name
            );
        }
    }

    // Legacy compatibility tests (macOS only)
    #[cfg(target_os = "macos")]
    mod legacy_tests {
        use super::*;

        #[test]
        fn test_keycode_from_name_letters() {
            assert_eq!(keycode_from_name("a"), Some(0));
            assert_eq!(keycode_from_name("u"), Some(32));
            assert_eq!(keycode_from_name("q"), Some(12));
            assert_eq!(keycode_from_name("U"), Some(32)); // Case insensitive
        }

        #[test]
        fn test_keycode_from_name_special() {
            assert_eq!(keycode_from_name("escape"), Some(53));
            assert_eq!(keycode_from_name("Escape"), Some(53));
            assert_eq!(keycode_from_name("esc"), Some(53));
            assert_eq!(keycode_from_name("return"), Some(36));
            assert_eq!(keycode_from_name("enter"), Some(36));
            assert_eq!(keycode_from_name("space"), Some(49));
            assert_eq!(keycode_from_name("tab"), Some(48));
        }

        #[test]
        fn test_keycode_from_name_function_keys() {
            assert_eq!(keycode_from_name("f1"), Some(122));
            assert_eq!(keycode_from_name("F12"), Some(111));
        }

        #[test]
        fn test_keycode_from_name_unknown() {
            assert_eq!(keycode_from_name("unknown"), None);
            assert_eq!(keycode_from_name(""), None);
        }

        #[test]
        fn test_keycode_to_name() {
            assert_eq!(keycode_to_name(0), Some("A"));
            assert_eq!(keycode_to_name(53), Some("Escape"));
            assert_eq!(keycode_to_name(122), Some("F1"));
            assert_eq!(keycode_to_name(999), None);
        }
    }
}
