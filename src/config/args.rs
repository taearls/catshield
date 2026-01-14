//! CLI argument parsing for Cat Shield

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
}

/// Parse exit key string into ExitKey struct (for clap value_parser)
fn parse_exit_key(s: &str) -> Result<ExitKey, String> {
    ExitKey::parse(s)
}

/// Check if the app was launched with arguments that should trigger immediate shield activation
pub fn has_immediate_start_args(args: &Args) -> bool {
    // If timer or exit-key CLI args are provided, start shield immediately
    args.timer.is_some() || args.exit_key.is_some()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_has_immediate_start_args_none() {
        let args = Args {
            timer: None,
            hide_timer: false,
            exit_key: None,
            verbose: 0,
        };
        assert!(!has_immediate_start_args(&args));
    }

    #[test]
    fn test_has_immediate_start_args_with_timer() {
        let args = Args {
            timer: Some(60),
            hide_timer: false,
            exit_key: None,
            verbose: 0,
        };
        assert!(has_immediate_start_args(&args));
    }

    #[test]
    fn test_has_immediate_start_args_with_exit_key() {
        let args = Args {
            timer: None,
            hide_timer: false,
            exit_key: Some(ExitKey::default()),
            verbose: 0,
        };
        assert!(has_immediate_start_args(&args));
    }

    #[test]
    fn test_has_immediate_start_args_with_both() {
        let args = Args {
            timer: Some(120),
            hide_timer: true,
            exit_key: Some(ExitKey::default()),
            verbose: 0,
        };
        assert!(has_immediate_start_args(&args));
    }

    #[test]
    fn test_has_immediate_start_args_hide_timer_alone_is_menu_mode() {
        // hide_timer alone should NOT trigger immediate mode
        let args = Args {
            timer: None,
            hide_timer: true,
            exit_key: None,
            verbose: 0,
        };
        assert!(!has_immediate_start_args(&args));
    }
}
