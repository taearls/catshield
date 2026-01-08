# Cat Shield 🐱🛡️

A cat-proof screen overlay for macOS that protects your work from curious felines walking on your keyboard.

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
- **Rust 1.70+** - Install from [rustup.rs](https://rustup.rs)

### Uninstall

```bash
./uninstall.sh
```

## Usage

```bash
# Start in menu bar mode
catshield

# Start with a timer (auto-exit after duration)
catshield --timer 30m
catshield -t 2h

# Start with custom exit key
catshield --exit-key "Cmd+Shift+Q"

# Combine options
catshield --timer 1h --hide-timer

# Show help
catshield --help
```

## Project Goals

This project aims to create a simple macOS utility written in Rust that:

1. **Protects laptops from cat interference** - Prevents accidental input when cats walk on the keyboard

2. **Maintains visibility** - Uses a semi-transparent overlay so you can still see your work

3. **Keeps the machine awake** - Prevents the display from sleeping during downloads or long-running tasks

4. **Provides quick recovery** - Simple key combination (Cmd+Option+U) to unlock and exit

5. **Demonstrates macOS/Rust integration** - Shows how to use macOS frameworks from Rust

## Core Requirements

- **Semi-transparent fullscreen overlay** - Borderless window at 30% opacity
- **Input blocking** - Intercept and block all keyboard/mouse events via CGEventTap
- **Sleep prevention** - Use IOKit power assertions to prevent display sleep
- **Unlock mechanism** - Cmd+Option+U combination to deactivate
- **Accessibility awareness** - Detect and warn about missing permissions

## Technical Stack

- **Language**: Rust (1.70+)
- **Platform**: macOS 10.12+
- **Frameworks**:
  - Cocoa/AppKit (window management)
  - CoreGraphics (event interception)
  - IOKit (power management)
  - CoreFoundation (run loop)

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