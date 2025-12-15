# FFI Migration Guide: From Manual to AI-Native Workflow

This guide helps you migrate from the old manual FFI workflow to the new AI-native, contract-first workflow. Follow these steps to modernize your FFI development process.

## Table of Contents

- [Overview](#overview)
- [What's Changing](#whats-changing)
- [Before You Start](#before-you-start)
- [Migration Steps](#migration-steps)
- [Workflow Comparison](#workflow-comparison)
- [Common Migration Scenarios](#common-migration-scenarios)
- [Troubleshooting](#troubleshooting)
- [Checklist](#checklist)

## Overview

The AI-native FFI workflow represents a fundamental shift in how we develop FFI interfaces:

**Old Workflow:**
- Manual JSON contract editing
- Manual Dart binding generation
- Separate Rust and Flutter terminals
- No contract validation
- Large monolithic modules (>15,000 lines)

**New Workflow:**
- Contract-first with `#[ffi_export]` macro
- Automatic Dart binding generation
- Integrated development environment (`just dev`)
- Pre-commit contract validation
- Modular structure (<500 lines per file)
- Watch mode with incremental regeneration
- Implementation logging for AI agents

## What's Changing

### 1. Contract Generation

**Before:**
```json
// Manual editing of core/src/ffi/contracts/engine.ffi-contract.json
{
  "domain": "engine",
  "functions": [
    {
      "name": "start_loop",
      "rust_name": "keyrx_engine_start_loop",
      "parameters": [...]
    }
  ]
}
```

**After:**
```rust
// Contracts derive from Rust code
#[ffi_export(domain = "engine", version = "1.0")]
pub fn start_loop(ctx: &FfiContext, profile_id: &str) -> FfiResult<EngineState> {
    // Implementation
}
```

Contracts are **automatically generated** from the `#[ffi_export]` annotation.

### 2. Dart Bindings

**Before:**
- Python script (`scripts/generate_dart_bindings.py`)
- Manual invocation
- No incremental generation

**After:**
- Rust tool (`core/tools/generate_dart_bindings`)
- Automatic in watch mode
- Incremental generation (skips unchanged contracts)
- Integrated with `just dev`

### 3. Development Workflow

**Before:**
```bash
# Terminal 1: Rust
cd core && cargo build

# Terminal 2: Regenerate bindings
python scripts/generate_dart_bindings.py

# Terminal 3: Flutter
cd ui && flutter run
```

**After:**
```bash
# Single command - integrated workflow
just dev
```

Auto-compiles Rust, regenerates bindings, and hot reloads Flutter.

### 4. Module Organization

**Before:**
- Huge files: `exports_engine.rs` (15,843 lines), `exports_compat.rs` (15,754 lines)
- All functions in single file
- Doesn't fit in LLM context windows

**After:**
- Modular structure: ≤500 lines per file
- Organized by domain and feature
- AI-agent accessible

### 5. Contract Validation

**Before:**
- No validation
- Runtime errors from mismatched contracts
- Manual checking required

**After:**
- Pre-commit validation
- Automatic contract-to-Rust signature checks
- Validation integrated in CI/CD

## Before You Start

### Prerequisites

1. **Backup your work:**
   ```bash
   git checkout -b ffi-migration-$(date +%Y%m%d)
   git push -u origin ffi-migration-$(date +%Y%m%d)
   ```

2. **Install FFI tools:**
   ```bash
   just setup-ffi-tools
   ```

3. **Ensure tests pass:**
   ```bash
   just test
   ```

4. **Read the AI-native workflow guide:**
   See [FFI_DEVELOPMENT.md](./FFI_DEVELOPMENT.md) for complete workflow documentation.

## Migration Steps

### Step 1: Install Development Tools

Install the AI-native FFI development tools:

```bash
just setup-ffi-tools
```

This installs:
- `cargo-watch`: Auto-rebuild on changes
- `mold` (Linux) / `zld` (macOS): Fast linker (2-3x speedup)
- `tokei`: Code metrics analyzer

**Verify installation:**
```bash
cargo watch --version
mold --version  # or zld --version on macOS
tokei --version
```

### Step 2: Remove Python Codegen (Optional)

The old Python codegen script is deprecated. The Rust tool has feature parity.

**Verify Rust tool works:**
```bash
just gen-dart-bindings
```

**Remove Python script** (after verification):
```bash
rm scripts/generate_dart_bindings.py
```

Update any scripts or documentation that reference the Python script.

### Step 3: Add #[ffi_export] to Existing Functions

For each FFI function, add the `#[ffi_export]` macro:

**Before:**
```rust
// core/src/ffi/exports_engine.rs

#[no_mangle]
pub extern "C" fn keyrx_engine_start_loop(
    profile_id_ptr: *const c_char
) -> *mut c_char {
    // Manual FFI wrapping...
}
```

**After:**
```rust
// core/src/ffi/domains/engine.rs

use keyrx_ffi_macros::ffi_export;

#[ffi_export(domain = "engine", version = "1.0")]
pub fn start_loop(ctx: &FfiContext, profile_id: &str) -> FfiResult<EngineState> {
    // Clean business logic
}
```

The macro automatically generates the C-ABI wrapper.

### Step 4: Regenerate Contracts

After adding `#[ffi_export]` annotations, regenerate contracts:

```bash
just gen-dart-bindings
```

**Verify contracts match:**
```bash
git diff core/src/ffi/contracts/
```

Check that generated contracts match your manual contracts.

### Step 5: Validate Contracts

Run contract validation:

```bash
./scripts/validate_ffi_contracts.sh --verbose
```

**Expected output:**
```
✓ All contracts are valid!
```

**If validation fails:**
```bash
# Auto-fix mismatches
./scripts/validate_ffi_contracts.sh --fix
```

### Step 6: Enable Pre-Commit Hooks

Install git hooks to run validation automatically:

```bash
./scripts/install-hooks.sh
```

Hooks installed:
- **pre-commit**: Contract validation, linting
- **post-commit**: Implementation log reminder

**Test the hook:**
```bash
# Make a trivial change
touch test_file
git add test_file
git commit -m "Test pre-commit hook"
```

You should see contract validation run before commit.

### Step 7: Start Using Integrated Development

Replace your multi-terminal workflow with `just dev`:

**Stop:**
- Separate Rust terminal
- Separate Flutter terminal
- Manual binding regeneration

**Start:**
```bash
just dev
```

This provides:
- Automatic Rust rebuild on changes
- Automatic binding regeneration
- Flutter hot reload
- ABI change detection

**Try it:**
1. Run `just dev`
2. Edit a Rust FFI function in `core/src/ffi/`
3. Watch automatic rebuild + binding regeneration
4. See Flutter hot reload

### Step 8: Split Large Modules (If Needed)

If you have modules >500 lines, split them:

```bash
# Analyze module sizes
./scripts/analyze_module_sizes.sh

# View violations
xdg-open .spec-workflow/analysis/module-sizes.html
```

**For modules >500 lines:**

See the module splitting guide or use the automated tool:
```bash
# Preview split plan
./scripts/split_ffi_module.sh --module core/src/ffi/exports_large.rs --preview

# Execute split
./scripts/split_ffi_module.sh --module core/src/ffi/exports_large.rs --apply
```

### Step 9: Add Tracing Instrumentation

Add tracing to all FFI functions for observability:

```rust
use tracing::instrument;
use crate::ffi::tracing::{new_correlation_id, correlation_id};

#[instrument(skip(ctx), fields(correlation_id = %correlation_id()))]
#[ffi_export(domain = "engine", version = "1.0")]
pub fn start_loop(ctx: &FfiContext, profile_id: &str) -> FfiResult<EngineState> {
    new_correlation_id(); // Generate correlation ID at entry

    tracing::info!(profile = profile_id, "Starting engine loop");

    // Implementation

    tracing::info!("Engine loop started successfully");
    Ok(state)
}
```

**Test tracing:**
```bash
just dev-trace
```

FFI calls should appear in logs with correlation IDs.

### Step 10: Set Up Breaking Change Detection

Configure breaking change detection in your workflow:

```bash
# Detect breaking changes
just diff-ffi-contracts main
```

**Add to CI** (`.github/workflows/ffi-breaking-changes.yml`):
- See [FFI_VERSIONING.md](./FFI_VERSIONING.md) for CI integration

### Step 11: Create Implementation Logs

Start logging FFI implementations for AI agents:

```bash
# After completing work on a task
just ffi-log <task_id>

# Fill in the generated log with:
# - Summary of changes
# - Artifacts created (functions, contracts, etc.)
# - Tests added
# - AI notes (what worked, what failed)
```

**Log location:**
```
.spec-workflow/specs/rust-first-ai-native-ffi/Implementation Logs/
```

### Step 12: Update Documentation

Update your team documentation:

1. **Add link to FFI_DEVELOPMENT.md** in your README
2. **Update onboarding docs** with `just dev` workflow
3. **Add FFI commands** to team cheat sheet
4. **Document any custom workflows** specific to your project

### Step 13: Verify Migration

Run the full quality gate:

```bash
just check
```

This runs:
- `cargo fmt --check`
- `cargo clippy -- -D warnings`
- `cargo test`
- Contract validation
- Binding verification

**All checks should pass.**

## Workflow Comparison

### Adding a New FFI Function

**Old Workflow:**

1. Write Rust function
2. Manually edit `core/src/ffi/contracts/<domain>.ffi-contract.json`
3. Run `python scripts/generate_dart_bindings.py`
4. Manually test for contract sync errors
5. Write Dart code using bindings
6. Rebuild Rust
7. Restart Flutter app

**New Workflow:**

1. Write Rust function with `#[ffi_export]`
2. Bindings auto-regenerate (watch mode)
3. Flutter hot reloads automatically
4. Pre-commit hook validates contracts

**Time saved:** ~70% (5-10 minutes → 1-2 minutes)

### Modifying an Existing FFI Function

**Old Workflow:**

1. Edit Rust function
2. Update JSON contract manually
3. Regenerate bindings
4. Restart Flutter
5. Hope contracts match

**New Workflow:**

1. Edit Rust function (keep `#[ffi_export]` updated)
2. Bindings auto-regenerate
3. Flutter hot reloads
4. Validation ensures contracts match

**Time saved:** ~80% (10-15 minutes → 2-3 minutes)

### Debugging FFI Issues

**Old Workflow:**

1. Add `println!` statements
2. Rebuild Rust
3. Restart Flutter
4. Check logs
5. Repeat

**New Workflow:**

1. Enable tracing: `just dev-trace`
2. Structured logs with correlation IDs
3. View in DevTools Timeline
4. Hot reload for fast iteration

**Time saved:** ~60% (30 minutes → 12 minutes)

## Common Migration Scenarios

### Scenario 1: Converting a Simple FFI Function

**Before:**
```rust
#[no_mangle]
pub unsafe extern "C" fn keyrx_config_get_value(
    key_ptr: *const c_char
) -> *mut c_char {
    if key_ptr.is_null() {
        return error_json("NULL_POINTER", "key is null");
    }

    let key = match CStr::from_ptr(key_ptr).to_str() {
        Ok(s) => s,
        Err(_) => return error_json("INVALID_UTF8", "key is not valid UTF-8"),
    };

    // Business logic
    let value = get_config_value(key);

    result_to_json(value)
}
```

**After:**
```rust
#[ffi_export(domain = "config", version = "1.0")]
pub fn get_value(ctx: &FfiContext, key: &str) -> FfiResult<String> {
    ctx.require_ready()?;

    let config = ctx.domain::<ConfigState>()?;

    config.get(key)
        .ok_or_else(|| FfiError::not_found("config_key"))
}
```

**Benefits:**
- No manual null checks (macro handles it)
- No manual UTF-8 validation (macro handles it)
- Clean business logic
- Type-safe error handling
- Automatic JSON serialization

### Scenario 2: Migrating a Function with Callbacks

**Before:**
```rust
static ON_PROGRESS: OnceLock<Mutex<Option<ProgressCallback>>> = OnceLock::new();

#[no_mangle]
pub extern "C" fn keyrx_scan_register_progress_callback(
    callback: Option<ProgressCallback>
) {
    ON_PROGRESS.get_or_init(|| Mutex::new(None))
        .lock()
        .unwrap()
        .replace(callback);
}

#[no_mangle]
pub extern "C" fn keyrx_scan_start() -> *mut c_char {
    // Emit progress via global callback
    if let Some(cb) = ON_PROGRESS.get().and_then(|m| m.lock().ok()?.as_ref()) {
        cb(progress_data);
    }
}
```

**After:**
```rust
#[ffi_export(domain = "scan", version = "1.0")]
pub fn start_scan(ctx: &FfiContext) -> FfiResult<()> {
    ctx.require_ready()?;

    let registry = global_event_registry();

    std::thread::spawn(move || {
        for progress in 0..100 {
            registry.invoke(
                EventType::ScanProgress,
                &json!({"progress": progress})
            );
        }
    });

    Ok(())
}
```

**Benefits:**
- No global state
- Unified event registry
- Simpler callback management
- Thread-safe by design

### Scenario 3: Splitting a Large Module

**Before:**
```rust
// core/src/ffi/exports_engine.rs (15,843 lines)

pub fn engine_start() { /* ... */ }
pub fn engine_stop() { /* ... */ }
pub fn engine_get_state() { /* ... */ }
pub fn engine_set_config() { /* ... */ }
// ... 500+ more functions
```

**After:**
```
core/src/ffi/domains/engine/
├── mod.rs           // Re-exports
├── lifecycle.rs     // start, stop, pause, resume
├── state.rs         // get_state, set_state
├── config.rs        // get_config, set_config
├── audio.rs         // set_volume, mute, unmute
└── events.rs        // subscribe, unsubscribe
```

Each file ≤500 lines, organized by feature.

**Benefits:**
- Fits in LLM context windows
- Easier to navigate and maintain
- Clear separation of concerns
- Better for code review

### Scenario 4: Adding Tracing to Existing Function

**Before:**
```rust
pub fn process_data(data: &str) -> FfiResult<ProcessedData> {
    println!("Processing: {}", data);  // Debug logging

    // Implementation
    let result = do_processing(data)?;

    println!("Processed successfully");
    Ok(result)
}
```

**After:**
```rust
#[instrument(skip(data), fields(
    correlation_id = %correlation_id(),
    data_len = data.len()
))]
pub fn process_data(data: &str) -> FfiResult<ProcessedData> {
    new_correlation_id();

    tracing::info!(data_len = data.len(), "Processing data");

    let result = do_processing(data)
        .map_err(|e| {
            tracing::error!(error = %e, "Processing failed");
            FfiError::internal(format!("processing failed: {}", e))
        })?;

    tracing::info!("Processing completed successfully");
    Ok(result)
}
```

**Benefits:**
- Structured logging
- Correlation ID tracking
- DevTools Timeline integration
- Better debugging

## Troubleshooting

### Issue: Contract validation fails after migration

**Symptom:**
```
✗ Missing contract entry: engine.new_function
```

**Solution:**
```bash
# Regenerate contracts
just gen-dart-bindings

# Verify fix
./scripts/validate_ffi_contracts.sh
```

### Issue: Dart bindings don't match Rust

**Symptom:** Dart code fails to compile after Rust changes.

**Solution:**
```bash
# Regenerate bindings
just gen-dart-bindings

# Regenerate Flutter code
just ui:setup
```

### Issue: Watch mode not triggering

**Symptom:** Changes to Rust files don't trigger rebuild.

**Solution:**
1. Check `cargo-watch` is installed: `cargo watch --version`
2. Restart watch mode: Stop (`Ctrl+C`) and run `just dev` again
3. Verify file is in watched directory (`core/src/ffi/`)

### Issue: Fast linker not working

**Symptom:** Linking still slow (~10s instead of ~3s).

**Solution:**
```bash
# Check linker installation
mold --version  # Linux
zld --version   # macOS

# Re-run setup
just setup-ffi-tools

# Verify .cargo/config.toml configuration
cat .cargo/config.toml
```

### Issue: Pre-commit hook blocks legitimate changes

**Symptom:** Hook blocks commit even though changes are valid.

**Solution:**
```bash
# Verify validation is correct
./scripts/validate_ffi_contracts.sh --verbose

# If validation is wrong, fix contracts
./scripts/validate_ffi_contracts.sh --fix

# Last resort: bypass (not recommended)
git commit --no-verify
```

### Issue: Module splitting breaks tests

**Symptom:** Tests fail after splitting a large module.

**Solution:**
1. Check all imports are updated
2. Verify `mod.rs` has correct re-exports
3. Run `cargo build` to check for compilation errors
4. Fix import paths in test files

## Checklist

Use this checklist to track your migration progress:

### Setup
- [ ] Install FFI development tools (`just setup-ffi-tools`)
- [ ] Verify tools installed (cargo-watch, mold/zld, tokei)
- [ ] Create migration branch
- [ ] Backup existing contracts

### Contract Migration
- [ ] Add `#[ffi_export]` to all FFI functions
- [ ] Regenerate contracts (`just gen-dart-bindings`)
- [ ] Verify generated contracts match manual contracts
- [ ] Delete manual contracts (after verification)

### Workflow Migration
- [ ] Install git hooks (`./scripts/install-hooks.sh`)
- [ ] Test pre-commit validation
- [ ] Replace multi-terminal workflow with `just dev`
- [ ] Test integrated dev workflow end-to-end

### Module Organization
- [ ] Run module size analysis (`./scripts/analyze_module_sizes.sh`)
- [ ] Split modules >500 lines
- [ ] Verify all modules compile
- [ ] Update imports and re-exports

### Tracing & Observability
- [ ] Add `#[instrument]` to all FFI functions
- [ ] Add correlation ID tracking
- [ ] Test tracing (`just dev-trace`)
- [ ] Verify DevTools Timeline integration

### Breaking Change Detection
- [ ] Test breaking change detection (`just diff-ffi-contracts`)
- [ ] Add to CI/CD pipeline
- [ ] Document versioning policy

### Documentation
- [ ] Update team documentation
- [ ] Add FFI_DEVELOPMENT.md link to README
- [ ] Update onboarding docs
- [ ] Document custom workflows

### Validation
- [ ] Run full quality gate (`just check`)
- [ ] All tests pass
- [ ] Contract validation passes
- [ ] Bindings verified
- [ ] No clippy warnings

### Final Steps
- [ ] Create implementation log
- [ ] Commit migration
- [ ] Create PR with migration notes
- [ ] Team review and approval
- [ ] Merge to main
- [ ] Update team training materials

## Next Steps

After completing the migration:

1. **Train your team:** Share [FFI_DEVELOPMENT.md](./FFI_DEVELOPMENT.md) with all developers
2. **Update CI/CD:** Add FFI quality gates to your pipeline
3. **Monitor metrics:** Run `just ffi-dashboard` weekly to track FFI health
4. **Log implementations:** Create implementation logs for all FFI work
5. **Iterate:** Continuously improve your AI-native workflow

## Resources

- [FFI Development Guide](./FFI_DEVELOPMENT.md) - Complete AI-native workflow
- [FFI Architecture](./ffi-architecture.md) - High-level FFI design
- [FFI Tracing](./FFI_TRACING.md) - Tracing and observability
- [FFI Metrics](./FFI_METRICS.md) - Metrics and dashboards
- [FFI Versioning](./FFI_VERSIONING.md) - Versioning and breaking changes
- [Agent Directives](../.keyrx/agent.md) - AI agent rules

## Support

Questions or issues during migration?

1. Check [Troubleshooting](#troubleshooting)
2. Review recent implementation logs
3. Ask in team chat
4. Open a GitHub discussion
