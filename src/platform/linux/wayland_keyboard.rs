//! Wayland keyboard input handling for Cat Shield
//!
//! This module implements keyboard input capture using the `keyboard-shortcuts-inhibit`
//! protocol on Wayland compositors that support it (wlroots-based: Sway, Hyprland, etc.).
//!
//! # How It Works
//!
//! 1. The overlay window binds to `wl_seat` to get the seat (input device collection)
//! 2. Gets `wl_keyboard` from the seat to receive keyboard events
//! 3. Uses `zwp_keyboard_shortcuts_inhibit_manager_v1` to request shortcut inhibition
//! 4. When the overlay surface has focus, all keyboard events are received
//! 5. Processes events for exit key detection and allowed keys filtering
//!
//! # Protocol Support
//!
//! - **keyboard-shortcuts-inhibit-unstable-v1**: Inhibits compositor keyboard shortcuts
//! - Works on wlroots-based compositors (Sway, Hyprland, River, Wayfire)
//! - **Not supported on GNOME/Mutter**
//!
//! # Limitations
//!
//! - Requires keyboard focus - user can click away to remove focus
//! - Compositor may keep certain key combinations for itself
//! - Only works when overlay is visible and focused
//!
//! See [docs/WAYLAND_INPUT_RESEARCH.md](docs/WAYLAND_INPUT_RESEARCH.md) for full research.

use log::{debug, info, warn};
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::RwLock;
use wayland_client::protocol::wl_registry;
use wayland_client::{Connection, Dispatch, QueueHandle};

/// Global exit key configuration
static WAYLAND_EXIT_KEY_KEYCODE: AtomicU32 = AtomicU32::new(0);
static WAYLAND_EXIT_KEY_REQUIRES_SUPER: AtomicBool = AtomicBool::new(false);
static WAYLAND_EXIT_KEY_REQUIRES_ALT: AtomicBool = AtomicBool::new(false);
static WAYLAND_EXIT_KEY_REQUIRES_SHIFT: AtomicBool = AtomicBool::new(false);
static WAYLAND_EXIT_KEY_REQUIRES_CTRL: AtomicBool = AtomicBool::new(false);

/// Sets the exit key configuration for the Wayland keyboard handler.
///
/// # Arguments
///
/// * `keycode` - Wayland/Linux keycode (evdev keycode)
/// * `requires_super` - Whether Super (Windows/Meta) key is required
/// * `requires_alt` - Whether Alt key is required
/// * `requires_shift` - Whether Shift key is required
/// * `requires_ctrl` - Whether Ctrl key is required
pub fn set_wayland_exit_key_config(
    keycode: u32,
    requires_super: bool,
    requires_alt: bool,
    requires_shift: bool,
    requires_ctrl: bool,
) {
    WAYLAND_EXIT_KEY_KEYCODE.store(keycode, Ordering::Release);
    WAYLAND_EXIT_KEY_REQUIRES_SUPER.store(requires_super, Ordering::Release);
    WAYLAND_EXIT_KEY_REQUIRES_ALT.store(requires_alt, Ordering::Release);
    WAYLAND_EXIT_KEY_REQUIRES_SHIFT.store(requires_shift, Ordering::Release);
    WAYLAND_EXIT_KEY_REQUIRES_CTRL.store(requires_ctrl, Ordering::Release);
}

/// Configuration for an allowed key combination on Wayland.
#[derive(Clone, Debug)]
pub struct WaylandAllowedKeyConfig {
    /// Wayland/Linux keycode (evdev keycode)
    pub keycode: u32,
    /// Whether Super (Windows/Meta) key is required
    pub requires_super: bool,
    /// Whether Alt key is required
    pub requires_alt: bool,
    /// Whether Shift key is required
    pub requires_shift: bool,
    /// Whether Ctrl key is required
    pub requires_ctrl: bool,
}

/// Global storage for allowed keys on Wayland.
static WAYLAND_ALLOWED_KEYS: RwLock<Vec<WaylandAllowedKeyConfig>> = RwLock::new(Vec::new());

/// Sets the allowed keys configuration for Wayland.
pub fn set_wayland_allowed_keys(keys: Vec<WaylandAllowedKeyConfig>) {
    match WAYLAND_ALLOWED_KEYS.write() {
        Ok(mut guard) => *guard = keys,
        Err(poisoned) => {
            warn!("WAYLAND_ALLOWED_KEYS RwLock was poisoned, recovering...");
            let mut guard = poisoned.into_inner();
            *guard = keys;
        }
    }
}

/// Clears all allowed keys for Wayland.
pub fn clear_wayland_allowed_keys() {
    match WAYLAND_ALLOWED_KEYS.write() {
        Ok(mut guard) => guard.clear(),
        Err(poisoned) => {
            warn!("WAYLAND_ALLOWED_KEYS RwLock was poisoned during clear, recovering...");
            poisoned.into_inner().clear();
        }
    }
}

/// Result of processing a Wayland keyboard event.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WaylandKeyboardResult {
    /// No keyboard focus or handler not active
    NotActive,
    /// The exit key combination was detected
    ExitRequested,
    /// The key is in the allowed list and should be forwarded
    Allowed,
    /// The key should be blocked (absorbed by the overlay)
    Blocked,
}

/// Current modifier key state for Wayland keyboard events.
#[derive(Debug, Clone, Default)]
pub struct WaylandModifierState {
    /// Whether Ctrl is pressed
    pub ctrl_pressed: bool,
    /// Whether Alt is pressed
    pub alt_pressed: bool,
    /// Whether Shift is pressed
    pub shift_pressed: bool,
    /// Whether Super/Logo key is pressed
    pub super_pressed: bool,
}

impl WaylandModifierState {
    /// Creates a new modifier state from the wl_keyboard modifiers event.
    ///
    /// The `mods_depressed` field contains a bitmask of currently pressed modifiers:
    /// - Bit 0: Shift
    /// - Bit 2: Control
    /// - Bit 3: Mod1 (Alt)
    /// - Bit 6: Mod4 (Super/Logo)
    pub fn from_depressed(mods_depressed: u32) -> Self {
        Self {
            shift_pressed: (mods_depressed & (1 << 0)) != 0,
            ctrl_pressed: (mods_depressed & (1 << 2)) != 0,
            alt_pressed: (mods_depressed & (1 << 3)) != 0,
            super_pressed: (mods_depressed & (1 << 6)) != 0,
        }
    }
}

/// Checks if the given keycode and modifiers match the configured exit key.
fn check_wayland_exit_key(keycode: u32, modifiers: &WaylandModifierState) -> bool {
    let expected_keycode = WAYLAND_EXIT_KEY_KEYCODE.load(Ordering::Acquire);
    if keycode != expected_keycode {
        return false;
    }

    let requires_super = WAYLAND_EXIT_KEY_REQUIRES_SUPER.load(Ordering::Acquire);
    let requires_alt = WAYLAND_EXIT_KEY_REQUIRES_ALT.load(Ordering::Acquire);
    let requires_shift = WAYLAND_EXIT_KEY_REQUIRES_SHIFT.load(Ordering::Acquire);
    let requires_ctrl = WAYLAND_EXIT_KEY_REQUIRES_CTRL.load(Ordering::Acquire);

    requires_super == modifiers.super_pressed
        && requires_alt == modifiers.alt_pressed
        && requires_shift == modifiers.shift_pressed
        && requires_ctrl == modifiers.ctrl_pressed
}

/// Checks if the given keycode and modifiers match any allowed key.
fn is_wayland_key_allowed(keycode: u32, modifiers: &WaylandModifierState) -> bool {
    let guard = match WAYLAND_ALLOWED_KEYS.read() {
        Ok(guard) => guard,
        Err(poisoned) => {
            warn!("WAYLAND_ALLOWED_KEYS RwLock was poisoned during check, recovering...");
            poisoned.into_inner()
        }
    };

    guard.iter().any(|key| {
        key.keycode == keycode
            && key.requires_super == modifiers.super_pressed
            && key.requires_alt == modifiers.alt_pressed
            && key.requires_shift == modifiers.shift_pressed
            && key.requires_ctrl == modifiers.ctrl_pressed
    })
}

/// Processes a Wayland keyboard key event.
///
/// Returns the appropriate action based on the key pressed and current configuration.
pub fn process_wayland_key_event(
    keycode: u32,
    modifiers: &WaylandModifierState,
) -> WaylandKeyboardResult {
    // Check for exit key combination
    if check_wayland_exit_key(keycode, modifiers) {
        info!("Wayland: Exit key combination detected!");
        return WaylandKeyboardResult::ExitRequested;
    }

    // Check if this key is allowed
    if is_wayland_key_allowed(keycode, modifiers) {
        return WaylandKeyboardResult::Allowed;
    }

    // Block the event
    WaylandKeyboardResult::Blocked
}

/// Checks if the keyboard-shortcuts-inhibit protocol is available.
pub fn is_keyboard_shortcuts_inhibit_available() -> bool {
    // Try to connect and check for the protocol
    let connection = match Connection::connect_to_env() {
        Ok(c) => c,
        Err(_) => return false,
    };

    let mut event_queue = connection.new_event_queue::<ProtocolCheckState>();
    let qh = event_queue.handle();

    let display = connection.display();
    let _registry = display.get_registry(&qh, ());

    let mut state = ProtocolCheckState::new();
    if event_queue.roundtrip(&mut state).is_err() {
        return false;
    }

    state.shortcuts_inhibit_available
}

/// Minimal state for checking protocol availability.
struct ProtocolCheckState {
    shortcuts_inhibit_available: bool,
}

impl ProtocolCheckState {
    fn new() -> Self {
        Self {
            shortcuts_inhibit_available: false,
        }
    }
}

// Registry dispatch for protocol check
impl Dispatch<wl_registry::WlRegistry, ()> for ProtocolCheckState {
    fn event(
        state: &mut Self,
        _: &wl_registry::WlRegistry,
        event: wl_registry::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        if let wl_registry::Event::Global { interface, .. } = event {
            if interface == "zwp_keyboard_shortcuts_inhibit_manager_v1" {
                state.shortcuts_inhibit_available = true;
                debug!("Found keyboard-shortcuts-inhibit protocol");
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wayland_modifier_state_from_depressed() {
        // No modifiers
        let state = WaylandModifierState::from_depressed(0);
        assert!(!state.shift_pressed);
        assert!(!state.ctrl_pressed);
        assert!(!state.alt_pressed);
        assert!(!state.super_pressed);

        // Shift only (bit 0)
        let state = WaylandModifierState::from_depressed(0b0000_0001);
        assert!(state.shift_pressed);
        assert!(!state.ctrl_pressed);
        assert!(!state.alt_pressed);
        assert!(!state.super_pressed);

        // Ctrl only (bit 2)
        let state = WaylandModifierState::from_depressed(0b0000_0100);
        assert!(!state.shift_pressed);
        assert!(state.ctrl_pressed);
        assert!(!state.alt_pressed);
        assert!(!state.super_pressed);

        // Alt only (bit 3)
        let state = WaylandModifierState::from_depressed(0b0000_1000);
        assert!(!state.shift_pressed);
        assert!(!state.ctrl_pressed);
        assert!(state.alt_pressed);
        assert!(!state.super_pressed);

        // Super only (bit 6)
        let state = WaylandModifierState::from_depressed(0b0100_0000);
        assert!(!state.shift_pressed);
        assert!(!state.ctrl_pressed);
        assert!(!state.alt_pressed);
        assert!(state.super_pressed);

        // All modifiers
        let state = WaylandModifierState::from_depressed(0b0100_1101);
        assert!(state.shift_pressed);
        assert!(state.ctrl_pressed);
        assert!(state.alt_pressed);
        assert!(state.super_pressed);
    }

    #[test]
    fn test_set_wayland_exit_key_config() {
        set_wayland_exit_key_config(16, true, false, false, false); // Super+Q (Q is keycode 16)
        assert_eq!(WAYLAND_EXIT_KEY_KEYCODE.load(Ordering::Acquire), 16);
        assert!(WAYLAND_EXIT_KEY_REQUIRES_SUPER.load(Ordering::Acquire));
        assert!(!WAYLAND_EXIT_KEY_REQUIRES_ALT.load(Ordering::Acquire));
        assert!(!WAYLAND_EXIT_KEY_REQUIRES_SHIFT.load(Ordering::Acquire));
        assert!(!WAYLAND_EXIT_KEY_REQUIRES_CTRL.load(Ordering::Acquire));
    }

    #[test]
    fn test_check_wayland_exit_key() {
        // Set up exit key as Super+Q (keycode 16)
        set_wayland_exit_key_config(16, true, false, false, false);

        // Matching key and modifier
        let modifiers = WaylandModifierState {
            super_pressed: true,
            alt_pressed: false,
            shift_pressed: false,
            ctrl_pressed: false,
        };
        assert!(check_wayland_exit_key(16, &modifiers));

        // Wrong keycode
        assert!(!check_wayland_exit_key(17, &modifiers));

        // Wrong modifiers
        let wrong_modifiers = WaylandModifierState {
            super_pressed: false,
            alt_pressed: false,
            shift_pressed: false,
            ctrl_pressed: false,
        };
        assert!(!check_wayland_exit_key(16, &wrong_modifiers));
    }

    #[test]
    fn test_set_and_clear_wayland_allowed_keys() {
        let keys = vec![
            WaylandAllowedKeyConfig {
                keycode: 59, // F1
                requires_super: false,
                requires_alt: false,
                requires_shift: false,
                requires_ctrl: false,
            },
            WaylandAllowedKeyConfig {
                keycode: 60, // F2
                requires_super: false,
                requires_alt: false,
                requires_shift: false,
                requires_ctrl: false,
            },
        ];

        set_wayland_allowed_keys(keys);
        {
            let guard = WAYLAND_ALLOWED_KEYS.read().unwrap();
            assert_eq!(guard.len(), 2);
        }

        clear_wayland_allowed_keys();
        {
            let guard = WAYLAND_ALLOWED_KEYS.read().unwrap();
            assert!(guard.is_empty());
        }
    }

    #[test]
    fn test_is_wayland_key_allowed() {
        let keys = vec![WaylandAllowedKeyConfig {
            keycode: 59, // F1
            requires_super: false,
            requires_alt: false,
            requires_shift: false,
            requires_ctrl: false,
        }];
        set_wayland_allowed_keys(keys);

        let modifiers = WaylandModifierState::default();
        assert!(is_wayland_key_allowed(59, &modifiers));
        assert!(!is_wayland_key_allowed(60, &modifiers));

        clear_wayland_allowed_keys();
    }

    #[test]
    fn test_process_wayland_key_event() {
        // Set up exit key as Super+Q
        set_wayland_exit_key_config(16, true, false, false, false);

        // Set up F1 as allowed
        set_wayland_allowed_keys(vec![WaylandAllowedKeyConfig {
            keycode: 59,
            requires_super: false,
            requires_alt: false,
            requires_shift: false,
            requires_ctrl: false,
        }]);

        // Test exit key
        let super_mods = WaylandModifierState {
            super_pressed: true,
            ..Default::default()
        };
        assert_eq!(
            process_wayland_key_event(16, &super_mods),
            WaylandKeyboardResult::ExitRequested
        );

        // Test allowed key
        let no_mods = WaylandModifierState::default();
        assert_eq!(
            process_wayland_key_event(59, &no_mods),
            WaylandKeyboardResult::Allowed
        );

        // Test blocked key
        assert_eq!(
            process_wayland_key_event(30, &no_mods),
            WaylandKeyboardResult::Blocked
        );

        clear_wayland_allowed_keys();
    }

    #[test]
    fn test_wayland_keyboard_result_variants() {
        assert_ne!(
            WaylandKeyboardResult::NotActive,
            WaylandKeyboardResult::ExitRequested
        );
        assert_ne!(
            WaylandKeyboardResult::Allowed,
            WaylandKeyboardResult::Blocked
        );
    }
}
