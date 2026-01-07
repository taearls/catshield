//! Cat Shield - A cat-proof screen overlay for macOS
//!
//! Creates a semi-transparent overlay that:
//! - Blocks all keyboard and mouse input
//! - Keeps the machine awake
//! - Click and hold close button (3 seconds) to exit
//! - Or unlock with configurable keyboard shortcut (default: Cmd+Option+U)
//! - Optional timer-based auto-exit
//!
//! Usage: Run the application, and it will immediately activate the shield.
//! Click and hold the X button in the top-right corner for 3 seconds to exit.
//!
//! Timer: Use --timer or -t to set auto-exit timer:
//!   cat_shield --timer 30m      # Exit after 30 minutes
//!   cat_shield --timer 2h       # Exit after 2 hours
//!   cat_shield -t 45m           # Short form
//!
//! Exit Key: Use --exit-key or -e to set custom exit shortcut:
//!   cat_shield --exit-key "Cmd+Shift+Q"
//!   cat_shield --exit-key "Ctrl+Option+Escape"
//!   cat_shield -e "Cmd+Shift+X"
//!
//! Config File: Persistent settings can be stored in ~/.config/catshield/config.toml:
//!   exit_key = "Cmd+Option+U"
//!
//! Note: Keyboard shortcuts require Accessibility permissions.
//! Go to System Preferences → Security & Privacy → Privacy → Accessibility
//! and add this application.

use clap::Parser;
use objc2::rc::Retained;
use objc2::runtime::ProtocolObject;
use objc2::{define_class, msg_send, MainThreadOnly};
use objc2_app_kit::{
    NSApplication, NSApplicationActivationPolicy, NSBackingStoreType, NSBezierPath, NSButton,
    NSButtonType, NSColor, NSControlSize, NSControlStateValueOff, NSControlStateValueOn, NSEvent,
    NSFont, NSMenu, NSMenuItem, NSPanel, NSPopUpButton, NSScreen, NSSlider, NSStatusBar,
    NSStatusItem, NSTextAlignment, NSTextField, NSView, NSWindow, NSWindowCollectionBehavior,
    NSWindowDelegate, NSWindowStyleMask, NSWorkspace,
};
use objc2_core_foundation::{
    kCFRunLoopCommonModes, kCFRunLoopDefaultMode, CFMachPort, CFRetained, CFString, CGFloat,
    CGPoint, CGRect, CGSize,
};
use objc2_core_graphics::{
    CGEvent, CGEventField, CGEventFlags, CGEventMask, CGEventTapLocation, CGEventTapOptions,
    CGEventTapPlacement, CGEventTapProxy, CGEventType,
};
use objc2_foundation::{
    ns_string, MainThreadMarker, NSNotification, NSObject, NSObjectProtocol, NSString, NSURL,
};
use serde::{Deserialize, Serialize};
use std::cell::Cell;
use std::ffi::c_void;
use std::fs;
use std::path::PathBuf;
use std::process;
use std::ptr::NonNull;
use std::sync::atomic::{AtomicBool, AtomicI64, AtomicPtr, AtomicU64, Ordering};
use std::time::Instant;

// IOKit power management bindings
#[link(name = "IOKit", kind = "framework")]
extern "C" {
    fn IOPMAssertionCreateWithName(
        assertion_type: *const c_void,
        level: u32,
        reason_for_activity: *const c_void,
        assertion_id: *mut u32,
    ) -> i32;
    fn IOPMAssertionRelease(assertion_id: u32) -> i32;
}

// Additional CoreGraphics functions not in objc2-core-graphics
#[link(name = "CoreGraphics", kind = "framework")]
extern "C" {
    fn CGEventTapEnable(tap: *mut c_void, enable: bool);
    fn AXIsProcessTrusted() -> bool;
}

// ApplicationServices framework for accessibility permission prompting
#[link(name = "ApplicationServices", kind = "framework")]
extern "C" {
    fn AXIsProcessTrustedWithOptions(options: *const c_void) -> bool;
}

// CoreFoundation bindings
#[link(name = "CoreFoundation", kind = "framework")]
extern "C" {
    // Run loop source management
    fn CFMachPortCreateRunLoopSource(
        allocator: *const c_void,
        port: *mut c_void,
        order: i64,
    ) -> *mut c_void;
    fn CFRunLoopAddSource(rl: *mut c_void, source: *mut c_void, mode: *const c_void);
    fn CFRunLoopRemoveSource(rl: *mut c_void, source: *mut c_void, mode: *const c_void);

    // Run loop access
    fn CFRunLoopGetCurrent() -> *mut c_void;

    // Timer management
    fn CFRunLoopAddTimer(rl: *mut c_void, timer: *mut c_void, mode: *const c_void);
    fn CFRunLoopTimerCreate(
        allocator: *const c_void,
        fire_date: f64,
        interval: f64,
        flags: u32,
        order: i64,
        callout: unsafe extern "C" fn(*mut c_void, *mut c_void),
        context: *const c_void,
    ) -> *mut c_void;
    fn CFRunLoopTimerInvalidate(timer: *mut c_void);
    fn CFAbsoluteTimeGetCurrent() -> f64;

    // Run loop execution (for polling with event processing)
    fn CFRunLoopRunInMode(
        mode: *const c_void,
        seconds: f64,
        return_after_source_handled: bool,
    ) -> i32;

    // Dictionary creation for accessibility options
    static kCFBooleanTrue: *const c_void;
    fn CFDictionaryCreate(
        allocator: *const c_void,
        keys: *const *const c_void,
        values: *const *const c_void,
        num_values: isize,
        key_callbacks: *const c_void,
        value_callbacks: *const c_void,
    ) -> *mut c_void;
    fn CFRelease(cf: *const c_void);
}

// Accessibility options key
#[link(name = "ApplicationServices", kind = "framework")]
extern "C" {
    static kAXTrustedCheckOptionPrompt: *const c_void;
}

const K_IOPM_ASSERTION_LEVEL_ON: u32 = 255;

// Default exit key configuration
const DEFAULT_EXIT_KEY: &str = "Cmd+Option+U";

// macOS virtual key codes
// See: https://developer.apple.com/documentation/coregraphics/cgkeycode
fn keycode_from_name(name: &str) -> Option<i64> {
    match name.to_lowercase().as_str() {
        // Letters
        "a" => Some(0),
        "s" => Some(1),
        "d" => Some(2),
        "f" => Some(3),
        "h" => Some(4),
        "g" => Some(5),
        "z" => Some(6),
        "x" => Some(7),
        "c" => Some(8),
        "v" => Some(9),
        "b" => Some(11),
        "q" => Some(12),
        "w" => Some(13),
        "e" => Some(14),
        "r" => Some(15),
        "y" => Some(16),
        "t" => Some(17),
        "1" | "!" => Some(18),
        "2" | "@" => Some(19),
        "3" | "#" => Some(20),
        "4" | "$" => Some(21),
        "6" | "^" => Some(22),
        "5" | "%" => Some(23),
        "=" | "+" => Some(24),
        "9" | "(" => Some(25),
        "7" | "&" => Some(26),
        "-" | "_" => Some(27),
        "8" | "*" => Some(28),
        "0" | ")" => Some(29),
        "]" | "}" => Some(30),
        "o" => Some(31),
        "u" => Some(32),
        "[" | "{" => Some(33),
        "i" => Some(34),
        "p" => Some(35),
        "l" => Some(37),
        "j" => Some(38),
        "'" | "\"" => Some(39),
        "k" => Some(40),
        ";" | ":" => Some(41),
        "\\" | "|" => Some(42),
        "," | "<" => Some(43),
        "/" | "?" => Some(44),
        "n" => Some(45),
        "m" => Some(46),
        "." | ">" => Some(47),
        "`" | "~" => Some(50),
        // Special keys
        "return" | "enter" => Some(36),
        "tab" => Some(48),
        "space" => Some(49),
        "delete" | "backspace" => Some(51),
        "escape" | "esc" => Some(53),
        "f1" => Some(122),
        "f2" => Some(120),
        "f3" => Some(99),
        "f4" => Some(118),
        "f5" => Some(96),
        "f6" => Some(97),
        "f7" => Some(98),
        "f8" => Some(100),
        "f9" => Some(101),
        "f10" => Some(109),
        "f11" => Some(103),
        "f12" => Some(111),
        "home" => Some(115),
        "end" => Some(119),
        "pageup" => Some(116),
        "pagedown" => Some(121),
        "left" | "leftarrow" => Some(123),
        "right" | "rightarrow" => Some(124),
        "down" | "downarrow" => Some(125),
        "up" | "uparrow" => Some(126),
        _ => None,
    }
}

/// Represents a parsed exit key combination
#[derive(Debug, Clone)]
struct ExitKey {
    keycode: i64,
    requires_cmd: bool,
    requires_option: bool,
    requires_shift: bool,
    requires_ctrl: bool,
    display_name: String,
}

impl Default for ExitKey {
    fn default() -> Self {
        // Default: Cmd+Option+U
        ExitKey {
            keycode: 32, // U
            requires_cmd: true,
            requires_option: true,
            requires_shift: false,
            requires_ctrl: false,
            display_name: DEFAULT_EXIT_KEY.to_string(),
        }
    }
}

impl ExitKey {
    /// Parse a key combination string like "Cmd+Option+U" or "Ctrl+Shift+Escape"
    fn parse(input: &str) -> Result<Self, String> {
        let input = input.trim();
        if input.is_empty() {
            return Err("Exit key cannot be empty".to_string());
        }

        let parts: Vec<&str> = input.split('+').map(|s| s.trim()).collect();
        if parts.is_empty() {
            return Err("Invalid key combination format".to_string());
        }

        let mut requires_cmd = false;
        let mut requires_option = false;
        let mut requires_shift = false;
        let mut requires_ctrl = false;
        let mut key_name: Option<&str> = None;

        for part in &parts {
            let lower = part.to_lowercase();
            match lower.as_str() {
                "cmd" | "command" | "⌘" => requires_cmd = true,
                "opt" | "option" | "alt" | "⌥" => requires_option = true,
                "shift" | "⇧" => requires_shift = true,
                "ctrl" | "control" | "⌃" => requires_ctrl = true,
                _ => {
                    if let Some(existing) = key_name {
                        return Err(format!(
                            "Multiple keys specified: '{}' and '{}'",
                            existing, part
                        ));
                    }
                    key_name = Some(part);
                }
            }
        }

        let key_name = key_name.ok_or("No key specified in combination")?;
        let keycode = keycode_from_name(key_name)
            .ok_or_else(|| format!("Unknown key: '{}'. Valid keys include: A-Z, 0-9, F1-F12, Escape, Return, Tab, Space, Delete, Arrow keys", key_name))?;

        // Require at least one modifier
        if !requires_cmd && !requires_option && !requires_shift && !requires_ctrl {
            return Err(
                "At least one modifier key required (Cmd, Option, Shift, or Ctrl)".to_string(),
            );
        }

        Ok(ExitKey {
            keycode,
            requires_cmd,
            requires_option,
            requires_shift,
            requires_ctrl,
            display_name: input.to_string(),
        })
    }
}

// Global storage for exit key configuration (atomic for thread safety)
static EXIT_KEY_KEYCODE: AtomicI64 = AtomicI64::new(32); // Default: U
static EXIT_KEY_REQUIRES_CMD: AtomicBool = AtomicBool::new(true);
static EXIT_KEY_REQUIRES_OPTION: AtomicBool = AtomicBool::new(true);
static EXIT_KEY_REQUIRES_SHIFT: AtomicBool = AtomicBool::new(false);
static EXIT_KEY_REQUIRES_CTRL: AtomicBool = AtomicBool::new(false);

/// Set the global exit key configuration
fn set_exit_key(key: &ExitKey) {
    EXIT_KEY_KEYCODE.store(key.keycode, Ordering::SeqCst);
    EXIT_KEY_REQUIRES_CMD.store(key.requires_cmd, Ordering::SeqCst);
    EXIT_KEY_REQUIRES_OPTION.store(key.requires_option, Ordering::SeqCst);
    EXIT_KEY_REQUIRES_SHIFT.store(key.requires_shift, Ordering::SeqCst);
    EXIT_KEY_REQUIRES_CTRL.store(key.requires_ctrl, Ordering::SeqCst);
}

/// Get the current exit key configuration from global storage
fn get_exit_key() -> ExitKey {
    ExitKey {
        keycode: EXIT_KEY_KEYCODE.load(Ordering::SeqCst),
        requires_cmd: EXIT_KEY_REQUIRES_CMD.load(Ordering::SeqCst),
        requires_option: EXIT_KEY_REQUIRES_OPTION.load(Ordering::SeqCst),
        requires_shift: EXIT_KEY_REQUIRES_SHIFT.load(Ordering::SeqCst),
        requires_ctrl: EXIT_KEY_REQUIRES_CTRL.load(Ordering::SeqCst),
        display_name: format_exit_key_display(),
    }
}

/// Format the exit key for display
fn format_exit_key_display() -> String {
    let mut parts = Vec::new();
    if EXIT_KEY_REQUIRES_CMD.load(Ordering::SeqCst) {
        parts.push("Cmd");
    }
    if EXIT_KEY_REQUIRES_OPTION.load(Ordering::SeqCst) {
        parts.push("Option");
    }
    if EXIT_KEY_REQUIRES_SHIFT.load(Ordering::SeqCst) {
        parts.push("Shift");
    }
    if EXIT_KEY_REQUIRES_CTRL.load(Ordering::SeqCst) {
        parts.push("Ctrl");
    }
    let keycode = EXIT_KEY_KEYCODE.load(Ordering::SeqCst);
    if let Some(key_name) = keycode_to_name(keycode) {
        parts.push(key_name);
    }
    parts.join("+")
}

/// Convert keycode back to key name for display
fn keycode_to_name(keycode: i64) -> Option<&'static str> {
    match keycode {
        // Letters
        0 => Some("A"),
        11 => Some("B"),
        8 => Some("C"),
        2 => Some("D"),
        14 => Some("E"),
        3 => Some("F"),
        5 => Some("G"),
        4 => Some("H"),
        34 => Some("I"),
        38 => Some("J"),
        40 => Some("K"),
        37 => Some("L"),
        46 => Some("M"),
        45 => Some("N"),
        31 => Some("O"),
        35 => Some("P"),
        12 => Some("Q"),
        15 => Some("R"),
        1 => Some("S"),
        17 => Some("T"),
        32 => Some("U"),
        9 => Some("V"),
        13 => Some("W"),
        7 => Some("X"),
        16 => Some("Y"),
        6 => Some("Z"),
        // Numbers
        18 => Some("1"),
        19 => Some("2"),
        20 => Some("3"),
        21 => Some("4"),
        23 => Some("5"),
        22 => Some("6"),
        26 => Some("7"),
        28 => Some("8"),
        25 => Some("9"),
        29 => Some("0"),
        // Punctuation and symbols
        24 => Some("="),
        27 => Some("-"),
        30 => Some("]"),
        33 => Some("["),
        39 => Some("'"),
        41 => Some(";"),
        42 => Some("\\"),
        43 => Some(","),
        44 => Some("/"),
        47 => Some("."),
        50 => Some("`"),
        // Special keys
        53 => Some("Escape"),
        36 => Some("Return"),
        48 => Some("Tab"),
        49 => Some("Space"),
        51 => Some("Delete"),
        // Function keys
        122 => Some("F1"),
        120 => Some("F2"),
        99 => Some("F3"),
        118 => Some("F4"),
        96 => Some("F5"),
        97 => Some("F6"),
        98 => Some("F7"),
        100 => Some("F8"),
        101 => Some("F9"),
        109 => Some("F10"),
        103 => Some("F11"),
        111 => Some("F12"),
        // Navigation keys
        115 => Some("Home"),
        119 => Some("End"),
        116 => Some("PageUp"),
        121 => Some("PageDown"),
        123 => Some("Left"),
        124 => Some("Right"),
        125 => Some("Down"),
        126 => Some("Up"),
        _ => None,
    }
}

/// Check if the given key event matches the configured exit key
fn check_exit_key(keycode: i64, flags: CGEventFlags) -> bool {
    let expected_keycode = EXIT_KEY_KEYCODE.load(Ordering::SeqCst);
    if keycode != expected_keycode {
        return false;
    }

    let has_cmd = flags.contains(CGEventFlags::MaskCommand);
    let has_option = flags.contains(CGEventFlags::MaskAlternate);
    let has_shift = flags.contains(CGEventFlags::MaskShift);
    let has_ctrl = flags.contains(CGEventFlags::MaskControl);

    let requires_cmd = EXIT_KEY_REQUIRES_CMD.load(Ordering::SeqCst);
    let requires_option = EXIT_KEY_REQUIRES_OPTION.load(Ordering::SeqCst);
    let requires_shift = EXIT_KEY_REQUIRES_SHIFT.load(Ordering::SeqCst);
    let requires_ctrl = EXIT_KEY_REQUIRES_CTRL.load(Ordering::SeqCst);

    requires_cmd == has_cmd
        && requires_option == has_option
        && requires_shift == has_shift
        && requires_ctrl == has_ctrl
}

/// Configuration file structure for persistent settings
#[derive(Debug, Deserialize, Serialize, Default, Clone)]
struct Config {
    /// Custom exit key combination (e.g., "Cmd+Option+U")
    exit_key: Option<String>,

    /// Default auto-exit timer duration (e.g., "30m", "1h")
    default_timer: Option<String>,

    /// Overlay opacity (0.2 to 0.8, default 0.5)
    overlay_opacity: Option<f64>,
}

/// Default overlay opacity when not specified
const DEFAULT_OVERLAY_OPACITY: f64 = 0.5;
/// Minimum allowed overlay opacity
const MIN_OVERLAY_OPACITY: f64 = 0.2;
/// Maximum allowed overlay opacity
const MAX_OVERLAY_OPACITY: f64 = 0.8;

impl Config {
    /// Get the path to the config file (~/.config/catshield/config.toml)
    fn config_path() -> Option<PathBuf> {
        dirs::config_dir().map(|p| p.join("catshield").join("config.toml"))
    }

    /// Load configuration from the config file, if it exists
    fn load() -> Self {
        let Some(path) = Self::config_path() else {
            return Self::default();
        };

        if !path.exists() {
            return Self::default();
        }

        match fs::read_to_string(&path) {
            Ok(contents) => match toml::from_str(&contents) {
                Ok(config) => config,
                Err(e) => {
                    eprintln!("  ⚠️  Warning: Failed to parse config file: {}", e);
                    Self::default()
                }
            },
            Err(e) => {
                eprintln!("  ⚠️  Warning: Failed to read config file: {}", e);
                Self::default()
            }
        }
    }

    /// Get the overlay opacity, clamped to valid range
    fn opacity(&self) -> f64 {
        self.overlay_opacity
            .unwrap_or(DEFAULT_OVERLAY_OPACITY)
            .clamp(MIN_OVERLAY_OPACITY, MAX_OVERLAY_OPACITY)
    }

    /// Save configuration to the config file
    fn save(&self) -> Result<(), String> {
        let path = Self::config_path().ok_or("Could not determine config path")?;

        // Create directory if it doesn't exist
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create config directory: {}", e))?;
        }

        let content = toml::to_string_pretty(self)
            .map_err(|e| format!("Failed to serialize config: {}", e))?;
        fs::write(&path, content).map_err(|e| format!("Failed to write config file: {}", e))?;

        Ok(())
    }

    /// Load configuration from a specific path (for testing)
    #[cfg(test)]
    fn load_from_path(path: &std::path::Path) -> Self {
        if !path.exists() {
            return Self::default();
        }

        match fs::read_to_string(path) {
            Ok(contents) => toml::from_str(&contents).unwrap_or_default(),
            Err(_) => Self::default(),
        }
    }

    /// Save configuration to a specific path (for testing)
    #[cfg(test)]
    fn save_to_path(&self, path: &std::path::Path) -> Result<(), String> {
        // Create directory if it doesn't exist
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create config directory: {}", e))?;
        }

        let content = toml::to_string_pretty(self)
            .map_err(|e| format!("Failed to serialize config: {}", e))?;
        fs::write(path, content).map_err(|e| format!("Failed to write config file: {}", e))?;

        Ok(())
    }
}

/// Parse a timer string (e.g., "30m", "2h", "90s") into a numeric value and unit index.
/// Returns (value_string, unit_index) where unit_index is: 0=minutes, 1=hours, 2=seconds
fn parse_timer_value_and_unit(timer_str: &str) -> (String, isize) {
    let trimmed = timer_str.trim().to_lowercase();

    // Try to extract number and unit
    if let Some(pos) = trimmed.find(|c: char| c.is_alphabetic()) {
        let (num_part, unit_part) = trimmed.split_at(pos);
        let unit_index = match unit_part {
            "h" | "hr" | "hrs" | "hour" | "hours" => 1,
            "s" | "sec" | "secs" | "second" | "seconds" => 2,
            _ => 0, // Default to minutes for "m", "min", etc.
        };
        (num_part.to_string(), unit_index)
    } else {
        // Just a number, assume minutes
        (trimmed, 0)
    }
}

// Close button configuration
const CLOSE_BUTTON_SIZE: CGFloat = 80.0; // Large, easy-to-see button
const CLOSE_BUTTON_MARGIN: CGFloat = 30.0;
const CLOSE_BUTTON_LABEL_HEIGHT: CGFloat = 30.0;
const CLOSE_BUTTON_LABEL_WIDTH: CGFloat = 120.0;
const HOLD_DURATION_SECS: f64 = 3.0;
const TIMER_INTERVAL_SECS: f64 = 1.0 / 60.0; // 60 FPS for smooth animation

// Window levels from NSWindow.h
const NS_SCREEN_SAVER_WINDOW_LEVEL: isize = 1000;

// Timer configuration
const MIN_TIMER_SECONDS: u64 = 60; // Minimum 1 minute
const MAX_TIMER_SECONDS: u64 = 24 * 60 * 60; // Maximum 24 hours
const WARNING_SECONDS: u64 = 60; // Show warning 1 minute before exit

// Timer display configuration
const TIMER_DISPLAY_HEIGHT: CGFloat = 70.0;
const TIMER_DISPLAY_WIDTH: CGFloat = 260.0;
const TIMER_DISPLAY_MARGIN: CGFloat = 30.0;

/// CLI arguments for Cat Shield
#[derive(Parser, Debug)]
#[command(name = "cat_shield")]
#[command(author = "Tyler Earls")]
#[command(version)]
#[command(about = "A cat-proof screen overlay that keeps your machine awake and blocks input")]
#[command(after_help = "EXAMPLES:
    cat_shield                          # Use default exit key (Cmd+Option+U)
    cat_shield --exit-key \"Cmd+Shift+Q\" # Custom exit shortcut
    cat_shield --timer 30m              # Auto-exit after 30 minutes
    cat_shield -e \"Ctrl+Option+X\" -t 2h # Custom key + timer

CONFIG FILE:
    Settings can be persisted in ~/.config/catshield/config.toml:

    exit_key = \"Cmd+Shift+Escape\"

SUPPORTED KEYS:
    Letters: A-Z
    Numbers: 0-9
    Function keys: F1-F12
    Special: Escape, Return, Tab, Space, Delete
    Arrow keys: Left, Right, Up, Down, Home, End, PageUp, PageDown

MODIFIERS:
    Cmd (Command), Option (Alt), Shift, Ctrl (Control)")]
struct Args {
    /// Auto-exit after specified duration (e.g., 30m, 2h, 1h30m)
    #[arg(short, long, value_parser = parse_duration)]
    timer: Option<u64>,

    /// Hide the countdown timer display
    #[arg(long)]
    hide_timer: bool,

    /// Custom exit keyboard shortcut (e.g., "Cmd+Shift+Q", "Ctrl+Option+Escape")
    /// Requires at least one modifier key (Cmd, Option, Shift, or Ctrl).
    /// CLI argument overrides config file setting.
    #[arg(short = 'e', long = "exit-key", value_parser = parse_exit_key)]
    exit_key: Option<ExitKey>,
}

/// Parse exit key string into ExitKey struct (for clap value_parser)
fn parse_exit_key(s: &str) -> Result<ExitKey, String> {
    ExitKey::parse(s)
}

/// Parse duration string like "30m", "2h", "1h30m" into seconds
fn parse_duration(s: &str) -> Result<u64, String> {
    let s = s.trim().to_lowercase();
    if s.is_empty() {
        return Err("Duration cannot be empty".to_string());
    }

    let mut total_seconds: u64 = 0;
    let mut current_num = String::new();

    for c in s.chars() {
        if c.is_ascii_digit() {
            current_num.push(c);
        } else if c == 'h' {
            if current_num.is_empty() {
                return Err("Missing number before 'h'".to_string());
            }
            let hours: u64 = current_num
                .parse()
                .map_err(|_| format!("Invalid number: {}", current_num))?;
            total_seconds += hours * 3600;
            current_num.clear();
        } else if c == 'm' {
            if current_num.is_empty() {
                return Err("Missing number before 'm'".to_string());
            }
            let minutes: u64 = current_num
                .parse()
                .map_err(|_| format!("Invalid number: {}", current_num))?;
            total_seconds += minutes * 60;
            current_num.clear();
        } else if c == 's' {
            if current_num.is_empty() {
                return Err("Missing number before 's'".to_string());
            }
            let secs: u64 = current_num
                .parse()
                .map_err(|_| format!("Invalid number: {}", current_num))?;
            total_seconds += secs;
            current_num.clear();
        } else if !c.is_whitespace() {
            return Err(format!("Invalid character in duration: '{}'", c));
        }
    }

    // If there are remaining digits without a unit, assume minutes
    if !current_num.is_empty() {
        let minutes: u64 = current_num
            .parse()
            .map_err(|_| format!("Invalid number: {}", current_num))?;
        total_seconds += minutes * 60;
    }

    if total_seconds == 0 {
        return Err("Duration must be greater than zero".to_string());
    }

    if total_seconds < MIN_TIMER_SECONDS {
        return Err(format!(
            "Duration must be at least {} seconds (1 minute)",
            MIN_TIMER_SECONDS
        ));
    }

    if total_seconds > MAX_TIMER_SECONDS {
        return Err(format!(
            "Duration must not exceed {} seconds (24 hours)",
            MAX_TIMER_SECONDS
        ));
    }

    Ok(total_seconds)
}

/// Calculate hold progress as a value from 0.0 to 1.0.
///
/// # Arguments
/// * `elapsed_secs` - Time elapsed since mouse down in seconds
/// * `hold_duration_secs` - Required hold duration in seconds
///
/// # Returns
/// Progress value clamped to range [0.0, 1.0]
#[inline]
fn calculate_hold_progress(elapsed_secs: f64, hold_duration_secs: f64) -> f64 {
    (elapsed_secs / hold_duration_secs).min(1.0)
}

/// Check if the hold duration has been met.
///
/// # Arguments
/// * `elapsed_secs` - Time elapsed since mouse down in seconds
/// * `hold_duration_secs` - Required hold duration in seconds
///
/// # Returns
/// `true` if the hold duration has been met or exceeded
#[inline]
fn is_hold_complete(elapsed_secs: f64, hold_duration_secs: f64) -> bool {
    elapsed_secs >= hold_duration_secs
}

/// Create a configured NSTextField label.
///
/// Creates a non-editable, non-selectable text field suitable for use as a label.
/// The text field has no border and can have a transparent or colored background.
fn create_label(
    mtm: MainThreadMarker,
    text: &str,
    frame: CGRect,
    font_size: CGFloat,
    text_color: &NSColor,
    is_bold: bool,
) -> Retained<NSTextField> {
    let label = NSTextField::new(mtm);
    label.setStringValue(&NSString::from_str(text));
    label.setEditable(false);
    label.setSelectable(false);
    label.setBordered(false);
    label.setDrawsBackground(false);
    label.setTextColor(Some(text_color));

    let font = if is_bold {
        NSFont::boldSystemFontOfSize(font_size)
    } else {
        NSFont::systemFontOfSize(font_size)
    };
    label.setFont(Some(&font));
    label.setFrame(frame);

    label
}

// Global timer reference for cleanup
static TIMER_REF: AtomicPtr<c_void> = AtomicPtr::new(std::ptr::null_mut());

// Global view reference for timer callback
static CLOSE_BUTTON_VIEW: AtomicPtr<c_void> = AtomicPtr::new(std::ptr::null_mut());

// Global pointer to the event tap for re-enabling from callback
static EVENT_TAP: AtomicPtr<c_void> = AtomicPtr::new(std::ptr::null_mut());
// Global pointer to the event tap's run loop source for cleanup
static EVENT_TAP_RUN_LOOP_SOURCE: AtomicPtr<c_void> = AtomicPtr::new(std::ptr::null_mut());

// Global timer state for auto-exit feature
static AUTO_EXIT_ENABLED: AtomicBool = AtomicBool::new(false);
static AUTO_EXIT_START_TIME: AtomicU64 = AtomicU64::new(0);
static AUTO_EXIT_DURATION_SECS: AtomicU64 = AtomicU64::new(0);
static WARNING_SHOWN: AtomicBool = AtomicBool::new(false);

// Global reference to the timer display view for updates
static TIMER_DISPLAY_VIEW: AtomicPtr<c_void> = AtomicPtr::new(std::ptr::null_mut());

// Global references to timer display labels (NSTextField)
static TIMER_HEADER_LABEL: AtomicPtr<c_void> = AtomicPtr::new(std::ptr::null_mut());
static TIMER_TIME_LABEL: AtomicPtr<c_void> = AtomicPtr::new(std::ptr::null_mut());
static TIMER_WARNING_LABEL: AtomicPtr<c_void> = AtomicPtr::new(std::ptr::null_mut());

// Global reference to close button label (NSTextField)
static CLOSE_BUTTON_TEXT_LABEL: AtomicPtr<c_void> = AtomicPtr::new(std::ptr::null_mut());

// Shield state for on-demand activation (Issue #17)
// Track whether we're in menu bar mode (app stays running after shield deactivates)
static MENU_BAR_MODE: AtomicBool = AtomicBool::new(false);
// Track whether the shield is currently active
static SHIELD_ACTIVE: AtomicBool = AtomicBool::new(false);
// Global reference to the shield window for cleanup
static SHIELD_WINDOW: AtomicPtr<c_void> = AtomicPtr::new(std::ptr::null_mut());
// Global reference to the "Start Protection" menu item for enabling/disabling
static START_MENU_ITEM: AtomicPtr<c_void> = AtomicPtr::new(std::ptr::null_mut());
// Global storage for sleep assertion ID so we can release it on shield deactivate
static SLEEP_ASSERTION_ID: AtomicU64 = AtomicU64::new(0);
static HAS_SLEEP_ASSERTION: AtomicBool = AtomicBool::new(false);

// Close button state stored in thread-local for the view
thread_local! {
    static MOUSE_DOWN_TIME: Cell<Option<Instant>> = const { Cell::new(None) };
    static IS_MOUSE_INSIDE: Cell<bool> = const { Cell::new(false) };
}

// Timer callback to update progress, check for exit condition, and trigger redraw
unsafe extern "C" fn timer_callback(_timer: *mut c_void, _info: *mut c_void) {
    // Check if hold duration has been exceeded (close button)
    let should_exit_from_button = MOUSE_DOWN_TIME.with(|time| {
        if let Some(start) = time.get() {
            let is_inside = IS_MOUSE_INSIDE.with(|inside| inside.get());
            is_inside && is_hold_complete(start.elapsed().as_secs_f64(), HOLD_DURATION_SECS)
        } else {
            false
        }
    });

    if should_exit_from_button {
        // In menu bar mode, deactivate shield and return to menu bar
        // In immediate mode, terminate the app
        if MENU_BAR_MODE.load(Ordering::SeqCst) {
            deactivate_shield();
        } else if let Some(mtm) = MainThreadMarker::new() {
            let app = NSApplication::sharedApplication(mtm);
            app.terminate(None);
        }
        return;
    }

    // Check auto-exit timer
    if AUTO_EXIT_ENABLED.load(Ordering::SeqCst) {
        let remaining = get_remaining_seconds();

        // Show warning when approaching exit
        if remaining <= WARNING_SECONDS && !WARNING_SHOWN.swap(true, Ordering::SeqCst) {
            println!();
            println!("  ⚠️  Auto-exit in {} seconds!", remaining);
            println!();
        }

        // Check if timer has expired
        if remaining == 0 {
            println!();
            println!("  ⏰ Timer expired - auto-exiting...");
            // In menu bar mode, deactivate shield and return to menu bar
            // In immediate mode, terminate the app
            if MENU_BAR_MODE.load(Ordering::SeqCst) {
                deactivate_shield();
            } else if let Some(mtm) = MainThreadMarker::new() {
                let app = NSApplication::sharedApplication(mtm);
                app.terminate(None);
            }
            return;
        }
    }

    // Trigger redraw of close button
    let view_ptr = CLOSE_BUTTON_VIEW.load(Ordering::SeqCst);
    if !view_ptr.is_null() {
        let view: &NSView = &*(view_ptr as *const NSView);
        view.setNeedsDisplay(true);
    }

    // Trigger redraw of close button label (for countdown during hold)
    let label_view_ptr = CLOSE_BUTTON_LABEL_VIEW.load(Ordering::SeqCst);
    if !label_view_ptr.is_null() {
        let view: &NSView = &*(label_view_ptr as *const NSView);
        view.setNeedsDisplay(true);
    }

    // Trigger redraw of timer display
    let timer_view_ptr = TIMER_DISPLAY_VIEW.load(Ordering::SeqCst);
    if !timer_view_ptr.is_null() {
        let view: &NSView = &*(timer_view_ptr as *const NSView);
        view.setNeedsDisplay(true);
    }
}

/// Start the animation timer for the close button
fn start_close_button_timer() {
    unsafe {
        let timer = CFRunLoopTimerCreate(
            std::ptr::null(),
            CFAbsoluteTimeGetCurrent() + TIMER_INTERVAL_SECS,
            TIMER_INTERVAL_SECS,
            0,
            0,
            timer_callback,
            std::ptr::null(),
        );

        if !timer.is_null() {
            let run_loop = CFRunLoopGetCurrent();
            let mode = kCFRunLoopCommonModes.expect("kCFRunLoopCommonModes should exist");
            CFRunLoopAddTimer(run_loop, timer, (mode as *const CFString) as *const c_void);
            TIMER_REF.store(timer, Ordering::SeqCst);
        }
    }
}

/// Stop the animation timer
fn stop_close_button_timer() {
    unsafe {
        let timer = TIMER_REF.swap(std::ptr::null_mut(), Ordering::SeqCst);
        if !timer.is_null() {
            CFRunLoopTimerInvalidate(timer);
        }
    }
}

/// Initialize the auto-exit timer with the specified duration in seconds
fn init_auto_exit_timer(duration_secs: u64) {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    AUTO_EXIT_START_TIME.store(now, Ordering::SeqCst);
    AUTO_EXIT_DURATION_SECS.store(duration_secs, Ordering::SeqCst);
    AUTO_EXIT_ENABLED.store(true, Ordering::SeqCst);
}

/// Get the remaining seconds until auto-exit, or 0 if expired
fn get_remaining_seconds() -> u64 {
    if !AUTO_EXIT_ENABLED.load(Ordering::SeqCst) {
        return u64::MAX;
    }

    let start = AUTO_EXIT_START_TIME.load(Ordering::SeqCst);
    let duration = AUTO_EXIT_DURATION_SECS.load(Ordering::SeqCst);
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();

    let elapsed = now.saturating_sub(start);
    duration.saturating_sub(elapsed)
}

/// Format seconds as a human-readable string (e.g., "1h 30m 45s")
fn format_duration(total_secs: u64) -> String {
    let hours = total_secs / 3600;
    let minutes = (total_secs % 3600) / 60;
    let secs = total_secs % 60;

    if hours > 0 {
        format!("{}h {:02}m {:02}s", hours, minutes, secs)
    } else if minutes > 0 {
        format!("{}m {:02}s", minutes, secs)
    } else {
        format!("{}s", secs)
    }
}

/// Ivars for the TimerDisplayView
struct TimerDisplayViewIvars {}

define_class!(
    #[unsafe(super(NSView))]
    #[name = "TimerDisplayView"]
    #[ivars = TimerDisplayViewIvars]
    struct TimerDisplayView;

    impl TimerDisplayView {
        #[unsafe(method(drawRect:))]
        unsafe fn draw_rect(&self, _dirty_rect: CGRect) {
            draw_timer_display(self);
        }
    }
);

impl TimerDisplayView {
    fn new(mtm: MainThreadMarker, frame: CGRect) -> Retained<Self> {
        let this = mtm.alloc::<TimerDisplayView>();
        let this = this.set_ivars(TimerDisplayViewIvars {});
        unsafe { msg_send![super(this), initWithFrame: frame] }
    }
}

/// Draw the timer countdown display
fn draw_timer_display(view: &NSView) {
    let bounds = view.bounds();
    let remaining = get_remaining_seconds();
    let is_warning = remaining <= WARNING_SECONDS;

    // Background rounded rectangle (fully opaque to block overlay behind)
    let bg_color = if is_warning {
        // Red/orange warning color
        NSColor::colorWithRed_green_blue_alpha(0.8, 0.3, 0.1, 1.0)
    } else {
        // Dark opaque background
        NSColor::colorWithRed_green_blue_alpha(0.1, 0.1, 0.15, 1.0)
    };
    bg_color.set();

    let corner_radius = 10.0;
    let bg_rect = CGRect {
        origin: CGPoint { x: 0.0, y: 0.0 },
        size: bounds.size,
    };
    let bg_path = NSBezierPath::bezierPathWithRoundedRect_xRadius_yRadius(
        bg_rect,
        corner_radius,
        corner_radius,
    );
    bg_path.fill();

    // Border
    let border_color = if is_warning {
        NSColor::colorWithRed_green_blue_alpha(1.0, 0.5, 0.2, 1.0)
    } else {
        NSColor::colorWithRed_green_blue_alpha(0.5, 0.5, 0.5, 0.8)
    };
    border_color.set();
    bg_path.setLineWidth(2.0);
    bg_path.stroke();

    // Format time string
    let time_str = format_duration(remaining);

    // Text colors
    let text_color = NSColor::colorWithRed_green_blue_alpha(1.0, 1.0, 1.0, 1.0);
    let label_color = NSColor::colorWithRed_green_blue_alpha(0.8, 0.8, 0.8, 1.0);
    let warning_color = NSColor::colorWithRed_green_blue_alpha(1.0, 1.0, 0.0, 1.0);

    // Get main thread marker (we're guaranteed to be on main thread in drawRect)
    let mtm = unsafe { MainThreadMarker::new_unchecked() };

    // Create or update NSTextField labels
    let header_ptr = TIMER_HEADER_LABEL.load(Ordering::SeqCst);
    let time_ptr = TIMER_TIME_LABEL.load(Ordering::SeqCst);
    let warning_ptr = TIMER_WARNING_LABEL.load(Ordering::SeqCst);

    // Header label "Time Remaining:"
    let header_y = bounds.size.height - 22.0;
    let header_frame = CGRect {
        origin: CGPoint {
            x: 12.0,
            y: header_y,
        },
        size: CGSize {
            width: 120.0,
            height: 16.0,
        },
    };

    if header_ptr.is_null() {
        // Create the header label
        let header_label = create_label(
            mtm,
            "Time Remaining:",
            header_frame,
            12.0,
            &label_color,
            false,
        );
        TIMER_HEADER_LABEL.store(
            Retained::as_ptr(&header_label) as *mut c_void,
            Ordering::SeqCst,
        );
        view.addSubview(&header_label);
        // Keep retained to prevent deallocation
        std::mem::forget(header_label);
    }

    // Time label (large bold countdown)
    let time_y = bounds.size.height - 48.0;
    let time_frame = CGRect {
        origin: CGPoint { x: 12.0, y: time_y },
        size: CGSize {
            width: 130.0,
            height: 26.0,
        },
    };

    if time_ptr.is_null() {
        // Create the time label
        let time_label = create_label(mtm, &time_str, time_frame, 20.0, &text_color, true);
        TIMER_TIME_LABEL.store(
            Retained::as_ptr(&time_label) as *mut c_void,
            Ordering::SeqCst,
        );
        view.addSubview(&time_label);
        std::mem::forget(time_label);
    } else {
        // Update existing time label
        unsafe {
            let time_label: &NSTextField = &*(time_ptr as *const NSTextField);
            time_label.setStringValue(&NSString::from_str(&time_str));
            let color = if is_warning {
                &warning_color
            } else {
                &text_color
            };
            time_label.setTextColor(Some(color));
        }
    }

    // Warning label "Exiting soon!"
    let warning_frame = CGRect {
        origin: CGPoint {
            x: 140.0,
            y: time_y + 4.0,
        },
        size: CGSize {
            width: 100.0,
            height: 18.0,
        },
    };

    if warning_ptr.is_null() {
        // Create the warning label (initially hidden)
        let warning_label = create_label(
            mtm,
            "Exiting soon!",
            warning_frame,
            14.0,
            &warning_color,
            false,
        );
        warning_label.setHidden(!is_warning);
        TIMER_WARNING_LABEL.store(
            Retained::as_ptr(&warning_label) as *mut c_void,
            Ordering::SeqCst,
        );
        view.addSubview(&warning_label);
        std::mem::forget(warning_label);
    } else {
        // Update existing warning label visibility
        unsafe {
            let warning_label: &NSTextField = &*(warning_ptr as *const NSTextField);
            warning_label.setHidden(!is_warning);
        }
    }

    // Draw a progress bar showing remaining time
    let duration = AUTO_EXIT_DURATION_SECS.load(Ordering::SeqCst);
    let progress = if duration > 0 {
        remaining as f64 / duration as f64
    } else {
        0.0
    };

    // Progress bar background
    let bar_margin = 12.0;
    let bar_height = 8.0;
    let bar_y = 10.0;
    let bar_width = bounds.size.width - (bar_margin * 2.0);

    let bar_bg_color = NSColor::colorWithRed_green_blue_alpha(0.2, 0.2, 0.2, 1.0);
    bar_bg_color.set();

    let bar_bg_rect = CGRect {
        origin: CGPoint {
            x: bar_margin,
            y: bar_y,
        },
        size: CGSize {
            width: bar_width,
            height: bar_height,
        },
    };
    let bar_bg_path =
        NSBezierPath::bezierPathWithRoundedRect_xRadius_yRadius(bar_bg_rect, 4.0, 4.0);
    bar_bg_path.fill();

    // Progress bar fill
    let bar_fill_color = if is_warning {
        NSColor::colorWithRed_green_blue_alpha(1.0, 0.3, 0.1, 1.0)
    } else {
        NSColor::colorWithRed_green_blue_alpha(0.2, 0.8, 0.3, 1.0)
    };
    bar_fill_color.set();

    let fill_width = bar_width * progress;
    if fill_width > 0.0 {
        let bar_fill_rect = CGRect {
            origin: CGPoint {
                x: bar_margin,
                y: bar_y,
            },
            size: CGSize {
                width: fill_width,
                height: bar_height,
            },
        };
        let bar_fill_path =
            NSBezierPath::bezierPathWithRoundedRect_xRadius_yRadius(bar_fill_rect, 4.0, 4.0);
        bar_fill_path.fill();
    }
}

/// Ivars for the CloseButtonView
struct CloseButtonViewIvars {}

define_class!(
    #[unsafe(super(NSView))]
    #[name = "CloseButtonView"]
    #[ivars = CloseButtonViewIvars]
    struct CloseButtonView;

    impl CloseButtonView {
        #[unsafe(method(drawRect:))]
        unsafe fn draw_rect(&self, _dirty_rect: CGRect) {
            draw_close_button(self);
        }

        #[unsafe(method(mouseDown:))]
        unsafe fn mouse_down(&self, _event: &NSEvent) {
            MOUSE_DOWN_TIME.with(|time| {
                time.set(Some(Instant::now()));
            });
            IS_MOUSE_INSIDE.with(|inside| inside.set(true));
            self.setNeedsDisplay(true);
        }

        #[unsafe(method(mouseUp:))]
        unsafe fn mouse_up(&self, _event: &NSEvent) {
            MOUSE_DOWN_TIME.with(|time| {
                time.set(None);
            });
            self.setNeedsDisplay(true);
        }

        #[unsafe(method(mouseDragged:))]
        unsafe fn mouse_dragged(&self, event: &NSEvent) {
            // Check if mouse is still inside the button
            let location = event.locationInWindow();
            let bounds = self.bounds();

            // Convert to view coordinates
            let local_point = self.convertPoint_fromView(location, None);

            let is_inside = local_point.x >= 0.0
                && local_point.x <= bounds.size.width
                && local_point.y >= 0.0
                && local_point.y <= bounds.size.height;

            let was_inside = IS_MOUSE_INSIDE.with(|inside| inside.get());

            if is_inside != was_inside {
                IS_MOUSE_INSIDE.with(|inside| inside.set(is_inside));

                // Reset timer if mouse left the button
                if !is_inside {
                    MOUSE_DOWN_TIME.with(|time| {
                        time.set(None);
                    });
                } else {
                    // Restart timer if mouse re-entered
                    MOUSE_DOWN_TIME.with(|time| {
                        time.set(Some(Instant::now()));
                    });
                }
            }

            self.setNeedsDisplay(true);
        }
    }
);

impl CloseButtonView {
    fn new(mtm: MainThreadMarker, frame: CGRect) -> Retained<Self> {
        let this = mtm.alloc::<CloseButtonView>();
        let this = this.set_ivars(CloseButtonViewIvars {});
        unsafe { msg_send![super(this), initWithFrame: frame] }
    }
}

/// Ivars for the CloseButtonLabelView
struct CloseButtonLabelViewIvars {}

define_class!(
    #[unsafe(super(NSView))]
    #[name = "CloseButtonLabelView"]
    #[ivars = CloseButtonLabelViewIvars]
    struct CloseButtonLabelView;

    impl CloseButtonLabelView {
        #[unsafe(method(drawRect:))]
        unsafe fn draw_rect(&self, _dirty_rect: CGRect) {
            draw_close_button_label(self);
        }
    }
);

impl CloseButtonLabelView {
    fn new(mtm: MainThreadMarker, frame: CGRect) -> Retained<Self> {
        let this = mtm.alloc::<CloseButtonLabelView>();
        let this = this.set_ivars(CloseButtonLabelViewIvars {});
        unsafe { msg_send![super(this), initWithFrame: frame] }
    }
}

// Global reference to the close button label view for updating during hold
static CLOSE_BUTTON_LABEL_VIEW: AtomicPtr<c_void> = AtomicPtr::new(std::ptr::null_mut());

/// Draw the close button label with hold instructions
fn draw_close_button_label(view: &NSView) {
    let bounds = view.bounds();

    // Get current hold progress
    let progress = MOUSE_DOWN_TIME.with(|time| {
        if let Some(start) = time.get() {
            calculate_hold_progress(start.elapsed().as_secs_f64(), HOLD_DURATION_SECS)
        } else {
            0.0
        }
    });
    let is_inside = IS_MOUSE_INSIDE.with(|inside| inside.get());
    let is_holding = progress > 0.0 && is_inside;

    // Background rounded rectangle (fully opaque to block overlay behind)
    let bg_color = if is_holding {
        NSColor::colorWithRed_green_blue_alpha(0.2, 0.2, 0.25, 1.0)
    } else {
        NSColor::colorWithRed_green_blue_alpha(0.1, 0.1, 0.15, 1.0)
    };
    bg_color.set();

    let corner_radius = 8.0;
    let bg_rect = CGRect {
        origin: CGPoint { x: 0.0, y: 0.0 },
        size: bounds.size,
    };
    let bg_path = NSBezierPath::bezierPathWithRoundedRect_xRadius_yRadius(
        bg_rect,
        corner_radius,
        corner_radius,
    );
    bg_path.fill();

    // Border
    let border_color = if is_holding {
        NSColor::colorWithRed_green_blue_alpha(0.4, 0.9, 0.4, 1.0)
    } else {
        NSColor::colorWithRed_green_blue_alpha(0.5, 0.5, 0.5, 0.8)
    };
    border_color.set();
    bg_path.setLineWidth(1.5);
    bg_path.stroke();

    // Text colors
    let hint_color = NSColor::colorWithRed_green_blue_alpha(0.7, 0.7, 0.7, 1.0);
    let progress_color = NSColor::colorWithRed_green_blue_alpha(0.4, 1.0, 0.4, 1.0);

    // Get main thread marker (we're guaranteed to be on main thread in drawRect)
    let mtm = unsafe { MainThreadMarker::new_unchecked() };

    // Create or update NSTextField label
    let label_ptr = CLOSE_BUTTON_TEXT_LABEL.load(Ordering::SeqCst);

    // Calculate centered frame for the label
    let label_frame = CGRect {
        origin: CGPoint { x: 0.0, y: 0.0 },
        size: bounds.size,
    };

    // Determine text and color based on state
    let (text, color, font_size) = if is_holding {
        let remaining_secs = ((1.0 - progress) * HOLD_DURATION_SECS).ceil() as u32;
        (format!("{}s...", remaining_secs), &progress_color, 16.0)
    } else {
        ("Hold 3s to exit".to_string(), &hint_color, 12.0)
    };

    if label_ptr.is_null() {
        // Create the label
        let label = create_label(mtm, &text, label_frame, font_size, color, false);
        // Center the text horizontally
        label.setAlignment(NSTextAlignment::Center);
        CLOSE_BUTTON_TEXT_LABEL.store(Retained::as_ptr(&label) as *mut c_void, Ordering::SeqCst);
        view.addSubview(&label);
        std::mem::forget(label);
    } else {
        // Update existing label
        unsafe {
            let label: &NSTextField = &*(label_ptr as *const NSTextField);
            label.setStringValue(&NSString::from_str(&text));
            label.setTextColor(Some(color));
            let font = NSFont::systemFontOfSize(font_size);
            label.setFont(Some(&font));
        }
    }
}

/// Empty ivars for the MenuActionHandler
struct MenuActionHandlerIvars {}

// Menu action handler for the "Start Protection" menu item
// This class provides a target for the menu item action selector
define_class!(
    #[unsafe(super(NSObject))]
    #[name = "MenuActionHandler"]
    #[ivars = MenuActionHandlerIvars]
    struct MenuActionHandler;

    impl MenuActionHandler {
        /// Action method called when "Start Protection" is clicked
        #[unsafe(method(startProtection:))]
        unsafe fn start_protection(&self, _sender: Option<&NSMenuItem>) {
            // Prevent double-activation
            if SHIELD_ACTIVE.load(Ordering::SeqCst) {
                return;
            }

            // Call the activate_shield function
            if let Some(mtm) = MainThreadMarker::new() {
                activate_shield(mtm);
            }
        }
    }
);

impl MenuActionHandler {
    fn new(mtm: MainThreadMarker) -> Retained<Self> {
        let this = mtm.alloc::<MenuActionHandler>();
        let this = this.set_ivars(MenuActionHandlerIvars {});
        unsafe { msg_send![super(this), init] }
    }
}

// Global reference to the menu action handler to keep it alive
static MENU_ACTION_HANDLER: AtomicPtr<c_void> = AtomicPtr::new(std::ptr::null_mut());

// Global reference to the settings action handler to keep it alive
static SETTINGS_ACTION_HANDLER: AtomicPtr<c_void> = AtomicPtr::new(std::ptr::null_mut());

// Global reference to the settings window to prevent multiple instances
static SETTINGS_WINDOW: AtomicPtr<c_void> = AtomicPtr::new(std::ptr::null_mut());

// Global reference to the settings menu item for enabling/disabling
static SETTINGS_MENU_ITEM: AtomicPtr<c_void> = AtomicPtr::new(std::ptr::null_mut());

// Global reference to the current loaded config
static CURRENT_CONFIG: std::sync::OnceLock<std::sync::Mutex<Config>> = std::sync::OnceLock::new();

/// Get a reference to the current config
fn get_current_config() -> Config {
    CURRENT_CONFIG
        .get_or_init(|| std::sync::Mutex::new(Config::load()))
        .lock()
        .unwrap()
        .clone()
}

/// Update the current config
fn set_current_config(config: Config) {
    let mutex = CURRENT_CONFIG.get_or_init(|| std::sync::Mutex::new(Config::load()));
    *mutex.lock().unwrap() = config;
}

// Settings window UI element references for reading values on save
static SETTINGS_EXIT_KEY_FIELD: AtomicPtr<c_void> = AtomicPtr::new(std::ptr::null_mut());
static SETTINGS_TIMER_VALUE_FIELD: AtomicPtr<c_void> = AtomicPtr::new(std::ptr::null_mut());
static SETTINGS_TIMER_UNIT_DROPDOWN: AtomicPtr<c_void> = AtomicPtr::new(std::ptr::null_mut());
static SETTINGS_TIMER_CHECKBOX: AtomicPtr<c_void> = AtomicPtr::new(std::ptr::null_mut());
static SETTINGS_OPACITY_SLIDER: AtomicPtr<c_void> = AtomicPtr::new(std::ptr::null_mut());
static SETTINGS_OPACITY_LABEL: AtomicPtr<c_void> = AtomicPtr::new(std::ptr::null_mut());
static SETTINGS_EXIT_KEY_VALIDATION_LABEL: AtomicPtr<c_void> = AtomicPtr::new(std::ptr::null_mut());
static SETTINGS_TIMER_VALIDATION_LABEL: AtomicPtr<c_void> = AtomicPtr::new(std::ptr::null_mut());
static SETTINGS_WINDOW_DELEGATE: AtomicPtr<c_void> = AtomicPtr::new(std::ptr::null_mut());

/// Empty ivars for the SettingsActionHandler
struct SettingsActionHandlerIvars {}

/// Empty ivars for the SettingsWindowDelegate
struct SettingsWindowDelegateIvars {}

// Window delegate to handle window close events
define_class!(
    #[unsafe(super(NSObject))]
    #[thread_kind = MainThreadOnly]
    #[name = "SettingsWindowDelegate"]
    #[ivars = SettingsWindowDelegateIvars]
    struct SettingsWindowDelegate;

    unsafe impl NSObjectProtocol for SettingsWindowDelegate {}

    unsafe impl NSWindowDelegate for SettingsWindowDelegate {
        /// Called when the window is about to close (including via X button)
        #[unsafe(method(windowWillClose:))]
        fn window_will_close(&self, _notification: &NSNotification) {
            cleanup_settings_window_references();
        }
    }
);

impl SettingsWindowDelegate {
    fn new(mtm: MainThreadMarker) -> Retained<Self> {
        let this = mtm.alloc::<SettingsWindowDelegate>();
        let this = this.set_ivars(SettingsWindowDelegateIvars {});
        unsafe { msg_send![super(this), init] }
    }
}

// Settings action handler for menu items and buttons
define_class!(
    #[unsafe(super(NSObject))]
    #[name = "SettingsActionHandler"]
    #[ivars = SettingsActionHandlerIvars]
    struct SettingsActionHandler;

    impl SettingsActionHandler {
        /// Action method called when "Settings..." menu item is clicked
        #[unsafe(method(showSettings:))]
        unsafe fn show_settings(&self, _sender: Option<&NSMenuItem>) {
            if let Some(mtm) = MainThreadMarker::new() {
                show_settings_window(mtm);
            }
        }

        /// Action method called when "Save" button is clicked
        #[unsafe(method(saveSettings:))]
        unsafe fn save_settings(&self, _sender: Option<&NSButton>) {
            save_settings_from_window();
        }

        /// Action method called when "Cancel" button is clicked
        #[unsafe(method(cancelSettings:))]
        unsafe fn cancel_settings(&self, _sender: Option<&NSButton>) {
            close_settings_window();
        }

        /// Action method called when opacity slider value changes
        #[unsafe(method(opacityChanged:))]
        unsafe fn opacity_changed(&self, sender: Option<&NSSlider>) {
            if let Some(slider) = sender {
                update_opacity_label(slider.doubleValue());
            }
        }

        /// Action method called when timer checkbox state changes
        #[unsafe(method(timerCheckboxChanged:))]
        unsafe fn timer_checkbox_changed(&self, sender: Option<&NSButton>) {
            if let Some(checkbox) = sender {
                let is_enabled = checkbox.state() == NSControlStateValueOn;
                update_timer_field_enabled(is_enabled);
            }
        }
    }
);

impl SettingsActionHandler {
    fn new(mtm: MainThreadMarker) -> Retained<Self> {
        let this = mtm.alloc::<SettingsActionHandler>();
        let this = this.set_ivars(SettingsActionHandlerIvars {});
        unsafe { msg_send![super(this), init] }
    }
}

/// Update the opacity percentage label
fn update_opacity_label(value: f64) {
    let label_ptr = SETTINGS_OPACITY_LABEL.load(Ordering::SeqCst);
    if !label_ptr.is_null() {
        unsafe {
            let label: &NSTextField = &*(label_ptr as *const NSTextField);
            let percentage = (value * 100.0) as i32;
            label.setStringValue(&NSString::from_str(&format!("{}%", percentage)));
        }
    }
}

/// Update the timer field and dropdown enabled state based on checkbox
fn update_timer_field_enabled(enabled: bool) {
    // Update number field
    let field_ptr = SETTINGS_TIMER_VALUE_FIELD.load(Ordering::SeqCst);
    if !field_ptr.is_null() {
        unsafe {
            let field: &NSTextField = &*(field_ptr as *const NSTextField);
            field.setEnabled(enabled);
            if enabled {
                field.setTextColor(Some(&NSColor::labelColor()));
            } else {
                field.setTextColor(Some(&NSColor::disabledControlTextColor()));
            }
        }
    }

    // Update unit dropdown
    let dropdown_ptr = SETTINGS_TIMER_UNIT_DROPDOWN.load(Ordering::SeqCst);
    if !dropdown_ptr.is_null() {
        unsafe {
            let dropdown: &NSPopUpButton = &*(dropdown_ptr as *const NSPopUpButton);
            dropdown.setEnabled(enabled);
        }
    }
}

/// Clean up settings window UI element references
/// Called both when window is closed programmatically and via X button
fn cleanup_settings_window_references() {
    // Clear window reference (swap to null to avoid double-cleanup)
    SETTINGS_WINDOW.store(std::ptr::null_mut(), Ordering::SeqCst);

    // Clear UI element references
    SETTINGS_EXIT_KEY_FIELD.store(std::ptr::null_mut(), Ordering::SeqCst);
    SETTINGS_TIMER_VALUE_FIELD.store(std::ptr::null_mut(), Ordering::SeqCst);
    SETTINGS_TIMER_UNIT_DROPDOWN.store(std::ptr::null_mut(), Ordering::SeqCst);
    SETTINGS_TIMER_CHECKBOX.store(std::ptr::null_mut(), Ordering::SeqCst);
    SETTINGS_OPACITY_SLIDER.store(std::ptr::null_mut(), Ordering::SeqCst);
    SETTINGS_OPACITY_LABEL.store(std::ptr::null_mut(), Ordering::SeqCst);
    SETTINGS_EXIT_KEY_VALIDATION_LABEL.store(std::ptr::null_mut(), Ordering::SeqCst);
    SETTINGS_TIMER_VALIDATION_LABEL.store(std::ptr::null_mut(), Ordering::SeqCst);

    // Re-enable the settings menu item
    let menu_item_ptr = SETTINGS_MENU_ITEM.load(Ordering::SeqCst);
    if !menu_item_ptr.is_null() {
        unsafe {
            let menu_item: &NSMenuItem = &*(menu_item_ptr as *const NSMenuItem);
            menu_item.setEnabled(true);
        }
    }

    println!("  Settings window closed");
}

/// Close the settings window (called by Cancel/Save buttons)
fn close_settings_window() {
    let window_ptr = SETTINGS_WINDOW.load(Ordering::SeqCst);
    if !window_ptr.is_null() {
        unsafe {
            let window: &NSPanel = &*(window_ptr as *const NSPanel);
            window.close();
        }
        // Note: cleanup_settings_window_references() will be called by the window delegate
    }
}

/// Save settings from the window to config file
fn save_settings_from_window() {
    let mut config = get_current_config();
    let mut has_errors = false;

    // Get exit key value and validate
    let exit_key_ptr = SETTINGS_EXIT_KEY_FIELD.load(Ordering::SeqCst);
    if !exit_key_ptr.is_null() {
        unsafe {
            let field: &NSTextField = &*(exit_key_ptr as *const NSTextField);
            let value = field.stringValue().to_string();
            let trimmed = value.trim();

            if !trimmed.is_empty() {
                match ExitKey::parse(trimmed) {
                    Ok(_) => {
                        config.exit_key = Some(trimmed.to_string());
                        update_validation_label(
                            SETTINGS_EXIT_KEY_VALIDATION_LABEL.load(Ordering::SeqCst),
                            true,
                            "✓ Valid",
                        );
                    }
                    Err(e) => {
                        update_validation_label(
                            SETTINGS_EXIT_KEY_VALIDATION_LABEL.load(Ordering::SeqCst),
                            false,
                            &e,
                        );
                        has_errors = true;
                    }
                }
            } else {
                config.exit_key = None;
                update_validation_label(
                    SETTINGS_EXIT_KEY_VALIDATION_LABEL.load(Ordering::SeqCst),
                    true,
                    "Using default",
                );
            }
        }
    }

    // Get timer value and validate (if checkbox is checked)
    let timer_checkbox_ptr = SETTINGS_TIMER_CHECKBOX.load(Ordering::SeqCst);
    let timer_value_ptr = SETTINGS_TIMER_VALUE_FIELD.load(Ordering::SeqCst);
    let timer_unit_ptr = SETTINGS_TIMER_UNIT_DROPDOWN.load(Ordering::SeqCst);
    if !timer_checkbox_ptr.is_null() && !timer_value_ptr.is_null() && !timer_unit_ptr.is_null() {
        unsafe {
            let checkbox: &NSButton = &*(timer_checkbox_ptr as *const NSButton);
            let value_field: &NSTextField = &*(timer_value_ptr as *const NSTextField);
            let unit_dropdown: &NSPopUpButton = &*(timer_unit_ptr as *const NSPopUpButton);
            let is_enabled = checkbox.state() == NSControlStateValueOn;

            if is_enabled {
                let value_str = value_field.stringValue().to_string();
                let trimmed = value_str.trim();

                if !trimmed.is_empty() {
                    // Parse the number
                    match trimmed.parse::<u64>() {
                        Ok(num) if num > 0 => {
                            // Get the selected unit suffix
                            let unit_index = unit_dropdown.indexOfSelectedItem();
                            let unit_suffix = match unit_index {
                                0 => "m", // Minutes
                                1 => "h", // Hours
                                2 => "s", // Seconds
                                _ => "m", // Default to minutes
                            };

                            // Construct the duration string
                            let duration_str = format!("{}{}", num, unit_suffix);

                            // Validate using parse_duration
                            match parse_duration(&duration_str) {
                                Ok(_) => {
                                    config.default_timer = Some(duration_str);
                                    update_validation_label(
                                        SETTINGS_TIMER_VALIDATION_LABEL.load(Ordering::SeqCst),
                                        true,
                                        "✓ Valid",
                                    );
                                }
                                Err(e) => {
                                    update_validation_label(
                                        SETTINGS_TIMER_VALIDATION_LABEL.load(Ordering::SeqCst),
                                        false,
                                        &e,
                                    );
                                    has_errors = true;
                                }
                            }
                        }
                        Ok(_) => {
                            update_validation_label(
                                SETTINGS_TIMER_VALIDATION_LABEL.load(Ordering::SeqCst),
                                false,
                                "Must be greater than 0",
                            );
                            has_errors = true;
                        }
                        Err(_) => {
                            update_validation_label(
                                SETTINGS_TIMER_VALIDATION_LABEL.load(Ordering::SeqCst),
                                false,
                                "Enter a number",
                            );
                            has_errors = true;
                        }
                    }
                } else {
                    update_validation_label(
                        SETTINGS_TIMER_VALIDATION_LABEL.load(Ordering::SeqCst),
                        false,
                        "Duration required",
                    );
                    has_errors = true;
                }
            } else {
                config.default_timer = None;
                update_validation_label(
                    SETTINGS_TIMER_VALIDATION_LABEL.load(Ordering::SeqCst),
                    true,
                    "",
                );
            }
        }
    }

    // Get opacity value
    let opacity_ptr = SETTINGS_OPACITY_SLIDER.load(Ordering::SeqCst);
    if !opacity_ptr.is_null() {
        unsafe {
            let slider: &NSSlider = &*(opacity_ptr as *const NSSlider);
            config.overlay_opacity = Some(slider.doubleValue());
        }
    }

    if has_errors {
        println!("  ⚠️  Settings have validation errors, please fix them");
        return;
    }

    // Save config to file
    match config.save() {
        Ok(()) => {
            println!("  ✓ Settings saved to config file");
            set_current_config(config.clone());

            // Update the global exit key if changed
            if let Some(ref key_str) = config.exit_key {
                if let Ok(key) = ExitKey::parse(key_str) {
                    set_exit_key(&key);
                    println!("  ✓ Exit key updated to: {}", key.display_name);
                }
            }

            close_settings_window();
        }
        Err(e) => {
            eprintln!("  ✗ Failed to save settings: {}", e);
        }
    }
}

/// Update a validation label with success or error state
fn update_validation_label(label_ptr: *mut c_void, is_valid: bool, message: &str) {
    if !label_ptr.is_null() {
        unsafe {
            let label: &NSTextField = &*(label_ptr as *const NSTextField);
            label.setStringValue(&NSString::from_str(message));
            if is_valid {
                label.setTextColor(Some(&NSColor::systemGreenColor()));
            } else {
                label.setTextColor(Some(&NSColor::systemRedColor()));
            }
        }
    }
}

/// Show the settings window
fn show_settings_window(mtm: MainThreadMarker) {
    // Check if settings window is already open
    let existing = SETTINGS_WINDOW.load(Ordering::SeqCst);
    if !existing.is_null() {
        // Bring existing window to front
        unsafe {
            let window: &NSPanel = &*(existing as *const NSPanel);
            window.makeKeyAndOrderFront(None);
        }
        return;
    }

    // Disable the settings menu item while window is open
    let menu_item_ptr = SETTINGS_MENU_ITEM.load(Ordering::SeqCst);
    if !menu_item_ptr.is_null() {
        unsafe {
            let menu_item: &NSMenuItem = &*(menu_item_ptr as *const NSMenuItem);
            menu_item.setEnabled(false);
        }
    }

    // Window dimensions
    let window_width: CGFloat = 400.0;
    let window_height: CGFloat = 340.0;

    // Calculate center position on screen
    let screen_frame = NSScreen::mainScreen(mtm)
        .map(|s| s.frame())
        .unwrap_or(CGRect {
            origin: CGPoint { x: 0.0, y: 0.0 },
            size: CGSize {
                width: 1920.0,
                height: 1080.0,
            },
        });

    let window_x = (screen_frame.size.width - window_width) / 2.0;
    let window_y = (screen_frame.size.height - window_height) / 2.0;

    let window_frame = CGRect {
        origin: CGPoint {
            x: window_x,
            y: window_y,
        },
        size: CGSize {
            width: window_width,
            height: window_height,
        },
    };

    // Create the settings panel (utility window style)
    let panel = {
        let panel = NSPanel::alloc(mtm);
        NSPanel::initWithContentRect_styleMask_backing_defer(
            panel,
            window_frame,
            NSWindowStyleMask::Titled | NSWindowStyleMask::Closable,
            NSBackingStoreType::Buffered,
            false,
        )
    };

    panel.setTitle(ns_string!("Cat Shield Settings"));
    panel.setLevel(3); // NSFloatingWindowLevel
    unsafe {
        panel.setReleasedWhenClosed(false);
    }

    // Create and set window delegate to handle close button (X) cleanup
    let delegate = unsafe {
        let delegate_ptr = SETTINGS_WINDOW_DELEGATE.load(Ordering::SeqCst);
        if delegate_ptr.is_null() {
            let new_delegate = SettingsWindowDelegate::new(mtm);
            SETTINGS_WINDOW_DELEGATE.store(
                Retained::as_ptr(&new_delegate) as *mut c_void,
                Ordering::SeqCst,
            );
            std::mem::forget(new_delegate);
            &*(SETTINGS_WINDOW_DELEGATE.load(Ordering::SeqCst) as *const SettingsWindowDelegate)
        } else {
            &*(delegate_ptr as *const SettingsWindowDelegate)
        }
    };
    panel.setDelegate(Some(ProtocolObject::from_ref(delegate)));

    // Store window reference
    SETTINGS_WINDOW.store(Retained::as_ptr(&panel) as *mut c_void, Ordering::SeqCst);

    // Get or create settings action handler
    let handler = unsafe {
        let handler_ptr = SETTINGS_ACTION_HANDLER.load(Ordering::SeqCst);
        if handler_ptr.is_null() {
            let new_handler = SettingsActionHandler::new(mtm);
            SETTINGS_ACTION_HANDLER.store(
                Retained::as_ptr(&new_handler) as *mut c_void,
                Ordering::SeqCst,
            );
            std::mem::forget(new_handler);
            &*(SETTINGS_ACTION_HANDLER.load(Ordering::SeqCst) as *const SettingsActionHandler)
        } else {
            &*(handler_ptr as *const SettingsActionHandler)
        }
    };

    // Load current config values
    let config = get_current_config();

    // Create content view with controls
    if let Some(content_view) = panel.contentView() {
        let margin: CGFloat = 20.0;
        let label_height: CGFloat = 20.0;
        let field_height: CGFloat = 24.0;
        let row_spacing: CGFloat = 8.0;
        let section_spacing: CGFloat = 20.0;
        let field_width = window_width - (margin * 2.0);

        let mut y_offset = window_height - margin - 10.0;

        // ========================================
        // Exit Key Section
        // ========================================
        y_offset -= label_height;
        let exit_key_label = create_label(
            mtm,
            "Exit Key Shortcut:",
            CGRect {
                origin: CGPoint {
                    x: margin,
                    y: y_offset,
                },
                size: CGSize {
                    width: field_width,
                    height: label_height,
                },
            },
            13.0,
            &NSColor::labelColor(),
            true,
        );
        content_view.addSubview(&exit_key_label);

        y_offset -= field_height + row_spacing;
        let exit_key_field = NSTextField::new(mtm);
        exit_key_field.setFrame(CGRect {
            origin: CGPoint {
                x: margin,
                y: y_offset,
            },
            size: CGSize {
                width: field_width,
                height: field_height,
            },
        });
        exit_key_field.setStringValue(&NSString::from_str(
            config.exit_key.as_deref().unwrap_or(DEFAULT_EXIT_KEY),
        ));
        exit_key_field.setPlaceholderString(Some(ns_string!("e.g., Cmd+Option+U")));
        content_view.addSubview(&exit_key_field);
        SETTINGS_EXIT_KEY_FIELD.store(
            Retained::as_ptr(&exit_key_field) as *mut c_void,
            Ordering::SeqCst,
        );
        std::mem::forget(exit_key_field);

        y_offset -= label_height + 2.0;
        let exit_key_validation = NSTextField::new(mtm);
        exit_key_validation.setFrame(CGRect {
            origin: CGPoint {
                x: margin,
                y: y_offset,
            },
            size: CGSize {
                width: field_width,
                height: label_height,
            },
        });
        exit_key_validation.setEditable(false);
        exit_key_validation.setSelectable(false);
        exit_key_validation.setBordered(false);
        exit_key_validation.setDrawsBackground(false);
        exit_key_validation.setStringValue(ns_string!(""));
        exit_key_validation.setFont(Some(&NSFont::systemFontOfSize(11.0)));
        content_view.addSubview(&exit_key_validation);
        SETTINGS_EXIT_KEY_VALIDATION_LABEL.store(
            Retained::as_ptr(&exit_key_validation) as *mut c_void,
            Ordering::SeqCst,
        );
        std::mem::forget(exit_key_validation);

        // ========================================
        // Default Timer Section
        // ========================================
        y_offset -= section_spacing;
        y_offset -= label_height;
        let timer_label = create_label(
            mtm,
            "Default Timer:",
            CGRect {
                origin: CGPoint {
                    x: margin,
                    y: y_offset,
                },
                size: CGSize {
                    width: field_width,
                    height: label_height,
                },
            },
            13.0,
            &NSColor::labelColor(),
            true,
        );
        content_view.addSubview(&timer_label);

        y_offset -= field_height + row_spacing;

        // Checkbox for enabling default timer
        let timer_checkbox = unsafe {
            let checkbox = NSButton::checkboxWithTitle_target_action(
                ns_string!("Enable auto-exit timer"),
                Some(handler),
                Some(objc2::sel!(timerCheckboxChanged:)),
                mtm,
            );
            checkbox.setFrame(CGRect {
                origin: CGPoint {
                    x: margin,
                    y: y_offset,
                },
                size: CGSize {
                    width: 200.0,
                    height: field_height,
                },
            });
            checkbox.setControlSize(NSControlSize::Regular);
            if config.default_timer.is_some() {
                checkbox.setState(NSControlStateValueOn);
            } else {
                checkbox.setState(NSControlStateValueOff);
            }
            checkbox
        };
        content_view.addSubview(&timer_checkbox);
        SETTINGS_TIMER_CHECKBOX.store(
            Retained::as_ptr(&timer_checkbox) as *mut c_void,
            Ordering::SeqCst,
        );
        std::mem::forget(timer_checkbox);

        y_offset -= field_height + row_spacing;

        // Parse existing timer value to extract number and unit
        let (timer_value, timer_unit_index) = if let Some(ref timer_str) = config.default_timer {
            parse_timer_value_and_unit(timer_str)
        } else {
            ("".to_string(), 0) // Default to minutes
        };

        // Number input field (narrower, on the left)
        let number_field_width: CGFloat = 80.0;
        let dropdown_width: CGFloat = 100.0;
        let spacing: CGFloat = 10.0;

        let timer_value_field = NSTextField::new(mtm);
        timer_value_field.setFrame(CGRect {
            origin: CGPoint {
                x: margin,
                y: y_offset,
            },
            size: CGSize {
                width: number_field_width,
                height: field_height,
            },
        });
        timer_value_field.setStringValue(&NSString::from_str(&timer_value));
        timer_value_field.setPlaceholderString(Some(ns_string!("30")));
        timer_value_field.setEnabled(config.default_timer.is_some());
        if config.default_timer.is_none() {
            timer_value_field.setTextColor(Some(&NSColor::disabledControlTextColor()));
        }
        content_view.addSubview(&timer_value_field);
        SETTINGS_TIMER_VALUE_FIELD.store(
            Retained::as_ptr(&timer_value_field) as *mut c_void,
            Ordering::SeqCst,
        );
        std::mem::forget(timer_value_field);

        // Unit dropdown (to the right of the number field)
        let timer_unit_dropdown = NSPopUpButton::new(mtm);
        timer_unit_dropdown.setFrame(CGRect {
            origin: CGPoint {
                x: margin + number_field_width + spacing,
                y: y_offset,
            },
            size: CGSize {
                width: dropdown_width,
                height: field_height,
            },
        });
        timer_unit_dropdown.addItemWithTitle(ns_string!("Minutes"));
        timer_unit_dropdown.addItemWithTitle(ns_string!("Hours"));
        timer_unit_dropdown.addItemWithTitle(ns_string!("Seconds"));
        timer_unit_dropdown.selectItemAtIndex(timer_unit_index);
        timer_unit_dropdown.setEnabled(config.default_timer.is_some());
        content_view.addSubview(&timer_unit_dropdown);
        SETTINGS_TIMER_UNIT_DROPDOWN.store(
            Retained::as_ptr(&timer_unit_dropdown) as *mut c_void,
            Ordering::SeqCst,
        );
        std::mem::forget(timer_unit_dropdown);

        y_offset -= label_height + 2.0;
        let timer_validation = NSTextField::new(mtm);
        timer_validation.setFrame(CGRect {
            origin: CGPoint {
                x: margin,
                y: y_offset,
            },
            size: CGSize {
                width: field_width,
                height: label_height,
            },
        });
        timer_validation.setEditable(false);
        timer_validation.setSelectable(false);
        timer_validation.setBordered(false);
        timer_validation.setDrawsBackground(false);
        timer_validation.setStringValue(ns_string!(""));
        timer_validation.setFont(Some(&NSFont::systemFontOfSize(11.0)));
        content_view.addSubview(&timer_validation);
        SETTINGS_TIMER_VALIDATION_LABEL.store(
            Retained::as_ptr(&timer_validation) as *mut c_void,
            Ordering::SeqCst,
        );
        std::mem::forget(timer_validation);

        // ========================================
        // Overlay Opacity Section
        // ========================================
        y_offset -= section_spacing;
        y_offset -= label_height;
        let opacity_label = create_label(
            mtm,
            "Overlay Opacity:",
            CGRect {
                origin: CGPoint {
                    x: margin,
                    y: y_offset,
                },
                size: CGSize {
                    width: 120.0,
                    height: label_height,
                },
            },
            13.0,
            &NSColor::labelColor(),
            true,
        );
        content_view.addSubview(&opacity_label);

        // Current percentage label (right side)
        let percentage_label = NSTextField::new(mtm);
        percentage_label.setFrame(CGRect {
            origin: CGPoint {
                x: window_width - margin - 50.0,
                y: y_offset,
            },
            size: CGSize {
                width: 50.0,
                height: label_height,
            },
        });
        percentage_label.setEditable(false);
        percentage_label.setSelectable(false);
        percentage_label.setBordered(false);
        percentage_label.setDrawsBackground(false);
        percentage_label.setAlignment(NSTextAlignment::Right);
        let current_opacity = config.opacity();
        percentage_label.setStringValue(&NSString::from_str(&format!(
            "{}%",
            (current_opacity * 100.0) as i32
        )));
        percentage_label.setFont(Some(&NSFont::boldSystemFontOfSize(13.0)));
        content_view.addSubview(&percentage_label);
        SETTINGS_OPACITY_LABEL.store(
            Retained::as_ptr(&percentage_label) as *mut c_void,
            Ordering::SeqCst,
        );
        std::mem::forget(percentage_label);

        y_offset -= field_height + row_spacing;

        // Slider with min/max labels
        let slider_margin = 35.0;
        let slider_width = field_width - (slider_margin * 2.0);

        // Min label (20%)
        let min_label = create_label(
            mtm,
            "20%",
            CGRect {
                origin: CGPoint {
                    x: margin,
                    y: y_offset,
                },
                size: CGSize {
                    width: 30.0,
                    height: field_height,
                },
            },
            11.0,
            &NSColor::secondaryLabelColor(),
            false,
        );
        content_view.addSubview(&min_label);

        // Max label (80%)
        let max_label = create_label(
            mtm,
            "80%",
            CGRect {
                origin: CGPoint {
                    x: window_width - margin - 30.0,
                    y: y_offset,
                },
                size: CGSize {
                    width: 30.0,
                    height: field_height,
                },
            },
            11.0,
            &NSColor::secondaryLabelColor(),
            false,
        );
        max_label.setAlignment(NSTextAlignment::Right);
        content_view.addSubview(&max_label);

        // Opacity slider
        let opacity_slider = {
            let slider = NSSlider::new(mtm);
            slider.setFrame(CGRect {
                origin: CGPoint {
                    x: margin + slider_margin,
                    y: y_offset,
                },
                size: CGSize {
                    width: slider_width,
                    height: field_height,
                },
            });
            slider.setMinValue(MIN_OVERLAY_OPACITY);
            slider.setMaxValue(MAX_OVERLAY_OPACITY);
            slider.setDoubleValue(current_opacity);
            unsafe {
                slider.setTarget(Some(handler));
                slider.setAction(Some(objc2::sel!(opacityChanged:)));
            }
            slider
        };
        content_view.addSubview(&opacity_slider);
        SETTINGS_OPACITY_SLIDER.store(
            Retained::as_ptr(&opacity_slider) as *mut c_void,
            Ordering::SeqCst,
        );
        std::mem::forget(opacity_slider);

        // ========================================
        // Buttons Section
        // ========================================
        let button_height: CGFloat = 28.0;
        let button_width: CGFloat = 80.0;
        let button_spacing: CGFloat = 12.0;
        let button_y: CGFloat = margin;

        // Cancel button (left)
        let cancel_button = unsafe {
            let button = NSButton::buttonWithTitle_target_action(
                ns_string!("Cancel"),
                Some(handler),
                Some(objc2::sel!(cancelSettings:)),
                mtm,
            );
            button.setFrame(CGRect {
                origin: CGPoint {
                    x: window_width - margin - button_width - button_spacing - button_width,
                    y: button_y,
                },
                size: CGSize {
                    width: button_width,
                    height: button_height,
                },
            });
            button.setButtonType(NSButtonType::MomentaryPushIn);
            button.setKeyEquivalent(ns_string!("\u{1b}")); // Escape key
            button
        };
        content_view.addSubview(&cancel_button);
        std::mem::forget(cancel_button);

        // Save button (right)
        let save_button = unsafe {
            let button = NSButton::buttonWithTitle_target_action(
                ns_string!("Save"),
                Some(handler),
                Some(objc2::sel!(saveSettings:)),
                mtm,
            );
            button.setFrame(CGRect {
                origin: CGPoint {
                    x: window_width - margin - button_width,
                    y: button_y,
                },
                size: CGSize {
                    width: button_width,
                    height: button_height,
                },
            });
            button.setButtonType(NSButtonType::MomentaryPushIn);
            button.setKeyEquivalent(ns_string!("\r")); // Return key
            button
        };
        content_view.addSubview(&save_button);
        std::mem::forget(save_button);
    }

    // Activate the application so the window can receive focus
    // This is needed because the app runs in accessory mode (no dock icon)
    let app = NSApplication::sharedApplication(mtm);
    #[allow(deprecated)]
    app.activateIgnoringOtherApps(true);

    // Show the window
    panel.makeKeyAndOrderFront(None);

    // Keep panel alive
    std::mem::forget(panel);

    println!("  Settings window opened");
}

/// Draw the close button with progress indicator
fn draw_close_button(view: &NSView) {
    let bounds = view.bounds();
    let center_x = bounds.size.width / 2.0;
    let center_y = bounds.size.height / 2.0;
    let radius = (bounds.size.width.min(bounds.size.height) / 2.0) - 2.0;

    // Calculate progress (0.0 to 1.0)
    let progress = MOUSE_DOWN_TIME.with(|time| {
        if let Some(start) = time.get() {
            calculate_hold_progress(start.elapsed().as_secs_f64(), HOLD_DURATION_SECS)
        } else {
            0.0
        }
    });

    let is_inside = IS_MOUSE_INSIDE.with(|inside| inside.get());

    // Background circle - bright red for visibility
    let bg_color = if is_inside && progress > 0.0 {
        NSColor::colorWithRed_green_blue_alpha(0.9, 0.2, 0.2, 1.0) // Bright red when pressed
    } else {
        NSColor::colorWithRed_green_blue_alpha(0.8, 0.1, 0.1, 0.95) // Dark red normally
    };

    bg_color.set();

    let bg_path = NSBezierPath::bezierPathWithOvalInRect(CGRect {
        origin: CGPoint {
            x: center_x - radius,
            y: center_y - radius,
        },
        size: CGSize {
            width: radius * 2.0,
            height: radius * 2.0,
        },
    });
    bg_path.fill();

    // White border for extra visibility
    let border_color = NSColor::colorWithRed_green_blue_alpha(1.0, 1.0, 1.0, 0.9);
    border_color.set();
    let border_path = NSBezierPath::bezierPathWithOvalInRect(CGRect {
        origin: CGPoint {
            x: center_x - radius,
            y: center_y - radius,
        },
        size: CGSize {
            width: radius * 2.0,
            height: radius * 2.0,
        },
    });
    border_path.setLineWidth(3.0);
    border_path.stroke();

    // Progress arc (if holding) - bright green
    if progress > 0.0 && is_inside {
        let progress_color = NSColor::colorWithRed_green_blue_alpha(0.2, 1.0, 0.2, 1.0);
        progress_color.set();

        // Draw arc from top, going clockwise
        let start_angle = 90.0; // Top of circle
        let end_angle = 90.0 - (progress * 360.0);

        let arc_path = NSBezierPath::bezierPath();
        arc_path.setLineWidth(6.0); // Thicker progress ring

        arc_path.appendBezierPathWithArcWithCenter_radius_startAngle_endAngle_clockwise(
            CGPoint {
                x: center_x,
                y: center_y,
            },
            radius - 5.0,
            start_angle,
            end_angle,
            true, // clockwise
        );
        arc_path.stroke();
    }

    // Draw X - always white and bold
    let x_color = NSColor::colorWithRed_green_blue_alpha(1.0, 1.0, 1.0, 1.0);
    x_color.set();

    let x_size = radius * 0.4;
    let x_path = NSBezierPath::bezierPath();
    x_path.setLineWidth(5.0); // Thicker X

    // First line of X (top-left to bottom-right)
    x_path.moveToPoint(CGPoint {
        x: center_x - x_size,
        y: center_y + x_size,
    });
    x_path.lineToPoint(CGPoint {
        x: center_x + x_size,
        y: center_y - x_size,
    });

    // Second line of X (top-right to bottom-left)
    x_path.moveToPoint(CGPoint {
        x: center_x + x_size,
        y: center_y + x_size,
    });
    x_path.lineToPoint(CGPoint {
        x: center_x - x_size,
        y: center_y - x_size,
    });

    x_path.stroke();
}

/// Creates an IOKit assertion to prevent the system from sleeping
fn prevent_sleep() -> Option<u32> {
    let assertion_type = CFString::from_static_str("PreventUserIdleDisplaySleep");
    let reason =
        CFString::from_static_str("Cat Shield is active - protecting your work from cats!");

    let mut assertion_id: u32 = 0;

    let result = unsafe {
        IOPMAssertionCreateWithName(
            CFRetained::as_ptr(&assertion_type).as_ptr() as *const c_void,
            K_IOPM_ASSERTION_LEVEL_ON,
            CFRetained::as_ptr(&reason).as_ptr() as *const c_void,
            &mut assertion_id,
        )
    };

    if result == 0 {
        println!("  ✓ Sleep prevention enabled");
        Some(assertion_id)
    } else {
        eprintln!("  ✗ Failed to create power assertion: {}", result);
        None
    }
}

/// Releases the sleep prevention assertion
fn allow_sleep(assertion_id: u32) {
    let result = unsafe { IOPMAssertionRelease(assertion_id) };
    if result == 0 {
        println!("  ✓ Sleep prevention disabled");
    }
}

/// Disable and clean up the event tap
fn disable_event_tap() {
    let tap_ptr = EVENT_TAP.swap(std::ptr::null_mut(), Ordering::SeqCst);
    let source_ptr = EVENT_TAP_RUN_LOOP_SOURCE.swap(std::ptr::null_mut(), Ordering::SeqCst);

    if !tap_ptr.is_null() {
        unsafe {
            // Disable the tap
            CGEventTapEnable(tap_ptr, false);

            // Remove the run loop source from the run loop before releasing
            if !source_ptr.is_null() {
                let current_run_loop = CFRunLoopGetCurrent();
                let run_loop_mode =
                    kCFRunLoopCommonModes.expect("kCFRunLoopCommonModes should exist");
                CFRunLoopRemoveSource(
                    current_run_loop,
                    source_ptr,
                    (run_loop_mode as *const CFString) as *const c_void,
                );
                // Release the run loop source
                CFRelease(source_ptr);
            }

            // Release the CFMachPort
            CFRelease(tap_ptr);
        }
        println!("  ✓ Input blocking disabled");
    }
}

/// Deactivate the shield and return to menu bar mode
///
/// This function:
/// - Closes the shield window
/// - Stops the animation timer
/// - Disables the event tap
/// - Releases the sleep assertion
/// - Re-enables the "Start Protection" menu item
/// - Resets shield state
fn deactivate_shield() {
    // Only deactivate if shield is active
    if !SHIELD_ACTIVE.swap(false, Ordering::SeqCst) {
        return;
    }

    println!();
    println!("  🛡️  Deactivating Cat Shield...");

    // Stop the animation timer first
    stop_close_button_timer();

    // Disable the event tap
    disable_event_tap();

    // Release sleep assertion if we have one
    if HAS_SLEEP_ASSERTION.swap(false, Ordering::SeqCst) {
        let assertion_id = SLEEP_ASSERTION_ID.load(Ordering::SeqCst) as u32;
        allow_sleep(assertion_id);
    }

    // Close the shield window and properly release it
    // We use Retained::from_raw to reclaim ownership from the raw pointer,
    // which ensures the NSWindow is properly released when dropped
    let window_ptr = SHIELD_WINDOW.swap(std::ptr::null_mut(), Ordering::SeqCst);
    if !window_ptr.is_null() {
        unsafe {
            // Reconstruct Retained to take ownership and properly release
            let window: Retained<NSWindow> =
                Retained::from_raw(window_ptr as *mut NSWindow).expect("SHIELD_WINDOW was valid");
            window.close();
            // window is dropped here, calling release() on the NSWindow
        }
        println!("  ✓ Shield window closed");
    }

    // Release the close button view properly
    // The window's content view also holds a reference, but we need to release our ownership
    let close_button_ptr = CLOSE_BUTTON_VIEW.swap(std::ptr::null_mut(), Ordering::SeqCst);
    if !close_button_ptr.is_null() {
        unsafe {
            // Reconstruct Retained to take ownership and properly release
            let _close_button: Retained<CloseButtonView> =
                Retained::from_raw(close_button_ptr as *mut CloseButtonView)
                    .expect("CLOSE_BUTTON_VIEW was valid");
            // Dropped here, calling release()
        }
    }

    // Clear close button label view reference
    let close_button_label_ptr =
        CLOSE_BUTTON_LABEL_VIEW.swap(std::ptr::null_mut(), Ordering::SeqCst);
    if !close_button_label_ptr.is_null() {
        unsafe {
            let _label: Retained<CloseButtonLabelView> =
                Retained::from_raw(close_button_label_ptr as *mut CloseButtonLabelView)
                    .expect("CLOSE_BUTTON_LABEL_VIEW was valid");
        }
    }

    // Clear timer display view reference (only set in immediate mode, but clear for safety)
    TIMER_DISPLAY_VIEW.store(std::ptr::null_mut(), Ordering::SeqCst);

    // Clear NSTextField label references
    TIMER_HEADER_LABEL.store(std::ptr::null_mut(), Ordering::SeqCst);
    TIMER_TIME_LABEL.store(std::ptr::null_mut(), Ordering::SeqCst);
    TIMER_WARNING_LABEL.store(std::ptr::null_mut(), Ordering::SeqCst);
    CLOSE_BUTTON_TEXT_LABEL.store(std::ptr::null_mut(), Ordering::SeqCst);

    // Reset auto-exit timer state
    AUTO_EXIT_ENABLED.store(false, Ordering::SeqCst);
    WARNING_SHOWN.store(false, Ordering::SeqCst);

    // Reset close button state
    MOUSE_DOWN_TIME.with(|time| time.set(None));
    IS_MOUSE_INSIDE.with(|inside| inside.set(false));

    // Re-enable the "Start Protection" menu item
    let menu_item_ptr = START_MENU_ITEM.load(Ordering::SeqCst);
    if !menu_item_ptr.is_null() {
        unsafe {
            let menu_item: &NSMenuItem = &*(menu_item_ptr as *const NSMenuItem);
            menu_item.setEnabled(true);
        }
    }

    println!();
    println!("  ✓ Cat Shield deactivated");
    println!("  Click the 🐱 icon to activate protection again.");
    println!();
}

/// Activate the shield protection
///
/// This function creates the fullscreen overlay window, sets up input blocking,
/// and prevents sleep. It's called either from the menu item action or from
/// the CLI immediate mode.
fn activate_shield(mtm: MainThreadMarker) {
    // Prevent double-activation
    if SHIELD_ACTIVE.swap(true, Ordering::SeqCst) {
        return;
    }

    // Disable the "Start Protection" menu item while shield is active
    let menu_item_ptr = START_MENU_ITEM.load(Ordering::SeqCst);
    if !menu_item_ptr.is_null() {
        unsafe {
            let menu_item: &NSMenuItem = &*(menu_item_ptr as *const NSMenuItem);
            menu_item.setEnabled(false);
        }
    }

    // Get the exit key configuration
    let exit_key = get_exit_key();

    // Check accessibility permissions FIRST, before any UI
    let mut has_accessibility = check_accessibility();

    if !has_accessibility {
        println!();
        println!("  🐱 CAT SHIELD 🛡️");
        println!("  ════════════════════════════════════════");
        println!();
        eprintln!("  ⚠️  ACCESSIBILITY PERMISSION REQUIRED");
        eprintln!();
        eprintln!("  To block keyboard/mouse input and use the exit");
        eprintln!(
            "  shortcut ({}), this app needs Accessibility permissions.",
            exit_key.display_name
        );
        eprintln!();

        // Try to prompt user with native dialog
        println!("  Requesting accessibility permissions...");
        has_accessibility = check_accessibility_with_prompt();

        if has_accessibility {
            println!("  ✓ Permissions granted!");
            println!();
        } else {
            eprintln!();
            eprintln!("  Opening System Settings → Accessibility...");

            if open_accessibility_settings() {
                eprintln!("  ✓ System Settings opened");
            }
            eprintln!();
            eprintln!("  Please add Cat Shield to the Accessibility list.");
            eprintln!("  Waiting for permissions...");
            eprintln!();

            // Poll for permissions every 1 second using CFRunLoopRunInMode
            const POLL_INTERVAL_SECS: f64 = 1.0;
            loop {
                unsafe {
                    let mode = kCFRunLoopDefaultMode.expect("kCFRunLoopDefaultMode should exist");
                    CFRunLoopRunInMode((mode as *const CFString).cast(), POLL_INTERVAL_SECS, false);
                }
                if check_accessibility() {
                    println!("  ✓ Permissions granted! Starting Cat Shield...");
                    println!();
                    break;
                }
            }
        }
    }

    println!();
    println!("  🐱 CAT SHIELD 🛡️");
    println!("  ════════════════════════════════════════");
    println!("  Protecting your work from curious cats!");
    println!();

    // Get the main screen dimensions
    let screen = NSScreen::mainScreen(mtm);
    let screen = match screen {
        Some(s) => s,
        None => {
            eprintln!("  ✗ Failed to get main screen");
            SHIELD_ACTIVE.store(false, Ordering::SeqCst);
            // Re-enable menu item
            if !menu_item_ptr.is_null() {
                unsafe {
                    let menu_item: &NSMenuItem = &*(menu_item_ptr as *const NSMenuItem);
                    menu_item.setEnabled(true);
                }
            }
            return;
        }
    };
    let screen_frame = screen.frame();

    // Create a fullscreen, borderless window
    let window = unsafe {
        let window = NSWindow::alloc(mtm);
        NSWindow::initWithContentRect_styleMask_backing_defer(
            window,
            screen_frame,
            NSWindowStyleMask::Borderless,
            NSBackingStoreType::Buffered,
            false,
        )
    };

    // Store window reference for cleanup
    SHIELD_WINDOW.store(Retained::as_ptr(&window) as *mut c_void, Ordering::SeqCst);

    // Configure window to be topmost
    window.setLevel(NS_SCREEN_SAVER_WINDOW_LEVEL);

    // Set window to appear on all spaces and stay visible
    window.setCollectionBehavior(
        NSWindowCollectionBehavior::CanJoinAllSpaces
            | NSWindowCollectionBehavior::Stationary
            | NSWindowCollectionBehavior::IgnoresCycle,
    );

    // Window must be non-opaque to allow transparent background, but keep alphaValue at 1.0
    // so that subviews (like label containers) can render fully opaque
    window.setOpaque(false);
    window.setAlphaValue(1.0);

    // Set a semi-transparent dark background color (the overlay effect)
    let bg_color = NSColor::colorWithRed_green_blue_alpha(0.1, 0.1, 0.15, 0.5);
    window.setBackgroundColor(Some(&bg_color));

    // Keep window visible
    window.setHidesOnDeactivate(false);

    // Accept mouse events (needed for blocking)
    window.setIgnoresMouseEvents(false);

    // Set title
    window.setTitle(ns_string!("Cat Shield"));

    // Required when creating NSWindow outside a window controller
    unsafe {
        window.setReleasedWhenClosed(false);
    }

    // Show the window
    window.makeKeyAndOrderFront(None);

    println!("  ✓ Overlay window active");

    // Create and add the close button in top-right corner
    let close_button_frame = CGRect {
        origin: CGPoint {
            x: screen_frame.size.width - CLOSE_BUTTON_SIZE - CLOSE_BUTTON_MARGIN,
            y: screen_frame.size.height - CLOSE_BUTTON_SIZE - CLOSE_BUTTON_MARGIN,
        },
        size: CGSize {
            width: CLOSE_BUTTON_SIZE,
            height: CLOSE_BUTTON_SIZE,
        },
    };

    let close_button = CloseButtonView::new(mtm, close_button_frame);

    // Store view reference for timer callback.
    CLOSE_BUTTON_VIEW.store(
        Retained::as_ptr(&close_button) as *mut c_void,
        Ordering::SeqCst,
    );

    // Create the close button label view (positioned below the button)
    let label_x = screen_frame.size.width
        - CLOSE_BUTTON_MARGIN
        - CLOSE_BUTTON_SIZE / 2.0
        - CLOSE_BUTTON_LABEL_WIDTH / 2.0;
    let label_y = screen_frame.size.height
        - CLOSE_BUTTON_SIZE
        - CLOSE_BUTTON_MARGIN
        - CLOSE_BUTTON_LABEL_HEIGHT
        - 5.0; // 5px gap between button and label

    let close_button_label_frame = CGRect {
        origin: CGPoint {
            x: label_x,
            y: label_y,
        },
        size: CGSize {
            width: CLOSE_BUTTON_LABEL_WIDTH,
            height: CLOSE_BUTTON_LABEL_HEIGHT,
        },
    };
    let close_button_label = CloseButtonLabelView::new(mtm, close_button_label_frame);

    // Store label view reference for timer callback updates
    CLOSE_BUTTON_LABEL_VIEW.store(
        Retained::as_ptr(&close_button_label) as *mut c_void,
        Ordering::SeqCst,
    );

    // Add close button and label to the window's content view
    if let Some(content_view) = window.contentView() {
        content_view.addSubview(&close_button);
        content_view.addSubview(&close_button_label);
    }

    // Start the animation timer
    start_close_button_timer();

    println!("  ✓ Close button active (hold 3s to exit)");
    println!("  ✓ Exit key: {}", exit_key.display_name);

    // Prevent sleep
    if let Some(assertion_id) = prevent_sleep() {
        SLEEP_ASSERTION_ID.store(assertion_id as u64, Ordering::SeqCst);
        HAS_SLEEP_ASSERTION.store(true, Ordering::SeqCst);
    }

    // Set up event tap - this is the core security feature
    // Without input blocking, the shield is just a visual overlay
    if !setup_event_tap() {
        eprintln!("  ✗ Failed to create event tap");
        eprintln!();
        eprintln!("  ════════════════════════════════════════");
        eprintln!("  ⚠️  SHIELD ACTIVATION FAILED");
        eprintln!("  ════════════════════════════════════════");
        eprintln!();
        eprintln!("  Input blocking could not be enabled.");
        eprintln!("  The shield cannot protect without this feature.");
        eprintln!();
        eprintln!("  Please check:");
        eprintln!("  - Accessibility permissions are granted");
        eprintln!("  - No other apps are blocking event taps");
        eprintln!();
        // Deactivate and return to menu bar mode
        deactivate_shield();
        return;
    }
    println!("  ✓ Input blocking active");

    println!();
    println!("  ═══════════════════════════════════════");
    println!("  🛡️  CAT SHIELD IS NOW ACTIVE!");
    println!("  ═══════════════════════════════════════");
    println!();
    println!("  Exit: Hold X button (top-right) for 3 seconds");
    println!("        Or press {}", exit_key.display_name);
    println!();

    // Keep the window and views retained so they don't get deallocated
    std::mem::forget(window);
    std::mem::forget(close_button);
    std::mem::forget(close_button_label);
}

/// Callback for the CGEventTap - intercepts and blocks events
unsafe extern "C-unwind" fn event_tap_callback(
    _proxy: CGEventTapProxy,
    event_type: CGEventType,
    event: NonNull<CGEvent>,
    _user_info: *mut c_void,
) -> *mut CGEvent {
    // Handle tap disabled event (system can disable taps if they're too slow)
    if event_type == CGEventType::TapDisabledByTimeout
        || event_type == CGEventType::TapDisabledByUserInput
    {
        eprintln!("  ⚠️  Event tap was disabled, re-enabling...");
        // Re-enable the tap using the stored pointer
        let tap = EVENT_TAP.load(Ordering::SeqCst);
        if !tap.is_null() {
            CGEventTapEnable(tap, true);
        }
        return event.as_ptr();
    }

    // Check for configured exit key combination
    if event_type == CGEventType::KeyDown {
        let cg_event = event.as_ref();

        let flags = CGEvent::flags(Some(cg_event));
        let keycode =
            CGEvent::integer_value_field(Some(cg_event), CGEventField::KeyboardEventKeycode);

        // Check if the key combination matches the configured exit key
        if check_exit_key(keycode, flags) {
            println!("\n  🔓 Exit key combination detected!");

            // In menu bar mode, deactivate shield and return to menu bar
            // In immediate mode, terminate the app
            if MENU_BAR_MODE.load(Ordering::SeqCst) {
                deactivate_shield();
            } else if let Some(mtm) = MainThreadMarker::new() {
                let app = NSApplication::sharedApplication(mtm);
                app.terminate(None);
            }

            // Let this event through
            return event.as_ptr();
        }
    }

    // Block keyboard events by returning NULL
    // Mouse events are allowed through so our close button can work
    // (our topmost window captures all mouse events anyway)
    if event_type == CGEventType::KeyDown
        || event_type == CGEventType::KeyUp
        || event_type == CGEventType::FlagsChanged
    {
        // Return NULL to block the event
        return std::ptr::null_mut();
    }

    event.as_ptr()
}

/// Check if we have accessibility permissions
fn check_accessibility() -> bool {
    unsafe { AXIsProcessTrusted() }
}

/// Check accessibility permissions and prompt user with native dialog if not granted
fn check_accessibility_with_prompt() -> bool {
    unsafe {
        let keys = [kAXTrustedCheckOptionPrompt];
        let values = [kCFBooleanTrue];

        let dict = CFDictionaryCreate(
            std::ptr::null(),
            keys.as_ptr(),
            values.as_ptr(),
            1,
            std::ptr::null(),
            std::ptr::null(),
        );

        let result = AXIsProcessTrustedWithOptions(dict);

        if !dict.is_null() {
            CFRelease(dict);
        }

        result
    }
}

/// Open System Settings to the Accessibility privacy pane
fn open_accessibility_settings() -> bool {
    let url_string =
        ns_string!("x-apple.systempreferences:com.apple.preference.security?Privacy_Accessibility");

    if let Some(url) = NSURL::URLWithString(url_string) {
        let workspace = NSWorkspace::sharedWorkspace();
        return workspace.openURL(&url);
    }
    false
}

/// Create and enable the event tap
fn setup_event_tap() -> bool {
    // Define event mask for keyboard events only
    // Mouse events are NOT blocked - our topmost fullscreen window captures them,
    // and we need mouse events to reach our close button
    let event_mask: CGEventMask = (1u64 << CGEventType::KeyDown.0)
        | (1u64 << CGEventType::KeyUp.0)
        | (1u64 << CGEventType::FlagsChanged.0);

    unsafe {
        // Create the event tap using CGEvent::tap_create
        let tap_opt = CGEvent::tap_create(
            CGEventTapLocation::HIDEventTap, // Intercept at the HID level (earliest)
            CGEventTapPlacement::HeadInsertEventTap,
            CGEventTapOptions::Default, // Active tap that can modify/block events
            event_mask,
            Some(event_tap_callback),
            std::ptr::null_mut(),
        );

        let tap: CFRetained<CFMachPort> = match tap_opt {
            Some(t) => t,
            None => return false,
        };

        // Get raw pointer for storing and run loop source creation
        let tap_ptr = CFRetained::as_ptr(&tap).as_ptr() as *mut c_void;

        // Store the tap pointer globally so we can re-enable it from the callback
        EVENT_TAP.store(tap_ptr, Ordering::SeqCst);

        // Create a run loop source and add it to the current run loop
        let run_loop_source = CFMachPortCreateRunLoopSource(std::ptr::null(), tap_ptr, 0);

        if run_loop_source.is_null() {
            EVENT_TAP.store(std::ptr::null_mut(), Ordering::SeqCst);
            return false;
        }

        // Store run loop source for cleanup in disable_event_tap()
        EVENT_TAP_RUN_LOOP_SOURCE.store(run_loop_source, Ordering::SeqCst);

        // Add to run loop
        let current_run_loop = CFRunLoopGetCurrent();
        let run_loop_mode = kCFRunLoopCommonModes.expect("kCFRunLoopCommonModes should exist");
        CFRunLoopAddSource(
            current_run_loop,
            run_loop_source,
            (run_loop_mode as *const CFString) as *const c_void,
        );

        // Enable the tap
        CGEventTapEnable(tap_ptr, true);

        // Transfer ownership of CFMachPort to raw pointer stored in EVENT_TAP.
        // We call std::mem::forget to prevent CFRetained from releasing it here;
        // instead, we'll release it manually via CFRelease in disable_event_tap().
        std::mem::forget(tap);

        true
    }
}

/// Set up the menu bar status item with cat emoji icon
///
/// Creates an NSStatusItem in the system menu bar with:
/// - Cat emoji (🐱) as the icon
/// - "Cat Shield" tooltip on hover
/// - Comprehensive dropdown menu with all application features
///
/// Menu Structure:
/// - Header: "🐱 Cat Shield" (branding)
/// - Protection: Start/Stop Protection (for Issue #17)
/// - Configuration: Settings (for Issue #16)
/// - Information: About and Help (About for Issue #19)
/// - Exit: Quit with Cmd+Q
///
/// Returns the Retained<NSStatusItem> which must be kept alive for the duration
/// of the app to prevent the status item from being deallocated.
fn setup_menu_bar(mtm: MainThreadMarker) -> Retained<NSStatusItem> {
    // Get the system status bar
    let status_bar = NSStatusBar::systemStatusBar();

    // Create a status item with variable length (adjusts to content)
    // NSVariableStatusItemLength = -1.0
    let status_item = status_bar.statusItemWithLength(-1.0);

    // Configure the button (the clickable part of the status item)
    if let Some(button) = status_item.button(mtm) {
        // Set the cat emoji as the title
        button.setTitle(ns_string!("🐱"));

        // Set tooltip for accessibility
        button.setToolTip(Some(ns_string!(
            "Cat Shield - Protect your work from curious cats"
        )));
    }

    // Create the main dropdown menu
    let menu = NSMenu::new(mtm);

    // ============================================
    // HEADER SECTION
    // ============================================

    // Add "Cat Shield" title (disabled, just for branding)
    let title_item = NSMenuItem::new(mtm);
    title_item.setTitle(ns_string!("🐱 Cat Shield"));
    title_item.setEnabled(false);
    menu.addItem(&title_item);

    menu.addItem(&NSMenuItem::separatorItem(mtm));

    // ============================================
    // PROTECTION SECTION
    // ============================================

    // Create menu action handler and wire it to the Start Protection item
    let action_handler = MenuActionHandler::new(mtm);

    // Store the handler globally to keep it alive
    MENU_ACTION_HANDLER.store(
        Retained::as_ptr(&action_handler) as *mut c_void,
        Ordering::SeqCst,
    );

    // Add "Start Protection" item - activates the shield overlay on-demand
    let start_item = NSMenuItem::new(mtm);
    start_item.setTitle(ns_string!("Start Protection"));
    start_item.setToolTip(Some(ns_string!("Activate cat shield overlay")));

    // Set the target and action for the menu item
    unsafe {
        start_item.setTarget(Some(&action_handler));
        start_item.setAction(Some(objc2::sel!(startProtection:)));
    }

    // Store the start menu item reference for enabling/disabling
    START_MENU_ITEM.store(
        Retained::as_ptr(&start_item) as *mut c_void,
        Ordering::SeqCst,
    );

    menu.addItem(&start_item);

    // Keep handler alive
    std::mem::forget(action_handler);

    // Add "Stop Protection" item - deactivates the shield overlay when active
    // Initially hidden, will be shown when protection is active
    let stop_item = NSMenuItem::new(mtm);
    stop_item.setTitle(ns_string!("Stop Protection"));
    stop_item.setToolTip(Some(ns_string!("Deactivate cat shield overlay")));
    stop_item.setEnabled(false); // Will be enabled when shield is active
    stop_item.setHidden(true); // Hidden until protection is active
    menu.addItem(&stop_item);

    menu.addItem(&NSMenuItem::separatorItem(mtm));

    // ============================================
    // CONFIGURATION SECTION
    // ============================================

    // Add "Settings..." item - opens settings window for configuring preferences
    let settings_item = NSMenuItem::new(mtm);
    settings_item.setTitle(ns_string!("Settings..."));
    settings_item.setToolTip(Some(ns_string!(
        "Configure exit key, timer, and overlay opacity"
    )));
    settings_item.setKeyEquivalent(ns_string!(",")); // Standard Cmd+, for settings

    // Create settings action handler and wire it to the Settings item
    let settings_handler = SettingsActionHandler::new(mtm);

    // Set the target and action for the settings menu item
    unsafe {
        settings_item.setTarget(Some(&settings_handler));
        settings_item.setAction(Some(objc2::sel!(showSettings:)));
    }

    // Store the settings menu item reference for enabling/disabling
    SETTINGS_MENU_ITEM.store(
        Retained::as_ptr(&settings_item) as *mut c_void,
        Ordering::SeqCst,
    );

    // Store handler globally to keep it alive
    SETTINGS_ACTION_HANDLER.store(
        Retained::as_ptr(&settings_handler) as *mut c_void,
        Ordering::SeqCst,
    );

    menu.addItem(&settings_item);

    // Keep handler alive
    std::mem::forget(settings_handler);

    menu.addItem(&NSMenuItem::separatorItem(mtm));

    // ============================================
    // INFORMATION SECTION
    // ============================================

    // Add "About Cat Shield" item (will be functional in Issue #19)
    // Shows version, credits, and app information
    let about_item = NSMenuItem::new(mtm);
    about_item.setTitle(ns_string!("About Cat Shield"));
    about_item.setToolTip(Some(ns_string!(
        "About this application (Available in Issue #19)"
    )));
    about_item.setEnabled(false); // Disabled until Issue #19 implements about panel
    menu.addItem(&about_item);

    // Add "Help" submenu
    // Contains links to documentation, GitHub, and support resources
    let help_item = NSMenuItem::new(mtm);
    help_item.setTitle(ns_string!("Help"));

    // Create Help submenu
    let help_submenu = NSMenu::new(mtm);

    // Help -> View Documentation
    let docs_item = NSMenuItem::new(mtm);
    docs_item.setTitle(ns_string!("View Documentation"));
    docs_item.setToolTip(Some(ns_string!("Open README on GitHub")));
    docs_item.setEnabled(false); // Will need custom action handler to open URL
    help_submenu.addItem(&docs_item);

    // Help -> Report Issue
    let issue_item = NSMenuItem::new(mtm);
    issue_item.setTitle(ns_string!("Report Issue"));
    issue_item.setToolTip(Some(ns_string!("Report a bug on GitHub")));
    issue_item.setEnabled(false); // Will need custom action handler to open URL
    help_submenu.addItem(&issue_item);

    // Help -> Release Notes
    let release_item = NSMenuItem::new(mtm);
    release_item.setTitle(ns_string!("Release Notes"));
    release_item.setToolTip(Some(ns_string!("View ROADMAP and release notes")));
    release_item.setEnabled(false); // Will need custom action handler to open URL
    help_submenu.addItem(&release_item);

    help_item.setSubmenu(Some(&help_submenu));
    menu.addItem(&help_item);

    menu.addItem(&NSMenuItem::separatorItem(mtm));

    // ============================================
    // EXIT SECTION
    // ============================================

    // Add "Quit Cat Shield" item
    // Note: This uses the standard terminate: action which NSApplication handles
    let quit_item = NSMenuItem::new(mtm);
    quit_item.setTitle(ns_string!("Quit Cat Shield"));
    quit_item.setToolTip(Some(ns_string!("Quit the application")));
    unsafe {
        quit_item.setAction(Some(objc2::sel!(terminate:)));
    }
    // Set keyboard shortcut Cmd+Q
    quit_item.setKeyEquivalent(ns_string!("q"));
    menu.addItem(&quit_item);

    // Attach menu to status item
    status_item.setMenu(Some(&menu));

    println!("  ✓ Menu bar icon active (🐱) with comprehensive dropdown menu");

    status_item
}

/// Check if the app was launched with arguments that should trigger immediate shield activation
fn has_immediate_start_args(args: &Args) -> bool {
    // If timer or exit-key CLI args are provided, start shield immediately
    args.timer.is_some() || args.exit_key.is_some()
}

fn main() {
    // Parse command line arguments
    let args = Args::parse();

    // Load config file
    let config = Config::load();

    // Determine exit key: CLI arg > config file > default
    let exit_key = if let Some(ref key) = args.exit_key {
        key.clone()
    } else if let Some(ref key_str) = config.exit_key {
        match ExitKey::parse(key_str) {
            Ok(key) => key,
            Err(e) => {
                eprintln!("  ⚠️  Invalid exit_key in config file: {}", e);
                eprintln!("      Using default: {}", DEFAULT_EXIT_KEY);
                ExitKey::default()
            }
        }
    } else {
        ExitKey::default()
    };

    // Set the global exit key configuration
    set_exit_key(&exit_key);

    // Get main thread marker - required for AppKit operations
    let mtm = MainThreadMarker::new().expect("Must run on main thread");

    // Initialize the application
    let app = NSApplication::sharedApplication(mtm);
    app.setActivationPolicy(NSApplicationActivationPolicy::Accessory);

    // Check if we should enter menu bar mode (no CLI args that trigger immediate start)
    if !has_immediate_start_args(&args) {
        // Menu bar mode: show icon in menu bar and wait for user interaction
        MENU_BAR_MODE.store(true, Ordering::SeqCst);

        println!();
        println!("  🐱 CAT SHIELD 🛡️");
        println!("  ════════════════════════════════════════");
        println!("  Menu bar mode active");
        println!();

        // Set up menu bar icon
        let _status_item = setup_menu_bar(mtm);

        println!();
        println!("  Click the 🐱 icon in your menu bar to access Cat Shield.");
        println!("  Use 'Start Protection' to activate the shield.");
        println!("  Or run with --timer or --exit-key to start immediately.");
        println!();

        // Finish launching the application (required for menu bar apps)
        app.finishLaunching();

        // Run the NSApplication event loop
        // The status item keeps the app alive in the menu bar
        app.run();

        println!();
        println!("  👋 Cat Shield closed. Goodbye!");
        println!();
        return;
    }

    // Immediate shield mode: CLI args provided, start protection now
    // Check accessibility permissions FIRST, before any UI
    let mut has_accessibility = check_accessibility();

    if !has_accessibility {
        println!();
        println!("  🐱 CAT SHIELD 🛡️");
        println!("  ════════════════════════════════════════");
        println!();
        eprintln!("  ⚠️  ACCESSIBILITY PERMISSION REQUIRED");
        eprintln!();
        eprintln!("  To block keyboard/mouse input and use the exit");
        eprintln!(
            "  shortcut ({}), this app needs Accessibility permissions.",
            exit_key.display_name
        );
        eprintln!();

        // Try to prompt user with native dialog
        println!("  Requesting accessibility permissions...");
        has_accessibility = check_accessibility_with_prompt();

        if has_accessibility {
            println!("  ✓ Permissions granted!");
            println!();
        } else {
            eprintln!();
            eprintln!("  Opening System Settings → Accessibility...");

            if open_accessibility_settings() {
                eprintln!("  ✓ System Settings opened");
            }
            eprintln!();
            eprintln!("  Please add Cat Shield to the Accessibility list.");
            eprintln!("  Waiting for permissions...");
            eprintln!();

            // Poll for permissions every 1 second using CFRunLoopRunInMode
            // This allows the run loop to process events while waiting,
            // which is necessary for macOS to update accessibility permission state
            const POLL_INTERVAL_SECS: f64 = 1.0;
            loop {
                unsafe {
                    let mode = kCFRunLoopDefaultMode.expect("kCFRunLoopDefaultMode should exist");
                    CFRunLoopRunInMode((mode as *const CFString).cast(), POLL_INTERVAL_SECS, false);
                }
                if check_accessibility() {
                    println!("  ✓ Permissions granted! Starting Cat Shield...");
                    println!();
                    break;
                }
            }
        }
    }

    println!();
    println!("  🐱 CAT SHIELD 🛡️");
    println!("  ════════════════════════════════════════");
    println!("  Protecting your work from curious cats!");
    println!();

    // Get the main screen dimensions
    let screen = NSScreen::mainScreen(mtm);
    let screen = match screen {
        Some(s) => s,
        None => {
            eprintln!("  ✗ Failed to get main screen");
            process::exit(1);
        }
    };
    let screen_frame = screen.frame();

    // Create a fullscreen, borderless window
    let window = unsafe {
        let window = NSWindow::alloc(mtm);
        NSWindow::initWithContentRect_styleMask_backing_defer(
            window,
            screen_frame,
            NSWindowStyleMask::Borderless,
            NSBackingStoreType::Buffered,
            false,
        )
    };

    // Configure window to be topmost
    window.setLevel(NS_SCREEN_SAVER_WINDOW_LEVEL);

    // Set window to appear on all spaces and stay visible
    window.setCollectionBehavior(
        NSWindowCollectionBehavior::CanJoinAllSpaces
            | NSWindowCollectionBehavior::Stationary
            | NSWindowCollectionBehavior::IgnoresCycle,
    );

    // Window must be non-opaque to allow transparent background, but keep alphaValue at 1.0
    // so that subviews (like label containers) can render fully opaque
    window.setOpaque(false);
    window.setAlphaValue(1.0);

    // Set a semi-transparent dark background color (the overlay effect)
    let bg_color = NSColor::colorWithRed_green_blue_alpha(0.1, 0.1, 0.15, 0.5);
    window.setBackgroundColor(Some(&bg_color));

    // Keep window visible
    window.setHidesOnDeactivate(false);

    // Accept mouse events (needed for blocking)
    window.setIgnoresMouseEvents(false);

    // Set title
    window.setTitle(ns_string!("Cat Shield"));

    // Required when creating NSWindow outside a window controller
    unsafe {
        window.setReleasedWhenClosed(false);
    }

    // Show the window
    window.makeKeyAndOrderFront(None);

    println!("  ✓ Overlay window active");

    // Create and add the close button in top-right corner
    let close_button_frame = CGRect {
        origin: CGPoint {
            x: screen_frame.size.width - CLOSE_BUTTON_SIZE - CLOSE_BUTTON_MARGIN,
            y: screen_frame.size.height - CLOSE_BUTTON_SIZE - CLOSE_BUTTON_MARGIN,
        },
        size: CGSize {
            width: CLOSE_BUTTON_SIZE,
            height: CLOSE_BUTTON_SIZE,
        },
    };

    let close_button = CloseButtonView::new(mtm, close_button_frame);

    // Store view reference for timer callback.
    // Safety: The view remains valid because contentView retains it and
    // app.run() blocks until we're ready to exit. The timer is stopped
    // before cleanup begins.
    CLOSE_BUTTON_VIEW.store(
        Retained::as_ptr(&close_button) as *mut c_void,
        Ordering::SeqCst,
    );

    // Create the close button label view (positioned below the button)
    let label_x = screen_frame.size.width
        - CLOSE_BUTTON_MARGIN
        - CLOSE_BUTTON_SIZE / 2.0
        - CLOSE_BUTTON_LABEL_WIDTH / 2.0;
    let label_y = screen_frame.size.height
        - CLOSE_BUTTON_SIZE
        - CLOSE_BUTTON_MARGIN
        - CLOSE_BUTTON_LABEL_HEIGHT
        - 5.0; // 5px gap between button and label

    let close_button_label_frame = CGRect {
        origin: CGPoint {
            x: label_x,
            y: label_y,
        },
        size: CGSize {
            width: CLOSE_BUTTON_LABEL_WIDTH,
            height: CLOSE_BUTTON_LABEL_HEIGHT,
        },
    };
    let close_button_label = CloseButtonLabelView::new(mtm, close_button_label_frame);

    // Store label view reference for timer callback updates
    CLOSE_BUTTON_LABEL_VIEW.store(
        Retained::as_ptr(&close_button_label) as *mut c_void,
        Ordering::SeqCst,
    );

    // Add close button and label to the window's content view
    if let Some(content_view) = window.contentView() {
        content_view.addSubview(&close_button);
        content_view.addSubview(&close_button_label);
    }

    // Start the animation timer
    start_close_button_timer();

    println!("  ✓ Close button active (hold 3s to exit)");
    println!("  ✓ Exit key: {}", exit_key.display_name);

    // Set up auto-exit timer if specified
    if let Some(duration_secs) = args.timer {
        init_auto_exit_timer(duration_secs);
        println!(
            "  ✓ Auto-exit timer set: {}",
            format_duration(duration_secs)
        );

        // Create timer display view if not hidden
        if !args.hide_timer {
            let timer_display_frame = CGRect {
                origin: CGPoint {
                    x: TIMER_DISPLAY_MARGIN,
                    y: screen_frame.size.height - TIMER_DISPLAY_HEIGHT - TIMER_DISPLAY_MARGIN,
                },
                size: CGSize {
                    width: TIMER_DISPLAY_WIDTH,
                    height: TIMER_DISPLAY_HEIGHT,
                },
            };

            let timer_display = TimerDisplayView::new(mtm, timer_display_frame);

            // Store view reference for timer callback
            TIMER_DISPLAY_VIEW.store(
                Retained::as_ptr(&timer_display) as *mut c_void,
                Ordering::SeqCst,
            );

            // Add timer display to the window's content view
            if let Some(content_view) = window.contentView() {
                content_view.addSubview(&timer_display);
            }

            println!("  ✓ Timer display active");
        }
    }

    // Prevent sleep
    let assertion_id = prevent_sleep();

    // Set up event tap - this is the core security feature
    // Without input blocking, the shield is just a visual overlay
    if !setup_event_tap() {
        eprintln!("  ✗ Failed to create event tap");
        eprintln!();
        eprintln!("  ════════════════════════════════════════");
        eprintln!("  ⚠️  SHIELD ACTIVATION FAILED");
        eprintln!("  ════════════════════════════════════════");
        eprintln!();
        eprintln!("  Input blocking could not be enabled.");
        eprintln!("  The shield cannot protect without this feature.");
        eprintln!();
        eprintln!("  Please check:");
        eprintln!("  - Accessibility permissions are granted");
        eprintln!("  - No other apps are blocking event taps");
        eprintln!();
        std::process::exit(1);
    }
    println!("  ✓ Input blocking active");

    println!();
    println!("  ═══════════════════════════════════════");
    println!("  🛡️  CAT SHIELD IS NOW ACTIVE!");
    println!("  ═══════════════════════════════════════");
    println!();
    println!("  Exit: Hold X button (top-right) for 3 seconds");
    println!("        Or press {}", exit_key.display_name);
    if args.timer.is_some() {
        println!(
            "        Or wait for timer ({} remaining)",
            format_duration(get_remaining_seconds())
        );
    }
    println!();

    // Run the NSApplication event loop (required for AppKit event handling)
    app.run();

    // Cleanup
    stop_close_button_timer();

    if let Some(id) = assertion_id {
        allow_sleep(id);
    }

    println!();
    println!("  👋 Cat Shield deactivated. Goodbye!");
    println!();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_hold_progress_zero() {
        assert_eq!(calculate_hold_progress(0.0, 3.0), 0.0);
    }

    #[test]
    fn test_calculate_hold_progress_partial() {
        let progress = calculate_hold_progress(1.5, 3.0);
        assert!((progress - 0.5).abs() < f64::EPSILON);
    }

    #[test]
    fn test_calculate_hold_progress_complete() {
        assert_eq!(calculate_hold_progress(3.0, 3.0), 1.0);
    }

    #[test]
    fn test_calculate_hold_progress_exceeds() {
        // Should clamp to 1.0 when elapsed exceeds duration
        assert_eq!(calculate_hold_progress(5.0, 3.0), 1.0);
    }

    #[test]
    fn test_is_hold_complete_false() {
        assert!(!is_hold_complete(2.0, 3.0));
        assert!(!is_hold_complete(2.999, 3.0));
    }

    #[test]
    fn test_is_hold_complete_exact() {
        assert!(is_hold_complete(3.0, 3.0));
    }

    #[test]
    fn test_is_hold_complete_exceeds() {
        assert!(is_hold_complete(5.0, 3.0));
    }

    #[test]
    fn test_parse_duration_minutes() {
        assert_eq!(parse_duration("30m").unwrap(), 30 * 60);
        assert_eq!(parse_duration("1m").unwrap(), 60);
        assert_eq!(parse_duration("90m").unwrap(), 90 * 60);
    }

    #[test]
    fn test_parse_duration_hours() {
        assert_eq!(parse_duration("1h").unwrap(), 3600);
        assert_eq!(parse_duration("2h").unwrap(), 2 * 3600);
        assert_eq!(parse_duration("24h").unwrap(), 24 * 3600);
    }

    #[test]
    fn test_parse_duration_combined() {
        assert_eq!(parse_duration("1h30m").unwrap(), 3600 + 30 * 60);
        assert_eq!(parse_duration("2h45m").unwrap(), 2 * 3600 + 45 * 60);
    }

    #[test]
    fn test_parse_duration_with_spaces() {
        assert_eq!(parse_duration(" 30m ").unwrap(), 30 * 60);
        assert_eq!(parse_duration("1h 30m").unwrap(), 3600 + 30 * 60);
    }

    #[test]
    fn test_parse_duration_bare_number_as_minutes() {
        // A bare number without unit is treated as minutes
        assert_eq!(parse_duration("30").unwrap(), 30 * 60);
        assert_eq!(parse_duration("60").unwrap(), 60 * 60);
    }

    #[test]
    fn test_parse_duration_seconds() {
        assert_eq!(parse_duration("90s").unwrap(), 90);
        assert_eq!(parse_duration("1m30s").unwrap(), 90);
    }

    #[test]
    fn test_parse_duration_errors() {
        assert!(parse_duration("").is_err());
        assert!(parse_duration("0m").is_err());
        assert!(parse_duration("abc").is_err());
        assert!(parse_duration("30x").is_err());
        assert!(parse_duration("30s").is_err()); // Less than 1 minute
        assert!(parse_duration("25h").is_err()); // More than 24 hours
    }

    #[test]
    fn test_format_duration_seconds_only() {
        assert_eq!(format_duration(45), "45s");
        assert_eq!(format_duration(1), "1s");
    }

    #[test]
    fn test_format_duration_minutes_and_seconds() {
        assert_eq!(format_duration(90), "1m 30s");
        assert_eq!(format_duration(3599), "59m 59s");
    }

    #[test]
    fn test_format_duration_hours_minutes_seconds() {
        assert_eq!(format_duration(3600), "1h 00m 00s");
        assert_eq!(format_duration(3661), "1h 01m 01s");
        assert_eq!(format_duration(7200 + 1800 + 45), "2h 30m 45s");
    }

    // Exit key parsing tests
    #[test]
    fn test_keycode_from_name_letters() {
        assert_eq!(keycode_from_name("a"), Some(0));
        assert_eq!(keycode_from_name("u"), Some(32));
        assert_eq!(keycode_from_name("q"), Some(12));
        assert_eq!(keycode_from_name("U"), Some(32)); // Case insensitive
    }

    #[test]
    fn test_keycode_from_name_special() {
        assert_eq!(keycode_from_name("escape"), Some(53));
        assert_eq!(keycode_from_name("Escape"), Some(53));
        assert_eq!(keycode_from_name("esc"), Some(53));
        assert_eq!(keycode_from_name("return"), Some(36));
        assert_eq!(keycode_from_name("enter"), Some(36));
        assert_eq!(keycode_from_name("space"), Some(49));
        assert_eq!(keycode_from_name("tab"), Some(48));
    }

    #[test]
    fn test_keycode_from_name_function_keys() {
        assert_eq!(keycode_from_name("f1"), Some(122));
        assert_eq!(keycode_from_name("F12"), Some(111));
    }

    #[test]
    fn test_keycode_from_name_unknown() {
        assert_eq!(keycode_from_name("unknown"), None);
        assert_eq!(keycode_from_name(""), None);
    }

    #[test]
    fn test_exit_key_parse_default() {
        let key = ExitKey::parse("Cmd+Option+U").unwrap();
        assert_eq!(key.keycode, 32);
        assert!(key.requires_cmd);
        assert!(key.requires_option);
        assert!(!key.requires_shift);
        assert!(!key.requires_ctrl);
    }

    #[test]
    fn test_exit_key_parse_cmd_shift_q() {
        let key = ExitKey::parse("Cmd+Shift+Q").unwrap();
        assert_eq!(key.keycode, 12);
        assert!(key.requires_cmd);
        assert!(!key.requires_option);
        assert!(key.requires_shift);
        assert!(!key.requires_ctrl);
    }

    #[test]
    fn test_exit_key_parse_ctrl_option_escape() {
        let key = ExitKey::parse("Ctrl+Option+Escape").unwrap();
        assert_eq!(key.keycode, 53);
        assert!(!key.requires_cmd);
        assert!(key.requires_option);
        assert!(!key.requires_shift);
        assert!(key.requires_ctrl);
    }

    #[test]
    fn test_exit_key_parse_case_insensitive() {
        let key1 = ExitKey::parse("CMD+OPTION+U").unwrap();
        let key2 = ExitKey::parse("cmd+option+u").unwrap();
        assert_eq!(key1.keycode, key2.keycode);
        assert_eq!(key1.requires_cmd, key2.requires_cmd);
        assert_eq!(key1.requires_option, key2.requires_option);
    }

    #[test]
    fn test_exit_key_parse_alternative_modifier_names() {
        let key = ExitKey::parse("Command+Alt+U").unwrap();
        assert!(key.requires_cmd);
        assert!(key.requires_option);

        let key2 = ExitKey::parse("Control+Opt+X").unwrap();
        assert!(key2.requires_ctrl);
        assert!(key2.requires_option);
    }

    #[test]
    fn test_exit_key_parse_with_spaces() {
        let key = ExitKey::parse(" Cmd + Option + U ").unwrap();
        assert_eq!(key.keycode, 32);
        assert!(key.requires_cmd);
        assert!(key.requires_option);
    }

    #[test]
    fn test_exit_key_parse_errors() {
        // No modifier
        assert!(ExitKey::parse("U").is_err());

        // Unknown key
        assert!(ExitKey::parse("Cmd+Option+Unknown").is_err());

        // Empty
        assert!(ExitKey::parse("").is_err());

        // No key, only modifiers
        assert!(ExitKey::parse("Cmd+Option").is_err());

        // Multiple keys
        assert!(ExitKey::parse("Cmd+A+B").is_err());
    }

    #[test]
    fn test_exit_key_default() {
        let key = ExitKey::default();
        assert_eq!(key.keycode, 32);
        assert!(key.requires_cmd);
        assert!(key.requires_option);
        assert!(!key.requires_shift);
        assert!(!key.requires_ctrl);
        assert_eq!(key.display_name, "Cmd+Option+U");
    }

    // Menu bar mode tests
    #[test]
    fn test_has_immediate_start_args_none() {
        let args = Args {
            timer: None,
            hide_timer: false,
            exit_key: None,
        };
        assert!(!has_immediate_start_args(&args));
    }

    #[test]
    fn test_has_immediate_start_args_with_timer() {
        let args = Args {
            timer: Some(60),
            hide_timer: false,
            exit_key: None,
        };
        assert!(has_immediate_start_args(&args));
    }

    #[test]
    fn test_has_immediate_start_args_with_exit_key() {
        let args = Args {
            timer: None,
            hide_timer: false,
            exit_key: Some(ExitKey::default()),
        };
        assert!(has_immediate_start_args(&args));
    }

    #[test]
    fn test_has_immediate_start_args_with_both() {
        let args = Args {
            timer: Some(120),
            hide_timer: true,
            exit_key: Some(ExitKey::default()),
        };
        assert!(has_immediate_start_args(&args));
    }

    #[test]
    fn test_has_immediate_start_args_hide_timer_alone_is_menu_mode() {
        // hide_timer alone should NOT trigger immediate mode
        let args = Args {
            timer: None,
            hide_timer: true,
            exit_key: None,
        };
        assert!(!has_immediate_start_args(&args));
    }

    // =========================================================
    // Config Integration Tests
    // =========================================================

    // Config Loading Tests

    #[test]
    fn test_config_load_missing_file_returns_default() {
        let temp_dir = tempfile::tempdir().unwrap();
        let config_path = temp_dir.path().join("nonexistent.toml");

        let config = Config::load_from_path(&config_path);

        assert!(config.exit_key.is_none());
        assert!(config.default_timer.is_none());
        assert!(config.overlay_opacity.is_none());
    }

    #[test]
    fn test_config_load_empty_file_returns_default() {
        let temp_dir = tempfile::tempdir().unwrap();
        let config_path = temp_dir.path().join("config.toml");
        std::fs::write(&config_path, "").unwrap();

        let config = Config::load_from_path(&config_path);

        assert!(config.exit_key.is_none());
        assert!(config.default_timer.is_none());
        assert!(config.overlay_opacity.is_none());
    }

    #[test]
    fn test_config_load_valid_toml() {
        let temp_dir = tempfile::tempdir().unwrap();
        let config_path = temp_dir.path().join("config.toml");
        std::fs::write(
            &config_path,
            r#"
exit_key = "Cmd+Shift+Q"
default_timer = "30m"
overlay_opacity = 0.6
"#,
        )
        .unwrap();

        let config = Config::load_from_path(&config_path);

        assert_eq!(config.exit_key, Some("Cmd+Shift+Q".to_string()));
        assert_eq!(config.default_timer, Some("30m".to_string()));
        assert_eq!(config.overlay_opacity, Some(0.6));
    }

    #[test]
    fn test_config_load_partial_toml_uses_defaults() {
        let temp_dir = tempfile::tempdir().unwrap();
        let config_path = temp_dir.path().join("config.toml");
        std::fs::write(&config_path, r#"exit_key = "Cmd+Shift+Q""#).unwrap();

        let config = Config::load_from_path(&config_path);

        assert_eq!(config.exit_key, Some("Cmd+Shift+Q".to_string()));
        assert!(config.default_timer.is_none());
        assert!(config.overlay_opacity.is_none());
    }

    #[test]
    fn test_config_load_invalid_toml_returns_default() {
        let temp_dir = tempfile::tempdir().unwrap();
        let config_path = temp_dir.path().join("config.toml");
        std::fs::write(&config_path, "this is not valid toml {{{{").unwrap();

        let config = Config::load_from_path(&config_path);

        assert!(config.exit_key.is_none());
        assert!(config.default_timer.is_none());
        assert!(config.overlay_opacity.is_none());
    }

    #[test]
    fn test_config_load_with_unknown_fields_ignores_them() {
        let temp_dir = tempfile::tempdir().unwrap();
        let config_path = temp_dir.path().join("config.toml");
        std::fs::write(
            &config_path,
            r#"
exit_key = "Cmd+Option+X"
unknown_field = "should be ignored"
another_unknown = 42
"#,
        )
        .unwrap();

        let config = Config::load_from_path(&config_path);

        assert_eq!(config.exit_key, Some("Cmd+Option+X".to_string()));
    }

    // Config Saving Tests

    #[test]
    fn test_config_save_creates_directory() {
        let temp_dir = tempfile::tempdir().unwrap();
        let config_path = temp_dir.path().join("subdir").join("config.toml");

        let config = Config {
            exit_key: Some("Cmd+Option+U".to_string()),
            default_timer: None,
            overlay_opacity: None,
        };

        config.save_to_path(&config_path).unwrap();

        assert!(config_path.parent().unwrap().exists());
    }

    #[test]
    fn test_config_save_creates_file() {
        let temp_dir = tempfile::tempdir().unwrap();
        let config_path = temp_dir.path().join("config.toml");

        let config = Config {
            exit_key: Some("Cmd+Option+U".to_string()),
            default_timer: None,
            overlay_opacity: None,
        };

        config.save_to_path(&config_path).unwrap();

        assert!(config_path.exists());
    }

    #[test]
    fn test_config_save_writes_valid_toml() {
        let temp_dir = tempfile::tempdir().unwrap();
        let config_path = temp_dir.path().join("config.toml");

        let config = Config {
            exit_key: Some("Cmd+Shift+X".to_string()),
            default_timer: Some("1h".to_string()),
            overlay_opacity: Some(0.7),
        };

        config.save_to_path(&config_path).unwrap();

        let content = std::fs::read_to_string(&config_path).unwrap();
        assert!(content.contains("exit_key = \"Cmd+Shift+X\""));
        assert!(content.contains("default_timer = \"1h\""));
        assert!(content.contains("overlay_opacity = 0.7"));
    }

    #[test]
    fn test_config_save_overwrites_existing() {
        let temp_dir = tempfile::tempdir().unwrap();
        let config_path = temp_dir.path().join("config.toml");

        // Write initial config
        let config1 = Config {
            exit_key: Some("Cmd+Option+A".to_string()),
            default_timer: None,
            overlay_opacity: None,
        };
        config1.save_to_path(&config_path).unwrap();

        // Overwrite with new config
        let config2 = Config {
            exit_key: Some("Cmd+Option+B".to_string()),
            default_timer: Some("2h".to_string()),
            overlay_opacity: Some(0.3),
        };
        config2.save_to_path(&config_path).unwrap();

        let content = std::fs::read_to_string(&config_path).unwrap();
        assert!(content.contains("exit_key = \"Cmd+Option+B\""));
        assert!(content.contains("default_timer = \"2h\""));
        assert!(!content.contains("Cmd+Option+A"));
    }

    // Config Round-Trip Tests

    #[test]
    fn test_config_round_trip_all_fields() {
        let temp_dir = tempfile::tempdir().unwrap();
        let config_path = temp_dir.path().join("config.toml");

        let original = Config {
            exit_key: Some("Ctrl+Option+Escape".to_string()),
            default_timer: Some("45m".to_string()),
            overlay_opacity: Some(0.65),
        };

        original.save_to_path(&config_path).unwrap();
        let loaded = Config::load_from_path(&config_path);

        assert_eq!(original.exit_key, loaded.exit_key);
        assert_eq!(original.default_timer, loaded.default_timer);
        assert_eq!(original.overlay_opacity, loaded.overlay_opacity);
    }

    #[test]
    fn test_config_round_trip_partial_fields() {
        let temp_dir = tempfile::tempdir().unwrap();
        let config_path = temp_dir.path().join("config.toml");

        let original = Config {
            exit_key: Some("Cmd+Shift+Q".to_string()),
            default_timer: None,
            overlay_opacity: Some(0.4),
        };

        original.save_to_path(&config_path).unwrap();
        let loaded = Config::load_from_path(&config_path);

        assert_eq!(original.exit_key, loaded.exit_key);
        assert!(loaded.default_timer.is_none());
        assert_eq!(original.overlay_opacity, loaded.overlay_opacity);
    }

    // Config Opacity Tests

    #[test]
    fn test_config_opacity_default_when_none() {
        let config = Config {
            exit_key: None,
            default_timer: None,
            overlay_opacity: None,
        };

        assert_eq!(config.opacity(), DEFAULT_OVERLAY_OPACITY);
        assert_eq!(config.opacity(), 0.5);
    }

    #[test]
    fn test_config_opacity_clamps_below_min() {
        let config = Config {
            exit_key: None,
            default_timer: None,
            overlay_opacity: Some(0.1), // Below MIN_OVERLAY_OPACITY (0.2)
        };

        assert_eq!(config.opacity(), MIN_OVERLAY_OPACITY);
        assert_eq!(config.opacity(), 0.2);
    }

    #[test]
    fn test_config_opacity_clamps_above_max() {
        let config = Config {
            exit_key: None,
            default_timer: None,
            overlay_opacity: Some(0.95), // Above MAX_OVERLAY_OPACITY (0.8)
        };

        assert_eq!(config.opacity(), MAX_OVERLAY_OPACITY);
        assert_eq!(config.opacity(), 0.8);
    }

    #[test]
    fn test_config_opacity_valid_value_unchanged() {
        let config = Config {
            exit_key: None,
            default_timer: None,
            overlay_opacity: Some(0.6),
        };

        assert_eq!(config.opacity(), 0.6);
    }
}
