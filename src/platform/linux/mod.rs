//! Linux-specific platform implementations for Cat Shield
//!
//! This module contains Linux-specific code:
//! - X11 keyboard grab for input blocking
//! - Wayland keyboard shortcuts inhibit for input capture
//! - D-Bus power management for sleep prevention
//! - System tray via StatusNotifierItem protocol
//! - Overlay window (X11 and Wayland with wlr-layer-shell)
//! - Platform trait implementations
//!
//! # Implementation Status
//!
//! - [x] InputBlocker via X11 keyboard grab (XGrabKeyboard)
//! - [x] InputBlocker via Wayland (keyboard-shortcuts-inhibit) - Issue #131
//! - [x] PowerManager via D-Bus (FreeDesktop ScreenSaver, GNOME SessionManager)
//! - [ ] PermissionChecker (placeholder) - future issue
//! - [x] SystemTray via ksni (StatusNotifierItem protocol)
//! - [x] OverlayWindow via X11 (_NET_WM_STATE_ABOVE, _NET_WM_STATE_FULLSCREEN)
//! - [x] OverlayWindow via Wayland (wlr-layer-shell protocol)
//!
//! # Overlay Window Support
//!
//! The overlay window implementation supports both X11 and Wayland:
//!
//! - **X11**: Full support using EWMH properties for topmost fullscreen behavior
//! - **Wayland (wlr-layer-shell)**: Supported on wlroots-based compositors (Sway, Wayfire, Hyprland)
//! - **GNOME Wayland**: Not supported (uses different layer-shell protocol)
//! - **XWayland**: Fallback to X11 implementation works on most Wayland compositors
//!
//! Use `is_native_wayland_overlay_available()` to check for native Wayland support.
//!
//! # Wayland Keyboard Input
//!
//! On Wayland compositors that support the `keyboard-shortcuts-inhibit` protocol:
//! - All keyboard input is captured when the overlay has focus
//! - Exit key detection works for configured key combinations
//! - Allowed keys passthrough is supported
//! - **Limitation**: User can click away to remove keyboard focus (Wayland security model)
//!
//! Use `is_keyboard_shortcuts_inhibit_available()` to check for protocol support.

mod overlay;
mod power;
mod tray;
mod wayland_keyboard;
mod wayland_overlay;
mod x11_keyboard;

pub use overlay::LinuxOverlayWindow;
pub use power::LinuxPowerManager;
pub use tray::{LinuxSystemTray, MenuCallback, TrayMenuAction};
pub use wayland_keyboard::{
    clear_wayland_allowed_keys, is_keyboard_shortcuts_inhibit_available, set_wayland_allowed_keys,
    set_wayland_exit_key_config, WaylandAllowedKeyConfig, WaylandKeyboardResult,
    WaylandModifierState,
};
pub use wayland_overlay::{is_wlr_layer_shell_available, WaylandOverlayWindow};
pub use x11_keyboard::{
    allow_keyboard_event, block_keyboard_event, clear_allowed_keys, set_allowed_keys,
    set_exit_key_config, AllowedKeyConfig, ProcessResult, X11InputBlocker,
};

/// Checks if native Wayland overlay is available (wlr-layer-shell protocol).
///
/// Returns `true` if the current Wayland compositor supports the wlr-layer-shell
/// protocol, which is required for native Wayland overlay windows.
///
/// # Compositor Support
///
/// - **Sway**: Supported
/// - **Wayfire**: Supported
/// - **Hyprland**: Supported
/// - **GNOME**: Not supported (uses gtk-layer-shell)
/// - **KDE Plasma**: May be supported (depends on version)
///
/// If this returns `false` on a Wayland session, fall back to XWayland.
pub fn is_native_wayland_overlay_available() -> bool {
    // First check if we're on Wayland at all
    if !is_wayland_session() {
        return false;
    }

    // Then check if wlr-layer-shell is available
    is_wlr_layer_shell_available()
}

/// Checks if the current session is running on Wayland.
pub fn is_wayland_session() -> bool {
    std::env::var("WAYLAND_DISPLAY").is_ok()
        || std::env::var("XDG_SESSION_TYPE")
            .map(|v| v.to_lowercase() == "wayland")
            .unwrap_or(false)
}

/// Checks if the current session is running on X11.
pub fn is_x11_session() -> bool {
    std::env::var("DISPLAY").is_ok()
}
