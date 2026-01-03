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

## Open Issues

### Phase 4: Menu-Based Application Interface

**Epic #13**: Transform Cat Shield from an immediate-launch utility into a menu bar application.

| Priority | Issue | Title | Dependencies | Effort |
|----------|-------|-------|--------------|--------|
| ✅ Done | #14 | Create Menu Bar Infrastructure (NSStatusItem) | None | ~1 day |
| ✅ Done | #15 | Create Main Dropdown Menu | ✅ #14 | ~1 day |
| 🔴 Critical | #17 | Refactor Overlay to On-Demand Activation | ✅ #14 | ~2 days |
| 🟡 High | #18 | Extend Config for New Settings | None | ~1 day |
| 🟢 Medium | #16 | Create Settings Window | ✅ #15, #18 | ~3 days |
| 🔵 Low | #19 | Add About Panel | ✅ #15 | ~0.5 day |

**Implementation Order:**
```
#14: Menu Bar Infrastructure
    ├── #17: Refactor Overlay (parallel with #15)
    └── #15: Dropdown Menu
            ├── #18: Extend Config (parallel)
            └── #16: Settings Window
                    └── #19: About Panel (optional)
```

### Other Open Issues

| Priority | Issue | Title | Effort |
|----------|-------|-------|--------|
| 🟢 Medium | #10 | Add informative labels to overlay UI elements | ~1 day |
| 🔵 Low | #11 | Add install script for easy CLI access | ~0.5 day |

## Issue Summary

| Status | Count | Issues |
|--------|-------|--------|
| Open | 7 | #10, #11, #13, #16, #17, #18, #19 |
| Closed | 6 | #3, #5, #6, #7, #14, #15 |

### By Priority
- 🔴 Critical: 1 (#17)
- 🟡 High: 1 (#18)
- 🟢 Medium: 2 (#10, #16)
- 🔵 Low: 2 (#11, #19)
- Epic: 1 (#13)

## Recommended Implementation Order

### Current Sprint: Phase 4 Foundation
1. ~~**#14** - Menu Bar Infrastructure~~ ✅ COMPLETED
2. ~~**#15** - Main Dropdown Menu~~ ✅ COMPLETED
3. **#17** - Refactor Overlay (critical, unblocked by #14)
4. **#18** - Extend Config (can parallel with overlay work)

### Next Up
5. **#16** - Settings Window (biggest effort, depends on #15 + #18)
6. **#19** - About Panel (polish, low priority)
7. **#10** - UI Labels (can be done anytime)
8. **#11** - Install Script (can be done anytime)

## Critical Path

```
Foundation:     #14 (Menu Bar) ✅ ──┬── #17 (Refactor Overlay)
                                    │
                                    └── #15 (Dropdown Menu) ✅ ── #16 (Settings) ── #19 (About)
                                                 │
Parallel:       #18 (Config) ───────────────────┘

Independent:    #10 (UI Labels), #11 (Install Script)
```

## Future Considerations

Potential future enhancements (not yet tracked as issues):

- Multi-monitor support improvements
- Auto-start on login option
- Activity logging
- Sound effects/feedback
- Custom overlay themes

## Changelog

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
