# Code Quality Metrics

This document tracks code quality metrics for the keyrx2 project, aligned with quality standards specified in `.claude/CLAUDE.md`.

## Quality Gates

### 1. Clippy Warnings
- **Target:** 0 warnings
- **Command:** `cargo clippy --workspace -- -D warnings`
- **Status:** ✓ PASSING
- **Last Check:** 2025-12-29

All clippy warnings must be resolved before merging. Warnings are treated as errors via `-D warnings` flag.

### 2. Code Formatting
- **Target:** 100% formatted
- **Command:** `cargo fmt --check`
- **Status:** ✓ PASSING
- **Last Check:** 2025-12-29

Code must be formatted with `cargo fmt` before committing. Pre-commit hooks enforce this.

### 3. Test Coverage
- **Overall Target:** ≥80%
- **Business Logic Target:** ≥90%
- **Command:** `cargo llvm-cov --workspace --json`
- **Status:** Measurement in progress
- **Last Check:** 2025-12-29

**Business Logic Modules** (90% target):
- `keyrx_daemon/src/config/device_registry.rs`
- `keyrx_daemon/src/config/profile_manager.rs`
- `keyrx_daemon/src/config/simulation_engine.rs`
- `keyrx_daemon/src/config/rhai_generator.rs`
- `keyrx_daemon/src/config/layout_manager.rs`

**CLI Modules** (80% target):
- `keyrx_daemon/src/cli/*.rs`

**Web API** (75% target):
- `keyrx_daemon/src/web/api.rs`

### 4. File Size Limits
- **Target:** ≤500 lines per file (excluding comments/blank lines)
- **Function Size:** ≤50 lines per function
- **Status:** ⚠ 15 violations (pre-existing, documented for refactoring)
- **Last Check:** 2025-12-29

**Files Exceeding Limit:**
| File | Lines | Excess |
|------|-------|--------|
| `keyrx_daemon/src/platform/linux/mod.rs` | 1948 | +1448 |
| `keyrx_daemon/src/test_utils/output_capture.rs` | 1664 | +1164 |
| `keyrx_daemon/src/daemon/mod.rs` | 1591 | +1091 |
| `keyrx_daemon/src/cli/config.rs` | 893 | +393 |
| `keyrx_daemon/src/platform/linux/keycode_map.rs` | 872 | +372 |
| `keyrx_daemon/src/main.rs` | 846 | +346 |
| `keyrx_daemon/src/config/simulation_engine.rs` | 678 | +178 |
| `keyrx_daemon/src/config/rhai_generator.rs` | 649 | +149 |
| `keyrx_daemon/src/config/layout_manager.rs` | 613 | +113 |
| `keyrx_daemon/src/config/profile_manager.rs` | 593 | +93 |
| `keyrx_daemon/src/cli/profiles.rs` | 589 | +89 |
| `keyrx_daemon/src/test_utils/virtual_keyboard.rs` | 582 | +82 |
| `keyrx_daemon/src/platform/windows/rawinput.rs` | 550 | +50 |
| `keyrx_daemon/src/config/device_registry.rs` | 524 | +24 |

**Refactoring Plan:**
- `platform/linux/mod.rs`: Split into sub-modules for evdev, uinput, device management
- `daemon/mod.rs`: Extract state machine, event loop, IPC server
- `cli/config.rs`: Split subcommands into separate files
- `test_utils/output_capture.rs`: Extract helper functions to utility module

### 5. Documentation Coverage
- **Target:** 100% of public API items
- **Command:** `cargo doc --no-deps`
- **Status:** In progress
- **Last Check:** 2025-12-29

All `pub` items (functions, structs, enums, traits) must have documentation comments explaining:
- Purpose and behavior
- Parameters and return values
- Examples (for complex APIs)
- Error conditions

## Quality Validation

Run comprehensive quality validation:

```bash
./scripts/validate_quality.sh
```

This script checks:
1. ✓ Clippy (zero warnings)
2. ✓ Format (all code formatted)
3. ⚠ File sizes (15 violations documented)
4. ✓ Tests (all passing)
5. Coverage (≥80% overall, ≥90% business logic)
6. Documentation (public API coverage)

## Continuous Integration

Quality gates are enforced in CI via `.github/workflows/ci.yml`:
- Pre-commit hooks run clippy + format checks
- CI runs full test suite + coverage analysis
- Benchmark regressions detected and reported
- Coverage reports uploaded as artifacts

## Metrics History

### 2025-12-29
- Clippy: ✓ 0 warnings
- Format: ✓ 100% formatted
- File Size: ⚠ 15 violations (pre-existing, will refactor)
- Tests: ✓ All passing
- Coverage: Measurement in progress
- Docs: Public API documentation in progress

## Improvement Targets

### Short-term (v1.0)
- [ ] Achieve ≥80% overall coverage
- [ ] Document all public API items
- [ ] Fix clippy warnings in benchmarks

### Medium-term (v1.1)
- [ ] Refactor files >500 lines (reduce violations to 0)
- [ ] Achieve ≥90% coverage on business logic
- [ ] Add property-based tests for core logic

### Long-term (v2.0)
- [ ] Maintain 0 technical debt (no clippy warnings, all tests passing)
- [ ] Achieve ≥95% overall coverage
- [ ] Zero file size violations
