//! CLI argument parsing for Cat Shield on Linux
//!
//! This module handles command-line arguments specific to Linux,
//! including the `--wayland-mode` flag for controlling display server behavior.

use crate::platform::WaylandMode;
use crate::timer::parse_duration;
use clap::Parser;

/// CLI arguments for Cat Shield on Linux
#[derive(Parser, Debug)]
#[command(name = "catshield")]
#[command(author = "Tyler Earls")]
#[command(version)]
#[command(about = "A cat-proof screen overlay that keeps your machine awake and blocks input")]
#[command(after_help = "EXAMPLES:
    catshield                             # Start with default settings
    catshield --timer 30m                 # Auto-exit after 30 minutes
    catshield --wayland-mode=accept       # Accept Wayland limitations silently
    catshield --wayland-mode=x11          # Force XWayland for full input blocking

WAYLAND MODE OPTIONS:
    auto     - Auto-detect and show warnings about limitations (default)
    accept   - Accept Wayland limitations without showing warnings
    x11      - Force XWayland fallback for full input blocking
    native   - Force native Wayland (error if protocols unavailable)

WAYLAND LIMITATIONS:
    On Wayland, Cat Shield can capture keyboard input when focused,
    but pointer clicks can bypass the overlay. For full input blocking:
    - Run under XWayland: GDK_BACKEND=x11 catshield
    - Use an X11 session instead of Wayland
    - Use a wlroots-based compositor (Sway, Wayfire, Hyprland)

CONFIG FILE:
    Settings can be persisted in ~/.config/catshield/config.toml:

    default_timer = \"30m\"
    wayland_mode = \"accept\"

For more information about Wayland limitations:
    https://github.com/taearls/catshield/blob/main/docs/WAYLAND_INPUT_RESEARCH.md")]
pub struct LinuxArgs {
    /// Auto-exit after specified duration (e.g., 30m, 2h, 1h30m)
    #[arg(short, long, value_parser = parse_duration)]
    pub timer: Option<u64>,

    /// Hide the countdown timer display
    #[arg(long)]
    pub hide_timer: bool,

    /// How to handle Wayland sessions (auto, accept, x11, native)
    ///
    /// auto: Auto-detect and show warnings (default)
    /// accept: Accept limitations without warnings
    /// x11: Force XWayland fallback
    /// native: Force native Wayland (error if unavailable)
    #[arg(long, value_parser = parse_wayland_mode, default_value = "auto")]
    pub wayland_mode: WaylandMode,

    /// Enable verbose logging output (use multiple times for more detail: -v, -vv, -vvv)
    #[arg(short, long, action = clap::ArgAction::Count)]
    pub verbose: u8,
}

/// Parse wayland mode string into WaylandMode enum (for clap value_parser)
fn parse_wayland_mode(s: &str) -> Result<WaylandMode, String> {
    s.parse()
}

/// Check if the app was launched with arguments that should trigger immediate shield activation
pub fn has_immediate_start_args_linux(args: &LinuxArgs) -> bool {
    // If timer CLI arg is provided, start shield immediately
    // wayland_mode alone doesn't trigger immediate start
    args.timer.is_some()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_has_immediate_start_args_none() {
        let args = LinuxArgs {
            timer: None,
            hide_timer: false,
            wayland_mode: WaylandMode::Auto,
            verbose: 0,
        };
        assert!(!has_immediate_start_args_linux(&args));
    }

    #[test]
    fn test_has_immediate_start_args_with_timer() {
        let args = LinuxArgs {
            timer: Some(60),
            hide_timer: false,
            wayland_mode: WaylandMode::Auto,
            verbose: 0,
        };
        assert!(has_immediate_start_args_linux(&args));
    }

    #[test]
    fn test_has_immediate_start_args_wayland_mode_alone() {
        let args = LinuxArgs {
            timer: None,
            hide_timer: false,
            wayland_mode: WaylandMode::Accept,
            verbose: 0,
        };
        // wayland_mode alone should NOT trigger immediate mode
        assert!(!has_immediate_start_args_linux(&args));
    }

    #[test]
    fn test_parse_wayland_mode_valid() {
        assert!(parse_wayland_mode("auto").is_ok());
        assert!(parse_wayland_mode("accept").is_ok());
        assert!(parse_wayland_mode("x11").is_ok());
        assert!(parse_wayland_mode("native").is_ok());
    }

    #[test]
    fn test_parse_wayland_mode_invalid() {
        assert!(parse_wayland_mode("invalid").is_err());
    }
}
