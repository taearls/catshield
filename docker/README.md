# Docker Development Environment

This directory contains Docker configurations for cross-platform development and validation of Cat Shield.

## Purpose

Enable macOS developers to validate code for Linux and Windows without requiring VMs or additional hardware.

## Quick Start

```bash
# From the project root directory:

# Run Linux unit tests
docker compose -f docker/docker-compose.yml run linux-test

# Run Linux linting
docker compose -f docker/docker-compose.yml run linux-clippy

# Check Windows compilation
docker compose -f docker/docker-compose.yml run windows-check

# Run all Linux validations
docker compose -f docker/docker-compose.yml up linux-test linux-clippy linux-fmt
```

## Available Services

### Linux Services

| Service | Command | Description |
|---------|---------|-------------|
| `linux-test` | `cargo test` | Run unit tests |
| `linux-clippy` | `cargo clippy` | Run linting checks |
| `linux-fmt` | `cargo fmt --check` | Check code formatting |
| `linux-build` | `cargo build --release` | Build release binary |

### Windows Services

| Service | Command | Description |
|---------|---------|-------------|
| `windows-check` | `cargo check` | Validate compilation |
| `windows-build` | `cargo build --release --target x86_64-pc-windows-gnu` | Cross-compile release binary |

## Prerequisites

- [Docker Desktop](https://www.docker.com/products/docker-desktop/) for macOS
- At least 4GB of available RAM for Docker

## Limitations

### Windows Containers on macOS

Docker Desktop for Mac runs a Linux VM, which means:
- **Windows containers cannot run** - only Linux containers work
- **Cross-compilation only** - we can build Windows binaries but not execute them
- **No runtime testing** - use GitHub Actions or a real Windows machine for that

### GUI/Integration Testing

Docker containers don't have display servers, so:
- **No overlay window testing** - requires real X11/Wayland
- **No system tray testing** - requires desktop environment
- **No input blocking testing** - requires keyboard hardware access

For full integration testing, use:
- GitHub Actions (runs on real Windows/Linux/macOS runners)
- Virtual machines (VirtualBox, UTM, Parallels)
- Real hardware

### Platform Compilation Status

Issue #99 (conditional platform dependencies) has been implemented. Current status:

- ✅ **Cargo.toml**: macOS dependencies are conditionally compiled
- ⚠️ **Source code**: The `ui` module still imports macOS-specific code unconditionally

**Expected behavior**:
- Linux/Windows `cargo check` will fail until the `ui` module is gated with `#[cfg(target_os = "macos")]`
- This is tracked in issues #103 (Windows overlay) and #110 (Linux overlay)
- Cross-platform modules (config, timer, keycodes) can be validated

## Caching

Docker Compose uses named volumes to cache:
- **Cargo registry** (`catshield-cargo-cache`) - shared across all containers
- **Linux target directory** (`catshield-target-linux`) - incremental builds
- **Windows target directory** (`catshield-target-windows`) - incremental builds

To clear caches:
```bash
docker volume rm catshield-cargo-cache catshield-target-linux catshield-target-windows
```

## Building Images

Images are built automatically on first run. To rebuild manually:

```bash
docker compose -f docker/docker-compose.yml build
```

## Troubleshooting

### "Cannot connect to Docker daemon"
Ensure Docker Desktop is running.

### Slow first build
The first build downloads base images and Rust dependencies. Subsequent runs use cached layers and volumes.

### Compilation errors on Linux/Windows
Check if issue #99 (conditional platform dependencies) has been implemented. Until then, platform-specific code may not compile.

### Out of disk space
Docker images and volumes can consume significant space. Clean up with:
```bash
docker system prune -a
docker volume prune
```
