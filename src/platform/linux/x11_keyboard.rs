//! X11 keyboard grab for input blocking in Cat Shield
//!
//! Uses XGrabKeyboard to intercept and block keyboard input on X11.
//!
//! # How It Works
//!
//! X11 keyboard grabbing works by:
//! 1. Connecting to the X11 display server
//! 2. Using `XGrabKeyboard` to redirect all keyboard events to our window
//! 3. The grab is "synchronous" - we can choose which events to allow through
//! 4. `XUngrabKeyboard` releases the grab when done
//!
//! # Thread Safety
//!
//! The X11 connection is single-threaded by default. Operations must be
//! performed on the thread that created the connection.

use crate::platform::errors::InputBlockError;
use crate::platform::traits::InputBlocker;
use std::sync::atomic::{AtomicBool, AtomicPtr, Ordering};
use std::sync::RwLock;
use x11rb::connection::Connection;
use x11rb::protocol::xproto::{
    ConnectionExt, GrabMode, GrabStatus, KeyPressEvent, Keycode, KeyButMask,
};
use x11rb::rust_connection::RustConnection;

/// Global pointer to the X11 connection
static X11_CONNECTION: AtomicPtr<RustConnection> = AtomicPtr::new(std::ptr::null_mut());

/// Flag indicating whether input blocking is currently enabled
static BLOCKING_ENABLED: AtomicBool = AtomicBool::new(false);

/// Global storage for exit key configuration
static EXIT_KEY_KEYCODE: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);
static EXIT_KEY_REQUIRES_SUPER: AtomicBool = AtomicBool::new(false);
static EXIT_KEY_REQUIRES_ALT: AtomicBool = AtomicBool::new(false);
static EXIT_KEY_REQUIRES_SHIFT: AtomicBool = AtomicBool::new(false);
static EXIT_KEY_REQUIRES_CTRL: AtomicBool = AtomicBool::new(false);

/// Set the exit key configuration for the keyboard grab.
///
/// # Arguments
///
/// * `keycode` - X11 keycode
/// * `requires_super` - Whether Super (Windows) key is required
/// * `requires_alt` - Whether Alt key is required
/// * `requires_shift` - Whether Shift key is required
/// * `requires_ctrl` - Whether Ctrl key is required
pub fn set_exit_key_config(
    keycode: u32,
    requires_super: bool,
    requires_alt: bool,
    requires_shift: bool,
    requires_ctrl: bool,
) {
    EXIT_KEY_KEYCODE.store(keycode, Ordering::Release);
    EXIT_KEY_REQUIRES_SUPER.store(requires_super, Ordering::Release);
    EXIT_KEY_REQUIRES_ALT.store(requires_alt, Ordering::Release);
    EXIT_KEY_REQUIRES_SHIFT.store(requires_shift, Ordering::Release);
    EXIT_KEY_REQUIRES_CTRL.store(requires_ctrl, Ordering::Release);
}

/// Configuration for an allowed key combination
#[derive(Clone, Debug)]
pub struct AllowedKeyConfig {
    /// X11 keycode
    pub keycode: u32,
    /// Whether Super (Windows) key is required
    pub requires_super: bool,
    /// Whether Alt key is required
    pub requires_alt: bool,
    /// Whether Shift key is required
    pub requires_shift: bool,
    /// Whether Ctrl key is required
    pub requires_ctrl: bool,
}

/// Global storage for allowed keys
static ALLOWED_KEYS: RwLock<Vec<AllowedKeyConfig>> = RwLock::new(Vec::new());

/// Set the allowed keys configuration.
pub fn set_allowed_keys(keys: Vec<AllowedKeyConfig>) {
    match ALLOWED_KEYS.write() {
        Ok(mut guard) => *guard = keys,
        Err(poisoned) => {
            log::warn!("ALLOWED_KEYS RwLock was poisoned, recovering...");
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
            log::warn!("ALLOWED_KEYS RwLock was poisoned during clear, recovering...");
            poisoned.into_inner().clear();
        }
    }
}

/// Current modifier key state extracted from X11 event state
struct ModifierState {
    ctrl_pressed: bool,
    alt_pressed: bool,
    shift_pressed: bool,
    super_pressed: bool,
}

impl ModifierState {
    /// Extract modifier state from X11 key event state mask
    fn from_state(state: KeyButMask) -> Self {
        Self {
            ctrl_pressed: state.contains(KeyButMask::CONTROL),
            alt_pressed: state.contains(KeyButMask::MOD1), // Mod1 is typically Alt
            shift_pressed: state.contains(KeyButMask::SHIFT),
            super_pressed: state.contains(KeyButMask::MOD4), // Mod4 is typically Super
        }
    }
}

/// Check if the given key event matches the configured exit key combination.
fn check_exit_key(keycode: Keycode, modifiers: &ModifierState) -> bool {
    let expected_keycode = EXIT_KEY_KEYCODE.load(Ordering::Acquire);
    if u32::from(keycode) != expected_keycode {
        return false;
    }

    let requires_super = EXIT_KEY_REQUIRES_SUPER.load(Ordering::Acquire);
    let requires_alt = EXIT_KEY_REQUIRES_ALT.load(Ordering::Acquire);
    let requires_shift = EXIT_KEY_REQUIRES_SHIFT.load(Ordering::Acquire);
    let requires_ctrl = EXIT_KEY_REQUIRES_CTRL.load(Ordering::Acquire);

    requires_super == modifiers.super_pressed
        && requires_alt == modifiers.alt_pressed
        && requires_shift == modifiers.shift_pressed
        && requires_ctrl == modifiers.ctrl_pressed
}

/// Check if the current key event matches any allowed key.
fn is_key_allowed(keycode: Keycode, modifiers: &ModifierState) -> bool {
    let guard = match ALLOWED_KEYS.read() {
        Ok(guard) => guard,
        Err(poisoned) => {
            log::warn!("ALLOWED_KEYS RwLock was poisoned during check, recovering...");
            poisoned.into_inner()
        }
    };
    guard.iter().any(|key| {
        key.keycode == u32::from(keycode)
            && key.requires_super == modifiers.super_pressed
            && key.requires_alt == modifiers.alt_pressed
            && key.requires_shift == modifiers.shift_pressed
            && key.requires_ctrl == modifiers.ctrl_pressed
    })
}

/// X11 implementation of the `InputBlocker` trait.
///
/// Uses `XGrabKeyboard` to grab all keyboard input and selectively allow
/// configured keys through.
#[derive(Debug, Default)]
pub struct X11InputBlocker {
    /// Tracks whether the keyboard grab is currently active
    active: bool,
    /// The root window ID for the grab
    root_window: u32,
}

impl X11InputBlocker {
    /// Creates a new `X11InputBlocker`.
    #[must_use]
    pub fn new() -> Self {
        Self {
            active: false,
            root_window: 0,
        }
    }

    /// Process keyboard events during the grab.
    ///
    /// This function should be called in the main event loop to handle
    /// keyboard events and determine which ones to allow through.
    ///
    /// Returns `true` if the exit key was detected.
    pub fn process_event(&self, event: &KeyPressEvent) -> ProcessResult {
        if !self.active {
            return ProcessResult::NotBlocking;
        }

        let modifiers = ModifierState::from_state(event.state);

        // Check for exit key combination
        if check_exit_key(event.detail, &modifiers) {
            log::info!("🔓 Exit key combination detected!");
            return ProcessResult::ExitRequested;
        }

        // Check if this key is allowed
        if is_key_allowed(event.detail, &modifiers) {
            return ProcessResult::Allowed;
        }

        // Block the event
        ProcessResult::Blocked
    }

    /// Get a reference to the X11 connection if available.
    fn get_connection() -> Option<&'static RustConnection> {
        let ptr = X11_CONNECTION.load(Ordering::Acquire);
        if ptr.is_null() {
            None
        } else {
            // SAFETY: We only store valid pointers in X11_CONNECTION from setup(),
            // and the pointer remains valid until disable() is called.
            Some(unsafe { &*ptr })
        }
    }
}

/// Result of processing a keyboard event
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProcessResult {
    /// Input blocking is not active
    NotBlocking,
    /// The exit key combination was detected
    ExitRequested,
    /// The key is in the allowed list and should be forwarded
    Allowed,
    /// The key should be blocked
    Blocked,
}

impl InputBlocker for X11InputBlocker {
    fn setup(&mut self) -> Result<(), InputBlockError> {
        if self.active {
            return Ok(());
        }

        // Check if another instance already has the connection established
        if !X11_CONNECTION.load(Ordering::Acquire).is_null() {
            // Another instance owns the connection - just mark ourselves as tracking it
            self.active = true;
            return Ok(());
        }

        // Connect to the X11 display
        let (conn, screen_num) = RustConnection::connect(None).map_err(|e| {
            InputBlockError::CreationFailed(format!("Failed to connect to X11 display: {}", e))
        })?;

        // Get the root window
        let screen = &conn.setup().roots[screen_num];
        self.root_window = screen.root;

        // Grab the keyboard on the root window
        // owner_events: false - all events go to the grab window
        // pointer_mode: Async - don't affect pointer events
        // keyboard_mode: Sync - we'll control which events are allowed
        let grab_result = conn
            .grab_keyboard(
                false,                // owner_events
                self.root_window,     // grab_window (root = entire screen)
                x11rb::CURRENT_TIME,  // time
                GrabMode::ASYNC,      // pointer_mode
                GrabMode::SYNC,       // keyboard_mode - sync allows us to replay events
            )
            .map_err(|e| {
                InputBlockError::CreationFailed(format!("Failed to send keyboard grab request: {}", e))
            })?
            .reply()
            .map_err(|e| {
                InputBlockError::CreationFailed(format!("Failed to grab keyboard: {}", e))
            })?;

        if grab_result.status != GrabStatus::SUCCESS {
            return Err(InputBlockError::CreationFailed(format!(
                "Keyboard grab failed with status: {:?}",
                grab_result.status
            )));
        }

        // Store the connection globally
        let conn_box = Box::new(conn);
        let conn_ptr = Box::into_raw(conn_box);
        X11_CONNECTION.store(conn_ptr, Ordering::Release);

        BLOCKING_ENABLED.store(true, Ordering::Release);
        self.active = true;

        log::info!("✓ X11 keyboard grab enabled");
        Ok(())
    }

    fn disable(&mut self) {
        if !self.active {
            return;
        }

        BLOCKING_ENABLED.store(false, Ordering::Release);

        // Get and clear the connection pointer
        let conn_ptr = X11_CONNECTION.swap(std::ptr::null_mut(), Ordering::AcqRel);

        if !conn_ptr.is_null() {
            // SAFETY: We only store valid Box pointers in X11_CONNECTION,
            // and we're the only ones who will call disable() with a non-null pointer.
            let conn = unsafe { Box::from_raw(conn_ptr) };

            // Ungrab the keyboard
            if let Err(e) = conn.ungrab_keyboard(x11rb::CURRENT_TIME) {
                log::warn!("Failed to ungrab keyboard: {}", e);
            }

            // Flush any pending requests
            if let Err(e) = conn.flush() {
                log::warn!("Failed to flush X11 connection: {}", e);
            }

            // Connection is dropped here, closing it
            log::info!("✓ X11 keyboard grab disabled");
        }

        self.active = false;
    }

    fn is_active(&self) -> bool {
        self.active && !X11_CONNECTION.load(Ordering::Acquire).is_null()
    }
}

// SAFETY: X11InputBlocker is Send + Sync because:
// - The `active` and `root_window` fields are only accessed through &mut self or &self
// - The underlying X11_CONNECTION is an AtomicPtr which is inherently thread-safe
// - X11 operations use proper atomic ordering for synchronization
//
// Note: The actual X11 connection operations should be performed on the same thread
// that created the connection. The struct itself can be sent between threads, but
// the caller is responsible for ensuring proper thread affinity for X11 calls.
unsafe impl Send for X11InputBlocker {}
unsafe impl Sync for X11InputBlocker {}

/// Allow the next keyboard event through by replaying it.
///
/// In synchronous keyboard grab mode, we must call `allow_events` to let
/// events through. This function replays the current event.
pub fn allow_keyboard_event() {
    if let Some(conn) = X11InputBlocker::get_connection() {
        // ReplayKeyboard replays the current event as if the grab wasn't active
        if let Err(e) = conn.allow_events(x11rb::protocol::xproto::Allow::REPLAY_KEYBOARD, x11rb::CURRENT_TIME) {
            log::warn!("Failed to replay keyboard event: {}", e);
        }
        let _ = conn.flush();
    }
}

/// Discard the next keyboard event (block it).
///
/// In synchronous keyboard grab mode, we call `allow_events` with SyncKeyboard
/// to freeze keyboard input while we decide what to do next.
pub fn block_keyboard_event() {
    if let Some(conn) = X11InputBlocker::get_connection() {
        // SyncKeyboard continues processing but discards the current event
        if let Err(e) = conn.allow_events(x11rb::protocol::xproto::Allow::SYNC_KEYBOARD, x11rb::CURRENT_TIME) {
            log::warn!("Failed to block keyboard event: {}", e);
        }
        let _ = conn.flush();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_x11_input_blocker_new() {
        let blocker = X11InputBlocker::new();
        assert!(!blocker.active);
        assert_eq!(blocker.root_window, 0);
    }

    #[test]
    fn test_x11_input_blocker_default() {
        let blocker = X11InputBlocker::default();
        assert!(!blocker.active);
        assert_eq!(blocker.root_window, 0);
    }

    #[test]
    fn test_allowed_key_config() {
        let config = AllowedKeyConfig {
            keycode: 38, // 'a' on many keyboards
            requires_super: false,
            requires_alt: false,
            requires_shift: false,
            requires_ctrl: true,
        };
        assert_eq!(config.keycode, 38);
        assert!(config.requires_ctrl);
        assert!(!config.requires_super);
    }

    #[test]
    fn test_set_exit_key_config() {
        set_exit_key_config(24, true, false, false, false); // Super+Q
        assert_eq!(EXIT_KEY_KEYCODE.load(Ordering::Acquire), 24);
        assert!(EXIT_KEY_REQUIRES_SUPER.load(Ordering::Acquire));
        assert!(!EXIT_KEY_REQUIRES_ALT.load(Ordering::Acquire));
    }

    #[test]
    fn test_set_and_clear_allowed_keys() {
        let keys = vec![
            AllowedKeyConfig {
                keycode: 67, // F1
                requires_super: false,
                requires_alt: false,
                requires_shift: false,
                requires_ctrl: false,
            },
            AllowedKeyConfig {
                keycode: 68, // F2
                requires_super: false,
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
    fn test_process_result_variants() {
        assert_ne!(ProcessResult::NotBlocking, ProcessResult::ExitRequested);
        assert_ne!(ProcessResult::Allowed, ProcessResult::Blocked);
    }

    #[test]
    fn test_input_blocker_is_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<X11InputBlocker>();
    }

    #[test]
    fn test_modifier_state_from_state() {
        // Test with no modifiers
        let state = KeyButMask::from(0u16);
        let mods = ModifierState::from_state(state);
        assert!(!mods.ctrl_pressed);
        assert!(!mods.alt_pressed);
        assert!(!mods.shift_pressed);
        assert!(!mods.super_pressed);

        // Test with shift modifier
        let state = KeyButMask::SHIFT;
        let mods = ModifierState::from_state(state);
        assert!(!mods.ctrl_pressed);
        assert!(!mods.alt_pressed);
        assert!(mods.shift_pressed);
        assert!(!mods.super_pressed);

        // Test with ctrl modifier
        let state = KeyButMask::CONTROL;
        let mods = ModifierState::from_state(state);
        assert!(mods.ctrl_pressed);
        assert!(!mods.alt_pressed);
        assert!(!mods.shift_pressed);
        assert!(!mods.super_pressed);
    }
}
