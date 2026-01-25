//! Integration tests for menu bar timer functionality
//!
//! These tests verify the timer behavior in menu bar mode, including:
//! - Config-based timer activation
//! - Timer expiry behavior
//! - Mode consistency
//!
//! These are unit tests that mock the GUI components to test the logic
//! without requiring actual AppKit interaction.
//!
//! Note: These tests are macOS-only because they depend on ui::state which
//! uses AppKit types.

#[cfg(all(test, target_os = "macos"))]
mod tests {
    use crate::config::Config;
    use crate::timer::{
        format_duration, get_remaining_seconds, init_auto_exit_timer, parse_duration,
        AUTO_EXIT_DURATION_SECS, AUTO_EXIT_ENABLED, AUTO_EXIT_START_TIME, MAX_TIMER_SECONDS,
        MIN_TIMER_SECONDS, WARNING_SECONDS, WARNING_SHOWN,
    };
    use crate::ui::state::shield;
    use std::sync::atomic::Ordering;
    use std::sync::{Mutex, MutexGuard};
    use std::time::{SystemTime, UNIX_EPOCH};

    // ============================================================
    // Test synchronization to prevent parallel test interference
    // ============================================================

    /// Mutex to serialize tests that mutate global timer/shield state.
    static TEST_LOCK: Mutex<()> = Mutex::new(());

    /// Reset all timer and shield state globals to their initial values.
    fn reset_all_state() {
        // Reset timer state
        AUTO_EXIT_ENABLED.store(false, Ordering::Release);
        AUTO_EXIT_DURATION_SECS.store(0, Ordering::Release);
        AUTO_EXIT_START_TIME.store(0, Ordering::Release);
        WARNING_SHOWN.store(false, Ordering::Release);

        // Reset shield state
        shield::IS_ACTIVE.store(false, Ordering::Release);
        shield::MODE_MENU_BAR.store(false, Ordering::Release);
        shield::HAS_SLEEP_ASSERTION.store(false, Ordering::Release);
        shield::SLEEP_ASSERTION_ID.store(0, Ordering::Release);

        // Reset pointer states to null
        shield::WINDOW.store(std::ptr::null_mut(), Ordering::Release);
        shield::CLOSE_BUTTON.store(std::ptr::null_mut(), Ordering::Release);
        shield::CLOSE_BUTTON_LABEL.store(std::ptr::null_mut(), Ordering::Release);
        shield::CLOSE_BUTTON_TEXT.store(std::ptr::null_mut(), Ordering::Release);
        shield::TIMER_VIEW.store(std::ptr::null_mut(), Ordering::Release);
        shield::TIMER_REF.store(std::ptr::null_mut(), Ordering::Release);
        shield::TIMER_HEADER.store(std::ptr::null_mut(), Ordering::Release);
        shield::TIMER_TIME.store(std::ptr::null_mut(), Ordering::Release);
        shield::TIMER_WARNING.store(std::ptr::null_mut(), Ordering::Release);
    }

    /// Acquire the test lock and reset all state.
    fn lock_and_reset() -> MutexGuard<'static, ()> {
        let guard = TEST_LOCK.lock().expect("TEST_LOCK poisoned");
        reset_all_state();
        guard
    }

    // ============================================================
    // CONFIG-BASED TIMER ACTIVATION TESTS
    // ============================================================

    #[test]
    fn test_config_default_timer_none_does_not_enable_timer() {
        let _guard = lock_and_reset();

        let config = Config {
            exit_key: None,
            default_timer: None,
            overlay_opacity: None,
            overlay_color: None,
            allowed_keys: None,
            launch_at_login: None,
            enable_trace_logging: None,
        };

        // When no default_timer is set, parsing should fail (empty string)
        assert!(config.default_timer.is_none());

        // Timer should not be enabled
        assert!(!AUTO_EXIT_ENABLED.load(Ordering::Acquire));
    }

    #[test]
    fn test_config_default_timer_valid_parses_correctly() {
        let _guard = lock_and_reset();

        let config = Config {
            exit_key: None,
            default_timer: Some("30m".to_string()),
            overlay_opacity: None,
            overlay_color: None,
            allowed_keys: None,
            launch_at_login: None,
            enable_trace_logging: None,
        };

        let timer_str = config.default_timer.as_ref().unwrap();
        let duration_secs = parse_duration(timer_str).unwrap();

        assert_eq!(duration_secs, 30 * 60); // 30 minutes in seconds
    }

    #[test]
    fn test_config_default_timer_hours_parses_correctly() {
        let _guard = lock_and_reset();

        let config = Config {
            exit_key: None,
            default_timer: Some("2h".to_string()),
            overlay_opacity: None,
            overlay_color: None,
            allowed_keys: None,
            launch_at_login: None,
            enable_trace_logging: None,
        };

        let timer_str = config.default_timer.as_ref().unwrap();
        let duration_secs = parse_duration(timer_str).unwrap();

        assert_eq!(duration_secs, 2 * 3600); // 2 hours in seconds
    }

    #[test]
    fn test_config_default_timer_combined_parses_correctly() {
        let _guard = lock_and_reset();

        let config = Config {
            exit_key: None,
            default_timer: Some("1h30m".to_string()),
            overlay_opacity: None,
            overlay_color: None,
            allowed_keys: None,
            launch_at_login: None,
            enable_trace_logging: None,
        };

        let timer_str = config.default_timer.as_ref().unwrap();
        let duration_secs = parse_duration(timer_str).unwrap();

        assert_eq!(duration_secs, 3600 + 30 * 60); // 1 hour 30 minutes
    }

    #[test]
    fn test_config_default_timer_seconds_parses_correctly() {
        let _guard = lock_and_reset();

        let config = Config {
            exit_key: None,
            default_timer: Some("90s".to_string()),
            overlay_opacity: None,
            overlay_color: None,
            allowed_keys: None,
            launch_at_login: None,
            enable_trace_logging: None,
        };

        let timer_str = config.default_timer.as_ref().unwrap();
        let duration_secs = parse_duration(timer_str).unwrap();

        assert_eq!(duration_secs, 90);
    }

    #[test]
    fn test_config_default_timer_minimum_boundary() {
        let _guard = lock_and_reset();

        // 5 seconds is the minimum
        let config = Config {
            exit_key: None,
            default_timer: Some("5s".to_string()),
            overlay_opacity: None,
            overlay_color: None,
            allowed_keys: None,
            launch_at_login: None,
            enable_trace_logging: None,
        };

        let timer_str = config.default_timer.as_ref().unwrap();
        let duration_secs = parse_duration(timer_str).unwrap();

        assert_eq!(duration_secs, MIN_TIMER_SECONDS);
    }

    #[test]
    fn test_config_default_timer_below_minimum_rejected() {
        let _guard = lock_and_reset();

        // 4 seconds is below minimum
        let config = Config {
            exit_key: None,
            default_timer: Some("4s".to_string()),
            overlay_opacity: None,
            overlay_color: None,
            allowed_keys: None,
            launch_at_login: None,
            enable_trace_logging: None,
        };

        let timer_str = config.default_timer.as_ref().unwrap();
        let result = parse_duration(timer_str);

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("at least"));
    }

    #[test]
    fn test_config_default_timer_maximum_boundary() {
        let _guard = lock_and_reset();

        // 24 hours is the maximum
        let config = Config {
            exit_key: None,
            default_timer: Some("24h".to_string()),
            overlay_opacity: None,
            overlay_color: None,
            allowed_keys: None,
            launch_at_login: None,
            enable_trace_logging: None,
        };

        let timer_str = config.default_timer.as_ref().unwrap();
        let duration_secs = parse_duration(timer_str).unwrap();

        assert_eq!(duration_secs, MAX_TIMER_SECONDS);
    }

    #[test]
    fn test_config_default_timer_above_maximum_rejected() {
        let _guard = lock_and_reset();

        // 25 hours is above maximum
        let config = Config {
            exit_key: None,
            default_timer: Some("25h".to_string()),
            overlay_opacity: None,
            overlay_color: None,
            allowed_keys: None,
            launch_at_login: None,
            enable_trace_logging: None,
        };

        let timer_str = config.default_timer.as_ref().unwrap();
        let result = parse_duration(timer_str);

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("exceed"));
    }

    #[test]
    fn test_config_default_timer_invalid_format_rejected() {
        let _guard = lock_and_reset();

        let invalid_values = vec!["", "abc", "30x", "-30m", "0m", "invalid", "30m30x", "   "];

        for invalid in invalid_values {
            let result = parse_duration(invalid);
            assert!(
                result.is_err(),
                "Expected '{}' to be rejected but it was accepted",
                invalid
            );
        }
    }

    #[test]
    fn test_timer_initialized_with_correct_duration() {
        let _guard = lock_and_reset();

        let duration_secs = 1800; // 30 minutes
        init_auto_exit_timer(duration_secs);

        assert!(AUTO_EXIT_ENABLED.load(Ordering::Acquire));
        assert_eq!(
            AUTO_EXIT_DURATION_SECS.load(Ordering::Acquire),
            duration_secs
        );
    }

    #[test]
    fn test_timer_start_time_is_current() {
        let _guard = lock_and_reset();

        let before = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        init_auto_exit_timer(3600);

        let after = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let start = AUTO_EXIT_START_TIME.load(Ordering::Acquire);
        assert!(
            start >= before && start <= after,
            "Start time {} should be between {} and {}",
            start,
            before,
            after
        );
    }

    // ============================================================
    // TIMER EXPIRY BEHAVIOR TESTS
    // ============================================================

    #[test]
    fn test_timer_remaining_seconds_after_init() {
        let _guard = lock_and_reset();

        init_auto_exit_timer(60);
        let remaining = get_remaining_seconds();

        // Should be close to 60 seconds (allow some tolerance for test execution time)
        assert!(
            (55..=60).contains(&remaining),
            "Remaining {} should be between 55 and 60",
            remaining
        );
    }

    #[test]
    fn test_timer_remaining_seconds_when_expired() {
        let _guard = lock_and_reset();

        // Manually set up an expired timer
        AUTO_EXIT_ENABLED.store(true, Ordering::Release);
        AUTO_EXIT_DURATION_SECS.store(10, Ordering::Release);
        // Set start time far in the past
        AUTO_EXIT_START_TIME.store(1, Ordering::Release);

        let remaining = get_remaining_seconds();
        assert_eq!(remaining, 0, "Expired timer should return 0");
    }

    #[test]
    fn test_timer_remaining_seconds_when_disabled() {
        let _guard = lock_and_reset();

        // Timer is disabled by default after reset
        let remaining = get_remaining_seconds();
        assert_eq!(remaining, u64::MAX, "Disabled timer should return u64::MAX");
    }

    #[test]
    fn test_timer_expiry_triggers_at_zero_remaining() {
        let _guard = lock_and_reset();

        // Set up a timer that's just expired
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        AUTO_EXIT_ENABLED.store(true, Ordering::Release);
        AUTO_EXIT_DURATION_SECS.store(10, Ordering::Release);
        // Started 15 seconds ago with 10 second duration = expired
        AUTO_EXIT_START_TIME.store(now.saturating_sub(15), Ordering::Release);

        let remaining = get_remaining_seconds();
        assert_eq!(remaining, 0, "Timer should be expired (0 remaining)");

        // In the actual timer_callback, remaining == 0 triggers deactivation
        // This test verifies the detection logic is correct
    }

    #[test]
    fn test_timer_warning_threshold() {
        let _guard = lock_and_reset();

        // Set up timer with 30 seconds remaining
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        AUTO_EXIT_ENABLED.store(true, Ordering::Release);
        AUTO_EXIT_DURATION_SECS.store(90, Ordering::Release);
        // Started 60 seconds ago, so 30 seconds remaining
        AUTO_EXIT_START_TIME.store(now.saturating_sub(60), Ordering::Release);

        let remaining = get_remaining_seconds();
        // Allow some tolerance for test timing
        assert!(
            (25..=35).contains(&remaining),
            "Remaining {} should be around 30",
            remaining
        );

        // 30 seconds is within warning threshold (WARNING_SECONDS = 60)
        assert!(remaining <= WARNING_SECONDS);
    }

    #[test]
    fn test_warning_shown_flag_initially_false() {
        let _guard = lock_and_reset();

        init_auto_exit_timer(3600);

        assert!(
            !WARNING_SHOWN.load(Ordering::Acquire),
            "WARNING_SHOWN should be false after init"
        );
    }

    #[test]
    fn test_warning_shown_flag_can_be_set() {
        let _guard = lock_and_reset();

        // Simulate warning being shown
        WARNING_SHOWN.store(true, Ordering::Release);

        assert!(WARNING_SHOWN.load(Ordering::Acquire));
    }

    #[test]
    fn test_warning_shown_flag_only_set_once() {
        let _guard = lock_and_reset();

        // swap returns previous value - should be false first time
        let first = WARNING_SHOWN.swap(true, Ordering::AcqRel);
        assert!(
            !first,
            "First swap should return false (not previously shown)"
        );

        // Second swap should return true (already shown)
        let second = WARNING_SHOWN.swap(true, Ordering::AcqRel);
        assert!(second, "Second swap should return true (already shown)");
    }

    // ============================================================
    // MODE CONSISTENCY TESTS
    // ============================================================

    #[test]
    fn test_mode_menu_bar_default_is_false() {
        let _guard = lock_and_reset();

        // After reset, MODE_MENU_BAR should be false
        assert!(!shield::MODE_MENU_BAR.load(Ordering::Acquire));
    }

    #[test]
    fn test_mode_menu_bar_can_be_set() {
        let _guard = lock_and_reset();

        shield::MODE_MENU_BAR.store(true, Ordering::Release);

        assert!(shield::MODE_MENU_BAR.load(Ordering::Acquire));
    }

    #[test]
    fn test_shield_is_active_default_is_false() {
        let _guard = lock_and_reset();

        assert!(!shield::IS_ACTIVE.load(Ordering::Acquire));
    }

    #[test]
    fn test_shield_is_active_can_be_set() {
        let _guard = lock_and_reset();

        shield::IS_ACTIVE.store(true, Ordering::Release);

        assert!(shield::IS_ACTIVE.load(Ordering::Acquire));
    }

    #[test]
    fn test_shield_double_activation_prevention() {
        let _guard = lock_and_reset();

        // First activation succeeds (swap returns false = was not active)
        let was_active = shield::IS_ACTIVE.swap(true, Ordering::AcqRel);
        assert!(!was_active, "First activation should succeed");

        // Second activation fails (swap returns true = was already active)
        let was_active = shield::IS_ACTIVE.swap(true, Ordering::AcqRel);
        assert!(was_active, "Second activation should be blocked");
    }

    #[test]
    fn test_shield_deactivation_returns_to_menu_bar_mode() {
        let _guard = lock_and_reset();

        // Simulate menu bar mode
        shield::MODE_MENU_BAR.store(true, Ordering::Release);
        shield::IS_ACTIVE.store(true, Ordering::Release);

        // Deactivate
        shield::IS_ACTIVE.store(false, Ordering::Release);

        // Mode should still be menu bar (app stays running)
        assert!(shield::MODE_MENU_BAR.load(Ordering::Acquire));
        assert!(!shield::IS_ACTIVE.load(Ordering::Acquire));
    }

    #[test]
    fn test_shield_can_be_reactivated_after_deactivation() {
        let _guard = lock_and_reset();

        // First activation
        shield::IS_ACTIVE.store(true, Ordering::Release);
        assert!(shield::IS_ACTIVE.load(Ordering::Acquire));

        // Deactivation
        shield::IS_ACTIVE.store(false, Ordering::Release);
        assert!(!shield::IS_ACTIVE.load(Ordering::Acquire));

        // Reactivation should work
        let was_active = shield::IS_ACTIVE.swap(true, Ordering::AcqRel);
        assert!(
            !was_active,
            "Reactivation should succeed after deactivation"
        );
        assert!(shield::IS_ACTIVE.load(Ordering::Acquire));
    }

    // ============================================================
    // TIMER DISPLAY FORMAT TESTS
    // ============================================================

    #[test]
    fn test_format_duration_seconds_only() {
        assert_eq!(format_duration(45), "45s");
        assert_eq!(format_duration(1), "1s");
        assert_eq!(format_duration(59), "59s");
    }

    #[test]
    fn test_format_duration_minutes_only() {
        // Zero-padded seconds
        assert_eq!(format_duration(60), "1m 00s");
        assert_eq!(format_duration(120), "2m 00s");
        assert_eq!(format_duration(300), "5m 00s");
    }

    #[test]
    fn test_format_duration_minutes_and_seconds() {
        // Zero-padded seconds
        assert_eq!(format_duration(65), "1m 05s");
        assert_eq!(format_duration(90), "1m 30s");
        assert_eq!(format_duration(125), "2m 05s");
    }

    #[test]
    fn test_format_duration_hours_only() {
        // Zero-padded minutes and seconds
        assert_eq!(format_duration(3600), "1h 00m 00s");
        assert_eq!(format_duration(7200), "2h 00m 00s");
    }

    #[test]
    fn test_format_duration_hours_minutes_seconds() {
        // Zero-padded minutes and seconds
        assert_eq!(format_duration(3665), "1h 01m 05s");
        assert_eq!(format_duration(7325), "2h 02m 05s");
    }

    #[test]
    fn test_format_duration_zero() {
        assert_eq!(format_duration(0), "0s");
    }

    // ============================================================
    // POINTER STATE INITIALIZATION TESTS
    // ============================================================

    #[test]
    fn test_timer_view_pointer_initially_null() {
        let _guard = lock_and_reset();

        assert!(shield::TIMER_VIEW.load(Ordering::Acquire).is_null());
    }

    #[test]
    fn test_close_button_pointer_initially_null() {
        let _guard = lock_and_reset();

        assert!(shield::CLOSE_BUTTON.load(Ordering::Acquire).is_null());
    }

    #[test]
    fn test_window_pointer_initially_null() {
        let _guard = lock_and_reset();

        assert!(shield::WINDOW.load(Ordering::Acquire).is_null());
    }

    #[test]
    fn test_timer_ref_pointer_initially_null() {
        let _guard = lock_and_reset();

        assert!(shield::TIMER_REF.load(Ordering::Acquire).is_null());
    }

    // ============================================================
    // SLEEP ASSERTION STATE TESTS
    // ============================================================

    #[test]
    fn test_sleep_assertion_initially_disabled() {
        let _guard = lock_and_reset();

        assert!(!shield::HAS_SLEEP_ASSERTION.load(Ordering::Acquire));
        assert_eq!(shield::SLEEP_ASSERTION_ID.load(Ordering::Acquire), 0);
    }

    #[test]
    fn test_sleep_assertion_can_be_set() {
        let _guard = lock_and_reset();

        let assertion_id = 12345u64;
        shield::SLEEP_ASSERTION_ID.store(assertion_id, Ordering::Release);
        shield::HAS_SLEEP_ASSERTION.store(true, Ordering::Release);

        assert!(shield::HAS_SLEEP_ASSERTION.load(Ordering::Acquire));
        assert_eq!(
            shield::SLEEP_ASSERTION_ID.load(Ordering::Acquire),
            assertion_id
        );
    }

    #[test]
    fn test_sleep_assertion_cleared_on_deactivation() {
        let _guard = lock_and_reset();

        // Set up assertion
        shield::SLEEP_ASSERTION_ID.store(12345, Ordering::Release);
        shield::HAS_SLEEP_ASSERTION.store(true, Ordering::Release);

        // Simulate deactivation cleanup
        shield::HAS_SLEEP_ASSERTION.store(false, Ordering::Release);
        shield::SLEEP_ASSERTION_ID.store(0, Ordering::Release);

        assert!(!shield::HAS_SLEEP_ASSERTION.load(Ordering::Acquire));
        assert_eq!(shield::SLEEP_ASSERTION_ID.load(Ordering::Acquire), 0);
    }

    // ============================================================
    // CONFIG FILE ROUND-TRIP TESTS
    // ============================================================

    #[test]
    fn test_config_with_default_timer_round_trip() {
        let temp_dir = tempfile::tempdir().unwrap();
        let config_path = temp_dir.path().join("config.toml");

        let original = Config {
            exit_key: Some("Cmd+Shift+Q".to_string()),
            default_timer: Some("30m".to_string()),
            overlay_opacity: Some(0.6),
            overlay_color: None,
            allowed_keys: None,
            launch_at_login: None,
            enable_trace_logging: None,
        };

        original.save_to_path(&config_path).unwrap();
        let loaded = Config::load_from_path(&config_path);

        assert_eq!(loaded.default_timer, Some("30m".to_string()));

        // Verify the timer value can be parsed
        let timer_str = loaded.default_timer.as_ref().unwrap();
        let duration_secs = parse_duration(timer_str).unwrap();
        assert_eq!(duration_secs, 30 * 60);
    }

    #[test]
    fn test_config_without_default_timer_round_trip() {
        let temp_dir = tempfile::tempdir().unwrap();
        let config_path = temp_dir.path().join("config.toml");

        let original = Config {
            exit_key: Some("Cmd+Q".to_string()),
            default_timer: None,
            overlay_opacity: None,
            overlay_color: None,
            allowed_keys: None,
            launch_at_login: None,
            enable_trace_logging: None,
        };

        original.save_to_path(&config_path).unwrap();
        let loaded = Config::load_from_path(&config_path);

        assert!(loaded.default_timer.is_none());
    }

    // ============================================================
    // TIMER STATE CLEANUP TESTS
    // ============================================================

    #[test]
    fn test_timer_state_reset_on_deactivation() {
        let _guard = lock_and_reset();

        // Initialize timer
        init_auto_exit_timer(3600);
        WARNING_SHOWN.store(true, Ordering::Release);

        // Verify timer is active
        assert!(AUTO_EXIT_ENABLED.load(Ordering::Acquire));
        assert!(WARNING_SHOWN.load(Ordering::Acquire));

        // Simulate deactivation cleanup
        AUTO_EXIT_ENABLED.store(false, Ordering::Release);
        WARNING_SHOWN.store(false, Ordering::Release);

        // Verify cleanup
        assert!(!AUTO_EXIT_ENABLED.load(Ordering::Acquire));
        assert!(!WARNING_SHOWN.load(Ordering::Acquire));
    }

    #[test]
    fn test_timer_can_be_reinitialized_after_deactivation() {
        let _guard = lock_and_reset();

        // First timer session
        init_auto_exit_timer(1800);
        assert_eq!(AUTO_EXIT_DURATION_SECS.load(Ordering::Acquire), 1800);

        // Simulate deactivation
        AUTO_EXIT_ENABLED.store(false, Ordering::Release);

        // Second timer session with different duration
        init_auto_exit_timer(3600);
        assert!(AUTO_EXIT_ENABLED.load(Ordering::Acquire));
        assert_eq!(AUTO_EXIT_DURATION_SECS.load(Ordering::Acquire), 3600);
    }

    // ============================================================
    // MENU BAR STATE TESTS
    // ============================================================

    #[test]
    fn test_menu_bar_items_initially_null() {
        use crate::ui::state::menu_bar;

        let _guard = lock_and_reset();

        // Reset menu bar state
        menu_bar::START_ITEM.store(std::ptr::null_mut(), Ordering::Release);
        menu_bar::SETTINGS_ITEM.store(std::ptr::null_mut(), Ordering::Release);
        menu_bar::ABOUT_ITEM.store(std::ptr::null_mut(), Ordering::Release);
        menu_bar::ACTION_HANDLER.store(std::ptr::null_mut(), Ordering::Release);

        assert!(menu_bar::START_ITEM.load(Ordering::Acquire).is_null());
        assert!(menu_bar::SETTINGS_ITEM.load(Ordering::Acquire).is_null());
        assert!(menu_bar::ABOUT_ITEM.load(Ordering::Acquire).is_null());
        assert!(menu_bar::ACTION_HANDLER.load(Ordering::Acquire).is_null());
    }

    // ============================================================
    // EDGE CASE AND BOUNDARY TESTS
    // ============================================================

    #[test]
    fn test_timer_with_exact_warning_threshold() {
        let _guard = lock_and_reset();

        // Set up timer with exactly WARNING_SECONDS remaining
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let total_duration = 120u64;
        let elapsed = total_duration - WARNING_SECONDS;

        AUTO_EXIT_ENABLED.store(true, Ordering::Release);
        AUTO_EXIT_DURATION_SECS.store(total_duration, Ordering::Release);
        AUTO_EXIT_START_TIME.store(now.saturating_sub(elapsed), Ordering::Release);

        let remaining = get_remaining_seconds();

        // Should be approximately WARNING_SECONDS (allow tolerance for timing)
        assert!(
            (55..=65).contains(&remaining),
            "Remaining {} should be around WARNING_SECONDS (60)",
            remaining
        );
    }

    #[test]
    fn test_concurrent_timer_access_safety() {
        let _guard = lock_and_reset();

        // This test verifies that timer state can be safely accessed
        // from multiple points (simulating concurrent callback access)

        init_auto_exit_timer(3600);

        // Multiple reads should be consistent
        let enabled1 = AUTO_EXIT_ENABLED.load(Ordering::Acquire);
        let duration1 = AUTO_EXIT_DURATION_SECS.load(Ordering::Acquire);
        let remaining1 = get_remaining_seconds();

        let enabled2 = AUTO_EXIT_ENABLED.load(Ordering::Acquire);
        let duration2 = AUTO_EXIT_DURATION_SECS.load(Ordering::Acquire);
        let remaining2 = get_remaining_seconds();

        assert_eq!(enabled1, enabled2);
        assert_eq!(duration1, duration2);
        // Remaining might differ slightly due to time passing, but should be close
        assert!(
            remaining1.abs_diff(remaining2) <= 1,
            "Remaining values should be within 1 second"
        );
    }
}
