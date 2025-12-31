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

## Open Issues

Currently no open issues.

## Future Considerations

Potential future enhancements (not yet tracked as issues):

- Custom unlock key combinations
- Configurable overlay opacity and color
- Multi-monitor support improvements
- System tray/menu bar integration
- Auto-start on login option
- Activity logging

## Changelog

### 2025-12-31
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
