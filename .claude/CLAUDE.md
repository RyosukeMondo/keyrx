# AI Agent Development Guide

## Active Specs

- **uat-ui-fixes**: Dashboard virtual/physical indicator, device enable/disable toggle, profile inline edit + active indicator, config RPC fix, 256-layer display, key dropdown population. See `.spec-workflow/specs/uat-ui-fixes/tasks.md`

## Windows Testing on Linux (Vagrant VM)

**Testing Windows code on Linux is fully supported via Vagrant.**

### Quick Start

```bash
# From project root - automated testing
./scripts/windows_test_vm.sh

# Manual control
cd vagrant/windows
vagrant up              # Start Windows VM (first time: ~20 min)
vagrant winrm -c 'cd C:\vagrant_project; cargo test -p keyrx_daemon --features windows'
vagrant halt            # Stop VM

# Restore to clean state anytime
vagrant snapshot restore provisioned
```

### VM Specifications

- **OS**: Windows 10 Enterprise with Visual Studio Build Tools
- **Tools**: Rust 1.92.0, Git, MSVC compiler (link.exe)
- **RAM**: 16GB, 4 CPU cores
- **Access**: SSH, RDP (localhost:13389), GUI (virt-manager)
- **Sync**: Project root → `C:\vagrant_project` (use `vagrant rsync` after changes)

### Documentation

- Quick guide: `vagrant/windows/README.md`
- Full guide: `docs/development/windows-vm-setup.md`
- Helper script: `scripts/windows_test_vm.sh`

**Important**: VM snapshot "provisioned" saves ~15 minutes of setup time. Always restore from snapshot after testing.

## AI-Agent Quick Start

### 1. Verify Environment

**Check Rust toolchain:**
```bash
rustc --version  # Requires Rust 1.70+
cargo --version
```

**Check Node.js (for keyrx_ui):**
```bash
node --version   # Requires Node.js 18+
npm --version
```

### 2. Setup Development Environment

**Install required tools:**
```bash
make setup
```

This installs:
- `cargo-watch` - Continuous build on file changes
- `cargo-tarpaulin` - Code coverage analysis
- `cargo-fuzz` - Fuzzing support
- `wasm-pack` - WASM compilation
- Git pre-commit hooks - Automated quality gates

### 3. Run First Build

**Build all crates:**
```bash
make build
# Or: scripts/build.sh
```

Expected output:
```
[2025-12-21 10:30:45] [INFO] Building workspace...
[2025-12-21 10:30:50] === accomplished ===
```

### 4. Run Tests

**Execute all tests:**
```bash
make test
# Or: scripts/test.sh
```

### 5. Run Verification

**Run complete quality checks (clippy, fmt, tests, coverage):**
```bash
make verify
# Or: scripts/verify.sh
```

This runs:
- Cargo build (clean workspace build)
- Clippy linting (-D warnings, treats warnings as errors)
- Rustfmt check (code formatting)
- Cargo test (all unit and integration tests)
- Coverage analysis (80% minimum required)

**Success criteria:** All checks must pass before committing code.

## Project Structure

### 4-Crate Workspace

```
keyrx2/
├── Cargo.toml              # Workspace root
├── Makefile                # Top-level commands
├── .github/workflows/      # CI/CD automation
│   ├── ci.yml              # Automated verification
│   └── release.yml         # Release builds
├── keyrx_core/             # Core library (no_std, WASM-compatible)
│   ├── src/
│   │   ├── lib.rs          # Public API exports
│   │   ├── config.rs       # rkyv-serialized config structures
│   │   ├── lookup.rs       # MPHF-based O(1) key lookup
│   │   ├── dfa.rs          # Deterministic Finite Automaton
│   │   ├── state.rs        # 255-bit modifier/lock state
│   │   └── simulator.rs    # Deterministic Simulation Testing
│   ├── benches/            # Criterion benchmarks
│   └── fuzz/               # cargo-fuzz targets
├── keyrx_compiler/         # Rhai-to-binary compiler (CLI)
│   ├── src/
│   │   ├── main.rs         # CLI entry point
│   │   ├── parser.rs       # Rhai AST evaluation
│   │   ├── mphf_gen.rs     # MPHF generation
│   │   ├── dfa_gen.rs      # DFA compilation
│   │   └── serialize.rs    # rkyv binary output
│   └── tests/integration/  # Integration tests
├── keyrx_daemon/           # OS-specific daemon + web server
│   ├── src/
│   │   ├── main.rs         # Daemon entry point
│   │   ├── platform/       # Platform-specific code
│   │   │   ├── mod.rs      # Platform trait abstraction
│   │   │   ├── linux.rs    # evdev/uinput
│   │   │   └── windows.rs  # Low-Level Hooks
│   │   └── web/            # Embedded web server (optional)
│   │       ├── mod.rs      # axum server setup
│   │       ├── api.rs      # REST API
│   │       ├── ws.rs       # WebSocket handler
│   │       └── static_files.rs # Serve embedded UI
│   └── ui_dist/            # Embedded UI files
├── keyrx_ui/               # React + WASM frontend
│   ├── src/
│   │   ├── App.tsx         # Root component
│   │   ├── components/     # React components
│   │   ├── wasm/           # WASM bindings
│   │   └── hooks/          # React hooks
│   └── vite.config.ts      # Vite bundler config
└── scripts/                # Build/test/launch automation
    ├── build.sh            # Build workspace
    ├── verify.sh           # Quality checks
    ├── test.sh             # Run tests
    ├── launch.sh           # Start daemon
    ├── setup_hooks.sh      # Install pre-commit hooks
    ├── lib/common.sh       # Shared utilities
    ├── logs/               # Timestamped execution logs
    └── CLAUDE.md           # Script documentation
```

### Crate Purposes

| Crate | Type | Purpose | Key Dependencies |
|-------|------|---------|------------------|
| `keyrx_core` | Library (no_std) | Platform-agnostic remapping logic | rkyv, boomphf, fixedbitset, arrayvec |
| `keyrx_compiler` | Binary | Compile Rhai configs to .krx binaries | rhai, serde, clap |
| `keyrx_daemon` | Binary | OS-level keyboard interception + web server | evdev (Linux), windows-sys (Windows), axum (web) |
| `keyrx_ui` | Frontend | React-based web interface with WASM | React 18+, TypeScript 5+, Vite |

## Code Quality Rules

### File and Function Size Limits

**Enforced by clippy and pre-commit hooks:**

- **Maximum 500 lines per file** (excluding comments and blank lines)
  - If exceeded: Extract helper modules or split into sub-modules
- **Maximum 50 lines per function**
  - If exceeded: Extract helper functions, apply SLAP (Single Level of Abstraction Principle)

### Test Coverage

- **Minimum 80% coverage** (enforced by verify.sh)
- **Minimum 90% coverage for critical paths** (keyrx_core)
- Coverage measured by `cargo tarpaulin`
- Coverage reports uploaded to CI artifacts

### Code Quality Checks

All checks enforced by pre-commit hooks and CI:

1. **Clippy**: `cargo clippy --workspace -- -D warnings`
   - Treats warnings as errors
   - Enforces best practices and idiomatic code
2. **Rustfmt**: `cargo fmt --check`
   - Consistent code formatting
   - Fails if code is not formatted
3. **Tests**: `cargo test --workspace`
   - All tests must pass
   - No ignored tests in production code

### Production Quality Gates

All quality gates are enforced in CI/CD pipeline (.github/workflows/ci.yml). Some gates are currently warnings due to known issues being addressed.

#### Backend Quality Gates (Strict - Blocks Merge)

**1. Backend Tests and Verification**
```bash
# Run locally
make verify
# Or: scripts/verify.sh

# CI enforces:
# - Cargo build (workspace compilation)
# - Cargo clippy (no warnings)
# - Cargo fmt --check (formatting)
# - Cargo test (all tests pass)
```

**Status**: ✅ 962/962 tests passing (100%)

**2. Backend Documentation Tests**
```bash
# Run locally
scripts/fix_doc_tests.sh

# Or manually:
cargo clean
cargo build --workspace
cargo test --doc
```

**Status**: ✅ 9/9 doc tests passing (100%)
**Requirement**: All documentation examples must compile and execute correctly

#### Frontend Quality Gates

**3. Frontend Tests (Currently Warning - Will Become Strict)**
```bash
# Run locally
cd keyrx_ui
npm test

# Check specific test files
npm test -- ProfilesPage.test.tsx
```

**Status**: ⚠️ 681/897 tests passing (75.9%)
**Target**: ≥95% pass rate (≥852/897 tests)
**Blocker**: WebSocket mock infrastructure needs improvement
**Note**: Currently non-blocking in CI (continue-on-error: true). Will become strict once WebSocket infrastructure is fixed.

**4. Frontend Coverage (Currently Warning)**
```bash
# Run locally
cd keyrx_ui
npm run test:coverage

# View HTML report
npm run test:coverage && open coverage/index.html
```

**Status**: ⚠️ Limited due to test failures
**Target**: ≥80% line and branch coverage
**Critical Components Verified**:
- MonacoEditor: 85.91% line, 90.32% branch ✅
- useAutoSave: 100% line, 90.62% branch ✅

**5. Accessibility Audit (Strict - Blocks Merge)**
```bash
# Run locally
cd keyrx_ui
npm run test:a11y

# Run specific accessibility tests
npm test -- a11y
```

**Status**: ✅ 23/23 tests passing (100%)
**Requirement**: Zero WCAG 2.2 Level AA violations
**Enforcement**: Strict - any violation blocks merge

**WCAG Criteria Verified**:
- 1.4.3 Color Contrast (≥4.5:1 normal text, ≥3:1 large text)
- 2.1.1 Keyboard Accessibility (all interactive elements)
- 2.1.2 No Keyboard Trap
- 2.4.7 Focus Visible (clear focus indicators)
- 4.1.2 ARIA Labels and Semantic HTML

#### Running All Quality Gates Locally

**Complete verification (backend + frontend):**
```bash
# Backend
make verify                    # Tests, clippy, fmt, coverage
scripts/fix_doc_tests.sh       # Doc tests

# Frontend
cd keyrx_ui
npm test                       # All tests
npm run test:coverage          # Coverage analysis
npm run test:a11y              # Accessibility audit
```

**Quick pre-commit check:**
```bash
make verify && cd keyrx_ui && npm test && npm run test:a11y
```

#### Quality Gate Thresholds Summary

| Quality Gate | Threshold | Current Status | Enforcement |
|--------------|-----------|----------------|-------------|
| Backend Tests | 100% pass | ✅ 962/962 (100%) | Strict |
| Backend Doc Tests | 100% pass | ✅ 9/9 (100%) | Strict |
| Frontend Tests | ≥95% pass rate | ⚠️ 681/897 (75.9%) | Warning* |
| Frontend Coverage | ≥80% line/branch | ⚠️ Blocked by tests | Warning* |
| Accessibility | Zero WCAG violations | ✅ 23/23 (100%) | Strict |

*Currently warnings in CI due to known WebSocket infrastructure issues. Will become strict enforcement once fixed.

#### Fixing Quality Gate Failures

**Backend test failures:**
1. Run `cargo test --workspace` to see failures
2. Fix failing tests
3. Re-run `make verify` to confirm

**Frontend test failures:**
1. Run `npm test` to see failures
2. Check error messages for context (missing providers, async issues)
3. Update tests to use `renderWithProviders` from `tests/testUtils.tsx`
4. Ensure async operations use `waitFor` or `findBy` queries

**Coverage below threshold:**
1. Run `npm run test:coverage` to see uncovered lines
2. Write tests for uncovered code paths
3. Focus on critical paths first (aim for ≥90%)
4. Re-run coverage to verify improvement

**Accessibility violations:**
1. Run `npm run test:a11y` to see specific violations
2. Check violation output for WCAG criterion and element
3. Fix violations (add ARIA labels, improve contrast, fix semantic HTML)
4. Re-run to verify zero violations

**Doc test failures:**
1. Run `scripts/fix_doc_tests.sh` to see failures
2. Update documentation examples to match current API
3. Ensure examples compile and execute correctly
4. Re-run to verify all pass

#### CI/CD Integration

Quality gates run automatically on every push and PR:
- Ubuntu: All gates enforced
- Windows: Basic verification only (build, clippy, tests)

**Viewing CI results:**
- Check GitHub Actions for quality gate summary
- Download artifacts for detailed reports (coverage, accessibility)
- CI logs show pass/fail for each gate with metrics

**Production readiness report:**
- See `.spec-workflow/specs/production-readiness-remediation/PRODUCTION_READINESS_REPORT.md`
- Current status: Backend ✅ READY, Frontend ⚠️ CONDITIONAL

## Architecture Patterns

### SOLID Principles

**S - Single Responsibility Principle:**
```rust
// Good: Each module has one clear purpose
// lookup.rs - handles MPHF lookup only
// dfa.rs - handles DFA state transitions only

// Bad: lookup.rs mixing lookup + DFA + state management
```

**O - Open/Closed Principle:**
```rust
// Good: Extensible via traits
pub trait Platform {
    fn capture_input(&mut self) -> Result<KeyEvent>;
    fn inject_output(&mut self, event: KeyEvent) -> Result<()>;
}

// Add new platforms by implementing trait, no changes to core
```

**L - Liskov Substitution Principle:**
```rust
// Platform implementations must be substitutable
fn process_events<P: Platform>(platform: &mut P) {
    // Works with any Platform implementation
}
```

**I - Interface Segregation Principle:**
```rust
// Good: Small, focused traits
pub trait EventCapture {
    fn capture(&mut self) -> Result<KeyEvent>;
}

pub trait EventInjection {
    fn inject(&mut self, event: KeyEvent) -> Result<()>;
}

// Bad: One giant trait with all methods
```

**D - Dependency Inversion Principle:**
```rust
// Good: Depend on abstractions (traits), not concrete types
pub fn process_events<S: EventStream>(stream: &mut S) {
    // Testable, mockable
}

// Bad: Hard-coded concrete dependency
pub fn process_events() {
    let stream = evdev::open(); // NOT testable
}
```

### Dependency Injection Pattern

**All external dependencies injected:**
```rust
// Platform-specific code abstracted via traits
pub trait Platform {
    fn capture_input(&mut self) -> Result<KeyEvent>;
    fn inject_output(&mut self, event: KeyEvent) -> Result<()>;
}

// Inject dependency for testability
pub struct Daemon<P: Platform> {
    platform: P,
}

impl<P: Platform> Daemon<P> {
    pub fn new(platform: P) -> Self {
        Self { platform }
    }

    pub fn run(&mut self) -> Result<()> {
        loop {
            let event = self.platform.capture_input()?;
            let action = self.process(event)?;
            self.platform.inject_output(action)?;
        }
    }
}

// Test with mock implementation
#[cfg(test)]
mod tests {
    struct MockPlatform;
    impl Platform for MockPlatform {
        fn capture_input(&mut self) -> Result<KeyEvent> { /* ... */ }
        fn inject_output(&mut self, event: KeyEvent) -> Result<()> { /* ... */ }
    }

    #[test]
    fn test_daemon() {
        let mut daemon = Daemon::new(MockPlatform);
        // Test without real OS dependencies
    }
}
```

### SSOT (Single Source of Truth)

**Configuration:**
- `.krx` binary file is the ONLY config source
- Daemon, UI, tests all read same `.krx` file
- No duplication in JSON, TOML, or other formats
- Hash-based verification ensures integrity

**State:**
- `ExtendedState` struct is the ONLY state representation
- No shadow copies, no stale caches
- All components reference the same state

**Example:**
```rust
// Good: Single source of truth
pub struct Config {
    krx_data: &'static [u8],  // Memory-mapped .krx file
}

// Bad: Multiple representations
// pub struct JsonConfig { /* ... */ }
// pub struct TomlConfig { /* ... */ }
```

### KISS (Keep It Simple, Stupid)

**Prefer simplicity over premature optimization:**
```rust
// Good: Simple, clear logic
pub fn is_modifier_active(state: &State, modifier_id: u8) -> bool {
    state.modifiers.contains(modifier_id)
}

// Bad: Over-engineered
pub fn is_modifier_active<S: StateProvider, M: ModifierId>(
    state: &S,
    modifier: M,
    cache: &mut ModifierCache,
) -> Result<bool, ModifierError> {
    // Unnecessary complexity
}
```

**Only add complexity when required:**
- Don't add features not explicitly needed
- Don't abstract until you have 3+ similar cases
- Don't optimize without profiling first

## Naming Conventions

### Rust Code

| Element | Convention | Example |
|---------|-----------|---------|
| Modules | `snake_case` | `mphf_gen`, `static_files` |
| Files | `snake_case.rs` | `lookup.rs`, `dfa_gen.rs` |
| Functions | `snake_case` | `load_config()`, `process_event()` |
| Structs | `PascalCase` | `ExtendedState`, `EventStream` |
| Enums | `PascalCase` | `EventType`, `ModifierState` |
| Traits | `PascalCase` | `Platform`, `EventCapture` |
| Constants | `UPPER_SNAKE_CASE` | `MAX_MODIFIERS`, `DEFAULT_PORT` |
| Variables | `snake_case` | `modifier_state`, `event_queue` |
| Type parameters | `PascalCase` or single uppercase | `T`, `EventType` |

### TypeScript/React Code

| Element | Convention | Example |
|---------|-----------|---------|
| Components | `PascalCase` | `KeyboardVisualizer`, `DFADiagram` |
| Files (components) | `PascalCase.tsx` | `App.tsx`, `KeyboardVisualizer.tsx` |
| Functions | `camelCase` | `connectToDaemon()`, `parseEvent()` |
| Hooks | `use[Feature]` | `useSimulator()`, `useDaemon()` |
| Files (hooks) | `use[Feature].ts` | `useSimulator.ts`, `useDaemon.ts` |
| Interfaces/Types | `PascalCase` | `DaemonState`, `KeyEvent` |
| Constants | `UPPER_SNAKE_CASE` | `WS_PORT`, `MAX_RETRIES` |
| Variables | `camelCase` | `eventQueue`, `modifierState` |

### Files and Directories

| Element | Convention | Example |
|---------|-----------|---------|
| Rust modules | `snake_case.rs` | `config.rs`, `lookup.rs` |
| Test files | `[module]_test.rs` | `dfa_test.rs` |
| Benchmark files | `[feature]_bench.rs` | `lookup_bench.rs` |
| Scripts | `lowercase.sh` | `build.sh`, `verify.sh` |
| Log files | `[script]_[epoch].log` | `build_1766292917.log` |

## Import Patterns

### Rust Import Order

```rust
// 1. Standard library
use std::collections::HashMap;
use std::fs::File;

// 2. External dependencies (alphabetically)
use rkyv::{Archive, Serialize};
use serde::Deserialize;

// 3. Internal workspace crates
use keyrx_core::{EventStream, State};
use keyrx_compiler::Parser;

// 4. Current crate modules (relative)
use crate::config::Config;
use super::utils;

// Example from keyrx_daemon/src/platform/linux.rs:
use std::os::unix::io::AsRawFd;

use evdev::{Device, InputEventKind};
use nix::ioctl_write_int_bad;

use keyrx_core::{EventStream, KeyEvent};

use crate::platform::Platform;
```

### TypeScript Import Order

```typescript
// 1. React and framework
import React, { useState, useEffect } from 'react';

// 2. External dependencies
import { WebSocket } from 'ws';

// 3. Internal modules (absolute from src/)
import { WasmCore } from '@/wasm/core';
import type { KeyEvent } from '@/types';

// 4. Relative imports
import { Button } from './Button';

// 5. Styles (last)
import './App.css';

// Example from keyrx_ui/src/App.tsx:
import React, { useState } from 'react';

import { WasmCore } from '@/wasm/core';
import { useDaemon } from '@/hooks/useDaemon';

import { KeyboardVisualizer } from './components/KeyboardVisualizer';

import './App.css';
```

### Module Organization

**Absolute imports between crates:**
```rust
// Good: Workspace-relative imports
use keyrx_core::config::Config;

// Bad: Relative paths between crates
use ../keyrx_core/src/config.rs;  // DON'T DO THIS
```

**Re-exports for public API:**
```rust
// keyrx_core/src/lib.rs
pub use self::config::Config;
pub use self::dfa::DFA;
pub use self::state::State;

// Users import from crate root:
use keyrx_core::{Config, DFA, State};
```

**Feature gates for optional dependencies:**
```rust
#[cfg(feature = "web")]
pub mod web;

#[cfg(feature = "linux")]
pub mod linux;
```

## Common Tasks

### How to Add a New Module

**Example: Adding a new module `validator.rs` to keyrx_core**

1. **Create the module file:**
```bash
touch keyrx_core/src/validator.rs
```

2. **Declare the module in lib.rs:**
```rust
// keyrx_core/src/lib.rs
pub mod validator;
```

3. **Implement the module:**
```rust
// keyrx_core/src/validator.rs
//! Input validation for configuration data.

use crate::config::Config;

/// Validates a configuration for correctness.
pub fn validate_config(config: &Config) -> Result<(), ValidationError> {
    // Validation logic
    Ok(())
}

#[derive(Debug)]
pub enum ValidationError {
    InvalidModifier,
    InvalidKeyCode,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_config() {
        // Test validation logic
    }
}
```

4. **Re-export if public API:**
```rust
// keyrx_core/src/lib.rs
pub use self::validator::{validate_config, ValidationError};
```

5. **Add tests:**
```bash
# Tests are in the same file (see above)
# Or create integration test:
touch keyrx_core/tests/validator_integration.rs
```

6. **Verify:**
```bash
make verify
```

### How to Add a Test

**Unit test (in same file as code):**
```rust
// keyrx_core/src/lookup.rs
pub fn lookup_key(hash: u64) -> Option<KeyCode> {
    // Implementation
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lookup_key_found() {
        let result = lookup_key(12345);
        assert!(result.is_some());
    }

    #[test]
    fn test_lookup_key_not_found() {
        let result = lookup_key(99999);
        assert!(result.is_none());
    }
}
```

**Integration test (separate file):**
```bash
# Create test file
touch keyrx_core/tests/dfa_integration.rs
```

```rust
// keyrx_core/tests/dfa_integration.rs
use keyrx_core::{DFA, State, KeyEvent};

#[test]
fn test_dfa_tap_hold_sequence() {
    let mut dfa = DFA::new();
    let mut state = State::new();

    // Test tap/hold logic
    let event1 = KeyEvent::new(/* ... */);
    let result = dfa.process(&mut state, event1);
    assert_eq!(result, /* expected */);
}
```

**Run the tests:**
```bash
# Run all tests
make test

# Run specific test
cargo test test_lookup_key_found

# Run tests in a specific crate
cargo test -p keyrx_core

# Run integration tests only
scripts/test.sh --integration
```

### How to Run Specific Tests

```bash
# Run all tests
cargo test

# Run tests matching a pattern
cargo test lookup

# Run tests in a specific crate
cargo test -p keyrx_core

# Run a specific test
cargo test test_lookup_key_found

# Run only unit tests (lib tests)
scripts/test.sh --unit

# Run only integration tests
scripts/test.sh --integration

# Run with verbose output
cargo test -- --nocapture

# Run benchmarks (requires nightly)
scripts/test.sh --bench
```

### How to Test Windows Code on Linux

**Use the Vagrant Windows VM for testing Windows-specific code:**

```bash
# Automated - runs tests and returns results
./scripts/windows_test_vm.sh

# Manual - full control over the VM
cd vagrant/windows
vagrant up                    # Start VM (first time: ~20 min with provisioning)
vagrant winrm -c 'cd C:\vagrant_project; cargo test -p keyrx_daemon --features windows'
vagrant halt                  # Stop VM

# After making changes on Linux
vagrant rsync                 # Sync files to Windows VM
vagrant winrm -c 'cd C:\vagrant_project; cargo test -p keyrx_daemon --features windows'

# Restore to clean state
vagrant snapshot restore provisioned

# Check VM status
vagrant status
```

**Important notes:**
- The VM has all tools pre-installed: Rust, MSVC compiler, Git
- Project root syncs to `C:\vagrant_project` in the VM
- Always use `vagrant winrm` (not `vagrant ssh`) for Windows VMs
- Snapshot "provisioned" saves 15+ minutes of setup time
- See `vagrant/windows/README.md` for detailed documentation

**Common Windows testing commands:**
```bash
# Build Windows daemon
vagrant winrm -c 'cd C:\vagrant_project; cargo build -p keyrx_daemon --features windows'

# Run specific Windows test
vagrant winrm -c 'cd C:\vagrant_project; cargo test -p keyrx_daemon --features windows test_name'

# Check Rust installation
vagrant winrm -c 'rustc --version; cargo --version'

# Clean build
vagrant winrm -c 'cd C:\vagrant_project; cargo clean; cargo build --features windows'
```

### How to Add a Dependency

**Add to Cargo.toml:**
```bash
cd keyrx_core
cargo add rkyv --features validation
```

Or manually edit `Cargo.toml`:
```toml
[dependencies]
rkyv = { version = "0.7", features = ["validation"] }
```

**For platform-specific dependencies:**
```toml
[target.'cfg(target_os = "linux")'.dependencies]
evdev = "0.12"

[target.'cfg(target_os = "windows")'.dependencies]
windows-sys = { version = "0.48", features = ["Win32_UI_Input_KeyboardAndMouse"] }
```

**Verify the dependency:**
```bash
make build
```

### How to Fix Clippy Warnings

**Run clippy to see warnings:**
```bash
cargo clippy --workspace -- -D warnings
```

**Common warnings and fixes:**

1. **Needless borrow:**
```rust
// Warning: needless borrow
foo(&x);

// Fix:
foo(x);
```

2. **Unnecessary mut:**
```rust
// Warning: variable does not need to be mutable
let mut x = 5;

// Fix:
let x = 5;
```

3. **Match can be simplified:**
```rust
// Warning: match expression can be simplified
match result {
    Ok(val) => Some(val),
    Err(_) => None,
}

// Fix:
result.ok()
```

4. **Unused variable:**
```rust
// Warning: unused variable
let unused = 5;

// Fix: Remove or prefix with underscore
let _unused = 5;
```

### How to Format Code

**Format all code:**
```bash
cargo fmt
```

**Check formatting without modifying:**
```bash
cargo fmt --check
```

**Format is automatically checked by:**
- Pre-commit hooks
- CI/CD pipeline
- `make verify`

## Troubleshooting

### Build Failures

**Error: "error: could not compile"**
- **Cause**: Syntax error in Rust code
- **Fix**: Read error message, locate file and line number, fix syntax
- **Example**: Missing semicolon, unclosed brace, type mismatch

**Error: "error[E0425]: cannot find value"**
- **Cause**: Undefined variable or function
- **Fix**: Check imports, add missing dependency, or define the item
- **Example**: Forgot to import `use std::collections::HashMap;`

**Error: "error: package collision"**
- **Cause**: Duplicate dependency versions
- **Fix**: Run `cargo update` or specify exact versions in Cargo.toml

### Test Failures

**Error: "test result: FAILED"**
- **Cause**: Test assertion failed
- **Fix**:
  1. Run with verbose output: `cargo test -- --nocapture`
  2. Read assertion message
  3. Fix implementation or test
  4. Re-run: `make test`

**Error: Coverage below 80%**
- **Cause**: Insufficient test coverage
- **Fix**:
  1. Run `cargo tarpaulin` to see coverage report
  2. Identify uncovered lines
  3. Add tests for uncovered code paths
  4. Re-run: `make verify`

### Pre-Commit Hook Blocks Commit

**Error: "Pre-commit checks failed. Commit aborted."**
- **Cause**: Code fails quality checks (clippy, fmt, tests)
- **Fix**:
  1. Run `make verify` to see detailed errors
  2. Fix reported issues
  3. Re-run `make verify` to confirm fixes
  4. Commit again

**Bypass hook (NOT recommended):**
```bash
git commit --no-verify
```

### CI/CD Failures

**Error: "Clippy check failed"**
- **Cause**: Code has warnings treated as errors
- **Fix**: Run `cargo clippy --workspace -- -D warnings` locally, fix warnings

**Error: "Format check failed"**
- **Cause**: Code is not formatted
- **Fix**: Run `cargo fmt`, commit formatted code

**Error: "Tests failed"**
- **Cause**: Tests fail on CI but pass locally
- **Fix**: Check for platform-specific issues, ensure tests are deterministic (no wall-clock time, no random data)

### Missing Tools

**Error: "Required tool 'cargo-watch' not found"**
- **Cause**: Development tool not installed
- **Fix**: Run `make setup` to install all required tools

**Manual installation:**
```bash
cargo install cargo-watch
cargo install cargo-tarpaulin
cargo install cargo-fuzz
cargo install wasm-pack
```

### Script Errors

**Error: "=== failed ==="**
- **Cause**: Script operation failed
- **Fix**:
  1. Check log file: `scripts/logs/[script]_[timestamp].log`
  2. Read error messages
  3. Fix underlying issue
  4. Re-run script

**Get detailed output:**
```bash
# Run without --quiet flag
scripts/build.sh

# View log file
cat scripts/logs/build_*.log | tail -50
```

### Windows VM Errors

**Error: "vagrant: command not found"**
- **Cause**: Vagrant not installed
- **Fix**: `sudo apt install vagrant`

**Error: "No provider available"**
- **Cause**: vagrant-libvirt plugin not installed
- **Fix**: `vagrant plugin install vagrant-libvirt`

**Error: "Permission denied" when accessing libvirt**
- **Cause**: User not in libvirt group
- **Fix**:
  ```bash
  sudo usermod -aG libvirt $USER
  # Log out and log back in
  groups | grep libvirt  # Verify
  ```

**Error: "VM won't start" or hangs**
- **Cause**: Multiple possible causes
- **Fix**:
  ```bash
  # Check status
  vagrant status
  virsh list --all

  # Check libvirt daemon
  systemctl status libvirtd

  # View detailed logs
  cd vagrant/windows
  vagrant up --debug
  ```

**Error: "Build fails with 'link.exe not found'"**
- **Cause**: Visual Studio Build Tools not installed in VM
- **Fix**: Re-provision the VM (this installs MSVC):
  ```bash
  cd vagrant/windows
  vagrant provision --provision-with install-tools
  ```

**Error: "Files not syncing to VM"**
- **Cause**: Rsync not run after changes
- **Fix**:
  ```bash
  cd vagrant/windows
  vagrant rsync

  # Verify sync
  vagrant winrm -c 'dir C:\vagrant_project\Cargo.toml'
  ```

**Error: "Tests pass on Linux but fail on Windows"**
- **Cause**: Platform-specific behavior or test assumptions
- **Fix**:
  1. Check if test uses platform-specific features
  2. Use `#[cfg(target_os = "windows")]` for Windows-only tests
  3. Verify file paths use correct separators (`\` on Windows)
  4. Check for timing-dependent tests that may fail on slower VM

**VM is slow or unresponsive**
- **Cause**: Insufficient resources
- **Fix**: Edit `vagrant/windows/Vagrantfile`:
  ```ruby
  libvirt.cpus = 8        # Increase CPUs
  libvirt.memory = 32768  # Increase RAM to 32GB
  ```
  Then: `vagrant reload`

**Want to start fresh**
- **Fix**: Destroy and recreate VM:
  ```bash
  cd vagrant/windows
  vagrant destroy
  vagrant up
  # Then restore snapshot:
  vagrant snapshot restore provisioned
  ```

**For detailed Windows VM troubleshooting**, see:
- `vagrant/windows/README.md`
- `docs/development/windows-vm-setup.md`

## Advanced Usage

### Continuous Development Mode

**Watch mode for automatic rebuilds:**
```bash
scripts/build.sh --watch
```

Code changes trigger automatic rebuilds. Press Ctrl+C to stop.

### JSON Output for CI/CD

**Get machine-readable output:**
```bash
scripts/verify.sh --json
```

Parse output with `jq`:
```bash
scripts/verify.sh --json | jq '.checks.coverage'
```

### Custom Log Files

**Specify custom log location:**
```bash
scripts/build.sh --log-file /tmp/my-build.log
```

### Debug Mode

**Enable debug logging:**
```bash
scripts/launch.sh --debug
```

Outputs debug-level messages for troubleshooting.

### Headless Mode

**Run daemon without web UI:**
```bash
scripts/launch.sh --headless
```

Useful for servers or automated testing.

## Shared Utilities and Patterns

### Frontend Utilities (TypeScript/React)

The following utility modules provide reusable functionality across the frontend:

#### Time Formatting (`keyrx_ui/src/utils/timeFormatting.ts`)

Centralized time formatting functions:
- `formatTimestampMs(micros: number): string` - Converts microseconds to human-readable format (ms/s)
- `formatTimestampRelative(timestamp: number): string` - Formats as relative time ("2 hours ago")
- `formatDuration(durationMs: number): string` - Formats duration in milliseconds

**Usage:**
```typescript
import { formatTimestampMs, formatTimestampRelative } from '@/utils/timeFormatting';

const formatted = formatTimestampMs(1234567); // "1.23s"
const relative = formatTimestampRelative(Date.now() - 3600000); // "1 hour ago"
```

#### Key Code Mapping (`keyrx_ui/src/utils/keyCodeMapping.ts`)

Key code translation utilities:
- `formatKeyCode(code: number): string` - Formats numeric key code as string
- `keyCodeToLabel(code: number): string` - Converts to human-readable label ("A", "Enter")
- `parseKeyCode(label: string): number | null` - Parses label back to numeric code

**Usage:**
```typescript
import { keyCodeToLabel, parseKeyCode } from '@/utils/keyCodeMapping';

const label = keyCodeToLabel(65); // "A"
const code = parseKeyCode("Enter"); // 13
```

#### Test Utilities (`keyrx_ui/tests/testUtils.tsx`)

Shared test infrastructure for React components:
- `renderWithProviders(ui: ReactElement, options?: RenderOptions)` - Wraps components with necessary providers
- `createMockStore(initialState?: Partial<ConfigState>)` - Creates mock Zustand store
- `waitForAsync(callback: () => void, timeout?: number)` - Waits for async operations

**Usage:**
```typescript
import { renderWithProviders, createMockStore } from '../tests/testUtils';

test('component renders', () => {
  const mockStore = createMockStore({ layers: [...] });
  const { getByText } = renderWithProviders(<MyComponent />, { store: mockStore });
  expect(getByText('Hello')).toBeInTheDocument();
});
```

### Backend Utilities (Rust)

#### CLI Common Output (`keyrx_daemon/src/cli/common.rs`)

Standardized CLI output formatting:
- `output_success<T: Serialize>(data: T, json: bool)` - Outputs successful results
- `output_error(message: &str, code: u32, json: bool)` - Outputs errors
- `output_list<T: Serialize>(items: Vec<T>, json: bool)` - Outputs lists

**Usage:**
```rust
use crate::cli::common::{output_success, output_error};

// Success response
output_success(&profile_data, args.json)?;

// Error response
output_error("Profile not found", 1001, args.json);
```

### Dependency Injection Patterns

#### API Context (`keyrx_ui/src/contexts/ApiContext.tsx`)

Provides injectable API endpoints for testing:
- Default URLs configurable via environment variables
- `useApi()` hook returns `{ apiBaseUrl, wsBaseUrl }`

**Usage:**
```typescript
import { useApi } from '@/contexts/ApiContext';

function MyComponent() {
  const { apiBaseUrl } = useApi();

  async function fetchData() {
    const res = await fetch(`${apiBaseUrl}/api/profiles`);
    return res.json();
  }
}
```

**Testing:**
```typescript
import { ApiProvider } from '@/contexts/ApiContext';

test('component uses custom API', () => {
  render(
    <ApiProvider baseUrl="http://mock-api:3000">
      <MyComponent />
    </ApiProvider>
  );
});
```

#### ConfigStorage Abstraction (`keyrx_ui/src/services/ConfigStorage.ts`)

Abstract interface for storage operations:
- `LocalStorageImpl` - Browser localStorage implementation
- `MockStorageImpl` - In-memory implementation for testing

**Usage:**
```typescript
import { LocalStorageImpl, MockStorageImpl } from '@/services/ConfigStorage';

// Production
const storage = new LocalStorageImpl();
await storage.save('config', data);

// Testing
const mockStorage = new MockStorageImpl();
render(<ConfigurationPage storage={mockStorage} />);
```

## Technical Debt Prevention

Based on the technical-debt-remediation spec, follow these guidelines to prevent future technical debt:

### 1. File Size Monitoring

**Rule**: Maximum 500 lines of code (excluding comments/blanks) per file

**Enforcement:**
- Run `scripts/verify_file_sizes.sh` before committing
- Script uses `tokei` for accurate line counting
- Violations documented with refactoring plans

**When approaching limit:**
1. Extract helper functions to separate modules
2. Split large enums/structs into submodules
3. Move handlers to dedicated files
4. Consider if module has multiple responsibilities (violates SRP)

**Example refactoring:**
```
cli/config.rs (730 lines) → Split into:
  - cli/config.rs (dispatch only, ~100 lines)
  - cli/config/commands.rs (enum definitions, ~200 lines)
  - cli/config/handlers.rs (implementations, ~400 lines)
```

### 2. Extract Shared Utilities Early

**Warning signs of duplication:**
- Same function copied across 2+ files
- Similar formatting/conversion logic in multiple places
- Repeated validation or error handling patterns

**Action:**
- Extract after second duplication, not third
- Create utility module with comprehensive tests (≥90% coverage)
- Update all usage sites to import from utility

**TypeScript utilities location:** `keyrx_ui/src/utils/`
**Rust utilities location:** `keyrx_daemon/src/cli/common.rs` or crate-specific modules

### 3. Dependency Injection Requirements

**All external dependencies must be injectable:**
- API endpoints (use context providers)
- Storage mechanisms (use abstraction interfaces)
- WebSocket connections (pass as props or context)
- Platform-specific code (use trait abstractions)

**Benefits:**
- Enables unit testing without real dependencies
- Allows mock implementations for tests
- Improves component isolation and reusability

**Example - Before:**
```typescript
// ❌ Hard-coded dependency
async function fetchProfiles() {
  return fetch('http://localhost:3030/api/profiles');
}
```

**Example - After:**
```typescript
// ✅ Injected dependency
function ProfilesPage({ apiBaseUrl }: Props) {
  async function fetchProfiles() {
    return fetch(`${apiBaseUrl}/api/profiles`);
  }
}
```

### 4. Test Coverage Standards

**Minimum requirements:**
- Overall: ≥80% code coverage
- Critical paths (keyrx_core): ≥90% coverage
- New components: Must have unit tests before merge

**Test utilities:**
- Use `keyrx_ui/tests/testUtils.tsx` for React components
- Use shared mock implementations (`MockStorageImpl`, etc.)
- Follow React Testing Library best practices (test behavior, not implementation)

**Coverage measurement:**
- Rust: `cargo tarpaulin`
- TypeScript: `npm test -- --coverage`

### 5. Error Handling Standards

**Never use silent catch blocks:**
```typescript
// ❌ Bad - error ignored
try {
  JSON.parse(message);
} catch {}

// ✅ Good - error logged
try {
  JSON.parse(message);
} catch (err) {
  console.debug('Non-JSON message received:', message, err);
}
```

**Always propagate errors to UI:**
```typescript
// ❌ Bad - user unaware of error
catch (err) {
  console.warn('Failed:', err);
}

// ✅ Good - user sees error
catch (err) {
  setError(`Failed to save: ${err.message}`);
  console.error('Save failed:', err);
}
```

### 6. Structured Logging

**All logging must be structured (JSON format):**

**Required fields:**
- `timestamp` - ISO 8601 format
- `level` - debug, info, warn, error
- `service` - Component/module name
- `event` - Event type (user action, error, state change)
- `context` - Relevant data as JSON object

**Never log:**
- Secrets, API keys, passwords
- Personal identifiable information (PII)
- Full request/response bodies with sensitive data

**Rust example:**
```rust
log::info!(
    event = "profile_activated",
    profile = profile_name,
    timestamp = Utc::now().to_rfc3339()
);
```

**TypeScript example:**
```typescript
console.info(JSON.stringify({
  timestamp: new Date().toISOString(),
  level: 'info',
  service: 'ProfilesPage',
  event: 'profile_activated',
  context: { profileName }
}));
```

### 7. Documentation Requirements

**All public APIs must have documentation:**

**Rust (rustdoc):**
- Module-level comments (`//!`) explaining purpose
- Function comments (`///`) with examples
- Document errors with `# Errors` section
- Include usage examples with `# Examples`

**TypeScript (JSDoc):**
- Component descriptions with purpose and usage
- All props documented with `@param` tags
- Return values with `@returns`
- Complex components include `@example`

**Run documentation checks:**
```bash
cargo doc --no-deps --document-private-items
npm run typedoc
```

## References

- **Script Documentation**: `scripts/CLAUDE.md`
- **Steering Documents**: `.spec-workflow/specs/ai-dev-foundation/`
  - `requirements.md` - Detailed requirements
  - `design.md` - Architecture and design decisions
  - `tasks.md` - Implementation task breakdown
- **Project Structure**: `.spec-workflow/steering/structure.md`
- **CI/CD Workflows**: `.github/workflows/`
- **Rust API Guidelines**: https://rust-lang.github.io/api-guidelines/
