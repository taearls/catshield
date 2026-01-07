//! macOS virtual key code mappings
//!
//! See: https://developer.apple.com/documentation/coregraphics/cgkeycode

/// Convert a key name to its macOS virtual keycode
pub fn keycode_from_name(name: &str) -> Option<i64> {
    match name.to_lowercase().as_str() {
        // Letters
        "a" => Some(0),
        "s" => Some(1),
        "d" => Some(2),
        "f" => Some(3),
        "h" => Some(4),
        "g" => Some(5),
        "z" => Some(6),
        "x" => Some(7),
        "c" => Some(8),
        "v" => Some(9),
        "b" => Some(11),
        "q" => Some(12),
        "w" => Some(13),
        "e" => Some(14),
        "r" => Some(15),
        "y" => Some(16),
        "t" => Some(17),
        "1" | "!" => Some(18),
        "2" | "@" => Some(19),
        "3" | "#" => Some(20),
        "4" | "$" => Some(21),
        "6" | "^" => Some(22),
        "5" | "%" => Some(23),
        "=" | "+" => Some(24),
        "9" | "(" => Some(25),
        "7" | "&" => Some(26),
        "-" | "_" => Some(27),
        "8" | "*" => Some(28),
        "0" | ")" => Some(29),
        "]" | "}" => Some(30),
        "o" => Some(31),
        "u" => Some(32),
        "[" | "{" => Some(33),
        "i" => Some(34),
        "p" => Some(35),
        "l" => Some(37),
        "j" => Some(38),
        "'" | "\"" => Some(39),
        "k" => Some(40),
        ";" | ":" => Some(41),
        "\\" | "|" => Some(42),
        "," | "<" => Some(43),
        "/" | "?" => Some(44),
        "n" => Some(45),
        "m" => Some(46),
        "." | ">" => Some(47),
        "`" | "~" => Some(50),
        // Special keys
        "return" | "enter" => Some(36),
        "tab" => Some(48),
        "space" => Some(49),
        "delete" | "backspace" => Some(51),
        "escape" | "esc" => Some(53),
        "f1" => Some(122),
        "f2" => Some(120),
        "f3" => Some(99),
        "f4" => Some(118),
        "f5" => Some(96),
        "f6" => Some(97),
        "f7" => Some(98),
        "f8" => Some(100),
        "f9" => Some(101),
        "f10" => Some(109),
        "f11" => Some(103),
        "f12" => Some(111),
        "home" => Some(115),
        "end" => Some(119),
        "pageup" => Some(116),
        "pagedown" => Some(121),
        "left" | "leftarrow" => Some(123),
        "right" | "rightarrow" => Some(124),
        "down" | "downarrow" => Some(125),
        "up" | "uparrow" => Some(126),
        _ => None,
    }
}

/// Convert keycode back to key name for display
pub fn keycode_to_name(keycode: i64) -> Option<&'static str> {
    match keycode {
        // Letters
        0 => Some("A"),
        11 => Some("B"),
        8 => Some("C"),
        2 => Some("D"),
        14 => Some("E"),
        3 => Some("F"),
        5 => Some("G"),
        4 => Some("H"),
        34 => Some("I"),
        38 => Some("J"),
        40 => Some("K"),
        37 => Some("L"),
        46 => Some("M"),
        45 => Some("N"),
        31 => Some("O"),
        35 => Some("P"),
        12 => Some("Q"),
        15 => Some("R"),
        1 => Some("S"),
        17 => Some("T"),
        32 => Some("U"),
        9 => Some("V"),
        13 => Some("W"),
        7 => Some("X"),
        16 => Some("Y"),
        6 => Some("Z"),
        // Numbers
        18 => Some("1"),
        19 => Some("2"),
        20 => Some("3"),
        21 => Some("4"),
        23 => Some("5"),
        22 => Some("6"),
        26 => Some("7"),
        28 => Some("8"),
        25 => Some("9"),
        29 => Some("0"),
        // Punctuation and symbols
        24 => Some("="),
        27 => Some("-"),
        30 => Some("]"),
        33 => Some("["),
        39 => Some("'"),
        41 => Some(";"),
        42 => Some("\\"),
        43 => Some(","),
        44 => Some("/"),
        47 => Some("."),
        50 => Some("`"),
        // Special keys
        53 => Some("Escape"),
        36 => Some("Return"),
        48 => Some("Tab"),
        49 => Some("Space"),
        51 => Some("Delete"),
        // Function keys
        122 => Some("F1"),
        120 => Some("F2"),
        99 => Some("F3"),
        118 => Some("F4"),
        96 => Some("F5"),
        97 => Some("F6"),
        98 => Some("F7"),
        100 => Some("F8"),
        101 => Some("F9"),
        109 => Some("F10"),
        103 => Some("F11"),
        111 => Some("F12"),
        // Navigation keys
        115 => Some("Home"),
        119 => Some("End"),
        116 => Some("PageUp"),
        121 => Some("PageDown"),
        123 => Some("Left"),
        124 => Some("Right"),
        125 => Some("Down"),
        126 => Some("Up"),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
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
}
