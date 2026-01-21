# macOS Testing Guide

This guide documents the comprehensive three-layer testing strategy for macOS platform support in KeyRx.

## Overview

KeyRx implements a three-layer testing strategy for macOS to maximize test coverage while working within the constraints of macOS Accessibility permissions:

1. **Layer 1: Mock Tests** - No permissions required (CI-friendly)
2. **Layer 2: E2E Tests** - Requires Accessibility permission (developer machines)
3. **Layer 3: Automated Test Runner** - Orchestrates all test layers with intelligent permission handling

## Quick Start

### Run All Tests (Automated)

```bash
# From project root
./scripts/platform/macos/test_full.sh
```

This script automatically:
- Runs mock tests (always)
- Checks Accessibility permission
- Runs E2E tests (if permission granted)
- Runs benchmarks
- Generates test summary with coverage metrics

### Run Specific Test Layers

```bash
# Mock tests only (no permission required)
cargo test --package keyrx_daemon --test macos_mock_tests

# E2E tests (requires Accessibility permission)
cargo test --package keyrx_daemon --test e2e_macos_basic --test e2e_macos_multidevice

# Check permission status
./scripts/platform/macos/check_permission.sh
echo "Exit code: $?"  # 0 = granted, 1 = denied
```

## Layer 1: Mock Tests

### Purpose

Mock tests validate core platform logic without requiring Accessibility permissions, making them suitable for CI environments and quick local iteration.

### What's Tested

- **Keycode Conversion**: All 140+ CGKeyCode ↔ KeyCode mappings with round-trip validation
- **Edge Cases**: Unknown codes, reserved values, boundary conditions
- **Error Handling**: Platform initialization without permissions
- **Device Discovery**: IOKit enumeration logic with mock responses

### Test File

`keyrx_daemon/tests/macos_mock_tests.rs`

### Running Mock Tests

```bash
# Run all mock tests
cargo test --package keyrx_daemon --test macos_mock_tests

# Run specific test
cargo test --package keyrx_daemon --test macos_mock_tests test_cgkeycode_letters_roundtrip

# Verbose output
cargo test --package keyrx_daemon --test macos_mock_tests -- --nocapture
```

### Key Features

- **Zero Permissions**: No Accessibility permission required
- **Fast Execution**: <1 second for all tests
- **Deterministic**: No hardware or system state dependencies
- **CI-Friendly**: Always runs in GitHub Actions

### Coverage

Mock tests achieve ≥90% coverage of platform/macos/*.rs modules:
- `keycode_map.rs`: 100% (all conversion functions)
- `permissions.rs`: Error paths tested
- `device_discovery.rs`: Mock enumeration logic

## Layer 2: E2E Tests

### Purpose

End-to-end tests validate the complete daemon lifecycle with real macOS APIs, requiring Accessibility permission.

### What's Tested

- **Daemon Lifecycle**: Startup, config loading, graceful shutdown
- **Device Discrimination**: Serial number-based device identification
- **Timing Behavior**: Tap-hold timing validation
- **Real Hardware**: Integration with actual keyboard devices

### Test Files

- `keyrx_daemon/tests/e2e_macos_basic.rs` - Basic lifecycle tests
- `keyrx_daemon/tests/e2e_macos_multidevice.rs` - Multi-device tests
- `keyrx_daemon/tests/e2e_macos_harness.rs` - Test infrastructure

### Running E2E Tests

```bash
# Run all E2E tests
cargo test --package keyrx_daemon --test e2e_macos_basic --test e2e_macos_multidevice

# Run specific test
cargo test --package keyrx_daemon --test e2e_macos_basic test_macos_e2e_basic_remap

# With serial execution (recommended)
cargo test --package keyrx_daemon --test e2e_macos_basic -- --test-threads=1
```

### Permission Checking

All E2E tests auto-skip gracefully without Accessibility permission:

```rust
if !permissions::check_accessibility_permission() {
    eprintln!("\n⚠️  Skipping E2E test: Accessibility permission not granted");
    return; // Exit test gracefully
}
```

This ensures:
- **CI Reliability**: Tests skip with exit code 0 (success) in CI
- **Clear Communication**: Developers see informative skip messages
- **No False Failures**: Missing permissions never fail the build

### Key Features

- **Auto-Skip**: Tests skip gracefully without permissions
- **Process Safety**: Daemon processes never orphaned (Drop cleanup)
- **Serial Execution**: Tests run sequentially with `#[serial]` attribute
- **Graceful Shutdown**: SIGTERM → 5s wait → SIGKILL if needed

### CI Behavior

In GitHub Actions (no Accessibility permission):
1. E2E tests run but skip immediately
2. Exit code: 0 (success)
3. Test results: All "ok" (passed)
4. Logs: Skip messages visible
5. CI status: Job passes ✅

## Layer 3: Automated Test Runner

### Purpose

The test runner script orchestrates all test layers, provides clear progress reporting, and handles permission checking intelligently.

### Script

`scripts/platform/macos/test_full.sh`

### Features

- **Phase Orchestration**: Mock tests → Permission check → E2E tests → Benchmarks
- **Progress Indicators**: Clear ✅ ⚠️ ❌ indicators for each phase
- **Interactive Mode**: Optional manual test prompt on developer machines
- **Non-Interactive Mode**: Auto-skip prompts in CI
- **Graceful Interruption**: Ctrl+C cleanup with trap handlers
- **Test Summary**: Comprehensive report with metrics and recommendations

### Usage

```bash
# Standard run (all phases)
./scripts/platform/macos/test_full.sh

# In CI (non-interactive)
./scripts/platform/macos/test_full.sh  # Prompts auto-skip

# Check exit code
echo $?  # 0 = success, 1 = failure
```

### Output Example

```
========================================
macOS Test Suite
========================================

========================================
Phase 1: Mock Tests
========================================

▶ Running mock tests (no permissions required)...
✅ Mock tests passed

========================================
Phase 2: Accessibility Permission Check
========================================

▶ Checking Accessibility permission...
✅ Accessibility permission granted

========================================
Phase 3: E2E Tests
========================================

▶ Running E2E tests (permission granted)...
✅ E2E tests passed

========================================
Phase 4: Benchmarks
========================================

▶ Running benchmarks...
✅ Benchmarks completed

========================================
Test Summary
========================================

Test Results:

✅ Mock tests: PASSED (15 tests)
✅ E2E tests: PASSED (5 tests)
✅ Coverage: 92.3% (threshold: 80%)
✅ Benchmarks: COMPLETED (123.45 ns 125.67 ns 127.89 ns)

Summary:
  • Total automated tests: 20
  • Test coverage: 92.3%
  • Benchmark result: 123.45 ns 125.67 ns 127.89 ns

Recommendations:
  • None - all tests passed successfully!
  • Excellent coverage! Consider this a benchmark for other modules.

✅ Test suite completed successfully

For more details, see: docs/development/MACOS_TESTING_GUIDE.md
```

## Developer Setup

### Prerequisites

- macOS 10.15+ (Catalina or later)
- Xcode Command Line Tools
- Rust 1.70+
- cargo-tarpaulin (optional, for coverage analysis)

### Install Tools

```bash
# Install Rust (if not already installed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install cargo-tarpaulin (optional)
cargo install cargo-tarpaulin
```

### Grant Accessibility Permission

To run E2E tests on your developer machine:

1. Open **System Settings** (or System Preferences on older macOS)
2. Navigate to **Privacy & Security** → **Accessibility**
3. Click the lock icon to make changes
4. Find your terminal application (Terminal.app, iTerm, or your IDE)
5. Enable the checkbox next to it
6. Close Settings

**Terminal Applications:**
- **Terminal.app**: Default macOS terminal
- **iTerm2**: Popular third-party terminal
- **VS Code**: If running tests from VS Code terminal
- **IntelliJ/CLion**: If running tests from JetBrains IDE

**Verification:**

```bash
./scripts/platform/macos/check_permission.sh
echo $?  # Should print: 0
```

If you see exit code 1, permission was not granted correctly.

### First Test Run

```bash
# Clone repository (if not already done)
cd /path/to/keyrx

# Run complete test suite
./scripts/platform/macos/test_full.sh

# Expected result: All tests pass
```

## Testing Workflows

### Local Development Workflow

```bash
# 1. Make code changes
vim keyrx_daemon/src/platform/macos/keycode_map.rs

# 2. Run mock tests (fast feedback)
cargo test --package keyrx_daemon --test macos_mock_tests

# 3. Run full test suite (if mock tests pass)
./scripts/platform/macos/test_full.sh

# 4. Commit if all tests pass
git add .
git commit -m "feat(macos): Improve keycode conversion"
```

### CI/CD Workflow

GitHub Actions automatically runs the test suite on every push:

```yaml
# .github/workflows/ci.yml
- name: Run Verification (macOS)
  if: matrix.os == 'macos-latest'
  run: |
    cargo test --workspace
    # E2E tests auto-skip without Accessibility permission
```

**Expected CI Behavior:**
- Mock tests: Always run and must pass
- E2E tests: Auto-skip with informative messages
- Exit code: 0 (success)
- Job status: Pass ✅

### Coverage Analysis

```bash
# Run coverage analysis (requires cargo-tarpaulin)
cargo tarpaulin --package keyrx_daemon --target-dir target/tarpaulin \
  --include-tests --exclude-files 'tests/*' --out Stdout

# View HTML report
cargo tarpaulin --package keyrx_daemon --out Html
open target/tarpaulin/tarpaulin-report.html
```

**Coverage Thresholds:**
- **Minimum**: 80% overall
- **Critical paths**: 90% (keycode_map.rs, event handling)
- **FFI bindings**: Excluded (cannot test without permissions)

## Troubleshooting

### Mock Tests Fail

**Symptom:** Mock tests fail with conversion errors

**Solution:**
```bash
# Check if test file is correct
cargo test --package keyrx_daemon --test macos_mock_tests -- --nocapture

# Verify keycode mappings
grep -n "cgkeycode_to_keyrx\|keyrx_to_cgkeycode" keyrx_daemon/src/platform/macos/keycode_map.rs

# Rebuild
cargo clean
cargo build --package keyrx_daemon
```

### E2E Tests Skip Unexpectedly

**Symptom:** E2E tests always skip even though permission was granted

**Possible Causes:**
1. Permission granted to wrong application
2. Terminal app needs restart after granting permission
3. Permission check binary failed to build

**Solution:**
```bash
# 1. Verify permission with check script
./scripts/platform/macos/check_permission.sh
echo $?  # Should be 0

# 2. Restart terminal application
# Quit and reopen Terminal/iTerm/IDE

# 3. Check for permission in System Settings
# Ensure your terminal app is listed and enabled

# 4. Try running check script with explicit path
/usr/bin/env bash ./scripts/platform/macos/check_permission.sh
```

### E2E Tests Hang

**Symptom:** E2E tests start but never complete

**Causes:**
- Daemon process not shutting down
- SIGTERM not received
- Infinite loop in test harness

**Solution:**
```bash
# 1. Kill orphaned processes
pkill -9 keyrx_daemon

# 2. Check for orphaned processes
ps aux | grep keyrx_daemon

# 3. Run with verbose logging
RUST_LOG=debug cargo test --package keyrx_daemon --test e2e_macos_basic -- --nocapture

# 4. If problem persists, check harness Drop implementation
grep -A 10 "impl Drop for MacosE2EHarness" keyrx_daemon/tests/e2e_macos_harness.rs
```

### Test Runner Script Fails

**Symptom:** `test_macos_full.sh` exits with error

**Solution:**
```bash
# 1. Check script permissions
ls -la scripts/platform/macos/test_full.sh
chmod +x scripts/platform/macos/test_full.sh  # If not executable

# 2. Run with debug output
bash -x ./scripts/platform/macos/test_full.sh

# 3. Check for syntax errors
shellcheck scripts/platform/macos/test_full.sh  # If shellcheck installed

# 4. Verify dependencies
which cargo  # Should print: /path/to/cargo
cargo --version  # Should print version
```

### Coverage Tool Not Installed

**Symptom:** "cargo-tarpaulin not found"

**Solution:**
```bash
# Install tarpaulin
cargo install cargo-tarpaulin

# Verify installation
cargo tarpaulin --version

# Alternative: Skip coverage analysis
# The test runner will show a warning but continue
```

### Permission Denied Errors

**Symptom:** "Operation not permitted" during test execution

**Causes:**
- Accessibility permission not granted
- Incorrect application in Accessibility list
- System Integrity Protection blocking

**Solution:**
```bash
# 1. Re-grant Accessibility permission
# System Settings > Privacy & Security > Accessibility
# Remove and re-add your terminal application

# 2. Restart terminal
# Quit and reopen

# 3. Check SIP status (should be enabled)
csrutil status

# 4. If running in VM, ensure nested permissions are correct
```

## Manual Testing

While automated tests cover most scenarios, some aspects require manual verification.

### When to Run Manual Tests

- After implementing new key mappings
- When testing tap-hold timing accuracy
- When verifying multi-device discrimination
- Before releases (UAT)

### Manual Test Procedures

#### 1. Basic Remapping Test

**Config:** A → B remapping

```bash
# 1. Compile test config
cd keyrx_daemon
cargo run --bin keyrx_compiler -- --config test_configs/a_to_b.rhai --output /tmp/test.krx

# 2. Start daemon
cargo run --bin keyrx_daemon -- --config /tmp/test.krx

# 3. Open text editor
open -a TextEdit

# 4. Press 'A' key
# Expected: 'B' appears

# 5. Press other keys
# Expected: Normal behavior

# 6. Stop daemon (Ctrl+C)
```

#### 2. Tap-Hold Timing Test

**Config:** Space tap → Space, Space hold → Shift

```bash
# 1. Compile tap-hold config
cargo run --bin keyrx_compiler -- --config test_configs/space_tap_hold.rhai --output /tmp/test.krx

# 2. Start daemon
cargo run --bin keyrx_daemon -- --config /tmp/test.krx

# 3. Open text editor

# 4. Quick tap Space
# Expected: Space character (word separator)

# 5. Hold Space + press 'a'
# Expected: Capital 'A' (Shift+A)

# 6. Measure timing
# Expected: Hold activation within 200ms
```

#### 3. Multi-Device Test

**Prerequisites:** 2+ keyboards connected

```bash
# 1. List devices
cargo run --bin keyrx_daemon -- --list-devices

# Output example:
# Device 1: Apple Internal Keyboard (serial: ABC123)
# Device 2: Logitech K380 (serial: XYZ789)

# 2. Create device-specific configs
# Config 1: Device ABC123 - A → B
# Config 2: Device XYZ789 - A → C

# 3. Start daemon with device configs

# 4. Press 'A' on Device 1
# Expected: 'B' appears

# 5. Press 'A' on Device 2
# Expected: 'C' appears
```

### Manual Test Checklist

```
□ Basic remapping works (A → B)
□ Modifiers work (Shift, Ctrl, Alt, Cmd)
□ Function keys work (F1-F12)
□ Special keys work (Arrows, Page Up/Down, Home/End)
□ Tap-hold timing feels natural (<1ms latency)
□ No double keypresses
□ No stuck modifiers
□ Multi-device discrimination works (if applicable)
□ Daemon logs show no errors
□ Graceful shutdown works (Ctrl+C)
```

## Performance Benchmarks

KeyRx includes benchmarks to measure processing latency.

### Running Benchmarks

```bash
# Run all benchmarks
cargo bench --package keyrx_daemon --bench macos_latency

# Run specific benchmark
cargo bench --package keyrx_daemon --bench macos_latency keycode_conversion
```

### Expected Results

- **Keycode Conversion**: <100 ns per conversion
- **Event Processing**: <1,000 ns (<1 µs) per event
- **End-to-End Latency**: <1 ms user-perceptible

### Performance Thresholds

| Metric | Threshold | Status |
|--------|-----------|--------|
| Keycode conversion | <100 ns | ✅ Pass |
| Event processing | <1 µs | ✅ Pass |
| E2E latency | <1 ms | ✅ Pass |

## Platform Differences

### macOS vs Linux/Windows

| Feature | macOS | Linux | Windows |
|---------|-------|-------|---------|
| Virtual device | ❌ No uinput | ✅ uinput | ✅ Interception |
| Permission model | Accessibility | None | Admin |
| Device enumeration | IOKit | evdev | WinAPI |
| Event injection | CGEvent | uinput | SendInput |
| CI-friendly E2E | ❌ Skip | ✅ Run | ✅ Run |

### macOS Limitations

1. **No Virtual Keyboard**: macOS has no uinput equivalent, so E2E tests cannot inject/capture events programmatically
2. **Permission Requirement**: Accessibility permission required for real event capture
3. **CI Constraints**: E2E tests must auto-skip in CI (no permission available)

### macOS Advantages

1. **Comprehensive Mock Tests**: 90%+ coverage without permissions
2. **Fast Iteration**: Mock tests provide quick feedback
3. **Reliable CI**: Mock tests always run, E2E auto-skip gracefully

## Integration with CI

### GitHub Actions Configuration

The CI workflow includes macOS testing:

```yaml
# .github/workflows/ci.yml
test-macos:
  runs-on: macos-latest
  steps:
    - uses: actions/checkout@v3
    - name: Setup Rust
      uses: actions-rs/toolchain@v1
      with:
        profile: minimal
        toolchain: stable
    - name: Run Tests
      run: |
        cargo test --workspace
        # E2E tests auto-skip without Accessibility permission
```

### CI Expectations

✅ **Expected Behavior:**
- Mock tests: Run and must pass
- E2E tests: Auto-skip with informative messages
- Exit code: 0 (success)
- Job duration: <10 minutes

❌ **Failure Conditions:**
- Mock tests fail
- Build errors
- Clippy warnings
- Format violations

## References

### Documentation

- **Architecture**: `docs/development/architecture.md`
- **Windows Testing**: `docs/development/windows-vm-setup.md`
- **E2E CI Verification**: `docs/development/MACOS_E2E_CI_VERIFICATION.md`
- **Project Guide**: `.claude/CLAUDE.md`

### Code Locations

- **Mock Tests**: `keyrx_daemon/tests/macos_mock_tests.rs`
- **E2E Tests**: `keyrx_daemon/tests/e2e_macos_*.rs`
- **Test Harness**: `keyrx_daemon/tests/e2e_macos_harness.rs`
- **Test Runner**: `scripts/platform/macos/test_full.sh`
- **Permission Checker**: `scripts/platform/macos/check_permission.sh`
- **Platform Code**: `keyrx_daemon/src/platform/macos/`

### Spec Documents

- **Requirements**: `.spec-workflow/specs/macos-testing-automation/requirements.md`
- **Design**: `.spec-workflow/specs/macos-testing-automation/design.md`
- **Tasks**: `.spec-workflow/specs/macos-testing-automation/tasks.md`

### External Resources

- **macOS Accessibility**: https://support.apple.com/guide/mac-help/allow-accessibility-apps-mchlc54b4b01/mac
- **CGEvent API**: https://developer.apple.com/documentation/coregraphics/cgevent
- **IOKit**: https://developer.apple.com/documentation/iokit
- **Rust Testing**: https://doc.rust-lang.org/book/ch11-00-testing.html
