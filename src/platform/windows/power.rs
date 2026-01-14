//! Windows power management for Cat Shield
//!
//! Provides functions to prevent and allow system sleep using the Windows
//! `SetThreadExecutionState` API.
//!
//! # How It Works
//!
//! Windows uses execution state flags to determine when the system can sleep:
//! - `ES_CONTINUOUS`: Persist the state until explicitly cleared
//! - `ES_DISPLAY_REQUIRED`: Keep the display on
//! - `ES_SYSTEM_REQUIRED`: Keep the system awake
//!
//! To prevent sleep: `SetThreadExecutionState(ES_CONTINUOUS | ES_DISPLAY_REQUIRED)`
//! To allow sleep: `SetThreadExecutionState(ES_CONTINUOUS)`
//!
//! # Thread Safety
//!
//! The execution state is per-thread in Windows. For simplicity, this implementation
//! uses a global atomic counter to track assertion state across threads.

use crate::platform::errors::PowerError;
use crate::platform::traits::PowerManager;
use crate::platform::types::SleepAssertion;
use std::sync::atomic::{AtomicU64, Ordering};

use windows::Win32::System::Power::{
    SetThreadExecutionState, ES_CONTINUOUS, ES_DISPLAY_REQUIRED, EXECUTION_STATE,
};

/// Counter for generating unique assertion IDs
static ASSERTION_COUNTER: AtomicU64 = AtomicU64::new(1);

/// Track the number of active assertions
static ACTIVE_ASSERTIONS: AtomicU64 = AtomicU64::new(0);

/// Windows implementation of the `PowerManager` trait.
///
/// Uses `SetThreadExecutionState` with `ES_CONTINUOUS | ES_DISPLAY_REQUIRED`
/// to prevent the system from sleeping and keep the display active.
#[derive(Debug, Default)]
pub struct WindowsPowerManager;

impl WindowsPowerManager {
    /// Creates a new `WindowsPowerManager`.
    #[must_use]
    pub fn new() -> Self {
        Self
    }
}

impl PowerManager for WindowsPowerManager {
    fn prevent_sleep(&self) -> Result<SleepAssertion, PowerError> {
        // SAFETY: SetThreadExecutionState is safe to call with valid execution state flags.
        // It returns the previous execution state on success, or 0 on failure.
        let result = unsafe { SetThreadExecutionState(ES_CONTINUOUS | ES_DISPLAY_REQUIRED) };

        if result == EXECUTION_STATE(0) {
            return Err(PowerError::AssertionFailed(
                "SetThreadExecutionState failed".to_string(),
            ));
        }

        // Generate a unique assertion ID and track it
        let id = ASSERTION_COUNTER.fetch_add(1, Ordering::Relaxed);
        ACTIVE_ASSERTIONS.fetch_add(1, Ordering::Release);

        println!("  ✓ Sleep prevention enabled (Windows)");

        Ok(SleepAssertion::new(id))
    }

    fn allow_sleep(&self, _assertion: SleepAssertion) -> Result<(), PowerError> {
        // Decrement active assertions
        let previous = ACTIVE_ASSERTIONS.fetch_sub(1, Ordering::AcqRel);

        // Only clear the execution state if this was the last assertion
        if previous == 1 {
            // SAFETY: SetThreadExecutionState is safe to call.
            // ES_CONTINUOUS alone clears all other flags, allowing sleep again.
            let result = unsafe { SetThreadExecutionState(ES_CONTINUOUS) };

            if result == EXECUTION_STATE(0) {
                // Restore the counter since we failed
                ACTIVE_ASSERTIONS.fetch_add(1, Ordering::Release);
                return Err(PowerError::ReleaseFailed(
                    "SetThreadExecutionState failed to clear execution state".to_string(),
                ));
            }

            println!("  ✓ Sleep prevention disabled (Windows)");
        }

        Ok(())
    }
}

// SAFETY: WindowsPowerManager is Send + Sync because:
// - It has no mutable state of its own
// - All shared state (ASSERTION_COUNTER, ACTIVE_ASSERTIONS) uses atomic operations
// - SetThreadExecutionState is thread-safe (it modifies per-thread state)
unsafe impl Send for WindowsPowerManager {}
unsafe impl Sync for WindowsPowerManager {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_windows_power_manager_new() {
        let manager = WindowsPowerManager::new();
        // Just verify it can be created
        drop(manager);
    }

    #[test]
    fn test_windows_power_manager_default() {
        let manager = WindowsPowerManager::default();
        // Just verify default works
        drop(manager);
    }

    #[test]
    fn test_power_manager_is_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<WindowsPowerManager>();
    }

    #[test]
    fn test_assertion_counter_increments() {
        let start = ASSERTION_COUNTER.load(Ordering::Relaxed);
        ASSERTION_COUNTER.fetch_add(1, Ordering::Relaxed);
        let end = ASSERTION_COUNTER.load(Ordering::Relaxed);
        assert!(end > start);
    }
}
