//! Platform abstraction traits for Cat Shield
//!
//! These traits define the interface for platform-specific functionality,
//! enabling Cat Shield to support multiple operating systems (macOS, Windows, Linux)
//! with a consistent API.
//!
//! # Architecture
//!
//! Each platform (macOS, Windows, Linux) will implement these traits with
//! platform-specific code, while the core application logic uses the trait
//! interfaces without needing to know which platform is active.
//!
//! # Traits
//!
//! - [`InputBlocker`] - Intercepts and blocks keyboard input
//! - [`PowerManager`] - Prevents system sleep during protection
//! - [`PermissionChecker`] - Handles platform permission requirements
//! - [`SystemTray`] - Manages the menu bar / system tray icon
//! - [`OverlayWindow`] - Creates and manages the fullscreen overlay

use super::errors::{InputBlockError, PermissionError, PowerError, TrayError, WindowError};
use super::types::{Rect, SleepAssertion};

/// Trait for blocking keyboard and mouse input.
///
/// Platform implementations:
/// - **macOS**: Uses `CGEventTap` to intercept events at the HID level
/// - **Windows**: Uses low-level keyboard hooks (`SetWindowsHookEx`)
/// - **Linux**: Uses X11 keyboard grab or Wayland input inhibitor
///
/// # Thread Safety
///
/// Implementations must be `Send + Sync` as the input blocker may be
/// controlled from different threads (e.g., UI thread vs. event thread).
///
/// # Example
///
/// ```ignore
/// let mut blocker = MacOSInputBlocker::new();
/// blocker.setup()?;
/// // ... input is now blocked ...
/// blocker.disable();
/// ```
pub trait InputBlocker: Send + Sync {
    /// Sets up and enables input blocking.
    ///
    /// This method should:
    /// 1. Create the platform-specific event interception mechanism
    /// 2. Register it with the system's event loop
    /// 3. Begin blocking keyboard events
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The event tap/hook cannot be created
    /// - Required permissions are not granted
    /// - The system rejects the input blocking request
    fn setup(&mut self) -> Result<(), InputBlockError>;

    /// Disables input blocking and cleans up resources.
    ///
    /// This method should:
    /// 1. Stop intercepting events
    /// 2. Unregister from the system's event loop
    /// 3. Release any held resources
    ///
    /// This method should be idempotent - calling it multiple times
    /// should be safe and have no additional effect.
    fn disable(&mut self);

    /// Returns whether input blocking is currently active.
    fn is_active(&self) -> bool;
}

/// Trait for preventing system sleep during protection.
///
/// Platform implementations:
/// - **macOS**: Uses IOKit power assertions (`IOPMAssertionCreateWithName`)
/// - **Windows**: Uses `SetThreadExecutionState` or power request API
/// - **Linux**: Uses D-Bus inhibit interface (GNOME, KDE, etc.)
///
/// # Thread Safety
///
/// Implementations must be `Send + Sync` as power management may be
/// accessed from different threads.
pub trait PowerManager: Send + Sync {
    /// Creates an assertion to prevent the system from sleeping.
    ///
    /// Returns a `SleepAssertion` handle that must be passed to
    /// `allow_sleep()` to release the assertion.
    ///
    /// # Errors
    ///
    /// Returns an error if the assertion cannot be created.
    fn prevent_sleep(&self) -> Result<SleepAssertion, PowerError>;

    /// Releases a previously created sleep prevention assertion.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The assertion handle is invalid
    /// - The assertion has already been released
    /// - The system refuses to release the assertion
    fn allow_sleep(&self, assertion: SleepAssertion) -> Result<(), PowerError>;
}

/// Trait for checking and requesting platform-specific permissions.
///
/// Platform implementations:
/// - **macOS**: Handles Accessibility permissions via `AXIsProcessTrusted`
/// - **Windows**: May handle UAC elevation or admin rights
/// - **Linux**: May handle polkit authorization or capability checks
///
/// # Note
///
/// Some platforms (like Windows) may not require explicit permission
/// grants for the functionality Cat Shield needs. In such cases,
/// implementations can return `true` from `check_permissions()`.
pub trait PermissionChecker {
    /// Checks if the required permissions are currently granted.
    ///
    /// Returns `true` if all necessary permissions are available,
    /// `false` otherwise.
    fn check_permissions(&self) -> bool;

    /// Checks permissions and prompts the user if not granted.
    ///
    /// On platforms that support it, this will display a system dialog
    /// asking the user to grant the required permissions.
    ///
    /// Returns `true` if permissions are granted (either already or
    /// after user approval), `false` if the user denied or the prompt
    /// failed.
    fn check_permissions_with_prompt(&self) -> bool;

    /// Opens the system settings to the permissions page.
    ///
    /// This allows users to manually grant permissions if the automatic
    /// prompt doesn't work or if they previously denied access.
    ///
    /// # Errors
    ///
    /// Returns an error if the settings cannot be opened.
    fn open_permissions_settings(&self) -> Result<(), PermissionError>;
}

/// Trait for managing the system tray / menu bar icon.
///
/// Platform implementations:
/// - **macOS**: Uses `NSStatusItem` in the menu bar
/// - **Windows**: Uses Shell_NotifyIcon for the system tray
/// - **Linux**: Uses libappindicator or platform-specific tray protocols
///
/// # Menu Structure
///
/// The tray icon should provide a menu with at least:
/// - Start/Stop Protection toggle
/// - Settings access
/// - About dialog
/// - Quit option
pub trait SystemTray {
    /// Sets up the system tray icon and menu.
    ///
    /// # Errors
    ///
    /// Returns an error if the tray icon cannot be created.
    fn setup(&mut self) -> Result<(), TrayError>;

    /// Updates the tray icon to reflect active/inactive state.
    ///
    /// When `active` is true, the icon should indicate that
    /// protection is currently running.
    fn set_state(&mut self, active: bool);

    /// Removes the tray icon from the system.
    ///
    /// This should be called during application shutdown.
    fn remove(&mut self);

    /// Returns whether the tray is currently visible.
    fn is_visible(&self) -> bool;
}

/// Trait for creating and managing the fullscreen overlay window.
///
/// Platform implementations:
/// - **macOS**: Uses `NSWindow` with appropriate level and style
/// - **Windows**: Uses `CreateWindowEx` with layered window styles
/// - **Linux**: Uses X11 override-redirect or Wayland layer shell
///
/// # Requirements
///
/// The overlay window must:
/// - Cover all displays (multi-monitor support)
/// - Be topmost (above all other windows)
/// - Be semi-transparent
/// - Capture mouse events (except for the close button)
pub trait OverlayWindow {
    /// Creates the overlay window covering all displays.
    ///
    /// The window is created but not shown. Call `show()` to make it visible.
    ///
    /// # Errors
    ///
    /// Returns an error if the window cannot be created.
    fn create(&mut self) -> Result<(), WindowError>;

    /// Shows the overlay window.
    fn show(&mut self);

    /// Hides the overlay window.
    fn hide(&mut self);

    /// Sets the opacity of the overlay.
    ///
    /// # Arguments
    ///
    /// * `opacity` - A value between 0.0 (fully transparent) and 1.0 (fully opaque).
    ///   Values outside this range will be clamped.
    fn set_opacity(&mut self, opacity: f64);

    /// Returns the current bounds of the overlay window.
    fn bounds(&self) -> Rect;

    /// Closes and destroys the overlay window.
    fn close(&mut self);

    /// Returns whether the window is currently visible.
    fn is_visible(&self) -> bool;
}

#[cfg(test)]
mod tests {
    use super::*;

    // Mock implementations for testing trait definitions

    struct MockInputBlocker {
        active: bool,
    }

    impl MockInputBlocker {
        fn new() -> Self {
            Self { active: false }
        }
    }

    impl InputBlocker for MockInputBlocker {
        fn setup(&mut self) -> Result<(), InputBlockError> {
            self.active = true;
            Ok(())
        }

        fn disable(&mut self) {
            self.active = false;
        }

        fn is_active(&self) -> bool {
            self.active
        }
    }

    struct MockPowerManager;

    impl PowerManager for MockPowerManager {
        fn prevent_sleep(&self) -> Result<SleepAssertion, PowerError> {
            Ok(SleepAssertion::new(1))
        }

        fn allow_sleep(&self, _assertion: SleepAssertion) -> Result<(), PowerError> {
            Ok(())
        }
    }

    struct MockPermissionChecker {
        granted: bool,
    }

    impl PermissionChecker for MockPermissionChecker {
        fn check_permissions(&self) -> bool {
            self.granted
        }

        fn check_permissions_with_prompt(&self) -> bool {
            self.granted
        }

        fn open_permissions_settings(&self) -> Result<(), PermissionError> {
            Ok(())
        }
    }

    struct MockSystemTray {
        visible: bool,
        active: bool,
    }

    impl MockSystemTray {
        fn new() -> Self {
            Self {
                visible: false,
                active: false,
            }
        }
    }

    impl SystemTray for MockSystemTray {
        fn setup(&mut self) -> Result<(), TrayError> {
            self.visible = true;
            Ok(())
        }

        fn set_state(&mut self, active: bool) {
            self.active = active;
        }

        fn remove(&mut self) {
            self.visible = false;
        }

        fn is_visible(&self) -> bool {
            self.visible
        }
    }

    struct MockOverlayWindow {
        visible: bool,
        opacity: f64,
        bounds: Rect,
    }

    impl MockOverlayWindow {
        fn new() -> Self {
            Self {
                visible: false,
                opacity: 0.5,
                bounds: Rect::zero(),
            }
        }
    }

    impl OverlayWindow for MockOverlayWindow {
        fn create(&mut self) -> Result<(), WindowError> {
            self.bounds = Rect::new(0.0, 0.0, 1920.0, 1080.0);
            Ok(())
        }

        fn show(&mut self) {
            self.visible = true;
        }

        fn hide(&mut self) {
            self.visible = false;
        }

        fn set_opacity(&mut self, opacity: f64) {
            self.opacity = opacity.clamp(0.0, 1.0);
        }

        fn bounds(&self) -> Rect {
            self.bounds
        }

        fn close(&mut self) {
            self.visible = false;
            self.bounds = Rect::zero();
        }

        fn is_visible(&self) -> bool {
            self.visible
        }
    }

    #[test]
    fn test_input_blocker_trait() {
        let mut blocker = MockInputBlocker::new();
        assert!(!blocker.is_active());

        blocker.setup().unwrap();
        assert!(blocker.is_active());

        blocker.disable();
        assert!(!blocker.is_active());
    }

    #[test]
    fn test_power_manager_trait() {
        let manager = MockPowerManager;

        let assertion = manager.prevent_sleep().unwrap();
        assert_eq!(assertion.id(), 1);

        manager.allow_sleep(assertion).unwrap();
    }

    #[test]
    fn test_permission_checker_trait() {
        let checker_granted = MockPermissionChecker { granted: true };
        let checker_denied = MockPermissionChecker { granted: false };

        assert!(checker_granted.check_permissions());
        assert!(!checker_denied.check_permissions());

        assert!(checker_granted.check_permissions_with_prompt());
        assert!(!checker_denied.check_permissions_with_prompt());

        checker_granted.open_permissions_settings().unwrap();
    }

    #[test]
    fn test_system_tray_trait() {
        let mut tray = MockSystemTray::new();
        assert!(!tray.is_visible());

        tray.setup().unwrap();
        assert!(tray.is_visible());

        tray.set_state(true);
        assert!(tray.active);

        tray.remove();
        assert!(!tray.is_visible());
    }

    #[test]
    fn test_overlay_window_trait() {
        let mut window = MockOverlayWindow::new();
        assert!(!window.is_visible());
        assert!(window.bounds().is_empty());

        window.create().unwrap();
        assert!(!window.is_visible()); // Created but not shown
        assert!(!window.bounds().is_empty());

        window.show();
        assert!(window.is_visible());

        window.set_opacity(0.8);
        assert_eq!(window.opacity, 0.8);

        // Test opacity clamping
        window.set_opacity(1.5);
        assert_eq!(window.opacity, 1.0);

        window.set_opacity(-0.5);
        assert_eq!(window.opacity, 0.0);

        window.hide();
        assert!(!window.is_visible());

        window.close();
        assert!(window.bounds().is_empty());
    }

    #[test]
    fn test_input_blocker_is_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<MockInputBlocker>();
    }

    #[test]
    fn test_power_manager_is_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<MockPowerManager>();
    }
}
