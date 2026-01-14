# Wayland Input Blocking Research

**Issue**: #107
**Date**: 2026-01-14
**Status**: Research Complete

## Executive Summary

Wayland's security model fundamentally restricts keyboard interception by arbitrary applications. This is **by design** for security reasons - preventing keyloggers and unauthorized input capture. After investigating available protocols and compositor-specific solutions, we conclude that **full input blocking as implemented on macOS/X11 is not currently possible** on Wayland for a user-space application like Cat Shield.

However, there are partial solutions and future possibilities documented below.

## Background

Cat Shield's core functionality requires:
1. **Global keyboard interception** - Capture all keyboard input system-wide
2. **Selective event blocking** - Block most keys, allow specific exit combinations
3. **Non-privileged execution** - Run as a normal user application

On macOS, this is achieved via `CGEventTap` with accessibility permissions. On X11, this uses `XGrabKeyboard`. Wayland's security model explicitly prohibits these patterns for regular applications.

## Wayland Protocols Investigated

### 1. zwp_input_method_v2

**Purpose**: Enables input methods (like CJK text input) to intercept keyboard events.

**How it works**:
- Provides `zwp_input_method_keyboard_grab_v2` for exclusive keyboard grab
- Compositor sends keyboard events to the grab holder
- Used by input method editors (fcitx, ibus) for text composition

**Limitations for Cat Shield**:
- Only activated when a text input surface is focused
- Designed for text composition, not general input blocking
- Cannot block input to non-text-input windows
- Compositor may decide not to forward specific events

**Verdict**: **Not suitable** - Protocol is designed for input methods, not screen/keyboard protection.

**References**:
- [Input method v2 protocol | Wayland Explorer](https://wayland.app/protocols/input-method-unstable-v2)
- [wayland-protocols input-method-v2 RFC](https://lists.freedesktop.org/archives/wayland-devel/2018-August/039255.html)

### 2. wlr-input-inhibitor-unstable-v1

**Purpose**: Allows a client to inhibit input to other clients (designed for screen lockers).

**How it works**:
- While active, other clients receive no input events
- Previous focused clients receive `leave` events
- Compositor disables its own shortcuts during inhibition

**Critical Issue**:
> "This protocol is **deprecated and not intended for production use.** Use the ext-session-lock-v1 protocol instead."

**Limitations**:
- Deprecated with no active compositor support documented
- Security concerns: input can still fall through in some implementations
- Mouse actions not fully inhibited in all implementations
- Single inhibitor limit - only one client can inhibit at a time

**Verdict**: **Not viable** - Deprecated protocol with security issues.

**References**:
- [wlr-input-inhibitor protocol | Wayland Explorer](https://wayland.app/protocols/wlr-input-inhibitor-unstable-v1)
- [wlr-protocols repository](https://github.com/swaywm/wlr-protocols/blob/master/unstable/wlr-input-inhibitor-unstable-v1.xml)

### 3. ext-session-lock-v1

**Purpose**: Secure session locking with arbitrary graphics.

**How it works**:
- Privileged client locks the session
- Compositor stops rendering and providing input to normal clients
- All outputs blanked with opaque color
- Client handles authentication and unlock

**Security Model**:
- If lock client crashes, session remains locked (unlike swaylock with older protocols)
- Race condition prevention for suspend/unlock timing
- Compositor controls access to privileged clients

**Limitations for Cat Shield**:
- **Compositor-managed access** - Only available to privileged clients
- **Session-wide lock** - Cannot allow partial input (our allowed keys feature)
- **No selective passthrough** - All input blocked or none
- **Intended for screen lockers** - Not general-purpose input control

**Verdict**: **Not suitable** - Designed for full session locking, not selective input blocking.

**References**:
- [ext-session-lock-v1 protocol | Wayland Explorer](https://wayland.app/protocols/ext-session-lock-v1)
- [waylock - secure Wayland screenlocker](https://github.com/ifreund/waylock)

### 4. keyboard-shortcuts-inhibit-unstable-v1

**Purpose**: Allow a surface to receive all keyboard input (for VMs, remote desktop).

**How it works**:
- When surface has focus, it receives all key events
- Compositor's shortcuts are bypassed
- User can deactivate via pointer to remove focus

**Limitations for Cat Shield**:
- **Requires focus** - Only works when our window has keyboard focus
- **User can circumvent** - Clicking elsewhere deactivates inhibitor
- **Compositor discretion** - Compositor may keep some key combos

**Verdict**: **Partially viable** - Could work with a fullscreen overlay, but user can bypass with pointer.

**References**:
- [Keyboard shortcuts inhibit protocol | Wayland Explorer](https://wayland.app/protocols/keyboard-shortcuts-inhibit-unstable-v1)
- [Keyboard shortcuts inhibit PR for wlroots](https://github.com/swaywm/wlroots/pull/2026)

### 5. xdg-desktop-portal GlobalShortcuts

**Purpose**: Register global shortcuts via desktop portal system.

**How it works**:
- Applications register shortcuts through D-Bus portal
- Works regardless of window focus
- Desktop environment handles the actual capture

**Limitations for Cat Shield**:
- **Registration, not blocking** - Only notifies when shortcuts are pressed
- **Cannot intercept all keys** - Only registered shortcuts
- **Limited by portal backend** - Support varies by desktop environment
- **No blocking capability** - Other windows still receive input

**Verdict**: **Not suitable** - Designed for triggering actions, not blocking input.

**References**:
- [GlobalShortcuts Portal documentation](https://flatpak.github.io/xdg-desktop-portal/docs/doc-org.freedesktop.portal.GlobalShortcuts.html)
- [GlobalShortcuts portal issue](https://github.com/flatpak/xdg-desktop-portal/issues/624)

### 6. zwp_virtual_keyboard_v1

**Purpose**: Create virtual keyboard devices for input injection.

**How it works**:
- Creates a virtual keyboard associated with a seat
- Can emulate key events to the compositor

**Limitations**:
- **Input injection, not blocking** - Adds events, doesn't remove them
- **Requires compositor trust** - May be rejected by security-conscious compositors

**Verdict**: **Not applicable** - Solves the opposite problem.

## Compositor-Specific Solutions

### KDE Plasma / KWin

**Approach**: KGlobalAccel integrated into KWin compositor.

**Status**:
- Global shortcuts work via compositor integration
- No external client API for input blocking
- KDE explicitly declined to standardize global shortcuts protocol with GNOME

**Cat Shield Viability**: Would require KWin plugin/extension - impractical for standalone app.

**References**:
- [Global shortcut handling in Plasma Wayland](https://blog.martin-graesslin.com/blog/2015/06/global-shortcut-handling-in-a-plasma-wayland-session/)
- [Plasma/Wayland Known Issues](https://community.kde.org/Plasma/Wayland_Showstoppers)

### GNOME / Mutter

**Approach**: RemoteDesktop portal for input simulation.

**Status**:
- Input injection mediated through portal with user permission
- No API for arbitrary input blocking
- Accessibility concerns acknowledged but not fully addressed

**Cat Shield Viability**: RemoteDesktop portal for injection, but no blocking capability.

**References**:
- [GNOME Wayland Remote Desktop challenges](https://discourse.gnome.org/t/solved-remote-desktop-portal-cannot-simulate-mouse-button-press-on-wayland/21455)

### Sway / wlroots

**Approach**: Various protocols in wlr-protocols repository.

**Protocols supported**:
- `keyboard-shortcuts-inhibit` - For VMs/remote desktop
- `ext-session-lock-v1` - For screen lockers
- `zwp_input_method_v2` - For input methods

**Cat Shield Viability**:
- Best option: Fullscreen layer-shell surface + keyboard-shortcuts-inhibit
- Limitation: User can click away to escape

**References**:
- [Sway input-method issues](https://github.com/swaywm/sway/issues/8143)
- [wlroots keyboard grabs routing](https://github.com/swaywm/wlroots/issues/2322)

### Hyprland

**Status**: Supports xdg-desktop-portal-hyprland with GlobalShortcuts.

**Cat Shield Viability**: Same limitations as GlobalShortcuts portal.

## Potential Implementation Approaches

### Approach A: Fullscreen Layer Shell + Keyboard Shortcuts Inhibit

**How it would work**:
1. Create fullscreen overlay using `wlr-layer-shell`
2. Request `keyboard-shortcuts-inhibit` for the surface
3. When overlay has focus, receive all keyboard events
4. Block by not forwarding events

**Pros**:
- Works on wlroots-based compositors (Sway, River, Wayfire, Hyprland)
- Legitimate use of existing protocols

**Cons**:
- User can escape by clicking (pointer not inhibited)
- Not available on GNOME/Mutter
- Compositor may keep some shortcuts

**Implementation Effort**: Medium (~2-3 days)

### Approach B: Run as Privileged Screen Locker

**How it would work**:
1. Implement `ext-session-lock-v1` protocol
2. Register as the session's screen locker
3. Full input blocking during lock

**Pros**:
- True input blocking on supporting compositors
- Secure, crash-safe implementation

**Cons**:
- **Cannot allow selective key passthrough** (breaks allowed_keys feature)
- Conflicts with user's actual screen locker
- May require special installation/privileges
- All-or-nothing blocking

**Implementation Effort**: Medium-High (~3-4 days)

### Approach C: Compositor Plugin/Extension

**How it would work**:
- KWin script/plugin for KDE
- GNOME Shell extension
- wlroots compositor modifications

**Pros**:
- Maximum control over input handling
- Could implement exact desired behavior

**Cons**:
- **Different code for each compositor** - maintenance nightmare
- Requires deep compositor integration
- Complex installation for users
- May require root/system modifications

**Implementation Effort**: Very High (weeks per compositor)

## Recommendations

### Short-Term: Implement X11 Support First

Since X11 remains widely used (especially on systems requiring accessibility features), implementing X11 input blocking (#106) provides immediate value for Linux users.

### Medium-Term: Approach A for wlroots Compositors

For Wayland, implement the layer-shell + keyboard-shortcuts-inhibit approach:
- Cover Sway, River, Wayfire, Hyprland users
- Accept the limitation that pointer input can escape
- Document this limitation clearly

### Long-Term: Monitor Protocol Development

Watch for:
- Evolution of `ext-session-lock-v1` to allow partial input passthrough
- New protocols for application-level input control
- Desktop portal developments for input blocking

### Alternative: Position as "Focus Protection"

Reframe Wayland support not as "input blocking" but as "focus protection":
- Fullscreen overlay captures attention
- Keyboard shortcuts inhibit prevents accidental commands
- Explicit exit mechanism (our existing hold-to-close button)
- Accept that determined users (or cats) can click away

## Conclusion

**Full input blocking on Wayland is not possible** with current protocols for a user-space application. This is a deliberate security decision in Wayland's design.

**Partial solutions exist** for wlroots-based compositors using keyboard-shortcuts-inhibit, but:
- Pointer input cannot be blocked (user can click away)
- GNOME and some other compositors don't support this
- Our allowed_keys feature works (we control what we forward)
- Exit key detection works (we receive all keyboard input when focused)

**Recommended path forward**:
1. Document Wayland limitations clearly in README
2. Implement X11 support for full Linux functionality
3. Implement partial Wayland support for wlroots compositors
4. Clearly communicate the UX differences to users

## References

### Protocol Documentation
- [Wayland Protocol Documentation | Wayland Explorer](https://wayland.app/protocols/)
- [ext-session-lock-v1](https://wayland.app/protocols/ext-session-lock-v1)
- [keyboard-shortcuts-inhibit](https://wayland.app/protocols/keyboard-shortcuts-inhibit-unstable-v1)
- [wlr-input-inhibitor (deprecated)](https://wayland.app/protocols/wlr-input-inhibitor-unstable-v1)
- [input-method-v2](https://wayland.app/protocols/input-method-unstable-v2)

### Compositor Resources
- [Sway Wiki](https://wiki.archlinux.org/title/Sway)
- [Plasma/Wayland Known Issues](https://community.kde.org/Plasma/Wayland_Showstoppers)
- [xdg-desktop-portal GlobalShortcuts](https://flatpak.github.io/xdg-desktop-portal/docs/doc-org.freedesktop.portal.GlobalShortcuts.html)

### Screen Locker Implementations
- [waylock](https://github.com/ifreund/waylock) - ext-session-lock-v1 implementation
- [swaylock](https://github.com/swaywm/swaylock) - Supports both protocols

### Accessibility Discussions
- [Accessibility Shortcuts Portal Proposal](https://github.com/flatpak/xdg-desktop-portal/issues/1046)
- [XDG Global Keybinds Portal in GNOME](https://discussion.fedoraproject.org/t/xdg-global-keybinds-portal-in-gnome/121019)
