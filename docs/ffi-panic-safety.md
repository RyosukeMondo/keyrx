# FFI Panic Safety

## Overview

This document describes the panic safety guarantees and error handling strategies used at the FFI (Foreign Function Interface) boundary in KeyRx. All FFI exports follow strict rules to ensure they never panic across language boundaries, which would cause undefined behavior and potential crashes in the Flutter UI.

## Core Principles

### 1. **Never Panic Across FFI Boundary**

All functions exported via `#[no_mangle] pub extern "C"` **must never panic**. Panicking across FFI boundaries causes undefined behavior in C/C++/Dart code.

**Enforcement:**
- All `Result` types are handled with `match`, `if let`, or safe methods like `unwrap_or()`, `unwrap_or_else()`
- No bare `unwrap()` or `expect()` calls in FFI export functions
- Errors are returned as error codes (integers) or null pointers with logging

### 2. **Safe Fallback Values**

FFI functions use safe fallback patterns:

```rust
// ✅ GOOD: Safe fallback to null pointer
fn ffi_json<T: serde::Serialize>(result: FfiResult<T>) -> *mut c_char {
    let payload = serialize_ffi_result(&result).unwrap_or_else(|e| {
        format!("error:{{\"code\":\"SERIALIZATION_FAILED\",\"message\":\"{e}\"}}")
    });
    CString::new(payload)
        .map(CString::into_raw)
        .unwrap_or(std::ptr::null_mut())
}

// ✅ GOOD: Safe fallback to error code
pub extern "C" fn keyrx_cancel_discovery() -> i32 {
    discovery::cancel_discovery().unwrap_or(-4)
}

// ✅ GOOD: Safe fallback to false
pub extern "C" fn keyrx_is_bypass_active() -> bool {
    engine::is_bypass_mode_active().unwrap_or(false)
}

// ❌ BAD: Can panic across FFI boundary
pub extern "C" fn bad_example() -> i32 {
    some_operation().unwrap()  // NEVER DO THIS
}
```

### 3. **Error Reporting Patterns**

#### Pattern 1: Error Codes (Integer Return)

Used for simple operations where detailed error information is not critical:

```rust
#[no_mangle]
pub unsafe extern "C" fn keyrx_select_device(path: *const c_char) -> i32 {
    let path = match cstr_to_str(path) {
        Ok(s) => s,
        Err(code) => return code,  // Return error code
    };

    let res = with_ctx(|ctx| {
        ensure_domain::<DeviceFfi>(ctx)?;
        device::select_device(ctx, path)
    });

    match res {
        Ok(_) => 0,        // Success
        Err(err) => {
            if err.code == "NOT_FOUND" {
                -3         // Specific error code
            } else {
                -4         // Generic error
            }
        }
    }
}
```

**Common error codes:**
- `0`: Success
- `-1`: Generic error / invalid input
- `-2`: UTF-8 conversion error
- `-3`: Not found
- `-4`: Internal error / poisoned lock

#### Pattern 2: JSON Response (String Return)

Used for operations that need to return detailed error information:

```rust
#[no_mangle]
pub unsafe extern "C" fn keyrx_eval(command: *const c_char) -> *mut c_char {
    let cmd = match cstr_to_str(command) {
        Ok(s) => s,
        Err(code) => {
            let err = FfiError::invalid_utf8(format!("command ({code})"));
            return ffi_error(err);  // Returns JSON error
        }
    };
    ffi_json(script::eval(cmd))  // Returns "ok:<json>" or "error:<json>"
}
```

The response format is:
- Success: `ok:{"result": ...}`
- Error: `error:{"code": "ERROR_CODE", "message": "...", "hint": "..."}`

#### Pattern 3: Null Pointer Return

Used for functions that return C strings:

```rust
#[no_mangle]
pub extern "C" fn keyrx_log_drain() -> *mut c_char {
    if let Some(bridge) = get_log_bridge() {
        let entries = bridge.drain();
        match serde_json::to_string(&entries) {
            Ok(json) => match CString::new(json) {
                Ok(c_string) => c_string.into_raw(),
                Err(_) => std::ptr::null_mut(),  // Null on error
            },
            Err(_) => std::ptr::null_mut(),  // Null on error
        }
    } else {
        std::ptr::null_mut()  // Null when bridge unavailable
    }
}
```

The caller must check for `NULL` before dereferencing.

### 4. **Structured Logging**

All errors are logged with structured context before returning to the caller:

```rust
#[no_mangle]
pub extern "C" fn keyrx_log_bridge_init() -> i32 {
    if let Ok(mut bridge) = LOG_BRIDGE.write() {
        if bridge.is_none() {
            *bridge = Some(LogBridge::new());
            tracing::debug!(
                service = "keyrx",
                event = "log_bridge_init",
                component = "ffi_observability",
                "Log bridge initialized"
            );
        }
        0
    } else {
        tracing::error!(
            service = "keyrx",
            event = "log_bridge_init_failed",
            component = "ffi_observability",
            error = "lock_poisoned",
            "Failed to acquire log bridge lock"
        );
        -1
    }
}
```

## FFI Export Files

### Core Exports

| File | Purpose | Panic-Safe |
|------|---------|------------|
| `exports_compat.rs` | Legacy FFI exports for Flutter UI | ✅ Yes |
| `exports_metrics.rs` | Metrics and observability re-exports | ✅ Yes |
| `exports_transition_log.rs` | Transition log access | ✅ Yes |

### Domain Implementations

All domain implementations in `core/src/ffi/domains/*.rs` follow panic-safe patterns:

| Domain | Module | Panic-Safe |
|--------|--------|------------|
| Device | `device.rs` | ✅ Yes |
| Discovery | `discovery.rs` | ✅ Yes |
| Engine | `engine.rs` | ✅ Yes |
| Script | `script.rs` | ✅ Yes |
| Testing | `testing.rs` | ✅ Yes |
| Diagnostics | `diagnostics.rs` | ✅ Yes |
| Recording | `recording.rs` | ✅ Yes |
| Analysis | `analysis.rs` | ✅ Yes |
| Observability | `observability.rs` | ✅ Yes |

## Audit Results

**Status: ✅ PASS - FFI Boundary is Panic-Safe**

### Findings

1. **No bare `unwrap()` or `expect()` in FFI exports**
   - All `unwrap()` calls found were in test code only
   - Production code uses `unwrap_or()`, `unwrap_or_else()`, or explicit error handling

2. **Safe C string conversion**
   - All C string conversions handle null pointers and UTF-8 errors
   - Helper function `cstr_to_str()` returns `Result` with error codes

3. **Lock poisoning handled**
   - All `Mutex`/`RwLock` acquisitions check for `Err` and return error codes
   - No assumptions about lock availability

4. **Serialization failures handled**
   - JSON serialization uses `match` or safe fallback
   - `ffi_json()` helper provides automatic error serialization

### Code Quality

- ✅ All FFI functions have `# Safety` documentation
- ✅ Memory management rules documented
- ✅ Thread safety guarantees specified
- ✅ Error codes and return values documented

## Testing Guidelines

### Unit Tests

Test FFI functions with invalid inputs to verify panic safety:

```rust
#[test]
fn test_null_pointer_handling() {
    let result = unsafe { keyrx_select_device(std::ptr::null()) };
    assert_eq!(result, -1);  // Should return error, not panic
}

#[test]
fn test_invalid_utf8() {
    let invalid_utf8 = vec![0xFF, 0xFE, 0x00];
    let c_str = std::ffi::CString::new(invalid_utf8).unwrap();
    let result = unsafe { keyrx_eval(c_str.as_ptr()) };
    assert!(!result.is_null());  // Should return error JSON
}
```

### Integration Tests

Test FFI boundary under stress conditions:
- Concurrent access from multiple threads
- Lock poisoning scenarios
- Resource exhaustion (OOM, full disk, etc.)

## Maintenance Guidelines

### When Adding New FFI Exports

1. **Never use bare `unwrap()` or `expect()`**
   - Use `unwrap_or()`, `unwrap_or_else()`, or `match`/`if let`

2. **Choose appropriate error pattern**
   - Integer codes for simple operations
   - JSON responses for detailed errors
   - Null pointers for optional results

3. **Log all errors**
   - Use structured logging with `tracing::error!()`
   - Include context: service, event, component, error type

4. **Document safety requirements**
   - Add `# Safety` section explaining pointer requirements
   - Document memory management expectations
   - Specify thread safety guarantees

5. **Add tests**
   - Test with null pointers
   - Test with invalid UTF-8
   - Test with poisoned locks
   - Test with serialization failures

### When Modifying Existing FFI Exports

1. **Verify panic safety**
   - Run `cargo clippy -- -D clippy::unwrap_used` on FFI modules
   - Check for new `unwrap()` or `expect()` calls

2. **Maintain backward compatibility**
   - Don't change error code meanings
   - Don't change JSON response format
   - Document breaking changes

3. **Update tests**
   - Add tests for new error conditions
   - Verify existing error handling still works

## Future Improvements

### Automated Checks

Consider adding to `.cargo/config.toml`:

```toml
[target.'cfg(all())']
rustflags = [
    "-W", "clippy::unwrap_used",
    "-W", "clippy::expect_used",
]
```

Then allow specific modules:
```rust
#![allow(clippy::unwrap_used)]  // Only in test modules
```

### Static Analysis

Run clippy with panic-detection lints:

```bash
cargo clippy --all-targets -- \
    -W clippy::unwrap_used \
    -W clippy::expect_used \
    -W clippy::panic \
    -W clippy::todo \
    -W clippy::unimplemented
```

### Runtime Monitoring

Track FFI error rates in metrics:
- Number of null pointer returns
- Number of error codes returned
- Lock poisoning incidents
- Serialization failures

## References

- [Rust FFI Omnibus](https://jakegoulding.com/rust-ffi-omnibus/)
- [Rustonomicon: FFI](https://doc.rust-lang.org/nomicon/ffi.html)
- [Safe FFI Guide](https://doc.rust-lang.org/nightly/nomicon/ffi.html#foreign-function-interface)

## Related Documents

- `panic-handling.md` - Overall panic handling architecture
- `error-handling.md` - Error type hierarchy and patterns
- `ffi-conventions.md` - General FFI conventions and patterns
