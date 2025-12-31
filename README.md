# Cat Shield 🐱🛡️

A cat-proof screen overlay for macOS that protects your work from curious felines walking on your keyboard.

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