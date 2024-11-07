# Use environment variables in all recipes
set dotenv-load

# List available commands
default:
    @just --list

# Install required system dependencies
install-deps:
    #!/usr/bin/env bash
    if [[ "$OSTYPE" == "darwin"* ]]; then
        brew install openssl@3
        # The correct package name for cross-compilation on macOS
        brew install filosottile/musl-cross/musl-cross
    elif [[ "$OSTYPE" == "linux-gnu"* ]]; then
        if command -v apt-get &> /dev/null; then
            sudo apt-get update
            sudo apt-get install -y pkg-config libssl-dev
        elif command -v dnf &> /dev/null; then
            sudo dnf install -y pkgconfig openssl-devel
        elif command -v pacman &> /dev/null; then
            sudo pacman -Sy pkg-config openssl
        fi
    fi

# Configure environment for cross-compilation
setup-env:
    #!/usr/bin/env bash
    if [[ "$OSTYPE" == "darwin"* ]]; then
        echo "# Build configuration for learner" > .env
        echo "OPENSSL_DIR=$(brew --prefix openssl@3)" >> .env
        echo "OPENSSL_INCLUDE_DIR=$(brew --prefix openssl@3)/include" >> .env
        echo "OPENSSL_LIB_DIR=$(brew --prefix openssl@3)/lib" >> .env
        echo "TARGET_CC=x86_64-linux-musl-gcc" >> .env
    fi

# Install required Rust targets
install-targets:
    rustup target add x86_64-unknown-linux-musl aarch64-apple-darwin

# Setup complete development environment
setup: install-deps install-targets setup-env
    @echo "Development environment setup complete!"

# Build for current platform
build:
    cargo build

# Build for x86_64 Linux (using musl for better compatibility)
build-linux:
    cargo build --target x86_64-unknown-linux-musl

# Run tests
test:
    cargo test --workspace --all-targets

# Run clippy
lint:
    cargo clippy --workspace --all-targets --all-features

# Format code
fmt:
    cargo fmt --all
    taplo fmt

# Clean build artifacts
clean:
    cargo clean

# Show environment info
info:
    @echo "OS: $OSTYPE"
    @rustc --version
    @cargo --version
    @echo "Installed targets:"
    @rustup target list --installed
    @if [ -f .env ]; then cat .env; fi