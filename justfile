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

# Run Flutter analysis
ui-check:
    cd ui && flutter analyze

# Verify Flutter build (Windows)
ui-verify: ui-check
    cd ui && flutter build windows --debug

# Run all quality checks (fmt, clippy, test, docs, bindings)
check: fmt-check clippy test docs verify-bindings
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

# Generate error documentation from registry
docs-errors:
    @echo "Generating error documentation..."
    cd core && cargo run --bin generate_error_docs

# Generate API documentation from DocRegistry
docs-api:
    @echo "Generating API documentation..."
    cd core && cargo run --bin generate_api_docs

# Generate all documentation (errors + API)
docs: docs-errors docs-api
    @echo "All documentation generated!"

# Generate Dart FFI bindings using Rust code generator
gen-dart-bindings:
    @echo "Generating Dart FFI bindings..."
    cargo run --release --manifest-path core/tools/generate_dart_bindings/Cargo.toml -- --verbose
    @echo "Dart FFI bindings generated successfully!"

# Check if Dart FFI bindings are up-to-date (fails if regeneration needed)
check-dart-bindings:
    @echo "Checking Dart FFI bindings are up-to-date..."
    cargo run --release --manifest-path core/tools/generate_dart_bindings/Cargo.toml -- --check

# Legacy: Generate Dart FFI bindings using Python script (deprecated)
gen-bindings-legacy:
    @echo "Generating Dart FFI bindings (legacy)..."
    python3 scripts/generate_dart_bindings.py
    @echo "Formatting generated Dart code..."
    cd ui && dart format lib/ffi/generated/bindings_generated.dart

# Verify Dart FFI bindings are in sync with Rust exports
verify-bindings: check-dart-bindings
    @echo "Dart FFI bindings are up-to-date!"

# Run comprehensive build (error docs + bindings + Rust + Flutter)
build-full:
    @echo "Running comprehensive build..."
    ./scripts/build.sh --release

# CI-specific checks (strict verification, fail on binding drift)
ci-check: fmt-check clippy test verify-bindings
    @echo "CI checks passed!"

# Build release binary for current platform
build: docs gen-dart-bindings
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
