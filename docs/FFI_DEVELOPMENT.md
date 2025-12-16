# FFI Development Guide (AI-Native Workflow)

This is the complete guide for developing FFI (Foreign Function Interface) exports in KeyRx using an AI-native, contract-first workflow. This guide consolidates all FFI development practices, tools, and workflows into a single reference.

## Table of Contents

- [Quick Start](#quick-start)
- [AI-Native Development Philosophy](#ai-native-development-philosophy)
- [Development Environment Setup](#development-environment-setup)
- [Integrated Development Workflow](#integrated-development-workflow)
- [Contract-First Development](#contract-first-development)
- [Adding New FFI Functions](#adding-new-ffi-functions)
- [Watch Mode & Hot Reload](#watch-mode--hot-reload)
- [Contract Validation](#contract-validation)
- [Implementation Logging](#implementation-logging)
- [Tracing & Observability](#tracing--observability)
- [Breaking Change Detection](#breaking-change-detection)
- [Metrics & Dashboards](#metrics--dashboards)
- [Testing](#testing)
- [Troubleshooting](#troubleshooting)
- [Best Practices](#best-practices)
- [Command Reference](#command-reference)

## Quick Start

### 1. Setup Development Tools

```bash
# Install all FFI development tools (fast linkers, cargo-watch, tokei)
just setup-ffi-tools
```

This installs:
- **cargo-watch**: For automatic Rust rebuilds
- **mold** (Linux) / **zld** (macOS): Fast linkers (2-3x faster)
- **tokei**: Code metrics analyzer

### 2. Choose Your Development Mode

**UI-Only Development (Ultra-Fast, Recommended for UI work):**
```bash
# Flutter with mock FFI bridge - instant hot reload, no Rust compilation
just dev-mock
```
- ✅ <1s hot reload (vs 60s+ with FFI)
- ✅ No Rust toolchain required
- ✅ Full UI feature development
- ✅ Realistic mock data

**Integrated Development (Full Stack):**
```bash
# Rust watch + Flutter hot reload in parallel
just dev
```
- Automatic Rust compilation on FFI changes
- Automatic Dart binding regeneration
- Flutter hot reload for instant feedback
- ABI change detection

**FFI Watch Mode (Auto-regenerate bindings):**
```bash
./scripts/dev/ffi_watch.sh
```
- Watches `core/src/ffi/**/*.rs`
- Regenerates Dart bindings on change
- <10s iteration cycle

### 3. Add Your First FFI Function

Follow the [Adding New FFI Functions](#adding-new-ffi-functions) section below.

## AI-Native Development Philosophy

KeyRx is designed for **AI-first development**. This means:

### Single Source of Truth (SSOT): Rust

- **Contracts derive from Rust code**, not manual JSON
- **Dart bindings are generated**, never hand-written
- **Schema changes happen in Rust first**, then propagate to Dart

### Contract-Driven Workflow

```
1. Define/modify Rust API with #[ffi_export]
2. Run codegen (automatic in watch mode)
3. Implement UI with generated Dart bindings
```

**Never:**
- Hand-edit generated Dart code
- Manually create JSON contracts
- Make Dart-side type changes without updating Rust

### AI Agent Directives

The `.keyrx/agent.md` file contains rules for AI coding agents. Key constraints:

- **Panic safety**: FFI boundaries must catch panics
- **Realtime threads**: No locks/allocations on audio threads
- **No globals**: Use `FfiContext` handle-based state
- **File size budget**: ≤500 lines per module

Read `.keyrx/agent.md` before making FFI changes.

## Development Environment Setup

### Prerequisites

- Rust (latest stable)
- Flutter (latest stable)
- Git hooks installed

### Install FFI-Specific Tools

```bash
just setup-ffi-tools
```

Or manually:

**Linux:**
```bash
cargo install cargo-watch tokei
sudo apt install mold  # Fast linker
```

**macOS:**
```bash
cargo install cargo-watch tokei
brew install michaeleisel/zld/zld  # Fast linker
```

**Windows:**
```bash
cargo install cargo-watch tokei
# LLD linker is used (bundled with Rust)
```

### Configure Fast Linker

The setup script automatically configures `.cargo/config.toml`:

```toml
[target.x86_64-unknown-linux-gnu]
linker = "clang"
rustflags = ["-C", "link-arg=-fuse-ld=mold"]

[target.x86_64-apple-darwin]
rustflags = ["-C", "link-arg=-fuse-ld=/usr/local/bin/zld"]
```

**Verify it works:**
```bash
# Should link 2-3x faster than default
just build
```

### Install Git Hooks

```bash
./scripts/setup/git_hooks.sh
```

Hooks installed:
- **pre-commit**: Contract validation, linting
- **post-commit**: Implementation log reminder

## Integrated Development Workflow

### Starting Development

**Full development** (Rust + Flutter):
```bash
just dev
```

**UI-only development** (Mock FFI):
```bash
just dev-mock-full
```

**With FFI tracing enabled**:
```bash
just dev-trace
```

### What Happens in Dev Mode

```
┌─────────────────────────────────────────────┐
│  You edit: core/src/ffi/exports_engine.rs  │
└──────────────────┬──────────────────────────┘
                   ↓
┌──────────────────────────────────────────────┐
│  [RUST] Cargo watch detects change          │
│  [RUST] Recompiling core...                 │
└──────────────────┬───────────────────────────┘
                   ↓
┌──────────────────────────────────────────────┐
│  [RUST] Running codegen...                  │
│  [RUST] Generating Dart bindings...         │
└──────────────────┬───────────────────────────┘
                   ↓
┌──────────────────────────────────────────────┐
│  [FLUTTER] Detected binding changes         │
│  [FLUTTER] Hot reloading...                 │
│  [FLUTTER] Reloaded in 234ms                │
└──────────────────────────────────────────────┘
```

**Timeline:**
- Edit Rust → Compile (2-5s with fast linker) → Generate bindings (1s) → Hot reload (<1s)
- Total feedback loop: ~5-8 seconds

### Controls While Running

In the Flutter terminal:
- Press **`r`**: Manual hot reload
- Press **`R`**: Full hot restart (for ABI changes)
- Press **`Ctrl+C`**: Gracefully stop both processes

### ABI Change Detection

The dev script detects when you change the ABI (add/remove/modify FFI functions):

```
[FLUTTER] ⚠️  ABI change detected - full restart recommended
[FLUTTER] Press 'R' to restart now
```

## Contract-First Development

### The #[ffi_export] Macro

All FFI functions must use the `#[ffi_export]` macro to derive contracts:

```rust
use keyrx_ffi_macros::ffi_export;
use crate::ffi::{FfiContext, FfiResult, FfiError};

#[ffi_export(domain = "engine", version = "1.0")]
pub fn start_loop(ctx: &FfiContext, profile_id: &str) -> FfiResult<EngineState> {
    ctx.require_ready()?;

    // Implementation
    let state = start_engine_loop(profile_id)?;

    Ok(EngineState {
        running: true,
        profile: profile_id.to_string(),
    })
}
```

**What the macro does:**
1. Generates C-ABI wrapper function
2. Adds null checks for pointer parameters
3. Validates UTF-8 for string parameters
4. Catches panics → converts to `FfiError::internal`
5. Serializes results to JSON
6. Manages string ownership

**Generated contract JSON:**
```json
{
  "domain": "engine",
  "version": "1.0",
  "functions": [
    {
      "name": "start_loop",
      "rust_name": "keyrx_engine_start_loop",
      "parameters": [
        {"name": "ctx", "type": "FfiContext", "required": true},
        {"name": "profile_id", "type": "string", "required": true}
      ],
      "return_type": "Result<EngineState>",
      "description": ""
    }
  ]
}
```

### Contract Schema

Contracts follow a strict schema defined in `core/src/ffi/contract_schema.rs`. Key fields:

- **domain**: Namespace for the function (e.g., "engine", "config", "device")
- **version**: Semantic version (MAJOR.MINOR.PATCH)
- **functions**: Array of function definitions
- **parameters**: Typed parameter list
- **return_type**: Result type or plain type

### Regenerating Contracts

Contracts are automatically regenerated in watch mode, but you can manually trigger:

```bash
# Regenerate all contracts + Dart bindings
just gen-dart-bindings

# With verbose output
cargo run --release --manifest-path core/tools/generate_dart_bindings/Cargo.toml -- --verbose
```

**Incremental generation:**
The codegen tool uses SHA256 hashing to skip unchanged contracts:
```
Generated: 2 files
Skipped: 14 files (unchanged)
Time saved: 3.2s
```

Force regeneration:
```bash
cargo run --release --manifest-path core/tools/generate_dart_bindings/Cargo.toml -- --force
```

## Adding New FFI Functions

### Step-by-Step Guide

#### 1. Define the Rust Function

```rust
// File: core/src/ffi/domains/engine.rs

use keyrx_ffi_macros::ffi_export;
use crate::ffi::{FfiContext, FfiResult, FfiError};
use serde::Serialize;

#[derive(Serialize)]
pub struct EngineMetrics {
    pub cpu_usage: f32,
    pub memory_mb: u32,
    pub uptime_sec: u64,
}

#[ffi_export(domain = "engine", version = "1.0")]
pub fn get_metrics(ctx: &FfiContext) -> FfiResult<EngineMetrics> {
    ctx.require_ready()?;

    let state = ctx.domain::<EngineState>()?;

    Ok(EngineMetrics {
        cpu_usage: state.measure_cpu(),
        memory_mb: state.measure_memory(),
        uptime_sec: state.uptime().as_secs(),
    })
}
```

#### 2. Regenerate Bindings

If running `just dev`, bindings regenerate automatically.

Otherwise:
```bash
just gen-dart-bindings
```

#### 3. Use Generated Dart Bindings

```dart
// File: ui/lib/features/engine/engine_service.dart

import 'package:keyrx/ffi/generated/engine_bindings.dart';

class EngineService {
  final EngineFfi _ffi;

  EngineService(this._ffi);

  Future<EngineMetrics> getMetrics() async {
    final result = await _ffi.getMetrics();
    return result; // Type-safe, auto-generated
  }
}
```

#### 4. Test the Function

```rust
// File: core/src/ffi/domains/engine.rs

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_metrics() {
        let mut ctx = FfiContext::new();
        EngineState::init(&mut ctx).unwrap();

        let metrics = get_metrics(&ctx).unwrap();

        assert!(metrics.cpu_usage >= 0.0);
        assert!(metrics.cpu_usage <= 100.0);
    }
}
```

Run tests:
```bash
just test
```

#### 5. Log the Implementation

```bash
just ffi-log 22
```

Fill in the implementation log with details about the new function.

### Function Patterns

**Query function (read-only):**
```rust
#[ffi_export(domain = "config", version = "1.0")]
pub fn get_setting(ctx: &FfiContext, key: &str) -> FfiResult<String> {
    ctx.require_ready()?;

    let config = ctx.domain::<ConfigState>()?;
    config.get(key)
        .ok_or_else(|| FfiError::not_found("setting"))
}
```

**Command function (side effects):**
```rust
#[ffi_export(domain = "device", version = "1.0")]
pub fn connect(ctx: &FfiContext, device_id: &str) -> FfiResult<()> {
    ctx.require_ready()?;

    let mut devices = ctx.domain_mut::<DeviceState>()?;
    devices.connect(device_id)?;

    Ok(())
}
```

**Streaming function (with callbacks):**
```rust
#[ffi_export(domain = "discovery", version = "1.0")]
pub fn start_scan(ctx: &FfiContext) -> FfiResult<()> {
    ctx.require_ready()?;

    let registry = global_event_registry();

    std::thread::spawn(move || {
        for progress in 0..100 {
            registry.invoke(
                EventType::DiscoveryProgress,
                &json!({"progress": progress})
            );
        }
    });

    Ok(())
}
```

## Watch Mode & Hot Reload

### FFI Watch Mode

Start Rust watch mode only:
```bash
just dev-watch
```

This monitors:
- `core/src/ffi/`
- `core/src/api.rs`

On changes:
- Recompiles Rust
- Regenerates contracts
- Regenerates Dart bindings

**Debouncing:**
Changes are debounced (500ms) to avoid excessive regeneration.

### Incremental Generation

The watch mode uses incremental generation to skip unchanged contracts:

```
[WATCH] Detected changes in core/src/ffi/exports_engine.rs
[CODEGEN] Hashing contracts...
[CODEGEN] Generated: engine.ffi-contract.json (changed)
[CODEGEN] Skipped: config.ffi-contract.json (unchanged)
[CODEGEN] Skipped: device.ffi-contract.json (unchanged)
[CODEGEN] Total: 1 generated, 13 skipped
[CODEGEN] Time: 0.8s (saved 2.4s)
```

### Hot Reload Behavior

**Dart-only changes:**
- Flutter hot reload (<1s)
- No Rust rebuild needed

**Rust implementation changes** (no ABI change):
- Rust rebuild + codegen
- Flutter hot reload
- Total: ~5s

**Rust ABI changes** (add/remove/modify FFI functions):
- Rust rebuild + codegen
- Flutter **hot restart** required
- Total: ~10s

## Contract Validation

### Pre-Commit Validation

Contract validation runs automatically on every commit via the pre-commit hook.

**What it validates:**
1. All `#[ffi_export]` functions have matching contracts
2. Contract JSON matches Rust signatures
3. No orphaned contracts (contracts without Rust functions)

**Example output:**
```
╔════════════════════════════════════════════════════════════╗
║           FFI Contract Validation Summary                  ║
╚════════════════════════════════════════════════════════════╝

  Valid contracts:     96
  Missing contracts:   0
  Mismatched:          0
  Orphaned:            0

✓ All contracts are valid!
```

### Manual Validation

```bash
# Validate contracts
./scripts/ffi/validate_contracts.sh

# Verbose output
./scripts/ffi/validate_contracts.sh --verbose

# Auto-fix mismatches
./scripts/ffi/validate_contracts.sh --fix
```

### Fixing Validation Errors

**Error: Missing contract**
```
✗ Missing contract entry: engine.get_metrics
  Expected in: core/src/ffi/contracts/engine.ffi-contract.json
  Source: core/src/ffi/domains/engine.rs
```

**Fix:**
```bash
./scripts/ffi/validate_contracts.sh --fix
# or
just gen-dart-bindings
```

**Error: Contract mismatch**
```
✗ Contract mismatch: engine.start_loop
  Expected params: [ctx: FfiContext, profile_id: String]
  Actual params:   [ctx: FfiContext]
```

**Fix:** Update the Rust function signature or regenerate contracts.

### Bypassing Validation

To bypass validation temporarily (not recommended):
```bash
git commit --no-verify
```

## Implementation Logging

### Why Log Implementations?

Implementation logs create a **searchable knowledge base** for AI agents:
- Discover existing FFI functions
- Learn from past implementations
- Avoid duplicating code
- Understand design decisions

### Creating a Log Entry

After completing work on a task:

```bash
just ffi-log 22
```

This generates:
```
.spec-workflow/specs/rust-first-ai-native-ffi/Implementation Logs/task-22_2025-12-15T1234_abcd1234.md
```

The log is pre-filled with:
- Timestamp
- Task ID
- Git diff statistics
- Files changed

### Completing the Log

Open the generated file and fill in:

**1. Summary:**
```markdown
## Summary
Implemented comprehensive FFI development documentation with AI-native workflow guide,
migration guide, and updated architecture docs.
```

**2. Artifacts:**
Document what you created:
```markdown
## Artifacts

### Documentation
- docs/FFI_DEVELOPMENT.md: Complete AI-native FFI development guide
- docs/FFI_MIGRATION_GUIDE.md: Step-by-step migration from manual to contract-first
- docs/ffi-architecture.md: Updated with AI-native workflow section
```

**3. Tests:**
```markdown
## Tests
- Verified all just commands documented work correctly
- Tested contract validation workflow end-to-end
- Validated watch mode with example FFI function addition
```

**4. AI Notes:**
```markdown
## AI Notes
**What worked:**
- Following the contract-first workflow prevented sync errors
- Watch mode provided fast feedback (~5s edit-to-reload)
- Pre-commit hooks caught mistakes before commit

**Lessons learned:**
- Always run `just dev` instead of separate Rust/Flutter terminals
- Use `just dev-trace` when debugging FFI call sequences
- Implementation logs are critical for AI agents to avoid duplication
```

### Log Template

The template is at `.spec-workflow/templates/ffi-implementation-log-template.md`.

## Tracing & Observability

### Enabling FFI Tracing

**Development with tracing:**
```bash
just dev-trace
```

This sets `RUST_LOG=keyrx_core::ffi=trace,info` and starts the dev environment.

**Manual control:**
```bash
export RUST_LOG=keyrx_core::ffi=debug
just dev
```

### Instrumentation

All FFI functions are instrumented with `#[instrument]`:

```rust
use tracing::instrument;
use crate::ffi::tracing::{new_correlation_id, correlation_id};

#[instrument(skip(ctx), fields(correlation_id = %correlation_id()))]
#[ffi_export(domain = "engine", version = "1.0")]
pub fn start_loop(ctx: &FfiContext, profile_id: &str) -> FfiResult<EngineState> {
    new_correlation_id(); // Generate correlation ID

    tracing::info!(profile = profile_id, "Starting engine loop");

    // Implementation

    tracing::info!("Engine loop started successfully");
    Ok(state)
}
```

### Correlation IDs

Correlation IDs track request flows across FFI boundaries:

```rust
// Entry point - create correlation ID
let id = new_correlation_id();

// Internal functions - use same correlation ID
let current_id = correlation_id();
```

In logs:
```
2025-12-15T10:00:00.123Z INFO keyrx_core::ffi::engine correlation_id=42: Starting engine loop
2025-12-15T10:00:00.456Z DEBUG keyrx_core::ffi::engine correlation_id=42: Loading profile
2025-12-15T10:00:00.789Z INFO keyrx_core::ffi::engine correlation_id=42: Engine loop started
```

### DevTools Timeline Integration

FFI calls appear in Flutter DevTools Timeline:

**In your Dart app:**
```dart
import 'package:keyrx/ffi/tracing_bridge.dart';

void main() async {
  // Enable FFI trace collection
  FfiTracingBridge.enable();

  // Start Timeline integration
  await FfiTracingBridge.startTimelineIntegration();

  runApp(MyApp());
}
```

**View in DevTools:**
1. Start app with `just dev-trace`
2. Open Flutter DevTools
3. Navigate to Timeline tab
4. FFI calls show as timeline events with:
   - Function name
   - Duration
   - Arguments (sanitized)
   - Correlation IDs
   - Result status

### FFI Metrics

View runtime metrics:
```bash
# Text format
just ffi-metrics

# JSON format
just ffi-metrics json
```

**Metrics provided:**
- Call count per function
- Average latency per function
- Error rate per function
- Most frequently called functions
- Slowest functions

See [FFI_TRACING.md](./FFI_TRACING.md) for detailed tracing documentation.

## Breaking Change Detection

### Detecting Breaking Changes

```bash
# Compare with previous commit
just diff-ffi-contracts

# Compare with main branch
just diff-ffi-contracts main

# JSON output
just diff-ffi-contracts HEAD~1 json

# Markdown output
just diff-ffi-contracts HEAD~1 markdown
```

### What Constitutes a Breaking Change?

**Major (breaking):**
- Removing a function
- Changing parameter types
- Changing return types
- Adding required parameters
- Removing parameters
- Changing function domain

**Minor (compatible):**
- Adding new functions
- Adding optional parameters
- Making required parameters optional

**Patch (non-breaking):**
- Updating documentation
- Renaming parameters (same type)
- Updating metadata

### Handling Breaking Changes

**Option A: Bump version** (recommended)

Edit `core/Cargo.toml`:
```toml
[package]
version = "2.0.0"  # Major bump for breaking changes
```

**Option B: Allow breaking changes** (with PR label)

Add the `allow-breaking` label to your PR and document the migration path.

### CI Integration

The CI pipeline automatically:
1. Detects contract changes
2. Classifies as major/minor/patch
3. Checks if version was bumped
4. Posts PR comment if breaking
5. Blocks PR if breaking without version bump or label

See [FFI_VERSIONING.md](./FFI_VERSIONING.md) for the complete versioning policy.

## Metrics & Dashboards

### Contract Metrics Dashboard

Generate a comprehensive dashboard:
```bash
just ffi-dashboard
```

Opens `.spec-workflow/analysis/contract-dashboard.html` with:
- Total FFI functions by domain (bar chart)
- Evolution timeline (functions added/removed per week)
- Return type distribution (Result vs non-Result)
- Breaking changes count

### Implementation History Analysis

Analyze development velocity and patterns:
```bash
just analyze-ffi
```

Generates `.spec-workflow/analysis/implementation-insights.html` with:
- **Velocity metrics**: Functions added per week
- **Churn detection**: Frequently modified files
- **Coverage analysis**: Tested vs untested implementations
- **AI effectiveness**: Success rate of AI-generated code

### Module Size Analysis

Ensure modules fit in LLM context windows:
```bash
./scripts/metrics/module_sizes.sh
```

Generates `.spec-workflow/analysis/module-sizes.html` showing:
- Files exceeding 500-line limit
- Splitting recommendations
- Trend analysis

See [FFI_METRICS.md](./FFI_METRICS.md) for detailed metrics documentation.

## Testing

### Unit Tests

Test FFI functions directly:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_metrics() {
        let mut ctx = FfiContext::new();
        EngineDomain::init(&mut ctx).unwrap();

        let metrics = get_metrics(&ctx).unwrap();
        assert!(metrics.cpu_usage >= 0.0);
    }
}
```

Run tests:
```bash
just test
```

### Property Tests

Use `proptest` for FFI boundary robustness:

```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn fuzz_get_setting(key in ".*") {
        let mut ctx = FfiContext::new();
        ConfigDomain::init(&mut ctx).unwrap();

        // Should never panic, always return valid FfiResult
        let result = get_setting(&ctx, &key);
        assert!(result.is_ok() || result.is_err());
    }
}
```

### Integration Tests

Test the FFI boundary:

```rust
#[test]
fn test_ffi_string_handling() {
    use std::ffi::{CString, CStr};

    let key = CString::new("test_key").unwrap();

    unsafe {
        let result_ptr = keyrx_config_get_setting(key.as_ptr());
        assert!(!result_ptr.is_null());

        let result = CStr::from_ptr(result_ptr).to_str().unwrap();
        assert!(result.starts_with("ok:") || result.starts_with("error:"));

        keyrx_free_string(result_ptr);
    }
}
```

### Test Coverage

Check coverage:
```bash
just coverage-check
```

Target: **≥80% coverage** for all FFI code.

## Troubleshooting

### Watch Mode Not Triggering

**Symptom:** Changes to Rust files don't trigger rebuild.

**Solution:**
1. Check `cargo-watch` is installed: `cargo watch --version`
2. Verify file is in watched directory: `core/src/ffi/` or `core/src/api.rs`
3. Check for errors in terminal output
4. Restart watch mode: Stop (`Ctrl+C`) and run `just dev` again

### Bindings Out of Sync

**Symptom:** Dart code doesn't match Rust FFI exports.

**Solution:**
```bash
just gen-dart-bindings
just ui:setup  # Regenerate Flutter build_runner code
```

### Fast Linker Not Working

**Symptom:** Linking is slow (~10s instead of ~3s).

**Solution:**
1. Check linker is installed:
   - Linux: `mold --version`
   - macOS: `/usr/local/bin/zld --version`
2. Verify `.cargo/config.toml` configuration
3. Re-run setup: `just setup-ffi-tools`

### Contract Validation Failing

**Symptom:** Pre-commit hook blocks commit with validation errors.

**Solution:**
```bash
# See validation errors
./scripts/ffi/validate_contracts.sh --verbose

# Auto-fix
./scripts/ffi/validate_contracts.sh --fix

# Verify fix
./scripts/ffi/validate_contracts.sh
```

### Hot Reload Not Working After FFI Change

**Symptom:** Dart hot reload fails after modifying Rust FFI.

**Solution:**
1. Check terminal for ABI change notification
2. Press **`R`** for full hot restart (capital R)
3. If that fails, stop and restart `just dev`

### Tracing Shows No Output

**Symptom:** No FFI traces in logs despite enabling tracing.

**Solution:**
1. Check `RUST_LOG` is set: `echo $RUST_LOG`
2. Verify tracing is initialized in Rust code
3. Use `just dev-trace` instead of `just dev`
4. Check log level (should be `debug` or `trace`)

## Best Practices

### 1. Always Use Contract-First Workflow

✅ **DO:**
```rust
#[ffi_export(domain = "engine", version = "1.0")]
pub fn new_function() -> FfiResult<()> { /* ... */ }
```
Then run codegen.

❌ **DON'T:**
- Manually edit JSON contracts
- Hand-write Dart FFI bindings
- Change Dart types without updating Rust

### 2. Keep Modules Under 500 Lines

✅ **DO:**
- Split modules when approaching 450-500 lines
- Organize by domain/feature
- Use `mod.rs` for re-exports

❌ **DON'T:**
- Create mega-modules (>1000 lines)
- Split arbitrarily without logical boundaries

### 3. Ensure Panic Safety

✅ **DO:**
```rust
#[ffi_export(domain = "engine", version = "1.0")]
pub fn safe_function() -> FfiResult<()> {
    // #[ffi_export] catches panics automatically
    might_panic()?;
    Ok(())
}
```

❌ **DON'T:**
```rust
pub extern "C" fn unsafe_function() {
    panic!("This crosses FFI boundary!"); // ❌ Bad!
}
```

### 4. Use Structured Errors

✅ **DO:**
```rust
if key.is_empty() {
    return Err(FfiError::invalid_input("key must not be empty"));
}
```

❌ **DON'T:**
```rust
panic!("key is empty"); // ❌ Use Result instead
```

### 5. Instrument All FFI Functions

✅ **DO:**
```rust
#[instrument(skip(ctx), fields(correlation_id = %correlation_id()))]
#[ffi_export(domain = "engine", version = "1.0")]
pub fn traced_function(ctx: &FfiContext) -> FfiResult<()> {
    new_correlation_id();
    // Implementation
}
```

### 6. Log All Implementations

✅ **DO:**
```bash
# After completing task
just ffi-log 22
# Fill in the log with artifacts, tests, AI notes
git add .
git commit -m "Task 22: Documentation updates"
```

❌ **DON'T:**
- Skip implementation logs
- Leave logs incomplete
- Forget to document AI failures

### 7. Test Before Committing

✅ **DO:**
```bash
just check  # Runs fmt, clippy, tests, binding validation
```

❌ **DON'T:**
```bash
git commit --no-verify  # Bypasses validation
```

### 8. Use Incremental Development

✅ **DO:**
1. Add one function at a time
2. Test immediately
3. Commit after each logical unit

❌ **DON'T:**
- Add 10 functions at once
- Test at the end
- Make large uncommitted changes

## Command Reference

### Development

| Command | Description |
|---------|-------------|
| `just dev` | Start integrated dev environment (Rust watch + Flutter hot reload) |
| `just dev-mock-full` | Start dev with mock FFI bridge (UI-only) |
| `just dev-trace` | Start dev with FFI tracing enabled |
| `just dev-watch` | Start Rust watch mode only |
| `just ui` | Start Flutter only |

### Code Generation

| Command | Description |
|---------|-------------|
| `just gen-dart-bindings` | Generate Dart bindings from Rust contracts |
| `just check-dart-bindings` | Check if bindings are up-to-date (CI mode) |
| `just verify-bindings` | Verify bindings are in sync |

### Validation

| Command | Description |
|---------|-------------|
| `./scripts/ffi/validate_contracts.sh` | Validate all contracts |
| `./scripts/ffi/validate_contracts.sh --fix` | Auto-fix contract mismatches |
| `./scripts/ffi/validate_contracts.sh --verbose` | Verbose validation output |

### Logging

| Command | Description |
|---------|-------------|
| `just ffi-log <task_id>` | Create implementation log entry |
| `just ffi-log <task_id> <base_ref>` | Create log with git diff vs base |

### Breaking Changes

| Command | Description |
|---------|-------------|
| `just diff-ffi-contracts` | Detect breaking changes vs HEAD~1 |
| `just diff-ffi-contracts main` | Detect breaking changes vs main |
| `just diff-ffi-contracts HEAD~1 json` | JSON output format |

### Metrics & Dashboards

| Command | Description |
|---------|-------------|
| `just ffi-dashboard` | Generate contract metrics dashboard |
| `just ffi-metrics` | Display FFI call metrics (text) |
| `just ffi-metrics json` | Display FFI call metrics (JSON) |
| `just analyze-ffi` | Analyze FFI implementation history |
| `./scripts/metrics/module_sizes.sh` | Analyze module sizes |

### Testing

| Command | Description |
|---------|-------------|
| `just test` | Run all tests |
| `just check` | Run all quality checks (fmt, clippy, tests, bindings) |
| `just coverage-check` | Check test coverage (≥80%) |

### Setup

| Command | Description |
|---------|-------------|
| `just setup` | Full setup (dev env + FFI tools) |
| `just setup-ffi-tools` | Install FFI-specific tools only |

## Related Documentation

- [FFI Architecture](./ffi-architecture.md) - High-level FFI design and patterns
- [FFI Migration Guide](./FFI_MIGRATION_GUIDE.md) - Migrating from manual to contract-first
- [FFI Tracing](./FFI_TRACING.md) - Detailed tracing and DevTools integration
- [FFI Metrics](./FFI_METRICS.md) - Metrics, dashboards, and analytics
- [FFI Versioning](./FFI_VERSIONING.md) - Versioning policy and breaking changes
- [FFI Instrumentation](./FFI_INSTRUMENTATION.md) - Instrumentation guide
- [AI Agent Guide](../AI_AGENT_GUIDE.md) - Guide for AI agents
- [Agent Directives](../.keyrx/agent.md) - AI agent rules and constraints

## Support

For issues or questions:
1. Check this documentation first
2. Review [Troubleshooting](#troubleshooting)
3. Check recent implementation logs for similar issues
4. Ask in team chat or open a GitHub discussion
