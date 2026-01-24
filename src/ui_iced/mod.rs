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
//! - [`settings`]: Settings window UI (future)
//! - [`theme`]: Custom theming and styling
//!
//! # Integration
//!
//! The iced UI runs alongside the existing platform-specific input blocking:
//! - macOS: CGEventTap remains unchanged
//! - Windows: Low-level keyboard hook remains unchanged
//! - Linux: X11 grab / Wayland inhibitor remains unchanged
//!
//! iced handles only the visual rendering; input blocking is external.

pub mod overlay;
pub mod settings;
pub mod theme;

// Re-export main types for convenience
pub use overlay::{ExitKeyConfig, OverlayApp, OverlayMessage};
pub use theme::CatShieldTheme;
