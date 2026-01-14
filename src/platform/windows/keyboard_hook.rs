//! Windows keyboard hook for input blocking in Cat Shield
//!
//! Creates and manages a low-level keyboard hook using `SetWindowsHookEx`
//! to intercept and block keyboard input.
//!
//! # How It Works
//!
//! Windows low-level keyboard hooks work by:
//! 1. Installing a hook via `SetWindowsHookExW(WH_KEYBOARD_LL, callback, ...)`.
//! 2. The callback receives all keyboard events system-wide.
//! 3. Returning a non-zero value from the callback blocks the event.
//! 4. Calling `CallNextHookEx` passes the event to the next hook in the chain.
//!
//! # Thread Safety
//!
//! The hook must be installed and processed on a thread with a message loop.
//! The hook callback is called synchronously by the system when keyboard events occur.

use crate::platform::errors::InputBlockError;
use crate::platform::traits::InputBlocker;
use std::ffi::c_void;
use std::sync::atomic::{AtomicBool, AtomicPtr, Ordering};

use windows::Win32::Foundation::{HINSTANCE, LPARAM, LRESULT, WPARAM};
use windows::Win32::UI::WindowsAndMessaging::{
    CallNextHookEx, SetWindowsHookExW, UnhookWindowsHookEx, HHOOK, KBDLLHOOKSTRUCT, WH_KEYBOARD_LL,
    WM_KEYDOWN, WM_KEYUP, WM_SYSKEYDOWN, WM_SYSKEYUP,
};

/// Virtual key code for the Windows/Super key
const VK_LWIN: u32 = 0x5B;
const VK_RWIN: u32 = 0x5C;

/// Virtual key code for modifier keys
const VK_CONTROL: u32 = 0x11;
const VK_MENU: u32 = 0x12; // Alt key
const VK_SHIFT: u32 = 0x10;

/// Global pointer to the keyboard hook handle
static KEYBOARD_HOOK: AtomicPtr<c_void> = AtomicPtr::new(std::ptr::null_mut());

/// Flag indicating whether input blocking is currently enabled
static BLOCKING_ENABLED: AtomicBool = AtomicBool::new(false);

/// Global storage for exit key configuration
static EXIT_KEY_VKCODE: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);
static EXIT_KEY_REQUIRES_WIN: AtomicBool = AtomicBool::new(false);
static EXIT_KEY_REQUIRES_ALT: AtomicBool = AtomicBool::new(false);
static EXIT_KEY_REQUIRES_SHIFT: AtomicBool = AtomicBool::new(false);
static EXIT_KEY_REQUIRES_CTRL: AtomicBool = AtomicBool::new(false);

/// Set the exit key configuration for the keyboard hook.
///
/// # Arguments
///
/// * `vk_code` - Windows virtual key code
/// * `requires_win` - Whether Windows/Super key is required
/// * `requires_alt` - Whether Alt key is required
/// * `requires_shift` - Whether Shift key is required
/// * `requires_ctrl` - Whether Ctrl key is required
pub fn set_exit_key_config(
    vk_code: u32,
    requires_win: bool,
    requires_alt: bool,
    requires_shift: bool,
    requires_ctrl: bool,
) {
    EXIT_KEY_VKCODE.store(vk_code, Ordering::Release);
    EXIT_KEY_REQUIRES_WIN.store(requires_win, Ordering::Release);
    EXIT_KEY_REQUIRES_ALT.store(requires_alt, Ordering::Release);
    EXIT_KEY_REQUIRES_SHIFT.store(requires_shift, Ordering::Release);
    EXIT_KEY_REQUIRES_CTRL.store(requires_ctrl, Ordering::Release);
}

/// Global storage for allowed keys
///
/// This is a simplified implementation. A production version would use
/// a thread-safe collection to store multiple allowed key combinations.
static ALLOWED_KEYS: std::sync::RwLock<Vec<AllowedKeyConfig>> = std::sync::RwLock::new(Vec::new());

/// Configuration for an allowed key combination
#[derive(Clone, Debug)]
pub struct AllowedKeyConfig {
    /// Windows virtual key code
    pub vk_code: u32,
    /// Whether Windows/Super key is required
    pub requires_win: bool,
    /// Whether Alt key is required
    pub requires_alt: bool,
    /// Whether Shift key is required
    pub requires_shift: bool,
    /// Whether Ctrl key is required
    pub requires_ctrl: bool,
}

/// Set the allowed keys configuration.
pub fn set_allowed_keys(keys: Vec<AllowedKeyConfig>) {
    match ALLOWED_KEYS.write() {
        Ok(mut guard) => *guard = keys,
        Err(poisoned) => {
            // Recover from poisoned lock - data may be inconsistent but usable
            eprintln!("  ⚠️  Warning: ALLOWED_KEYS RwLock was poisoned, recovering...");
            let mut guard = poisoned.into_inner();
            *guard = keys;
        }
    }
}

/// Clear all allowed keys.
pub fn clear_allowed_keys() {
    match ALLOWED_KEYS.write() {
        Ok(mut guard) => guard.clear(),
        Err(poisoned) => {
            // Recover from poisoned lock
            eprintln!(
                "  ⚠️  Warning: ALLOWED_KEYS RwLock was poisoned during clear, recovering..."
            );
            poisoned.into_inner().clear();
        }
    }
}

/// Check if the current key event matches any allowed key.
fn is_key_allowed(vk_code: u32, modifiers: &ModifierState) -> bool {
    let guard = match ALLOWED_KEYS.read() {
        Ok(guard) => guard,
        Err(poisoned) => {
            // Recover from poisoned lock during read
            eprintln!(
                "  ⚠️  Warning: ALLOWED_KEYS RwLock was poisoned during check, recovering..."
            );
            poisoned.into_inner()
        }
    };
    guard.iter().any(|key| {
        key.vk_code == vk_code
            && key.requires_win == modifiers.win_pressed
            && key.requires_alt == modifiers.alt_pressed
            && key.requires_shift == modifiers.shift_pressed
            && key.requires_ctrl == modifiers.ctrl_pressed
    })
}

/// Current modifier key state
struct ModifierState {
    ctrl_pressed: bool,
    alt_pressed: bool,
    shift_pressed: bool,
    win_pressed: bool,
}

impl ModifierState {
    /// Get the current modifier key state using `GetAsyncKeyState`.
    fn current() -> Self {
        use windows::Win32::UI::Input::KeyboardAndMouse::GetAsyncKeyState;

        // SAFETY: GetAsyncKeyState is safe to call with any virtual key code.
        // A negative return value indicates the key is currently pressed.
        unsafe {
            Self {
                ctrl_pressed: GetAsyncKeyState(VK_CONTROL as i32) < 0,
                alt_pressed: GetAsyncKeyState(VK_MENU as i32) < 0,
                shift_pressed: GetAsyncKeyState(VK_SHIFT as i32) < 0,
                win_pressed: GetAsyncKeyState(VK_LWIN as i32) < 0
                    || GetAsyncKeyState(VK_RWIN as i32) < 0,
            }
        }
    }
}

/// Check if the given key event matches the configured exit key combination.
fn check_exit_key(vk_code: u32, modifiers: &ModifierState) -> bool {
    let expected_vk = EXIT_KEY_VKCODE.load(Ordering::Acquire);
    if vk_code != expected_vk {
        return false;
    }

    let requires_win = EXIT_KEY_REQUIRES_WIN.load(Ordering::Acquire);
    let requires_alt = EXIT_KEY_REQUIRES_ALT.load(Ordering::Acquire);
    let requires_shift = EXIT_KEY_REQUIRES_SHIFT.load(Ordering::Acquire);
    let requires_ctrl = EXIT_KEY_REQUIRES_CTRL.load(Ordering::Acquire);

    requires_win == modifiers.win_pressed
        && requires_alt == modifiers.alt_pressed
        && requires_shift == modifiers.shift_pressed
        && requires_ctrl == modifiers.ctrl_pressed
}

/// Low-level keyboard hook callback.
///
/// # Safety
///
/// This function is called by Windows as a hook callback. It must:
/// - Return quickly to avoid input lag
/// - Not block or perform long-running operations
/// - Call `CallNextHookEx` for events it doesn't handle
///
/// The `lparam` parameter points to a `KBDLLHOOKSTRUCT` when `ncode` is `HC_ACTION`.
unsafe extern "system" fn keyboard_hook_callback(
    ncode: i32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    // HC_ACTION (0) means we should process this message
    const HC_ACTION: i32 = 0;

    if ncode == HC_ACTION && BLOCKING_ENABLED.load(Ordering::Acquire) {
        let message = wparam.0 as u32;

        // Only handle key down/up events
        if message == WM_KEYDOWN
            || message == WM_KEYUP
            || message == WM_SYSKEYDOWN
            || message == WM_SYSKEYUP
        {
            // SAFETY: lparam points to a valid KBDLLHOOKSTRUCT when ncode == HC_ACTION
            let kb_struct = &*(lparam.0 as *const KBDLLHOOKSTRUCT);
            let vk_code = kb_struct.vkCode;

            // Get current modifier state
            let modifiers = ModifierState::current();

            // Check for exit key combination (only on key down)
            if (message == WM_KEYDOWN || message == WM_SYSKEYDOWN)
                && check_exit_key(vk_code, &modifiers)
            {
                // Exit key detected - let the event propagate to the application
                // TODO: In a full implementation, this would signal the main thread to exit
                return CallNextHookEx(HHOOK::default(), ncode, wparam, lparam);
            }

            // Check if this key is in the allowed list
            if is_key_allowed(vk_code, &modifiers) {
                // Let allowed keys through
                return CallNextHookEx(HHOOK::default(), ncode, wparam, lparam);
            }

            // Block the keyboard event by returning non-zero
            // This prevents the event from being passed to the target window
            return LRESULT(1);
        }
    }

    // Pass unhandled events to the next hook in the chain
    CallNextHookEx(HHOOK::default(), ncode, wparam, lparam)
}

/// Windows implementation of the `InputBlocker` trait.
///
/// Uses `SetWindowsHookExW` with `WH_KEYBOARD_LL` to create a low-level
/// keyboard hook that intercepts and blocks keyboard input system-wide.
#[derive(Debug, Default)]
pub struct WindowsInputBlocker {
    /// Tracks whether the keyboard hook is currently active
    active: bool,
}

impl WindowsInputBlocker {
    /// Creates a new `WindowsInputBlocker`.
    #[must_use]
    pub fn new() -> Self {
        Self { active: false }
    }
}

impl InputBlocker for WindowsInputBlocker {
    fn setup(&mut self) -> Result<(), InputBlockError> {
        if self.active {
            return Ok(());
        }

        // Check if another instance already has the hook installed
        // This prevents leaking hook handles when multiple instances exist
        if !KEYBOARD_HOOK.load(Ordering::Acquire).is_null() {
            // Another instance owns the hook - just mark ourselves as tracking it
            self.active = true;
            return Ok(());
        }

        // SAFETY: SetWindowsHookExW is safe to call when:
        // - The callback function has the correct signature
        // - We store the returned handle for later cleanup
        // - The hook is installed on a thread that will pump messages
        let hook = unsafe {
            SetWindowsHookExW(
                WH_KEYBOARD_LL,
                Some(keyboard_hook_callback),
                HINSTANCE::default(),
                0, // Thread ID 0 = all threads (system-wide hook)
            )
        };

        match hook {
            Ok(h) => {
                // Store the hook handle for later cleanup
                KEYBOARD_HOOK.store(h.0, Ordering::Release);
                BLOCKING_ENABLED.store(true, Ordering::Release);
                self.active = true;
                Ok(())
            }
            Err(e) => Err(InputBlockError::CreationFailed(format!(
                "Failed to install keyboard hook: {}",
                e
            ))),
        }
    }

    fn disable(&mut self) {
        if !self.active {
            return;
        }

        // Disable blocking first
        BLOCKING_ENABLED.store(false, Ordering::Release);

        // Get and clear the hook handle
        let hook_ptr = KEYBOARD_HOOK.swap(std::ptr::null_mut(), Ordering::AcqRel);

        if !hook_ptr.is_null() {
            // SAFETY: We only call UnhookWindowsHookEx with a valid hook handle
            // that we obtained from SetWindowsHookExW
            unsafe {
                let hook = HHOOK(hook_ptr);
                let _ = UnhookWindowsHookEx(hook);
            }
        }

        self.active = false;
    }

    fn is_active(&self) -> bool {
        self.active && !KEYBOARD_HOOK.load(Ordering::Acquire).is_null()
    }
}

// SAFETY: WindowsInputBlocker is Send + Sync because:
// - The `active` field is only accessed through &mut self (setup/disable) or &self (is_active)
// - The underlying KEYBOARD_HOOK is an AtomicPtr which is inherently thread-safe
// - The hook callback uses atomic operations for all shared state access
unsafe impl Send for WindowsInputBlocker {}
unsafe impl Sync for WindowsInputBlocker {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_windows_input_blocker_new() {
        let blocker = WindowsInputBlocker::new();
        assert!(!blocker.active);
    }

    #[test]
    fn test_windows_input_blocker_default() {
        let blocker = WindowsInputBlocker::default();
        assert!(!blocker.active);
    }

    #[test]
    fn test_allowed_key_config() {
        let config = AllowedKeyConfig {
            vk_code: 0x41, // A
            requires_win: false,
            requires_alt: false,
            requires_shift: false,
            requires_ctrl: true,
        };
        assert_eq!(config.vk_code, 0x41);
        assert!(config.requires_ctrl);
        assert!(!config.requires_win);
    }

    #[test]
    fn test_set_exit_key_config() {
        set_exit_key_config(0x51, true, false, false, false); // Win+Q
        assert_eq!(EXIT_KEY_VKCODE.load(Ordering::Acquire), 0x51);
        assert!(EXIT_KEY_REQUIRES_WIN.load(Ordering::Acquire));
        assert!(!EXIT_KEY_REQUIRES_ALT.load(Ordering::Acquire));
    }

    #[test]
    fn test_set_and_clear_allowed_keys() {
        let keys = vec![
            AllowedKeyConfig {
                vk_code: 0x70, // F1
                requires_win: false,
                requires_alt: false,
                requires_shift: false,
                requires_ctrl: false,
            },
            AllowedKeyConfig {
                vk_code: 0x71, // F2
                requires_win: false,
                requires_alt: false,
                requires_shift: false,
                requires_ctrl: false,
            },
        ];

        set_allowed_keys(keys);
        {
            let guard = ALLOWED_KEYS.read().unwrap();
            assert_eq!(guard.len(), 2);
        }

        clear_allowed_keys();
        {
            let guard = ALLOWED_KEYS.read().unwrap();
            assert!(guard.is_empty());
        }
    }

    #[test]
    fn test_input_blocker_is_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<WindowsInputBlocker>();
    }
}
