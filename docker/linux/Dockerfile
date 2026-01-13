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

FROM rust:1.83-slim-bookworm

# Install build dependencies
# - pkg-config: Required for linking native libraries
# - libx11-dev, libxcb1-dev: X11 development headers for future Linux GUI support
# - libdbus-1-dev: DBus for future Linux power management
RUN apt-get update && apt-get install -y --no-install-recommends \
    pkg-config \
    libx11-dev \
    libxcb1-dev \
    libdbus-1-dev \
    && rm -rf /var/lib/apt/lists/*

# Set working directory
WORKDIR /app

# Set environment variables for better Rust experience
ENV CARGO_HOME=/usr/local/cargo
ENV RUSTUP_HOME=/usr/local/rustup
ENV PATH="${CARGO_HOME}/bin:${PATH}"

# Install additional Rust components
RUN rustup component add clippy rustfmt

# Default command: run tests
CMD ["cargo", "test", "--verbose"]
