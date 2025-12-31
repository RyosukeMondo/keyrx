# Systematic Unwrap() Replacement Plan

## Executive Summary

**Total unwrap/expect calls**: 1,616
**Target**: Reduce to <50 calls (97% reduction)
**Timeline**: 4-6 weeks
**Priority**: High Risk → Medium Risk → Low Risk

---

## Distribution Analysis

| Crate | Calls | Percentage | Priority |
|-------|-------|------------|----------|
| `keyrx_daemon` | 1,216 | 75% | **CRITICAL** |
| `keyrx_compiler` | 340 | 21% | High |
| `keyrx_core` | 60 | 4% | Medium |

---

## Risk Classification

### 1. CRITICAL RISK (Runtime Panics)

**Daemon runtime code** - These can crash the running daemon:
- Event processing loops
- IPC message handling
- WebSocket communication
- Platform-specific input/output

**Impact**: Service crash, user data loss, system instability

### 2. HIGH RISK (Compilation Failures)

**Compiler** - These crash during configuration compilation:
- Rhai AST parsing
- Config validation
- Binary serialization

**Impact**: Users cannot compile configs, poor UX

### 3. MEDIUM RISK (Initialization)

**Setup/Configuration loading**:
- CLI argument parsing
- File loading at startup
- Test utilities

**Impact**: Startup failure (better than runtime panic)

### 4. LOW RISK (Test Code)

**Test-only code**:
- Test helpers
- Mock implementations
- Assertion utilities

**Impact**: Test failures only (acceptable in tests)

---

## Replacement Strategy

### Phase 1: Critical Runtime Code (Week 1-2)

**Target**: Daemon event processing and IPC

**Files to Fix**:
1. `keyrx_daemon/src/daemon/mod.rs` - Event loop unwraps
2. `keyrx_daemon/src/ipc/*.rs` - IPC message handling
3. `keyrx_daemon/src/web/ws.rs` - WebSocket handlers
4. `keyrx_daemon/src/platform/linux/*.rs` - evdev unwraps
5. `keyrx_daemon/src/platform/windows/*.rs` - Win32 unwraps

**Pattern**: Replace with proper `?` propagation

**Example**:
```rust
// BEFORE (PANIC RISK)
fn process_event(event: RawEvent) -> KeyEvent {
    let keycode = event.code().unwrap();  // ❌ Can panic
    KeyEvent::new(keycode)
}

// AFTER (SAFE)
fn process_event(event: RawEvent) -> Result<KeyEvent, ProcessError> {
    let keycode = event.code()
        .ok_or(ProcessError::InvalidKeyCode)?;  // ✅ Propagates error
    Ok(KeyEvent::new(keycode))
}
```

**Success Metric**: 0 unwraps in daemon runtime paths

---

### Phase 2: Compiler Error Handling (Week 3)

**Target**: keyrx_compiler parsing and validation

**Files to Fix**:
1. `keyrx_compiler/src/parser.rs` - Rhai AST unwraps
2. `keyrx_compiler/src/mphf_gen.rs` - Hash generation unwraps
3. `keyrx_compiler/src/dfa_gen.rs` - DFA construction unwraps
4. `keyrx_compiler/src/serialize.rs` - Serialization unwraps

**Pattern**: Use `anyhow::Context` for rich error messages

**Example**:
```rust
// BEFORE
fn parse_config(path: &Path) -> Config {
    let content = fs::read_to_string(path).unwrap();  // ❌
    let ast = parse_rhai(&content).unwrap();  // ❌
    compile_ast(ast).unwrap()  // ❌
}

// AFTER
use anyhow::{Context, Result};

fn parse_config(path: &Path) -> Result<Config> {
    let content = fs::read_to_string(path)
        .with_context(|| format!("Failed to read config: {}", path.display()))?;  // ✅

    let ast = parse_rhai(&content)
        .context("Rhai parse error")?;  // ✅

    compile_ast(ast)
        .context("Compilation failed")?  // ✅
}
```

**Success Metric**: Compiler never panics, always returns CompileError

---

### Phase 3: Initialization and CLI (Week 4)

**Target**: CLI argument parsing and startup

**Files to Fix**:
1. `keyrx_daemon/src/main.rs` - CLI parsing
2. `keyrx_daemon/src/cli/*.rs` - Subcommand handlers
3. `keyrx_daemon/src/config/*.rs` - Config loading

**Pattern**: Use `expect()` with clear messages only for initialization invariants

**Example**:
```rust
// BEFORE
fn main() {
    let config = load_config("config.toml").unwrap();  // ❌ Silent failure
    run_daemon(config);
}

// AFTER
fn main() -> anyhow::Result<()> {
    let config = load_config("config.toml")
        .context("Failed to load config.toml")?;  // ✅ Clear error

    run_daemon(config)?;
    Ok(())
}
```

**Acceptable `expect()` uses**:
- Mutex poisoning (indicates critical bug)
- Channel send failures (indicates bug in channel lifecycle)
- Thread spawn failures (OS-level issue)

**Success Metric**: <20 unwraps in initialization code, all with clear error messages

---

### Phase 4: Test Code Review (Week 5-6)

**Target**: Test utilities and helpers

**Strategy**: KEEP unwraps in test code (acceptable)

**Example**:
```rust
#[test]
fn test_simple_mapping() {
    let config = create_test_config().unwrap();  // ✅ OK in tests
    let output = process_event(event, &config).unwrap();  // ✅ OK in tests
    assert_eq!(output.len(), 1);
}
```

**Justification**: Test panics are acceptable - they indicate test failures, not production issues

**Action**: Document acceptable test unwraps, fix any in shared test utilities

**Success Metric**: 0 unwraps in reusable test utilities (e.g., `test_utils/`)

---

## Detailed File Breakdown

### Top 20 Files by Unwrap Count

(Generated via audit - use `grep -c` to verify)

| Rank | File | Approx. Calls | Risk Level |
|------|------|---------------|------------|
| 1 | `keyrx_daemon/src/daemon/mod.rs` | 80+ | CRITICAL |
| 2 | `keyrx_daemon/src/cli/config.rs` | 60+ | HIGH |
| 3 | `keyrx_daemon/src/main.rs` | 50+ | MEDIUM |
| 4 | `keyrx_compiler/src/parser.rs` | 45+ | HIGH |
| 5 | `keyrx_daemon/src/ipc/mod.rs` | 40+ | CRITICAL |
| 6 | `keyrx_daemon/src/web/api/profiles.rs` | 35+ | HIGH |
| 7 | `keyrx_daemon/src/platform/linux/input_capture.rs` | 30+ | CRITICAL |
| 8 | `keyrx_daemon/src/test_utils/output_capture.rs` | 30+ | LOW |
| 9 | `keyrx_compiler/src/mphf_gen.rs` | 25+ | HIGH |
| 10 | `keyrx_daemon/src/web/ws.rs` | 20+ | CRITICAL |

---

## Implementation Checklist

### Week 1: Critical Daemon Runtime
- [ ] Audit `keyrx_daemon/src/daemon/mod.rs` for event loop unwraps
- [ ] Replace unwraps in platform input/output code
- [ ] Add proper error types for platform operations
- [ ] Test error propagation with unit tests

### Week 2: IPC and WebSocket
- [ ] Replace unwraps in IPC message parsing
- [ ] Add error handling for WebSocket communication
- [ ] Implement graceful degradation for WS errors
- [ ] Add integration tests for error scenarios

### Week 3: Compiler Error Handling
- [ ] Audit parser unwraps, replace with `anyhow::Context`
- [ ] Add detailed error messages for parse failures
- [ ] Test compiler error reporting with invalid configs
- [ ] Document common compilation errors

### Week 4: CLI and Initialization
- [ ] Replace CLI unwraps with clear error messages
- [ ] Improve config loading error messages
- [ ] Add `--help` examples for common errors
- [ ] Test CLI error output formatting

### Week 5-6: Review and Documentation
- [ ] Run `cargo clippy` to find remaining unwraps
- [ ] Document acceptable unwrap patterns (tests, initialization invariants)
- [ ] Add `#[allow(clippy::unwrap_used)]` with justification where needed
- [ ] Update CLAUDE.md with error handling guidelines

---

## Error Type Hierarchy

### Recommended Error Structure

```rust
// keyrx_daemon/src/error.rs
use thiserror::Error;

#[derive(Error, Debug)]
pub enum DaemonError {
    #[error("Platform initialization failed: {0}")]
    PlatformInit(#[from] PlatformError),

    #[error("Event processing error: {0}")]
    EventProcessing(String),

    #[error("IPC communication error: {0}")]
    Ipc(#[from] IpcError),

    #[error("WebSocket error: {0}")]
    WebSocket(#[from] WsError),

    #[error("Configuration error: {0}")]
    Config(#[from] ConfigError),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

#[derive(Error, Debug)]
pub enum PlatformError {
    #[error("Failed to open input device: {0}")]
    DeviceOpen(String),

    #[error("Invalid event code: {0}")]
    InvalidEventCode(u16),

    #[error("Failed to inject output: {0}")]
    OutputInjection(String),
}
```

---

## Forbidden Patterns

❌ **NEVER do this**:
```rust
let value = some_option.unwrap();  // No error message
let result = fallible_op().expect("failed");  // Vague message
```

✅ **ALWAYS do this**:
```rust
let value = some_option
    .ok_or_else(|| Error::new("Specific reason why this is None"))?;

let result = fallible_op()
    .context("What operation failed and why it matters")?;
```

---

## Progress Tracking

### Metrics to Track

- **Weekly**:
  - Total unwrap count (run: `grep -r "\.unwrap()" --include="*.rs" | wc -l`)
  - Unwraps per crate
  - Unwraps in critical paths (daemon runtime, IPC)

- **Final Goal**:
  - `keyrx_core`: <10 unwraps (only in tests)
  - `keyrx_compiler`: <20 unwraps (only in tests + initialization)
  - `keyrx_daemon`: <30 unwraps (only in tests + initialization invariants)
  - **Total**: <60 unwraps (<4% of original 1,616)

---

## Success Criteria

1. **Zero runtime panics** - Daemon never crashes from unwrap
2. **Compiler errors are actionable** - Users get helpful error messages
3. **CLI failures are clear** - Users know what to fix
4. **Tests still use unwraps** - Test code clarity maintained
5. **Clippy compliance** - No unwrap warnings in production code

---

## Next Steps

1. Get approval for this plan
2. Create tracking issue in GitHub
3. Break down into 10-15 smaller tasks
4. Assign weekly milestones
5. Start with Phase 1 (daemon runtime)

---

**Document Version**: 1.0
**Last Updated**: 2025-12-31
**Owner**: Architecture Refactoring Initiative
