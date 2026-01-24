# Linux Development Environment for Cat Shield
#
# This Dockerfile provides a Linux environment for:
# - Running unit tests
# - Running clippy linting
# - Format checking
# - Building for Linux target
#
# Note: Full compilation requires issue #99 (conditional platform dependencies)
# to be implemented first. Until then, only cross-platform code can be validated.

FROM rust:1.88-slim-bookworm

# Install build dependencies
# - pkg-config: Required for linking native libraries
# - libx11-dev, libxcb1-dev: X11 development headers for Linux GUI support
# - libdbus-1-dev: DBus for Linux power management
# - libwayland-dev: Wayland client library for iced
# - libxkbcommon-dev: Keyboard handling for iced/winit
# - libvulkan-dev: Vulkan graphics backend for wgpu
# - libssl-dev: Required by some iced dependencies
RUN apt-get update && apt-get install -y --no-install-recommends \
    pkg-config \
    libx11-dev \
    libxcb1-dev \
    libdbus-1-dev \
    libwayland-dev \
    libxkbcommon-dev \
    libvulkan-dev \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Set working directory
WORKDIR /app

# Install additional Rust components
RUN rustup component add clippy rustfmt

# Default command: run tests
CMD ["cargo", "test", "--verbose"]
