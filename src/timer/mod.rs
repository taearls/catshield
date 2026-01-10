//! Timer module for Cat Shield
//!
//! This module handles:
//! - Duration parsing from user input (e.g., "30m", "2h", "1h30m")
//! - Duration formatting for display
//! - Auto-exit timer state management

mod formatting;
#[cfg(test)]
mod integration_tests;
mod parsing;
mod state;

pub use formatting::format_duration;
pub use parsing::{parse_duration, parse_timer_value_and_unit};
pub use state::{
    get_remaining_seconds, init_auto_exit_timer, AUTO_EXIT_DURATION_SECS, AUTO_EXIT_ENABLED,
    AUTO_EXIT_START_TIME, WARNING_SHOWN,
};

/// Timer configuration constants
pub const MIN_TIMER_SECONDS: u64 = 5; // Minimum 5 seconds
pub const MAX_TIMER_SECONDS: u64 = 24 * 60 * 60; // Maximum 24 hours
pub const WARNING_SECONDS: u64 = 60; // Show warning 1 minute before exit
