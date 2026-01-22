# Cat Shield Roadmap

## Quick Reference
<!-- AI-OPTIMIZED: This section contains actionable info for automated tools -->
<!-- Keep under 150 lines - agents read this first -->

**Current Phase**: 8 - Cross-Platform Support
**Next Priority**: #110 (Linux overlay window)
**Blocking Issues**: None

### Open Issues by Priority

| # | Title | Priority | Effort | Blocked By | Phase |
|---|-------|----------|--------|------------|-------|
| 110 | Linux overlay window (X11 + Wayland) | 🟢 Medium | ~2-3 days | None | 8.3 |
| 131 | Wayland keyboard-shortcuts-inhibit | 🟢 Medium | ~1 day | #110 | 8.3 |
| 132 | XWayland fallback detection | 🟢 Medium | ~0.5 day | #110, #131 | 8.3 |
| 113 | Platform-specific integration tests | 🟢 Medium | ~2 days | None | 8.4 |
| 114 | Animated cat companion overlay | 🔵 Low | ~2-3 days | None | 9 |

**Summary**: 5 open issues (0 Critical, 0 High, 4 Medium, 1 Low)

### Recommended Next Issues

1. **#110** - Linux overlay window (X11 + Wayland)
2. **#113** - Platform-specific integration tests
3. **#131** - Wayland keyboard-shortcuts-inhibit

---

## Current Sprint: Phase 8 - Cross-Platform Support

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
| 🔲 | #110 | Linux overlay window | None |
| 🔲 | #131 | Wayland shortcuts-inhibit | #110 |
| 🔲 | #132 | XWayland fallback | #110, #131 |

### Phase 8.4: CI & Testing

| Status | Issue | Title | Dependencies |
|--------|-------|-------|--------------|
| ✅ | #112 | Cross-platform CI | None |
| ✅ | #120 | Docker dev containers | None |
| 🔲 | #113 | Platform integration tests | None |

---

## Critical Path

```
Windows: Complete ✅
#100-#105 all implemented

Linux (remaining):
#109 (System Tray) ✅
#110 (Overlay) ─── #131 (Shortcuts Inhibit) ─── #132 (XWayland) ─── Linux Complete ✅

Future:
#114 (Animated Cat) ─── Phase 9 Complete
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
| 8 | 🔄 In Progress | Cross-platform support |
| 9 | 📋 Planned | Feature enhancements |

---

## Future Considerations

Potential future enhancements (not yet tracked as issues):
- Multi-monitor support improvements
- Auto-start on login option
- Activity logging
- Sound effects/feedback
- Custom overlay themes

---

## Recently Completed

| Issue | Title | Date |
|-------|-------|------|
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
