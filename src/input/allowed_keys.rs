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
    #[must_use = "parsing may fail, check the Result"]
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
#[must_use = "parsing may fail, check the Result for errors"]
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

// ============================================================================
// KEY PRESETS
// ============================================================================

/// Preset key combinations for common use cases
pub mod presets {
    /// Media Keys preset: F10 (Mute), F11 (Volume Down), F12 (Volume Up)
    pub const MEDIA_KEYS: &[&str] = &["F10", "F11", "F12"];
    /// Media Keys preset display name
    pub const MEDIA_KEYS_NAME: &str = "Media Keys";
    /// Media Keys preset tooltip
    pub const MEDIA_KEYS_TOOLTIP: &str = "Adds F10 (Mute), F11 (Volume Down), F12 (Volume Up)";

    /// Spotlight preset: Cmd+Space
    pub const SPOTLIGHT: &[&str] = &["Cmd+Space"];
    /// Spotlight preset display name
    pub const SPOTLIGHT_NAME: &str = "Spotlight";
    /// Spotlight preset tooltip
    pub const SPOTLIGHT_TOOLTIP: &str = "Adds Cmd+Space (Spotlight search)";

    /// Mission Control preset: Ctrl+Up/Down/Left/Right
    pub const MISSION_CONTROL: &[&str] = &["Ctrl+Up", "Ctrl+Down", "Ctrl+Left", "Ctrl+Right"];
    /// Mission Control preset display name
    pub const MISSION_CONTROL_NAME: &str = "Mission Control";
    /// Mission Control preset tooltip
    pub const MISSION_CONTROL_TOOLTIP: &str =
        "Adds Ctrl+Up/Down/Left/Right (Mission Control & Spaces)";

    /// Screenshots preset: Cmd+Shift+3/4/5
    pub const SCREENSHOTS: &[&str] = &["Cmd+Shift+3", "Cmd+Shift+4", "Cmd+Shift+5"];
    /// Screenshots preset display name
    pub const SCREENSHOTS_NAME: &str = "Screenshots";
    /// Screenshots preset tooltip
    pub const SCREENSHOTS_TOOLTIP: &str =
        "Adds Cmd+Shift+3 (full), Cmd+Shift+4 (selection), Cmd+Shift+5 (options)";
}

/// Add keys from a preset to the current allowed keys list.
/// Returns the number of keys actually added (excluding duplicates).
pub fn add_preset_keys(preset_keys: &[&str], current_keys: &mut Vec<String>) -> usize {
    let mut added_count = 0;

    for &key in preset_keys {
        // Check for duplicates (case-insensitive)
        let key_lower = key.to_lowercase();
        let already_exists = current_keys.iter().any(|k| k.to_lowercase() == key_lower);

        if !already_exists {
            // Validate the key before adding
            if AllowedKey::parse(key).is_ok() {
                current_keys.push(key.to_string());
                added_count += 1;
            }
        }
    }

    added_count
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
        // Clear any state from other tests for isolation
        clear_allowed_keys();

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
        // Clear any state from other tests for isolation
        clear_allowed_keys();

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
        // Clear any state from other tests for isolation
        clear_allowed_keys();

        let keys = vec!["F11".to_string(), "F12".to_string()];
        parse_and_set_allowed_keys(&keys).unwrap();

        assert_eq!(get_allowed_keys().len(), 2);

        clear_allowed_keys();
        assert_eq!(get_allowed_keys().len(), 0);
    }

    #[test]
    fn test_is_key_allowed_matches() {
        // Clear any state from other tests for isolation
        clear_allowed_keys();

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
        // Clear any state from other tests for isolation
        clear_allowed_keys();

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

    // ========================================================================
    // Preset Tests
    // ========================================================================

    #[test]
    fn test_add_preset_keys_media_keys() {
        let mut keys = Vec::new();
        let added = add_preset_keys(presets::MEDIA_KEYS, &mut keys);
        assert_eq!(added, 3);
        assert_eq!(keys.len(), 3);
        assert!(keys.contains(&"F10".to_string()));
        assert!(keys.contains(&"F11".to_string()));
        assert!(keys.contains(&"F12".to_string()));
    }

    #[test]
    fn test_add_preset_keys_spotlight() {
        let mut keys = Vec::new();
        let added = add_preset_keys(presets::SPOTLIGHT, &mut keys);
        assert_eq!(added, 1);
        assert_eq!(keys.len(), 1);
        assert!(keys.contains(&"Cmd+Space".to_string()));
    }

    #[test]
    fn test_add_preset_keys_mission_control() {
        let mut keys = Vec::new();
        let added = add_preset_keys(presets::MISSION_CONTROL, &mut keys);
        assert_eq!(added, 4);
        assert_eq!(keys.len(), 4);
        assert!(keys.contains(&"Ctrl+Up".to_string()));
        assert!(keys.contains(&"Ctrl+Down".to_string()));
        assert!(keys.contains(&"Ctrl+Left".to_string()));
        assert!(keys.contains(&"Ctrl+Right".to_string()));
    }

    #[test]
    fn test_add_preset_keys_screenshots() {
        let mut keys = Vec::new();
        let added = add_preset_keys(presets::SCREENSHOTS, &mut keys);
        assert_eq!(added, 3);
        assert_eq!(keys.len(), 3);
        assert!(keys.contains(&"Cmd+Shift+3".to_string()));
        assert!(keys.contains(&"Cmd+Shift+4".to_string()));
        assert!(keys.contains(&"Cmd+Shift+5".to_string()));
    }

    #[test]
    fn test_add_preset_keys_no_duplicates() {
        let mut keys = vec!["F11".to_string()]; // Already has F11
        let added = add_preset_keys(presets::MEDIA_KEYS, &mut keys);
        // Should only add F10 and F12, not F11 (already exists)
        assert_eq!(added, 2);
        assert_eq!(keys.len(), 3);
    }

    #[test]
    fn test_add_preset_keys_case_insensitive_duplicates() {
        let mut keys = vec!["cmd+space".to_string()]; // Lowercase version
        let added = add_preset_keys(presets::SPOTLIGHT, &mut keys);
        // Should not add Cmd+Space since cmd+space already exists (case-insensitive)
        assert_eq!(added, 0);
        assert_eq!(keys.len(), 1);
    }

    #[test]
    fn test_add_preset_keys_all_existing() {
        let mut keys = vec!["F10".to_string(), "F11".to_string(), "F12".to_string()];
        let added = add_preset_keys(presets::MEDIA_KEYS, &mut keys);
        // All keys already exist
        assert_eq!(added, 0);
        assert_eq!(keys.len(), 3);
    }

    #[test]
    fn test_add_preset_keys_multiple_presets() {
        let mut keys = Vec::new();

        // Add Media Keys
        let added1 = add_preset_keys(presets::MEDIA_KEYS, &mut keys);
        assert_eq!(added1, 3);

        // Add Spotlight
        let added2 = add_preset_keys(presets::SPOTLIGHT, &mut keys);
        assert_eq!(added2, 1);

        // Add Mission Control
        let added3 = add_preset_keys(presets::MISSION_CONTROL, &mut keys);
        assert_eq!(added3, 4);

        // Add Screenshots
        let added4 = add_preset_keys(presets::SCREENSHOTS, &mut keys);
        assert_eq!(added4, 3);

        // Total should be 11 keys
        assert_eq!(keys.len(), 11);
    }

    #[test]
    fn test_preset_constants_not_empty() {
        assert!(!presets::MEDIA_KEYS.is_empty());
        assert!(!presets::SPOTLIGHT.is_empty());
        assert!(!presets::MISSION_CONTROL.is_empty());
        assert!(!presets::SCREENSHOTS.is_empty());
    }

    #[test]
    fn test_preset_names_not_empty() {
        assert!(!presets::MEDIA_KEYS_NAME.is_empty());
        assert!(!presets::SPOTLIGHT_NAME.is_empty());
        assert!(!presets::MISSION_CONTROL_NAME.is_empty());
        assert!(!presets::SCREENSHOTS_NAME.is_empty());
    }

    #[test]
    fn test_preset_tooltips_not_empty() {
        assert!(!presets::MEDIA_KEYS_TOOLTIP.is_empty());
        assert!(!presets::SPOTLIGHT_TOOLTIP.is_empty());
        assert!(!presets::MISSION_CONTROL_TOOLTIP.is_empty());
        assert!(!presets::SCREENSHOTS_TOOLTIP.is_empty());
    }

    #[test]
    fn test_all_preset_keys_are_valid() {
        // Verify all preset keys can be parsed
        for key in presets::MEDIA_KEYS {
            assert!(AllowedKey::parse(key).is_ok(), "Failed to parse: {}", key);
        }
        for key in presets::SPOTLIGHT {
            assert!(AllowedKey::parse(key).is_ok(), "Failed to parse: {}", key);
        }
        for key in presets::MISSION_CONTROL {
            assert!(AllowedKey::parse(key).is_ok(), "Failed to parse: {}", key);
        }
        for key in presets::SCREENSHOTS {
            assert!(AllowedKey::parse(key).is_ok(), "Failed to parse: {}", key);
        }
    }
}
