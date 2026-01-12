//! macOS platform trait implementations
//!
//! This module provides macOS-specific implementations of the platform abstraction traits,
//! wrapping the existing event tap, power management, and accessibility code.

use super::{
    accessibility::{
        check_accessibility, check_accessibility_with_prompt, open_accessibility_settings,
    },
    event_tap::{disable_event_tap, setup_event_tap, EVENT_TAP},
    power::{allow_sleep, prevent_sleep},
};
use crate::platform::errors::{InputBlockError, PermissionError, PowerError};
use crate::platform::traits::{InputBlocker, PermissionChecker, PowerManager};
use crate::platform::types::SleepAssertion;
use std::sync::atomic::Ordering;

/// macOS implementation of the `InputBlocker` trait.
///
/// Wraps the existing `event_tap.rs` functionality to intercept and block
/// keyboard input using CGEventTap.
#[derive(Debug, Default)]
pub struct MacOSInputBlocker {
    /// Tracks whether the event tap is currently active
    active: bool,
}

impl MacOSInputBlocker {
    /// Creates a new `MacOSInputBlocker`.
    #[must_use]
    pub fn new() -> Self {
        Self { active: false }
    }
}

impl InputBlocker for MacOSInputBlocker {
    fn setup(&mut self) -> Result<(), InputBlockError> {
        if self.active {
            return Ok(());
        }

        if setup_event_tap() {
            self.active = true;
            Ok(())
        } else {
            Err(InputBlockError::CreationFailed(
                "Failed to create CGEventTap - accessibility permissions may be required"
                    .to_string(),
            ))
        }
    }

    fn disable(&mut self) {
        if self.active {
            disable_event_tap();
            self.active = false;
        }
    }

    fn is_active(&self) -> bool {
        // Check both our internal state and the actual event tap state
        self.active && !EVENT_TAP.load(Ordering::Acquire).is_null()
    }
}

// SAFETY: MacOSInputBlocker is Send + Sync because:
// - The `active` field is only accessed through &mut self (setup/disable) or &self (is_active)
// - The underlying EVENT_TAP is an AtomicPtr which is inherently thread-safe
// - The event_tap functions use proper atomic ordering for synchronization
unsafe impl Send for MacOSInputBlocker {}
unsafe impl Sync for MacOSInputBlocker {}

/// macOS implementation of the `PowerManager` trait.
///
/// Wraps the existing `power.rs` functionality to prevent system sleep
/// using IOKit power assertions.
#[derive(Debug, Default)]
pub struct MacOSPowerManager;

impl MacOSPowerManager {
    /// Creates a new `MacOSPowerManager`.
    #[must_use]
    pub fn new() -> Self {
        Self
    }
}

impl PowerManager for MacOSPowerManager {
    fn prevent_sleep(&self) -> Result<SleepAssertion, PowerError> {
        prevent_sleep()
            .map(|id| SleepAssertion::new(u64::from(id)))
            .ok_or_else(|| {
                PowerError::AssertionFailed("Failed to create IOKit power assertion".to_string())
            })
    }

    fn allow_sleep(&self, assertion: SleepAssertion) -> Result<(), PowerError> {
        let id = assertion.id();
        if id > u64::from(u32::MAX) {
            return Err(PowerError::InvalidAssertion(format!(
                "Assertion ID {id} exceeds u32::MAX"
            )));
        }
        #[allow(clippy::cast_possible_truncation)]
        allow_sleep(id as u32);
        Ok(())
    }
}

/// macOS implementation of the `PermissionChecker` trait.
///
/// Wraps the existing `accessibility.rs` functionality to check and request
/// accessibility permissions using the Accessibility API.
#[derive(Debug, Default)]
pub struct MacOSPermissionChecker;

impl MacOSPermissionChecker {
    /// Creates a new `MacOSPermissionChecker`.
    #[must_use]
    pub fn new() -> Self {
        Self
    }
}

impl PermissionChecker for MacOSPermissionChecker {
    fn check_permissions(&self) -> bool {
        check_accessibility()
    }

    fn check_permissions_with_prompt(&self) -> bool {
        check_accessibility_with_prompt()
    }

    fn open_permissions_settings(&self) -> Result<(), PermissionError> {
        if open_accessibility_settings() {
            Ok(())
        } else {
            Err(PermissionError::SettingsFailed(
                "Failed to open System Settings to Accessibility pane".to_string(),
            ))
        }
    }
}

/// Combined macOS platform implementation.
///
/// Provides access to all macOS-specific platform functionality through
/// the trait interfaces.
#[derive(Debug, Default)]
pub struct MacOSPlatform {
    /// Input blocking via CGEventTap
    pub input_blocker: MacOSInputBlocker,
    /// Power management via IOKit assertions
    pub power_manager: MacOSPowerManager,
    /// Permission checking via Accessibility API
    pub permission_checker: MacOSPermissionChecker,
}

impl MacOSPlatform {
    /// Creates a new `MacOSPlatform` with default components.
    #[must_use]
    pub fn new() -> Self {
        Self {
            input_blocker: MacOSInputBlocker::new(),
            power_manager: MacOSPowerManager::new(),
            permission_checker: MacOSPermissionChecker::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_input_blocker_new() {
        let blocker = MacOSInputBlocker::new();
        assert!(!blocker.is_active());
    }

    #[test]
    fn test_input_blocker_default() {
        let blocker = MacOSInputBlocker::default();
        assert!(!blocker.is_active());
    }

    #[test]
    fn test_power_manager_new() {
        let _manager = MacOSPowerManager::new();
        // Just verify it can be constructed
    }

    #[test]
    fn test_power_manager_default() {
        let _manager: MacOSPowerManager = Default::default();
        // Just verify it can be constructed
    }

    #[test]
    fn test_permission_checker_new() {
        let _checker = MacOSPermissionChecker::new();
        // Just verify it can be constructed
    }

    #[test]
    fn test_permission_checker_default() {
        let _checker: MacOSPermissionChecker = Default::default();
        // Just verify it can be constructed
    }

    #[test]
    fn test_macos_platform_new() {
        let platform = MacOSPlatform::new();
        assert!(!platform.input_blocker.is_active());
    }

    #[test]
    fn test_macos_platform_default() {
        let platform = MacOSPlatform::default();
        assert!(!platform.input_blocker.is_active());
    }

    #[test]
    fn test_input_blocker_is_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<MacOSInputBlocker>();
    }

    #[test]
    fn test_power_manager_is_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<MacOSPowerManager>();
    }

    #[test]
    fn test_power_manager_invalid_assertion_id() {
        let manager = MacOSPowerManager::new();
        let invalid_assertion = SleepAssertion::new(u64::MAX);
        let result = manager.allow_sleep(invalid_assertion);
        assert!(result.is_err());
        if let Err(PowerError::InvalidAssertion(msg)) = result {
            assert!(msg.contains("exceeds u32::MAX"));
        } else {
            panic!("Expected InvalidAssertion error");
        }
    }
}
