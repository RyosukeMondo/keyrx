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
    @echo "Installing coverage tooling (dart coverage + lcov)..."
    ./scripts/install-coverage-tools.sh
    @echo "Installing git hooks..."
    ./scripts/install-hooks.sh
    @echo "Setup complete!"

# Run core in development mode with auto-reload
dev:
    cd core && cargo watch -x run

# Run Flutter UI in development mode
ui:
    cd ui && flutter run -d linux

# Run Flutter UI in development mode with mock FFI bridge
dev-mock:
    cd ui && flutter run --dart-define=MOCK_FFI=true -d ${DEVICE:-linux}

# Install Flutter dependencies and run code generation
ui-setup:
    cd ui && flutter pub get
    cd ui && flutter pub run build_runner build --delete-conflicting-outputs

# Run Flutter analysis with infos treated as errors
ui-analyze:
    cd ui && flutter analyze --fatal-infos

# Check Flutter formatting without writing changes
ui-format:
    cd ui && dart format --output=none --set-exit-if-changed lib test scripts

# Check Dart code metrics (file/function length limits)
ui-metrics:
    cd ui && dart run scripts/check_metrics.dart

# Run Flutter tests
ui-test:
    cd ui && flutter test --concurrency ${CONCURRENCY:-4}

# Verify golden baselines
ui-golden-verify:
    cd ui && flutter test --concurrency ${CONCURRENCY:-2} test/widgets/golden

# Update golden baselines
ui-golden-update:
    cd ui && flutter test --update-goldens --concurrency ${CONCURRENCY:-2} test/widgets/golden

# Run Flutter tests in watch mode
ui-test-watch:
    cd ui && flutter test --watch --concurrency ${CONCURRENCY:-4}

# Generate Flutter coverage and HTML report
ui-coverage:
    @echo "Collecting Flutter coverage and generating HTML report..."
    ./ui/scripts/generate_coverage_report.sh
    @echo "Coverage report available at ui/coverage/html/index.html"

# Run full Flutter quality gate (format, analyze, metrics, tests)
ui-quality: ui-format ui-analyze ui-metrics ui-test

# Legacy alias retained for compatibility
ui-check: ui-analyze

# Verify Flutter build (Windows)
ui-verify: ui-check
    cd ui && flutter build windows --debug

# Run all quality checks (fmt, clippy, test, docs, bindings, Flutter quality)
check: fmt-check clippy test docs verify-bindings ui-quality
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
    cd core && KEYRX_SKIP_DEVICE_SCAN=1 cargo nextest run

# Run tests with standard cargo (fallback)
test-cargo:
    cd core && KEYRX_SKIP_DEVICE_SCAN=1 cargo test

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

# Check documentation for warnings (fails if any warnings exist)
doc-check:
    @echo "Checking documentation for warnings..."
    @cd core && cargo doc --no-deps 2>&1 | tee /tmp/doc-output.txt
    @if grep -qi "warning" /tmp/doc-output.txt; then \
        echo "Documentation warnings found:"; \
        grep -i "warning" /tmp/doc-output.txt; \
        exit 1; \
    fi
    @echo "Documentation check passed - no warnings found!"

# Check test coverage threshold (80% minimum)
coverage-check:
    @echo "Checking coverage threshold (80% minimum)..."
    cd core && cargo llvm-cov --all-features --fail-under-lines 80

# CI-specific checks (strict verification, fail on binding drift)
ci-check: fmt-check clippy test verify-bindings doc-check
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
