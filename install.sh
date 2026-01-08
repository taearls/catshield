#!/bin/bash
set -e

# Cat Shield Install Script
# Builds and installs Cat Shield to make it accessible from anywhere in your terminal.

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Default installation directory
DEFAULT_INSTALL_DIR="/usr/local/bin"
INSTALL_DIR="${INSTALL_DIR:-$DEFAULT_INSTALL_DIR}"
BINARY_NAME="catshield"
SOURCE_BINARY="target/release/cat_shield"

echo "========================================="
echo "  Cat Shield Installer"
echo "========================================="
echo ""

# Check for Rust/Cargo
if ! command -v cargo &> /dev/null; then
    echo -e "${RED}Error: Rust/Cargo is required but not installed.${NC}"
    echo ""
    echo "Install Rust from https://rustup.rs by running:"
    echo "  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
    echo ""
    exit 1
fi

echo -e "${GREEN}[1/4]${NC} Rust/Cargo detected: $(cargo --version)"

# Build release binary
echo ""
echo -e "${GREEN}[2/4]${NC} Building release binary..."
cargo build --release

if [ ! -f "$SOURCE_BINARY" ]; then
    echo -e "${RED}Error: Build failed. Binary not found at $SOURCE_BINARY${NC}"
    exit 1
fi

echo -e "      Build successful!"

# Check if install directory exists
if [ ! -d "$INSTALL_DIR" ]; then
    echo ""
    echo -e "${YELLOW}Warning: Install directory $INSTALL_DIR does not exist.${NC}"
    echo "Creating directory..."

    if [ "$INSTALL_DIR" = "$DEFAULT_INSTALL_DIR" ]; then
        sudo mkdir -p "$INSTALL_DIR"
    else
        mkdir -p "$INSTALL_DIR"
    fi
fi

# Install the binary
echo ""
echo -e "${GREEN}[3/4]${NC} Installing to $INSTALL_DIR/$BINARY_NAME..."

# Determine if we need sudo
NEED_SUDO=false
if [ ! -w "$INSTALL_DIR" ]; then
    NEED_SUDO=true
fi

if [ "$NEED_SUDO" = true ]; then
    echo "      (requires sudo for $INSTALL_DIR)"
    sudo cp "$SOURCE_BINARY" "$INSTALL_DIR/$BINARY_NAME"
    sudo chmod +x "$INSTALL_DIR/$BINARY_NAME"
else
    cp "$SOURCE_BINARY" "$INSTALL_DIR/$BINARY_NAME"
    chmod +x "$INSTALL_DIR/$BINARY_NAME"
fi

# Verify installation
echo ""
echo -e "${GREEN}[4/4]${NC} Verifying installation..."

if command -v "$BINARY_NAME" &> /dev/null; then
    INSTALLED_PATH=$(command -v "$BINARY_NAME")
    echo -e "      ${GREEN}Success!${NC} Installed at: $INSTALLED_PATH"
else
    # Check if INSTALL_DIR is in PATH
    if [[ ":$PATH:" != *":$INSTALL_DIR:"* ]]; then
        echo -e "${YELLOW}Note: $INSTALL_DIR is not in your PATH.${NC}"
        echo ""
        echo "Add it to your shell profile (.bashrc, .zshrc, etc.):"
        echo "  export PATH=\"\$PATH:$INSTALL_DIR\""
        echo ""
        echo "Then restart your terminal or run:"
        echo "  source ~/.bashrc  # or ~/.zshrc"
    fi
fi

echo ""
echo "========================================="
echo -e "  ${GREEN}Cat Shield installed successfully!${NC}"
echo "========================================="
echo ""
echo "Usage:"
echo "  catshield              # Start in menu bar mode"
echo "  catshield --timer 30m  # Start with 30 minute timer"
echo "  catshield -t 2h        # Start with 2 hour timer"
echo "  catshield --help       # Show all options"
echo ""
echo "To uninstall, run: ./uninstall.sh"
echo ""
