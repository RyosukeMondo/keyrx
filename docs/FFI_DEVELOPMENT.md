# FFI Development Workflow (AI-Native)

This workflow keeps FFI changes traceable and fast for humans and AI agents.

## Quick Start - Integrated Development

Start the complete development environment with a single command:

```bash
just dev
```

This launches:
- **Rust watch mode**: Auto-compiles on FFI changes → regenerates Dart bindings
- **Flutter hot reload**: Auto-reloads UI on Dart changes
- **ABI change detection**: Notifies when full restart is needed vs hot reload
- **Unified logging**: All output prefixed with `[RUST]` or `[FLUTTER]`

### Development Modes

**Full Development** (Rust + Flutter):
```bash
just dev
# or
./scripts/integrated_dev.sh
```

**UI-Only Development** (Mock FFI):
```bash
just dev-mock-full
# or
./scripts/integrated_dev.sh --mock
```

**Custom Device**:
```bash
./scripts/integrated_dev.sh --device=chrome
```

### Workflow Timeline

When you edit code, the system automatically handles:

```
Edit Rust → Compile → Generate Bindings → Flutter Reload
Edit Dart → Flutter Hot Reload (instant)
Edit FFI ABI → Notification: "Full restart recommended"
```

### Controls

While running `just dev`:
- Press **`r`** in Flutter terminal for hot reload
- Press **`R`** in Flutter terminal for hot restart (ABI changes)
- Press **`Ctrl+C`** to gracefully stop all processes

### Startup Time

- **Initial startup**: ~10s (Rust compile + Flutter launch)
- **Rust incremental rebuild**: ~2-5s (with fast linker)
- **Dart hot reload**: <1s (instant feedback)

## FFI Contract Validation

### Overview

All FFI functions must use the `#[ffi_export]` macro to derive contracts from Rust code. The validation system ensures contracts stay synchronized with the actual Rust implementation.

### Validation Commands

**Validate contracts only**:
```bash
./scripts/validate_ffi_contracts.sh
```

**Validate with verbose output**:
```bash
./scripts/validate_ffi_contracts.sh --verbose
```

**Auto-fix contract mismatches**:
```bash
./scripts/validate_ffi_contracts.sh --fix
```

**CI mode (strict validation)**:
```bash
./scripts/validate_ffi_contracts.sh --check
```

### What Gets Validated

The validation script checks:

1. **All `#[ffi_export]` functions have matching contracts** - Every Rust function annotated with `#[ffi_export]` must have a corresponding entry in the domain's contract JSON file

2. **Contract entries match Rust signatures** - Function names, parameters, and return types must match between Rust code and JSON contracts

3. **No orphaned contracts** - Contract JSON files should not contain entries for functions that no longer exist in Rust code (warning only in normal mode, error in CI mode)

### Pre-commit Hook

Contract validation runs automatically on every commit via the pre-commit hook. If validation fails, the commit is blocked until you fix the issues.

To bypass validation temporarily (not recommended):
```bash
git commit --no-verify
```

### Fixing Validation Errors

When validation fails, you have several options:

**Option 1: Auto-regenerate contracts**
```bash
./scripts/validate_ffi_contracts.sh --fix
```
This runs `just gen-dart-bindings` to regenerate all contracts from Rust code.

**Option 2: Manual regeneration**
```bash
just gen-dart-bindings
```

**Option 3: Fix the Rust code**
Ensure all FFI functions have the `#[ffi_export]` annotation with the correct domain:
```rust
#[ffi_export(domain = "engine", version = "1.0")]
pub fn start_loop(profile_id: &str) -> Result<EngineState, EngineError> {
    // Implementation
}
```

### Contract Files Location

Contract JSON files are located at:
```
core/src/ffi/contracts/<domain>.ffi-contract.json
```

Examples:
- `core/src/ffi/contracts/engine.ffi-contract.json`
- `core/src/ffi/contracts/config.ffi-contract.json`
- `core/src/ffi/contracts/device_registry.ffi-contract.json`

### Validation Output

The validation script provides clear output:

✅ **Success**:
```
╔════════════════════════════════════════════════════════════╗
║           FFI Contract Validation Summary                  ║
╚════════════════════════════════════════════════════════════╝

  Valid contracts:     42
  Missing contracts:   0
  Mismatched:          0
  Orphaned:            0

✓ All contracts are valid!
```

❌ **Errors**:
```
  ✗ Missing contract entry: engine.start_loop
    Expected in: core/src/ffi/contracts/engine.ffi-contract.json
    Source: core/src/ffi/exports_engine.rs

To fix these issues:
  1. Run with --fix flag to auto-regenerate contracts:
     ./scripts/validate_ffi_contracts.sh --fix
```

⚠️ **Warnings** (orphaned contracts):
```
  ! Orphaned contract: engine.deprecated_function
    Found in: core/src/ffi/contracts/engine.ffi-contract.json
    No matching #[ffi_export] annotation in Rust code
```

## Logging Every FFI Change

1) After completing work (ideally right after a commit), generate a log:

```
just ffi-log <task-number>
```

Optional: compare against a base ref for clearer diffs:

```
just ffi-log 3 main
```

2) The script writes a pre-filled entry at `.spec-workflow/specs/rust-first-ai-native-ffi/Implementation Logs/`.
   - Template: `.spec-workflow/templates/ffi-implementation-log-template.md`
   - Sections cover: functions added/modified, contract changes, files touched, tests, performance, AI notes, follow-ups.

3) Open the generated file, fill the placeholders, and commit it with your code.

## Automation Helpers

- `scripts/log_ffi_implementation.sh` populates:
  - Timestamp, log id, task id
  - `git status` snapshot and `git diff --stat` (or `--base-ref` diff)
- `.githooks/post-commit` reminds you to log after each commit. Install hooks once:

```
./scripts/install-hooks.sh
```

## When to Log

- Any FFI-facing change: new or modified exports, contract/schema updates, codegen changes, watch-mode/tracing tweaks.
- Include tests you ran and performance impacts (codegen time, watch latency, tracing overhead).
- Capture AI notes: prompts that worked/failed and constraints to remember.

## Tips for Fast, Good Logs

- Keep bullets short; prefer concrete facts over prose.
- Note contract compatibility (breaking vs compatible) and follow-up actions.
- If nothing changed since last commit, the generated diff/stat blocks will show “no diff” — leave them as-is.

## Reference

- Spec: `rust-first-ai-native-ffi`
- Log directory: `.spec-workflow/specs/rust-first-ai-native-ffi/Implementation Logs/`
- Template: `.spec-workflow/templates/ffi-implementation-log-template.md`
- Script: `scripts/log_ffi_implementation.sh`
- Just command: `just ffi-log <id> [<base_ref>]`
