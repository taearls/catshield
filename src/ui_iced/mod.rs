//! iced-based UI module for Cat Shield
//!
//! This module provides the new cross-platform UI implementation using the iced framework.
//! It replaces the platform-specific overlay implementations with a unified, declarative UI.
//!
//! # Architecture
//!
//! The iced UI follows the Elm architecture with three main components:
//! - **State**: Application state held in the main struct
//! - **Message**: Events that can modify state
//! - **View**: Pure function that renders state to UI elements
//!
//! # Modules
//!
//! - [`overlay`]: Shield overlay window (fullscreen, semi-transparent)
//! - [`settings`]: Settings window UI for configuring preferences
//! - [`about`]: About window showing version and credits
//! - [`theme`]: Custom theming and styling
//! - [`integration`]: Platform integration for macOS menu bar (macOS only)
//!
//! # Integration
//!
//! The iced UI runs alongside the existing platform-specific input blocking:
//! - macOS: CGEventTap remains unchanged
//! - Windows: Low-level keyboard hook remains unchanged
//! - Linux: X11 grab / Wayland inhibitor remains unchanged
//!
//! iced handles only the visual rendering; input blocking is external.
//!
//! ## macOS Menu Bar Integration
//!
//! On macOS, the NSStatusItem (menu bar icon) remains in AppKit for native integration.
//! When "Preferences..." or "About..." is clicked, the [`integration`] module spawns
//! the corresponding iced window in a separate thread.

pub mod about;
#[cfg(target_os = "macos")]
pub mod integration;
pub mod overlay;
pub mod settings;
pub mod theme;

// Re-export main types for convenience
pub use about::{AboutMessage, AboutWindow};
#[cfg(target_os = "macos")]
pub use integration::{
    close_all_windows, is_about_window_open, is_settings_window_open, open_about_window,
    open_settings_window,
};
pub use overlay::{ExitKeyConfig, OverlayApp, OverlayMessage, TimerConfig};
pub use settings::{OverlayColor, SettingsMessage, SettingsSection, SettingsWindow, TimerPreset};
pub use theme::CatShieldTheme;
