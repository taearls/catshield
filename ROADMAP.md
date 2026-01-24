# Cat Shield Roadmap

## Quick Reference
<!-- AI-OPTIMIZED: This section contains actionable info for automated tools -->
<!-- Keep under 150 lines - agents read this first -->

**Current Phase**: 9 - iced UI Migration
**Next Priority**: #153 (Research: iced integration feasibility)
**Blocking Issues**: None

### Open Issues by Priority

| # | Title | Priority | Effort | Blocked By | Phase |
|---|-------|----------|--------|------------|-------|
| 152 | Epic: Migrate UI Rendering to iced Framework | 🟡 High | ~4-6 weeks | None | 9 |
| 153 | Research: iced integration feasibility | 🟡 High | ~2-3 days | None | 9 |
| 154 | Set up iced dependency and scaffold | 🟡 High | ~1-2 days | #153 | 9 |
| 155 | Implement basic overlay window in iced | 🟡 High | ~3-5 days | #154 | 9 |
| 156 | Add timer countdown display | 🟢 Medium | ~1 day | #155 | 9 |
| 157 | Create settings window UI | 🟢 Medium | ~2-3 days | #155 | 9 |
| 158 | Implement settings persistence | 🟢 Medium | ~1-2 days | #157 | 9 |
| 159 | Add overlay customization | 🟢 Medium | ~1-2 days | #155, #158 | 9 |
| 160 | macOS menu bar integration | 🟢 Medium | ~1-2 days | #157 | 9 |
| 161 | Windows system tray integration | 🟢 Medium | ~1-2 days | #157 | 9 |
| 162 | Linux tray integration | 🟢 Medium | ~1-2 days | #157 | 9 |
| 163 | Visual polish and theming | 🔵 Low | ~2-3 days | #155, #157 | 9 |
| 164 | Performance testing | 🔵 Low | ~1-2 days | #155, #156 | 9 |
| 165 | Animated cat companion overlay | 🔵 Low | ~2-3 days | #155 | 9 |
| 166 | Documentation update | 🔵 Low | ~1 day | All others | 9 |

**Summary**: 15 open issues (0 Critical, 3 High, 7 Medium, 5 Low)

### Completed This Session

| # | Title | Priority | Date |
|---|-------|----------|------|
| 150 | Add trace logging infrastructure with persistent file output | 🟢 Medium | 2026-01-24 |
| 114 | Animated cat companion overlay (migrated to #165) | 🔵 Low | 2026-01-24 |

### Previously Completed

| # | Title | Priority | Date |
|---|-------|----------|------|
| 148 | Add 'Launch at Login' setting | 🟢 Medium | 2026-01-23 |
| 139 | Windows overlay code quality improvements | 🔵 Low | 2026-01-23 |

### Recommended Next Issues

1. **#153** - Research: iced integration feasibility (start here!)
2. **#154** - Set up iced dependency and scaffold (after #153)
3. **#155** - Implement basic overlay window (after #154)

---

## Current Sprint: Phase 9 - iced UI Migration

### Epic #152: Migrate UI Rendering to iced Framework

| Status | Issue | Title | Dependencies |
|--------|-------|-------|--------------|
| 📋 | #153 | Research: iced integration feasibility | None |
| 📋 | #154 | Set up iced dependency and scaffold | #153 |
| 📋 | #155 | Implement basic overlay window | #154 |
| 📋 | #156 | Add timer countdown display | #155 |
| 📋 | #157 | Create settings window UI | #155 |
| 📋 | #158 | Implement settings persistence | #157 |
| 📋 | #159 | Add overlay customization | #155, #158 |
| 📋 | #160 | macOS menu bar integration | #157 |
| 📋 | #161 | Windows system tray integration | #157 |
| 📋 | #162 | Linux tray integration | #157 |
| 📋 | #163 | Visual polish and theming | #155, #157 |
| 📋 | #164 | Performance testing | #155, #156 |
| 📋 | #165 | Animated cat companion overlay | #155 |
| 📋 | #166 | Documentation update | All |

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
#153 (Research)
    │
    ▼
#154 (Scaffold)
    │
    ▼
#155 (Basic Overlay) ─────────────────────────────────────────┐
    │                                                         │
    ├─► #156 (Timer) ──────────────────────────────┐          │
    │                                               │          │
    ├─► #157 (Settings UI) ─┬─► #158 (Persistence)─┤          │
    │                       │                       │          │
    │                       ├─► #160 (macOS)        │          │
    │                       ├─► #161 (Windows)      │          │
    │                       └─► #162 (Linux)        │          │
    │                                               │          │
    ├─► #163 (Polish) ◄─────────────────────────────┤          │
    │                                               │          │
    ├─► #164 (Performance) ◄────────────────────────┘          │
    │                                                          │
    ├─► #159 (Customization) ◄── #158                          │
    │                                                          │
    └─► #165 (Cat Animation) ◄─────────────────────────────────┘
                │
                ▼
        #166 (Documentation) ◄── All complete
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
| 9 | 🚧 In Progress | iced UI migration (Epic #152) |
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
