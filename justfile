# List available commands
default:
    @just --list

# Install required system dependencies
install-deps:
    #!/usr/bin/env bash
    if [[ "$OSTYPE" == "darwin"* ]]; then
        brew install filosottile/musl-cross/musl-cross
    elif [[ "$OSTYPE" == "linux-gnu"* ]]; then
        if command -v apt-get &> /dev/null; then
            sudo apt-get update
            sudo apt-get install -y musl-tools
        elif command -v dnf &> /dev/null; then
            sudo dnf install -y musl-gcc
        elif command -v pacman &> /dev/null; then
            sudo pacman -Sy musl
        fi
    fi

# Install required Rust targets
install-targets:
    rustup target add x86_64-unknown-linux-musl aarch64-apple-darwin

# Setup complete development environment
setup: install-deps install-targets
    @echo "Development environment setup complete!"

# Build native target (lib, tests, examples, etc)
build:
    cargo build --workspace --all-targets

# Build all platforms
build-all: build-mac build-linux
    @echo "All platform builds completed!"

# Build macOS ARM64
build-mac:
    @echo "Building macOS ARM64..."
    cargo build --workspace --target aarch64-apple-darwin

# Build Linux x86_64
build-linux:
    @echo "Building Linux x86_64..."
    cargo build --workspace --target x86_64-unknown-linux-musl

# Test everything
test:
    cargo test --workspace --all-targets

# Lint all code
lint:
    cargo clippy --workspace --all-targets --all-features

# Format code
fmt:
    cargo fmt --all
    taplo fmt

# Check unused dependencies
udeps:
    cargo +nightly udeps --workspace

# Clean build artifacts
clean:
    cargo clean

# Run all CI checks
ci: fmt lint test build-all
    @echo "All CI checks passed!"

# Show environment info
info:
    @echo "OS: $OSTYPE"
    @rustc --version
    @cargo --version
    @echo "Installed targets:"
    @rustup target list --installed