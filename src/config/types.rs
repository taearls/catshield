//! Configuration type definitions for Cat Shield

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

/// Default overlay opacity when not specified
pub const DEFAULT_OVERLAY_OPACITY: f64 = 0.5;
/// Minimum allowed overlay opacity (10%)
pub const MIN_OVERLAY_OPACITY: f64 = 0.1;
/// Maximum allowed overlay opacity (90%)
pub const MAX_OVERLAY_OPACITY: f64 = 0.9;
/// Default overlay color preset
pub const DEFAULT_OVERLAY_COLOR: &str = "gray";

/// Configuration file structure for persistent settings
#[derive(Debug, Deserialize, Serialize, Default, Clone)]
pub struct Config {
    /// Custom exit key combination (e.g., "Cmd+Option+U")
    pub exit_key: Option<String>,

    /// Default auto-exit timer duration (e.g., "30m", "1h")
    pub default_timer: Option<String>,

    /// Overlay opacity (0.1 to 0.9, default 0.5)
    pub overlay_opacity: Option<f64>,

    /// Overlay color preset ("gray", "blue", "green", "red", "purple") or hex color (e.g., "#FF5500")
    pub overlay_color: Option<String>,

    /// Keys that are allowed to pass through the shield (e.g., ["Cmd+Space", "F11", "F12"])
    pub allowed_keys: Option<Vec<String>>,

    /// Whether to launch Cat Shield automatically at login (default: false)
    pub launch_at_login: Option<bool>,

    /// Enable trace logging to file (default: false)
    /// When enabled, detailed event traces are written to ~/.config/catshield/logs/
    pub enable_trace_logging: Option<bool>,
}

impl Config {
    /// Get the path to the config file (~/.config/catshield/config.toml)
    pub fn config_path() -> Option<PathBuf> {
        dirs::config_dir().map(|p| p.join("catshield").join("config.toml"))
    }

    /// Load configuration from the config file, if it exists
    pub fn load() -> Self {
        let Some(path) = Self::config_path() else {
            return Self::default();
        };

        if !path.exists() {
            return Self::default();
        }

        match fs::read_to_string(&path) {
            Ok(contents) => match toml::from_str(&contents) {
                Ok(config) => config,
                Err(e) => {
                    log::warn!("Failed to parse config file: {e}");
                    Self::default()
                }
            },
            Err(e) => {
                log::warn!("Failed to read config file: {e}");
                Self::default()
            }
        }
    }

    /// Get the overlay opacity, clamped to valid range
    pub fn opacity(&self) -> f64 {
        self.overlay_opacity
            .unwrap_or(DEFAULT_OVERLAY_OPACITY)
            .clamp(MIN_OVERLAY_OPACITY, MAX_OVERLAY_OPACITY)
    }

    /// Get the overlay color setting
    pub fn color(&self) -> &str {
        self.overlay_color
            .as_deref()
            .unwrap_or(DEFAULT_OVERLAY_COLOR)
    }

    /// Save configuration to the config file atomically.
    ///
    /// Uses a write-to-temp-then-rename strategy to ensure the config file
    /// is never left in a corrupted state if the process is interrupted.
    pub fn save(&self) -> Result<(), String> {
        let path = Self::config_path().ok_or("Could not determine config path")?;

        // Create directory if it doesn't exist
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create config directory: {e}"))?;
        }

        let content =
            toml::to_string_pretty(self).map_err(|e| format!("Failed to serialize config: {e}"))?;

        // Write to a temporary file first, then rename for atomicity
        let temp_path = path.with_extension("toml.tmp");
        fs::write(&temp_path, &content)
            .map_err(|e| format!("Failed to write temporary config file: {e}"))?;

        // Atomically rename temp file to final path
        // On Unix, rename() is atomic. On Windows, it's not fully atomic but
        // is still safer than direct write as it reduces the window for corruption.
        fs::rename(&temp_path, &path).map_err(|e| {
            // Try to clean up the temp file if rename fails
            let _ = fs::remove_file(&temp_path);
            format!("Failed to save config file: {e}")
        })?;

        Ok(())
    }

    /// Parse overlay color to RGB values (r, g, b each 0.0-1.0)
    ///
    /// Supports:
    /// - Preset names: "gray", "blue", "green", "red", "purple"
    /// - Hex colors: "#RRGGBB" or "#RGB"
    ///
    /// Returns None if the color string is invalid.
    pub fn parse_color_to_rgb(color: &str) -> Option<(f32, f32, f32)> {
        let color_lower = color.to_lowercase();

        // Check preset colors first
        match color_lower.as_str() {
            "gray" | "grey" | "dark" => Some((0.1, 0.1, 0.1)),
            "blue" => Some((0.05, 0.1, 0.2)),
            "green" => Some((0.05, 0.15, 0.1)),
            "red" => Some((0.15, 0.05, 0.05)),
            "purple" => Some((0.12, 0.08, 0.18)),
            _ if color.starts_with('#') => Self::parse_hex_color(color),
            _ => None,
        }
    }

    /// Parse a hex color string to RGB values
    ///
    /// Supports both #RRGGBB and #RGB formats.
    fn parse_hex_color(hex: &str) -> Option<(f32, f32, f32)> {
        let hex = hex.trim_start_matches('#');

        match hex.len() {
            // #RGB format
            3 => {
                let r = u8::from_str_radix(&hex[0..1], 16).ok()? * 17;
                let g = u8::from_str_radix(&hex[1..2], 16).ok()? * 17;
                let b = u8::from_str_radix(&hex[2..3], 16).ok()? * 17;
                Some((r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0))
            }
            // #RRGGBB format
            6 => {
                let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
                let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
                let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
                Some((r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0))
            }
            _ => None,
        }
    }

    /// Load configuration from a specific path (for testing)
    #[cfg(test)]
    pub fn load_from_path(path: &std::path::Path) -> Self {
        if !path.exists() {
            return Self::default();
        }

        match fs::read_to_string(path) {
            Ok(contents) => toml::from_str(&contents).unwrap_or_default(),
            Err(_) => Self::default(),
        }
    }

    /// Save configuration to a specific path (for testing)
    ///
    /// Uses atomic write-then-rename like `save()`.
    #[cfg(test)]
    pub fn save_to_path(&self, path: &std::path::Path) -> Result<(), String> {
        // Create directory if it doesn't exist
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create config directory: {e}"))?;
        }

        let content =
            toml::to_string_pretty(self).map_err(|e| format!("Failed to serialize config: {e}"))?;

        // Write to temp file first, then rename for atomicity
        let temp_path = path.with_extension("toml.tmp");
        fs::write(&temp_path, &content)
            .map_err(|e| format!("Failed to write temporary config file: {e}"))?;

        fs::rename(&temp_path, path).map_err(|e| {
            let _ = fs::remove_file(&temp_path);
            format!("Failed to save config file: {e}")
        })?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_load_missing_file_returns_default() {
        let temp_dir = tempfile::tempdir().unwrap();
        let config_path = temp_dir.path().join("nonexistent.toml");

        let config = Config::load_from_path(&config_path);

        assert!(config.exit_key.is_none());
        assert!(config.default_timer.is_none());
        assert!(config.overlay_opacity.is_none());
    }

    #[test]
    fn test_config_load_empty_file_returns_default() {
        let temp_dir = tempfile::tempdir().unwrap();
        let config_path = temp_dir.path().join("config.toml");
        std::fs::write(&config_path, "").unwrap();

        let config = Config::load_from_path(&config_path);

        assert!(config.exit_key.is_none());
        assert!(config.default_timer.is_none());
        assert!(config.overlay_opacity.is_none());
    }

    #[test]
    fn test_config_load_valid_toml() {
        let temp_dir = tempfile::tempdir().unwrap();
        let config_path = temp_dir.path().join("config.toml");
        std::fs::write(
            &config_path,
            r#"
exit_key = "Cmd+Shift+Q"
default_timer = "30m"
overlay_opacity = 0.6
"#,
        )
        .unwrap();

        let config = Config::load_from_path(&config_path);

        assert_eq!(config.exit_key, Some("Cmd+Shift+Q".to_string()));
        assert_eq!(config.default_timer, Some("30m".to_string()));
        assert_eq!(config.overlay_opacity, Some(0.6));
    }

    #[test]
    fn test_config_load_partial_toml_uses_defaults() {
        let temp_dir = tempfile::tempdir().unwrap();
        let config_path = temp_dir.path().join("config.toml");
        std::fs::write(&config_path, r#"exit_key = "Cmd+Shift+Q""#).unwrap();

        let config = Config::load_from_path(&config_path);

        assert_eq!(config.exit_key, Some("Cmd+Shift+Q".to_string()));
        assert!(config.default_timer.is_none());
        assert!(config.overlay_opacity.is_none());
    }

    #[test]
    fn test_config_load_invalid_toml_returns_default() {
        let temp_dir = tempfile::tempdir().unwrap();
        let config_path = temp_dir.path().join("config.toml");
        std::fs::write(&config_path, "this is not valid toml {{{{").unwrap();

        let config = Config::load_from_path(&config_path);

        assert!(config.exit_key.is_none());
        assert!(config.default_timer.is_none());
        assert!(config.overlay_opacity.is_none());
    }

    #[test]
    fn test_config_load_with_unknown_fields_ignores_them() {
        let temp_dir = tempfile::tempdir().unwrap();
        let config_path = temp_dir.path().join("config.toml");
        std::fs::write(
            &config_path,
            r#"
exit_key = "Cmd+Option+X"
unknown_field = "should be ignored"
another_unknown = 42
"#,
        )
        .unwrap();

        let config = Config::load_from_path(&config_path);

        assert_eq!(config.exit_key, Some("Cmd+Option+X".to_string()));
    }

    #[test]
    fn test_config_save_creates_directory() {
        let temp_dir = tempfile::tempdir().unwrap();
        let config_path = temp_dir.path().join("subdir").join("config.toml");

        let config = Config {
            exit_key: Some("Cmd+Option+U".to_string()),
            default_timer: None,
            overlay_opacity: None,
            overlay_color: None,
            allowed_keys: None,
            launch_at_login: None,
            enable_trace_logging: None,
        };

        config.save_to_path(&config_path).unwrap();

        assert!(config_path.parent().unwrap().exists());
    }

    #[test]
    fn test_config_save_creates_file() {
        let temp_dir = tempfile::tempdir().unwrap();
        let config_path = temp_dir.path().join("config.toml");

        let config = Config {
            exit_key: Some("Cmd+Option+U".to_string()),
            default_timer: None,
            overlay_opacity: None,
            overlay_color: None,
            allowed_keys: None,
            launch_at_login: None,
            enable_trace_logging: None,
        };

        config.save_to_path(&config_path).unwrap();

        assert!(config_path.exists());
    }

    #[test]
    fn test_config_save_writes_valid_toml() {
        let temp_dir = tempfile::tempdir().unwrap();
        let config_path = temp_dir.path().join("config.toml");

        let config = Config {
            exit_key: Some("Cmd+Shift+X".to_string()),
            default_timer: Some("1h".to_string()),
            overlay_opacity: Some(0.7),
            overlay_color: None,
            allowed_keys: None,
            launch_at_login: None,
            enable_trace_logging: None,
        };

        config.save_to_path(&config_path).unwrap();

        let content = std::fs::read_to_string(&config_path).unwrap();
        assert!(content.contains("exit_key = \"Cmd+Shift+X\""));
        assert!(content.contains("default_timer = \"1h\""));
        assert!(content.contains("overlay_opacity = 0.7"));
    }

    #[test]
    fn test_config_save_overwrites_existing() {
        let temp_dir = tempfile::tempdir().unwrap();
        let config_path = temp_dir.path().join("config.toml");

        // Write initial config
        let config1 = Config {
            exit_key: Some("Cmd+Option+A".to_string()),
            default_timer: None,
            overlay_opacity: None,
            overlay_color: None,
            allowed_keys: None,
            launch_at_login: None,
            enable_trace_logging: None,
        };
        config1.save_to_path(&config_path).unwrap();

        // Overwrite with new config
        let config2 = Config {
            exit_key: Some("Cmd+Option+B".to_string()),
            default_timer: Some("2h".to_string()),
            overlay_opacity: Some(0.3),
            overlay_color: None,
            allowed_keys: None,
            launch_at_login: None,
            enable_trace_logging: None,
        };
        config2.save_to_path(&config_path).unwrap();

        let content = std::fs::read_to_string(&config_path).unwrap();
        assert!(content.contains("exit_key = \"Cmd+Option+B\""));
        assert!(content.contains("default_timer = \"2h\""));
        assert!(!content.contains("Cmd+Option+A"));
    }

    #[test]
    fn test_config_round_trip_all_fields() {
        let temp_dir = tempfile::tempdir().unwrap();
        let config_path = temp_dir.path().join("config.toml");

        let original = Config {
            exit_key: Some("Ctrl+Option+Escape".to_string()),
            default_timer: Some("45m".to_string()),
            overlay_opacity: Some(0.65),
            overlay_color: None,
            allowed_keys: None,
            launch_at_login: Some(true),
            enable_trace_logging: None,
        };

        original.save_to_path(&config_path).unwrap();
        let loaded = Config::load_from_path(&config_path);

        assert_eq!(original.exit_key, loaded.exit_key);
        assert_eq!(original.default_timer, loaded.default_timer);
        assert_eq!(original.overlay_opacity, loaded.overlay_opacity);
        assert_eq!(original.launch_at_login, loaded.launch_at_login);
    }

    #[test]
    fn test_config_round_trip_partial_fields() {
        let temp_dir = tempfile::tempdir().unwrap();
        let config_path = temp_dir.path().join("config.toml");

        let original = Config {
            exit_key: Some("Cmd+Shift+Q".to_string()),
            default_timer: None,
            overlay_opacity: Some(0.4),
            overlay_color: None,
            allowed_keys: None,
            launch_at_login: None,
            enable_trace_logging: None,
        };

        original.save_to_path(&config_path).unwrap();
        let loaded = Config::load_from_path(&config_path);

        assert_eq!(original.exit_key, loaded.exit_key);
        assert!(loaded.default_timer.is_none());
        assert_eq!(original.overlay_opacity, loaded.overlay_opacity);
    }

    #[test]
    fn test_config_opacity_default_when_none() {
        let config = Config {
            exit_key: None,
            default_timer: None,
            overlay_opacity: None,
            overlay_color: None,
            allowed_keys: None,
            launch_at_login: None,
            enable_trace_logging: None,
        };

        assert_eq!(config.opacity(), DEFAULT_OVERLAY_OPACITY);
        assert_eq!(config.opacity(), 0.5);
    }

    #[test]
    fn test_config_opacity_clamps_below_min() {
        let config = Config {
            exit_key: None,
            default_timer: None,
            overlay_opacity: Some(0.05), // Below MIN_OVERLAY_OPACITY (0.1)
            overlay_color: None,
            allowed_keys: None,
            launch_at_login: None,
            enable_trace_logging: None,
        };

        assert_eq!(config.opacity(), MIN_OVERLAY_OPACITY);
        assert_eq!(config.opacity(), 0.1);
    }

    #[test]
    fn test_config_opacity_clamps_above_max() {
        let config = Config {
            exit_key: None,
            default_timer: None,
            overlay_opacity: Some(0.95), // Above MAX_OVERLAY_OPACITY (0.9)
            overlay_color: None,
            allowed_keys: None,
            launch_at_login: None,
            enable_trace_logging: None,
        };

        assert_eq!(config.opacity(), MAX_OVERLAY_OPACITY);
        assert_eq!(config.opacity(), 0.9);
    }

    #[test]
    fn test_config_opacity_valid_value_unchanged() {
        let config = Config {
            exit_key: None,
            default_timer: None,
            overlay_opacity: Some(0.6),
            overlay_color: None,
            allowed_keys: None,
            launch_at_login: None,
            enable_trace_logging: None,
        };

        assert_eq!(config.opacity(), 0.6);
    }

    #[test]
    fn test_config_allowed_keys_single_key() {
        let temp_dir = tempfile::tempdir().unwrap();
        let config_path = temp_dir.path().join("config.toml");
        std::fs::write(
            &config_path,
            r#"
allowed_keys = ["Cmd+Space"]
"#,
        )
        .unwrap();

        let config = Config::load_from_path(&config_path);

        assert_eq!(config.allowed_keys, Some(vec!["Cmd+Space".to_string()]));
    }

    #[test]
    fn test_config_allowed_keys_multiple_keys() {
        let temp_dir = tempfile::tempdir().unwrap();
        let config_path = temp_dir.path().join("config.toml");
        std::fs::write(
            &config_path,
            r#"
allowed_keys = ["Cmd+Space", "F11", "F12", "Ctrl+Option+A"]
"#,
        )
        .unwrap();

        let config = Config::load_from_path(&config_path);

        assert_eq!(
            config.allowed_keys,
            Some(vec![
                "Cmd+Space".to_string(),
                "F11".to_string(),
                "F12".to_string(),
                "Ctrl+Option+A".to_string()
            ])
        );
    }

    #[test]
    fn test_config_allowed_keys_empty_array() {
        let temp_dir = tempfile::tempdir().unwrap();
        let config_path = temp_dir.path().join("config.toml");
        std::fs::write(&config_path, r#"allowed_keys = []"#).unwrap();

        let config = Config::load_from_path(&config_path);

        assert_eq!(config.allowed_keys, Some(vec![]));
    }

    #[test]
    fn test_config_allowed_keys_none_when_missing() {
        let temp_dir = tempfile::tempdir().unwrap();
        let config_path = temp_dir.path().join("config.toml");
        std::fs::write(&config_path, r#"exit_key = "Cmd+Q""#).unwrap();

        let config = Config::load_from_path(&config_path);

        assert!(config.allowed_keys.is_none());
    }

    #[test]
    fn test_config_round_trip_with_allowed_keys() {
        let temp_dir = tempfile::tempdir().unwrap();
        let config_path = temp_dir.path().join("config.toml");

        let original = Config {
            exit_key: Some("Cmd+Q".to_string()),
            default_timer: Some("30m".to_string()),
            overlay_opacity: Some(0.6),
            overlay_color: None,
            allowed_keys: Some(vec![
                "Cmd+Space".to_string(),
                "F11".to_string(),
                "F12".to_string(),
            ]),
            launch_at_login: None,
            enable_trace_logging: None,
        };

        original.save_to_path(&config_path).unwrap();
        let loaded = Config::load_from_path(&config_path);

        assert_eq!(original.exit_key, loaded.exit_key);
        assert_eq!(original.default_timer, loaded.default_timer);
        assert_eq!(original.overlay_opacity, loaded.overlay_opacity);
        assert_eq!(original.allowed_keys, loaded.allowed_keys);
    }

    #[test]
    fn test_config_launch_at_login_true() {
        let temp_dir = tempfile::tempdir().unwrap();
        let config_path = temp_dir.path().join("config.toml");
        std::fs::write(&config_path, r#"launch_at_login = true"#).unwrap();

        let config = Config::load_from_path(&config_path);

        assert_eq!(config.launch_at_login, Some(true));
    }

    #[test]
    fn test_config_launch_at_login_false() {
        let temp_dir = tempfile::tempdir().unwrap();
        let config_path = temp_dir.path().join("config.toml");
        std::fs::write(&config_path, r#"launch_at_login = false"#).unwrap();

        let config = Config::load_from_path(&config_path);

        assert_eq!(config.launch_at_login, Some(false));
    }

    #[test]
    fn test_config_launch_at_login_none_when_missing() {
        let temp_dir = tempfile::tempdir().unwrap();
        let config_path = temp_dir.path().join("config.toml");
        std::fs::write(&config_path, r#"exit_key = "Cmd+Q""#).unwrap();

        let config = Config::load_from_path(&config_path);

        assert!(config.launch_at_login.is_none());
    }

    #[test]
    fn test_config_launch_at_login_round_trip() {
        let temp_dir = tempfile::tempdir().unwrap();
        let config_path = temp_dir.path().join("config.toml");

        let original = Config {
            exit_key: None,
            default_timer: None,
            overlay_opacity: None,
            overlay_color: None,
            allowed_keys: None,
            launch_at_login: Some(true),
            enable_trace_logging: None,
        };

        original.save_to_path(&config_path).unwrap();
        let loaded = Config::load_from_path(&config_path);

        assert_eq!(original.launch_at_login, loaded.launch_at_login);
    }

    #[test]
    fn test_config_enable_trace_logging_true() {
        let temp_dir = tempfile::tempdir().unwrap();
        let config_path = temp_dir.path().join("config.toml");
        std::fs::write(&config_path, r#"enable_trace_logging = true"#).unwrap();

        let config = Config::load_from_path(&config_path);

        assert_eq!(config.enable_trace_logging, Some(true));
    }

    #[test]
    fn test_config_enable_trace_logging_false() {
        let temp_dir = tempfile::tempdir().unwrap();
        let config_path = temp_dir.path().join("config.toml");
        std::fs::write(&config_path, r#"enable_trace_logging = false"#).unwrap();

        let config = Config::load_from_path(&config_path);

        assert_eq!(config.enable_trace_logging, Some(false));
    }

    #[test]
    fn test_config_enable_trace_logging_none_when_missing() {
        let temp_dir = tempfile::tempdir().unwrap();
        let config_path = temp_dir.path().join("config.toml");
        std::fs::write(&config_path, r#"exit_key = "Cmd+Q""#).unwrap();

        let config = Config::load_from_path(&config_path);

        assert!(config.enable_trace_logging.is_none());
    }

    #[test]
    fn test_config_enable_trace_logging_round_trip() {
        let temp_dir = tempfile::tempdir().unwrap();
        let config_path = temp_dir.path().join("config.toml");

        let original = Config {
            exit_key: None,
            default_timer: None,
            overlay_opacity: None,
            overlay_color: None,
            allowed_keys: None,
            launch_at_login: None,
            enable_trace_logging: Some(true),
        };

        original.save_to_path(&config_path).unwrap();
        let loaded = Config::load_from_path(&config_path);

        assert_eq!(original.enable_trace_logging, loaded.enable_trace_logging);
    }

    // ============================================================
    // Atomic save tests (Issue #158 - Settings Persistence)
    // ============================================================

    #[test]
    fn test_config_save_atomic_no_temp_file_left() {
        let temp_dir = tempfile::tempdir().unwrap();
        let config_path = temp_dir.path().join("config.toml");

        let config = Config {
            exit_key: Some("Cmd+Option+U".to_string()),
            default_timer: Some("30m".to_string()),
            overlay_opacity: Some(0.6),
            overlay_color: None,
            allowed_keys: None,
            launch_at_login: None,
            enable_trace_logging: None,
        };

        config.save_to_path(&config_path).unwrap();

        // Verify temp file doesn't exist after successful save
        let temp_path = config_path.with_extension("toml.tmp");
        assert!(
            !temp_path.exists(),
            "Temp file should be removed after save"
        );
        assert!(config_path.exists(), "Config file should exist");
    }

    #[test]
    fn test_config_save_atomic_overwrites_cleanly() {
        let temp_dir = tempfile::tempdir().unwrap();
        let config_path = temp_dir.path().join("config.toml");

        // Save initial config
        let config1 = Config {
            exit_key: Some("Cmd+Option+A".to_string()),
            default_timer: None,
            overlay_opacity: Some(0.3),
            overlay_color: None,
            allowed_keys: None,
            launch_at_login: None,
            enable_trace_logging: None,
        };
        config1.save_to_path(&config_path).unwrap();

        // Save updated config
        let config2 = Config {
            exit_key: Some("Cmd+Option+B".to_string()),
            default_timer: Some("1h".to_string()),
            overlay_opacity: Some(0.7),
            overlay_color: None,
            allowed_keys: Some(vec!["F11".to_string()]),
            launch_at_login: Some(true),
            enable_trace_logging: Some(false),
        };
        config2.save_to_path(&config_path).unwrap();

        // Verify the new config was saved correctly
        let loaded = Config::load_from_path(&config_path);
        assert_eq!(loaded.exit_key, Some("Cmd+Option+B".to_string()));
        assert_eq!(loaded.default_timer, Some("1h".to_string()));
        assert_eq!(loaded.overlay_opacity, Some(0.7));
        assert_eq!(loaded.allowed_keys, Some(vec!["F11".to_string()]));
        assert_eq!(loaded.launch_at_login, Some(true));
        assert_eq!(loaded.enable_trace_logging, Some(false));
    }

    // ============================================================
    // Full persistence workflow tests (Issue #158 Acceptance Criteria)
    // ============================================================

    /// Acceptance Criteria: Settings persist across restarts
    #[test]
    fn test_settings_persist_across_simulated_restart() {
        let temp_dir = tempfile::tempdir().unwrap();
        let config_path = temp_dir.path().join("config.toml");

        // Simulate first "session" - user changes settings and saves
        {
            let config = Config {
                exit_key: Some("Cmd+Shift+Escape".to_string()),
                default_timer: Some("45m".to_string()),
                overlay_opacity: Some(0.65),
                overlay_color: Some("blue".to_string()),
                allowed_keys: Some(vec!["Cmd+Space".to_string(), "F12".to_string()]),
                launch_at_login: Some(true),
                enable_trace_logging: Some(false),
            };
            config.save_to_path(&config_path).unwrap();
        }

        // Simulate "restart" - load settings from disk
        {
            let loaded = Config::load_from_path(&config_path);

            // All settings should be preserved
            assert_eq!(loaded.exit_key, Some("Cmd+Shift+Escape".to_string()));
            assert_eq!(loaded.default_timer, Some("45m".to_string()));
            assert_eq!(loaded.overlay_opacity, Some(0.65));
            assert_eq!(
                loaded.allowed_keys,
                Some(vec!["Cmd+Space".to_string(), "F12".to_string()])
            );
            assert_eq!(loaded.launch_at_login, Some(true));
            assert_eq!(loaded.enable_trace_logging, Some(false));
        }
    }

    /// Acceptance Criteria: Missing file handled gracefully
    #[test]
    fn test_missing_file_handled_gracefully() {
        let temp_dir = tempfile::tempdir().unwrap();
        let config_path = temp_dir
            .path()
            .join("nonexistent_subdir")
            .join("config.toml");

        // Loading from nonexistent path should return defaults, not panic
        let config = Config::load_from_path(&config_path);

        // Should get default values for all fields
        assert!(config.exit_key.is_none());
        assert!(config.default_timer.is_none());
        assert!(config.overlay_opacity.is_none());
        assert!(config.allowed_keys.is_none());
        assert!(config.launch_at_login.is_none());
        assert!(config.enable_trace_logging.is_none());

        // Default opacity should still work
        assert_eq!(config.opacity(), DEFAULT_OVERLAY_OPACITY);
    }

    /// Acceptance Criteria: Corrupted file handled gracefully
    #[test]
    fn test_corrupted_file_handled_gracefully_binary_garbage() {
        let temp_dir = tempfile::tempdir().unwrap();
        let config_path = temp_dir.path().join("config.toml");

        // Write binary garbage to the config file
        std::fs::write(&config_path, [0xFF, 0xFE, 0x00, 0x01, 0xAB, 0xCD]).unwrap();

        // Should return defaults, not panic
        let config = Config::load_from_path(&config_path);
        assert!(config.exit_key.is_none());
        assert_eq!(config.opacity(), DEFAULT_OVERLAY_OPACITY);
    }

    /// Acceptance Criteria: Corrupted file handled gracefully
    #[test]
    fn test_corrupted_file_handled_gracefully_truncated() {
        let temp_dir = tempfile::tempdir().unwrap();
        let config_path = temp_dir.path().join("config.toml");

        // Write truncated/incomplete TOML
        std::fs::write(&config_path, r#"exit_key = "Cmd+Option"#).unwrap();

        // Should return defaults (the truncated string will be parsed as valid)
        // Actually TOML allows unterminated strings at EOF in some parsers
        // but the key test is that it doesn't panic
        let config = Config::load_from_path(&config_path);

        // Either it parses the truncated value or returns default - both are acceptable
        // The important thing is no panic
        assert!(config.default_timer.is_none());
    }

    /// Acceptance Criteria: Corrupted file handled gracefully
    #[test]
    fn test_corrupted_file_handled_gracefully_wrong_types() {
        let temp_dir = tempfile::tempdir().unwrap();
        let config_path = temp_dir.path().join("config.toml");

        // Write TOML with wrong types (number where string expected)
        std::fs::write(
            &config_path,
            r#"
exit_key = 12345
overlay_opacity = "not a number"
"#,
        )
        .unwrap();

        // Should return defaults due to type mismatch
        let config = Config::load_from_path(&config_path);
        assert!(config.exit_key.is_none());
        assert!(config.overlay_opacity.is_none());
        assert_eq!(config.opacity(), DEFAULT_OVERLAY_OPACITY);
    }

    /// Test that all config fields round-trip correctly (comprehensive)
    #[test]
    fn test_full_config_round_trip_all_fields() {
        let temp_dir = tempfile::tempdir().unwrap();
        let config_path = temp_dir.path().join("config.toml");

        let original = Config {
            exit_key: Some("Ctrl+Alt+Delete".to_string()),
            default_timer: Some("2h30m".to_string()),
            overlay_opacity: Some(0.42),
            overlay_color: Some("#FF5500".to_string()),
            allowed_keys: Some(vec![
                "F1".to_string(),
                "F2".to_string(),
                "Cmd+Tab".to_string(),
            ]),
            launch_at_login: Some(true),
            enable_trace_logging: Some(true),
        };

        original.save_to_path(&config_path).unwrap();
        let loaded = Config::load_from_path(&config_path);

        // Verify every single field
        assert_eq!(original.exit_key, loaded.exit_key);
        assert_eq!(original.default_timer, loaded.default_timer);
        assert_eq!(original.overlay_opacity, loaded.overlay_opacity);
        assert_eq!(original.overlay_color, loaded.overlay_color);
        assert_eq!(original.allowed_keys, loaded.allowed_keys);
        assert_eq!(original.launch_at_login, loaded.launch_at_login);
        assert_eq!(original.enable_trace_logging, loaded.enable_trace_logging);
    }

    // ============================================================
    // Color parsing tests (Issue #159 - Overlay Customization)
    // ============================================================

    #[test]
    fn test_parse_color_preset_gray() {
        let (r, g, b) = Config::parse_color_to_rgb("gray").unwrap();
        assert!((r - 0.1).abs() < f32::EPSILON);
        assert!((g - 0.1).abs() < f32::EPSILON);
        assert!((b - 0.1).abs() < f32::EPSILON);
    }

    #[test]
    fn test_parse_color_preset_grey_alias() {
        let result = Config::parse_color_to_rgb("grey");
        assert!(result.is_some());
    }

    #[test]
    fn test_parse_color_preset_blue() {
        let (r, g, b) = Config::parse_color_to_rgb("blue").unwrap();
        assert!((r - 0.05).abs() < f32::EPSILON);
        assert!((g - 0.1).abs() < f32::EPSILON);
        assert!((b - 0.2).abs() < f32::EPSILON);
    }

    #[test]
    fn test_parse_color_preset_green() {
        let (r, g, b) = Config::parse_color_to_rgb("green").unwrap();
        assert!((r - 0.05).abs() < f32::EPSILON);
        assert!((g - 0.15).abs() < f32::EPSILON);
        assert!((b - 0.1).abs() < f32::EPSILON);
    }

    #[test]
    fn test_parse_color_preset_red() {
        let (r, g, b) = Config::parse_color_to_rgb("red").unwrap();
        assert!((r - 0.15).abs() < f32::EPSILON);
        assert!((g - 0.05).abs() < f32::EPSILON);
        assert!((b - 0.05).abs() < f32::EPSILON);
    }

    #[test]
    fn test_parse_color_preset_purple() {
        let (r, g, b) = Config::parse_color_to_rgb("purple").unwrap();
        assert!((r - 0.12).abs() < f32::EPSILON);
        assert!((g - 0.08).abs() < f32::EPSILON);
        assert!((b - 0.18).abs() < f32::EPSILON);
    }

    #[test]
    fn test_parse_color_preset_case_insensitive() {
        assert!(Config::parse_color_to_rgb("BLUE").is_some());
        assert!(Config::parse_color_to_rgb("Blue").is_some());
        assert!(Config::parse_color_to_rgb("bLuE").is_some());
    }

    #[test]
    fn test_parse_color_hex_6_digits() {
        let (r, g, b) = Config::parse_color_to_rgb("#FF0000").unwrap();
        assert!((r - 1.0).abs() < f32::EPSILON);
        assert!((g - 0.0).abs() < f32::EPSILON);
        assert!((b - 0.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_parse_color_hex_3_digits() {
        let (r, g, b) = Config::parse_color_to_rgb("#F00").unwrap();
        assert!((r - 1.0).abs() < f32::EPSILON);
        assert!((g - 0.0).abs() < f32::EPSILON);
        assert!((b - 0.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_parse_color_hex_lowercase() {
        let result = Config::parse_color_to_rgb("#abc");
        assert!(result.is_some());
    }

    #[test]
    fn test_parse_color_hex_mixed_case() {
        let result = Config::parse_color_to_rgb("#AaBbCc");
        assert!(result.is_some());
    }

    #[test]
    fn test_parse_color_invalid_preset() {
        assert!(Config::parse_color_to_rgb("invalid").is_none());
        assert!(Config::parse_color_to_rgb("orange").is_none());
    }

    #[test]
    fn test_parse_color_invalid_hex() {
        assert!(Config::parse_color_to_rgb("#GG0000").is_none());
        assert!(Config::parse_color_to_rgb("#12345").is_none()); // 5 digits
        assert!(Config::parse_color_to_rgb("#1234567").is_none()); // 7 digits
        assert!(Config::parse_color_to_rgb("FF0000").is_none()); // Missing #
    }

    #[test]
    fn test_config_color_default_when_none() {
        let config = Config::default();
        assert_eq!(config.color(), "gray");
    }

    #[test]
    fn test_config_color_returns_value() {
        let config = Config {
            overlay_color: Some("blue".to_string()),
            ..Default::default()
        };
        assert_eq!(config.color(), "blue");
    }

    #[test]
    fn test_config_color_round_trip() {
        let temp_dir = tempfile::tempdir().unwrap();
        let config_path = temp_dir.path().join("config.toml");

        let original = Config {
            exit_key: None,
            default_timer: None,
            overlay_opacity: None,
            overlay_color: Some("purple".to_string()),
            allowed_keys: None,
            launch_at_login: None,
            enable_trace_logging: None,
        };

        original.save_to_path(&config_path).unwrap();
        let loaded = Config::load_from_path(&config_path);

        assert_eq!(loaded.overlay_color, Some("purple".to_string()));
    }

    #[test]
    fn test_config_hex_color_round_trip() {
        let temp_dir = tempfile::tempdir().unwrap();
        let config_path = temp_dir.path().join("config.toml");

        let original = Config {
            exit_key: None,
            default_timer: None,
            overlay_opacity: None,
            overlay_color: Some("#1a2b3c".to_string()),
            allowed_keys: None,
            launch_at_login: None,
            enable_trace_logging: None,
        };

        original.save_to_path(&config_path).unwrap();
        let loaded = Config::load_from_path(&config_path);

        assert_eq!(loaded.overlay_color, Some("#1a2b3c".to_string()));
    }
}
