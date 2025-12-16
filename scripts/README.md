# Scripts Directory

This directory contains utility scripts organized by category following MECE (Mutually Exclusive, Collectively Exhaustive) principles.

## Directory Structure

```
scripts/
├── setup/          # Environment setup (run once or rarely)
├── dev/            # Development workflows (run daily)
├── build/          # Build orchestration
├── ffi/            # FFI-specific tooling
├── metrics/        # Analysis & measurement
├── test/           # Testing utilities
├── docs/           # Documentation generation
├── release/        # Release management
└── utils/          # Miscellaneous utilities
```

## Setup Scripts (`setup/`)

Scripts for initial environment configuration (run once or rarely).

### `dev_environment.sh`

Automated developer onboarding script for local environments.

**What it does:**
1. Installs Rust components (rustfmt, clippy) and cargo tools (cargo-nextest, cargo-watch, watchexec-cli, sccache)
2. Installs coverage tooling (dart coverage + lcov via apt/brew) by delegating to `coverage_tools.sh`
3. Fetches Rust and Flutter dependencies; runs Flutter `build_runner` to prime generated code
4. Installs repository git hooks
5. Runs initial Rust build and a Flutter test compile unless skipped

**Usage:**
```bash
./scripts/setup/dev_environment.sh                 # Full setup
./scripts/setup/dev_environment.sh --skip-build    # Skip initial builds
# Or via just: just setup [with SETUP_OPTS="--skip-build"]
```

**Notes:**
- Skips Rust/Flutter steps gracefully if those toolchains are not installed yet
- Uses `apt-get` or Homebrew for lcov when present; install manually on other platforms

### `ffi_tools.sh`

Install FFI-focused tooling and fast linkers for Rust-first development.

**What it does:**
- Installs: cargo-watch, tokei (metrics), mold (Linux), zld (macOS)
- Configures linker preference in .cargo/config.toml when available
- Safe to re-run; skips already installed tools

**Usage:**
```bash
./scripts/setup/ffi_tools.sh
# Or via just: just setup-ffi-tools
```

### `linux_input.sh`

Setup script for KeyRx Linux input permissions.

**What it does:**
- Adds the current user to the input group (for evdev/uinput access)
- Loads the uinput kernel module
- Installs recommended udev rules to set permissions on input/uinput devices

**Usage:**
```bash
./scripts/setup/linux_input.sh
```

### `git_hooks.sh`

Install git pre-commit hooks for code quality checks.

**What it does:**
- Installs git hooks from `.githooks/` to `.git/hooks/`
- Enforces quality gates (fmt, clippy, tests, binding verification)

**Usage:**
```bash
./scripts/setup/git_hooks.sh
# Or automatically via: just setup
```

### `coverage_tools.sh`

Install coverage tooling for Flutter/Dart.

**What it does:**
- Installs dart coverage package via `dart pub global`
- Installs lcov CLI via native package managers (apt/brew)

**Usage:**
```bash
./scripts/setup/coverage_tools.sh
```

### CI Setup Scripts (`setup/ci/`)

#### `flutter.sh`

Modular Flutter/FVM setup script for CI/CD environments.

**Prerequisites:**
- Android SDK installed and `ANDROID_HOME` environment variable set
- Git repository initialized
- Internet connection for downloading FVM and Flutter

**What it does:**
1. Installs FVM (Flutter Version Manager)
2. Installs Flutter stable version (reads from `.fvmrc`)
3. Configures project to use FVM-managed Flutter
4. Configures Flutter with Android SDK
5. Verifies setup with `flutter doctor`
6. Cleans up FVM-generated git modifications

**Usage:**
```bash
export ANDROID_HOME=/path/to/android-sdk
./scripts/setup/ci/flutter.sh
```

**Exit codes:**
- `0`: Success
- `1`: ANDROID_HOME not set

#### `jules.sh`

Complete Jules CI setup script for Android/Flutter build environment.

**What it does:**
1. Installs system dependencies (Java 17, wget, unzip, curl, git)
2. Downloads and configures Android SDK (API 34, Build Tools 34.0.0)
3. Runs modular Flutter/FVM setup via `flutter.sh`

**Usage:**
```bash
# In Jules CI environment
./scripts/setup/ci/jules.sh
```

**Note:** This script expects to run in `/app` directory (Jules CI default).

## Development Scripts (`dev/`)

Scripts for daily development workflows.

### `integrated.sh`

Integrated development workflow: Rust watch + Flutter hot reload in parallel.

**What it does:**
- Runs cargo-watch for Rust FFI changes (compile → codegen → reload)
- Runs Flutter hot reload for UI changes
- Coordinates logs with [RUST] and [FLUTTER] prefixes
- Handles Ctrl+C gracefully (kills both processes)
- Detects ABI changes requiring full restart vs Dart-only hot reload

**Usage:**
```bash
./scripts/dev/integrated.sh
# Or via just: just dev

# With mock FFI bridge (UI-only development)
./scripts/dev/integrated.sh --mock
# Or via just: just dev-mock-full
```

**Supports:**
- `--mock` flag to run with mock FFI bridge (ultra-fast UI iteration)

### `ffi_watch.sh`

Watch helper invoked by cargo-watch to regenerate Dart bindings.

**What it does:**
- Tracks SHA256 of contract JSON files to skip unnecessary generation
- Emits human-friendly notifications
- Never crashes the watcher on failure

**Usage:**
```bash
# Direct invocation
./scripts/dev/ffi_watch.sh

# Via cargo-watch (recommended)
just ffi-watch
```

## Build Scripts (`build/`)

Scripts for build orchestration.

### `main.sh`

Comprehensive build script for KeyRX.

**What it does:**
1. Generates error documentation
2. Generates Dart FFI bindings
3. Builds Rust core library
4. Builds Flutter UI

**Usage:**
```bash
./scripts/build/main.sh                # Debug build
./scripts/build/main.sh --release      # Release build
./scripts/build/main.sh --rust-only    # Rust only
./scripts/build/main.sh --flutter-only # Flutter only

# Or via just: just build-full
```

### `flutter_prebuild.sh`

Flutter pre-build hook to verify FFI bindings are in sync.

**What it does:**
- Ensures Dart FFI bindings match the current Rust exports
- Intended to be run before Flutter builds

**Usage:**
```bash
./scripts/build/flutter_prebuild.sh
```

### Build Wrappers (`build/wrappers/`)

#### `rustc.sh`

Cargo invokes this wrapper to optionally use sccache for faster compilation.

**What it does:**
- Prefers sccache when available
- Falls back to rustc so builds never fail

#### `linker.sh`

Wrapper linker that prefers fast linkers when available.

**What it does:**
- Uses mold on Linux
- Uses zld on macOS
- Falls back to system ld if not available

## FFI Scripts (`ffi/`)

FFI-specific development tooling.

### `validate_contracts.sh`

FFI Contract Validation Script.

**What it does:**
- Validates that all `#[ffi_export]` annotated functions have matching contract JSON entries
- Ensures contracts match Rust signatures
- Auto-fixes contract mismatches with `--fix` flag

**Usage:**
```bash
./scripts/ffi/validate_contracts.sh                # Validate all contracts
./scripts/ffi/validate_contracts.sh --verbose      # Verbose validation
./scripts/ffi/validate_contracts.sh --fix          # Auto-fix mismatches
```

### `verify_bindings.py`

Verify that Dart FFI bindings are in sync with Rust exports.

**What it does:**
- Compares existing bindings with freshly generated ones
- Detects binding drift
- Used in CI to ensure bindings are committed and up-to-date

**Usage:**
```bash
python3 scripts/ffi/verify_bindings.py
# Or via just: just verify-bindings
```

### `detect_breaking_changes.sh`

Detect FFI contract breaking changes against a base reference.

**What it does:**
- Compares current FFI contracts with a base Git reference
- Identifies breaking changes (signature changes, removals)
- Outputs in text or JSON format

**Usage:**
```bash
./scripts/ffi/detect_breaking_changes.sh --base HEAD~1 --format text
./scripts/ffi/detect_breaking_changes.sh --base main --format json

# Or via just: just diff-ffi-contracts HEAD~1
```

### `add_instrumentation.sh`

Add FFI instrumentation to all export functions.

**What it does:**
- Adds `#[instrument]` attributes to all FFI export functions
- Skips functions that already have instrumentation

**Usage:**
```bash
./scripts/ffi/add_instrumentation.sh
```

### `split_module.sh`

Split large FFI modules into smaller, focused files.

**What it does:**
- Analyzes FFI module structure
- Splits modules by function groups
- Maintains contract integrity

**Usage:**
```bash
./scripts/ffi/split_module.sh <command> <module-file> [options]
```

### `log_implementation.sh`

Create an FFI implementation log entry from the template.

**What it does:**
- Generates timestamped implementation logs
- Records FFI development progress
- Captures metrics and AI effectiveness

**Usage:**
```bash
./scripts/ffi/log_implementation.sh --task-id <id>
./scripts/ffi/log_implementation.sh --task-id <id> --base-ref <ref>

# Or via just: just ffi-log <task_id> [<base_ref>]
```

## Metrics Scripts (`metrics/`)

Analysis and measurement tools.

### `ffi_summary.sh`

Display FFI metrics summary.

**What it does:**
- Shows FFI call count, latency, error rates
- Supports text and JSON output

**Usage:**
```bash
./scripts/metrics/ffi_summary.sh
./scripts/metrics/ffi_summary.sh --json

# Or via just: just ffi-metrics [format]
```

### `contract_dashboard.sh`

Generate FFI Contract Metrics Dashboard.

**What it does:**
- Analyzes all FFI contracts
- Generates HTML dashboard with metrics
- Outputs to `.spec-workflow/analysis/contract-dashboard.html`

**Usage:**
```bash
./scripts/metrics/contract_dashboard.sh
# Or via just: just ffi-dashboard
```

### `module_sizes.sh`

Analyze FFI module sizes.

**What it does:**
- Reports lines of code per module
- Identifies oversized modules
- Helps maintain code metrics compliance

**Usage:**
```bash
./scripts/metrics/module_sizes.sh
```

### `ffi_history.sh`

Analyze FFI implementation history.

**What it does:**
- Calculates velocity, churn, coverage, and AI effectiveness
- Analyzes implementation logs for FFI development metrics
- Generates insights HTML report

**Usage:**
```bash
./scripts/metrics/ffi_history.sh
# Or via just: just analyze-ffi
```

### `benchmark.sh`

FFI Performance Benchmarking Wrapper.

**What it does:**
- Runs comprehensive FFI tooling benchmarks
- Generates reports with regression detection
- Manages baseline comparisons

**Usage:**
```bash
./scripts/metrics/benchmark.sh                        # Run benchmarks
./scripts/metrics/benchmark.sh --baseline <name>      # Save as baseline
./scripts/metrics/benchmark.sh --compare <name>       # Compare against baseline

# Or via just:
# just bench-ffi
# just bench-ffi-baseline <name>
# just bench-ffi-compare <name>
```

### `check_regression.py`

Benchmark regression checker for Criterion benchmark results.

**What it does:**
- Analyzes Criterion benchmark output
- Detects performance regressions
- Used by benchmark.sh for regression detection

**Usage:**
```bash
python3 scripts/metrics/check_regression.py <baseline> <current>
```

## Test Scripts (`test/`)

Testing utilities.

### `feature_combinations.sh`

Test all feature combinations for build-optimization.

**What it does:**
- Tests all Cargo feature flags work correctly in isolation and combination
- Ensures all feature combinations build successfully

**Usage:**
```bash
./scripts/test/feature_combinations.sh
```

## Documentation Scripts (`docs/`)

Documentation generation tools.

### `generate_errors.sh`

Generate error documentation from registry.

**What it does:**
- Generates documentation from error code definitions
- Creates markdown documentation for all error types

**Usage:**
```bash
./scripts/docs/generate_errors.sh
# Or via just: just docs-errors
```

## Release Scripts (`release/`)

Release management tools.

### `bump_version.sh`

Release helper script for KeyRx.

**What it does:**
- Automates version bump
- Generates changelog
- Creates Git tag

**Usage:**
```bash
./scripts/release/bump_version.sh <version>
# Example: ./scripts/release/bump_version.sh 1.0.0

# Or via just: just release <version>
```

## Utility Scripts (`utils/`)

Miscellaneous utilities.

### `show_key_position.sh`

Debugging tool for keyboard input positions.

**What it does:**
- Shows key position: scan_code → KEY_NAME → row_col
- Useful for keyboard mapping development

**Usage:**
```bash
./scripts/utils/show_key_position.sh
```

## Naming Conventions

All scripts follow these conventions:

1. **Use underscores consistently** (snake_case for filenames)
2. **Remove category prefixes** (folder provides context)
3. **Use descriptive names** (what it does, not how)
4. **Keep extensions** (.sh for bash, .py for python)

## Organization Principles

The scripts folder follows:

- **MECE (Mutually Exclusive, Collectively Exhaustive)**: Every script belongs to exactly one category, all scripts are categorized
- **SLAP (Single Level of Abstraction Principle)**: Each folder contains scripts at the same level of abstraction
- **Consistent naming**: snake_case, descriptive, no redundant prefixes
