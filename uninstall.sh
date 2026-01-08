#!/bin/bash
set -e

# Cat Shield Uninstall Script
# Removes Cat Shield from your system.

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Default installation directory
DEFAULT_INSTALL_DIR="/usr/local/bin"
INSTALL_DIR="${INSTALL_DIR:-$DEFAULT_INSTALL_DIR}"
BINARY_NAME="catshield"
BINARY_PATH="$INSTALL_DIR/$BINARY_NAME"
CONFIG_DIR="$HOME/.config/catshield"

echo "========================================="
echo "  Cat Shield Uninstaller"
echo "========================================="
echo ""

# Check if the binary exists
if [ ! -f "$BINARY_PATH" ]; then
    echo -e "${YELLOW}Cat Shield is not installed at $BINARY_PATH${NC}"
    echo ""

    # Check if it exists elsewhere in PATH
    if command -v "$BINARY_NAME" &> /dev/null; then
        FOUND_PATH=$(command -v "$BINARY_NAME")
        echo "However, it was found at: $FOUND_PATH"
        echo "To remove it, run:"
        echo "  sudo rm \"$FOUND_PATH\""
    fi
    echo ""
    exit 0
fi

# Confirm uninstallation
echo "This will remove:"
echo "  - Binary: $BINARY_PATH"
echo ""
read -p "Do you want to continue? [y/N] " -n 1 -r
echo ""

if [[ ! $REPLY =~ ^[Yy]$ ]]; then
    echo "Uninstallation cancelled."
    exit 0
fi

# Remove the binary
echo ""
echo -e "${GREEN}[1/2]${NC} Removing binary..."

if [ ! -w "$INSTALL_DIR" ]; then
    echo "      (requires sudo for $INSTALL_DIR)"
    sudo rm "$BINARY_PATH"
else
    rm "$BINARY_PATH"
fi

echo -e "      Removed: $BINARY_PATH"

# Ask about config directory
if [ -d "$CONFIG_DIR" ]; then
    echo ""
    echo -e "${GREEN}[2/2]${NC} Configuration directory found: $CONFIG_DIR"
    read -p "Do you want to remove the configuration directory? [y/N] " -n 1 -r
    echo ""

    if [[ $REPLY =~ ^[Yy]$ ]]; then
        rm -rf "$CONFIG_DIR"
        echo -e "      Removed: $CONFIG_DIR"
    else
        echo -e "      Kept: $CONFIG_DIR"
    fi
else
    echo ""
    echo -e "${GREEN}[2/2]${NC} No configuration directory found."
fi

echo ""
echo "========================================="
echo -e "  ${GREEN}Cat Shield uninstalled successfully!${NC}"
echo "========================================="
echo ""
