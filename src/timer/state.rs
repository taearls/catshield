//! Auto-exit timer state management for Cat Shield

use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};

/// Global timer state for auto-exit feature
pub static AUTO_EXIT_ENABLED: AtomicBool = AtomicBool::new(false);
pub static AUTO_EXIT_START_TIME: AtomicU64 = AtomicU64::new(0);
pub static AUTO_EXIT_DURATION_SECS: AtomicU64 = AtomicU64::new(0);
pub static WARNING_SHOWN: AtomicBool = AtomicBool::new(false);

/// Initialize the auto-exit timer with the specified duration in seconds
pub fn init_auto_exit_timer(duration_secs: u64) {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    // Use Release ordering for initialization writes - readers will use Acquire
    AUTO_EXIT_START_TIME.store(now, Ordering::Release);
    AUTO_EXIT_DURATION_SECS.store(duration_secs, Ordering::Release);
    AUTO_EXIT_ENABLED.store(true, Ordering::Release);
}

/// Get the remaining seconds until auto-exit.
///
/// Returns:
/// - `u64::MAX` if the timer is disabled (AUTO_EXIT_ENABLED == false)
/// - `0` if the timer has expired (elapsed time >= duration)
/// - The remaining seconds otherwise
pub fn get_remaining_seconds() -> u64 {
    // Use Acquire ordering to synchronize with Release stores in init_auto_exit_timer
    if !AUTO_EXIT_ENABLED.load(Ordering::Acquire) {
        return u64::MAX;
    }

    let start = AUTO_EXIT_START_TIME.load(Ordering::Acquire);
    let duration = AUTO_EXIT_DURATION_SECS.load(Ordering::Acquire);
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();

    let elapsed = now.saturating_sub(start);
    duration.saturating_sub(elapsed)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Mutex, MutexGuard};
    use std::time::{SystemTime, UNIX_EPOCH};

    // ============================================================
    // Test synchronization to prevent parallel test interference
    // ============================================================

    /// Mutex to serialize tests that mutate global timer state.
    /// This prevents race conditions between parallel tests.
    static TEST_LOCK: Mutex<()> = Mutex::new(());

    /// Reset all timer state globals to their initial values.
    /// Must only be called while holding the TEST_LOCK.
    fn reset_timer_state() {
        AUTO_EXIT_ENABLED.store(false, Ordering::Release);
        AUTO_EXIT_DURATION_SECS.store(0, Ordering::Release);
        AUTO_EXIT_START_TIME.store(0, Ordering::Release);
        WARNING_SHOWN.store(false, Ordering::Release);
    }

    /// Acquire the test lock and reset all timer state globals.
    /// Returns a MutexGuard that must be held for the duration of the test.
    fn lock_and_reset_timer_state() -> MutexGuard<'static, ()> {
        let guard = TEST_LOCK.lock().expect("TEST_LOCK poisoned");
        reset_timer_state();
        guard
    }

    // ============================================================
    // Tests for init_auto_exit_timer()
    // ============================================================

    #[test]
    fn test_init_auto_exit_timer_sets_enabled() {
        let _guard = lock_and_reset_timer_state();
        assert!(!AUTO_EXIT_ENABLED.load(Ordering::Acquire));

        init_auto_exit_timer(3600);

        assert!(
            AUTO_EXIT_ENABLED.load(Ordering::Acquire),
            "AUTO_EXIT_ENABLED should be true after init"
        );
    }

    #[test]
    fn test_init_auto_exit_timer_sets_duration() {
        let _guard = lock_and_reset_timer_state();
        assert_eq!(AUTO_EXIT_DURATION_SECS.load(Ordering::Acquire), 0);

        init_auto_exit_timer(3600);

        assert_eq!(
            AUTO_EXIT_DURATION_SECS.load(Ordering::Acquire),
            3600,
            "Duration should be 3600 seconds"
        );
    }

    #[test]
    fn test_init_auto_exit_timer_sets_start_time() {
        let _guard = lock_and_reset_timer_state();
        let before = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        init_auto_exit_timer(3600);

        let start = AUTO_EXIT_START_TIME.load(Ordering::Acquire);
        let after = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        assert!(
            start >= before && start <= after,
            "Start time {start} should be between {before} and {after}"
        );
    }

    #[test]
    fn test_init_auto_exit_timer_with_zero_duration() {
        let _guard = lock_and_reset_timer_state();

        init_auto_exit_timer(0);

        assert!(AUTO_EXIT_ENABLED.load(Ordering::Acquire));
        assert_eq!(AUTO_EXIT_DURATION_SECS.load(Ordering::Acquire), 0);
    }

    #[test]
    fn test_init_auto_exit_timer_with_max_duration() {
        let _guard = lock_and_reset_timer_state();

        init_auto_exit_timer(u64::MAX);

        assert!(AUTO_EXIT_ENABLED.load(Ordering::Acquire));
        assert_eq!(AUTO_EXIT_DURATION_SECS.load(Ordering::Acquire), u64::MAX);
    }

    #[test]
    fn test_init_auto_exit_timer_overwrites_previous() {
        let _guard = lock_and_reset_timer_state();

        // First initialization
        init_auto_exit_timer(100);
        assert_eq!(AUTO_EXIT_DURATION_SECS.load(Ordering::Acquire), 100);

        // Second initialization should overwrite
        init_auto_exit_timer(200);
        assert_eq!(
            AUTO_EXIT_DURATION_SECS.load(Ordering::Acquire),
            200,
            "Duration should be overwritten to 200"
        );
    }

    // ============================================================
    // Tests for get_remaining_seconds()
    // ============================================================

    #[test]
    fn test_get_remaining_seconds_disabled_returns_max() {
        let _guard = lock_and_reset_timer_state();
        AUTO_EXIT_ENABLED.store(false, Ordering::Release);

        let remaining = get_remaining_seconds();

        assert_eq!(
            remaining,
            u64::MAX,
            "Should return u64::MAX when timer is disabled"
        );
    }

    #[test]
    fn test_get_remaining_seconds_just_started() {
        let _guard = lock_and_reset_timer_state();

        init_auto_exit_timer(60);
        let remaining = get_remaining_seconds();

        // Allow extra slack for loaded CI / scheduling delays
        assert!(
            (55..=60).contains(&remaining),
            "Remaining {remaining} should be between 55 and 60"
        );
    }

    #[test]
    fn test_get_remaining_seconds_expired() {
        let _guard = lock_and_reset_timer_state();

        // Manually set up an expired timer
        AUTO_EXIT_ENABLED.store(true, Ordering::Release);
        AUTO_EXIT_DURATION_SECS.store(10, Ordering::Release);
        // Set start time far in the past (Unix epoch + 1 second)
        AUTO_EXIT_START_TIME.store(1, Ordering::Release);

        let remaining = get_remaining_seconds();

        assert_eq!(remaining, 0, "Expired timer should return 0");
    }

    #[test]
    fn test_get_remaining_seconds_halfway_through() {
        let _guard = lock_and_reset_timer_state();

        // Set up a timer that started 30 seconds ago with 60 second duration
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        AUTO_EXIT_ENABLED.store(true, Ordering::Release);
        AUTO_EXIT_DURATION_SECS.store(60, Ordering::Release);
        AUTO_EXIT_START_TIME.store(now - 30, Ordering::Release);

        let remaining = get_remaining_seconds();

        // Should be around 30 seconds, allow some tolerance
        assert!(
            (28..=32).contains(&remaining),
            "Remaining {remaining} should be around 30"
        );
    }

    #[test]
    fn test_get_remaining_seconds_near_end() {
        let _guard = lock_and_reset_timer_state();

        // Set up a timer that's almost expired (started 59 seconds ago, 60 second duration)
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        AUTO_EXIT_ENABLED.store(true, Ordering::Release);
        AUTO_EXIT_DURATION_SECS.store(60, Ordering::Release);
        AUTO_EXIT_START_TIME.store(now - 59, Ordering::Release);

        let remaining = get_remaining_seconds();

        // Should be around 1 second
        assert!(
            remaining <= 3,
            "Remaining {remaining} should be 3 or less (near end)"
        );
    }

    #[test]
    fn test_get_remaining_seconds_with_zero_duration() {
        let _guard = lock_and_reset_timer_state();

        init_auto_exit_timer(0);
        let remaining = get_remaining_seconds();

        assert_eq!(remaining, 0, "Zero duration timer should return 0");
    }

    // ============================================================
    // Tests for WARNING_SHOWN state
    // ============================================================

    #[test]
    fn test_warning_shown_initially_false() {
        let _guard = lock_and_reset_timer_state();

        init_auto_exit_timer(3600);

        assert!(
            !WARNING_SHOWN.load(Ordering::Acquire),
            "WARNING_SHOWN should be false after init"
        );
    }

    #[test]
    fn test_warning_shown_can_be_set() {
        let _guard = lock_and_reset_timer_state();

        WARNING_SHOWN.store(true, Ordering::Release);

        assert!(
            WARNING_SHOWN.load(Ordering::Acquire),
            "WARNING_SHOWN should be settable to true"
        );
    }

    #[test]
    fn test_warning_shown_reset_clears_value() {
        let _guard = lock_and_reset_timer_state();

        WARNING_SHOWN.store(true, Ordering::Release);
        assert!(WARNING_SHOWN.load(Ordering::Acquire));

        // Call reset again (we already hold the lock)
        reset_timer_state();

        assert!(
            !WARNING_SHOWN.load(Ordering::Acquire),
            "WARNING_SHOWN should be false after reset"
        );
    }

    // ============================================================
    // Tests for edge cases and boundary conditions
    // ============================================================

    #[test]
    fn test_elapsed_time_overflow_protection() {
        let _guard = lock_and_reset_timer_state();

        // Set start time to a future time (should result in 0 elapsed)
        let future_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
            + 3600;

        AUTO_EXIT_ENABLED.store(true, Ordering::Release);
        AUTO_EXIT_DURATION_SECS.store(60, Ordering::Release);
        AUTO_EXIT_START_TIME.store(future_time, Ordering::Release);

        let remaining = get_remaining_seconds();

        // saturating_sub should prevent underflow
        // When start is in future, now - start = 0 (saturating)
        // So remaining = duration - 0 = duration
        assert_eq!(
            remaining, 60,
            "Should return full duration when start time is in future"
        );
    }

    #[test]
    fn test_duration_saturating_sub_protection() {
        let _guard = lock_and_reset_timer_state();

        // Set up a timer where elapsed > duration
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        AUTO_EXIT_ENABLED.store(true, Ordering::Release);
        AUTO_EXIT_DURATION_SECS.store(10, Ordering::Release);
        // Started 100 seconds ago, but duration is only 10
        AUTO_EXIT_START_TIME.store(now - 100, Ordering::Release);

        let remaining = get_remaining_seconds();

        assert_eq!(
            remaining, 0,
            "Should return 0 when elapsed exceeds duration (saturating_sub)"
        );
    }

    #[test]
    fn test_timer_state_independence() {
        let _guard = lock_and_reset_timer_state();

        // Modify each atomic independently
        AUTO_EXIT_ENABLED.store(true, Ordering::Release);
        assert!(AUTO_EXIT_ENABLED.load(Ordering::Acquire));
        assert_eq!(AUTO_EXIT_DURATION_SECS.load(Ordering::Acquire), 0);
        assert_eq!(AUTO_EXIT_START_TIME.load(Ordering::Acquire), 0);
        assert!(!WARNING_SHOWN.load(Ordering::Acquire));

        AUTO_EXIT_DURATION_SECS.store(3600, Ordering::Release);
        assert!(AUTO_EXIT_ENABLED.load(Ordering::Acquire));
        assert_eq!(AUTO_EXIT_DURATION_SECS.load(Ordering::Acquire), 3600);
        assert_eq!(AUTO_EXIT_START_TIME.load(Ordering::Acquire), 0);
        assert!(!WARNING_SHOWN.load(Ordering::Acquire));
    }

    #[test]
    fn test_long_duration_timer() {
        let _guard = lock_and_reset_timer_state();

        // Test with 24 hours (max allowed by the app)
        let seconds_in_24_hours = 24 * 60 * 60;
        init_auto_exit_timer(seconds_in_24_hours);

        let remaining = get_remaining_seconds();

        // Allow extra slack for loaded CI / scheduling delays
        assert!(
            remaining >= seconds_in_24_hours - 5 && remaining <= seconds_in_24_hours,
            "Remaining {remaining} should be close to {seconds_in_24_hours}"
        );
    }

    #[test]
    fn test_one_second_timer() {
        let _guard = lock_and_reset_timer_state();

        init_auto_exit_timer(1);

        let remaining = get_remaining_seconds();

        // Should be 0 or 1
        assert!(
            remaining <= 1,
            "1-second timer remaining {remaining} should be 0 or 1"
        );
    }

    // ============================================================
    // Tests for global state module exports
    // ============================================================

    #[test]
    fn test_static_atomics_are_accessible() {
        // Verify that all static atomics can be accessed
        let _ = AUTO_EXIT_ENABLED.load(Ordering::Acquire);
        let _ = AUTO_EXIT_DURATION_SECS.load(Ordering::Acquire);
        let _ = AUTO_EXIT_START_TIME.load(Ordering::Acquire);
        let _ = WARNING_SHOWN.load(Ordering::Acquire);
    }

    #[test]
    fn test_functions_are_callable() {
        let _guard = lock_and_reset_timer_state();

        // Just verify the functions can be called without panicking
        init_auto_exit_timer(60);
        let _ = get_remaining_seconds();
    }
}
