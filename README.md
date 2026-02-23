# Cat Shield 🐱🛡️

A cross-platform cat-proof screen overlay that protects your work from curious felines walking on your keyboard. Available for macOS, Windows, and Linux.

## Features

- **Fullscreen overlay protection** - Semi-transparent overlay blocks input while keeping your work visible
- **Animated cat companion** - A cute animated cat watches over your screen (can be disabled with `--no-cat`)
- **Customizable appearance** - Adjust overlay opacity (10-90%) and color (presets or hex colors)
- **Timer-based auto-exit** - Set a countdown timer to automatically deactivate protection
- **Custom exit key** - Configure your own keyboard shortcut to unlock (default: Cmd+Option+U on macOS)
- **Key allowlist** - Allow specific keys to pass through (e.g., media keys, Spotlight)
- **System tray/menu bar integration** - Quick access from macOS menu bar or Windows/Linux system tray
- **Settings persistence** - Your preferences are saved to a config file
- **Dark/light mode support** - Automatically adapts to your system theme
- **Cross-platform** - Native support for macOS, Windows, and Linux (X11, Wayland)

## Installation

### Quick Install (Recommended)

```bash
# Clone the repository
git clone https://github.com/taearls/catshield.git
cd catshield

# Run the install script
./install.sh
```

The install script will:
1. Check for Rust/Cargo (required)
2. Build the release binary
3. Install to `/usr/local/bin/catshield` (may require sudo)

### Custom Install Location

To install to a different location:

```bash
INSTALL_DIR=~/.local/bin ./install.sh
```

### Manual Installation

```bash
# Build the release binary
cargo build --release

# Copy to your preferred location
sudo cp target/release/cat_shield /usr/local/bin/catshield
sudo chmod +x /usr/local/bin/catshield
```

### Prerequisites

- **macOS 10.12+**
- **Rust 1.71+** - Install from [rustup.rs](https://rustup.rs)

### Uninstall

```bash
./uninstall.sh
```

### Auto-Start on Shell Startup

Add the following snippet to your shell configuration file (`~/.bashrc`, `~/.zshrc`, etc.) to launch Cat Shield automatically whenever you open a new terminal. If an instance is already running, it will be skipped.

```bash
# Start Cat Shield in the background (if not already running)
# Only launch in graphical sessions — skip for SSH/headless environments
if [ -n "$DISPLAY" ] || [ -n "$WAYLAND_DISPLAY" ]; then
  if [ -z "$SSH_TTY" ] && [ -z "$SSH_CONNECTION" ]; then
    if command -v catshield &>/dev/null && ! pgrep -x catshield &>/dev/null; then
      catshield &>/dev/null &
      disown
    fi
  fi
fi
```

After adding the snippet, restart your shell or source the config file:

```bash
source ~/.bashrc   # for Bash
source ~/.zshrc    # for Zsh
```

## Usage

```bash
# Start in menu bar/system tray mode
catshield

# Start with a timer (auto-exit after duration)
catshield --timer 30m
catshield -t 2h

# Start with custom exit key
catshield --exit-key "Cmd+Shift+Q"

# Customize overlay appearance
catshield --opacity 70          # 70% opacity (more opaque)
catshield --color blue          # Blue preset color
catshield --color "#2a3f5f"     # Custom hex color

# Disable the animated cat companion
catshield --no-cat

# Enable verbose logging (for troubleshooting)
catshield -v                    # Info level
catshield -vv                   # Debug level
catshield -vvv                  # Trace level

# Combine options
catshield --timer 1h --hide-timer --opacity 60 --color green

# Show help
catshield --help
```

## Configuration File

Settings are persisted in `~/.config/catshield/config.toml`:

```toml
# Custom exit key combination
exit_key = "Cmd+Option+U"

# Default auto-exit timer (e.g., "30m", "1h", "2h30m")
default_timer = "30m"

# Overlay opacity (0.1 to 0.9, default 0.5)
overlay_opacity = 0.5

# Overlay color: preset name or hex code
# Presets: "gray", "blue", "green", "red", "purple"
# Hex format: "#RRGGBB" or "#RGB"
overlay_color = "gray"

# Keys allowed to pass through the shield
allowed_keys = ["Cmd+Space", "F11", "F12"]

# Launch Cat Shield automatically at login
launch_at_login = false

# Enable trace logging to file (~/.config/catshield/logs/)
enable_trace_logging = false

# Color scheme: "dark", "light", or "system" (follows OS preference)
color_scheme = "system"

# Show animated cat companion on overlay
show_cat = true

# Cat position: "bottom-right", "bottom-left", "top-right", "top-left"
cat_position = "bottom-right"
```

CLI arguments override config file settings for that session.

## Project Goals

This project aims to create a cross-platform utility written in Rust that:

1. **Protects laptops from cat interference** - Prevents accidental input when cats walk on the keyboard

2. **Maintains visibility** - Uses a semi-transparent overlay so you can still see your work

3. **Keeps the machine awake** - Prevents the display from sleeping during downloads or long-running tasks

4. **Provides quick recovery** - Configurable key combination to unlock and exit

5. **Works across platforms** - Native support for macOS, Windows, and Linux

## Core Requirements

- **Semi-transparent fullscreen overlay** - Borderless window with customizable opacity (10-90%)
- **Input blocking** - Platform-native event interception (CGEventTap, Win32 hooks, X11 grab)
- **Sleep prevention** - Platform-specific power assertions to prevent display sleep
- **Unlock mechanism** - Configurable keyboard shortcut to deactivate
- **Accessibility awareness** - Detect and warn about missing permissions (macOS)
- **Settings persistence** - TOML config file at `~/.config/catshield/config.toml`

## Technical Stack

- **Language**: Rust (1.71+)
- **UI Framework**: [iced](https://iced.rs) 0.14 (cross-platform GUI)
- **Platforms**: macOS 10.12+, Windows 10+, Linux (X11/Wayland)

### Architecture

Cat Shield uses a hybrid architecture combining the iced framework for UI rendering with platform-native APIs for input blocking:

```text
┌─────────────────────────────────────────────────────────────┐
│                      Cat Shield                             │
├─────────────────────────────────────────────────────────────┤
│                    iced UI Layer                            │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────┐  │
│  │   Overlay   │  │  Settings   │  │    About Window     │  │
│  │   Window    │  │   Window    │  │                     │  │
│  └─────────────┘  └─────────────┘  └─────────────────────┘  │
├─────────────────────────────────────────────────────────────┤
│              Platform Integration Layer                      │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────┐  │
│  │   macOS     │  │   Windows   │  │       Linux         │  │
│  │ NSStatusItem│  │  Shell_     │  │  StatusNotifierItem │  │
│  │             │  │ NotifyIcon  │  │       (ksni)        │  │
│  └─────────────┘  └─────────────┘  └─────────────────────┘  │
├─────────────────────────────────────────────────────────────┤
│                Platform-Native Input Blocking                │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────┐  │
│  │  CGEventTap │  │ WH_KEYBOARD │  │   X11 Grab /        │  │
│  │   (macOS)   │  │ _LL (Win32) │  │ Wayland Inhibitor   │  │
│  └─────────────┘  └─────────────┘  └─────────────────────┘  │
└─────────────────────────────────────────────────────────────┘
```

The iced framework provides:
- **Cross-platform overlay window** with semi-transparent background
- **Timer countdown display** with progress bar
- **Settings window** for configuring all options
- **Animated cat companion** with idle, blinking, and sleeping states
- **Theme support** with automatic dark/light mode detection

Input blocking remains platform-native for security:
- **macOS**: CGEventTap with Accessibility permissions
- **Windows**: Low-level keyboard hook (WH_KEYBOARD_LL)
- **Linux**: X11 keyboard grab or Wayland keyboard-shortcuts-inhibit protocol

### Dependencies

**Cross-platform:**
- `iced` 0.14 - UI framework
- `clap` - CLI argument parsing
- `serde` + `toml` - Configuration serialization
- `log` + `env_logger` - Logging infrastructure
- `dark-light` - System theme detection

**macOS:**
- `objc2` ecosystem - AppKit, CoreGraphics, IOKit bindings

**Windows:**
- `windows` crate - Win32 API bindings

**Linux:**
- `x11rb` - X11 protocol bindings
- `zbus` - D-Bus (for power management)
- `ksni` - StatusNotifierItem (system tray)
- `wayland-client` / `wayland-protocols` - Wayland support

## Platform Support

| Platform | Status | Input Blocking | Notes |
|----------|--------|----------------|-------|
| **macOS** | ✅ Full Support | Complete | Uses CGEventTap with Accessibility permissions |
| **Windows** | ✅ Full Support | Complete | Uses low-level keyboard hook (WH_KEYBOARD_LL) |
| **Linux (X11)** | ✅ Full Support | Complete | Uses X11 keyboard grab |
| **Linux (Wayland)** | ⚠️ Limited | Partial | See [Wayland Limitations](#wayland-limitations) |

### Wayland Limitations

Wayland's security model fundamentally restricts keyboard interception by arbitrary applications. This is **by design** for security reasons - preventing keyloggers and unauthorized input capture.

**What works on Wayland (wlroots compositors only):**
- Fullscreen overlay display via `wlr-layer-shell`
- Exit key detection when overlay is focused
- Allowed keys passthrough
- Visual cat protection

**What doesn't work on Wayland:**
- **Pointer input cannot be blocked** - Users (or cats) can click away from the overlay
- **GNOME/Mutter not supported** - No layer-shell or keyboard-shortcuts-inhibit protocol
- **Cannot truly prevent input to other windows** - Only captures keyboard when focused

**Recommendation**: For full input blocking on Linux, use X11 or the XWayland compatibility layer.

For detailed technical analysis, see [docs/WAYLAND_INPUT_RESEARCH.md](docs/WAYLAND_INPUT_RESEARCH.md).

## GitHub Actions Integration

This repository uses the official [Anthropic Claude Code GitHub Action](https://github.com/anthropics/claude-code-action) to provide AI-powered assistance on issues and pull requests.

### How to Use

Mention `@claude` in any issue comment, pull request comment, or pull request review to get Claude's assistance. For example:

- `@claude can you review this PR for potential bugs?`
- `@claude help me understand how the event tap works`
- `@claude implement error handling for the power assertion`

> **Note**: The workflow triggers on various events (issue creation, PR updates, comments, reviews), but Claude only responds when explicitly mentioned with `@claude`.

### Setup Requirements

To enable this feature, a repository administrator needs to:

1. Get an Anthropic API key from [platform.claude.com](https://platform.claude.com/)
2. Add it as a repository secret named `ANTHROPIC_API_KEY`:
   - Go to repository Settings → Secrets and variables → Actions
   - Click "New repository secret"
   - Name: `ANTHROPIC_API_KEY`
   - Value: Your Anthropic API key
3. The workflow file is already configured at `.github/workflows/claude.yml`
4. **Note**: Using this integration will consume Anthropic API credits. Monitor your usage at [platform.claude.com](https://platform.claude.com/) to track costs.

### Features

The Claude Code action can:
- Answer questions about the codebase
- Review pull requests for bugs and improvements
- Implement features and bug fixes
- Explain complex code sections
- Suggest architectural improvements

For more information, see the [Claude Code GitHub Actions documentation](https://github.com/anthropics/claude-code-action).