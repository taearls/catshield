# Changelog

All notable changes to Cat Shield are documented in this file.

*See [ROADMAP.md](ROADMAP.md) for current project status and open issues.*

## [Unreleased]

### Phase 9: iced UI Migration (Epic #152)

This release represents a major architectural change: migration from platform-specific UI implementations to the [iced](https://iced.rs) cross-platform GUI framework. The iced framework now handles all visual rendering while platform-native APIs continue to handle input blocking for security.

**New Features:**
- **Animated cat companion** (#165) - A cute animated cat watches over your screen with idle, blinking, and sleeping states. Disable with `--no-cat` flag or `show_cat = false` in config
- **Overlay customization** (#159) - Adjust opacity (10-90%) with `--opacity` and color with `--color` (presets: gray, blue, green, red, purple, or hex codes like `#1a2b3c`)
- **Timer countdown display** (#156) - Visual countdown with progress bar when using `--timer`
- **Settings window** (#157) - Full GUI for configuring all options with live preview
- **Settings persistence** (#158) - All settings saved to `~/.config/catshield/config.toml`
- **Dark/light theme support** (#163, #179) - Automatically adapts to system theme preference
- **Performance optimizations** (#164) - Criterion benchmarks, optimized release profile

**Platform Integration:**
- macOS menu bar integration (#160) - NSStatusItem works alongside iced windows
- Windows system tray integration (#161) - Shell_NotifyIcon works alongside iced windows
- Linux tray integration (#162) - ksni StatusNotifierItem works alongside iced windows

**Architecture:**
- Hybrid approach: iced for UI rendering, platform-native APIs for input blocking
- iced 0.14 with canvas and tokio features
- Elm architecture: State, Message, View pattern
- Thread-spawned iced windows from native menu/tray callbacks

**Technical Details:**
- Added `iced` 0.14 dependency with `canvas` and `tokio` features
- New `ui_iced` module with: `overlay`, `settings`, `about`, `theme`, `cat_animation`, `integration`
- Theme-aware styling with 24+ style functions supporting dark/light modes
- Cat animation: ~30 FPS bobbing, 3-6 second random blink intervals
- Sub-microsecond timer formatting (<100ns per benchmark)

### 2026-01-15
- Completed Issue #102: Add Windows system tray implementation
  - Implemented `WindowsSystemTray` in `src/platform/windows/tray.rs`
  - Implements the `SystemTray` trait for Windows
  - Uses `Shell_NotifyIcon` with `NOTIFYICONDATA` for tray icon management
  - Custom window for receiving tray messages via `WM_TRAYICON`
  - Context menu via `TrackPopupMenu` with Start/Stop Protection, Settings, About, Quit
  - Menu callbacks via atomic function pointer storage

### 2026-01-14
- Completed Issue #108: Add Linux power management via DBus
  - Implemented `LinuxPowerManager` in `src/platform/linux/power.rs`
  - Implements the `PowerManager` trait for Linux desktop environments
  - Supports multiple D-Bus interfaces for different desktop environments:
    - `org.freedesktop.ScreenSaver` - Standard FreeDesktop interface (most DEs)
    - `org.gnome.SessionManager` - GNOME-specific interface with suspend + idle inhibit flags
    - `org.freedesktop.PowerManagement.Inhibit` - Legacy interface for older systems
  - Automatically tries interfaces in order of preference until one succeeds
  - Uses `zbus` crate (v5.5) with blocking API for D-Bus communication
  - Thread-safe implementation with atomic counters and mutex-protected state
  - Proper cookie tracking ensures inhibitions can be released correctly
  - Works on GNOME, KDE, XFCE, and other common desktop environments
  - Added 8 unit tests covering constructor, trait compliance, and error handling

- Completed Issue #111: Add Linux keycode mappings
  - Fully implemented `src/input/keycodes/linux.rs` (was previously a stub)
  - Added `key_to_keycode()` function: converts `Key` enum to X11 keysyms
  - Added `keycode_to_key()` function: converts X11 keysyms to `Key` enum
  - Complete mappings for all 57 supported keys using X11 keysym constants
  - Added 14 comprehensive unit tests for key mappings

- Completed Issue #107: Research Wayland input blocking solutions
  - Created comprehensive research document at `docs/WAYLAND_INPUT_RESEARCH.md`
  - **Key finding**: Full input blocking is not possible on Wayland by design (security)
  - **Partial solution**: Fullscreen layer-shell + keyboard-shortcuts-inhibit for wlroots
  - Created follow-up issues #131 and #132

- Completed Issue #128: Migrate from println!/eprintln! to log crate
  - Added `log` v0.4 and `env_logger` v0.11 dependencies
  - Log level controlled via CLI `-v` flags: `-v` (info), `-vv` (debug), `-vvv` (trace)
  - Migrated all `println!` and `eprintln!` calls across 15 source files

- Completed Issue #101: Add Windows power management (PowerManager)
  - Created `src/platform/windows/power.rs` with `WindowsPowerManager` implementation
  - Uses `SetThreadExecutionState` API with `ES_CONTINUOUS | ES_DISPLAY_REQUIRED`

- Completed Issue #100: Add Windows keyboard hook implementation (InputBlocker)
  - Implemented `WindowsInputBlocker` using `SetWindowsHookExW` with `WH_KEYBOARD_LL`
  - Support for exit key detection and allowed keys list

- Completed Issue #104: Add Windows keycode mappings
  - Complete mappings for all 57 supported keys to Windows virtual key codes

- Completed Issue #112: Set up cross-platform GitHub Actions CI
  - Build and test jobs run on `macos-latest`, `windows-latest`, and `ubuntu-latest`

- Completed Issue #98: Update shield_core.rs to use platform traits
  - Refactored to use `PermissionChecker` trait with platform-specific implementations

- Completed Issue #120: Add Docker containers for cross-platform local development

### 2026-01-13
- Completed Issue #97: Create canonical Key enum and split keycodes by platform
  - Created `src/input/keycodes/` directory structure
  - Platform-agnostic `Key` enum with macOS, Windows, and Linux implementations
  - Added 27 new unit tests

- Completed Issue #99: Update Cargo.toml for conditional platform dependencies
  - Windows: `windows` crate with Win32 features
  - Linux: `x11rb` and `zbus` crates

### 2026-01-12
- Completed Issue #96: Implement platform traits for macOS
  - Created `MacOSInputBlocker`, `MacOSPowerManager`, `MacOSPermissionChecker`

- Completed Issue #95: Reorganize macOS platform code into subdirectory
  - Moved platform code to `src/platform/macos/`

- Completed Issue #94: Create platform abstraction traits
  - `InputBlocker`, `PowerManager`, `PermissionChecker`, `SystemTray`, `OverlayWindow`

- Completed Issue #85: Add Undo button to Settings window
- Completed Issue #91: Fix AppKit threading violation in action feedback
- Completed Issue #86: Show duplicate key validation in Settings
- Completed Issue #87: Fix Settings menu item staying disabled

### 2026-01-11
- Completed Issue #83: Add ability to select and remove individual allowed keys
- Completed Issue #80: Fix Settings Cancel button not discarding allowed_keys
- Completed Issue #65: Add preset groups for key allowlist

### 2026-01-10
- Completed Issue #64: Add configurable key allowlist
- Completed Issue #54: Cache formatted duration string
- Completed Issue #60: Expand UI state and settings validation tests
- Completed Issue #75: Add integration tests for menu bar timer
- Completed Issue #73: Reduce minimum auto-exit timer to 5 seconds
- Completed Issue #53: Cache redundant atomic loads in timer callback
- Completed Issue #57: Add safety documentation to unsafe blocks
- Completed Issue #56: Create pointer helper to reduce null-check duplication
- Completed Issue #52: Optimize atomic orderings from SeqCst
- Completed Issue #55: Refactor show_settings_window() into smaller functions

### 2026-01-09
- Completed Issue #59: Add unit tests for timer state module (22 tests)
- Completed Issue #58: Add unit tests for lock module (22 tests)
- Completed Issue #49: Add real-time validation for Timer duration field
- Completed Issue #48: Exit Key validation should not show on initial open
- Completed Issue #46: Add 'Reset to Default' button in Settings
- Completed Issue #44: Refactoring - consolidate global state
- Completed Issue #42: Enable Help menu links

### 2026-01-08
- Completed Issue #11: Add install script for easy CLI access
- Completed Issue #38: Fix menu items not visually disabled
- Completed Issue #30: Add real-time validation for Exit Key field

### 2026-01-07
- Completed Issue #19: Add About Panel
- Completed Issue #35: Split main.rs into multiple modules (88% reduction)
- Completed Issue #24: Fix multiple menu bar icons bug
- Completed Issue #31: Add CI workflow

### 2026-01-06
- Completed Issue #16: Create Settings Window
- Completed Issue #18: Extend Config for New Settings
- Completed Issue #10: Add informative labels to overlay UI
- Completed Issue #25: Limit Claude GitHub Action to comments only
- Completed Issue #17: Refactor Overlay to On-Demand Activation

### 2026-01-03
- Completed Issue #15: Create Main Dropdown Menu

### 2026-01-02
- Completed Issue #14: Create Menu Bar Infrastructure (NSStatusItem)

### 2025-12-31
- Completed Issue #7: Add configurable keyboard shortcut for exit
- Completed Issue #6: Add configurable timer-based auto-exit
- Completed Issue #5: Add click-and-hold close button
- Completed Issue #3: Migration to objc2 ecosystem

## Earlier History

### Phase 1: Initial Release
- Core overlay window implementation
- Keyboard and mouse input blocking via CGEventTap
- Sleep prevention using IOKit power assertions
- Unlock mechanism (Cmd+Option+U)
- Accessibility permission handling
