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
- Completed migration to objc2 ecosystem (Issue #3)
- Improved code safety and maintainability
