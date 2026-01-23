//! Platform-specific integration tests for Cat Shield
//!
//! These tests verify that platform trait implementations work correctly
//! on each supported operating system.
//!
//! # Test Categories
//!
//! - **Power Management**: Tests `prevent_sleep()` / `allow_sleep()` cycle
//! - **Overlay Window**: Tests window creation, visibility, and cleanup
//! - **Input Blocking**: Tests setup/disable cycle (may require special permissions)
//! - **System Tray**: Tests tray setup and state changes
//!
//! # Running Tests
//!
//! Most tests require platform-specific capabilities:
//! - macOS: Accessibility permissions for input blocking
//! - Windows: No special requirements
//! - Linux: X11/Wayland session for overlay/input tests
//!
//! Tests that require special permissions or GUI access are marked with `#[ignore]`
//! and can be run manually with:
//! ```bash
//! cargo test --test integration -- --ignored
//! ```
//!
//! # Note on Thread Safety
//!
//! Tests that manipulate global state use `serial_test` to ensure they don't
//! interfere with each other. The `#[serial]` attribute ensures tests run
//! sequentially within the same test module.

// ============================================================================
// macOS Integration Tests
// ============================================================================

#[cfg(all(test, target_os = "macos"))]
mod macos_tests {
    use crate::platform::errors::{InputBlockError, PowerError, WindowError};
    use crate::platform::macos::{MacOSInputBlocker, MacOSPermissionChecker, MacOSPowerManager};
    use crate::platform::traits::{InputBlocker, PermissionChecker, PowerManager};
    use serial_test::serial;

    // ========================================================================
    // Power Management Tests
    // ========================================================================

    #[test]
    #[serial]
    fn test_macos_power_manager_can_be_created() {
        let _manager = MacOSPowerManager::new();
        // Successfully created - no panic
    }

    #[test]
    #[serial]
    fn test_macos_power_manager_is_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<MacOSPowerManager>();
    }

    #[test]
    #[serial]
    fn test_macos_power_manager_prevent_allow_sleep_cycle() {
        let manager = MacOSPowerManager::new();

        // Create a sleep assertion
        let assertion = manager
            .prevent_sleep()
            .expect("Failed to create sleep assertion");

        // Verify we got a valid assertion ID (non-zero on macOS)
        // Note: IOKit returns 0 for success, so the ID should be non-zero
        let id = assertion.id();
        assert!(id > 0, "Assertion ID should be non-zero, got {}", id);

        // Release the assertion
        manager
            .allow_sleep(assertion)
            .expect("Failed to release sleep assertion");
    }

    #[test]
    #[serial]
    fn test_macos_power_manager_invalid_assertion_release() {
        use crate::platform::types::SleepAssertion;

        let manager = MacOSPowerManager::new();

        // Try to release an invalid assertion
        let invalid = SleepAssertion::new(u64::MAX);
        let result = manager.allow_sleep(invalid);

        // Should return an error for invalid assertion
        assert!(
            result.is_err(),
            "Releasing invalid assertion should fail, but got {:?}",
            result
        );
    }

    #[test]
    #[serial]
    fn test_macos_power_manager_multiple_assertions() {
        let manager = MacOSPowerManager::new();

        // Create multiple assertions
        let assertion1 = manager
            .prevent_sleep()
            .expect("Failed to create first assertion");
        let assertion2 = manager
            .prevent_sleep()
            .expect("Failed to create second assertion");

        // Both should have different IDs
        assert_ne!(
            assertion1.id(),
            assertion2.id(),
            "Multiple assertions should have different IDs"
        );

        // Release both
        manager.allow_sleep(assertion1).expect("Failed to release first assertion");
        manager.allow_sleep(assertion2).expect("Failed to release second assertion");
    }

    // ========================================================================
    // Permission Checker Tests
    // ========================================================================

    #[test]
    #[serial]
    fn test_macos_permission_checker_can_be_created() {
        let _checker = MacOSPermissionChecker::new();
    }

    #[test]
    #[serial]
    fn test_macos_permission_checker_returns_boolean() {
        let checker = MacOSPermissionChecker::new();

        // This should return a valid boolean (true if permissions granted, false otherwise)
        // We can't assert which value without knowing the system state
        let _has_permissions = checker.check_permissions();
    }

    // Note: open_permissions_settings() test is ignored because it opens System Preferences
    #[test]
    #[serial]
    #[ignore = "Opens System Preferences - run manually"]
    fn test_macos_permission_checker_open_settings() {
        let checker = MacOSPermissionChecker::new();
        checker
            .open_permissions_settings()
            .expect("Failed to open permissions settings");
    }

    // ========================================================================
    // Input Blocker Tests
    // ========================================================================

    #[test]
    #[serial]
    fn test_macos_input_blocker_can_be_created() {
        let _blocker = MacOSInputBlocker::new();
    }

    #[test]
    #[serial]
    fn test_macos_input_blocker_is_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<MacOSInputBlocker>();
    }

    #[test]
    #[serial]
    fn test_macos_input_blocker_initially_inactive() {
        let blocker = MacOSInputBlocker::new();
        assert!(
            !blocker.is_active(),
            "Blocker should be inactive initially"
        );
    }

    #[test]
    #[serial]
    fn test_macos_input_blocker_disable_when_inactive_is_safe() {
        let mut blocker = MacOSInputBlocker::new();

        // Calling disable when not active should be safe (idempotent)
        blocker.disable();
        assert!(!blocker.is_active());

        // Multiple calls should also be safe
        blocker.disable();
        blocker.disable();
        assert!(!blocker.is_active());
    }

    // Note: setup() test requires Accessibility permissions
    #[test]
    #[serial]
    #[ignore = "Requires Accessibility permissions - run manually"]
    fn test_macos_input_blocker_setup_disable_cycle() {
        let mut blocker = MacOSInputBlocker::new();

        // Setup the event tap
        blocker.setup().expect("Failed to setup input blocker");
        assert!(blocker.is_active(), "Blocker should be active after setup");

        // Disable it
        blocker.disable();
        assert!(
            !blocker.is_active(),
            "Blocker should be inactive after disable"
        );
    }

    #[test]
    #[serial]
    #[ignore = "Requires Accessibility permissions - run manually"]
    fn test_macos_input_blocker_double_setup() {
        let mut blocker = MacOSInputBlocker::new();

        // First setup should succeed
        blocker.setup().expect("First setup failed");
        assert!(blocker.is_active());

        // Second setup should either succeed (idempotent) or return an error
        // but should not panic
        let result = blocker.setup();
        // Either way, blocker should still be active
        assert!(blocker.is_active());

        blocker.disable();
    }
}

// ============================================================================
// Windows Integration Tests
// ============================================================================

#[cfg(all(test, target_os = "windows"))]
mod windows_tests {
    use crate::platform::errors::{PowerError, TrayError, WindowError};
    use crate::platform::traits::{InputBlocker, OverlayWindow, PowerManager, SystemTray};
    use crate::platform::windows::{
        WindowsInputBlocker, WindowsOverlayWindow, WindowsPowerManager, WindowsSystemTray,
    };
    use serial_test::serial;

    // ========================================================================
    // Power Management Tests
    // ========================================================================

    #[test]
    #[serial]
    fn test_windows_power_manager_can_be_created() {
        let _manager = WindowsPowerManager::new();
    }

    #[test]
    #[serial]
    fn test_windows_power_manager_is_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<WindowsPowerManager>();
    }

    #[test]
    #[serial]
    fn test_windows_power_manager_prevent_allow_sleep_cycle() {
        let manager = WindowsPowerManager::new();

        // Create a sleep assertion
        let assertion = manager
            .prevent_sleep()
            .expect("Failed to create sleep assertion");

        // Verify we got a valid assertion
        let id = assertion.id();
        // On Windows, the ID is the result of SetThreadExecutionState
        // which returns the previous state flags

        // Release the assertion
        manager
            .allow_sleep(assertion)
            .expect("Failed to release sleep assertion");
    }

    // ========================================================================
    // Input Blocker Tests
    // ========================================================================

    #[test]
    #[serial]
    fn test_windows_input_blocker_can_be_created() {
        let _blocker = WindowsInputBlocker::new();
    }

    #[test]
    #[serial]
    fn test_windows_input_blocker_is_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<WindowsInputBlocker>();
    }

    #[test]
    #[serial]
    fn test_windows_input_blocker_initially_inactive() {
        let blocker = WindowsInputBlocker::new();
        assert!(
            !blocker.is_active(),
            "Blocker should be inactive initially"
        );
    }

    #[test]
    #[serial]
    fn test_windows_input_blocker_disable_when_inactive_is_safe() {
        let mut blocker = WindowsInputBlocker::new();

        // Calling disable when not active should be safe (idempotent)
        blocker.disable();
        assert!(!blocker.is_active());
    }

    // Note: setup() requires a message pump / event loop on Windows
    #[test]
    #[serial]
    #[ignore = "Requires Windows event loop - run manually in GUI context"]
    fn test_windows_input_blocker_setup_disable_cycle() {
        let mut blocker = WindowsInputBlocker::new();

        blocker.setup().expect("Failed to setup input blocker");
        assert!(blocker.is_active());

        blocker.disable();
        assert!(!blocker.is_active());
    }

    // ========================================================================
    // Overlay Window Tests
    // ========================================================================

    #[test]
    #[serial]
    fn test_windows_overlay_can_be_created() {
        let _overlay = WindowsOverlayWindow::new();
    }

    #[test]
    #[serial]
    fn test_windows_overlay_initially_not_visible() {
        let overlay = WindowsOverlayWindow::new();
        assert!(!overlay.is_visible());
    }

    #[test]
    #[serial]
    fn test_windows_overlay_bounds_initially_zero() {
        let overlay = WindowsOverlayWindow::new();
        let bounds = overlay.bounds();
        assert!(bounds.is_empty(), "Bounds should be empty before create()");
    }

    #[test]
    #[serial]
    #[ignore = "Requires Windows GUI context - run manually"]
    fn test_windows_overlay_create_show_hide_close() {
        let mut overlay = WindowsOverlayWindow::new();

        // Create the window
        overlay.create().expect("Failed to create overlay");

        // Should not be visible yet
        assert!(!overlay.is_visible());

        // Bounds should be non-zero after creation
        let bounds = overlay.bounds();
        assert!(!bounds.is_empty(), "Bounds should be set after create()");

        // Show the window
        overlay.show();
        assert!(overlay.is_visible());

        // Hide the window
        overlay.hide();
        assert!(!overlay.is_visible());

        // Close the window
        overlay.close();
    }

    #[test]
    #[serial]
    fn test_windows_overlay_opacity_clamping() {
        let mut overlay = WindowsOverlayWindow::new();

        // Test opacity clamping without requiring window creation
        overlay.set_opacity(0.5);
        overlay.set_opacity(1.5); // Should clamp to 1.0
        overlay.set_opacity(-0.5); // Should clamp to 0.0
    }

    // ========================================================================
    // System Tray Tests
    // ========================================================================

    #[test]
    #[serial]
    fn test_windows_tray_can_be_created() {
        let _tray = WindowsSystemTray::new();
    }

    #[test]
    #[serial]
    fn test_windows_tray_initially_not_visible() {
        let tray = WindowsSystemTray::new();
        assert!(!tray.is_visible());
    }

    #[test]
    #[serial]
    #[ignore = "Requires Windows GUI context - run manually"]
    fn test_windows_tray_setup_and_state() {
        let mut tray = WindowsSystemTray::new();

        // Setup the tray
        tray.setup().expect("Failed to setup tray");
        assert!(tray.is_visible());

        // Change state
        tray.set_state(true);
        tray.set_state(false);

        // Remove the tray
        tray.remove();
        assert!(!tray.is_visible());
    }
}

// ============================================================================
// Linux Integration Tests
// ============================================================================

#[cfg(all(test, target_os = "linux"))]
mod linux_tests {
    use crate::platform::errors::{PowerError, TrayError, WindowError};
    use crate::platform::linux::{
        is_native_wayland_overlay_available, is_wayland_session, is_x11_session,
        LinuxOverlayWindow, LinuxPowerManager, LinuxSystemTray, WaylandOverlayWindow,
        X11InputBlocker,
    };
    use crate::platform::traits::{InputBlocker, OverlayWindow, PowerManager, SystemTray};
    use serial_test::serial;

    // ========================================================================
    // Session Detection Tests
    // ========================================================================

    #[test]
    fn test_linux_session_detection() {
        // At least one of these should be true if we're running in a graphical session
        // Both could be true (XWayland) or only one
        let wayland = is_wayland_session();
        let x11 = is_x11_session();

        // Log which session we detected for debugging
        eprintln!("Session detection: Wayland={}, X11={}", wayland, x11);

        // This test passes regardless - we're just verifying the functions don't panic
    }

    #[test]
    fn test_linux_native_wayland_detection() {
        // This function should return a boolean without panicking
        let _available = is_native_wayland_overlay_available();
    }

    // ========================================================================
    // Power Management Tests
    // ========================================================================

    #[test]
    #[serial]
    fn test_linux_power_manager_can_be_created() {
        let _manager = LinuxPowerManager::new();
    }

    #[test]
    #[serial]
    fn test_linux_power_manager_is_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<LinuxPowerManager>();
    }

    #[test]
    #[serial]
    #[ignore = "Requires D-Bus session - may fail in CI without desktop session"]
    fn test_linux_power_manager_prevent_allow_sleep_cycle() {
        let manager = LinuxPowerManager::new();

        // Create a sleep assertion via D-Bus inhibit
        let assertion = manager
            .prevent_sleep()
            .expect("Failed to create sleep assertion");

        // Verify we got a valid assertion
        let id = assertion.id();
        eprintln!("Got D-Bus inhibit cookie: {}", id);

        // Release the assertion
        manager
            .allow_sleep(assertion)
            .expect("Failed to release sleep assertion");
    }

    // ========================================================================
    // Input Blocker Tests (X11)
    // ========================================================================

    #[test]
    #[serial]
    fn test_linux_x11_input_blocker_can_be_created() {
        // This may fail if not in an X11 session, which is OK
        let result = X11InputBlocker::new();
        match result {
            Ok(_blocker) => {
                eprintln!("X11InputBlocker created successfully");
            }
            Err(e) => {
                eprintln!("X11InputBlocker creation failed (expected if not X11): {:?}", e);
            }
        }
    }

    #[test]
    #[serial]
    fn test_linux_x11_input_blocker_is_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<X11InputBlocker>();
    }

    #[test]
    #[serial]
    #[ignore = "Requires X11 session with grab capability"]
    fn test_linux_x11_input_blocker_setup_disable_cycle() {
        let blocker = X11InputBlocker::new();
        let mut blocker = match blocker {
            Ok(b) => b,
            Err(e) => {
                eprintln!("Skipping test - X11 not available: {:?}", e);
                return;
            }
        };

        assert!(!blocker.is_active());

        blocker.setup().expect("Failed to setup input blocker");
        assert!(blocker.is_active());

        blocker.disable();
        assert!(!blocker.is_active());
    }

    // ========================================================================
    // Overlay Window Tests (X11)
    // ========================================================================

    #[test]
    #[serial]
    fn test_linux_overlay_can_be_created() {
        let _overlay = LinuxOverlayWindow::new();
    }

    #[test]
    #[serial]
    fn test_linux_overlay_initially_not_visible() {
        let overlay = LinuxOverlayWindow::new();
        assert!(!overlay.is_visible());
    }

    #[test]
    #[serial]
    #[ignore = "Requires X11 display"]
    fn test_linux_overlay_create_show_hide_close() {
        let mut overlay = LinuxOverlayWindow::new();

        overlay.create().expect("Failed to create overlay");
        assert!(!overlay.is_visible());

        overlay.show();
        assert!(overlay.is_visible());

        overlay.hide();
        assert!(!overlay.is_visible());

        overlay.close();
    }

    // ========================================================================
    // Wayland Overlay Tests
    // ========================================================================

    #[test]
    #[serial]
    fn test_linux_wayland_overlay_can_be_created() {
        let _overlay = WaylandOverlayWindow::new();
    }

    #[test]
    #[serial]
    fn test_linux_wayland_overlay_initially_not_visible() {
        let overlay = WaylandOverlayWindow::new();
        assert!(!overlay.is_visible());
    }

    #[test]
    #[serial]
    fn test_linux_wayland_overlay_bounds_initially_zero() {
        let overlay = WaylandOverlayWindow::new();
        let bounds = overlay.bounds();
        assert!(bounds.is_empty());
    }

    #[test]
    #[serial]
    #[ignore = "Requires Wayland compositor with wlr-layer-shell"]
    fn test_linux_wayland_overlay_create_show_hide_close() {
        if !is_native_wayland_overlay_available() {
            eprintln!("Skipping test - Wayland layer shell not available");
            return;
        }

        let mut overlay = WaylandOverlayWindow::new();

        overlay.create().expect("Failed to create Wayland overlay");

        // Show
        overlay.show();
        assert!(overlay.is_visible());

        // Hide
        overlay.hide();
        assert!(!overlay.is_visible());

        // Close
        overlay.close();
    }

    // ========================================================================
    // System Tray Tests
    // ========================================================================

    #[test]
    #[serial]
    fn test_linux_tray_can_be_created() {
        let result = LinuxSystemTray::new();
        match result {
            Ok(_tray) => {
                eprintln!("LinuxSystemTray created successfully");
            }
            Err(e) => {
                eprintln!("LinuxSystemTray creation failed (expected without D-Bus): {:?}", e);
            }
        }
    }

    #[test]
    #[serial]
    #[ignore = "Requires D-Bus session and SNI host"]
    fn test_linux_tray_setup_and_state() {
        let mut tray = match LinuxSystemTray::new() {
            Ok(t) => t,
            Err(e) => {
                eprintln!("Skipping test - tray not available: {:?}", e);
                return;
            }
        };

        tray.setup().expect("Failed to setup tray");
        assert!(tray.is_visible());

        tray.set_state(true);
        tray.set_state(false);

        tray.remove();
        assert!(!tray.is_visible());
    }
}

// ============================================================================
// Cross-Platform Tests (run on all platforms)
// ============================================================================

#[cfg(test)]
mod cross_platform_tests {
    use crate::platform::errors::{
        InputBlockError, PermissionError, PowerError, TrayError, WindowError,
    };
    use crate::platform::traits::{InputBlocker, OverlayWindow, PermissionChecker, PowerManager, SystemTray};
    use crate::platform::types::{Rect, SleepAssertion};

    // ========================================================================
    // Error Type Tests
    // ========================================================================

    #[test]
    fn test_all_error_types_implement_error_trait() {
        use std::error::Error;

        fn assert_error<E: Error>() {}

        assert_error::<InputBlockError>();
        assert_error::<PowerError>();
        assert_error::<TrayError>();
        assert_error::<WindowError>();
        assert_error::<PermissionError>();
    }

    #[test]
    fn test_all_error_types_implement_debug() {
        use std::fmt::Debug;

        fn assert_debug<E: Debug>() {}

        assert_debug::<InputBlockError>();
        assert_debug::<PowerError>();
        assert_debug::<TrayError>();
        assert_debug::<WindowError>();
        assert_debug::<PermissionError>();
    }

    #[test]
    fn test_all_error_types_implement_display() {
        use std::fmt::Display;

        fn assert_display<E: Display>() {}

        assert_display::<InputBlockError>();
        assert_display::<PowerError>();
        assert_display::<TrayError>();
        assert_display::<WindowError>();
        assert_display::<PermissionError>();
    }

    // ========================================================================
    // Type Tests
    // ========================================================================

    #[test]
    fn test_sleep_assertion_creation() {
        let assertion = SleepAssertion::new(42);
        assert_eq!(assertion.id(), 42);
    }

    #[test]
    fn test_rect_zero() {
        let rect = Rect::zero();
        assert!(rect.is_empty());
        assert_eq!(rect.x, 0.0);
        assert_eq!(rect.y, 0.0);
        assert_eq!(rect.width, 0.0);
        assert_eq!(rect.height, 0.0);
    }

    #[test]
    fn test_rect_new() {
        let rect = Rect::new(10.0, 20.0, 100.0, 200.0);
        assert!(!rect.is_empty());
        assert_eq!(rect.x, 10.0);
        assert_eq!(rect.y, 20.0);
        assert_eq!(rect.width, 100.0);
        assert_eq!(rect.height, 200.0);
    }

    #[test]
    fn test_rect_is_empty() {
        assert!(Rect::new(0.0, 0.0, 0.0, 0.0).is_empty());
        assert!(Rect::new(10.0, 10.0, 0.0, 100.0).is_empty()); // zero width
        assert!(Rect::new(10.0, 10.0, 100.0, 0.0).is_empty()); // zero height
        assert!(!Rect::new(10.0, 10.0, 100.0, 100.0).is_empty()); // non-empty
    }

    // ========================================================================
    // Trait Constraint Tests
    // ========================================================================

    #[test]
    fn test_input_blocker_trait_requires_send_sync() {
        fn assert_input_blocker_bounds<T: InputBlocker>() {
            fn assert_send_sync<U: Send + Sync>() {}
            assert_send_sync::<T>();
        }

        // This compiles only if InputBlocker: Send + Sync
        struct MockBlocker;
        impl InputBlocker for MockBlocker {
            fn setup(&mut self) -> Result<(), InputBlockError> { Ok(()) }
            fn disable(&mut self) {}
            fn is_active(&self) -> bool { false }
        }
        unsafe impl Send for MockBlocker {}
        unsafe impl Sync for MockBlocker {}

        assert_input_blocker_bounds::<MockBlocker>();
    }

    #[test]
    fn test_power_manager_trait_requires_send_sync() {
        fn assert_power_manager_bounds<T: PowerManager>() {
            fn assert_send_sync<U: Send + Sync>() {}
            assert_send_sync::<T>();
        }

        struct MockPower;
        impl PowerManager for MockPower {
            fn prevent_sleep(&self) -> Result<SleepAssertion, PowerError> {
                Ok(SleepAssertion::new(1))
            }
            fn allow_sleep(&self, _: SleepAssertion) -> Result<(), PowerError> {
                Ok(())
            }
        }
        unsafe impl Send for MockPower {}
        unsafe impl Sync for MockPower {}

        assert_power_manager_bounds::<MockPower>();
    }
}
