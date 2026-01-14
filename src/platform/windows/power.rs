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
use std::collections::HashSet;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Mutex;

use windows::Win32::System::Power::{
    SetThreadExecutionState, ES_CONTINUOUS, ES_DISPLAY_REQUIRED, EXECUTION_STATE,
};

/// Counter for generating unique assertion IDs
static ASSERTION_COUNTER: AtomicU64 = AtomicU64::new(1);

/// Track the number of active assertions
static ACTIVE_ASSERTIONS: AtomicU64 = AtomicU64::new(0);

/// Track valid assertion IDs to validate they were created by this manager
static VALID_ASSERTIONS: Mutex<Option<HashSet<u64>>> = Mutex::new(None);

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

        // Track this assertion ID as valid
        if let Ok(mut guard) = VALID_ASSERTIONS.lock() {
            let set = guard.get_or_insert_with(HashSet::new);
            set.insert(id);
        }

        log::info!("✓ Sleep prevention enabled (Windows)");

        Ok(SleepAssertion::new(id))
    }

    fn allow_sleep(&self, assertion: SleepAssertion) -> Result<(), PowerError> {
        let assertion_id = assertion.id();

        // Validate that this assertion was created by this manager
        {
            let mut guard = VALID_ASSERTIONS.lock().map_err(|_| {
                PowerError::InvalidAssertion("Failed to acquire assertion lock".to_string())
            })?;

            if let Some(set) = guard.as_mut() {
                if !set.remove(&assertion_id) {
                    return Err(PowerError::InvalidAssertion(format!(
                        "Assertion ID {} was not created by this manager or already released",
                        assertion_id
                    )));
                }
            } else {
                return Err(PowerError::InvalidAssertion(
                    "No assertions have been created".to_string(),
                ));
            }
        }

        // Decrement active assertions with underflow protection
        let previous = ACTIVE_ASSERTIONS.load(Ordering::Acquire);
        if previous == 0 {
            return Err(PowerError::InvalidAssertion(
                "No active assertions to release".to_string(),
            ));
        }
        ACTIVE_ASSERTIONS.fetch_sub(1, Ordering::AcqRel);

        // Only clear the execution state if this was the last assertion
        if previous == 1 {
            // SAFETY: SetThreadExecutionState is safe to call.
            // ES_CONTINUOUS alone clears all other flags, allowing sleep again.
            let result = unsafe { SetThreadExecutionState(ES_CONTINUOUS) };

            if result == EXECUTION_STATE(0) {
                // Restore the counter since we failed
                ACTIVE_ASSERTIONS.fetch_add(1, Ordering::Release);
                // Re-add the assertion ID since we failed
                if let Ok(mut guard) = VALID_ASSERTIONS.lock() {
                    if let Some(set) = guard.as_mut() {
                        set.insert(assertion_id);
                    }
                }
                return Err(PowerError::ReleaseFailed(
                    "SetThreadExecutionState failed to clear execution state".to_string(),
                ));
            }

            log::info!("✓ Sleep prevention disabled (Windows)");
        }

        Ok(())
    }
}

// SAFETY: WindowsPowerManager is Send + Sync because:
// - It has no mutable state of its own
// - All shared state (ASSERTION_COUNTER, ACTIVE_ASSERTIONS) uses atomic operations
// - VALID_ASSERTIONS is protected by a Mutex for thread-safe access
// - SetThreadExecutionState is thread-safe (it modifies per-thread state)
unsafe impl Send for WindowsPowerManager {}
unsafe impl Sync for WindowsPowerManager {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_windows_power_manager_new() {
        let _manager = WindowsPowerManager::new();
        // Just verify it can be created
    }

    #[test]
    fn test_windows_power_manager_default() {
        let _manager: WindowsPowerManager = Default::default();
        // Just verify default works
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

    #[test]
    fn test_allow_sleep_rejects_invalid_assertion_id() {
        let manager = WindowsPowerManager::new();
        // Create an assertion with an ID that was never created by prevent_sleep
        let fake_assertion = SleepAssertion::new(999_999_999);
        let result = manager.allow_sleep(fake_assertion);
        assert!(result.is_err());
        // Should return InvalidAssertion error (either "not created" or "No assertions")
        assert!(
            matches!(result, Err(PowerError::InvalidAssertion(_))),
            "Expected InvalidAssertion error, got: {:?}",
            result
        );
    }

    #[test]
    fn test_valid_assertions_tracking() {
        // Verify that the VALID_ASSERTIONS set is properly initialized
        if let Ok(mut guard) = VALID_ASSERTIONS.lock() {
            let set = guard.get_or_insert_with(HashSet::new);
            let test_id = 123_456_789u64;
            set.insert(test_id);
            assert!(set.contains(&test_id));
            set.remove(&test_id);
            assert!(!set.contains(&test_id));
        }
    }
}
