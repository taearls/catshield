# Cat Shield Roadmap

This document tracks the development progress and future plans for Cat Shield.

## Current Status

Cat Shield is a macOS utility that creates a semi-transparent overlay to block keyboard and mouse input, protecting your work from curious cats (or other interruptions).

## Completed

### Phase 1: Initial Release
- [x] Core overlay window implementation
- [x] Keyboard and mouse input blocking via CGEventTap
- [x] Sleep prevention using IOKit power assertions
- [x] Unlock mechanism (Cmd+Option+U)
- [x] Accessibility permission handling

### Phase 2: Technical Debt
- [x] **Issue #3**: Migrate from deprecated `cocoa`/`objc` crates to `objc2` ecosystem
  - Replaced deprecated crates (`cocoa`, `objc`, `core-graphics`, `core-foundation`) with modern `objc2` ecosystem
  - Updated to `objc2`, `objc2-foundation`, `objc2-app-kit`, `objc2-core-foundation`, `objc2-core-graphics`
  - Improved memory safety with modern Rust abstractions
  - Eliminated 50+ deprecation warnings from build

### Phase 3: User Experience
- [x] **Issue #5**: Add click-and-hold close button as default exit mechanism
  - Added close button (X) in top-right corner of overlay
  - Requires 3-second hold to activate (prevents accidental cat-triggered exits)
  - Visual progress ring indicator during hold
  - Works without Accessibility permissions
  - Provides reliable exit mechanism when keyboard shortcut not available
- [x] **Issue #6**: Add configurable timer-based auto-exit
  - CLI argument for timer duration: `--timer` / `-t` (supports minutes, hours, combined)
  - Visual countdown progress bar on overlay (can be hidden with `--hide-timer`)
  - Warning notification 1 minute before auto-exit
  - Clean exit when timer expires
  - Timer validation (5 seconds minimum, 24 hours maximum)
- [x] **Issue #7**: Add configurable keyboard shortcut for exit
  - CLI argument: `--exit-key` / `-e` (e.g., "Cmd+Shift+Q", "Ctrl+Option+Escape")
  - Config file support: `~/.config/catshield/config.toml` with `exit_key = "Cmd+Option+U"`
  - Full key combination validation with descriptive error messages
  - Support for common modifier keys (Cmd, Option, Shift, Ctrl) with aliases
  - Support for letters (A-Z), numbers (0-9), function keys (F1-F12), and special keys
  - CLI argument takes precedence over config file
  - Default is Cmd+Q (standard macOS quit shortcut)

### Configuration Improvements
- [x] **Issue #25**: Limit Claude GitHub Action to only respond to comments
  - Removed triggers for `pull_request`, `issues`, `pull_request_review`, `pull_request_review_comment`
  - Kept only `issue_comment` trigger (fires on both issue and PR comments)
  - Added `if: contains(github.event.comment.body, '@claude')` condition
  - Claude now only responds when explicitly mentioned with `@claude`

### CI/CD Infrastructure
- [x] **Issue #31**: Add CI workflow with lint, format, test, and build checks
  - Created `.github/workflows/ci.yml` with comprehensive CI pipeline
  - Triggers on push to main, PRs targeting main, and manual dispatch
  - Lint & Format job: `cargo fmt --check` and `cargo clippy -D warnings`
  - Test job: `cargo test --verbose`
  - Build job: `cargo build --release`
  - All jobs run on `macos-latest` (required for objc2/AppKit)
  - Uses `dtolnay/rust-toolchain@stable` and `Swatinem/rust-cache@v2` for caching

### Phase 3.5: Overlay Usability
- [x] **Issue #10**: Add informative labels to overlay UI elements
  - Timer display now shows "Time Remaining:" header with countdown text (e.g., "29m 45s")
  - Warning state shows yellow text with "Exiting soon!" indicator when < 1 minute
  - Close button has "Hold 3s to exit" instruction label below it
  - During hold, label shows countdown ("3s...", "2s...", "1s...")
  - Text rendering via NSAttributedString with proper font and color support
  - All labels have good contrast against the overlay background

### Phase 4: Menu-Based Application Interface (In Progress)
- [x] **Issue #14**: Create Menu Bar Infrastructure (NSStatusItem)
  - Added NSStatusItem with cat emoji (🐱) in menu bar
  - App enters menu bar mode when launched without CLI arguments
  - Tooltip shows "Cat Shield" on hover
  - Basic dropdown menu with branding and Quit option
  - CLI arguments (--timer, --exit-key) bypass menu bar and start shield immediately
  - Foundation for subsequent menu features (#15, #16, #17)
- [x] **Issue #15**: Create Main Dropdown Menu
  - Comprehensive menu structure with all application features
  - Protection section: Start/Stop Protection items (ready for #17)
  - Configuration section: Settings with Cmd+, shortcut (ready for #16)
  - Information section: About Cat Shield (ready for #19) and Help submenu
  - Help submenu with View Documentation, Report Issue, and Release Notes
  - All menu items include tooltips explaining their purpose
  - Keyboard shortcuts: Cmd+Q (Quit), Cmd+, (Settings)
  - Stop Protection initially hidden, will show when shield is active
  - Proper menu organization with section separators
  - Foundation complete for #16, #17, and #19 to implement functionality
- [x] **Issue #17**: Refactor Overlay to On-Demand Activation
  - Running `catshield` (no args) shows menu bar icon only, no overlay
  - Running `catshield --timer` or `catshield --exit-key` starts protection immediately
  - "Start Protection" menu item activates the shield on click
  - Shield exits (close button, exit key, timer) return to menu bar state
  - Prevents double-activation: menu item disabled while shield is active
  - Menu item re-enabled after shield deactivates
  - Clean state management with proper cleanup (event tap, sleep assertion, window)
- [x] **Issue #18**: Extend Config for New Settings
  - Added `default_timer: Option<String>` for persistent timer duration
  - Added `overlay_opacity: Option<f64>` for configurable opacity (0.2-0.8)
  - Added `Serialize` derive to Config for saving to file
  - Added `Config::save()` method to persist settings to `~/.config/catshield/config.toml`
  - Added `Config::opacity()` helper with clamping to valid range
  - Creates config directory if it doesn't exist
  - Default opacity is 0.5 (50%) when not specified
- [x] **Issue #16**: Create Settings Window
  - Settings window opens centered on screen from menu bar Settings... item (Cmd+,)
  - Exit Key field with real-time validation using `ExitKey::parse()`
  - Default Timer section with enable checkbox and duration field
  - Timer duration field accepts formats: 30m, 2h, 1h30m, 90s
  - Timer field disabled when checkbox unchecked
  - Overlay Opacity slider from 20% to 80% with live percentage display
  - Save button validates all fields and persists to config file
  - Cancel button (Escape) discards changes and closes window
  - Only one settings window can be open at a time
  - Window properly manages focus (text fields work)
  - Validation feedback with green checkmark or red error messages
- [x] **Issue #24**: Fix multiple menu bar icons bug (single-instance enforcement)
  - Prevents launching multiple Cat Shield instances simultaneously
  - Uses PID-based lock file at `~/.config/catshield/catshield.lock`
  - Checks if existing process is still running before acquiring lock
  - Cleans up stale lock files from crashed instances
  - Shows friendly error message if already running with existing PID
  - Lock is released on normal application exit
- [x] **Issue #19**: Add About Panel
  - "About Cat Shield" menu item added between "Settings..." and Help submenu
  - About panel displays:
    - Large cat emoji (🐱)
    - "Cat Shield" app name in bold
    - Version number (from Cargo.toml via `env!("CARGO_PKG_VERSION")`)
    - Brief description: "Protect your work from curious cats and keyboard-walking pets."
  - Close button dismisses the panel (Return key also works)
  - Only one About panel can be open at a time (clicking menu again brings existing to front)
  - Panel is centered on screen

## Open Issues

### Phase 4: Menu-Based Application Interface

**Epic #13**: Transform Cat Shield from an immediate-launch utility into a menu bar application.

| Priority | Issue | Title | Dependencies | Effort |
|----------|-------|-------|--------------|--------|
| ✅ Done | #14 | Create Menu Bar Infrastructure (NSStatusItem) | None | ~1 day |
| ✅ Done | #15 | Create Main Dropdown Menu | ✅ #14 | ~1 day |
| ✅ Done | #17 | Refactor Overlay to On-Demand Activation | ✅ #14 | ~2 days |
| ✅ Done | #18 | Extend Config for New Settings | None | ~1 day |
| ✅ Done | #16 | Create Settings Window | ✅ #15, ✅ #18 | ~3 days |
| ✅ Done | #19 | Add About Panel | ✅ #15 | ~0.5 day |
| ✅ Done | #42 | Enable Help Menu Links (Documentation & Report Issue) | ✅ #15 | ~0.5 day |

**Implementation Order:**
```text
#14: Menu Bar Infrastructure ✅
    ├── #17: Refactor Overlay ✅ (parallel with #15)
    └── #15: Dropdown Menu ✅
            ├── #18: Extend Config ✅ (parallel)
            ├── #42: Help Menu Links ✅
            └── #16: Settings Window ✅
                    └── #19: About Panel ✅
```

### Code Quality & Architecture
- [x] **Issue #35**: Split main.rs into multiple modules
  - Extracted `src/input/` module with keycodes and exit key handling
  - Extracted `src/timer/` module with duration parsing, formatting, and state
  - Extracted `src/config/` module with Config struct, file I/O, and CLI args
  - Extracted `src/platform/` module with FFI bindings, accessibility, power, and event tap
  - Extracted `src/ui/` module with views, windows, menu bar, and shield activation
  - Extracted `src/lock/` module with single-instance enforcement
  - Created `src/lib.rs` with public API re-exports
  - Reduced main.rs from ~4354 lines to ~530 lines (88% reduction)
  - All 50 tests pass, clippy clean, build successful
- [x] **Issue #44**: Refactoring - Improve code safety, reduce duplication, and consolidate global state
  - Created `src/shield_core.rs` with shared shield activation logic
  - Extracted `ensure_accessibility()`, `create_shield_window()`, `setup_close_button()` functions
  - Removed ~150 lines of duplicated code between `main.rs` and `ui/shield.rs`
  - Consolidated 25+ global `AtomicPtr` variables into structured modules:
    - `shield::` - Shield window, close button, timer display state
    - `menu_bar::` - Menu bar items and action handlers
    - `settings::` - Settings window UI elements
    - `about::` - About panel state
  - Centralized UI constants into submodules: `close_button::`, `timer_display::`, `animation::`, `window_level::`
  - Config opacity now actually applied to shield overlay background
  - Fixed mutex poison handling to recover gracefully instead of panicking
  - Legacy aliases maintained for backwards compatibility
  - All 52 tests pass, clippy clean, build successful

### Settings Window Improvements
- [x] **Issue #46**: Add 'Reset to Default' button and improve spacing in Settings window
  - Added "Reset to Default" button on the left side of the button row
  - When clicked, resets all fields to defaults: Exit Key (Cmd+Q), Timer (disabled/cleared), Opacity (50%)
  - Reset does NOT auto-save - user must click Save to persist changes
  - Increased window height from 340 to 370 pixels for better visual spacing
  - Provides easy way to restore factory settings without manually editing each field
- [x] **Issue #48**: Exit Key validation should not show on initial Settings window open
  - Removed `validate_exit_key_realtime()` call that ran on window open
  - Validation label now starts empty when Settings window opens
  - Validation only appears after user modifies the Exit Key field
  - Real-time validation still works via `controlTextDidChange:` delegate method
- [x] **Issue #49**: Add real-time validation for Timer duration field in Settings
  - Timer field now validates input as user types (when checkbox is enabled)
  - Created `TimerFieldDelegate` class implementing `NSControlTextEditingDelegate`
  - Created `validate_timer_realtime()` function for number validation
  - Valid input shows "✓ Valid" (green), invalid shows error (red)
  - "Must be greater than 0" for zero values, "Enter a number" for non-numeric input
  - No validation when field is empty (let Save handle "Duration required")
  - Validation cleared when checkbox is unchecked (disabled state)
  - No validation message on initial window open (matches Issue #48 pattern)

### Phase 5: Performance & Code Quality Improvements

**Summary**: Identified opportunities to improve performance, code maintainability, and test coverage.

#### Performance Optimizations

| Priority | Issue | Title | Effort |
|----------|-------|-------|--------|
| ✅ Done | #52 | Optimize atomic orderings from SeqCst to appropriate weaker orderings | ~1 day |
| ✅ Done | #53 | Cache redundant atomic loads in timer callback | ~2 hours |
| ✅ Done | #54 | Cache formatted duration string in timer display | ~2-3 hours |

#### Code Maintenance

| Priority | Issue | Title | Effort |
|----------|-------|-------|--------|
| ✅ Done | #55 | Refactor: Break show_settings_window() into smaller functions | ~4-6 hours |
| ✅ Done | #56 | Refactor: Create pointer helper to reduce null-check duplication | ~4-6 hours |
| ✅ Done | #57 | Docs: Add safety documentation to std::mem::forget and unsafe blocks | ~4-6 hours |

#### Testing Improvements

| Priority | Issue | Title | Effort |
|----------|-------|-------|--------|
| ✅ Done | #58 | Test: Add unit tests for lock module (single-instance enforcement) | ~4-6 hours |
| ✅ Done | #59 | Test: Add unit tests for timer state module | ~3-4 hours |
| ✅ Done | #60 | Test: Expand UI state and settings validation tests | ~2-3 hours |
| ✅ Done | #75 | Test: Add integration tests for menu bar timer functionality | ~4-6 hours |

**Implementation Order:**
```text
Performance (can be done in parallel):
    #52: Atomic Ordering Optimization ✅
    #53: Timer Callback Cache ✅
    #54: Duration Format Cache ✅

Code Quality (can be done in parallel):
    #55: Settings Window Refactor ✅
    #56: Pointer Helper ✅
    #57: Safety Documentation ✅

Testing (can be done in parallel):
    #58: Lock Module Tests ✅
    #59: Timer State Tests ✅
    #60: UI State & Validation Tests ✅
    #75: Menu Bar Timer Integration Tests ✅
```

### Phase 6: Enhanced Input Control

**Summary**: Allow users more granular control over which keys are blocked during shield activation.

| Priority | Issue | Title | Dependencies | Effort |
|----------|-------|-------|--------------|--------|
| ✅ Done | #64 | Add configurable key allowlist to pass specific keys through the shield | None | ~2-3 days |
| ✅ Done | #65 | Add preset groups for key allowlist (Media Keys, System Shortcuts) | ✅ #64 | ~0.5 day |

**Implementation Order:**
```text
#64: Key Allowlist Feature ✅
    └── #65: Preset Groups ✅
```

- [x] **Issue #64**: Add configurable key allowlist to pass specific keys through the shield
  - Added `allowed_keys` field to Config struct for persistent storage
  - Created `src/input/allowed_keys.rs` module for key management
  - Supports same format as exit keys (e.g., "Cmd+Space", "F11", "Ctrl+Option+A")
  - Also supports simple keys without modifiers (e.g., "F11", "Space")
  - Modified event tap callback to check allowed keys before blocking
  - Added "Allowed Keys" section to Settings window with:
    - Scrollable list view showing current allowed keys
    - Input field with real-time validation
    - "Add" and "Clear All" buttons for list management
    - Duplicate detection
  - Keys loaded from config on shield activation
  - Keys cleared on shield deactivation
  - Added 19 comprehensive tests (6 config + 13 allowed_keys module)
  - Enables media controls, Spotlight, Mission Control, and custom shortcuts while shield is active

### Phase 7: Settings Window Polish - COMPLETE ✅

**Summary**: Bug fixes and UX improvements for the Settings window.

| Priority | Issue | Title | Dependencies | Effort |
|----------|-------|-------|--------------|--------|
| ✅ Done | #87 | Settings menu item stays disabled after closing Settings window | None | ~0.5 day |
| ✅ Done | #91 | Action feedback clear called from background thread violates AppKit threading | None | ~0.5 day |
| ✅ Done | #86 | Show real-time validation error when entering duplicate allowed key | None | ~0.5 day |
| ✅ Done | #85 | Add Undo button to Settings window for reverting individual changes | None | ~1 day |

### Phase 8: Cross-Platform Support

**Summary**: Extend Cat Shield to support Windows and Linux in addition to macOS.

#### Phase 8.1: Platform Abstraction Foundation

| Priority | Issue | Title | Dependencies | Effort |
|----------|-------|-------|--------------|--------|
| ✅ Done | #94 | Create platform abstraction traits | None | ~2 days |
| ✅ Done | #95 | Reorganize macOS platform code into subdirectory | ✅ #94 | ~1 day |
| ✅ Done | #96 | Implement platform traits for macOS | ✅ #94, #95 | ~2 days |
| ✅ Done | #97 | Create canonical Key enum and split keycodes by platform | ✅ #94 | ~1 day |
| ✅ Done | #98 | Update shield_core.rs to use platform traits | ✅ #96 | ~1-2 days |
| ✅ Done | #99 | Update Cargo.toml for conditional platform dependencies | ✅ #94 | ~0.5 day |

**Implementation Order:**
```text
#94: Platform Abstraction Traits (foundation) ✅
    ├── #95: Reorganize macOS Code ✅
    │       └── #96: Implement macOS Traits ✅
    │               └── #98: Update shield_core.rs ✅
    └── #97: Canonical Key Enum ✅
            └── #99: Conditional Dependencies ✅
```

#### Phase 8.2: Windows Support

| Priority | Issue | Title | Dependencies | Effort |
|----------|-------|-------|--------------|--------|
| ✅ Done | #100 | Add Windows keyboard hook implementation (InputBlocker) | #94, #96 | ~2-3 days |
| ✅ Done | #101 | Add Windows power management (PowerManager) | #94 | ~1 day |
| 🟢 Medium | #102 | Add Windows system tray implementation | #94, #96 | ~2 days |
| 🟢 Medium | #103 | Add Windows overlay window implementation | #94, #96 | ~2 days |
| ✅ Done | #104 | Add Windows keycode mappings | #97 | ~1 day |
| 🟢 Medium | #105 | Create Windows entry point and event loop integration | #100, #102, #103 | ~1-2 days |

**Implementation Order:**
```text
#100: Windows Keyboard Hook ✅ ─┬─ #105: Windows Entry Point
#101: Windows Power Mgmt ✅ ────┤
#102: Windows System Tray ──────┤
#103: Windows Overlay ──────────┘
#104: Windows Keycodes ✅ (parallel with #97)
```

#### Phase 8.3: Linux Support

| Priority | Issue | Title | Dependencies | Effort |
|----------|-------|-------|--------------|--------|
| 🟡 High | #106 | Add X11 keyboard grab implementation (InputBlocker) | #94, #96 | ~2-3 days |
| 🟡 High | #107 | Research Wayland input blocking solutions | None | ~1-2 days |
| 🟢 Medium | #108 | Add Linux power management via DBus | #94 | ~1-2 days |
| 🟢 Medium | #109 | Add Linux system tray via libappindicator | #94, #96 | ~2 days |
| 🟢 Medium | #110 | Add Linux overlay window (X11 + Wayland) | #94, #96, #107 | ~2-3 days |
| 🟢 Medium | #111 | Add Linux keycode mappings | #97 | ~1 day |

**Implementation Order:**
```text
#107: Research Wayland ──┬─ #110: Linux Overlay
#106: X11 Keyboard Grab ─┤
#108: Linux Power Mgmt ──┤
#109: Linux System Tray ─┘
#111: Linux Keycodes (parallel with #97)
```

#### Phase 8.4: Cross-Platform CI & Testing

| Priority | Issue | Title | Dependencies | Effort |
|----------|-------|-------|--------------|--------|
| ✅ Done | #112 | Set up cross-platform GitHub Actions CI | #99 | ~1 day |
| 🟢 Medium | #113 | Add platform-specific integration tests | #112 | ~2 days |
| ✅ Done | #120 | Add Docker containers for cross-platform local development | None | ~0.5 day |

### Phase 9: Feature Enhancements

| Priority | Issue | Title | Dependencies | Effort |
|----------|-------|-------|--------------|--------|
| 🔵 Low | #114 | Add optional animated cat companion to shield overlay | None | ~2-3 days |

### Recently Completed

| Issue | Title | Completed |
|-------|-------|-----------|
| #101 | feat: Add Windows power management (PowerManager) | 2026-01-14 |
| #100 | feat: Add Windows keyboard hook implementation (InputBlocker) | 2026-01-14 |
| #104 | feat: Add Windows keycode mappings | 2026-01-14 |
| #112 | feat: Set up cross-platform GitHub Actions CI | 2026-01-14 |
| #98 | feat: Update shield_core.rs to use platform traits | 2026-01-14 |
| #120 | feat: Add Docker containers for cross-platform local development | 2026-01-14 |
| #99 | feat: Update Cargo.toml for conditional platform dependencies | 2026-01-13 |
| #97 | feat: Create canonical Key enum and split keycodes by platform | 2026-01-13 |
| #96 | feat: Implement platform traits for macOS | 2026-01-12 |
| #95 | feat: Reorganize macOS platform code into subdirectory | 2026-01-12 |
| #94 | feat: Create platform abstraction traits | 2026-01-12 |
| #85 | feat: Add Undo button to Settings window for reverting individual changes | 2026-01-12 |
| #91 | fix: Action feedback clear called from background thread violates AppKit threading | 2026-01-12 |
| #86 | feat: Show real-time validation error when entering duplicate allowed key | 2026-01-12 |
| #87 | fix: Settings menu item stays disabled after closing Settings window | 2026-01-12 |
| #83 | feat: Add ability to select and remove individual allowed keys in Settings | 2026-01-11 |
| #80 | fix: Settings Cancel button does not discard allowed_keys changes | 2026-01-11 |
| #65 | feat: Add preset groups for key allowlist (Media Keys, System Shortcuts) | 2026-01-11 |
| #64 | feat: Add configurable key allowlist to pass keys through shield | 2026-01-10 |
| #54 | perf: Cache formatted duration string in timer display | 2026-01-10 |
| #60 | test: Expand UI state and settings validation tests | 2026-01-10 |
| #75 | test: Add integration tests for menu bar timer functionality | 2026-01-10 |
| #73 | feat: Reduce minimum auto-exit timer to 5 seconds | 2026-01-10 |
| #53 | perf: Cache redundant atomic loads in timer callback | 2026-01-10 |
| #57 | Docs: Add safety documentation to std::mem::forget and unsafe blocks | 2026-01-10 |
| #56 | Refactor: Create pointer helper to reduce null-check duplication | 2026-01-10 |
| #52 | Optimize atomic orderings from SeqCst to appropriate weaker orderings | 2026-01-10 |
| #55 | Refactor: Break show_settings_window() into smaller functions | 2026-01-10 |
| #59 | Add unit tests for timer state module | 2026-01-09 |
| #58 | Add unit tests for lock module (single-instance enforcement) | 2026-01-09 |
| #49 | Add real-time validation for Timer duration field in Settings | 2026-01-09 |
| #48 | Exit Key validation should not show on initial Settings window open | 2026-01-09 |
| #46 | Add 'Reset to Default' button and improve spacing in Settings window | 2026-01-09 |
| #44 | Refactoring - Improve code safety, reduce duplication, consolidate state | 2026-01-09 |
| #42 | Enable Help menu links (Documentation & Report Issue) | 2026-01-09 |
| #11 | Add install script for easy CLI access | 2026-01-08 |
| #38 | Menu items not visually disabled when windows are open | 2026-01-08 |
| #30 | Add real-time validation for Exit Key field in Settings Window | 2026-01-08 |

## Issue Summary

| Status | Count | Issues |
|--------|-------|--------|
| Open | 11 | #102, #103, #105, #106, #107, #108, #109, #110, #111, #113, #114 |
| Closed | 55 | #3, #5, #6, #7, #10, #11, #13, #14, #15, #16, #17, #18, #19, #24, #25, #28, #30, #31, #35, #38, #42, #44, #46, #48, #49, #52, #53, #54, #55, #56, #57, #58, #59, #60, #64, #65, #73, #75, #80, #83, #85, #86, #87, #91, #94, #95, #96, #97, #98, #99, #100, #101, #104, #112, #120 |

### By Priority
- 🔴 Critical: 0
- 🟡 High: 2 (#106, #107)
- 🟢 Medium: 8 (#102, #103, #105, #108, #109, #110, #111, #113)
- 🔵 Low: 1 (#114)

## Recommended Implementation Order

### Previous Sprint: Phase 4 Foundation - COMPLETE ✅
1. ~~**#14** - Menu Bar Infrastructure~~ ✅
2. ~~**#15** - Main Dropdown Menu~~ ✅
3. ~~**#17** - Refactor Overlay~~ ✅
4. ~~**#18** - Extend Config~~ ✅
5. ~~**#16** - Settings Window~~ ✅
6. ~~**#24** - Fix multiple menu bar icons bug~~ ✅
7. ~~**#19** - About Panel~~ ✅
8. ~~**#30** - Add real-time Exit Key validation in Settings~~ ✅
9. ~~**#11** - Install Script~~ ✅

### Current Sprint: Phase 5 - Performance & Quality

**Completed:**
1. ~~**#58** - Add unit tests for lock module (security-critical, 0 tests currently)~~ ✅
2. ~~**#59** - Add unit tests for timer state module (core functionality, 0 tests)~~ ✅
3. ~~**#55** - Refactor show_settings_window() into smaller functions (600+ lines)~~ ✅
4. ~~**#52** - Optimize atomic orderings (158 SeqCst → weaker orderings)~~ ✅
5. ~~**#56** - Create pointer helper (reduces 50+ duplicated patterns)~~ ✅
6. ~~**#57** - Add safety documentation (35+ std::mem::forget and unsafe blocks)~~ ✅
7. ~~**#53** - Cache timer callback atomic loads~~ ✅

**Completed:**
8. ~~**#60** - Expand UI state and validation tests~~ ✅
9. ~~**#54** - Cache formatted duration string~~ ✅

### Phase 6: Enhanced Input Control - COMPLETE ✅

**Completed:**
10. ~~**#64** - Add configurable key allowlist (new feature, medium priority)~~ ✅
11. ~~**#65** - Add preset groups for key allowlist (depends on #64, low priority)~~ ✅

### Current Sprint: Phase 8 - Cross-Platform Support

**Completed:**
1. ~~**#94** - Create platform abstraction traits (critical foundation)~~ ✅
2. ~~**#95** - Reorganize macOS platform code into subdirectory (depends on #94 ✅)~~ ✅
3. ~~**#96** - Implement platform traits for macOS (depends on #94 ✅, #95 ✅)~~ ✅
4. ~~**#97** - Create canonical Key enum and split keycodes by platform (depends on #94 ✅)~~ ✅
5. ~~**#99** - Update Cargo.toml for conditional platform dependencies (depends on #94 ✅)~~ ✅

**Completed (Platform Abstraction Foundation):**
6. ~~**#98** - Update shield_core.rs to use platform traits (depends on #96 ✅)~~ ✅

**Parallel Work (can start after foundation):**
- Windows team: #100, #101, #102, #103, #104, #105
- Linux team: #106, #107, #108, #109, #110, #111

**Final Integration:**
- ~~**#112** - Set up cross-platform GitHub Actions CI (depends on #99 ✅)~~ ✅
- **#113** - Add platform-specific integration tests
- ~~**#120** - Add Docker containers for cross-platform local development~~ ✅

**Future Feature:**
- **#114** - Add optional animated cat companion to shield overlay (low priority, independent)

## Critical Path

```
Phase 4 (COMPLETE):
Foundation:     #14 (Menu Bar) ✅ ──┬── #17 (Refactor Overlay) ✅
                                    │
                                    └── #15 (Dropdown Menu) ✅ ── #16 (Settings) ✅ ─┬─ #19 (About) ✅
                                                 │                                    │
Parallel:       #18 (Config) ✅ ────────────────┘                                    └─ #30 (Exit Key Validation) ✅

Phase 5 (COMPLETE):
Testing:        #58 (Lock Tests) ✅ ──┬─── All Complete
                #59 (Timer Tests) ✅ ─┤
                #60 (UI State Tests) ✅ ┘

Performance:    #52 (Atomic Ordering) ✅ ─┬── All Complete
                #53 (Timer Cache) ✅ ─────┤
                #54 (Duration Cache) ✅ ──┘

Maintenance:    #55 (Settings Refactor) ✅ ─┬── All Complete
                #56 (Pointer Helper) ✅ ────┤
                #57 (Safety Docs) ✅ ────────┘

Phase 6 (COMPLETE):
Input Control:  #64 (Key Allowlist) ✅ ─── #65 (Preset Groups) ✅

Phase 7 (COMPLETE):
Settings Polish: #87 (Menu Bug) ✅ ─── #91 (Threading Fix) ✅ ─── #86 (Duplicate Validation) ✅ ─── #85 (Undo Button) ✅

Phase 8 (CURRENT):
Foundation:     #94 (Platform Traits) ✅ ─┬── #95 (Reorganize macOS) ✅ ── #96 (macOS Traits) ✅ ── #98 (Update shield_core) ✅
                                          │
                                          └── #97 (Key Enum) ✅ ── #99 (Conditional Deps)

Windows:        #100 (Keyboard Hook) ✅ ─┬── #105 (Entry Point)
                #101 (Power Mgmt) ✅ ────┤
                #102 (System Tray) ──────┤
                #103 (Overlay) ──────────┘
                #104 (Keycodes) ✅ ──────── (parallel with #97)

Linux:          #107 (Wayland Research) ─┬── #110 (Overlay)
                #106 (X11 Keyboard) ─────┤
                #108 (Power via DBus) ───┤
                #109 (System Tray) ──────┘
                #111 (Keycodes) ────────── (parallel with #97)

CI:             #112 (Cross-Platform CI) ✅ ── #113 (Platform Tests)

Phase 9 (FUTURE):
Enhancement:    #114 (Animated Cat Companion)
```

## Future Considerations

Potential future enhancements (not yet tracked as issues):

- Multi-monitor support improvements
- Auto-start on login option
- Activity logging
- Sound effects/feedback
- Custom overlay themes

## Changelog

### 2026-01-14
- Completed Issue #101: Add Windows power management (PowerManager)
  - Created `src/platform/windows/power.rs` with `WindowsPowerManager` implementation
  - Implements the `PowerManager` trait for Windows using `SetThreadExecutionState` API
  - `prevent_sleep()` sets `ES_CONTINUOUS | ES_DISPLAY_REQUIRED` to keep display awake
  - `allow_sleep()` clears execution state with `ES_CONTINUOUS` to resume normal sleep behavior
  - Uses atomic counters for assertion tracking across multiple calls
  - Thread-safe implementation with `Send + Sync` traits
  - Proper cleanup: sleep prevention only released when all assertions are released
  - Added unit tests for manager creation and trait verification
  - Updated `src/platform/windows/mod.rs` to export `WindowsPowerManager`
  - Updated `src/platform/mod.rs` to re-export from Windows module
  - Updated issue counts: 11 open, 55 closed
  - Updated priority counts: 0 Critical, 2 High, 8 Medium, 1 Low

- Completed Issue #100: Add Windows keyboard hook implementation (InputBlocker)
  - Created `src/platform/windows/` directory with Windows-specific implementations
  - Implemented `WindowsInputBlocker` struct implementing the `InputBlocker` trait
  - Uses `SetWindowsHookExW` with `WH_KEYBOARD_LL` for low-level keyboard hook
  - Hook callback intercepts and blocks keyboard events system-wide
  - Support for exit key detection with configurable key combinations
  - Support for allowed keys list to pass specific keys through the shield
  - Uses `GetAsyncKeyState` to check modifier key state (Ctrl, Alt, Shift, Win)
  - Thread-safe implementation using atomic operations for shared state
  - Added `set_exit_key_config()` function to configure exit key combination
  - Added `set_allowed_keys()` and `clear_allowed_keys()` for key allowlist
  - Added `AllowedKeyConfig` struct for configuring allowed key combinations
  - Proper cleanup via `UnhookWindowsHookEx` when input blocking is disabled
  - Added unit tests for configuration and state management
  - Added `windows` crate v0.58 dependency with required features:
    - `Win32_Foundation`, `Win32_UI_WindowsAndMessaging`
    - `Win32_System_Threading`, `Win32_System_LibraryLoader`
  - Updated `src/platform/mod.rs` to conditionally include Windows module

- Completed Issue #104: Add Windows keycode mappings
  - Fully implemented `src/input/keycodes/windows.rs` (was previously a stub)
  - Added `key_to_keycode()` function: converts `Key` enum to Windows virtual key codes
  - Added `keycode_to_key()` function: converts Windows virtual key codes to `Key` enum
  - Complete mappings for all 57 supported keys:
    - Letters A-Z (VK_A through VK_Z: 0x41-0x5A)
    - Numbers 0-9 (VK_0 through VK_9: 0x30-0x39)
    - Function keys F1-F12 (VK_F1 through VK_F12: 0x70-0x7B)
    - Special keys: Escape, Tab, Space, Return, Delete (Backspace)
    - Navigation keys: Arrow keys, Home, End, PageUp, PageDown
    - Punctuation: Minus, Equal, Brackets, Backslash, Semicolon, Quote, etc.
  - Uses OEM key codes for punctuation (may vary by keyboard layout)
  - Added 15 comprehensive unit tests for key mappings
  - Tests verify roundtrip conversion and complete coverage
  - Updated issue counts: 12 open, 54 closed
  - Updated priority counts: 0 Critical, 3 High, 8 Medium, 1 Low

- Completed Issue #112: Set up cross-platform GitHub Actions CI
  - Updated `.github/workflows/ci.yml` to use matrix strategy for all three platforms
  - Build job now runs on `macos-latest`, `windows-latest`, and `ubuntu-latest`
  - Test job now runs on all three platforms in parallel
  - Lint & Format job runs once on Ubuntu (not platform-specific)
  - Clippy runs per-platform (in build job) due to cfg-specific code
  - Uses `Swatinem/rust-cache@v2` with platform-specific cache keys for faster builds
  - Uses `dtolnay/rust-toolchain@stable` for reliable Rust toolchain installation
  - `fail-fast: false` ensures all platforms run even if one fails
  - Timeout increased to 20 minutes for cross-platform builds
  - Uses `actions/checkout@v4` (latest stable)
  - Updated issue counts: 14 open, 52 closed
  - Updated priority counts: 0 Critical, 4 High, 9 Medium, 1 Low

- Completed Issue #98: Update shield_core.rs to use platform traits
  - Refactored `ensure_accessibility()` to use the `PermissionChecker` trait
  - Created new `ensure_permissions<P: PermissionChecker>()` generic function
  - Added platform-specific `poll_for_permissions()` implementations:
    - macOS: Uses `CFRunLoopRunInMode` for responsive UI during polling
    - Other platforms: Uses `thread::sleep` for simple polling
  - Added `#[cfg(target_os = "macos")]` guards to macOS-specific functions:
    - `create_shield_window()`, `setup_close_button()`, `setup_timer_display()`
  - Maintained backward compatibility via `ensure_accessibility()` wrapper
  - Added 6 new unit tests for permission checking trait usage
  - All 305 tests pass, clippy clean, build successful
  - Updated issue counts: 15 open, 51 closed
  - Phase 8.1 Platform Abstraction Foundation is now complete ✅

- Completed Issue #120: Add Docker containers for cross-platform local development
  - Created `docker/` directory with Linux and Windows build environments
  - Added docker-compose.yml with services for testing, linting, and building
  - Added comprehensive documentation in docker/README.md

### 2026-01-13
- Completed Issue #97: Create canonical Key enum and split keycodes by platform
  - Created `src/input/keycodes/` directory structure for platform-specific keycode mappings
  - Created `src/input/keycodes/mod.rs` with:
    - `Key` enum: Platform-agnostic representation of all supported keys (letters A-Z, numbers 0-9, F1-F12, special keys, navigation keys, punctuation)
    - `key_from_name()`: Parse key names to `Key` enum (case-insensitive, supports aliases like "esc"/"escape")
    - `key_to_name()`: Convert `Key` to display name
    - Helper methods: `is_letter()`, `is_number()`, `is_function_key()`, `is_navigation()`, `all()`
    - Legacy compatibility functions: `keycode_from_name()`, `keycode_to_name()` for backward compatibility
  - Created `src/input/keycodes/macos.rs` with:
    - `key_to_keycode()`: Convert `Key` to macOS virtual keycode (CGKeyCode)
    - `keycode_to_key()`: Convert macOS virtual keycode to `Key`
    - Full bidirectional mapping for all 69 supported keys
  - Created `src/input/keycodes/windows.rs` (stub) for future Windows support
  - Created `src/input/keycodes/linux.rs` (stub) for future Linux support
  - Removed old `src/input/keycodes.rs` (functionality moved to new module)
  - Updated `src/input/mod.rs` to re-export new keycodes module
  - Added 27 new unit tests for Key enum and macOS mappings
  - All 299 tests pass, clippy clean, build successful
  - Updated issue counts: 17 open, 48 closed
  - Updated priority counts: 0 Critical, 4 High, 12 Medium, 1 Low

### 2026-01-12
- Completed Issue #96: Implement platform traits for macOS
  - Created `src/platform/macos/impls.rs` with trait implementations:
    - `MacOSInputBlocker`: Implements `InputBlocker` trait wrapping `event_tap.rs`
      - Uses `setup_event_tap()` and `disable_event_tap()` for input blocking
      - Tracks active state internally and via `EVENT_TAP` atomic pointer
      - Implements `Send + Sync` for thread-safe access
    - `MacOSPowerManager`: Implements `PowerManager` trait wrapping `power.rs`
      - Uses `prevent_sleep()` and `allow_sleep()` for IOKit power assertions
      - Converts between `u32` assertion IDs and `SleepAssertion` type
      - Validates assertion IDs don't exceed `u32::MAX`
    - `MacOSPermissionChecker`: Implements `PermissionChecker` trait wrapping `accessibility.rs`
      - Uses `check_accessibility()`, `check_accessibility_with_prompt()`, `open_accessibility_settings()`
      - Provides clean error handling via `PermissionError`
    - `MacOSPlatform`: Combined struct providing access to all implementations
  - Updated `src/platform/macos/mod.rs` to export new implementations
  - Updated `src/platform/mod.rs` to re-export macOS trait implementations
  - Added 12 unit tests for trait implementations
  - All 272 tests pass, clippy clean, build successful
  - Updated issue counts: 18 open, 47 closed
  - Updated priority counts: 0 Critical, 5 High, 12 Medium, 1 Low

- Completed Issue #95: Reorganize macOS platform code into subdirectory
  - Created `src/platform/macos/` directory for macOS-specific implementations
  - Moved `event_tap.rs` → `macos/event_tap.rs` (input blocking via CGEventTap)
  - Moved `power.rs` → `macos/power.rs` (sleep prevention via IOKit)
  - Moved `accessibility.rs` → `macos/accessibility.rs` (permission handling)
  - Moved `bindings.rs` → `macos/bindings.rs` (FFI declarations for macOS frameworks)
  - Created `macos/mod.rs` with re-exports for all public items
  - Updated `src/platform/mod.rs` with conditional compilation (`#[cfg(target_os = "macos")]`)
  - Re-exports maintain backward compatibility - no changes to public API
  - All 261 tests pass, clippy clean, build successful
  - Updated issue counts: 19 open, 46 closed
  - Updated priority counts: 0 Critical, 6 High, 12 Medium, 1 Low

- Completed Issue #94: Create platform abstraction traits
  - Created `src/platform/types.rs` with platform-agnostic types:
    - `Modifiers`: Represents keyboard modifier keys (command, option, control, shift)
    - `KeyEvent`: Represents keyboard events with keycode, modifiers, and key state
    - `SleepAssertion`: Opaque handle for sleep prevention assertions
    - `Rect`: Rectangle type for window positioning and sizing
  - Created `src/platform/errors.rs` with error types for each platform component:
    - `InputBlockError`: For input blocking operations
    - `PowerError`: For power management operations
    - `TrayError`: For system tray operations
    - `WindowError`: For overlay window operations
    - `PermissionError`: For permission checking operations
  - Created `src/platform/traits.rs` with platform abstraction traits:
    - `InputBlocker`: For intercepting and blocking keyboard input
    - `PowerManager`: For preventing system sleep during protection
    - `PermissionChecker`: For handling platform permission requirements
    - `SystemTray`: For managing the menu bar / system tray icon
    - `OverlayWindow`: For creating and managing the fullscreen overlay
  - Updated `src/platform/mod.rs` to export new modules and types
  - All traits include comprehensive documentation with platform-specific notes
  - Added 27 new unit tests for types, errors, and trait mock implementations
  - All 259 tests pass, clippy clean, build successful
  - Updated issue counts: 20 open, 45 closed
  - Updated priority counts: 0 Critical, 7 High, 12 Medium, 1 Low

### 2026-01-12 (Roadmap Update)
- Added Phase 8: Cross-Platform Support with 20 new issues organized into sub-phases:
  - Phase 8.1: Platform Abstraction Foundation (#94-#99) - 6 issues
    - #94: Create platform abstraction traits (critical foundation)
    - #95: Reorganize macOS platform code into subdirectory
    - #96: Implement platform traits for macOS
    - #97: Create canonical Key enum and split keycodes by platform
    - #98: Update shield_core.rs to use platform traits
    - #99: Update Cargo.toml for conditional platform dependencies
  - Phase 8.2: Windows Support (#100-#105) - 6 issues
    - #100: Add Windows keyboard hook implementation (InputBlocker)
    - #101: Add Windows power management (PowerManager)
    - #102: Add Windows system tray implementation
    - #103: Add Windows overlay window implementation
    - #104: Add Windows keycode mappings
    - #105: Create Windows entry point and event loop integration
  - Phase 8.3: Linux Support (#106-#111) - 6 issues
    - #106: Add X11 keyboard grab implementation (InputBlocker)
    - #107: Research Wayland input blocking solutions
    - #108: Add Linux power management via DBus
    - #109: Add Linux system tray via libappindicator
    - #110: Add Linux overlay window (X11 + Wayland)
    - #111: Add Linux keycode mappings
  - Phase 8.4: Cross-Platform CI & Testing (#112-#113) - 2 issues
    - #112: Set up cross-platform GitHub Actions CI
    - #113: Add platform-specific integration tests
- Added Phase 9: Feature Enhancements
  - #114: Add optional animated cat companion to shield overlay (low priority)
- Updated issue counts: 21 open, 44 closed
- Updated priority counts: 1 Critical, 7 High, 12 Medium, 1 Low
- Updated critical path diagram for Phase 8 cross-platform work
- Marked Phase 7: Settings Window Polish as COMPLETE

### 2026-01-12
- Completed Issue #85: Add Undo button to Settings window for reverting individual changes
  - Added `SettingsChange` enum in `src/ui/state.rs` with variants for all undoable operations:
    - `ExitKey`, `TimerEnabled`, `TimerValue`, `TimerUnit`, `Opacity`
    - `AllowedKeyAdded`, `AllowedKeyRemoved`, `AllowedKeysCleared`
    - `AllowedKeysPresetAdded`, `ResetToDefaults`
  - Added undo stack with `push_undo()`, `pop_undo()`, `has_undo()`, `clear_undo_stack()` functions
  - Added change tracking for opacity slider and timer unit dropdown (captures initial value)
  - Extended text field delegates with `controlTextDidBeginEditing` and `controlTextDidEndEditing`
    to track field changes (Exit Key, Timer Value)
  - Added "Undo" button to Settings window (left of Reset to Default button)
  - Button is disabled when undo stack is empty, enabled when changes exist
  - Undo reverses the most recent change and restores UI to previous state
  - Undo stack cleared on Save, Cancel, or window close
  - Added 15 new unit tests for undo stack and change tracking functionality
  - All 232 tests pass, clippy clean, build successful
  - Updated issue counts: 0 open, 44 closed
  - Phase 7: Settings Window Polish is now complete

- Completed Issue #91: Fix action feedback clear called from background thread (AppKit threading violation)
  - Root cause: `clear_action_feedback()` was called directly from a background thread spawned by `show_action_feedback()`
  - AppKit requires all UI operations to happen on the main thread
  - Fix: Added `dispatch2` crate dependency and wrapped the `clear_action_feedback()` call in `dispatch2::run_on_main()`
  - The closure is now dispatched to the main queue, ensuring thread-safe UI updates
  - Prevents undefined behavior, crashes, and UI glitches like the Settings menu item staying disabled
  - Updated issue counts: 1 open, 43 closed

- Completed Issue #86: Show real-time validation error when entering duplicate allowed key
  - Modified `validate_add_key_realtime()` in `src/ui/windows/settings.rs` to check for duplicates
  - When a valid key is entered, checks against pending allowed keys list (case-insensitive)
  - Shows "Key already in list" error (red) and keeps Add button disabled for duplicates
  - Added `revalidate_add_key_field()` function to re-validate after list changes
  - List-modifying functions now call `revalidate_add_key_field()`:
    - `remove_selected_allowed_key()`: removed key may now be valid to add
    - `clear_all_allowed_keys()`: all inputs become valid
    - `add_preset()`: input may now be a duplicate
    - `reset_settings_to_defaults()`: clears list and input field
  - Updated issue counts: 1 open, 42 closed
  - Updated priority counts: 0 Medium, 1 Low

- Completed Issue #87: Fix Settings menu item staying disabled after closing Settings window
  - Added missing `std::mem::forget(settings_item)` in `src/ui/menu_bar/setup.rs`
  - This matches the pattern used for `about_item` and prevents the menu item from being invalidated

### 2026-01-11 (Roadmap Update)
- Added Phase 7: Settings Window Polish with 3 new issues
  - #87: Settings menu item stays disabled after closing Settings window (🔴 Critical)
  - #86: Show real-time validation error when entering duplicate allowed key (🟢 Medium)
  - #85: Add Undo button to Settings window for reverting individual changes (🔵 Low)
- Closed Issue #83: Add ability to select and remove individual allowed keys in Settings
- Updated issue counts: 3 open, 40 closed
- Updated priority counts: 1 Critical, 0 High, 1 Medium, 1 Low
- Updated critical path diagram for Phase 7

### 2026-01-11
- Completed Issue #83: Add ability to select and remove individual allowed keys in Settings
  - Replaced NSTextView with custom AllowedKeysListView for selectable row display
  - Created `src/ui/views/allowed_keys_list.rs` with custom NSView implementation:
    - Draws keys as rows with selection highlighting
    - Handles mouse clicks to select/deselect rows
    - Supports Delete key to remove selected key (when view has focus)
    - Uses flipped coordinate system for proper row calculation
  - Added selection state management in `src/ui/state.rs`:
    - `SELECTED_ALLOWED_KEY_INDEX` thread-local Cell for tracking selected row
    - `get_selected_allowed_key_index()`, `set_selected_allowed_key_index()`, `clear_selected_allowed_key_index()` helpers
  - Added "Remove" button to Settings window (between Add and Clear All):
    - Button disabled by default, enabled when a row is selected
    - Removes selected key from pending state
    - Shows feedback message after removal
  - Integrated with existing pending state management:
    - Selection cleared on window close, Clear All, or Reset to Default
    - Removal modifies pending state (not saved until user clicks Save)
  - UI improvements:
    - Selection uses system highlight color (`selectedContentBackgroundColor`)
    - Text color adjusts for readability on selection
    - Placeholder text shown when list is empty
  - Added 5 new unit tests for selection state management
  - All 221 tests pass, clippy clean, build successful
  - Updated issue counts: 0 open, 40 closed

- Completed Issue #80: Fix Settings Cancel button not discarding allowed_keys changes
  - Root cause: allowed_keys modifications were immediately applied to global config via `set_current_config()`
  - Added pending state management for allowed_keys in Settings window
  - Created `PENDING_ALLOWED_KEYS` thread-local storage in `src/ui/state.rs`
  - Added helper functions: `init_pending_allowed_keys()`, `get_pending_allowed_keys()`, `set_pending_allowed_keys()`, `clear_pending_allowed_keys()`
  - Modified `add_allowed_key_from_field()`, `clear_all_allowed_keys()`, `add_preset()`, and `reset_settings_to_defaults()` to update pending state instead of global config
  - Modified `save_settings_from_window()` to commit pending state to config on Save
  - Modified `cleanup_settings_window_references()` to clear pending state on Cancel/close (discards changes)
  - Modified `show_settings_window()` to initialize pending state from config on window open
  - Behavior now matches other settings fields: changes are only persisted when user clicks Save
  - Added 6 new unit tests for pending allowed keys state management
  - All 211 tests pass, clippy clean, build successful
  - Updated issue counts: 0 open, 39 closed

- Completed Issue #65: Add preset groups for key allowlist (Media Keys, System Shortcuts)
  - Added `presets` module in `src/input/allowed_keys.rs` with four preset definitions:
    - Media Keys: F10 (Mute), F11 (Volume Down), F12 (Volume Up)
    - Spotlight: Cmd+Space
    - Mission Control: Ctrl+Up/Down/Left/Right
    - Screenshots: Cmd+Shift+3/4/5
  - Each preset includes name, key list, and descriptive tooltip
  - Added `add_preset_keys()` helper function to merge presets into existing key list
    - Handles duplicate detection (case-insensitive)
    - Validates keys before adding
    - Returns count of keys actually added
  - Settings window UI updates (`src/ui/windows/settings.rs`):
    - Increased window height from 550 to 600 pixels
    - Added "Quick Add:" label row with preset buttons
    - Two rows of small, clickable buttons: [Media Keys] [Spotlight] and [Mission Control] [Screenshots]
    - All preset buttons have tooltips showing which keys will be added
    - Clicking a preset adds all its keys (excluding duplicates) to the allowlist
    - Visual feedback shows how many keys were added
  - Added 4 new action handlers to SettingsActionHandler:
    - `addMediaKeysPreset:`, `addSpotlightPreset:`, `addMissionControlPreset:`, `addScreenshotsPreset:`
  - Added `add_preset()` helper function to handle preset button clicks
    - Updates config in memory
    - Refreshes keys display in text view
    - Shows appropriate feedback message
  - Added 13 new unit tests for preset functionality:
    - Tests for each preset adding correct keys
    - Tests for duplicate handling (case-insensitive)
    - Tests for multiple preset combinations
    - Tests validating preset constants are non-empty and all keys are valid
  - All 205 tests pass, clippy clean, build successful
  - Updated issue counts: 0 open, 38 closed
  - Phase 6: Enhanced Input Control is now complete

### 2026-01-10
- Completed Issue #64: Add configurable key allowlist to pass keys through shield
  - Added `allowed_keys: Option<Vec<String>>` field to Config struct
  - Added 6 comprehensive config tests for serialization/deserialization of allowed_keys
  - Created `src/input/allowed_keys.rs` module for key management:
    - `AllowedKey` struct with parsing and matching logic
    - Global RwLock-protected state for thread-safe key storage
    - `parse_and_set_allowed_keys()` for bulk key validation with detailed error messages
    - `is_key_allowed()` for event filtering
    - `clear_allowed_keys()` for cleanup
    - Added 13 comprehensive unit tests covering parsing, validation, and event matching
  - Supports same format as exit keys (e.g., "Cmd+Space", "F11", "Ctrl+Option+A")
  - Also supports simple keys without modifiers (e.g., "F11", "Space")
  - Modified event tap callback in `src/platform/event_tap.rs`:
    - Checks if incoming key events match allowed keys before blocking
    - Allowed keys pass through to the system
    - Exit key check remains highest priority
  - Shield integration in `src/ui/shield.rs`:
    - Load allowed keys from config during shield activation
    - Display configured keys in activation banner if any are set
    - Clear allowed keys on shield deactivation
    - Handle invalid keys gracefully with warning messages
  - Settings UI in `src/ui/windows/settings.rs`:
    - Increased window height from 370 to 550 pixels
    - Added "Allowed Keys" section with:
      - Scrollable NSTextView showing current allowed keys (one per line)
      - NSScrollView wrapper with vertical scrollbar
      - Input field for adding new keys with placeholder hint "e.g. Cmd+Space, F11"
      - Real-time validation as user types
      - "Add" button to add validated keys to the list
      - "Clear All" button to remove all keys from the list
      - Duplicate detection to prevent adding same key twice
    - Created `AddKeyFieldDelegate` for real-time input validation
    - Added action handlers: `addAllowedKey:` and `clearAllowedKeys:`
    - Keys are added/removed in memory, persisted when user clicks "Save"
    - "Reset to Default" button clears allowed keys list
  - UI state management in `src/ui/state.rs`:
    - Added 5 new atomic pointer state variables:
      - `ALLOWED_KEYS_VIEW` - NSTextView displaying keys
      - `ALLOWED_KEYS_SCROLL` - NSScrollView wrapper
      - `ADD_KEY_FIELD` - Input field for new keys
      - `ADD_KEY_VALIDATION` - Validation message label
      - `ADD_KEY_FIELD_DELEGATE` - Real-time validation delegate
    - Proper cleanup on window close
  - Total of 19 new tests added (6 config + 13 allowed_keys module)
  - Enables media controls (F10-F12), Spotlight (Cmd+Space), Mission Control, and custom shortcuts while shield is active
  - Config file format: `allowed_keys = ["Cmd+Space", "F11", "F12"]`
  - Updated issue counts: 1 open, 37 closed
  - Updated priority counts: 0 Medium, 1 Low

- Completed Issue #54: Cache formatted duration string in timer display
  - Added `format_duration_cached()` function in `src/timer/formatting.rs`
  - Uses thread-local storage to cache the formatted string and last-seen seconds value
  - When seconds value is unchanged (59 of 60 frames per second), returns cached string
  - When seconds value changes, recomputes and caches the new formatted string
  - Updated `src/ui/views/timer_display.rs` to use the cached version
  - Reduces 59 unnecessary string allocations per second during timer display
  - Added 4 new tests for the cached formatting function
  - All 182 tests pass (was 178), clippy clean, build successful
  - Updated issue counts: 2 open, 36 closed
  - Updated priority counts: 1 Medium, 1 Low

- Completed Issue #60: Expand UI state and settings validation tests
  - Added 8 new tests to `src/ui/state.rs` for UI constants validation:
    - `test_close_button_size_positive`: Verifies close button size is positive
    - `test_close_button_margin_positive`: Verifies close button margin is positive
    - `test_close_button_label_dimensions_positive`: Verifies label height and width are positive
    - `test_close_button_hold_duration_reasonable`: Verifies hold duration is between 1-10 seconds
    - `test_timer_display_dimensions_positive`: Verifies timer display width and height are positive
    - `test_timer_display_width_greater_than_margins`: Verifies width accommodates margins
    - `test_animation_interval_is_60fps`: Verifies animation interval matches 60 FPS
    - `test_screen_saver_window_level_valid`: Verifies window level is positive
  - Added 8 new tests to `src/ui/windows/settings.rs` for exit key validation:
    - `test_validate_exit_key_input_valid_key`: Tests valid single modifier key
    - `test_validate_exit_key_input_valid_with_multiple_modifiers`: Tests multiple modifiers
    - `test_validate_exit_key_input_empty_returns_none`: Tests empty input returns Valid(None)
    - `test_validate_exit_key_input_whitespace_only`: Tests whitespace-only input
    - `test_validate_exit_key_input_whitespace_tabs_newlines`: Tests tabs/newlines
    - `test_validate_exit_key_input_invalid_key`: Tests invalid key names
    - `test_validate_exit_key_input_missing_modifier`: Tests key without modifier
    - `test_validate_exit_key_input_invalid_modifier`: Tests unknown modifiers
  - Made `ExitKeyValidation` enum and `validate_exit_key_input` function `pub(crate)` for testing
  - Total tests increased from 159 to 175 (+16 new tests)
  - All 175 tests pass, clippy clean, build successful
  - Updated issue counts: 3 open, 35 closed
  - Updated priority counts: 1 Medium, 2 Low

- Completed Issue #75: Add integration tests for menu bar timer functionality
  - Created `src/timer/integration_tests.rs` with 47 comprehensive tests
  - Config-based timer activation tests:
    - Verify `default_timer` parsing for various formats (30m, 2h, 1h30m, 90s)
    - Verify minimum (5s) and maximum (24h) boundary validation
    - Verify invalid formats are rejected gracefully
  - Timer expiry behavior tests:
    - Verify timer initialization sets correct duration and start time
    - Verify `get_remaining_seconds()` returns correct values
    - Verify expired timer returns 0
    - Verify disabled timer returns `u64::MAX`
    - Verify `WARNING_SHOWN` flag behavior
  - Mode consistency tests:
    - Verify `MODE_MENU_BAR` and `IS_ACTIVE` state management
    - Verify double-activation prevention
    - Verify shield can be reactivated after deactivation
    - Verify deactivation returns to menu bar mode
  - Additional tests for pointer state initialization, sleep assertions, and config round-trips
  - All 159 tests pass, clippy clean, build successful
  - Updated issue counts: 4 open, 34 closed

- Completed Issue #73: Reduce minimum auto-exit timer to 5 seconds and fix timer behavior
  - Updated `MIN_TIMER_SECONDS` constant from 60 to 5 in `src/timer/mod.rs`
  - Updated validation error message to say "at least 5 seconds" instead of "1 minute"
  - Updated tests: `30s` (now valid), added `test_parse_duration_minimum_boundary` for edge cases
  - CLI now accepts `--timer 5s` (5 seconds minimum) and rejects `--timer 4s`
  - Settings window validation automatically uses the new minimum (uses `parse_duration`)
  - **Fixed**: Timer expiry now returns to menu bar instead of quitting application
    - Refactored `main.rs` to always set up menu bar (even with CLI args)
    - `MODE_MENU_BAR` is now always true, so `deactivate_shield()` is called on timer expiry
    - App stays running in menu bar after shield deactivation
  - **Fixed**: Default timer from Settings now works when starting protection from menu bar
    - Added `default_timer` config loading to `activate_shield()` in `src/ui/shield.rs`
    - Timer display view is created and shown when default timer is configured
    - Timer info is displayed in the shield active status
  - All 111 tests pass, clippy clean, build successful
  - Updated issue counts: 4 open, 33 closed

- Completed Issue #53: Cache redundant atomic loads in timer callback
  - Cached `MODE_MENU_BAR` load at the start of `timer_callback` (was loaded twice: lines 61 and 87)
  - Cached view pointer loads (`CLOSE_BUTTON`, `CLOSE_BUTTON_LABEL`, `TIMER_VIEW`) upfront
  - Replaced `with_ptr_void` calls with direct pointer dereference using cached values
  - Added SAFETY comments for each pointer dereference explaining validity guarantees
  - Reduces atomic load operations per callback from 7 to 4 (plus 1 for AUTO_EXIT_ENABLED)
  - Timer callback runs at 60Hz, so this reduces unnecessary overhead in the hot path
  - All 110 tests pass, clippy clean, build successful
  - Updated issue counts: 4 open, 32 closed
  - Updated priority counts: 2 Medium, 2 Low

- Completed Issue #57: Add safety documentation to std::mem::forget and unsafe blocks
  - Added SAFETY comments to 35+ `std::mem::forget` calls explaining:
    - Why preventing drop is correct (ownership transfer to Objective-C runtime or global storage)
    - What guarantees exist (retained by view hierarchy, stored in AtomicPtr)
    - How cleanup occurs (window close, deactivate_shield, or app duration)
  - Added SAFETY comments to key unsafe blocks in priority files:
    - `src/platform/event_tap.rs`: event_tap_callback and setup_event_tap
    - `src/shield_core.rs`: CFRunLoopRunInMode, NSWindow initialization, setReleasedWhenClosed
    - `src/ui/shield.rs`: timer_callback, start/stop timer, Retained::from_raw reclamation
    - `src/main.rs`: start_close_button_timer
  - Files documented:
    - `src/ui/windows/settings.rs`: 16 std::mem::forget calls (delegates, fields, buttons, window)
    - `src/ui/windows/about.rs`: 8 std::mem::forget calls (delegates, labels, buttons, window)
    - `src/ui/menu_bar/setup.rs`: 5 std::mem::forget calls (action handlers, menu items)
    - `src/ui/views/timer_display.rs`: 3 std::mem::forget calls (timer labels)
    - `src/ui/views/close_button_label.rs`: 1 std::mem::forget call (label)
    - `src/platform/event_tap.rs`: 1 std::mem::forget call (CFMachPort)
  - All 110 tests pass, cargo check clean
  - Updated issue counts: 5 open, 31 closed
  - Updated priority counts: 3 Medium, 2 Low

- Completed Issue #56: Create pointer helper to reduce null-check duplication
  - Created `src/ui/ptr_helper.rs` module with three helper functions:
    - `with_ptr<T, F, R>`: Execute closure with dereferenced AtomicPtr, returns `Option<R>`
    - `with_ptr_void<T, F>`: Execute closure with dereferenced AtomicPtr, no return value
    - `with_raw_ptr<T, F>`: Execute closure with raw pointer, for already-loaded pointers
  - Updated files to use new helpers:
    - `src/ui/windows/settings.rs`: 15+ null-check patterns replaced
    - `src/ui/windows/about.rs`: 4 null-check patterns replaced
    - `src/ui/shield.rs`: 6 null-check patterns replaced
    - `src/ui/views/timer_display.rs`: 2 null-check patterns replaced
    - `src/ui/views/close_button_label.rs`: 1 null-check pattern replaced
  - Added comprehensive unit tests (7 tests) for all helper functions
  - Re-exported helpers from `src/ui/mod.rs` for convenient access
- Completed Issue #52: Optimize atomic orderings from SeqCst to appropriate weaker orderings
  - Replaced 158+ SeqCst atomic orderings with appropriate weaker orderings across 11 files
  - Pattern-based optimization: loads → Acquire, stores → Release, swap → AcqRel
  - Files modified:
    - `src/timer/state.rs`: Initialize-once pattern (Release for writes, Acquire for reads)
    - `src/input/exit_key.rs`: Configuration pattern (Release for set, Acquire for get)
    - `src/platform/event_tap.rs`: Pointer lifecycle (Release for store, Acquire for load, AcqRel for swap)
    - `src/ui/shield.rs`: Shield state management with proper ordering
    - `src/ui/windows/settings.rs`: UI state pointer access
    - `src/ui/menu_bar/setup.rs`, `src/ui/menu_bar/handlers.rs`: Menu state
    - `src/ui/windows/about.rs`: About window state
    - `src/ui/views/timer_display.rs`, `src/ui/views/close_button_label.rs`: View state
    - `src/main.rs`: Initialization stores
  - All 100 tests pass, clippy clean, build successful
  - Updated issue counts: 7 open, 29 closed
  - Updated priority counts: 5 Medium, 2 Low

- Completed Issue #55: Refactor show_settings_window() into smaller functions
  - Reduced `show_settings_window()` from 600+ lines to ~35 lines (orchestration only)
  - Extracted constants: `WINDOW_WIDTH`, `WINDOW_HEIGHT`, `MARGIN`, layout constants
  - Created helper functions: `prepare_settings_window()`, `create_settings_panel()`
  - Created delegate getters: `get_or_create_window_delegate()`, `get_or_create_action_handler()`, etc.
  - Extracted section functions:
    - `setup_exit_key_section()` (~90 lines) - Exit key field with validation
    - `setup_timer_section()` (~160 lines) - Timer checkbox, value field, unit dropdown
    - `setup_opacity_section()` (~140 lines) - Opacity slider with min/max labels
    - `setup_button_section()` (~80 lines) - Reset, Cancel, Save buttons
  - Added `finalize_and_show_window()` for window activation and display
  - All 100 tests pass, clippy clean, build successful
  - Updated issue counts: 8 open, 28 closed
  - No more high-priority issues remaining (0 High, 6 Medium, 2 Low)

- Created Issue #64: Add configurable key allowlist to pass specific keys through the shield
  - New feature allowing users to specify keys/key combinations that won't be blocked
  - Supports same format as exit key (e.g., `Cmd+Space`, `F11`, `Ctrl+Option+A`)
  - Enables media controls, Spotlight, and other global shortcuts while shield is active
  - Config file format: `allowed_keys = ["Cmd+Space", "F11", "F12"]`
  - Settings UI with list management (add/remove keys with validation)
  - Priority: 🟢 Medium, Effort: ~2-3 days
- Created Issue #65: Add preset groups for key allowlist (Media Keys, System Shortcuts)
  - Follow-up to #64 for improved UX with one-click preset buttons
  - Presets: Media Keys (F10-F12), Spotlight (Cmd+Space), Mission Control, Screenshots
  - Depends on #64 (Key Allowlist feature)
  - Priority: 🔵 Low, Effort: ~0.5 day
- Added Phase 6: Enhanced Input Control section to roadmap
- Updated issue counts: 8 open, 28 closed (consistent with Issue Summary above)
- Updated priority counts: 6 Medium, 2 Low (added #64 Medium, #65 Low)
- Removed #28 from open issues (was closed)

### 2026-01-09
- Completed Issue #59: Add unit tests for timer state module
  - Added 22 comprehensive unit tests for the timer state module (`src/timer/state.rs`)
  - Tests cover all public functions: `init_auto_exit_timer()`, `get_remaining_seconds()`
  - Tests cover all global state variables: `AUTO_EXIT_ENABLED`, `AUTO_EXIT_DURATION_SECS`, `AUTO_EXIT_START_TIME`, `WARNING_SHOWN`
  - Test categories:
    - Initialization tests: sets enabled flag, duration, start time; handles zero/max/overwrite cases
    - Remaining seconds tests: disabled timer returns MAX, just started, expired, halfway, near end
    - Warning state tests: initially false, can be set, reset clears value
    - Edge cases: elapsed time overflow protection, duration saturating sub, state independence
    - Boundary conditions: 24-hour timer, 1-second timer
  - Tests use `reset_timer_state()` helper to isolate global state between tests
  - Total tests: 100 (was 78, +22 new timer state tests)
  - Updated issue counts: 8 open, 26 closed
  - Updated priority counts: 1 High, 5 Medium, 2 Low

- Completed Issue #58: Add unit tests for lock module (single-instance enforcement)
  - Added 22 comprehensive unit tests for the lock module (`src/lock/mod.rs`)
  - Tests cover all lock functions: `lock_file_path()`, `is_process_running()`, `acquire_instance_lock()`, `release_instance_lock()`
  - Test categories:
    - Path tests: verify lock file path is correct on macOS
    - Process detection: current process, non-existent PIDs, system processes
    - Lock lifecycle: acquisition, release, re-acquisition cycles
    - Edge cases: stale locks, invalid PIDs, empty files, whitespace, overflow values
    - Error handling: negative PIDs, invalid content, permission scenarios
  - Tests use `tempfile` crate for isolated temp directories (no interference with real lock file)
  - Made `lock_file_path()` and `is_process_running()` `pub(crate)` for testability
  - Total tests: 78 (was 56, +22 new lock module tests)
  - Updated issue counts: 9 open, 25 closed
  - Updated priority counts: 2 High, 5 Medium, 2 Low

### 2026-01-09 (Roadmap Update)
- Added 9 new issues for Phase 5: Performance & Code Quality Improvements
  - Performance: #52 (atomic orderings), #53 (timer callback cache), #54 (duration format cache)
  - Maintenance: #55 (settings refactor), #56 (pointer helper), #57 (safety docs)
  - Testing: #58 (lock tests), #59 (timer state tests), #60 (UI state tests)
- Updated issue counts: 10 open, 24 closed
- Updated priority breakdown: 3 High, 5 Medium, 2 Low
- Added Phase 5 section with categorized improvements
- Updated recommended implementation order for Phase 5
- Updated critical path diagram to show Phase 4 complete and Phase 5 current
- Total estimated effort for Phase 5: ~30-40 hours

### 2026-01-09
- Completed Issue #49: Add real-time validation for Timer duration field in Settings
  - Created `TimerFieldDelegate` class implementing `NSControlTextEditingDelegate` protocol
  - Added `validate_timer_realtime()` function for validating numeric timer input
  - Timer field now validates as user types when the "Enable auto-exit timer" checkbox is enabled
  - Valid numeric input (>0) shows "✓ Valid" (green)
  - Zero value shows "Must be greater than 0" (red)
  - Non-numeric input shows "Enter a number" (red)
  - Empty field shows no message (Save button handles "Duration required")
  - Validation label cleared when checkbox is unchecked (disabled state)
  - No validation on initial window open (follows Issue #48 pattern)
  - Added `TIMER_FIELD_DELEGATE` to `settings` state module
  - Updated issue counts: 1 open, 24 closed

- Completed Issue #48: Exit Key validation should not show on initial Settings window open
  - Removed the `validate_exit_key_realtime()` call that ran immediately when Settings window opens
  - Validation label now starts empty (no "✓ Valid" message on open)
  - Validation only appears after user modifies the Exit Key field
  - The `controlTextDidChange:` delegate method still handles real-time validation on edits
  - Updated issue counts: 1 open, 23 closed

- Completed Issue #46: Add 'Reset to Default' button and improve spacing in Settings window
  - Added "Reset to Default" button on the left side of the button row
  - Button resets all settings fields to their default values:
    - Exit Key: Cmd+Q (DEFAULT_EXIT_KEY constant)
    - Timer: disabled (checkbox unchecked) and value cleared
    - Opacity: 50% (DEFAULT_OVERLAY_OPACITY constant)
  - Reset does NOT auto-save - user must explicitly click Save to persist changes
  - Increased window height from 340 to 370 pixels for improved visual spacing
  - Added `resetDefaults:` action method to `SettingsActionHandler`
  - Created `reset_settings_to_defaults()` function to update all UI fields
  - Updated issue counts: 1 open, 22 closed

- Completed Issue #44: Refactoring - Improve code safety, reduce duplication, consolidate global state
  - Created new `src/shield_core.rs` module with shared shield activation logic
  - Extracted reusable functions: `ensure_accessibility()`, `create_shield_window()`, `setup_close_button()`
  - Removed ~150 lines of duplicated code between `main.rs` and `ui/shield.rs`
  - Consolidated 25+ individual `AtomicPtr<c_void>` globals into structured modules:
    - `shield::` - Shield window, close button, timer display, sleep assertion state
    - `menu_bar::` - Menu bar items and action handlers
    - `settings::` - Settings window UI element references
    - `about::` - About panel state
  - Centralized UI constants into organized submodules:
    - `close_button::` - SIZE, MARGIN, LABEL_HEIGHT, LABEL_WIDTH, HOLD_DURATION_SECS
    - `timer_display::` - HEIGHT, WIDTH, MARGIN
    - `animation::` - INTERVAL_SECS
    - `window_level::` - SCREEN_SAVER
  - Created `shield_core::theme` module with background color constants (BG_RED, BG_GREEN, BG_BLUE, BUTTON_LABEL_GAP)
  - Config opacity is now actually applied to shield overlay (was hardcoded to 0.5 before)
  - Fixed mutex poison handling in `config/file.rs` to recover gracefully instead of panicking
  - Legacy aliases maintained for backwards compatibility - existing code continues to work
  - Added new test: `test_constants_consistency` verifies legacy aliases match new module constants
  - All 52 tests pass, clippy clean, build successful
  - Updated issue counts: 1 open, 21 closed

- Completed Issue #42: Enable Help menu links (Documentation & Report Issue)
  - "View Documentation" menu item now opens the GitHub README in the default browser
  - "Report Issue" menu item now opens the GitHub new issue page in the default browser
  - Both menu items are now enabled and clickable
  - Created `HelpActionHandler` class with action methods for opening URLs
  - Uses `NSWorkspace.openURL()` to open URLs in the default browser
  - Removed unused "Release Notes" menu item (not in requirements)
  - Updated issue counts: 1 open, 20 closed

### 2026-01-08
- Completed Issue #11: Add install script for easy CLI access
  - Created `install.sh` script that builds and installs Cat Shield
  - Script checks for Rust/Cargo availability before building
  - Builds release binary with `cargo build --release`
  - Installs to `/usr/local/bin/catshield` by default (configurable via `INSTALL_DIR` env var)
  - Handles permissions correctly (uses sudo when needed for system directories)
  - Provides clear success/error messages with colored output
  - Created `uninstall.sh` script to remove the binary and optionally config files
  - Updated README.md with comprehensive installation instructions
  - Users can now run `catshield` from anywhere in their terminal
  - Updated issue counts: 1 open, 19 closed

- Completed Issue #38: Fix menu items not visually disabled when windows are open
  - Settings and About menu items now appear visually grayed out when their windows are open
  - Root cause: NSMenu's `autoenablesItems` was overriding manual `setEnabled(false)` calls
  - Fix: Added `menu.setAutoenablesItems(false)` in menu bar setup (`src/ui/menu_bar/setup.rs`)
  - This allows the existing `setEnabled()` calls in settings/about window code to work correctly
  - Menu items properly re-enable (visually and functionally) when windows are closed
  - Updated issue counts: 2 open, 18 closed

- Completed Issue #30: Add real-time validation for Exit Key field in Settings Window
  - Exit Key text field now validates input as the user types
  - Added `exitKeyChanged:` action method to `SettingsActionHandler`
  - Created `validate_exit_key_realtime()` function to check input and update validation label
  - Uses existing `ExitKey::parse()` for validation logic
  - Validation label shows "✓ Valid" (green), "Using default" (green for empty), or error message (red)
  - Initial validation displayed when settings window opens (pre-filled value is validated)
  - Improves UX by providing immediate feedback instead of waiting until Save button is clicked
  - Updated issue counts: 3 open, 16 closed

### 2026-01-07
- Completed Issue #19: Add About Panel
  - "About Cat Shield" menu item now functional between Settings and Help
  - About panel displays cat emoji (🐱), app name, version, and description
  - Version pulled from Cargo.toml using `env!("CARGO_PKG_VERSION")`
  - Close button dismisses panel (Return key also works)
  - Only one About panel can be open at a time (clicking menu again brings existing to front)
  - Panel centered on screen using NSPanel
  - Created `src/ui/windows/about.rs` with AboutWindowDelegate and AboutActionHandler
  - Added state variables: ABOUT_WINDOW, ABOUT_MENU_ITEM, ABOUT_ACTION_HANDLER, ABOUT_WINDOW_DELEGATE
  - Updated issue counts: 4 open, 15 closed
  - Phase 4 (Menu-Based Application Interface) now fully complete
- Created Issue #38: Menu items not visually disabled when windows are open (follow-up)
- Completed Issue #35: Split main.rs into multiple modules
  - Created modular architecture with 6 top-level modules: input, timer, config, platform, ui, lock
  - `src/input/`: keycodes.rs (macOS virtual keycode mappings), exit_key.rs (exit key parsing and state)
  - `src/timer/`: parsing.rs, formatting.rs, state.rs (auto-exit timer management)
  - `src/config/`: types.rs (Config struct), file.rs (I/O), args.rs (CLI parsing with clap)
  - `src/platform/`: bindings.rs (FFI), accessibility.rs, power.rs, event_tap.rs
  - `src/ui/`: state.rs, helpers.rs, shield.rs, views/ (CloseButtonView, TimerDisplayView), windows/ (settings), menu_bar/ (setup, handlers)
  - `src/lock/`: single-instance PID lock mechanism
  - `src/lib.rs`: public API with re-exports for convenient usage
  - Reduced main.rs from ~4354 lines to ~530 lines (88% reduction)
  - All 50 tests pass with no modifications needed
  - Clippy passes with no warnings
  - Improved code organization, discoverability, and maintainability
- Updated issue counts: 5 open, 14 closed
- Completed Issue #24: Fix multiple menu bar icons bug (single-instance enforcement)
  - Implemented PID-based lock file mechanism at `~/.config/catshield/catshield.lock`
  - Added `acquire_instance_lock()` function to check for existing instances
  - Added `release_instance_lock()` function to clean up on exit
  - Added `is_process_running()` helper using POSIX kill(pid, 0) to check process existence
  - Lock check happens early in `main()` before NSApplication initialization
  - Lock cleanup occurs in both menu bar mode and immediate mode exit paths
  - Stale lock files from crashed instances are automatically cleaned up
  - User-friendly error message when Cat Shield is already running
- Updated issue counts: 5 open, 13 closed
- Updated roadmap with 3 new issues (#24, #28, #30)
  - #24: Bug fix for multiple menu bar icons (🟡 High priority) - NOW COMPLETED
  - #28: Migration to Cacao library (🟢 Medium - future consideration)
  - #30: Real-time Exit Key validation in Settings (🟢 Medium)
- Revised critical path to include new issues and bug fix priority
- Updated recommended implementation order
- Completed Issue #31: Add CI workflow with lint, format, test, and build checks
  - Created `.github/workflows/ci.yml` with three parallel jobs
  - Lint & Format job: checks code formatting with `cargo fmt --check` and runs Clippy with `-D warnings`
  - Test job: runs `cargo test --verbose`
  - Build job: compiles release build with `cargo build --release`
  - All jobs run on `macos-latest` runners (required for objc2/AppKit dependencies)
  - Uses `dtolnay/rust-toolchain@stable` for reliable Rust toolchain installation
  - Uses `Swatinem/rust-cache@v2` for dependency caching to speed up builds
  - Triggers on push to main, PRs targeting main, and manual workflow dispatch

### 2026-01-06
- Completed Issue #16: Create Settings Window
  - Settings window accessible from menu bar via Settings... (Cmd+,)
  - Exit Key Shortcut text field with real-time validation using `ExitKey::parse()`
  - Default Timer section with enable checkbox and duration text field
  - Timer field accepts formats: 30m, 2h, 1h30m, 90s using existing `parse_duration()`
  - Timer text field disabled/grayed when checkbox is unchecked
  - Overlay Opacity slider from 20% to 80% with live percentage display
  - Save button validates all fields before persisting to `~/.config/catshield/config.toml`
  - Cancel button (Escape) discards changes and closes window
  - Only one settings window can be open at a time
  - Validation feedback with green checkmarks for valid input or red error messages
  - Settings window opens centered on screen
  - Uses NSPanel for utility window style
  - Created `SettingsActionHandler` class with action methods for all UI interactions
- Completed Issue #18: Extend Config for New Settings
  - Added `default_timer: Option<String>` field for persistent timer duration
  - Added `overlay_opacity: Option<f64>` field for configurable overlay opacity
  - Added `Serialize` derive to Config struct for saving
  - Added `Config::save()` method to write settings to TOML file
  - Added `Config::opacity()` helper method with clamping to valid range (0.2-0.8)
  - Creates config directory if it doesn't exist on save
  - Default opacity is 0.5 (50%) when not specified in config
  - Added global config storage with `CURRENT_CONFIG`, `get_current_config()`, `set_current_config()`
- Completed Issue #10: Add informative labels to overlay UI elements
  - Timer display now shows "Time Remaining:" header with countdown text (e.g., "29m 45s")
  - Warning state shows yellow text with "Exiting soon!" indicator when < 1 minute remaining
  - Close button has "Hold 3s to exit" instruction label positioned below the button
  - During hold, label dynamically shows countdown ("3s...", "2s...", "1s...")
  - Added text rendering via NSAttributedString with system fonts
  - Added helper functions: `draw_text()`, `draw_text_centered()`, `draw_text_bold()`
  - Created new `CloseButtonLabelView` custom NSView for the instruction label
  - All labels have good contrast with proper background styling
- Completed Issue #25: Limit Claude GitHub Action to only respond to comments
  - Removed `pull_request`, `issues`, `pull_request_review`, `pull_request_review_comment` triggers
  - Kept only `issue_comment` trigger (covers both issue and PR comments)
  - Added `if: contains(github.event.comment.body, '@claude')` condition
  - Reduces noise from automatic triggers; Claude now only responds when explicitly mentioned
- Completed Issue #17: Refactor Overlay to On-Demand Activation
  - `catshield` (no args) now shows menu bar icon only, no overlay
  - CLI args (--timer, --exit-key) start protection immediately (preserves scripting)
  - "Start Protection" menu item activates shield from menu bar
  - Shield exit (close button, exit key, timer) returns to menu bar state
  - Added `activate_shield()` and `deactivate_shield()` functions
  - Menu item disabled during active protection, re-enabled after deactivation
  - Proper cleanup: event tap, sleep assertion, window, timer state

### 2026-01-03
- Completed Issue #15: Create Main Dropdown Menu
  - Comprehensive menu structure with all application features organized into sections
  - Protection section: Start Protection and Stop Protection menu items (ready for #17)
  - Configuration section: Settings menu item with Cmd+, keyboard shortcut (ready for #16)
  - Information section: About Cat Shield (ready for #19) and Help submenu
  - Help submenu includes: View Documentation, Report Issue, and Release Notes
  - All menu items include descriptive tooltips explaining their purpose
  - Stop Protection initially hidden, will be shown when shield becomes active
  - Proper menu organization with section separators for clarity
  - Keyboard shortcuts: Cmd+Q for Quit, Cmd+, for Settings
  - Enhanced tooltip on menu bar icon: "Cat Shield - Protect your work from curious cats"
  - Foundation complete for #16 (Settings Window) and #19 (About Panel) to build upon
  - Unblocks #16 and #19 for continued Phase 4 development

### 2026-01-02
- Completed Issue #14: Create Menu Bar Infrastructure (NSStatusItem)
  - Cat emoji (🐱) appears in menu bar when app launches without CLI args
  - App stays running in background in menu bar mode
  - Tooltip shows "Cat Shield" on hover
  - Basic dropdown menu with branding, placeholder items, and Quit
  - CLI args (--timer, --exit-key) bypass menu and start shield immediately
  - Unblocks #15, #17 for continued Phase 4 development
- Added Phase 4: Menu-Based Application Interface (Epic #13)
  - #14: Create Menu Bar Infrastructure (NSStatusItem)
  - #15: Create Main Dropdown Menu
  - #16: Create Settings Window
  - #17: Refactor Overlay to On-Demand Activation
  - #18: Extend Config for New Settings (timer, opacity)
  - #19: Add About Panel
- Updated roadmap with 9 open issues
- Defined critical path for Phase 4 implementation
- Moved completed items (opacity, menu bar) from Future Considerations to active issues

### 2025-12-31
- Added configurable keyboard shortcut for exit (Issue #7)
  - Use `--exit-key "Cmd+Shift+Q"` or `-e "Ctrl+Option+Escape"` for custom exit shortcut
  - Config file support: `~/.config/catshield/config.toml` with `exit_key = "Cmd+Option+U"`
  - CLI argument overrides config file setting
  - Supports modifiers: Cmd, Option, Shift, Ctrl (and aliases like Command, Alt, Control)
  - Supports keys: A-Z, 0-9, F1-F12, Escape, Return, Tab, Space, Delete, arrow keys
- Added configurable timer-based auto-exit (Issue #6)
  - Use `--timer 30m` or `-t 2h` to set auto-exit duration
  - Visual progress bar shows remaining time on overlay
  - Warning shown 1 minute before auto-exit
  - Duration range: 1 minute to 24 hours
- Added click-and-hold close button in top-right corner (Issue #5)
  - 3-second hold requirement prevents accidental exits from cats
  - Visual progress ring indicator during hold
  - Works without Accessibility permissions
- Completed migration to objc2 ecosystem (Issue #3)
- Improved code safety and maintainability
