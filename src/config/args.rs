//! CLI argument parsing for Cat Shield

use crate::config::{MAX_OVERLAY_OPACITY, MIN_OVERLAY_OPACITY};
use crate::input::ExitKey;
use crate::timer::parse_duration;
use clap::Parser;

/// CLI arguments for Cat Shield
#[derive(Parser, Debug)]
#[command(name = "cat_shield")]
#[command(author = "Tyler Earls")]
#[command(version)]
#[command(about = "A cat-proof screen overlay that keeps your machine awake and blocks input")]
#[command(after_help = "EXAMPLES:
    cat_shield                          # Use default exit key (Cmd+Option+U)
    cat_shield --exit-key \"Cmd+Shift+Q\" # Custom exit shortcut
    cat_shield --timer 30m              # Auto-exit after 30 minutes
    cat_shield -e \"Ctrl+Option+X\" -t 2h # Custom key + timer

CONFIG FILE:
    Settings can be persisted in ~/.config/catshield/config.toml:

    exit_key = \"Cmd+Shift+Escape\"

SUPPORTED KEYS:
    Letters: A-Z
    Numbers: 0-9
    Function keys: F1-F12
    Special: Escape, Return, Tab, Space, Delete
    Arrow keys: Left, Right, Up, Down, Home, End, PageUp, PageDown

MODIFIERS:
    Cmd (Command), Option (Alt), Shift, Ctrl (Control)")]
pub struct Args {
    /// Auto-exit after specified duration (e.g., 30m, 2h, 1h30m)
    #[arg(short, long, value_parser = parse_duration)]
    pub timer: Option<u64>,

    /// Hide the countdown timer display
    #[arg(long)]
    pub hide_timer: bool,

    /// Custom exit keyboard shortcut (e.g., "Cmd+Shift+Q", "Ctrl+Option+Escape")
    /// Requires at least one modifier key (Cmd, Option, Shift, or Ctrl).
    /// CLI argument overrides config file setting.
    #[arg(short = 'e', long = "exit-key", value_parser = parse_exit_key)]
    pub exit_key: Option<ExitKey>,

    /// Enable verbose logging output (use multiple times for more detail: -v, -vv, -vvv)
    #[arg(short, long, action = clap::ArgAction::Count)]
    pub verbose: u8,

    /// Enable trace logging to file (~/.config/catshield/logs/)
    /// Logs detailed event traces for debugging intermittent issues.
    #[arg(long)]
    pub trace: bool,

    /// Overlay opacity percentage (10-90). Higher values are more opaque.
    /// CLI argument overrides config file setting.
    #[arg(short = 'o', long, value_parser = parse_opacity)]
    pub opacity: Option<f64>,

    /// Overlay color (preset name or hex code).
    /// Presets: gray, blue, green, red, purple
    /// Hex format: #RRGGBB or #RGB (e.g., "#1a2b3c" or "#abc")
    /// CLI argument overrides config file setting.
    #[arg(short = 'c', long, value_parser = parse_color)]
    pub color: Option<String>,

    /// Disable the animated cat companion on the overlay.
    /// By default, a cute animated cat is shown on the protection screen.
    #[arg(long)]
    pub no_cat: bool,
}

/// Parse exit key string into ExitKey struct (for clap value_parser)
fn parse_exit_key(s: &str) -> Result<ExitKey, String> {
    ExitKey::parse(s)
}

/// Parse opacity percentage (10-90) into a decimal value (0.1-0.9)
fn parse_opacity(s: &str) -> Result<f64, String> {
    let percent: f64 = s
        .parse()
        .map_err(|_| format!("Invalid opacity value: {s}"))?;

    // Convert percentage to decimal
    let opacity = percent / 100.0;

    if !(MIN_OVERLAY_OPACITY..=MAX_OVERLAY_OPACITY).contains(&opacity) {
        return Err(format!(
            "Opacity must be between {}% and {}%",
            (MIN_OVERLAY_OPACITY * 100.0) as i32,
            (MAX_OVERLAY_OPACITY * 100.0) as i32
        ));
    }

    Ok(opacity)
}

/// Parse and validate color argument (preset name or hex code)
fn parse_color(s: &str) -> Result<String, String> {
    use crate::config::Config;

    // Validate the color by attempting to parse it
    if Config::parse_color_to_rgb(s).is_some() {
        Ok(s.to_string())
    } else {
        Err(format!(
            "Invalid color '{}'. Use a preset (gray, blue, green, red, purple) or hex code (#RRGGBB)",
            s
        ))
    }
}

/// Check if the app was launched with arguments that should trigger immediate shield activation
pub fn has_immediate_start_args(args: &Args) -> bool {
    // If timer, exit-key, opacity, or color CLI args are provided, start shield immediately
    args.timer.is_some()
        || args.exit_key.is_some()
        || args.opacity.is_some()
        || args.color.is_some()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn default_args() -> Args {
        Args {
            timer: None,
            hide_timer: false,
            exit_key: None,
            verbose: 0,
            trace: false,
            opacity: None,
            color: None,
            no_cat: false,
        }
    }

    #[test]
    fn test_has_immediate_start_args_none() {
        let args = default_args();
        assert!(!has_immediate_start_args(&args));
    }

    #[test]
    fn test_has_immediate_start_args_with_timer() {
        let args = Args {
            timer: Some(60),
            ..default_args()
        };
        assert!(has_immediate_start_args(&args));
    }

    #[test]
    fn test_has_immediate_start_args_with_exit_key() {
        let args = Args {
            exit_key: Some(ExitKey::default()),
            ..default_args()
        };
        assert!(has_immediate_start_args(&args));
    }

    #[test]
    fn test_has_immediate_start_args_with_both() {
        let args = Args {
            timer: Some(120),
            hide_timer: true,
            exit_key: Some(ExitKey::default()),
            ..default_args()
        };
        assert!(has_immediate_start_args(&args));
    }

    #[test]
    fn test_has_immediate_start_args_hide_timer_alone_is_menu_mode() {
        // hide_timer alone should NOT trigger immediate mode
        let args = Args {
            hide_timer: true,
            ..default_args()
        };
        assert!(!has_immediate_start_args(&args));
    }

    #[test]
    fn test_has_immediate_start_args_with_opacity() {
        let args = Args {
            opacity: Some(0.7),
            ..default_args()
        };
        assert!(has_immediate_start_args(&args));
    }

    #[test]
    fn test_has_immediate_start_args_with_color() {
        let args = Args {
            color: Some("blue".to_string()),
            ..default_args()
        };
        assert!(has_immediate_start_args(&args));
    }

    #[test]
    fn test_parse_opacity_valid() {
        assert_eq!(parse_opacity("50").unwrap(), 0.5);
        assert_eq!(parse_opacity("10").unwrap(), 0.1);
        assert_eq!(parse_opacity("90").unwrap(), 0.9);
    }

    #[test]
    fn test_parse_opacity_invalid_range() {
        assert!(parse_opacity("5").is_err());
        assert!(parse_opacity("95").is_err());
        assert!(parse_opacity("0").is_err());
        assert!(parse_opacity("100").is_err());
    }

    #[test]
    fn test_parse_opacity_invalid_format() {
        assert!(parse_opacity("abc").is_err());
        assert!(parse_opacity("").is_err());
    }

    #[test]
    fn test_parse_color_presets() {
        assert!(parse_color("gray").is_ok());
        assert!(parse_color("blue").is_ok());
        assert!(parse_color("green").is_ok());
        assert!(parse_color("red").is_ok());
        assert!(parse_color("purple").is_ok());
    }

    #[test]
    fn test_parse_color_hex() {
        assert!(parse_color("#FF0000").is_ok());
        assert!(parse_color("#abc").is_ok());
        assert!(parse_color("#1a2b3c").is_ok());
    }

    #[test]
    fn test_parse_color_invalid() {
        assert!(parse_color("invalid").is_err());
        assert!(parse_color("#GG0000").is_err());
        assert!(parse_color("#12345").is_err());
    }
}
