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
| 🔵 Low | #65 | Add preset groups for key allowlist (Media Keys, System Shortcuts) | ✅ #64 | ~0.5 day |

**Implementation Order:**
```text
#64: Key Allowlist Feature ✅
    └── #65: Preset Groups (depends on #64)
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

### Recently Completed

| Issue | Title | Completed |
|-------|-------|-----------|
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
| Open | 1 | #65 |
| Closed | 37 | #3, #5, #6, #7, #10, #11, #13, #14, #15, #16, #17, #18, #19, #24, #25, #28, #30, #31, #35, #38, #42, #44, #46, #48, #49, #52, #53, #54, #55, #56, #57, #58, #59, #60, #64, #73, #75 |

### By Priority
- 🔴 Critical: 0
- 🟡 High: 0
- 🟢 Medium: 0
- 🔵 Low: 1 (#65)

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

### Phase 6: Enhanced Input Control

**Completed:**
10. ~~**#64** - Add configurable key allowlist (new feature, medium priority)~~ ✅

**Next:**
11. **#65** - Add preset groups for key allowlist (depends on #64, low priority)

## Critical Path

```
Phase 4 (COMPLETE):
Foundation:     #14 (Menu Bar) ✅ ──┬── #17 (Refactor Overlay) ✅
                                    │
                                    └── #15 (Dropdown Menu) ✅ ── #16 (Settings) ✅ ─┬─ #19 (About) ✅
                                                 │                                    │
Parallel:       #18 (Config) ✅ ────────────────┘                                    └─ #30 (Exit Key Validation) ✅

Phase 5 (CURRENT):
Testing:        #58 (Lock Tests) ✅ ──┬─── All Complete
                #59 (Timer Tests) ✅ ─┤
                #60 (UI State Tests) ✅ ┘

Performance:    #52 (Atomic Ordering) ✅ ─┬── All Complete
                #53 (Timer Cache) ✅ ─────┤
                #54 (Duration Cache) ✅ ──┘

Maintenance:    #55 (Settings Refactor) ✅ ─┬── All Complete
                #56 (Pointer Helper) ✅ ────┤
                #57 (Safety Docs) ✅ ────────┘

Phase 6 (CURRENT):
Input Control:  #64 (Key Allowlist) ✅ ─── #65 (Preset Groups)
```

## Future Considerations

Potential future enhancements (not yet tracked as issues):

- Multi-monitor support improvements
- Auto-start on login option
- Activity logging
- Sound effects/feedback
- Custom overlay themes

## Changelog

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
