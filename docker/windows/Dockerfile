# Windows Cross-Compilation Environment for Cat Shield
#
# This Dockerfile provides a Linux environment for:
# - Cross-compiling to Windows targets
# - Validating Windows code compiles correctly
#
# IMPORTANT LIMITATIONS:
# - Cannot execute Windows binaries (no Windows runtime)
# - Only validates compilation, not runtime behavior
# - Full compilation requires issue #99 (conditional platform dependencies)
#
# For actual Windows testing, use:
# - GitHub Actions with windows-latest runner
# - A real Windows VM or machine

FROM rust:1.83-slim-bookworm

# Install mingw-w64 for Windows cross-compilation
RUN apt-get update && apt-get install -y --no-install-recommends \
    mingw-w64 \
    && rm -rf /var/lib/apt/lists/*

# Set working directory
WORKDIR /app

# Set environment variables
ENV CARGO_HOME=/usr/local/cargo
ENV RUSTUP_HOME=/usr/local/rustup
ENV PATH="${CARGO_HOME}/bin:${PATH}"

# Add Windows GNU target
RUN rustup target add x86_64-pc-windows-gnu

# Configure cargo to use mingw-w64 linker for Windows target
RUN mkdir -p /usr/local/cargo && \
    echo '[target.x86_64-pc-windows-gnu]' >> /usr/local/cargo/config.toml && \
    echo 'linker = "x86_64-w64-mingw32-gcc"' >> /usr/local/cargo/config.toml

# Default command: check compilation for Windows
CMD ["cargo", "check", "--target", "x86_64-pc-windows-gnu"]
