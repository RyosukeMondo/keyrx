# Scripts Documentation (MECE Reorganization)

## Introduction

This directory contains automation scripts for building, testing, verifying, and launching the keyrx workspace. Scripts have been reorganized following MECE (Mutually Exclusive, Collectively Exhaustive) and SRP (Single Responsibility Principle) principles.

**Design Principles:**
- **MECE**: Each script has a single, non-overlapping responsibility
- **SRP**: One command = one purpose
- **Consistent Interface**: All scripts support common flags (`--error`, `--json`, `--quiet`, `--log-file`)
- **Predictable Output**: Standardized status markers and log formats
- **Fail Fast**: Scripts abort on first error with clear error messages

## Script Reference Table (MECE)

| Category | Script | Purpose | Replaces |
|----------|--------|---------|----------|
| **BUILD** | `build.sh` | Full build sequence (WASM → UI → Daemon) | - |
| **TEST** | `test.sh` | Run tests (unit, integration, fuzz, bench) | - |
| **VERIFY** | `verify.sh` | Quality gates (build, clippy, fmt, tests, coverage) | `validate_quality.sh` |
| **RUN** | `launch.sh` | Start daemon (production mode) | - |
| **RUN** | `dev.sh` | Development server with HMR | `dev_ui.sh` |
| **SETUP** | `setup.sh` | Environment setup (unified) | `setup_*.sh` (5 scripts) |
| **UAT** | `uat.sh` | User acceptance testing (with full build) | `UAT.sh`, `uat_*.sh`, `verify_uat.sh` (5 scripts) |
| **DEPLOY** | `install.sh` | System installation | - |

### Utility Scripts

| Script | Purpose |
|--------|---------|
| `check-types.sh` | TypeScript/Rust type sync verification |
| `fix_doc_tests.sh` | Fix and run documentation tests |
| `test_docs.sh` | Verify documentation examples compile |
| `windows_test_vm.sh` | Run tests in Windows VM |
| `deploy_windows.sh` | Windows deployment |
| `validate_performance.sh` | Run performance benchmarks |
| `verify_file_sizes.sh` | Check file size limits (uses tokei) |
| `audit_test_failures.sh` | Categorize test failures |

### Library Scripts (Internal)

| Script | Purpose |
|--------|---------|
| `lib/common.sh` | Shared utilities (logging, JSON, arg parsing) |
| `lib/build-wasm.sh` | WASM compilation (called by build.sh) |
| `lib/build-ui.sh` | UI compilation (called by build.sh) |

## Quick Reference

```bash
# Development workflow
./scripts/build.sh           # Build everything
./scripts/test.sh            # Run tests
./scripts/verify.sh          # Quality checks
./scripts/dev.sh             # Start dev server with HMR
./scripts/uat.sh             # Full UAT (builds UI first!)

# Setup
./scripts/setup.sh           # Full environment setup
./scripts/setup.sh --check   # Check setup status
./scripts/setup.sh --linux   # Linux-only setup

# UAT modes
./scripts/uat.sh             # Full UAT with UI build
./scripts/uat.sh --rebuild   # Force clean rebuild
./scripts/uat.sh --verify    # Verify running daemon
./scripts/uat.sh --stop      # Stop daemon
```

## Common Flags (All Scripts)

| Flag | Description | Effect |
|------|-------------|--------|
| `--error` | Show only errors | Suppresses info and warnings |
| `--json` | Output in JSON format | Structured JSON to stdout |
| `--quiet` | Suppress non-error output | Only shows errors |
| `--log-file PATH` | Custom log file path | Overrides default log location |

## Output Format Specification

### Status Markers

```
=== accomplished ===  # Operation succeeded (green)
=== failed ===        # Operation failed (red)
=== warning ===       # Completed with warnings (yellow)
```

### Log Format

```
[YYYY-MM-DD HH:MM:SS] [LEVEL] message
```

Levels: `[INFO]` (blue), `[ERROR]` (red), `[WARN]` (yellow), `[DEBUG]` (no color)

### Exit Codes

| Code | Meaning |
|------|---------|
| 0 | Success |
| 1 | Failure |
| 2 | Missing required tool |

## Detailed Script Reference

### build.sh - Full Build

Orchestrates the complete build sequence: WASM → UI → Daemon.

```bash
./scripts/build.sh                    # Debug build
./scripts/build.sh --release          # Release build (optimized)
./scripts/build.sh --watch            # Watch mode (auto-rebuild)
./scripts/build.sh --json             # JSON output for CI
```

**Build Sequence:**
1. Build WASM module (keyrx_core → WebAssembly)
2. Build UI (React + embedded WASM)
3. Build Daemon (with embedded UI)

### test.sh - Test Execution

Run tests with flexible filtering.

```bash
./scripts/test.sh                     # All tests
./scripts/test.sh --unit              # Unit tests only
./scripts/test.sh --integration       # Integration tests only
./scripts/test.sh --fuzz 60           # Fuzz tests for 60 seconds
./scripts/test.sh --bench             # Benchmarks (requires nightly)
```

### verify.sh - Quality Gates

Comprehensive quality verification.

```bash
./scripts/verify.sh                   # Full verification
./scripts/verify.sh --skip-coverage   # Skip coverage (faster)
./scripts/verify.sh --json            # JSON output
```

**Checks Performed:**
1. Build - `cargo build --workspace`
2. Clippy - `cargo clippy -- -D warnings`
3. Format - `cargo fmt --check`
4. Tests - `cargo test --workspace`
5. Coverage - `cargo llvm-cov` (80% minimum)
6. UI Tests - `npm test --coverage`
7. E2E Tests - Playwright

### launch.sh - Production Daemon

Build and launch the daemon.

```bash
./scripts/launch.sh                   # Launch (debug build)
./scripts/launch.sh --release         # Launch (release build)
./scripts/launch.sh --debug           # Debug logging
./scripts/launch.sh --headless        # No browser
./scripts/launch.sh --config PATH     # Custom config
```

### dev.sh - Development Server

Start development environment with hot module replacement.

```bash
./scripts/dev.sh                      # Start dev server + daemon
./scripts/dev.sh --no-daemon          # Dev server only (daemon already running)
./scripts/dev.sh --no-browser         # Don't open browser
```

**Development URLs:**
- UI Dev Server: http://localhost:5173
- Daemon API: http://localhost:9867

### setup.sh - Environment Setup (Unified)

Consolidates all setup operations.

```bash
./scripts/setup.sh                    # Full setup
./scripts/setup.sh --check            # Check status only
./scripts/setup.sh --dev-tools        # Dev tools only
./scripts/setup.sh --linux            # Linux environment only
./scripts/setup.sh --hooks            # Git hooks only
./scripts/setup.sh --desktop          # Desktop integration only
./scripts/setup.sh --windows-vm       # Windows VM setup only
```

**Components:**
1. Dev Tools - cargo-watch, cargo-llvm-cov, wasm-pack
2. Linux - User groups, udev rules, uinput module
3. Git Hooks - Pre-commit verification
4. Desktop - .desktop file, application icons
5. Windows VM - Vagrant + libvirt

### uat.sh - User Acceptance Testing (Unified)

Complete UAT workflow that **always builds the UI**.

```bash
./scripts/uat.sh                      # Full UAT (build + start)
./scripts/uat.sh --rebuild            # Force clean rebuild
./scripts/uat.sh --verify             # Verify running daemon
./scripts/uat.sh --stop               # Stop daemon
./scripts/uat.sh --headless           # No browser
./scripts/uat.sh --release            # Release build
./scripts/uat.sh --debug              # Debug logging
```

**UAT Sequence:**
1. Check prerequisites (groups, udev, modules)
2. Compile user_layout.rhai → .krx
3. Build WASM module
4. Build Web UI (Vite production build)
5. Build daemon with embedded UI
6. Stop existing daemon
7. Start daemon with system tray
8. Open browser (unless --headless)

### install.sh - System Installation

Install KeyRX daemon to system.

```bash
./scripts/install.sh                  # Install to ~/.local/bin
```

## Troubleshooting

### UAT shows old UI

This was the original problem. Use the new unified `uat.sh` which **always builds the UI**:

```bash
./scripts/uat.sh                      # Builds UI before starting daemon
./scripts/uat.sh --rebuild            # Force clean rebuild if needed
```

### Pre-commit hook blocks commit

```bash
./scripts/verify.sh                   # See detailed errors
cargo fmt                             # Auto-fix formatting
cargo clippy --fix                    # Auto-fix clippy warnings
```

### Setup issues

```bash
./scripts/setup.sh --check            # Check current status
./scripts/setup.sh --linux            # Fix Linux setup only
```

### Build failures

```bash
./scripts/build.sh                    # See build errors
cargo build --workspace 2>&1          # Direct cargo output
```

## Migration Guide

If you were using the old scripts:

| Old Script | New Command |
|------------|-------------|
| `UAT.sh` | `./scripts/uat.sh` |
| `uat_linux.sh` | `./scripts/uat.sh` |
| `uat_rebuild.sh` | `./scripts/uat.sh --rebuild` |
| `verify_uat.sh` | `./scripts/uat.sh --verify` |
| `dev_ui.sh` | `./scripts/dev.sh` |
| `setup_dev_environment.sh` | `./scripts/setup.sh --dev-tools` |
| `setup_linux.sh` | `./scripts/setup.sh --linux` |
| `setup_hooks.sh` | `./scripts/setup.sh --hooks` |
| `setup_desktop_integration.sh` | `./scripts/setup.sh --desktop` |
| `setup_keyrx_windows_vm.sh` | `./scripts/setup.sh --windows-vm` |
| `validate_quality.sh` | `./scripts/verify.sh` |
| `build_wasm.sh` | `./scripts/lib/build-wasm.sh` (internal) |
| `build_ui.sh` | `./scripts/lib/build-ui.sh` (internal) |

## For AI Agents

**Key Points:**
1. All scripts support `--json` for machine-parseable output
2. Exit codes: 0=success, 1=failure, 2=missing tool
3. Status markers (`=== accomplished ===`, `=== failed ===`)
4. Log files in `scripts/logs/` with epoch timestamps
5. **Use `uat.sh` for UAT** - it builds the UI unlike the old UAT.sh

**Recommended Workflow:**
```bash
./scripts/setup.sh --check            # 1. Verify environment
./scripts/verify.sh --skip-coverage   # 2. Quick quality check
./scripts/uat.sh                      # 3. Full UAT with fresh UI
./scripts/uat.sh --verify             # 4. Verify daemon is working
```

**Error Handling:**
- Always check exit codes
- Parse JSON output with `jq` for decision-making
- Read log files in `scripts/logs/` for debugging
