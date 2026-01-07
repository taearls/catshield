//! UI module for Cat Shield
//!
//! This module contains all UI-related functionality:
//! - Global UI state management
//! - Custom NSView classes (close button, timer display)
//! - Window creation (shield overlay, settings)
//! - Menu bar setup and handlers

pub mod helpers;
pub mod menu_bar;
pub mod shield;
pub mod state;
pub mod views;
pub mod windows;

pub use helpers::create_label;
pub use shield::{activate_shield, deactivate_shield};
pub use state::*;
