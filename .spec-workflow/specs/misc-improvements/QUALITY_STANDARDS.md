# KeyRx Quality Standards Reference Guide

**Version:** 1.0
**Last Updated:** 2025-12-12

This document defines the code quality standards for the KeyRx project. All contributions must adhere to these standards, which are enforced through CI/CD pipelines and pre-commit hooks.

---

## Table of Contents

1. [Overview](#1-overview)
2. [Code Metrics](#2-code-metrics)
3. [Test Coverage](#3-test-coverage)
4. [Structured Logging](#4-structured-logging)
5. [Documentation](#5-documentation)
6. [CI Enforcement](#6-ci-enforcement)
7. [Examples](#7-examples)
8. [Quick Reference](#8-quick-reference)

---

## 1. Overview

### Why Quality Standards?

Quality standards ensure:
- **Maintainability** - Code is easy to understand and modify
- **Testability** - Code can be tested in isolation
- **Reliability** - Comprehensive test coverage catches regressions
- **Consistency** - All contributors follow the same patterns

### Core Principles

- **SOLID** - Single responsibility, open/closed, Liskov substitution, interface segregation, dependency inversion
- **DI (Dependency Injection)** - All external dependencies must be injectable
- **SSOT (Single Source of Truth)** - No duplicated business logic
- **KISS (Keep It Simple)** - Prefer simple solutions over complex ones
- **Fail Fast** - Validate at entry, reject invalid immediately

---

## 2. Code Metrics

### 2.1 File Size Limit

**Target:** Maximum 500 lines per file (excluding comments/blank lines)

**Why:** Large files indicate poor separation of concerns and slow incremental compilation.

**Good Example:**
```
src/services/
├── device.rs       (89 lines)
├── profile.rs      (413 lines)
├── runtime.rs      (645 lines)  # Approaching limit
└── traits.rs       (386 lines)
```

**Bad Example:**
```
src/engine/
└── monolith.rs     (1,500 lines)  # Split into modules!
```

**How to Fix:** Split into logical submodules using Rust's module system.

### 2.2 Function Length Limit

**Target:** Maximum 50 lines per function (excluding comments/blank lines)

**Why:** Long functions are hard to test, understand, and maintain.

**Good Example:**
```rust
fn process_device_event(&self, event: DeviceEvent) -> Result<()> {
    let validated = self.validate_event(&event)?;
    let transformed = self.transform_event(validated)?;
    self.dispatch_event(transformed)
}
```

**Bad Example:**
```rust
fn process_device_event(&self, event: DeviceEvent) -> Result<()> {
    // 150 lines of mixed validation, transformation, and dispatch logic
    // Hard to test individual parts
    // Impossible to understand at a glance
}
```

**How to Fix:** Extract logical steps into helper functions.

### 2.3 Accepted Exceptions

Some code patterns are exempted from strict line limits:

| Pattern | Reason |
|---------|--------|
| Template functions | Static HTML/CSS/JS strings |
| Data definitions | Declarative data structures |
| State machines | Match dispatchers with clear branches |
| CLI handlers | Top-level command dispatchers |

---

## 3. Test Coverage

### 3.1 Overall Coverage

**Target:** >=80% line coverage

**Why:** Ensures most code paths are exercised by tests.

**Measurement:**
```bash
cargo llvm-cov --lib --summary-only
```

### 3.2 Critical Path Coverage

**Target:** >=90% line coverage for critical modules

**Critical Modules:**
- `src/services/` - Business logic
- `src/api.rs` - Public API layer
- `src/engine/` - Core engine logic
- `src/ffi/` - FFI boundary layer

**Why:** Critical paths handle core functionality and must be thoroughly tested.

### 3.3 Testing Best Practices

**Use Mocks for Unit Tests:**
```rust
#[tokio::test]
async fn test_list_devices_with_mock() {
    let mock = MockDeviceService::new()
        .with_devices(vec![test_device()]);

    let api = ApiContext::new(
        Arc::new(mock),
        Arc::new(MockProfileService::new()),
        Arc::new(MockRuntimeService::new()),
    );

    let devices = api.list_devices().await.unwrap();
    assert_eq!(devices.len(), 1);
}
```

**Test Error Paths:**
```rust
#[tokio::test]
async fn test_list_devices_error_handling() {
    let mock = MockDeviceService::new()
        .with_error(DeviceServiceError::NotFound("test".into()));

    let api = ApiContext::new(Arc::new(mock), ...);
    let result = api.list_devices().await;
    assert!(result.is_err());
}
```

---

## 4. Structured Logging

### 4.1 Required Fields

All log entries must include:

| Field | Type | Description |
|-------|------|-------------|
| timestamp | i64 | Unix timestamp in milliseconds |
| level | String | TRACE, DEBUG, INFO, WARN, ERROR |
| target | String | Module/service name |
| message | String | Human-readable event description |
| fields | Object | Additional context (optional) |

### 4.2 JSON Format

**Example Output:**
```json
{
  "timestamp": 1702339200000,
  "level": "INFO",
  "target": "keyrx::services::device",
  "message": "Device connected",
  "fields": {
    "device_key": "usb-001",
    "vendor_id": "0x1234"
  }
}
```

### 4.3 Security Requirements

**Never log:**
- Passwords or secrets
- API keys or tokens
- Personal Identifiable Information (PII)
- Session identifiers

**Good:**
```rust
info!(device_key = %device.key, "Device connected");
```

**Bad:**
```rust
info!(password = %user.password, "User authenticated");  // NEVER!
```

---

## 5. Documentation

### 5.1 Public API Documentation

**Target:** Zero warnings from `cargo doc --no-deps`

**All public items must have:**
- Description of purpose
- Parameter documentation
- Return value description
- Error conditions (for Result types)

### 5.2 Documentation Template

```rust
/// Short description of what this function does.
///
/// Longer explanation if needed, describing behavior,
/// edge cases, and important notes.
///
/// # Arguments
///
/// * `param1` - Description of first parameter
/// * `param2` - Description of second parameter
///
/// # Returns
///
/// Description of the return value.
///
/// # Errors
///
/// Returns an error if:
/// * Condition 1
/// * Condition 2
///
/// # Examples
///
/// ```rust
/// let result = my_function(arg1, arg2)?;
/// assert_eq!(result, expected);
/// ```
pub fn my_function(param1: Type1, param2: Type2) -> Result<ReturnType> {
    // implementation
}
```

### 5.3 When to Add Examples

Add examples for:
- Complex APIs with multiple use cases
- Functions with non-obvious behavior
- Public service methods
- FFI boundary functions

---

## 6. CI Enforcement

### 6.1 Pre-commit Hooks

Automatically run before each commit:

```bash
# 1. Format check
cargo fmt --check

# 2. Clippy lints
cargo clippy -- -D warnings

# 3. Unit tests
cargo test --lib
```

### 6.2 CI Pipeline Checks

| Check | Command | Failure Condition |
|-------|---------|-------------------|
| Format | `cargo fmt --check` | Any formatting issues |
| Clippy | `cargo clippy -- -D warnings` | Any warnings |
| Tests | `cargo nextest run` | Any test failures |
| Coverage | `cargo llvm-cov --fail-under-lines 80` | Coverage < 80% |
| Docs | `cargo doc --no-deps` | Any warnings |

### 6.3 Local Verification

Run all checks locally before pushing:

```bash
# Full CI check
just ci-check

# Individual checks
just fmt-check
just clippy
just test
just doc-check
just coverage-check
```

---

## 7. Examples

### 7.1 Good: Clean, Testable Function

```rust
/// Validates and processes a device connection event.
///
/// # Arguments
///
/// * `event` - The device connection event to process
///
/// # Returns
///
/// The processed device view on success.
///
/// # Errors
///
/// Returns an error if validation fails or the device is unknown.
pub async fn process_connection(
    &self,
    event: ConnectionEvent,
) -> Result<DeviceView, DeviceError> {
    let validated = self.validate_event(&event)?;
    let device = self.lookup_device(&validated.device_id)?;
    self.update_connection_state(device, validated.state).await
}
```

### 7.2 Bad: Untestable Monolith

```rust
// Missing documentation
pub async fn process_connection(event: ConnectionEvent) -> Result<DeviceView> {
    // 100+ lines mixing validation, lookup, and state update
    // Uses global singletons - cannot mock
    // No error handling granularity
}
```

### 7.3 Good: Proper Dependency Injection

```rust
pub struct DeviceProcessor {
    device_service: Arc<dyn DeviceServiceTrait>,
    config: Arc<dyn ConfigProvider>,
}

impl DeviceProcessor {
    pub fn new(
        device_service: Arc<dyn DeviceServiceTrait>,
        config: Arc<dyn ConfigProvider>,
    ) -> Self {
        Self { device_service, config }
    }
}
```

### 7.4 Bad: Hardcoded Dependencies

```rust
pub struct DeviceProcessor {
    device_service: DeviceService,  // Concrete type - cannot mock!
}

impl DeviceProcessor {
    pub fn new() -> Self {
        Self {
            device_service: DeviceService::new(),  // Hardcoded!
        }
    }
}
```

---

## 8. Quick Reference

### Commands Cheatsheet

| Command | Purpose |
|---------|---------|
| `just check` | Run all quality checks |
| `just ci-check` | Full CI verification |
| `just fmt` | Format code |
| `just clippy` | Run linter |
| `just test` | Run tests |
| `just doc-check` | Check documentation |
| `just coverage-check` | Check coverage threshold |

### Metrics Summary

| Metric | Target | Enforcement |
|--------|--------|-------------|
| File size | <500 lines | Code review |
| Function length | <50 lines | Code review |
| Overall coverage | >=80% | CI gate |
| Critical coverage | >=90% | CI gate (future) |
| Doc warnings | 0 | CI gate |
| Clippy warnings | 0 | Pre-commit + CI |

### Resources

- [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- [cargo-llvm-cov Documentation](https://github.com/taiki-e/cargo-llvm-cov)
- [KeyRx CODEBASE_EVALUATION.md](../../../CODEBASE_EVALUATION.md)

---

**Questions?** Reach out to the team or consult the CODEBASE_EVALUATION.md for detailed analysis.
