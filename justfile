# KeyRx Task Runner
# Usage: just <recipe>

# Default recipe - list all available commands
default:
    @just --list

# Setup development environment (install tools, dependencies, and hooks)
setup:
    @echo "Installing Rust toolchain components..."
    rustup component add clippy rustfmt
    @echo "Installing development tools..."
    cargo install cargo-nextest cargo-watch --locked || true
    @echo "Installing Rust dependencies..."
    cd core && cargo fetch
    @echo "Installing Flutter dependencies..."
    cd ui && flutter pub get
    @echo "Installing git hooks..."
    ./scripts/install-hooks.sh
    @echo "Setup complete!"

# Run core in development mode with auto-reload
dev:
    cd core && cargo watch -x run

# Run Flutter UI in development mode
ui:
    cd ui && flutter run -d linux

# Run all quality checks (fmt, clippy, test)
check: fmt-check clippy test
    @echo "All checks passed!"

# Format all Rust code
fmt:
    cd core && cargo fmt

# Check Rust formatting (no changes)
fmt-check:
    cd core && cargo fmt --check

# Run clippy linter with warnings as errors
clippy:
    cd core && cargo clippy -- -D warnings

# Run all tests
test:
    cd core && cargo nextest run

# Run tests with standard cargo (fallback)
test-cargo:
    cd core && cargo test

# Run benchmarks
bench:
    cd core && cargo bench

# Clean build artifacts
clean:
    cd core && cargo clean
    cd ui && flutter clean

# Build release binary for current platform
build:
    cd core && cargo build --release

# Build for all supported platforms
build-all: build-linux build-windows
    @echo "All platform builds complete!"

# Build release binary for Linux x86_64
build-linux:
    cd core && cargo build --release --target x86_64-unknown-linux-gnu

# Build release binary for Windows x86_64 (requires cross)
build-windows:
    cd core && cross build --release --target x86_64-pc-windows-msvc

# Create a new release with the specified version
release version:
    ./scripts/release.sh {{version}}
