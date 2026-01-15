//! Windows system tray implementation for Cat Shield
//!
//! This module implements the [`SystemTray`] trait for Windows using:
//! - `Shell_NotifyIcon` with `NOTIFYICONDATA` for the tray icon
//! - `TrackPopupMenu` for the context menu
//! - Custom window messages (`WM_TRAYICON`) for click events
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
//! Windows system tray requires a hidden window to receive messages.
//! The tray icon is managed via `Shell_NotifyIcon` with `NIM_ADD`,
//! `NIM_MODIFY`, and `NIM_DELETE` operations.

use std::sync::atomic::{AtomicBool, AtomicPtr, Ordering};

use log::{debug, error, info, warn};
use windows::core::{w, PCWSTR};
use windows::Win32::Foundation::{HINSTANCE, HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::Shell::{
    Shell_NotifyIconW, NIF_ICON, NIF_MESSAGE, NIF_TIP, NIM_ADD, NIM_DELETE, NIM_MODIFY,
    NOTIFYICONDATAW,
};
use windows::Win32::UI::WindowsAndMessaging::{
    AppendMenuW, CreatePopupMenu, CreateWindowExW, DefWindowProcW, DestroyMenu, DestroyWindow,
    GetCursorPos, LoadIconW, PostQuitMessage, RegisterClassW, SetForegroundWindow, TrackPopupMenu,
    IDI_APPLICATION, MF_ENABLED, MF_GRAYED, MF_SEPARATOR, MF_STRING, TPM_BOTTOMALIGN,
    TPM_LEFTALIGN, TPM_RIGHTBUTTON, WINDOW_EX_STYLE, WM_COMMAND, WM_DESTROY, WM_LBUTTONUP,
    WM_RBUTTONUP, WNDCLASSW, WS_OVERLAPPEDWINDOW,
};

use crate::platform::errors::TrayError;
use crate::platform::traits::SystemTray;

/// Custom message ID for tray icon events
const WM_TRAYICON: u32 = 0x0400 + 1; // WM_USER + 1

/// Menu item IDs
const ID_MENU_START: u16 = 1001;
const ID_MENU_STOP: u16 = 1002;
const ID_MENU_SETTINGS: u16 = 1003;
const ID_MENU_ABOUT: u16 = 1004;
const ID_MENU_QUIT: u16 = 1005;

/// Window class name for the hidden tray window
static WINDOW_CLASS_NAME: &[u16] = &[
    'C' as u16, 'a' as u16, 't' as u16, 'S' as u16, 'h' as u16, 'i' as u16, 'e' as u16, 'l' as u16,
    'd' as u16, 'T' as u16, 'r' as u16, 'a' as u16, 'y' as u16, 0,
];

/// Global state for whether the shield is active (used by menu callback)
static SHIELD_ACTIVE: AtomicBool = AtomicBool::new(false);

/// Global callback for menu actions
static MENU_CALLBACK: AtomicPtr<()> = AtomicPtr::new(std::ptr::null_mut());

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

/// Windows system tray implementation.
///
/// This struct manages the system tray icon and context menu on Windows.
/// It implements the [`SystemTray`] trait for platform abstraction.
///
/// # Example
///
/// ```ignore
/// let mut tray = WindowsSystemTray::new();
/// tray.setup()?;
/// tray.set_state(true); // Shield is active
/// // ... later ...
/// tray.remove();
/// ```
pub struct WindowsSystemTray {
    /// Handle to the hidden message window
    hwnd: HWND,
    /// Whether the tray icon is currently visible
    visible: bool,
    /// Whether the shield is currently active
    active: bool,
    /// The NOTIFYICONDATA structure for the tray icon
    nid: NOTIFYICONDATAW,
    /// Whether the window class has been registered
    class_registered: bool,
}

impl WindowsSystemTray {
    /// Creates a new Windows system tray instance.
    ///
    /// The tray icon is not created until [`setup()`](Self::setup) is called.
    pub fn new() -> Self {
        Self {
            hwnd: HWND::default(),
            visible: false,
            active: false,
            nid: NOTIFYICONDATAW::default(),
            class_registered: false,
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
        let boxed = Box::new(callback);
        let ptr = Box::into_raw(boxed) as *mut ();
        let old = MENU_CALLBACK.swap(ptr, Ordering::SeqCst);
        if !old.is_null() {
            // Clean up the old callback
            unsafe {
                drop(Box::from_raw(old as *mut MenuCallback));
            }
        }
    }

    /// Clears the menu callback.
    pub fn clear_callback() {
        let old = MENU_CALLBACK.swap(std::ptr::null_mut(), Ordering::SeqCst);
        if !old.is_null() {
            unsafe {
                drop(Box::from_raw(old as *mut MenuCallback));
            }
        }
    }

    /// Registers the window class for the hidden tray window.
    fn register_window_class(&mut self) -> Result<(), TrayError> {
        if self.class_registered {
            return Ok(());
        }

        let hinstance = unsafe { GetModuleHandleW(None) }
            .map_err(|e| TrayError::CreationFailed(format!("Failed to get module handle: {e}")))?;

        let wc = WNDCLASSW {
            lpfnWndProc: Some(tray_window_proc),
            hInstance: hinstance.into(),
            lpszClassName: PCWSTR(WINDOW_CLASS_NAME.as_ptr()),
            ..Default::default()
        };

        let result = unsafe { RegisterClassW(&wc) };
        if result == 0 {
            // Class might already be registered, which is OK
            debug!("Window class registration returned 0 (may already be registered)");
        }

        self.class_registered = true;
        Ok(())
    }

    /// Creates the hidden window for receiving tray messages.
    fn create_hidden_window(&mut self) -> Result<(), TrayError> {
        let hmodule = unsafe { GetModuleHandleW(None) }
            .map_err(|e| TrayError::CreationFailed(format!("Failed to get module handle: {e}")))?;
        let hinstance: HINSTANCE = hmodule.into();

        let hwnd = unsafe {
            CreateWindowExW(
                WINDOW_EX_STYLE::default(),
                PCWSTR(WINDOW_CLASS_NAME.as_ptr()),
                w!("CatShield Tray"),
                WS_OVERLAPPEDWINDOW,
                0,
                0,
                0,
                0,
                None,
                None,
                Some(hinstance),
                None,
            )
        }
        .map_err(|e| TrayError::CreationFailed(format!("Failed to create hidden window: {e}")))?;

        if hwnd.is_invalid() {
            return Err(TrayError::CreationFailed(
                "CreateWindowExW returned invalid handle".into(),
            ));
        }

        self.hwnd = hwnd;
        debug!("Created hidden tray window: {:?}", hwnd);
        Ok(())
    }

    /// Creates the tray icon.
    fn create_tray_icon(&mut self) -> Result<(), TrayError> {
        // Load the default application icon
        // Note: For predefined system icons like IDI_APPLICATION, hInstance must be None
        let hicon = unsafe { LoadIconW(None, IDI_APPLICATION) }
            .map_err(|e| TrayError::IconFailed(format!("Failed to load icon: {e}")))?;

        // Initialize NOTIFYICONDATAW
        self.nid = NOTIFYICONDATAW {
            cbSize: std::mem::size_of::<NOTIFYICONDATAW>() as u32,
            hWnd: self.hwnd,
            uID: 1,
            uFlags: NIF_ICON | NIF_MESSAGE | NIF_TIP,
            uCallbackMessage: WM_TRAYICON,
            hIcon: hicon,
            ..Default::default()
        };

        // Set the tooltip text: "Cat Shield"
        let tooltip: &[u16] = &[
            'C' as u16, 'a' as u16, 't' as u16, ' ' as u16, 'S' as u16, 'h' as u16, 'i' as u16,
            'e' as u16, 'l' as u16, 'd' as u16, 0,
        ];
        let tip_len = tooltip.len().min(128);
        self.nid.szTip[..tip_len].copy_from_slice(&tooltip[..tip_len]);

        // Add the tray icon
        let result = unsafe { Shell_NotifyIconW(NIM_ADD, &self.nid) };
        if !result.as_bool() {
            return Err(TrayError::CreationFailed(
                "Shell_NotifyIconW(NIM_ADD) failed".into(),
            ));
        }

        info!("Created Windows system tray icon");
        Ok(())
    }

    /// Shows the context menu at the current cursor position.
    fn show_context_menu(&self) {
        let hmenu = unsafe { CreatePopupMenu() };
        if let Err(e) = hmenu {
            error!("Failed to create popup menu: {}", e);
            return;
        }
        let hmenu = hmenu.unwrap();

        // Build the menu based on shield state
        unsafe {
            if self.active {
                // Shield is active, show Stop Protection (grayed Start)
                let _ = AppendMenuW(
                    hmenu,
                    MF_STRING | MF_GRAYED,
                    ID_MENU_START as usize,
                    w!("Start Protection"),
                );
                let _ = AppendMenuW(
                    hmenu,
                    MF_STRING | MF_ENABLED,
                    ID_MENU_STOP as usize,
                    w!("Stop Protection"),
                );
            } else {
                // Shield is inactive, show Start Protection (grayed Stop)
                let _ = AppendMenuW(
                    hmenu,
                    MF_STRING | MF_ENABLED,
                    ID_MENU_START as usize,
                    w!("Start Protection"),
                );
                let _ = AppendMenuW(
                    hmenu,
                    MF_STRING | MF_GRAYED,
                    ID_MENU_STOP as usize,
                    w!("Stop Protection"),
                );
            }

            let _ = AppendMenuW(hmenu, MF_SEPARATOR, 0, PCWSTR::null());
            let _ = AppendMenuW(
                hmenu,
                MF_STRING | MF_ENABLED,
                ID_MENU_SETTINGS as usize,
                w!("Settings..."),
            );
            let _ = AppendMenuW(
                hmenu,
                MF_STRING | MF_ENABLED,
                ID_MENU_ABOUT as usize,
                w!("About Cat Shield"),
            );
            let _ = AppendMenuW(hmenu, MF_SEPARATOR, 0, PCWSTR::null());
            let _ = AppendMenuW(
                hmenu,
                MF_STRING | MF_ENABLED,
                ID_MENU_QUIT as usize,
                w!("Quit"),
            );
        }

        // Get cursor position
        let mut pt = windows::Win32::Foundation::POINT::default();
        let _ = unsafe { GetCursorPos(&mut pt) };

        // Required for proper menu dismissal
        unsafe {
            let _ = SetForegroundWindow(self.hwnd);
        }

        // Show the menu
        unsafe {
            let _ = TrackPopupMenu(
                hmenu,
                TPM_RIGHTBUTTON | TPM_BOTTOMALIGN | TPM_LEFTALIGN,
                pt.x,
                pt.y,
                0,
                self.hwnd,
                None,
            );
        }

        // Clean up the menu
        unsafe {
            let _ = DestroyMenu(hmenu);
        }
    }

    /// Updates the tooltip to reflect the current state.
    fn update_tooltip(&mut self) {
        let tooltip: &[u16] = if self.active {
            // "Cat Shield - Active"
            &[
                'C' as u16, 'a' as u16, 't' as u16, ' ' as u16, 'S' as u16, 'h' as u16, 'i' as u16,
                'e' as u16, 'l' as u16, 'd' as u16, ' ' as u16, '-' as u16, ' ' as u16, 'A' as u16,
                'c' as u16, 't' as u16, 'i' as u16, 'v' as u16, 'e' as u16, 0,
            ]
        } else {
            // "Cat Shield"
            &[
                'C' as u16, 'a' as u16, 't' as u16, ' ' as u16, 'S' as u16, 'h' as u16, 'i' as u16,
                'e' as u16, 'l' as u16, 'd' as u16, 0,
            ]
        };

        // Clear and copy tooltip
        self.nid.szTip = [0; 128];
        let tip_len = tooltip.len().min(128);
        self.nid.szTip[..tip_len].copy_from_slice(&tooltip[..tip_len]);

        // Update the icon
        let result = unsafe { Shell_NotifyIconW(NIM_MODIFY, &self.nid) };
        if !result.as_bool() {
            warn!("Failed to update tray icon tooltip");
        }
    }
}

impl Default for WindowsSystemTray {
    fn default() -> Self {
        Self::new()
    }
}

impl SystemTray for WindowsSystemTray {
    fn setup(&mut self) -> Result<(), TrayError> {
        self.register_window_class()?;
        self.create_hidden_window()?;
        self.create_tray_icon()?;
        self.visible = true;
        Ok(())
    }

    fn set_state(&mut self, active: bool) {
        self.active = active;
        SHIELD_ACTIVE.store(active, Ordering::SeqCst);
        self.update_tooltip();
        debug!("Tray state updated: active={}", active);
    }

    fn remove(&mut self) {
        if self.visible {
            let result = unsafe { Shell_NotifyIconW(NIM_DELETE, &self.nid) };
            if !result.as_bool() {
                warn!("Failed to remove tray icon");
            }
            self.visible = false;
            info!("Removed Windows system tray icon");
        }

        if !self.hwnd.is_invalid() {
            let _ = unsafe { DestroyWindow(self.hwnd) };
            self.hwnd = HWND::default();
        }
    }

    fn is_visible(&self) -> bool {
        self.visible
    }
}

impl Drop for WindowsSystemTray {
    fn drop(&mut self) {
        self.remove();
    }
}

/// Window procedure for the hidden tray window.
///
/// Handles tray icon messages and menu commands.
unsafe extern "system" fn tray_window_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    match msg {
        WM_TRAYICON => {
            let event = lparam.0 as u32;
            match event {
                x if x == WM_RBUTTONUP => {
                    debug!("Tray right-click detected");
                    // Show context menu - we need to access the tray instance
                    // For now, create a temporary menu
                    show_tray_context_menu(hwnd);
                }
                x if x == WM_LBUTTONUP => {
                    debug!("Tray left-click detected");
                    // Left-click also shows the menu (common Windows behavior)
                    show_tray_context_menu(hwnd);
                }
                _ => {}
            }
            LRESULT(0)
        }
        WM_COMMAND => {
            let menu_id = (wparam.0 & 0xFFFF) as u16;
            handle_menu_command(menu_id);
            LRESULT(0)
        }
        WM_DESTROY => {
            PostQuitMessage(0);
            LRESULT(0)
        }
        _ => DefWindowProcW(hwnd, msg, wparam, lparam),
    }
}

/// Shows the context menu from the window procedure.
unsafe fn show_tray_context_menu(hwnd: HWND) {
    let hmenu = match CreatePopupMenu() {
        Ok(m) => m,
        Err(e) => {
            error!("Failed to create popup menu: {}", e);
            return;
        }
    };

    let active = SHIELD_ACTIVE.load(Ordering::SeqCst);

    // Build the menu based on shield state
    if active {
        let _ = AppendMenuW(
            hmenu,
            MF_STRING | MF_GRAYED,
            ID_MENU_START as usize,
            w!("Start Protection"),
        );
        let _ = AppendMenuW(
            hmenu,
            MF_STRING | MF_ENABLED,
            ID_MENU_STOP as usize,
            w!("Stop Protection"),
        );
    } else {
        let _ = AppendMenuW(
            hmenu,
            MF_STRING | MF_ENABLED,
            ID_MENU_START as usize,
            w!("Start Protection"),
        );
        let _ = AppendMenuW(
            hmenu,
            MF_STRING | MF_GRAYED,
            ID_MENU_STOP as usize,
            w!("Stop Protection"),
        );
    }

    let _ = AppendMenuW(hmenu, MF_SEPARATOR, 0, PCWSTR::null());
    let _ = AppendMenuW(
        hmenu,
        MF_STRING | MF_ENABLED,
        ID_MENU_SETTINGS as usize,
        w!("Settings..."),
    );
    let _ = AppendMenuW(
        hmenu,
        MF_STRING | MF_ENABLED,
        ID_MENU_ABOUT as usize,
        w!("About Cat Shield"),
    );
    let _ = AppendMenuW(hmenu, MF_SEPARATOR, 0, PCWSTR::null());
    let _ = AppendMenuW(
        hmenu,
        MF_STRING | MF_ENABLED,
        ID_MENU_QUIT as usize,
        w!("Quit"),
    );

    // Get cursor position
    let mut pt = windows::Win32::Foundation::POINT::default();
    let _ = GetCursorPos(&mut pt);

    // Required for proper menu dismissal
    let _ = SetForegroundWindow(hwnd);

    // Show the menu
    let _ = TrackPopupMenu(
        hmenu,
        TPM_RIGHTBUTTON | TPM_BOTTOMALIGN | TPM_LEFTALIGN,
        pt.x,
        pt.y,
        0,
        hwnd,
        None,
    );

    // Clean up
    let _ = DestroyMenu(hmenu);
}

/// Handles menu command selection.
fn handle_menu_command(menu_id: u16) {
    let action = match menu_id {
        ID_MENU_START => {
            info!("Menu: Start Protection selected");
            TrayMenuAction::StartProtection
        }
        ID_MENU_STOP => {
            info!("Menu: Stop Protection selected");
            TrayMenuAction::StopProtection
        }
        ID_MENU_SETTINGS => {
            info!("Menu: Settings selected");
            TrayMenuAction::OpenSettings
        }
        ID_MENU_ABOUT => {
            info!("Menu: About selected");
            TrayMenuAction::ShowAbout
        }
        ID_MENU_QUIT => {
            info!("Menu: Quit selected");
            TrayMenuAction::Quit
        }
        _ => {
            debug!("Unknown menu command: {}", menu_id);
            return;
        }
    };

    // Invoke the callback if set
    let ptr = MENU_CALLBACK.load(Ordering::SeqCst);
    if !ptr.is_null() {
        let callback = unsafe { &*(ptr as *const MenuCallback) };
        callback(action);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tray_menu_action_variants() {
        // Ensure all variants can be created and compared
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
    fn test_windows_system_tray_new() {
        let tray = WindowsSystemTray::new();
        assert!(!tray.visible);
        assert!(!tray.active);
        assert!(!tray.class_registered);
    }

    #[test]
    fn test_windows_system_tray_default() {
        let tray = WindowsSystemTray::default();
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
    fn test_menu_constants() {
        // Ensure menu IDs are unique
        assert_ne!(ID_MENU_START, ID_MENU_STOP);
        assert_ne!(ID_MENU_SETTINGS, ID_MENU_ABOUT);
        assert_ne!(ID_MENU_ABOUT, ID_MENU_QUIT);
    }

    #[test]
    fn test_wm_trayicon_constant() {
        // WM_USER is 0x0400
        assert_eq!(WM_TRAYICON, 0x0401);
    }
}
