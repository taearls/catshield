//! Linux power management for Cat Shield
//!
//! Provides functions to prevent and allow system sleep using the D-Bus
//! power management inhibit API.
//!
//! # How It Works
//!
//! Linux desktop environments provide D-Bus interfaces to inhibit system sleep
//! and screen blanking. This implementation tries multiple interfaces in order
//! of preference to support different desktop environments:
//!
//! 1. `org.freedesktop.ScreenSaver` - Standard FreeDesktop interface (most DEs)
//! 2. `org.gnome.SessionManager` - GNOME-specific interface
//! 3. `org.freedesktop.PowerManagement.Inhibit` - Legacy interface (older systems)
//!
//! Each interface returns a "cookie" (u32) that must be passed back to release
//! the inhibition.
//!
//! # Thread Safety
//!
//! The implementation uses `zbus` blocking API and tracks active assertions
//! using thread-safe atomic counters and a mutex-protected set.

use crate::platform::errors::PowerError;
use crate::platform::traits::PowerManager;
use crate::platform::types::SleepAssertion;
use std::collections::HashSet;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Mutex;
use zbus::blocking::Connection;

/// Counter for generating unique assertion IDs
static ASSERTION_COUNTER: AtomicU64 = AtomicU64::new(1);

/// Track valid assertion IDs to validate they were created by this manager
static VALID_ASSERTIONS: Mutex<Option<HashSet<u64>>> = Mutex::new(None);

/// Store the D-Bus cookie for each assertion ID
static DBUS_COOKIES: Mutex<Option<std::collections::HashMap<u64, DbusInhibitCookie>>> =
    Mutex::new(None);

/// Represents a D-Bus inhibit cookie and the interface it came from
#[derive(Debug, Clone)]
struct DbusInhibitCookie {
    /// The cookie value returned by the D-Bus interface
    cookie: u32,
    /// The interface that was used to create the inhibition
    interface: InhibitInterface,
}

/// The D-Bus interfaces we support for inhibiting sleep
#[derive(Debug, Clone, Copy, PartialEq)]
enum InhibitInterface {
    /// org.freedesktop.ScreenSaver - most desktop environments
    FreedesktopScreenSaver,
    /// org.gnome.SessionManager - GNOME
    GnomeSessionManager,
    /// org.freedesktop.PowerManagement.Inhibit - legacy systems
    FreedesktopPowerManagement,
}

impl InhibitInterface {
    /// Returns the D-Bus service name for this interface
    fn service(&self) -> &'static str {
        match self {
            Self::FreedesktopScreenSaver => "org.freedesktop.ScreenSaver",
            Self::GnomeSessionManager => "org.gnome.SessionManager",
            Self::FreedesktopPowerManagement => "org.freedesktop.PowerManagement.Inhibit",
        }
    }

    /// Returns the D-Bus object path for this interface
    fn path(&self) -> &'static str {
        match self {
            Self::FreedesktopScreenSaver => "/org/freedesktop/ScreenSaver",
            Self::GnomeSessionManager => "/org/gnome/SessionManager",
            Self::FreedesktopPowerManagement => "/org/freedesktop/PowerManagement/Inhibit",
        }
    }

    /// Returns the D-Bus interface name for this interface
    fn interface(&self) -> &'static str {
        match self {
            Self::FreedesktopScreenSaver => "org.freedesktop.ScreenSaver",
            Self::GnomeSessionManager => "org.gnome.SessionManager",
            Self::FreedesktopPowerManagement => "org.freedesktop.PowerManagement.Inhibit",
        }
    }
}

/// Linux implementation of the `PowerManager` trait.
///
/// Uses D-Bus to inhibit system sleep and screen blanking. Supports multiple
/// desktop environments by trying different D-Bus interfaces.
#[derive(Debug, Default)]
pub struct LinuxPowerManager;

impl LinuxPowerManager {
    /// Creates a new `LinuxPowerManager`.
    #[must_use]
    pub fn new() -> Self {
        Self
    }

    /// Attempts to create an inhibition using the FreeDesktop ScreenSaver interface.
    fn try_freedesktop_screensaver(conn: &Connection) -> Result<u32, PowerError> {
        let proxy = zbus::blocking::Proxy::new(
            conn,
            InhibitInterface::FreedesktopScreenSaver.service(),
            InhibitInterface::FreedesktopScreenSaver.path(),
            InhibitInterface::FreedesktopScreenSaver.interface(),
        )
        .map_err(|e| PowerError::Platform(format!("Failed to create D-Bus proxy: {e}")))?;

        // Call Inhibit(application_name: String, reason: String) -> cookie: u32
        let cookie: u32 = proxy
            .call_method("Inhibit", &("Cat Shield", "Protecting your work from cats"))
            .map_err(|e| PowerError::AssertionFailed(format!("FreeDesktop ScreenSaver: {e}")))?
            .body()
            .deserialize()
            .map_err(|e| PowerError::AssertionFailed(format!("Failed to parse cookie: {e}")))?;

        Ok(cookie)
    }

    /// Attempts to create an inhibition using the GNOME SessionManager interface.
    fn try_gnome_session_manager(conn: &Connection) -> Result<u32, PowerError> {
        let proxy = zbus::blocking::Proxy::new(
            conn,
            InhibitInterface::GnomeSessionManager.service(),
            InhibitInterface::GnomeSessionManager.path(),
            InhibitInterface::GnomeSessionManager.interface(),
        )
        .map_err(|e| PowerError::Platform(format!("Failed to create D-Bus proxy: {e}")))?;

        // GNOME SessionManager.Inhibit signature:
        // Inhibit(app_id: String, toplevel_xid: u32, reason: String, flags: u32) -> cookie: u32
        // Flags: 1 = logout, 2 = switch-user, 4 = suspend, 8 = idle
        // We want suspend (4) + idle (8) = 12
        let cookie: u32 = proxy
            .call_method(
                "Inhibit",
                &(
                    "cat-shield",                     // app_id
                    0u32,                             // toplevel_xid (0 = no window)
                    "Protecting your work from cats", // reason
                    12u32,                            // flags (suspend + idle)
                ),
            )
            .map_err(|e| PowerError::AssertionFailed(format!("GNOME SessionManager: {e}")))?
            .body()
            .deserialize()
            .map_err(|e| PowerError::AssertionFailed(format!("Failed to parse cookie: {e}")))?;

        Ok(cookie)
    }

    /// Attempts to create an inhibition using the legacy FreeDesktop PowerManagement interface.
    fn try_freedesktop_power_management(conn: &Connection) -> Result<u32, PowerError> {
        let proxy = zbus::blocking::Proxy::new(
            conn,
            InhibitInterface::FreedesktopPowerManagement.service(),
            InhibitInterface::FreedesktopPowerManagement.path(),
            InhibitInterface::FreedesktopPowerManagement.interface(),
        )
        .map_err(|e| PowerError::Platform(format!("Failed to create D-Bus proxy: {e}")))?;

        // Call Inhibit(application: String, reason: String) -> cookie: u32
        let cookie: u32 = proxy
            .call_method("Inhibit", &("Cat Shield", "Protecting your work from cats"))
            .map_err(|e| PowerError::AssertionFailed(format!("FreeDesktop PowerManagement: {e}")))?
            .body()
            .deserialize()
            .map_err(|e| PowerError::AssertionFailed(format!("Failed to parse cookie: {e}")))?;

        Ok(cookie)
    }

    /// Releases an inhibition using the FreeDesktop ScreenSaver interface.
    fn release_freedesktop_screensaver(conn: &Connection, cookie: u32) -> Result<(), PowerError> {
        let proxy = zbus::blocking::Proxy::new(
            conn,
            InhibitInterface::FreedesktopScreenSaver.service(),
            InhibitInterface::FreedesktopScreenSaver.path(),
            InhibitInterface::FreedesktopScreenSaver.interface(),
        )
        .map_err(|e| PowerError::Platform(format!("Failed to create D-Bus proxy: {e}")))?;

        proxy
            .call_method("UnInhibit", &(cookie,))
            .map_err(|e| PowerError::ReleaseFailed(format!("FreeDesktop ScreenSaver: {e}")))?;

        Ok(())
    }

    /// Releases an inhibition using the GNOME SessionManager interface.
    fn release_gnome_session_manager(conn: &Connection, cookie: u32) -> Result<(), PowerError> {
        let proxy = zbus::blocking::Proxy::new(
            conn,
            InhibitInterface::GnomeSessionManager.service(),
            InhibitInterface::GnomeSessionManager.path(),
            InhibitInterface::GnomeSessionManager.interface(),
        )
        .map_err(|e| PowerError::Platform(format!("Failed to create D-Bus proxy: {e}")))?;

        proxy
            .call_method("Uninhibit", &(cookie,))
            .map_err(|e| PowerError::ReleaseFailed(format!("GNOME SessionManager: {e}")))?;

        Ok(())
    }

    /// Releases an inhibition using the legacy FreeDesktop PowerManagement interface.
    fn release_freedesktop_power_management(
        conn: &Connection,
        cookie: u32,
    ) -> Result<(), PowerError> {
        let proxy = zbus::blocking::Proxy::new(
            conn,
            InhibitInterface::FreedesktopPowerManagement.service(),
            InhibitInterface::FreedesktopPowerManagement.path(),
            InhibitInterface::FreedesktopPowerManagement.interface(),
        )
        .map_err(|e| PowerError::Platform(format!("Failed to create D-Bus proxy: {e}")))?;

        proxy
            .call_method("UnInhibit", &(cookie,))
            .map_err(|e| PowerError::ReleaseFailed(format!("FreeDesktop PowerManagement: {e}")))?;

        Ok(())
    }
}

impl PowerManager for LinuxPowerManager {
    fn prevent_sleep(&self) -> Result<SleepAssertion, PowerError> {
        // Connect to the session bus
        let conn = Connection::session()
            .map_err(|e| PowerError::Platform(format!("Failed to connect to D-Bus: {e}")))?;

        // Try each interface in order of preference
        let (cookie, interface) = if let Ok(cookie) = Self::try_freedesktop_screensaver(&conn) {
            log::debug!("Using FreeDesktop ScreenSaver interface for sleep inhibition");
            (cookie, InhibitInterface::FreedesktopScreenSaver)
        } else if let Ok(cookie) = Self::try_gnome_session_manager(&conn) {
            log::debug!("Using GNOME SessionManager interface for sleep inhibition");
            (cookie, InhibitInterface::GnomeSessionManager)
        } else if let Ok(cookie) = Self::try_freedesktop_power_management(&conn) {
            log::debug!("Using FreeDesktop PowerManagement interface for sleep inhibition");
            (cookie, InhibitInterface::FreedesktopPowerManagement)
        } else {
            return Err(PowerError::AssertionFailed(
                "No compatible D-Bus power management interface found".to_string(),
            ));
        };

        // Generate a unique assertion ID
        let id = ASSERTION_COUNTER.fetch_add(1, Ordering::Relaxed);

        // Track this assertion ID as valid
        {
            let mut guard = VALID_ASSERTIONS.lock().map_err(|_| {
                PowerError::Platform("Failed to acquire assertion lock".to_string())
            })?;
            let set = guard.get_or_insert_with(HashSet::new);
            set.insert(id);
        }

        // Store the D-Bus cookie and interface for this assertion
        {
            let mut guard = DBUS_COOKIES.lock().map_err(|_| {
                // Rollback VALID_ASSERTIONS on failure
                if let Ok(mut va_guard) = VALID_ASSERTIONS.lock() {
                    if let Some(set) = va_guard.as_mut() {
                        set.remove(&id);
                    }
                }
                PowerError::Platform("Failed to acquire cookie lock".to_string())
            })?;
            let map = guard.get_or_insert_with(std::collections::HashMap::new);
            map.insert(id, DbusInhibitCookie { cookie, interface });
        }

        log::info!("✓ Sleep prevention enabled (Linux D-Bus)");

        Ok(SleepAssertion::new(id))
    }

    fn allow_sleep(&self, assertion: SleepAssertion) -> Result<(), PowerError> {
        let assertion_id = assertion.id();

        // Get and remove the D-Bus cookie for this assertion
        let dbus_cookie = {
            let mut guard = DBUS_COOKIES.lock().map_err(|_| {
                PowerError::InvalidAssertion("Failed to acquire D-Bus cookie lock".to_string())
            })?;

            if let Some(map) = guard.as_mut() {
                map.remove(&assertion_id).ok_or_else(|| {
                    PowerError::InvalidAssertion(format!(
                        "Assertion ID {} was not created by this manager or already released",
                        assertion_id
                    ))
                })?
            } else {
                return Err(PowerError::InvalidAssertion(
                    "No assertions have been created".to_string(),
                ));
            }
        };

        // Remove from valid assertions
        {
            let mut guard = VALID_ASSERTIONS.lock().map_err(|_| {
                PowerError::InvalidAssertion("Failed to acquire assertion lock".to_string())
            })?;

            if let Some(set) = guard.as_mut() {
                if !set.remove(&assertion_id) {
                    // Already removed above, but this is a sanity check
                    log::warn!(
                        "Assertion ID {} was not in valid assertions set",
                        assertion_id
                    );
                }
            }
        }

        // Connect to D-Bus and release the inhibition
        let conn = Connection::session()
            .map_err(|e| PowerError::Platform(format!("Failed to connect to D-Bus: {e}")))?;

        let result = match dbus_cookie.interface {
            InhibitInterface::FreedesktopScreenSaver => {
                Self::release_freedesktop_screensaver(&conn, dbus_cookie.cookie)
            }
            InhibitInterface::GnomeSessionManager => {
                Self::release_gnome_session_manager(&conn, dbus_cookie.cookie)
            }
            InhibitInterface::FreedesktopPowerManagement => {
                Self::release_freedesktop_power_management(&conn, dbus_cookie.cookie)
            }
        };

        if let Err(e) = &result {
            // Put the cookie back if release failed
            log::warn!("Failed to release D-Bus inhibition: {e}");
            if let Ok(mut guard) = DBUS_COOKIES.lock() {
                if let Some(map) = guard.as_mut() {
                    map.insert(assertion_id, dbus_cookie);
                }
            }
            if let Ok(mut guard) = VALID_ASSERTIONS.lock() {
                if let Some(set) = guard.as_mut() {
                    set.insert(assertion_id);
                }
            }
            return result;
        }

        log::info!("✓ Sleep prevention disabled (Linux D-Bus)");

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;

    #[test]
    fn test_linux_power_manager_new() {
        let _manager = LinuxPowerManager::new();
    }

    #[test]
    fn test_linux_power_manager_default() {
        let _manager: LinuxPowerManager = Default::default();
    }

    #[test]
    fn test_power_manager_is_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<LinuxPowerManager>();
    }

    #[test]
    #[serial]
    fn test_assertion_counter_increments() {
        let start = ASSERTION_COUNTER.load(Ordering::Relaxed);
        ASSERTION_COUNTER.fetch_add(1, Ordering::Relaxed);
        let end = ASSERTION_COUNTER.load(Ordering::Relaxed);
        assert!(end > start);
    }

    #[test]
    #[serial]
    fn test_allow_sleep_rejects_invalid_assertion_id() {
        let manager = LinuxPowerManager::new();
        // Create an assertion with an ID that was never created by prevent_sleep
        let fake_assertion = SleepAssertion::new(999_999_999);
        let result = manager.allow_sleep(fake_assertion);
        assert!(result.is_err());
        assert!(
            matches!(result, Err(PowerError::InvalidAssertion(_))),
            "Expected InvalidAssertion error, got: {:?}",
            result
        );
    }

    #[test]
    #[serial]
    fn test_valid_assertions_tracking() {
        if let Ok(mut guard) = VALID_ASSERTIONS.lock() {
            let set = guard.get_or_insert_with(HashSet::new);
            let test_id = 123_456_789u64;
            set.insert(test_id);
            assert!(set.contains(&test_id));
            set.remove(&test_id);
            assert!(!set.contains(&test_id));
        }
    }

    #[test]
    fn test_inhibit_interface_values() {
        assert_eq!(
            InhibitInterface::FreedesktopScreenSaver.service(),
            "org.freedesktop.ScreenSaver"
        );
        assert_eq!(
            InhibitInterface::FreedesktopScreenSaver.path(),
            "/org/freedesktop/ScreenSaver"
        );

        assert_eq!(
            InhibitInterface::GnomeSessionManager.service(),
            "org.gnome.SessionManager"
        );
        assert_eq!(
            InhibitInterface::GnomeSessionManager.path(),
            "/org/gnome/SessionManager"
        );

        assert_eq!(
            InhibitInterface::FreedesktopPowerManagement.service(),
            "org.freedesktop.PowerManagement.Inhibit"
        );
        assert_eq!(
            InhibitInterface::FreedesktopPowerManagement.path(),
            "/org/freedesktop/PowerManagement/Inhibit"
        );
    }

    #[test]
    #[serial]
    fn test_dbus_cookies_tracking() {
        if let Ok(mut guard) = DBUS_COOKIES.lock() {
            let map = guard.get_or_insert_with(std::collections::HashMap::new);
            let test_id = 987_654_321u64;
            let test_cookie = DbusInhibitCookie {
                cookie: 42,
                interface: InhibitInterface::FreedesktopScreenSaver,
            };
            map.insert(test_id, test_cookie);
            assert!(map.contains_key(&test_id));
            map.remove(&test_id);
            assert!(!map.contains_key(&test_id));
        }
    }
}
