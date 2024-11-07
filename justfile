# Use environment variables in all recipes
set dotenv-load

# List available commands
default:
    @just --list

# Ensure environment variables are set for cross compilation
export-vars:
    #!/usr/bin/env bash
    if [[ "$OSTYPE" == "darwin"* ]]; then
        # Create or update .env file
        ENV_FILE=".env"
        echo "# Auto-generated environment variables for learner build" > $ENV_FILE
        echo "OPENSSL_DIR=$(brew --prefix openssl@3)" >> $ENV_FILE
        echo "CC_x86_64_unknown_linux_gnu=x86_64-linux-gnu-gcc" >> $ENV_FILE
        echo "AR_x86_64_unknown_linux_gnu=x86_64-linux-gnu-ar" >> $ENV_FILE
    fi

# Install required system dependencies
install-deps:
    #!/usr/bin/env bash
    if [[ "$OSTYPE" == "linux-gnu"* ]]; then
        if command -v apt-get &> /dev/null; then
            sudo apt-get update
            sudo apt-get install -y pkg-config libssl-dev
        elif command -v dnf &> /dev/null; then
            sudo dnf install -y pkgconfig openssl-devel
        elif command -v pacman &> /dev/null; then
            sudo pacman -Sy pkg-config openssl
        else
            echo "Warning: Unsupported Linux distribution. Please install OpenSSL development packages manually."
        fi
    elif [[ "$OSTYPE" == "darwin"* ]]; then
        if ! command -v brew &> /dev/null; then
            echo "Homebrew not found. Please install from https://brew.sh/"
            exit 1
        fi
        brew install openssl@3 gcc-x86-64-linux-gnu
    fi

# Install required Rust targets
install-targets:
    rustup target add x86_64-unknown-linux-gnu aarch64-apple-darwin

# Setup complete development environment
setup: install-deps install-targets export-vars
    @echo "Development environment setup complete!"

# Build for all targets
build-all: build-x86-linux build-arm-mac

# Build for x86_64 Linux
build-x86-linux:
    cargo build --target x86_64-unknown-linux-gnu

# Build for ARM64 macOS
build-arm-mac:
    cargo build --target aarch64-apple-darwin

# Run all tests
test:
    cargo test --workspace --all-targets

# Run clippy on all targets
lint:
    cargo clippy --workspace --all-targets --all-features

# Format all code
fmt:
    cargo fmt --all
    taplo fmt

# Clean build artifacts
clean:
    cargo clean

# Check code without building
check:
    cargo check --workspace --all-targets

# Run full CI checks locally
ci: fmt lint test build-all
    @echo "All CI checks passed!"

# Update dependencies
update:
    cargo update

# Show current platform info
info:
    @echo "OS: $OSTYPE"
    @echo "Rust version:"
    @rustc --version
    @echo "Cargo version:"
    @cargo --version
    @echo "Installed targets:"
    @rustup target list --installed
    @echo "Environment variables:"
    @echo "OPENSSL_DIR=$OPENSSL_DIR"
    @echo "CC_x86_64_unknown_linux_gnu=$CC_x86_64_unknown_linux_gnu"
    @echo "AR_x86_64_unknown_linux_gnu=$AR_x86_64_unknown_linux_gnu"