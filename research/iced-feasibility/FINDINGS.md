# iced Integration Feasibility Research Findings

**Issue**: #153 - Research: iced integration feasibility for catshield
**Date**: 2026-01-24
**Researcher**: Claude (automated research)

## Executive Summary

**Recommendation: GO** - iced is suitable for catshield's UI migration with some caveats.

iced 0.14 provides the core capabilities needed for catshield's overlay window:
- Borderless transparent windows with AlwaysOnTop level
- Cross-platform support (macOS, Windows, Linux)
- Timer/subscription system for animations
- Modern Elm-inspired architecture

However, there are important limitations and considerations documented below.

---

## 1. Window Management Capabilities

### 1.1 Borderless Windows
**Status: Supported**

```rust
WindowSettings {
    decorations: false,  // Removes title bar and borders
    ..
}
```

### 1.2 Transparent Windows
**Status: Supported with caveats**

```rust
WindowSettings {
    transparent: true,
    ..
}
```

- Transparency works via RGBA background colors in container styles
- wgpu backend requires premultiplied alpha; glow backend more reliable
- Semi-transparent overlays confirmed working in prototype

### 1.3 Fullscreen Windows
**Status: Supported**

```rust
// Via Mode enum
window::Mode::Fullscreen
window::Mode::Windowed
```

- Fullscreen covers entire monitor
- Can switch modes at runtime via `window::set_mode()`

### 1.4 AlwaysOnTop (Window Level)
**Status: Supported**

```rust
WindowSettings {
    level: Level::AlwaysOnTop,
    ..
}
```

- Three levels: `Normal`, `AlwaysOnBottom`, `AlwaysOnTop`
- Window stays above normal windows

### 1.5 Multi-Monitor Support
**Status: Partial**

- `monitor_size()` function returns logical dimensions of containing monitor
- Fullscreen mode covers current monitor only
- **Gap**: No built-in enumeration of all monitors for multi-window overlay
- **Workaround**: Could create multiple windows, one per monitor

---

## 2. Event Handling

### 2.1 Timer/Animation Subscriptions
**Status: Supported**

```rust
fn subscription(&self) -> Subscription<Message> {
    time::every(Duration::from_millis(100)).map(Message::Tick)
}
```

- Requires `tokio` or `smol` feature
- Declarative subscription model
- Sufficient for 60fps+ animations

### 2.2 Keyboard Events
**Status: Supported but separate from input blocking**

- iced handles keyboard events for UI interaction
- **Critical**: Input blocking must remain separate from iced
  - macOS: CGEventTap at HID level
  - Windows: Low-level keyboard hook
  - Linux: X11 grab or Wayland inhibitor
- iced cannot replace the input blocking infrastructure

### 2.3 Mouse Events
**Status: Supported**

- Standard widget interactions (buttons, hover, etc.)
- Custom event handling via subscriptions
- Mouse passthrough available: `enable_mouse_passthrough()` / `disable_mouse_passthrough()`

---

## 3. Rendering Capabilities

### 3.1 Custom Drawing (Canvas)
**Status: Supported**

- `iced::widget::Canvas` for custom rendering
- Can draw circles, shapes, text for close button
- Hardware-accelerated via wgpu or software via tiny-skia

### 3.2 Theming
**Status: Supported**

- Built-in themes (Dark, Light, etc.)
- Custom theme functions for styling
- Per-widget style overrides

### 3.3 Text Rendering
**Status: Supported**

- cosmic-text for advanced text shaping
- Font loading and sizing
- Proper Unicode support

---

## 4. Platform-Specific Considerations

### 4.1 macOS
**Status: Well supported**

- Native Cocoa integration via objc2 bindings
- Metal backend (wgpu)
- Transparent windows work correctly
- Main thread requirement handled by iced

### 4.2 Windows
**Status: Well supported**

- Native Win32 integration
- DX12/Vulkan backends
- Transparent windows work with layered window attributes
- AlwaysOnTop works correctly

### 4.3 Linux (X11)
**Status: Supported**

- Standard X11 window creation
- Vulkan backend
- Transparency via compositors (may require ARGB visual)

### 4.4 Linux (Wayland)
**Status: Requires additional crate**

- Standard iced uses XWayland or direct Wayland via winit
- For proper layer-shell overlay: use `iced_layershell` crate
- `iced_layershell` provides wlr-layer-shell protocol support
- Needed for overlays that work on wlroots compositors (Sway, Hyprland)

---

## 5. Performance Assessment

### 5.1 Idle Usage
- wgpu backend: GPU-accelerated, minimal CPU when idle
- tiny-skia fallback: CPU-based, higher idle usage
- Timer subscriptions add minimal overhead

### 5.2 Memory
- iced brings ~385 dependencies (significant but manageable)
- Runtime memory reasonable for GUI application

### 5.3 Binary Size
- Release binary: ~15-20MB (stripped)
- Acceptable for desktop application

---

## 6. Identified Blockers

### 6.1 No Blockers for Core Functionality

The essential features for catshield overlay work in iced:
- Borderless transparent fullscreen window
- AlwaysOnTop window level
- Timer-based animations
- Custom styling

### 6.2 Known Limitations (Non-Blocking)

| Limitation | Impact | Mitigation |
|-----------|--------|------------|
| Multi-monitor not automatic | Medium | Create window per monitor manually |
| Wayland layer-shell not built-in | Medium | Use `iced_layershell` crate |
| Input blocking external | None | Keep existing infrastructure |
| wgpu alpha handling quirks | Low | Use premultiplied alpha or tiny-skia |

---

## 7. Integration Architecture

### 7.1 Recommended Approach

```
┌─────────────────────────────────────────────────┐
│                  catshield                      │
├─────────────────────────────────────────────────┤
│ ┌─────────────────┐  ┌────────────────────────┐ │
│ │   iced UI       │  │  Platform Traits       │ │
│ │                 │  │                        │ │
│ │ - Overlay View  │  │ - InputBlocker (keep)  │ │
│ │ - Settings UI   │  │ - PowerManager (keep)  │ │
│ │ - Timer Display │  │ - PermissionChecker    │ │
│ │ - Close Button  │  │                        │ │
│ └────────┬────────┘  └────────────────────────┘ │
│          │                                      │
│          ▼                                      │
│ ┌─────────────────────────────────────────────┐ │
│ │           Shared State (Arc/Mutex)          │ │
│ │  - shield_active                            │ │
│ │  - remaining_seconds                        │ │
│ │  - settings                                 │ │
│ └─────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────┘
```

### 7.2 Files to Modify/Replace

| File | Action |
|------|--------|
| `src/ui/shield.rs` | Replace with iced overlay |
| `src/ui/state.rs` | Adapt for iced state management |
| `src/ui/views/` | Rewrite as iced widgets |
| `src/platform/*/overlay.rs` | Remove (iced handles this) |
| `src/platform/*/keyboard_hook.rs` | Keep unchanged |
| `src/platform/*/event_tap.rs` | Keep unchanged |
| `src/platform/*/power.rs` | Keep unchanged |

---

## 8. Prototype Results

A minimal prototype was built and tested successfully on macOS:

**Verified Capabilities:**
- [x] Borderless window (no title bar)
- [x] Transparent background (semi-transparent container)
- [x] AlwaysOnTop window level
- [x] Timer subscription (elapsed time display)
- [x] Button interactions
- [x] Custom styling (colored buttons)

**Prototype Location:** `research/iced-feasibility/`

---

## 9. Dependencies Analysis

### 9.1 Recommended iced Features

```toml
[dependencies]
iced = { version = "0.14", features = ["canvas", "tokio"] }

# For Wayland overlay support (Linux only)
[target.'cfg(target_os = "linux")'.dependencies]
iced_layershell = "0.13"
```

### 9.2 Conflicting Dependencies

None identified. iced uses modern Rust ecosystem crates compatible with catshield's existing dependencies.

---

## 10. Conclusion & Recommendation

### Go/No-Go: **GO**

iced is suitable for catshield's UI migration:

1. **Core requirements met**: All essential overlay features are supported
2. **Architecture clean**: Elm-inspired update/view pattern is maintainable
3. **Platform coverage**: Works on all target platforms
4. **Input blocking preserved**: Existing infrastructure remains unchanged
5. **Active development**: iced 0.14 released December 2025, actively maintained

### Next Steps

1. **#154**: Set up iced dependency and scaffold app structure
2. **#155**: Implement basic overlay window with iced
3. Migrate settings window to iced
4. Add system tray integration
5. Remove platform-specific overlay code

---

## References

- [iced Documentation](https://docs.rs/iced/0.14.0/iced/)
- [iced GitHub](https://github.com/iced-rs/iced)
- [iced Book](https://book.iced.rs/)
- [iced_layershell](https://docs.rs/iced_layershell)
- [Window Settings](https://docs.rs/iced/latest/iced/window/struct.Settings.html)
- [Window Level](https://docs.rs/iced/0.14.0/iced/window/enum.Level.html)
- [Transparent Window Issue #272](https://github.com/iced-rs/iced/issues/272)
