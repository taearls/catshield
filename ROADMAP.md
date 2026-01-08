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
  - Timer validation (1 minute minimum, 24 hours maximum)
- [x] **Issue #7**: Add configurable keyboard shortcut for exit
  - CLI argument: `--exit-key` / `-e` (e.g., "Cmd+Shift+Q", "Ctrl+Option+Escape")
  - Config file support: `~/.config/catshield/config.toml` with `exit_key = "Cmd+Option+U"`
  - Full key combination validation with descriptive error messages
  - Support for common modifier keys (Cmd, Option, Shift, Ctrl) with aliases
  - Support for letters (A-Z), numbers (0-9), function keys (F1-F12), and special keys
  - CLI argument takes precedence over config file
  - Default remains Cmd+Option+U for backwards compatibility

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

**Implementation Order:**
```
#14: Menu Bar Infrastructure ✅
    ├── #17: Refactor Overlay ✅ (parallel with #15)
    └── #15: Dropdown Menu ✅
            ├── #18: Extend Config ✅ (parallel)
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

### Other Open Issues

| Priority | Issue | Title | Effort |
|----------|-------|-------|--------|
| 🟢 Medium | #30 | Add real-time validation for Exit Key field in Settings Window | ~0.5 day |
| 🟢 Medium | #28 | Migrate from raw objc2 bindings to Cacao library | ~1-2 weeks |
| 🔵 Low | #11 | Add install script for easy CLI access | ~0.5 day |

## Issue Summary

| Status | Count | Issues |
|--------|-------|--------|
| Open | 4 | #11, #13, #28, #30 |
| Closed | 15 | #3, #5, #6, #7, #10, #14, #15, #16, #17, #18, #19, #24, #25, #31, #35 |

### By Priority
- 🔴 Critical: 0
- 🟡 High: 0
- 🟢 Medium: 2 (#28, #30)
- 🔵 Low: 1 (#11)
- Epic: 1 (#13)

## Recommended Implementation Order

### Current Sprint: Phase 4 Foundation - COMPLETE
1. ~~**#14** - Menu Bar Infrastructure~~ ✅ COMPLETED
2. ~~**#15** - Main Dropdown Menu~~ ✅ COMPLETED
3. ~~**#17** - Refactor Overlay~~ ✅ COMPLETED
4. ~~**#18** - Extend Config~~ ✅ COMPLETED
5. ~~**#16** - Settings Window~~ ✅ COMPLETED

### Next Up
6. ~~**#24** - Fix multiple menu bar icons bug~~ ✅ COMPLETED
7. ~~**#19** - About Panel~~ ✅ COMPLETED
8. **#30** - Add real-time Exit Key validation in Settings (enhances #16)
9. **#11** - Install Script (can be done anytime)

### Future
10. **#28** - Migrate to Cacao library (major refactor, low priority)

## Critical Path

```
Foundation:     #14 (Menu Bar) ✅ ──┬── #17 (Refactor Overlay) ✅
                                    │
                                    └── #15 (Dropdown Menu) ✅ ── #16 (Settings) ✅ ─┬─ #19 (About) ✅
                                                 │                                    │
Parallel:       #18 (Config) ✅ ────────────────┘                                    └─ #30 (Exit Key Validation)

Bug Fix:        #24 (Multiple Menu Icons) ✅

Independent:    #10 (UI Labels) ✅, #11 (Install Script), #31 (CI) ✅

Major Refactor: #28 (Cacao Migration) - Future consideration
```

## Future Considerations

Potential future enhancements (not yet tracked as issues):

- Multi-monitor support improvements
- Auto-start on login option
- Activity logging
- Sound effects/feedback
- Custom overlay themes

## Changelog

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
