//! Allowed keys management for Cat Shield
//!
//! Provides parsing, validation, and checking of keyboard shortcuts
//! that are allowed to pass through the shield overlay.

use super::exit_key::{ExitKey, ERR_NO_MODIFIER};
use objc2_core_graphics::CGEventFlags;
use std::sync::RwLock;

/// Represents a single allowed key combination
#[derive(Debug, Clone, PartialEq)]
pub struct AllowedKey {
    /// Virtual keycode for the key
    pub keycode: i64,
    /// Whether Command/Cmd modifier is required
    pub requires_cmd: bool,
    /// Whether Option/Alt modifier is required
    pub requires_option: bool,
    /// Whether Shift modifier is required
    pub requires_shift: bool,
    /// Whether Control/Ctrl modifier is required
    pub requires_ctrl: bool,
    /// Display name for the key combination
    pub display_name: String,
}

impl AllowedKey {
    /// Parse a key combination string using the same format as ExitKey
    pub fn parse(input: &str) -> Result<Self, String> {
        // Reuse the ExitKey parser but relax the modifier requirement
        let exit_key = match ExitKey::parse(input) {
            Ok(key) => key,
            Err(e) => {
                // If parsing failed due to missing modifier, try parsing as a simple key
                // Uses shared constant to avoid fragile string matching
                if e.contains(ERR_NO_MODIFIER) {
                    return Self::parse_simple_key(input);
                }
                return Err(e);
            }
        };

        Ok(AllowedKey {
            keycode: exit_key.keycode,
            requires_cmd: exit_key.requires_cmd,
            requires_option: exit_key.requires_option,
            requires_shift: exit_key.requires_shift,
            requires_ctrl: exit_key.requires_ctrl,
            display_name: input.trim().to_string(),
        })
    }

    /// Parse a simple key without modifiers (e.g., "F11", "Space")
    fn parse_simple_key(input: &str) -> Result<Self, String> {
        use super::keycodes::keycode_from_name;

        let input = input.trim();
        if input.is_empty() {
            return Err("Key cannot be empty".to_string());
        }

        let keycode = keycode_from_name(input)
            .ok_or_else(|| format!("Unknown key: '{}'. Valid keys include: A-Z, 0-9, F1-F12, Escape, Return, Tab, Space, Delete, Arrow keys", input))?;

        Ok(AllowedKey {
            keycode,
            requires_cmd: false,
            requires_option: false,
            requires_shift: false,
            requires_ctrl: false,
            display_name: input.to_string(),
        })
    }

    /// Check if this allowed key matches the given key event
    pub fn matches(&self, keycode: i64, flags: CGEventFlags) -> bool {
        if keycode != self.keycode {
            return false;
        }

        let has_cmd = flags.contains(CGEventFlags::MaskCommand);
        let has_option = flags.contains(CGEventFlags::MaskAlternate);
        let has_shift = flags.contains(CGEventFlags::MaskShift);
        let has_ctrl = flags.contains(CGEventFlags::MaskControl);

        self.requires_cmd == has_cmd
            && self.requires_option == has_option
            && self.requires_shift == has_shift
            && self.requires_ctrl == has_ctrl
    }
}

/// Global storage for allowed keys (protected by RwLock for thread safety)
static ALLOWED_KEYS: RwLock<Vec<AllowedKey>> = RwLock::new(Vec::new());

/// Set the global allowed keys configuration
pub fn set_allowed_keys(keys: Vec<AllowedKey>) {
    match ALLOWED_KEYS.write() {
        Ok(mut guard) => {
            *guard = keys;
        }
        Err(poisoned) => {
            // Recover from poisoned mutex
            eprintln!("  ⚠️  Warning: ALLOWED_KEYS mutex was poisoned, recovering...");
            let mut guard = poisoned.into_inner();
            *guard = keys;
        }
    }
}

/// Parse and set allowed keys from string list
pub fn parse_and_set_allowed_keys(key_strings: &[String]) -> Result<(), Vec<String>> {
    let mut parsed_keys = Vec::new();
    let mut errors = Vec::new();

    for key_str in key_strings {
        match AllowedKey::parse(key_str) {
            Ok(key) => parsed_keys.push(key),
            Err(e) => errors.push(format!("'{}': {}", key_str, e)),
        }
    }

    if !errors.is_empty() {
        return Err(errors);
    }

    set_allowed_keys(parsed_keys);
    Ok(())
}

/// Get a copy of the current allowed keys configuration
pub fn get_allowed_keys() -> Vec<AllowedKey> {
    match ALLOWED_KEYS.read() {
        Ok(guard) => guard.clone(),
        Err(poisoned) => {
            eprintln!("  ⚠️  Warning: ALLOWED_KEYS mutex was poisoned during read");
            poisoned.into_inner().clone()
        }
    }
}

/// Check if a key event matches any of the allowed keys
pub fn is_key_allowed(keycode: i64, flags: CGEventFlags) -> bool {
    match ALLOWED_KEYS.read() {
        Ok(guard) => guard.iter().any(|key| key.matches(keycode, flags)),
        Err(poisoned) => {
            eprintln!("  ⚠️  Warning: ALLOWED_KEYS mutex was poisoned during check");
            poisoned
                .into_inner()
                .iter()
                .any(|key| key.matches(keycode, flags))
        }
    }
}

/// Clear all allowed keys
pub fn clear_allowed_keys() {
    match ALLOWED_KEYS.write() {
        Ok(mut guard) => {
            guard.clear();
        }
        Err(poisoned) => {
            eprintln!("  ⚠️  Warning: ALLOWED_KEYS mutex was poisoned during clear");
            poisoned.into_inner().clear();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_allowed_key_parse_with_modifiers() {
        let key = AllowedKey::parse("Cmd+Space").unwrap();
        assert_eq!(key.keycode, 49); // Space
        assert!(key.requires_cmd);
        assert!(!key.requires_option);
        assert!(!key.requires_shift);
        assert!(!key.requires_ctrl);
    }

    #[test]
    fn test_allowed_key_parse_simple_key() {
        let key = AllowedKey::parse("F11").unwrap();
        assert_eq!(key.keycode, 103); // F11
        assert!(!key.requires_cmd);
        assert!(!key.requires_option);
        assert!(!key.requires_shift);
        assert!(!key.requires_ctrl);
    }

    #[test]
    fn test_allowed_key_parse_multiple_modifiers() {
        let key = AllowedKey::parse("Ctrl+Option+A").unwrap();
        assert_eq!(key.keycode, 0); // A
        assert!(!key.requires_cmd);
        assert!(key.requires_option);
        assert!(!key.requires_shift);
        assert!(key.requires_ctrl);
    }

    #[test]
    fn test_allowed_key_parse_invalid_key() {
        assert!(AllowedKey::parse("Invalid").is_err());
        assert!(AllowedKey::parse("").is_err());
    }

    #[test]
    fn test_parse_and_set_allowed_keys_success() {
        let keys = vec![
            "Cmd+Space".to_string(),
            "F11".to_string(),
            "F12".to_string(),
        ];

        let result = parse_and_set_allowed_keys(&keys);
        assert!(result.is_ok());

        let allowed = get_allowed_keys();
        assert_eq!(allowed.len(), 3);
        assert_eq!(allowed[0].display_name, "Cmd+Space");
        assert_eq!(allowed[1].display_name, "F11");
        assert_eq!(allowed[2].display_name, "F12");
    }

    #[test]
    fn test_parse_and_set_allowed_keys_with_errors() {
        let keys = vec![
            "Cmd+Space".to_string(),
            "InvalidKey".to_string(),
            "F11".to_string(),
        ];

        let result = parse_and_set_allowed_keys(&keys);
        assert!(result.is_err());

        let errors = result.unwrap_err();
        assert_eq!(errors.len(), 1);
        assert!(errors[0].contains("InvalidKey"));
    }

    #[test]
    fn test_clear_allowed_keys() {
        let keys = vec!["F11".to_string(), "F12".to_string()];
        parse_and_set_allowed_keys(&keys).unwrap();

        assert_eq!(get_allowed_keys().len(), 2);

        clear_allowed_keys();
        assert_eq!(get_allowed_keys().len(), 0);
    }

    #[test]
    fn test_is_key_allowed_matches() {
        let keys = vec!["Cmd+Space".to_string(), "F11".to_string()];
        parse_and_set_allowed_keys(&keys).unwrap();

        // Test Cmd+Space match
        let flags = CGEventFlags::MaskCommand;
        assert!(is_key_allowed(49, flags)); // Space keycode

        // Test F11 match (no modifiers)
        let flags = CGEventFlags::empty();
        assert!(is_key_allowed(103, flags)); // F11 keycode
    }

    #[test]
    fn test_is_key_allowed_no_match() {
        let keys = vec!["Cmd+Space".to_string()];
        parse_and_set_allowed_keys(&keys).unwrap();

        // Test wrong keycode
        let flags = CGEventFlags::MaskCommand;
        assert!(!is_key_allowed(0, flags)); // A keycode, not Space

        // Test wrong modifiers
        let flags = CGEventFlags::MaskAlternate; // Option instead of Cmd
        assert!(!is_key_allowed(49, flags)); // Space keycode
    }

    #[test]
    fn test_allowed_key_matches_exact_modifiers() {
        let key = AllowedKey::parse("Cmd+Shift+A").unwrap();

        // Exact match
        let flags = CGEventFlags::MaskCommand | CGEventFlags::MaskShift;
        assert!(key.matches(0, flags)); // A keycode

        // Missing a modifier
        let flags = CGEventFlags::MaskCommand;
        assert!(!key.matches(0, flags));

        // Extra modifier
        let flags =
            CGEventFlags::MaskCommand | CGEventFlags::MaskShift | CGEventFlags::MaskAlternate;
        assert!(!key.matches(0, flags));
    }
}
