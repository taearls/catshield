//! Configuration type definitions for Cat Shield

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

/// Default overlay opacity when not specified
pub const DEFAULT_OVERLAY_OPACITY: f64 = 0.5;
/// Minimum allowed overlay opacity
pub const MIN_OVERLAY_OPACITY: f64 = 0.2;
/// Maximum allowed overlay opacity
pub const MAX_OVERLAY_OPACITY: f64 = 0.8;

/// Default cat animation speed multiplier
pub const DEFAULT_CAT_ANIMATION_SPEED: f64 = 1.0;
/// Minimum cat animation speed
pub const MIN_CAT_ANIMATION_SPEED: f64 = 0.5;
/// Maximum cat animation speed
pub const MAX_CAT_ANIMATION_SPEED: f64 = 2.0;

/// Configuration file structure for persistent settings
#[derive(Debug, Deserialize, Serialize, Default, Clone)]
pub struct Config {
    /// Custom exit key combination (e.g., "Cmd+Option+U")
    pub exit_key: Option<String>,

    /// Default auto-exit timer duration (e.g., "30m", "1h")
    pub default_timer: Option<String>,

    /// Overlay opacity (0.2 to 0.8, default 0.5)
    pub overlay_opacity: Option<f64>,

    /// Keys that are allowed to pass through the shield (e.g., ["Cmd+Space", "F11", "F12"])
    pub allowed_keys: Option<Vec<String>>,

    /// Whether to show the animated cat companion on the shield overlay (default: true)
    pub show_cat_animation: Option<bool>,

    /// Animation speed multiplier for the cat (0.5 to 2.0, default 1.0)
    pub cat_animation_speed: Option<f64>,
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
                    log::warn!("Failed to parse config file: {}", e);
                    Self::default()
                }
            },
            Err(e) => {
                log::warn!("Failed to read config file: {}", e);
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

    /// Get whether to show the cat animation (defaults to true)
    pub fn show_cat_animation(&self) -> bool {
        self.show_cat_animation.unwrap_or(true)
    }

    /// Get the cat animation speed, clamped to valid range
    pub fn cat_animation_speed(&self) -> f64 {
        self.cat_animation_speed
            .unwrap_or(DEFAULT_CAT_ANIMATION_SPEED)
            .clamp(MIN_CAT_ANIMATION_SPEED, MAX_CAT_ANIMATION_SPEED)
    }

    /// Save configuration to the config file
    pub fn save(&self) -> Result<(), String> {
        let path = Self::config_path().ok_or("Could not determine config path")?;

        // Create directory if it doesn't exist
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create config directory: {}", e))?;
        }

        let content = toml::to_string_pretty(self)
            .map_err(|e| format!("Failed to serialize config: {}", e))?;
        fs::write(&path, content).map_err(|e| format!("Failed to write config file: {}", e))?;

        Ok(())
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
    #[cfg(test)]
    pub fn save_to_path(&self, path: &std::path::Path) -> Result<(), String> {
        // Create directory if it doesn't exist
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create config directory: {}", e))?;
        }

        let content = toml::to_string_pretty(self)
            .map_err(|e| format!("Failed to serialize config: {}", e))?;
        fs::write(path, content).map_err(|e| format!("Failed to write config file: {}", e))?;

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
            allowed_keys: None,
            show_cat_animation: None,
            cat_animation_speed: None,
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
            allowed_keys: None,
            show_cat_animation: None,
            cat_animation_speed: None,
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
            allowed_keys: None,
            show_cat_animation: None,
            cat_animation_speed: None,
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
            allowed_keys: None,
            show_cat_animation: None,
            cat_animation_speed: None,
        };
        config1.save_to_path(&config_path).unwrap();

        // Overwrite with new config
        let config2 = Config {
            exit_key: Some("Cmd+Option+B".to_string()),
            default_timer: Some("2h".to_string()),
            overlay_opacity: Some(0.3),
            allowed_keys: None,
            show_cat_animation: None,
            cat_animation_speed: None,
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
            allowed_keys: None,
            show_cat_animation: Some(true),
            cat_animation_speed: Some(1.5),
        };

        original.save_to_path(&config_path).unwrap();
        let loaded = Config::load_from_path(&config_path);

        assert_eq!(original.exit_key, loaded.exit_key);
        assert_eq!(original.default_timer, loaded.default_timer);
        assert_eq!(original.overlay_opacity, loaded.overlay_opacity);
    }

    #[test]
    fn test_config_round_trip_partial_fields() {
        let temp_dir = tempfile::tempdir().unwrap();
        let config_path = temp_dir.path().join("config.toml");

        let original = Config {
            exit_key: Some("Cmd+Shift+Q".to_string()),
            default_timer: None,
            overlay_opacity: Some(0.4),
            allowed_keys: None,
            show_cat_animation: None,
            cat_animation_speed: None,
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
            allowed_keys: None,
            show_cat_animation: None,
            cat_animation_speed: None,
        };

        assert_eq!(config.opacity(), DEFAULT_OVERLAY_OPACITY);
        assert_eq!(config.opacity(), 0.5);
    }

    #[test]
    fn test_config_opacity_clamps_below_min() {
        let config = Config {
            exit_key: None,
            default_timer: None,
            overlay_opacity: Some(0.1), // Below MIN_OVERLAY_OPACITY (0.2)
            allowed_keys: None,
            show_cat_animation: None,
            cat_animation_speed: None,
        };

        assert_eq!(config.opacity(), MIN_OVERLAY_OPACITY);
        assert_eq!(config.opacity(), 0.2);
    }

    #[test]
    fn test_config_opacity_clamps_above_max() {
        let config = Config {
            exit_key: None,
            default_timer: None,
            overlay_opacity: Some(0.95), // Above MAX_OVERLAY_OPACITY (0.8)
            allowed_keys: None,
            show_cat_animation: None,
            cat_animation_speed: None,
        };

        assert_eq!(config.opacity(), MAX_OVERLAY_OPACITY);
        assert_eq!(config.opacity(), 0.8);
    }

    #[test]
    fn test_config_opacity_valid_value_unchanged() {
        let config = Config {
            exit_key: None,
            default_timer: None,
            overlay_opacity: Some(0.6),
            allowed_keys: None,
            show_cat_animation: None,
            cat_animation_speed: None,
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
            allowed_keys: Some(vec![
                "Cmd+Space".to_string(),
                "F11".to_string(),
                "F12".to_string(),
            ]),
            show_cat_animation: None,
            cat_animation_speed: None,
        };

        original.save_to_path(&config_path).unwrap();
        let loaded = Config::load_from_path(&config_path);

        assert_eq!(original.exit_key, loaded.exit_key);
        assert_eq!(original.default_timer, loaded.default_timer);
        assert_eq!(original.overlay_opacity, loaded.overlay_opacity);
        assert_eq!(original.allowed_keys, loaded.allowed_keys);
    }

    // ========================================================================
    // Cat Animation Config Tests
    // ========================================================================

    #[test]
    fn test_config_show_cat_animation_default_true() {
        let config = Config::default();
        assert!(config.show_cat_animation());
    }

    #[test]
    fn test_config_show_cat_animation_explicit_false() {
        let temp_dir = tempfile::tempdir().unwrap();
        let config_path = temp_dir.path().join("config.toml");
        std::fs::write(&config_path, r#"show_cat_animation = false"#).unwrap();

        let config = Config::load_from_path(&config_path);
        assert!(!config.show_cat_animation());
    }

    #[test]
    fn test_config_show_cat_animation_explicit_true() {
        let temp_dir = tempfile::tempdir().unwrap();
        let config_path = temp_dir.path().join("config.toml");
        std::fs::write(&config_path, r#"show_cat_animation = true"#).unwrap();

        let config = Config::load_from_path(&config_path);
        assert!(config.show_cat_animation());
    }

    #[test]
    fn test_config_cat_animation_speed_default() {
        let config = Config::default();
        assert_eq!(config.cat_animation_speed(), DEFAULT_CAT_ANIMATION_SPEED);
        assert_eq!(config.cat_animation_speed(), 1.0);
    }

    #[test]
    fn test_config_cat_animation_speed_valid_value() {
        let temp_dir = tempfile::tempdir().unwrap();
        let config_path = temp_dir.path().join("config.toml");
        std::fs::write(&config_path, r#"cat_animation_speed = 1.5"#).unwrap();

        let config = Config::load_from_path(&config_path);
        assert!((config.cat_animation_speed() - 1.5).abs() < 0.001);
    }

    #[test]
    fn test_config_cat_animation_speed_clamps_below_min() {
        let temp_dir = tempfile::tempdir().unwrap();
        let config_path = temp_dir.path().join("config.toml");
        std::fs::write(&config_path, r#"cat_animation_speed = 0.1"#).unwrap();

        let config = Config::load_from_path(&config_path);
        assert_eq!(config.cat_animation_speed(), MIN_CAT_ANIMATION_SPEED);
        assert_eq!(config.cat_animation_speed(), 0.5);
    }

    #[test]
    fn test_config_cat_animation_speed_clamps_above_max() {
        let temp_dir = tempfile::tempdir().unwrap();
        let config_path = temp_dir.path().join("config.toml");
        std::fs::write(&config_path, r#"cat_animation_speed = 5.0"#).unwrap();

        let config = Config::load_from_path(&config_path);
        assert_eq!(config.cat_animation_speed(), MAX_CAT_ANIMATION_SPEED);
        assert_eq!(config.cat_animation_speed(), 2.0);
    }

    #[test]
    fn test_config_round_trip_with_cat_animation_settings() {
        let temp_dir = tempfile::tempdir().unwrap();
        let config_path = temp_dir.path().join("config.toml");

        let original = Config {
            exit_key: Some("Cmd+Q".to_string()),
            default_timer: None,
            overlay_opacity: Some(0.5),
            allowed_keys: None,
            show_cat_animation: Some(false),
            cat_animation_speed: Some(1.25),
        };

        original.save_to_path(&config_path).unwrap();
        let loaded = Config::load_from_path(&config_path);

        assert_eq!(original.show_cat_animation, loaded.show_cat_animation);
        assert_eq!(original.cat_animation_speed, loaded.cat_animation_speed);
    }
}
