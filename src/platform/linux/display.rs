//! Display server detection for Linux
//!
//! This module provides comprehensive detection of the display server type
//! on Linux systems, supporting:
//! - X11 (pure X11 session)
//! - Wayland (native Wayland session)
//! - XWayland (running X11 app under Wayland)
//!
//! # Detection Logic
//!
//! The detection uses environment variables and protocol availability checks:
//! - `XDG_SESSION_TYPE`: Set by display managers (x11 or wayland)
//! - `WAYLAND_DISPLAY`: Present in Wayland sessions
//! - `DISPLAY`: Present in X11 and XWayland sessions
//!
//! # XWayland Detection
//!
//! XWayland is detected when:
//! 1. `WAYLAND_DISPLAY` is set (we're in a Wayland session), AND
//! 2. `DISPLAY` is also set (X11 compatibility is available), AND
//! 3. Either native Wayland protocols aren't available, OR we're explicitly running via X11

use super::wayland_keyboard::is_keyboard_shortcuts_inhibit_available;
use super::wayland_overlay::is_wlr_layer_shell_available;

/// Display server type detected on the system
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DisplayServer {
    /// Pure X11 session (no Wayland)
    X11,
    /// Native Wayland session with full protocol support
    WaylandNative,
    /// Wayland session but using XWayland for this application
    XWayland,
    /// Wayland session without full input blocking support
    WaylandLimited,
    /// Unknown or no display server detected
    Unknown,
}

impl std::fmt::Display for DisplayServer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DisplayServer::X11 => write!(f, "X11"),
            DisplayServer::WaylandNative => write!(f, "Wayland (native)"),
            DisplayServer::XWayland => write!(f, "XWayland"),
            DisplayServer::WaylandLimited => write!(f, "Wayland (limited)"),
            DisplayServer::Unknown => write!(f, "Unknown"),
        }
    }
}

/// Wayland feature support capabilities
#[derive(Debug, Clone, Copy, Default)]
pub struct WaylandCapabilities {
    /// Whether wlr-layer-shell protocol is available (for overlay windows)
    pub layer_shell: bool,
    /// Whether keyboard-shortcuts-inhibit protocol is available
    pub keyboard_shortcuts_inhibit: bool,
}

impl WaylandCapabilities {
    /// Check if we have full native Wayland support
    pub fn has_full_support(&self) -> bool {
        self.layer_shell && self.keyboard_shortcuts_inhibit
    }
}

/// Detailed display server detection result
#[derive(Debug, Clone)]
pub struct DisplayServerInfo {
    /// The detected display server type
    pub server: DisplayServer,
    /// The session type from XDG_SESSION_TYPE
    pub session_type: Option<String>,
    /// Whether WAYLAND_DISPLAY is set
    pub wayland_display: bool,
    /// Whether DISPLAY (X11) is set
    pub x11_display: bool,
    /// Wayland protocol capabilities (if applicable)
    pub wayland_caps: WaylandCapabilities,
    /// Recommended action for the user
    pub recommendation: DisplayRecommendation,
}

/// Recommendation for how to run Cat Shield
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DisplayRecommendation {
    /// Everything is good, proceed normally
    Proceed,
    /// Running on Wayland with limited input blocking
    WaylandLimited {
        /// Message explaining the limitation
        message: String,
    },
    /// XWayland fallback is available and recommended
    XWaylandFallback {
        /// Message explaining why XWayland is recommended
        message: String,
    },
    /// No supported display server found
    Unsupported {
        /// Message explaining why it's unsupported
        message: String,
    },
}

/// User preference for Wayland mode handling
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum WaylandMode {
    /// Auto-detect and show warning (default)
    #[default]
    Auto,
    /// Accept Wayland limitations without warning
    Accept,
    /// Force XWayland fallback
    ForceX11,
    /// Force native Wayland (error if not available)
    ForceNative,
}

impl std::str::FromStr for WaylandMode {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "auto" => Ok(WaylandMode::Auto),
            "accept" => Ok(WaylandMode::Accept),
            "x11" | "xwayland" | "force-x11" => Ok(WaylandMode::ForceX11),
            "native" | "wayland" | "force-native" => Ok(WaylandMode::ForceNative),
            _ => Err(format!(
                "Invalid wayland-mode '{s}'. Valid options: auto, accept, x11, native"
            )),
        }
    }
}

impl std::fmt::Display for WaylandMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WaylandMode::Auto => write!(f, "auto"),
            WaylandMode::Accept => write!(f, "accept"),
            WaylandMode::ForceX11 => write!(f, "x11"),
            WaylandMode::ForceNative => write!(f, "native"),
        }
    }
}

/// Detect the display server type and capabilities.
///
/// This performs a comprehensive check of the display environment including:
/// - Environment variables (XDG_SESSION_TYPE, WAYLAND_DISPLAY, DISPLAY)
/// - Wayland protocol availability (wlr-layer-shell, keyboard-shortcuts-inhibit)
///
/// # Returns
///
/// A `DisplayServerInfo` struct with complete detection results and recommendations.
pub fn detect_display_server() -> DisplayServerInfo {
    let session_type = std::env::var("XDG_SESSION_TYPE").ok();
    let wayland_display = std::env::var("WAYLAND_DISPLAY").is_ok();
    let x11_display = std::env::var("DISPLAY").is_ok();

    // Check Wayland capabilities if we appear to be in a Wayland session
    let wayland_caps = if wayland_display {
        WaylandCapabilities {
            layer_shell: is_wlr_layer_shell_available(),
            keyboard_shortcuts_inhibit: is_keyboard_shortcuts_inhibit_available(),
        }
    } else {
        WaylandCapabilities::default()
    };

    // Determine the display server type
    let (server, recommendation) = determine_server_and_recommendation(
        &session_type,
        wayland_display,
        x11_display,
        &wayland_caps,
    );

    DisplayServerInfo {
        server,
        session_type,
        wayland_display,
        x11_display,
        wayland_caps,
        recommendation,
    }
}

fn determine_server_and_recommendation(
    session_type: &Option<String>,
    wayland_display: bool,
    x11_display: bool,
    wayland_caps: &WaylandCapabilities,
) -> (DisplayServer, DisplayRecommendation) {
    // Pure X11 session
    if !wayland_display && x11_display {
        return (DisplayServer::X11, DisplayRecommendation::Proceed);
    }

    // No display server found
    if !wayland_display && !x11_display {
        return (
            DisplayServer::Unknown,
            DisplayRecommendation::Unsupported {
                message: "No display server detected. Ensure DISPLAY or WAYLAND_DISPLAY is set."
                    .to_string(),
            },
        );
    }

    // Wayland session detected
    if wayland_display {
        // Full native Wayland support available
        if wayland_caps.has_full_support() {
            return (
                DisplayServer::WaylandNative,
                DisplayRecommendation::WaylandLimited {
                    message: wayland_limitation_message(),
                },
            );
        }

        // Native Wayland not fully supported, but X11/XWayland available
        if x11_display {
            return (
                DisplayServer::XWayland,
                DisplayRecommendation::XWaylandFallback {
                    message: xwayland_recommendation_message(),
                },
            );
        }

        // Wayland without full support and no XWayland fallback
        return (
            DisplayServer::WaylandLimited,
            DisplayRecommendation::WaylandLimited {
                message: wayland_no_fallback_message(session_type.as_deref()),
            },
        );
    }

    // Fallback - shouldn't reach here
    (
        DisplayServer::Unknown,
        DisplayRecommendation::Unsupported {
            message: "Unable to determine display server configuration.".to_string(),
        },
    )
}

/// Print the Wayland warning message to stderr.
pub fn print_wayland_warning(info: &DisplayServerInfo) {
    match &info.recommendation {
        DisplayRecommendation::Proceed => {}
        DisplayRecommendation::WaylandLimited { message } => {
            eprintln!();
            eprintln!("⚠️  {message}");
            eprintln!();
        }
        DisplayRecommendation::XWaylandFallback { message } => {
            eprintln!();
            eprintln!("⚠️  {message}");
            eprintln!();
        }
        DisplayRecommendation::Unsupported { message } => {
            eprintln!();
            eprintln!("❌ {message}");
            eprintln!();
        }
    }
}

/// Check if we should proceed based on the display info and user preferences.
///
/// Returns `Ok(DisplayServer)` with the effective display server to use,
/// or `Err(String)` with an error message if we cannot proceed.
pub fn should_proceed(
    info: &DisplayServerInfo,
    wayland_mode: WaylandMode,
) -> Result<DisplayServer, String> {
    match wayland_mode {
        WaylandMode::Auto => {
            // In auto mode, always proceed but print warnings
            match &info.recommendation {
                DisplayRecommendation::Proceed => Ok(info.server),
                DisplayRecommendation::WaylandLimited { .. } => {
                    print_wayland_warning(info);
                    Ok(info.server)
                }
                DisplayRecommendation::XWaylandFallback { .. } => {
                    print_wayland_warning(info);
                    // Use XWayland fallback
                    Ok(DisplayServer::XWayland)
                }
                DisplayRecommendation::Unsupported { message } => Err(message.clone()),
            }
        }
        WaylandMode::Accept => {
            // Accept limitations silently
            match &info.recommendation {
                DisplayRecommendation::Unsupported { message } => Err(message.clone()),
                _ => Ok(info.server),
            }
        }
        WaylandMode::ForceX11 => {
            // Force X11/XWayland
            if info.x11_display {
                Ok(DisplayServer::XWayland)
            } else {
                Err("X11 display not available. Cannot use --wayland-mode=x11.".to_string())
            }
        }
        WaylandMode::ForceNative => {
            // Force native Wayland
            if info.wayland_display && info.wayland_caps.has_full_support() {
                Ok(DisplayServer::WaylandNative)
            } else if !info.wayland_display {
                Err(
                    "Not running in a Wayland session. Cannot use --wayland-mode=native."
                        .to_string(),
                )
            } else {
                Err(format!(
                    "Native Wayland protocols not available (layer_shell: {}, keyboard_shortcuts_inhibit: {}). \
                     Cannot use --wayland-mode=native.",
                    info.wayland_caps.layer_shell,
                    info.wayland_caps.keyboard_shortcuts_inhibit
                ))
            }
        }
    }
}

fn wayland_limitation_message() -> String {
    "Running on Wayland with limited input blocking capability.\n\n    \
     On Wayland, Cat Shield can capture keyboard input when focused,\n    \
     but pointer clicks can bypass the overlay (cats can click away!).\n\n    \
     For full input blocking, consider:\n    \
     • Run under XWayland: GDK_BACKEND=x11 catshield\n    \
     • Use an X11 session instead of Wayland\n    \
     • Accept limitations with: catshield --wayland-mode=accept\n\n    \
     See: https://github.com/taearls/catshield/blob/main/docs/WAYLAND_INPUT_RESEARCH.md"
        .to_string()
}

fn xwayland_recommendation_message() -> String {
    "Your Wayland compositor doesn't support full overlay protocols.\n\n    \
     Cat Shield will use XWayland fallback for better input blocking.\n    \
     To force native Wayland (with limitations): catshield --wayland-mode=native\n    \
     To silence this message: catshield --wayland-mode=accept"
        .to_string()
}

fn wayland_no_fallback_message(session_type: Option<&str>) -> String {
    let compositor_hint = match session_type {
        Some("wayland") => "Your compositor may be GNOME or another non-wlroots compositor.",
        _ => "The compositor type could not be determined.",
    };

    format!(
        "Running on Wayland without full protocol support or XWayland fallback.\n\n    \
         {compositor_hint}\n    \
         Cat Shield's input blocking will be limited:\n    \
         • Keyboard capture only works when the overlay is focused\n    \
         • Mouse clicks can bypass the overlay\n\n    \
         For full functionality, use an X11 session or wlroots-based compositor\n    \
         (Sway, Wayfire, Hyprland)."
    )
}

/// Get a summary string for logging purposes.
pub fn display_info_summary(info: &DisplayServerInfo) -> String {
    format!(
        "Display: {} (session_type={}, wayland={}, x11={}, layer_shell={}, kbd_inhibit={})",
        info.server,
        info.session_type.as_deref().unwrap_or("unset"),
        info.wayland_display,
        info.x11_display,
        info.wayland_caps.layer_shell,
        info.wayland_caps.keyboard_shortcuts_inhibit,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wayland_mode_from_str() {
        assert_eq!("auto".parse::<WaylandMode>().unwrap(), WaylandMode::Auto);
        assert_eq!(
            "accept".parse::<WaylandMode>().unwrap(),
            WaylandMode::Accept
        );
        assert_eq!("x11".parse::<WaylandMode>().unwrap(), WaylandMode::ForceX11);
        assert_eq!(
            "xwayland".parse::<WaylandMode>().unwrap(),
            WaylandMode::ForceX11
        );
        assert_eq!(
            "native".parse::<WaylandMode>().unwrap(),
            WaylandMode::ForceNative
        );
        assert!("invalid".parse::<WaylandMode>().is_err());
    }

    #[test]
    fn test_wayland_mode_display() {
        assert_eq!(WaylandMode::Auto.to_string(), "auto");
        assert_eq!(WaylandMode::Accept.to_string(), "accept");
        assert_eq!(WaylandMode::ForceX11.to_string(), "x11");
        assert_eq!(WaylandMode::ForceNative.to_string(), "native");
    }

    #[test]
    fn test_display_server_display() {
        assert_eq!(DisplayServer::X11.to_string(), "X11");
        assert_eq!(DisplayServer::WaylandNative.to_string(), "Wayland (native)");
        assert_eq!(DisplayServer::XWayland.to_string(), "XWayland");
        assert_eq!(
            DisplayServer::WaylandLimited.to_string(),
            "Wayland (limited)"
        );
        assert_eq!(DisplayServer::Unknown.to_string(), "Unknown");
    }

    #[test]
    fn test_wayland_capabilities_full_support() {
        let caps = WaylandCapabilities {
            layer_shell: true,
            keyboard_shortcuts_inhibit: true,
        };
        assert!(caps.has_full_support());

        let caps_partial = WaylandCapabilities {
            layer_shell: true,
            keyboard_shortcuts_inhibit: false,
        };
        assert!(!caps_partial.has_full_support());
    }

    #[test]
    fn test_determine_server_x11_only() {
        let (server, rec) = determine_server_and_recommendation(
            &Some("x11".to_string()),
            false,
            true,
            &WaylandCapabilities::default(),
        );
        assert_eq!(server, DisplayServer::X11);
        assert_eq!(rec, DisplayRecommendation::Proceed);
    }

    #[test]
    fn test_determine_server_no_display() {
        let (server, rec) = determine_server_and_recommendation(
            &None,
            false,
            false,
            &WaylandCapabilities::default(),
        );
        assert_eq!(server, DisplayServer::Unknown);
        assert!(matches!(rec, DisplayRecommendation::Unsupported { .. }));
    }

    #[test]
    fn test_determine_server_wayland_full_support() {
        let (server, rec) = determine_server_and_recommendation(
            &Some("wayland".to_string()),
            true,
            true,
            &WaylandCapabilities {
                layer_shell: true,
                keyboard_shortcuts_inhibit: true,
            },
        );
        assert_eq!(server, DisplayServer::WaylandNative);
        assert!(matches!(rec, DisplayRecommendation::WaylandLimited { .. }));
    }

    #[test]
    fn test_determine_server_wayland_xwayland_fallback() {
        let (server, rec) = determine_server_and_recommendation(
            &Some("wayland".to_string()),
            true,
            true,
            &WaylandCapabilities {
                layer_shell: false,
                keyboard_shortcuts_inhibit: false,
            },
        );
        assert_eq!(server, DisplayServer::XWayland);
        assert!(matches!(
            rec,
            DisplayRecommendation::XWaylandFallback { .. }
        ));
    }
}
