# Panic Prevention Lints

## Overview

As part of the unwrap-panic-hardening specification, the KeyRx core library enforces strict linting rules to prevent panic-inducing operations in production code. These lints ensure that all error paths are explicitly handled and prevent regressions.

## Configured Lints

The following Clippy lints are configured in `core/Cargo.toml`:

```toml
[lints.clippy]
unwrap_used = "deny"     # Prevents .unwrap() calls
expect_used = "deny"     # Prevents .expect() calls
panic = "deny"           # Prevents panic!() macro calls
# Note: indexing_slicing not currently enabled but should be considered for future work
```

### Why These Lints?

1. **`unwrap_used` (deny)**: `unwrap()` panics on `None`/`Err` values. All error cases must be explicitly handled.

2. **`expect_used` (deny)**: Similar to `unwrap()` but with a custom message. Still causes panics.

3. **`panic` (deny)**: Direct `panic!()` calls are not allowed. Use proper error handling instead.

Note: `indexing_slicing` lint (which catches array/slice indexing that can panic on out-of-bounds) is not currently enabled due to the large number of existing violations. This should be considered for future hardening work.

## How to Handle Violations

### 1. Replace unwrap() with Proper Error Handling

**Before:**
```rust
let value = some_option.unwrap();
```

**After:**
```rust
use crate::safety::extensions::OptionExt;

let value = some_option.unwrap_or_log("default value", "description");
```

Or use the `?` operator:
```rust
let value = some_option.ok_or_else(|| CriticalError::InvalidState {
    context: "Expected value to be present".into(),
})?;
```

### 2. Replace expect() with Context-Rich Errors

**Before:**
```rust
let config = load_config().expect("config must exist");
```

**After:**
```rust
let config = load_config().map_err(|e| {
    CriticalError::ConfigLoadFailed {
        reason: format!("Failed to load config: {}", e),
    }
})?;
```

### 3. Replace panic!() with Error Returns

**Before:**
```rust
if invalid_state {
    panic!("This should never happen!");
}
```

**After:**
```rust
if invalid_state {
    return Err(CriticalError::InvalidState {
        context: "Unexpected state detected".into(),
    }.into());
}
```

### 4. Handle Indexing Safely

**Before:**
```rust
let first = items[0];  // Warning: may panic if empty
```

**After:**
```rust
let first = items.get(0)
    .ok_or_else(|| CriticalError::InvalidState {
        context: "Expected non-empty items".into(),
    })?;
```

Or use iterators:
```rust
let first = items.first()
    .ok_or_else(|| /* error */)?;
```

## When to Use #[allow()] Annotations

In rare cases where an operation is provably safe but the compiler cannot verify it, you may use `#[allow()]` annotations. **This should be extremely rare** and requires:

1. A detailed comment explaining why it's safe
2. Proof that the operation cannot panic
3. Consideration of whether the code can be refactored to avoid the need

### Example: Justified Allow

```rust
// SAFETY: We just checked that the vector has exactly 3 elements above.
// This indexing cannot panic because we validated the length.
#[allow(clippy::indexing_slicing)]
let (r, g, b) = (rgb[0], rgb[1], rgb[2]);
```

### Example: Unjustified (Do Not Do This)

```rust
// SAFETY: This should always work
#[allow(clippy::unwrap_used)]
let value = map.get(key).unwrap();
```

This is not acceptable because:
- "Should always work" is not proof
- The map lookup can fail
- Proper error handling should be used

## Available Helper Traits

The codebase provides extension traits to make migration easier:

### OptionExt (in `core/src/safety/extensions.rs`)

```rust
use crate::safety::extensions::OptionExt;

// Log and use fallback value
let value = option.unwrap_or_log(fallback, "description of what failed");

// Log and return default
let value = option.unwrap_or_default_log("description");
```

### ResultExt (in `core/src/safety/extensions.rs`)

```rust
use crate::safety::extensions::ResultExt;

// Map Result to CriticalResult
let result: CriticalResult<T> = result.map_critical();

// Log error and use fallback
let value = result.unwrap_or_else_log(|e| {
    fallback_value
}, "operation description");
```

## Testing Code

Tests are more lenient and may use `unwrap()` in specific cases:

```rust
#[cfg(test)]
mod tests {
    #[test]
    fn test_something() {
        // In tests, unwrap() is acceptable for setup
        // that should never fail in a test environment
        let config = Config::default();

        // But prefer proper assertions:
        let result = operation();
        assert!(result.is_ok(), "operation should succeed");
        let value = result.unwrap();
    }
}
```

However, even in tests, consider using:
- `assert!` and `assert_eq!` for validation
- `expect()` with descriptive messages when unwrapping
- `unwrap()` only when the panic message clearly indicates the test failure

## Enforcement Strategy

1. **CI/CD**: The lints run in CI and will fail the build on violations
2. **Local Development**: Run `cargo clippy` before committing
3. **Pre-commit Hooks**: Configure pre-commit hooks to run clippy:

```bash
# .git/hooks/pre-commit
#!/bin/sh
cargo clippy --all-targets -- -D warnings
```

## Migration Guide

When you encounter lint violations in existing code:

1. **Understand the Error Path**: What can fail? Why?
2. **Choose the Right Approach**:
   - Use `?` operator if in a function returning `Result`
   - Use `unwrap_or_log()` for optional values with fallbacks
   - Use `map_err()` to convert errors to `CriticalError`
   - Use `unwrap_or_else()` for complex fallback logic
3. **Test the Error Path**: Add tests that trigger the error condition
4. **Document Assumptions**: Add comments explaining error handling strategy

## Related Components

- **CriticalError** (`core/src/errors/critical.rs`): Error type for critical paths
- **CriticalResult** (`core/src/errors/critical_result.rs`): Result type without panic methods
- **PanicGuard** (`core/src/safety/panic_guard.rs`): Catches panics in critical sections
- **Extension Traits** (`core/src/safety/extensions.rs`): Helper methods for safe unwrapping

## FAQ

**Q: Why deny unwrap/expect instead of just warning?**

A: Panics in a keyboard remapping tool can cause the user to lose control of their keyboard. This is a critical safety issue. By denying these operations at compile time, we prevent accidental panics from reaching production.

**Q: What if I need to unwrap in initialization code that only runs once?**

A: Even initialization code should handle errors gracefully. Return an error and let the application decide how to handle it (e.g., show error dialog, use defaults, exit gracefully).

**Q: Can I disable these lints for a whole module?**

A: No. These are project-wide safety requirements. If you believe a specific operation is safe, use a scoped `#[allow()]` on the smallest possible scope (single statement/expression) with a detailed comment.

**Q: What about third-party dependencies that might panic?**

A: Wrap calls to third-party code that might panic with `PanicGuard::catch()` in critical paths. See the panic handling architecture documentation for details.

## See Also

- [Panic Handling Architecture](./panic-handling.md) - Overview of panic recovery system
- [Error Handling Patterns](./errors/README.md) - Error type design and usage
- [FFI Safety](./ffi-panic-safety.md) - Panic handling at FFI boundaries
