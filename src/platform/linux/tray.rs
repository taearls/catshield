//! Linux system tray implementation for Cat Shield
//!
//! This module implements the [`SystemTray`] trait for Linux using:
//! - `ksni` crate for StatusNotifierItem (SNI) protocol
//! - D-Bus for desktop integration
//!
//! # StatusNotifierItem Protocol
//!
//! SNI is the modern replacement for XEmbed system trays and is supported by:
//! - KDE Plasma (native support)
//! - GNOME (via AppIndicator extension)
//! - XFCE (via plugin)
//! - Most modern Linux desktop environments
//!
//! # Menu Structure
//!
//! The tray menu provides:
//! - Start Protection / Stop Protection (toggle based on state)
//! - Settings...
//! - About Cat Shield
//! - Quit
//!
//! # Implementation Notes
//!
//! The `ksni` crate runs its own D-Bus event loop in a background thread.
//! Menu actions are communicated back via channels.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};

use ksni::{menu::StandardItem, Icon, MenuItem, Tray, TrayService};
use log::{debug, info, warn};

use crate::platform::errors::TrayError;
use crate::platform::traits::SystemTray;

/// Global state for whether the shield is active (used by menu callback)
static SHIELD_ACTIVE: AtomicBool = AtomicBool::new(false);

/// Menu action types that can be triggered by the tray menu
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrayMenuAction {
    /// User clicked Start Protection
    StartProtection,
    /// User clicked Stop Protection
    StopProtection,
    /// User clicked Settings
    OpenSettings,
    /// User clicked About
    ShowAbout,
    /// User clicked Quit
    Quit,
}

/// Callback type for handling menu actions
pub type MenuCallback = Box<dyn Fn(TrayMenuAction) + Send + Sync>;

/// Global callback for menu actions
static MENU_CALLBACK: Mutex<Option<Arc<MenuCallback>>> = Mutex::new(None);

/// Helper to invoke the menu callback
fn invoke_callback(action: TrayMenuAction) {
    match MENU_CALLBACK.lock() {
        Ok(guard) => {
            if let Some(callback) = guard.as_ref() {
                callback(action);
            }
        }
        Err(e) => {
            warn!("Menu callback mutex poisoned: {}", e);
        }
    }
}

/// The Cat Shield tray icon implementation for ksni
struct CatShieldTray {
    /// Whether the shield is currently active
    active: bool,
}

impl Tray for CatShieldTray {
    fn id(&self) -> String {
        "cat-shield".to_string()
    }

    fn title(&self) -> String {
        if self.active {
            "Cat Shield - Active".to_string()
        } else {
            "Cat Shield".to_string()
        }
    }

    fn icon_name(&self) -> String {
        // Use a standard icon that's available on most systems
        // The "security-high" icon is part of the freedesktop icon spec
        "security-high".to_string()
    }

    fn icon_pixmap(&self) -> Vec<Icon> {
        // Return empty - we'll use icon_name instead
        // This could be extended to provide a custom icon
        Vec::new()
    }

    fn tool_tip(&self) -> ksni::ToolTip {
        let description = if self.active {
            "Cat Shield is protecting your screen"
        } else {
            "Cat Shield - Screen protection utility"
        };

        ksni::ToolTip {
            title: self.title(),
            description: description.to_string(),
            icon_name: self.icon_name(),
            icon_pixmap: Vec::new(),
        }
    }

    fn menu(&self) -> Vec<MenuItem<Self>> {
        // Use self.active as the single source of truth for UI consistency
        // (title(), tool_tip(), and menu() all read from self.active)
        let active = self.active;

        let mut items = Vec::new();

        // Start Protection item
        if active {
            items.push(MenuItem::Standard(StandardItem {
                label: "Start Protection".to_string(),
                enabled: false,
                ..Default::default()
            }));
        } else {
            items.push(MenuItem::Standard(StandardItem {
                label: "Start Protection".to_string(),
                enabled: true,
                activate: Box::new(|_tray: &mut Self| {
                    info!("Menu: Start Protection selected");
                    invoke_callback(TrayMenuAction::StartProtection);
                }),
                ..Default::default()
            }));
        }

        // Stop Protection item
        if active {
            items.push(MenuItem::Standard(StandardItem {
                label: "Stop Protection".to_string(),
                enabled: true,
                activate: Box::new(|_tray: &mut Self| {
                    info!("Menu: Stop Protection selected");
                    invoke_callback(TrayMenuAction::StopProtection);
                }),
                ..Default::default()
            }));
        } else {
            items.push(MenuItem::Standard(StandardItem {
                label: "Stop Protection".to_string(),
                enabled: false,
                ..Default::default()
            }));
        }

        // Separator
        items.push(MenuItem::Separator);

        // Settings
        items.push(MenuItem::Standard(StandardItem {
            label: "Settings...".to_string(),
            enabled: true,
            activate: Box::new(|_tray: &mut Self| {
                info!("Menu: Settings selected");
                invoke_callback(TrayMenuAction::OpenSettings);
            }),
            ..Default::default()
        }));

        // About
        items.push(MenuItem::Standard(StandardItem {
            label: "About Cat Shield".to_string(),
            enabled: true,
            activate: Box::new(|_tray: &mut Self| {
                info!("Menu: About selected");
                invoke_callback(TrayMenuAction::ShowAbout);
            }),
            ..Default::default()
        }));

        // Separator
        items.push(MenuItem::Separator);

        // Quit
        items.push(MenuItem::Standard(StandardItem {
            label: "Quit".to_string(),
            enabled: true,
            activate: Box::new(|_tray: &mut Self| {
                info!("Menu: Quit selected");
                invoke_callback(TrayMenuAction::Quit);
            }),
            ..Default::default()
        }));

        items
    }
}

/// Linux system tray implementation using StatusNotifierItem.
///
/// This struct manages the system tray icon and context menu on Linux.
/// It implements the [`SystemTray`] trait for platform abstraction.
///
/// # Example
///
/// ```ignore
/// let mut tray = LinuxSystemTray::new();
/// tray.setup()?;
/// tray.set_state(true); // Shield is active
/// // ... later ...
/// tray.remove();
/// ```
pub struct LinuxSystemTray {
    /// Handle to the tray service for controlling the tray
    handle: Option<ksni::Handle<CatShieldTray>>,
    /// The background thread running the tray service
    service_thread: Option<JoinHandle<()>>,
    /// Whether the tray icon is currently visible
    visible: bool,
    /// Whether the shield is currently active
    active: bool,
}

impl LinuxSystemTray {
    /// Creates a new Linux system tray instance.
    ///
    /// The tray icon is not created until [`setup()`](Self::setup) is called.
    pub fn new() -> Self {
        Self {
            handle: None,
            service_thread: None,
            visible: false,
            active: false,
        }
    }

    /// Sets the callback function for menu actions.
    ///
    /// The callback will be invoked when the user selects a menu item.
    /// This must be called before [`setup()`](Self::setup) for the callback
    /// to receive events.
    ///
    /// # Arguments
    ///
    /// * `callback` - A function that receives [`TrayMenuAction`] events
    pub fn set_callback(callback: MenuCallback) {
        if let Ok(mut guard) = MENU_CALLBACK.lock() {
            *guard = Some(Arc::new(callback));
        }
    }

    /// Clears the menu callback.
    pub fn clear_callback() {
        if let Ok(mut guard) = MENU_CALLBACK.lock() {
            *guard = None;
        }
    }
}

impl Default for LinuxSystemTray {
    fn default() -> Self {
        Self::new()
    }
}

impl SystemTray for LinuxSystemTray {
    fn setup(&mut self) -> Result<(), TrayError> {
        if self.visible {
            debug!("Tray already set up");
            return Ok(());
        }

        let tray = CatShieldTray {
            active: self.active,
        };

        // Create the tray service
        let service = TrayService::new(tray);

        // Get the handle for controlling the tray
        let handle = service.handle();
        self.handle = Some(handle);

        // Run the service in a background thread
        let join_handle = thread::spawn(move || {
            debug!("Starting tray service thread");
            if let Err(e) = service.run() {
                warn!("Tray service exited with error: {}", e);
            }
            debug!("Tray service thread exited");
        });

        self.service_thread = Some(join_handle);
        self.visible = true;

        info!("Created Linux system tray icon (StatusNotifierItem)");
        Ok(())
    }

    fn set_state(&mut self, active: bool) {
        self.active = active;
        SHIELD_ACTIVE.store(active, Ordering::SeqCst);

        // Update the tray to reflect the new state
        if let Some(handle) = &self.handle {
            handle.update(|tray| {
                tray.active = active;
            });
        }

        debug!("Tray state updated: active={}", active);
    }

    fn remove(&mut self) {
        if !self.visible {
            return;
        }

        // Shutdown the tray service
        if let Some(handle) = self.handle.take() {
            handle.shutdown();
        }

        // Wait for the service thread to finish
        if let Some(thread) = self.service_thread.take() {
            let _ = thread.join();
        }

        self.visible = false;
        info!("Removed Linux system tray icon");
    }

    fn is_visible(&self) -> bool {
        self.visible
    }
}

impl Drop for LinuxSystemTray {
    fn drop(&mut self) {
        self.remove();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tray_menu_action_variants() {
        assert_eq!(
            TrayMenuAction::StartProtection,
            TrayMenuAction::StartProtection
        );
        assert_ne!(
            TrayMenuAction::StartProtection,
            TrayMenuAction::StopProtection
        );
        assert_ne!(TrayMenuAction::OpenSettings, TrayMenuAction::ShowAbout);
        assert_ne!(TrayMenuAction::Quit, TrayMenuAction::StartProtection);
    }

    #[test]
    fn test_tray_menu_action_debug() {
        let action = TrayMenuAction::StartProtection;
        let debug_str = format!("{:?}", action);
        assert!(debug_str.contains("StartProtection"));
    }

    #[test]
    fn test_linux_system_tray_new() {
        let tray = LinuxSystemTray::new();
        assert!(!tray.visible);
        assert!(!tray.active);
        assert!(tray.handle.is_none());
        assert!(tray.service_thread.is_none());
    }

    #[test]
    fn test_linux_system_tray_default() {
        let tray = LinuxSystemTray::default();
        assert!(!tray.visible);
        assert!(!tray.active);
    }

    #[test]
    fn test_shield_active_atomic() {
        SHIELD_ACTIVE.store(true, Ordering::SeqCst);
        assert!(SHIELD_ACTIVE.load(Ordering::SeqCst));
        SHIELD_ACTIVE.store(false, Ordering::SeqCst);
        assert!(!SHIELD_ACTIVE.load(Ordering::SeqCst));
    }

    #[test]
    fn test_cat_shield_tray_id() {
        let tray = CatShieldTray { active: false };
        assert_eq!(tray.id(), "cat-shield");
    }

    #[test]
    fn test_cat_shield_tray_title() {
        let inactive_tray = CatShieldTray { active: false };
        assert_eq!(inactive_tray.title(), "Cat Shield");

        let active_tray = CatShieldTray { active: true };
        assert_eq!(active_tray.title(), "Cat Shield - Active");
    }

    #[test]
    fn test_cat_shield_tray_icon_name() {
        let tray = CatShieldTray { active: false };
        assert_eq!(tray.icon_name(), "security-high");
    }

    #[test]
    fn test_cat_shield_tray_tooltip() {
        let inactive_tray = CatShieldTray { active: false };
        let tooltip = inactive_tray.tool_tip();
        assert_eq!(tooltip.title, "Cat Shield");
        assert!(tooltip.description.contains("utility"));

        let active_tray = CatShieldTray { active: true };
        let tooltip = active_tray.tool_tip();
        assert_eq!(tooltip.title, "Cat Shield - Active");
        assert!(tooltip.description.contains("protecting"));
    }
}
