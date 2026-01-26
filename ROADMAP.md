# Cat Shield Roadmap

## Quick Reference
<!-- AI-OPTIMIZED: This section contains actionable info for automated tools -->
<!-- Keep under 150 lines - agents read this first -->

**Current Phase**: 9 - iced UI Migration (Complete)
**Next Priority**: Phase 10 (Future Enhancements)
**Blocking Issues**: None

### Open Issues by Priority

| # | Title | Priority | Effort | Blocked By | Phase |
|---|-------|----------|--------|------------|-------|
| 152 | Epic: Migrate UI Rendering to iced Framework | ✅ Complete | ~4-6 weeks | None | 9 |

**Summary**: 0 open issues - Phase 9 complete!

### Completed This Session

| # | Title | Priority | Date |
|---|-------|----------|------|
| 184 | Add automatic retry for transient CI failures | 🔵 Low | 2026-01-26 |
| 166 | Documentation update for iced migration | 🔵 Low | 2026-01-26 |
| 165 | Animated cat companion overlay | 🔵 Low | 2026-01-25 |
| 164 | Performance testing and optimization | 🔵 Low | 2026-01-25 |
| 179 | Theme-aware light mode colors | 🔵 Low | 2026-01-25 |
| 163 | Visual polish and theming | 🔵 Low | 2026-01-25 |
| 162 | Linux tray integration | 🟢 Medium | 2026-01-25 |
| 161 | Windows system tray integration | 🟢 Medium | 2026-01-25 |
| 160 | macOS menu bar integration | 🟢 Medium | 2026-01-25 |
| 159 | Add overlay customization (opacity, color) | 🟢 Medium | 2026-01-25 |
| 158 | Implement settings persistence to disk | 🟢 Medium | 2026-01-25 |
| 157 | Create settings window UI in iced | 🟢 Medium | 2026-01-24 |
| 156 | Add timer countdown display to iced overlay | 🟢 Medium | 2026-01-24 |
| 155 | Implement basic overlay window in iced | 🟡 High | 2026-01-24 |
| 154 | Set up iced dependency and scaffold | 🟡 High | 2026-01-24 |
| 153 | Research: iced integration feasibility | 🟡 High | 2026-01-24 |
| 150 | Add trace logging infrastructure with persistent file output | 🟢 Medium | 2026-01-24 |
| 114 | Animated cat companion overlay (migrated to #165) | 🔵 Low | 2026-01-24 |

### Previously Completed

| # | Title | Priority | Date |
|---|-------|----------|------|
| 148 | Add 'Launch at Login' setting | 🟢 Medium | 2026-01-23 |
| 139 | Windows overlay code quality improvements | 🔵 Low | 2026-01-23 |

### Recommended Next Issues

Phase 9 is now complete. Future work items are tracked in Phase 10 (Future Considerations).

---

## Current Sprint: Phase 9 - iced UI Migration

### Epic #152: Migrate UI Rendering to iced Framework

| Status | Issue | Title | Dependencies |
|--------|-------|-------|--------------|
| ✅ | #153 | Research: iced integration feasibility | None |
| ✅ | #154 | Set up iced dependency and scaffold | None |
| ✅ | #155 | Implement basic overlay window | None |
| ✅ | #156 | Add timer countdown display | None |
| ✅ | #157 | Create settings window UI | None |
| ✅ | #158 | Implement settings persistence | None |
| ✅ | #159 | Add overlay customization | None |
| ✅ | #160 | macOS menu bar integration | None |
| ✅ | #161 | Windows system tray integration | None |
| ✅ | #162 | Linux tray integration | None |
| ✅ | #163 | Visual polish and theming | None |
| ✅ | #179 | Theme-aware light mode colors | None |
| ✅ | #164 | Performance testing and optimization | #156 |
| ✅ | #165 | Animated cat companion overlay | None |
| ✅ | #166 | Documentation update | All |

---

## Completed: Phase 8 - Cross-Platform Support

### Phase 8.2: Windows Support

| Status | Issue | Title | Dependencies |
|--------|-------|-------|--------------|
| ✅ | #100 | Windows keyboard hook (InputBlocker) | None |
| ✅ | #101 | Windows power management | None |
| ✅ | #102 | Windows system tray | None |
| ✅ | #103 | Windows overlay window | None |
| ✅ | #104 | Windows keycode mappings | None |
| ✅ | #105 | Windows entry point | None |

### Phase 8.3: Linux Support

| Status | Issue | Title | Dependencies |
|--------|-------|-------|--------------|
| ✅ | #106 | X11 keyboard grab (InputBlocker) | None |
| ✅ | #107 | Wayland input research | None |
| ✅ | #108 | Linux power management (DBus) | None |
| ✅ | #111 | Linux keycode mappings | None |
| ✅ | #109 | Linux system tray | None |
| ✅ | #110 | Linux overlay window | None |
| ✅ | #131 | Wayland shortcuts-inhibit | None |
| ✅ | #132 | XWayland fallback | None |

### Phase 8.4: CI & Testing

| Status | Issue | Title | Dependencies |
|--------|-------|-------|--------------|
| ✅ | #112 | Cross-platform CI | None |
| ✅ | #120 | Docker dev containers | None |
| ✅ | #113 | Platform integration tests | None |

---

## Critical Path

```
Phase 9 (iced Migration):
#153 (Research) ✅ GO - iced is suitable
    │
    ▼
#154 (Scaffold) ✅ iced 0.14 added, ui_iced module created
    │
    ▼
#155 (Basic Overlay) ✅ Fullscreen, transparent, exit key handling
    │
    ▼
#156 (Timer) ✅ MM:SS/HH:MM:SS countdown, progress bar, hide-timer support
    │
    ▼
#157 (Settings UI) ✅ Full settings window with 4 sections, controls, live preview
    │
    ▼
#158 (Persistence) ✅ Atomic save, platform paths, graceful error handling
    │
    ▼
#159 (Customization) ✅ Opacity 10-90%, color presets, hex colors, CLI flags
    │
    ├─► #160 (macOS) ✅ AppKit NSStatusItem + iced windows via thread spawn
    ├─► #161 (Windows) ✅ Win32 Shell_NotifyIcon + iced windows via thread spawn
    └─► #162 (Linux) ✅ ksni StatusNotifierItem + iced windows via thread spawn
                            │
    ├─► #163 (Polish) ✅ ◄──┤ Enhanced theme, widget styles, dark/light mode
    │       │
    │       └─► #179 (Light Mode Colors) ✅ All 24 style functions theme-aware
    │
    ├─► #164 (Performance) ✅ Criterion benchmarks, perf module, optimized release profile
    │
    └─► #165 (Cat Animation) ✅ Animated cat companion with bobbing, blinking, --no-cat flag
                │
                ▼
        #166 (Documentation) ✅ README, CHANGELOG, architecture docs updated
```

---

## Phase Summary

| Phase | Status | Description |
|-------|--------|-------------|
| 1 | ✅ Complete | Initial macOS release |
| 2 | ✅ Complete | Technical debt (objc2 migration) |
| 3 | ✅ Complete | User experience (timer, exit key, close button) |
| 3.5 | ✅ Complete | Overlay usability (labels) |
| 4 | ✅ Complete | Menu bar application |
| 5 | ✅ Complete | Performance & code quality |
| 6 | ✅ Complete | Enhanced input control (key allowlist) |
| 7 | ✅ Complete | Settings window polish |
| 8 | ✅ Complete | Cross-platform support |
| 9 | ✅ Complete | iced UI migration (Epic #152) |
| 10 | 📋 Planned | Future enhancements |

---

## Future Considerations (Phase 10+)

Potential future enhancements (not yet tracked as issues):
- Multi-monitor support improvements
- Activity logging
- Sound effects/feedback
- Additional overlay themes beyond initial customization

---

## Recently Completed

| Issue | Title | Date |
|-------|-------|------|
| #184 | Add automatic retry for transient CI failures | 2026-01-26 |
| #166 | Documentation update for iced migration | 2026-01-26 |
| #165 | Animated cat companion overlay | 2026-01-25 |
| #164 | Performance testing and optimization | 2026-01-25 |
| #179 | Theme-aware light mode colors | 2026-01-25 |
| #163 | Visual polish and theming | 2026-01-25 |
| #162 | Linux tray integration | 2026-01-25 |
| #161 | Windows system tray integration | 2026-01-25 |
| #160 | macOS menu bar integration | 2026-01-25 |
| #159 | Add overlay customization (opacity, color) | 2026-01-25 |
| #158 | Implement settings persistence to disk | 2026-01-25 |
| #157 | Create settings window UI in iced | 2026-01-24 |
| #156 | Add timer countdown display to iced overlay | 2026-01-24 |
| #155 | Implement basic overlay window in iced | 2026-01-24 |
| #154 | Set up iced dependency and scaffold | 2026-01-24 |
| #153 | Research: iced integration feasibility - **GO** recommendation | 2026-01-24 |
| #114 | Animated cat companion overlay (migrated to #165 in iced epic) | 2026-01-24 |
| #150 | Add trace logging infrastructure with persistent file output | 2026-01-24 |
| #148 | Add 'Launch at Login' setting | 2026-01-23 |
| #139 | Windows overlay code quality improvements | 2026-01-23 |
| #132 | XWayland fallback detection and recommendation | 2026-01-23 |
| #113 | Platform-specific integration tests | 2026-01-23 |
| #131 | Wayland keyboard-shortcuts-inhibit protocol support | 2026-01-23 |
| #110 | Linux overlay window (X11 + Wayland wlr-layer-shell) | 2026-01-23 |
| #109 | Linux system tray via ksni (StatusNotifierItem) | 2026-01-22 |
| #105 | Windows entry point and event loop integration | 2026-01-19 |
| #103 | Windows overlay window implementation | 2026-01-15 |
| #102 | Windows system tray implementation | 2026-01-15 |
| #108 | Linux power management via DBus | 2026-01-14 |
| #111 | Linux keycode mappings | 2026-01-14 |
| #106 | X11 keyboard grab implementation | 2026-01-14 |
| #107 | Wayland input blocking research | 2026-01-14 |
| #128 | Migrate to log crate | 2026-01-14 |
| #101 | Windows power management | 2026-01-14 |
| #100 | Windows keyboard hook | 2026-01-14 |
| #104 | Windows keycode mappings | 2026-01-14 |
| #112 | Cross-platform CI | 2026-01-14 |

*For full changelog, see [CHANGELOG.md](CHANGELOG.md)*

---

## Changelog

### 2026-01-26
- **Completed #184**: Add automatic retry for transient CI failures
  - Created `rerun-failed.yml` workflow that monitors CI and auto-retries failed jobs
  - Uses `actions/github-script@v7` to call `reRunWorkflowFailedJobs` API
  - Only retries on first failure (checks `run_attempt` to prevent infinite loops)
  - Updated CI workflow to use `nick-fields/retry@v3` for build/test steps
  - Added step-level retry (2 attempts, 10s wait) for clippy, build, and test commands
  - Increased job timeout from 20 to 25 minutes to accommodate retries
  - Handles transient Windows runner provisioning issues without manual intervention
- **Completed #166**: Documentation update for iced migration
  - Updated README with new features, CLI flags, config file format, and architecture diagram
  - Updated CHANGELOG with comprehensive Phase 9 (iced migration) documentation
  - Updated ROADMAP to mark Phase 9 as complete
  - Documented hybrid architecture: iced for UI, platform-native for input blocking
  - Added documentation for all new CLI flags: `--opacity`, `--color`, `--no-cat`, `-v`

### 2026-01-25
- **Completed #165**: Animated cat companion overlay
  - Created `cat_animation` module with `CatCompanion`, `CatPosition`, `CatAnimationState` types
  - Implemented idle, blinking, and sleeping animation states
  - Added smooth bobbing animation using sine wave interpolation (~30 FPS)
  - Added random blink intervals (3-6 seconds) for natural appearance
  - Cat companion shows emoji with state-based variation and caption text
  - Added `--no-cat` CLI flag to disable the cat companion
  - Added `show_cat` and `cat_position` config file settings
  - Integrated cat settings into Settings window (Overlay section)
  - Cat can be positioned in any corner: bottom-right (default), bottom-left, top-right, top-left
  - Falls back to static cat icon when cat animation is disabled
- **Completed #164**: Performance testing and optimization for iced UI
  - Added Criterion benchmark framework with performance test suite
  - Benchmarks for: timer formatting, duration parsing, config operations, exit key parsing
  - Created `perf` module with performance monitoring infrastructure
  - Added `PerformanceSnapshot`, `PerformanceStats`, `LatencyTimer`, `FrameTimer` types
  - Defined performance requirements: Idle CPU <2%, Memory <100MB
  - Optimized release profile: `codegen-units = 1`, `strip = true`
  - Benchmark results show sub-microsecond timer formatting (<100ns)
- **Completed #179**: Theme-aware light mode colors
  - Added missing colors to `colors_light` module (slider, progress, success/warning hover)
  - Updated all 24 style functions to use theme-aware color selection
  - Container styles: overlay, settings, card, elevated, input
  - Button styles: close, primary, secondary, ghost, danger, success, tab
  - Text styles: timer, secondary, muted, success, warning, danger, accent (added theme param)
  - Widget styles: text_input, slider, checkbox, pick_list, rule
- Updated open issues summary (3 total: 1 High, 2 Low)
- Updated critical path diagram to show #164 and #179 completion
