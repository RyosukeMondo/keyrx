# Scripts Documentation

## Introduction

This directory contains automation scripts for building, testing, verifying, and launching the keyrx workspace. All scripts follow consistent patterns for logging, error handling, and output formatting to enable both human-readable and machine-parseable (JSON) operation.

**Design Principles:**
- **Consistent Interface**: All scripts support common flags (`--error`, `--json`, `--quiet`, `--log-file`)
- **Predictable Output**: Standardized status markers and log formats
- **Fail Fast**: Scripts abort on first error with clear error messages
- **Machine-Parseable**: JSON mode for CI/CD integration and AI agent consumption
- **Comprehensive Logging**: All operations logged to timestamped files in `scripts/logs/`

## Script Reference Table

| Script | Purpose | Common Flags | Script-Specific Flags | Exit Codes |
|--------|---------|--------------|----------------------|------------|
| `build.sh` | Build workspace in debug or release mode, with optional watch mode | `--error`, `--json`, `--quiet`, `--log-file` | `--release`, `--watch` | 0=success, 1=build failed, 2=missing tool |
| `verify.sh` | Run comprehensive quality checks (build, clippy, fmt, tests, coverage) | `--error`, `--json`, `--quiet`, `--log-file` | `--skip-coverage` | 0=all passed, 1=check failed, 2=missing tool |
| `test.sh` | Execute tests with flexible filtering (unit, integration, fuzz, bench) | `--error`, `--json`, `--quiet`, `--log-file` | `--unit`, `--integration`, `--fuzz DURATION`, `--bench` | 0=tests passed, 1=tests failed, 2=missing tool |
| `launch.sh` | Build and launch the keyrx daemon with configuration options | `--error`, `--json`, `--quiet`, `--log-file` | `--headless`, `--debug`, `--config PATH`, `--release` | 0=launched, 1=launch failed, 2=missing tool |
| `setup_hooks.sh` | Install Git pre-commit hooks for automated quality gates | N/A | N/A | 0=installed, 1=not a git repo |

## Output Format Specification

### Status Markers

All scripts output standardized status markers to indicate completion status:

```
=== accomplished ===  # Operation completed successfully (green)
=== failed ===        # Operation failed (red)
=== warning ===       # Operation completed with warnings (yellow)
```

These markers are always prefixed with timestamps: `[YYYY-MM-DD HH:MM:SS]`

**Example:**
```
[2025-12-21 10:30:45] === accomplished ===
```

### Log Format

All log messages follow this format:
```
[YYYY-MM-DD HH:MM:SS] [LEVEL] message
```

**Log Levels:**
- `[INFO]` - Informational messages (blue)
- `[ERROR]` - Error messages (red, stderr)
- `[WARN]` - Warning messages (yellow, stderr)
- `[DEBUG]` - Debug messages (no color)

**Examples:**
```
[2025-12-21 10:30:40] [INFO] Building in release mode...
[2025-12-21 10:30:42] [ERROR] Build failed
[2025-12-21 10:30:43] [WARN] Coverage below threshold
```

### JSON Output Schema

When using `--json` flag, scripts output structured JSON to stdout. All JSON output is printed after the operation completes.

#### build.sh JSON Schema
```json
{
  "status": "success|failed",
  "build_type": "debug|release",
  "mode": "standard|watch",
  "exit_code": "0|1|2"
}
```

**Error variant:**
```json
{
  "status": "failed",
  "error": "error description",
  "exit_code": "2"
}
```

#### verify.sh JSON Schema
```json
{
  "status": "success|failed",
  "checks": {
    "build": "PASS|FAIL",
    "clippy": "PASS|FAIL",
    "fmt": "PASS|FAIL",
    "test": "PASS|FAIL",
    "coverage": "PASS (XX.XX%)|FAIL (XX.XX% < 80%)|SKIP"
  },
  "failed_checks": ["check1", "check2"],
  "exit_code": "0|1"
}
```

#### test.sh JSON Schema
```json
{
  "status": "success|failed",
  "mode": "all|unit|integration|fuzz|bench",
  "tests_passed": "N",
  "tests_failed": "N",
  "exit_code": "0|1"
}
```

#### launch.sh JSON Schema
```json
{
  "status": "success|failed",
  "pid": "12345",
  "port": "8080",
  "build_type": "debug|release",
  "headless": "true|false",
  "debug": "true|false",
  "exit_code": "0|1"
}
```

**Error variant:**
```json
{
  "status": "failed",
  "error": "build failed|daemon startup failed",
  "build_type": "debug|release",
  "exit_code": "1"
}
```

### Log Files

All scripts automatically create timestamped log files in `scripts/logs/`:
- Format: `{script_name}_{epoch_timestamp}.log`
- Example: `build_1734784800.log`
- Contains all output (info, warnings, errors, status markers)
- Never committed to git (ignored by `.gitignore`)

You can specify a custom log file path using `--log-file PATH`.

## Flag Reference

### Common Flags (Supported by All Scripts)

| Flag | Description | Effect |
|------|-------------|--------|
| `--error` | Show only errors | Suppresses info and warnings, only shows errors |
| `--json` | Output in JSON format | Outputs structured JSON to stdout, implies `--quiet` |
| `--quiet` | Suppress non-error output | Suppresses info messages, shows only errors |
| `--log-file PATH` | Custom log file path | Overrides default timestamped log file location |

### Script-Specific Flags

#### build.sh
| Flag | Description |
|------|-------------|
| `--release` | Build in release mode (optimized, slower build) |
| `--watch` | Watch mode - rebuild on file changes (requires cargo-watch) |

#### verify.sh
| Flag | Description |
|------|-------------|
| `--skip-coverage` | Skip coverage check (useful for faster iteration) |

#### test.sh
| Flag | Description |
|------|-------------|
| `--unit` | Run only unit tests (lib tests) |
| `--integration` | Run only integration tests (tests/ directory) |
| `--fuzz DURATION` | Run fuzz tests for DURATION seconds (requires cargo-fuzz) |
| `--bench` | Run benchmarks (requires nightly toolchain) |

#### launch.sh
| Flag | Description |
|------|-------------|
| `--headless` | Suppress browser launch (headless mode) |
| `--debug` | Enable debug logging (log-level: debug) |
| `--config PATH` | Specify custom configuration file path |
| `--release` | Build and run release binary (optimized) |

## Example Commands

### build.sh Examples

```bash
# Debug build (default)
./scripts/build.sh

# Release build (optimized)
./scripts/build.sh --release

# Watch mode - auto-rebuild on file changes
./scripts/build.sh --watch

# Watch mode with release build
./scripts/build.sh --watch --release

# JSON output for CI/CD integration
./scripts/build.sh --json

# Quiet build (only show errors)
./scripts/build.sh --quiet

# Build with custom log file
./scripts/build.sh --log-file /tmp/my-build.log
```

### verify.sh Examples

```bash
# Full verification (build, clippy, fmt, tests, coverage)
./scripts/verify.sh

# Fast verification (skip coverage check)
./scripts/verify.sh --skip-coverage

# JSON output for CI/CD
./scripts/verify.sh --json

# Quiet verification (used by pre-commit hook)
./scripts/verify.sh --quiet

# Show only errors
./scripts/verify.sh --error
```

### test.sh Examples

```bash
# Run all tests (default)
./scripts/test.sh

# Run only unit tests
./scripts/test.sh --unit

# Run only integration tests
./scripts/test.sh --integration

# Run fuzz tests for 60 seconds
./scripts/test.sh --fuzz 60

# Run benchmarks (requires nightly)
./scripts/test.sh --bench

# JSON output with test counts
./scripts/test.sh --json

# Quiet test run
./scripts/test.sh --quiet
```

### launch.sh Examples

```bash
# Launch daemon (debug build, info logging)
./scripts/launch.sh

# Launch with release build
./scripts/launch.sh --release

# Launch with debug logging
./scripts/launch.sh --debug

# Launch without opening browser
./scripts/launch.sh --headless

# Launch with custom config
./scripts/launch.sh --config custom.toml

# Launch and get PID/port in JSON
./scripts/launch.sh --json

# Combined flags
./scripts/launch.sh --release --debug --headless
```

### setup_hooks.sh Examples

```bash
# Install pre-commit hook (run once per clone)
./scripts/setup_hooks.sh

# Re-run to update hook (idempotent)
./scripts/setup_hooks.sh
```

## Failure Scenarios

### build.sh Failures

| Scenario | Output | Exit Code | Solution |
|----------|--------|-----------|----------|
| Cargo not installed | `[ERROR] Required tool 'cargo' not found` | 2 | Install Rust: https://rustup.rs |
| Compilation error | Build output with errors, `=== failed ===` | 1 | Fix compilation errors in source code |
| cargo-watch missing (--watch) | `[ERROR] Required tool 'cargo-watch' not found` | 2 | `cargo install cargo-watch` |

### verify.sh Failures

| Scenario | Output | Exit Code | Solution |
|----------|--------|-----------|----------|
| Build fails | `Build check: FAIL`, aborts | 1 | Run `./scripts/build.sh` to see details |
| Clippy warnings | `Clippy check: FAIL`, aborts | 1 | Run `cargo clippy --workspace` and fix warnings |
| Format issues | `Format check: FAIL - run 'cargo fmt' to fix` | 1 | Run `cargo fmt` to auto-format code |
| Test failures | `Test check: FAIL`, aborts | 1 | Run `cargo test --workspace` to see failing tests |
| Coverage < 80% | `Coverage check: FAIL (XX.XX% < 80% minimum)` | 1 | Add tests to increase coverage |
| cargo-tarpaulin missing | `[ERROR] Required tool 'cargo-tarpaulin' not found` | 2 | `cargo install cargo-tarpaulin` |

### test.sh Failures

| Scenario | Output | Exit Code | Solution |
|----------|--------|-----------|----------|
| Cargo not installed | `[ERROR] Required tool 'cargo' not found` | 2 | Install Rust: https://rustup.rs |
| Test failures | `Tests failed: N passed, M failed`, `=== failed ===` | 1 | Fix failing tests |
| cargo-fuzz missing (--fuzz) | `[ERROR] cargo-fuzz not found` | 1 | `cargo install cargo-fuzz` |
| Nightly missing (--bench) | `[ERROR] Nightly Rust toolchain not found` | 1 | `rustup install nightly` |
| Fuzz directory missing | `[ERROR] Fuzz directory not found: keyrx_core/fuzz` | 1 | `cd keyrx_core && cargo fuzz init` |

### launch.sh Failures

| Scenario | Output | Exit Code | Solution |
|----------|--------|-----------|----------|
| Build fails | `[ERROR] Build failed`, `=== failed ===` | 1 | Fix build errors (run `./scripts/build.sh`) |
| Daemon startup fails | `[ERROR] Daemon failed to start` | 1 | Check daemon output in error message |
| Binary not found | `[ERROR] Daemon binary not found at: target/.../keyrx_daemon` | 1 | Ensure build completed successfully |
| Config file missing | `[ERROR] Config file not found: PATH` | 1 | Provide valid config file path |

### setup_hooks.sh Failures

| Scenario | Output | Exit Code | Solution |
|----------|--------|-----------|----------|
| Not a git repo | `Error: Not a git repository` | 1 | Run from within a git repository |

## Troubleshooting

### Q: Build fails with "cargo not found"
**A:** Install Rust toolchain from https://rustup.rs, then run `cargo --version` to verify.

### Q: verify.sh takes too long during development
**A:** Use `./scripts/verify.sh --skip-coverage` to skip the coverage check (fastest), or run individual checks:
- Build only: `./scripts/build.sh`
- Linting only: `cargo clippy --workspace`
- Format only: `cargo fmt --check` (or `cargo fmt` to auto-fix)
- Tests only: `./scripts/test.sh`

### Q: Pre-commit hook blocks my commit
**A:** The hook runs `verify.sh --quiet` to ensure code quality. Options:
1. Fix the issues (recommended): Run `./scripts/verify.sh` to see details
2. Bypass the hook (not recommended): `git commit --no-verify`

### Q: How do I parse JSON output in CI/CD?
**A:** Use `jq` for JSON parsing:
```bash
# Example: Extract exit code from build
./scripts/build.sh --json | jq -r '.exit_code'

# Example: Check if verification passed
if ./scripts/verify.sh --json | jq -e '.status == "success"'; then
  echo "Verification passed"
fi

# Example: Get failed checks
./scripts/verify.sh --json | jq -r '.failed_checks[]'
```

### Q: Watch mode doesn't rebuild on file changes
**A:** Ensure cargo-watch is installed: `cargo install cargo-watch`

### Q: Coverage check fails with "cargo-tarpaulin not found"
**A:** Install tarpaulin: `cargo install cargo-tarpaulin`

### Q: Benchmark tests fail with "nightly toolchain not found"
**A:** Install nightly: `rustup install nightly`

### Q: Where are log files stored?
**A:** All log files are in `scripts/logs/` with format `{script}_{epoch}.log`. These are automatically ignored by git.

### Q: How do I run scripts from any directory?
**A:** Scripts use relative paths and auto-detect their location. However, they should be run from the project root:
```bash
# From project root (recommended)
./scripts/build.sh

# From scripts/ directory (works but not recommended)
cd scripts
./build.sh
```

### Q: Can I run multiple scripts in parallel?
**A:** Yes, scripts are independent. However, note:
- `verify.sh` runs `build.sh` internally, so don't run them simultaneously
- Multiple `launch.sh` instances will conflict (daemon uses same ports)
- Multiple `test.sh` instances may interfere with each other

### Q: How do I customize logging behavior?
**A:** Use flag combinations:
- Minimal output: `--quiet` (only errors)
- Errors only: `--error` (no info/warnings)
- Custom log location: `--log-file /path/to/log`
- No terminal output: `--json` (structured output only)

### Q: JSON output contains ANSI color codes
**A:** JSON mode automatically suppresses color codes. If you see them, ensure you're using `--json` flag correctly and capturing only stdout (not stderr).

### Q: How do I debug a failing script?
**A:** Remove `--quiet` flag and check the log file:
```bash
# Run without quiet mode
./scripts/build.sh

# Check the most recent log file
ls -lt scripts/logs/ | head -n 2
cat scripts/logs/build_*.log
```

## Integration with Makefile

Scripts are wrapped by the root Makefile for convenient access:

```bash
make build    # Calls scripts/build.sh
make verify   # Calls scripts/verify.sh
make test     # Calls scripts/test.sh
make launch   # Calls scripts/launch.sh
make setup    # Installs tools and runs scripts/setup_hooks.sh
```

See root `Makefile` for all available targets.

## For AI Agents

**Key Discovery Points:**
1. All scripts support `--json` for machine-parseable output
2. Exit codes: 0=success, 1=failure, 2=missing tool
3. Status markers (`=== accomplished ===`, `=== failed ===`) indicate completion status
4. Log files automatically created in `scripts/logs/` with epoch timestamps
5. Pre-commit hook runs `verify.sh --quiet` before every commit
6. Use `--skip-coverage` during development for faster iteration
7. JSON schemas documented above for parsing script output
8. All scripts fail fast - abort on first error with clear messaging

**Recommended AI Workflow:**
1. Start: `./scripts/verify.sh --json` to check current state
2. Build: `./scripts/build.sh --quiet` during development
3. Test: `./scripts/test.sh --json` to get test results
4. Verify: `./scripts/verify.sh --skip-coverage` for fast quality check
5. Commit: Git hooks automatically run verification
6. Parse JSON output with `jq` for decision-making

**Error Handling:**
- Always check exit codes (0=success, non-zero=failure)
- Parse JSON output for detailed status
- Read log files in `scripts/logs/` for full debugging context
