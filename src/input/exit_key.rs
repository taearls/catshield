//! Exit key handling for Cat Shield
//!
//! Provides parsing and validation of keyboard shortcut combinations
//! used to exit the shield overlay.

use super::keycodes::{keycode_from_name, keycode_to_name};
use objc2_core_graphics::CGEventFlags;
use std::sync::atomic::{AtomicBool, AtomicI64, Ordering};

/// Default exit key configuration
pub const DEFAULT_EXIT_KEY: &str = "Cmd+Option+U";

/// Represents a parsed exit key combination
#[derive(Debug, Clone)]
pub struct ExitKey {
    pub keycode: i64,
    pub requires_cmd: bool,
    pub requires_option: bool,
    pub requires_shift: bool,
    pub requires_ctrl: bool,
    pub display_name: String,
}

impl Default for ExitKey {
    fn default() -> Self {
        // Default: Cmd+Option+U
        ExitKey {
            keycode: 32, // U
            requires_cmd: true,
            requires_option: true,
            requires_shift: false,
            requires_ctrl: false,
            display_name: DEFAULT_EXIT_KEY.to_string(),
        }
    }
}

impl ExitKey {
    /// Parse a key combination string like "Cmd+Option+U" or "Ctrl+Shift+Escape"
    pub fn parse(input: &str) -> Result<Self, String> {
        let input = input.trim();
        if input.is_empty() {
            return Err("Exit key cannot be empty".to_string());
        }

        let parts: Vec<&str> = input.split('+').map(|s| s.trim()).collect();
        if parts.is_empty() {
            return Err("Invalid key combination format".to_string());
        }

        let mut requires_cmd = false;
        let mut requires_option = false;
        let mut requires_shift = false;
        let mut requires_ctrl = false;
        let mut key_name: Option<&str> = None;

        for part in &parts {
            let lower = part.to_lowercase();
            match lower.as_str() {
                "cmd" | "command" | "⌘" => requires_cmd = true,
                "opt" | "option" | "alt" | "⌥" => requires_option = true,
                "shift" | "⇧" => requires_shift = true,
                "ctrl" | "control" | "⌃" => requires_ctrl = true,
                _ => {
                    if let Some(existing) = key_name {
                        return Err(format!(
                            "Multiple keys specified: '{}' and '{}'",
                            existing, part
                        ));
                    }
                    key_name = Some(part);
                }
            }
        }

        let key_name = key_name.ok_or("No key specified in combination")?;
        let keycode = keycode_from_name(key_name)
            .ok_or_else(|| format!("Unknown key: '{}'. Valid keys include: A-Z, 0-9, F1-F12, Escape, Return, Tab, Space, Delete, Arrow keys", key_name))?;

        // Require at least one modifier
        if !requires_cmd && !requires_option && !requires_shift && !requires_ctrl {
            return Err(
                "At least one modifier key required (Cmd, Option, Shift, or Ctrl)".to_string(),
            );
        }

        Ok(ExitKey {
            keycode,
            requires_cmd,
            requires_option,
            requires_shift,
            requires_ctrl,
            display_name: input.to_string(),
        })
    }
}

// Global storage for exit key configuration (atomic for thread safety)
pub static EXIT_KEY_KEYCODE: AtomicI64 = AtomicI64::new(32); // Default: U
pub static EXIT_KEY_REQUIRES_CMD: AtomicBool = AtomicBool::new(true);
pub static EXIT_KEY_REQUIRES_OPTION: AtomicBool = AtomicBool::new(true);
pub static EXIT_KEY_REQUIRES_SHIFT: AtomicBool = AtomicBool::new(false);
pub static EXIT_KEY_REQUIRES_CTRL: AtomicBool = AtomicBool::new(false);

/// Set the global exit key configuration
pub fn set_exit_key(key: &ExitKey) {
    EXIT_KEY_KEYCODE.store(key.keycode, Ordering::SeqCst);
    EXIT_KEY_REQUIRES_CMD.store(key.requires_cmd, Ordering::SeqCst);
    EXIT_KEY_REQUIRES_OPTION.store(key.requires_option, Ordering::SeqCst);
    EXIT_KEY_REQUIRES_SHIFT.store(key.requires_shift, Ordering::SeqCst);
    EXIT_KEY_REQUIRES_CTRL.store(key.requires_ctrl, Ordering::SeqCst);
}

/// Get the current exit key configuration from global storage
pub fn get_exit_key() -> ExitKey {
    ExitKey {
        keycode: EXIT_KEY_KEYCODE.load(Ordering::SeqCst),
        requires_cmd: EXIT_KEY_REQUIRES_CMD.load(Ordering::SeqCst),
        requires_option: EXIT_KEY_REQUIRES_OPTION.load(Ordering::SeqCst),
        requires_shift: EXIT_KEY_REQUIRES_SHIFT.load(Ordering::SeqCst),
        requires_ctrl: EXIT_KEY_REQUIRES_CTRL.load(Ordering::SeqCst),
        display_name: format_exit_key_display(),
    }
}

/// Format the exit key for display
pub fn format_exit_key_display() -> String {
    let mut parts = Vec::new();
    if EXIT_KEY_REQUIRES_CMD.load(Ordering::SeqCst) {
        parts.push("Cmd");
    }
    if EXIT_KEY_REQUIRES_OPTION.load(Ordering::SeqCst) {
        parts.push("Option");
    }
    if EXIT_KEY_REQUIRES_SHIFT.load(Ordering::SeqCst) {
        parts.push("Shift");
    }
    if EXIT_KEY_REQUIRES_CTRL.load(Ordering::SeqCst) {
        parts.push("Ctrl");
    }
    let keycode = EXIT_KEY_KEYCODE.load(Ordering::SeqCst);
    if let Some(key_name) = keycode_to_name(keycode) {
        parts.push(key_name);
    }
    parts.join("+")
}

/// Check if the given key event matches the configured exit key
pub fn check_exit_key(keycode: i64, flags: CGEventFlags) -> bool {
    let expected_keycode = EXIT_KEY_KEYCODE.load(Ordering::SeqCst);
    if keycode != expected_keycode {
        return false;
    }

    let has_cmd = flags.contains(CGEventFlags::MaskCommand);
    let has_option = flags.contains(CGEventFlags::MaskAlternate);
    let has_shift = flags.contains(CGEventFlags::MaskShift);
    let has_ctrl = flags.contains(CGEventFlags::MaskControl);

    let requires_cmd = EXIT_KEY_REQUIRES_CMD.load(Ordering::SeqCst);
    let requires_option = EXIT_KEY_REQUIRES_OPTION.load(Ordering::SeqCst);
    let requires_shift = EXIT_KEY_REQUIRES_SHIFT.load(Ordering::SeqCst);
    let requires_ctrl = EXIT_KEY_REQUIRES_CTRL.load(Ordering::SeqCst);

    requires_cmd == has_cmd
        && requires_option == has_option
        && requires_shift == has_shift
        && requires_ctrl == has_ctrl
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_exit_key_parse_default() {
        let key = ExitKey::parse("Cmd+Option+U").unwrap();
        assert_eq!(key.keycode, 32);
        assert!(key.requires_cmd);
        assert!(key.requires_option);
        assert!(!key.requires_shift);
        assert!(!key.requires_ctrl);
    }

    #[test]
    fn test_exit_key_parse_cmd_shift_q() {
        let key = ExitKey::parse("Cmd+Shift+Q").unwrap();
        assert_eq!(key.keycode, 12);
        assert!(key.requires_cmd);
        assert!(!key.requires_option);
        assert!(key.requires_shift);
        assert!(!key.requires_ctrl);
    }

    #[test]
    fn test_exit_key_parse_ctrl_option_escape() {
        let key = ExitKey::parse("Ctrl+Option+Escape").unwrap();
        assert_eq!(key.keycode, 53);
        assert!(!key.requires_cmd);
        assert!(key.requires_option);
        assert!(!key.requires_shift);
        assert!(key.requires_ctrl);
    }

    #[test]
    fn test_exit_key_parse_case_insensitive() {
        let key1 = ExitKey::parse("CMD+OPTION+U").unwrap();
        let key2 = ExitKey::parse("cmd+option+u").unwrap();
        assert_eq!(key1.keycode, key2.keycode);
        assert_eq!(key1.requires_cmd, key2.requires_cmd);
        assert_eq!(key1.requires_option, key2.requires_option);
    }

    #[test]
    fn test_exit_key_parse_alternative_modifier_names() {
        let key = ExitKey::parse("Command+Alt+U").unwrap();
        assert!(key.requires_cmd);
        assert!(key.requires_option);

        let key2 = ExitKey::parse("Control+Opt+X").unwrap();
        assert!(key2.requires_ctrl);
        assert!(key2.requires_option);
    }

    #[test]
    fn test_exit_key_parse_with_spaces() {
        let key = ExitKey::parse(" Cmd + Option + U ").unwrap();
        assert_eq!(key.keycode, 32);
        assert!(key.requires_cmd);
        assert!(key.requires_option);
    }

    #[test]
    fn test_exit_key_parse_errors() {
        // No modifier
        assert!(ExitKey::parse("U").is_err());

        // Unknown key
        assert!(ExitKey::parse("Cmd+Option+Unknown").is_err());

        // Empty
        assert!(ExitKey::parse("").is_err());

        // No key, only modifiers
        assert!(ExitKey::parse("Cmd+Option").is_err());

        // Multiple keys
        assert!(ExitKey::parse("Cmd+A+B").is_err());
    }

    #[test]
    fn test_exit_key_default() {
        let key = ExitKey::default();
        assert_eq!(key.keycode, 32);
        assert!(key.requires_cmd);
        assert!(key.requires_option);
        assert!(!key.requires_shift);
        assert!(!key.requires_ctrl);
        assert_eq!(key.display_name, "Cmd+Option+U");
    }
}
