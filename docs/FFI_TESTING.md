# FFI Testing Strategy

This document describes the comprehensive testing strategy for the AI-native FFI workflow.

## Table of Contents

1. [Overview](#overview)
2. [Test Organization](#test-organization)
3. [Test Categories](#test-categories)
4. [Running Tests](#running-tests)
5. [Coverage Requirements](#coverage-requirements)
6. [Testing Tools](#testing-tools)
7. [Best Practices](#best-practices)
8. [Examples](#examples)

## Overview

The FFI testing infrastructure ensures reliability across three key areas:

1. **Contract Derivation** - Verify that `#[ffi_export]` macro correctly generates contracts
2. **FFI Boundary Invariants** - Ensure contracts and implementations stay in sync
3. **Tool Integration** - Validate that FFI development tools work correctly

## Test Organization

```
core/tests/unit/ffi/
├── mod.rs                           # Module declarations
├── contract_derivation_tests.rs     # Contract generation from macros
├── contract_validation_tests.rs     # Contract schema validation
├── breaking_change_tests.rs         # Breaking change detection
├── property_tests.rs                # FFI boundary invariants
└── tool_integration_tests.rs        # Tool functionality tests

examples/ffi_development/
├── minimal_ffi_function/            # Basic FFI function example
├── complex_ffi_types/               # Complex types and custom errors
├── error_handling/                  # Error handling patterns
└── streaming_ffi/                   # Async/streaming patterns (planned)
```

## Test Categories

### 1. Contract Derivation Tests

**Location**: `core/tests/unit/ffi/contract_derivation_tests.rs`

**Purpose**: Verify that the `#[ffi_export]` macro correctly derives contract JSON from Rust function signatures.

**Test Coverage**:
- ✅ Basic function with simple types
- ✅ `Result<T, E>` return types
- ✅ `Option<T>` parameters (optional fields)
- ✅ Doc comment extraction
- ✅ Domain and version attributes
- ✅ Complex custom types
- ✅ Generic functions
- ✅ Multiple error types

**Example**:
```rust
#[test]
fn test_basic_function_contract_derivation() {
    // Verify that a simple function generates correct contract JSON
    let expected = r#"{
        "name": "get_value",
        "parameters": [{"name": "key", "type": "string", "required": true}],
        "returns": {"type": "string"}
    }"#;

    // Contract derivation would happen here
    assert_contract_matches(expected);
}
```

### 2. Contract Validation Tests

**Location**: `core/tests/unit/ffi/contract_validation_tests.rs`

**Purpose**: Ensure contract JSON adheres to schema and catches common errors.

**Test Coverage**:
- ✅ Valid contracts pass validation
- ✅ Missing required fields are detected
- ✅ Function names must be valid identifiers
- ✅ Parameter types must be valid
- ✅ Duplicate function names are detected
- ✅ Version strings follow semantic versioning
- ✅ Contracts can be loaded from JSON files

**Example**:
```rust
#[test]
fn test_contract_requires_domain() {
    let json = r#"{
        "version": "1.0.0",
        "functions": []
    }"#;

    let result: Result<FfiContract, _> = serde_json::from_str(json);
    assert!(result.is_err(), "Contract without domain should fail");
}
```

### 3. Breaking Change Detection Tests

**Location**: `core/tests/unit/ffi/breaking_change_tests.rs`

**Purpose**: Verify that breaking changes in FFI contracts are correctly identified.

**Test Coverage**:
- ✅ Removed functions → Breaking
- ✅ Changed parameter types → Breaking
- ✅ Changed return types → Breaking
- ✅ Renamed functions → Breaking
- ✅ Added required parameters → Breaking
- ✅ Added optional parameters → Compatible
- ✅ Added new functions → Compatible
- ✅ Changed domain → Breaking
- ✅ Made parameters optional → Compatible
- ✅ Reordered parameters → Breaking

**Example**:
```rust
#[test]
fn test_removed_function_is_breaking() {
    let old_contract = json!({
        "functions": [
            {"name": "get_value"},
            {"name": "set_value"}
        ]
    });

    let new_contract = json!({
        "functions": [
            {"name": "get_value"}
        ]
    });

    let changes = detect_breaking_changes(&old_contract, &new_contract);
    assert!(!changes.is_empty());
}
```

### 4. Property Tests

**Location**: `core/tests/unit/ffi/property_tests.rs`

**Purpose**: Verify that fundamental invariants hold across the FFI boundary.

**Invariants Tested**:
- ✅ Every contract has a corresponding Rust function
- ✅ Every Rust FFI function has a contract
- ✅ Function names are unique within a domain
- ✅ All contract JSON files are valid
- ✅ Contract files follow naming convention
- ✅ Contracts specify valid domains
- ✅ Contract versions are semantic versions
- ✅ All FFI functions are annotated with `#[ffi_export]`

**Example**:
```rust
#[test]
fn property_all_contracts_have_functions() {
    let rust_functions = discover_rust_ffi_functions();
    let contract_functions = discover_contract_functions();

    for contract_fn in &contract_functions {
        let has_rust_impl = rust_functions.iter().any(|rust_fn| {
            rust_fn.name == contract_fn.name && rust_fn.domain == contract_fn.domain
        });

        assert!(has_rust_impl, "Contract {} has no Rust implementation", contract_fn.name);
    }
}
```

### 5. Tool Integration Tests

**Location**: `core/tests/unit/ffi/tool_integration_tests.rs`

**Purpose**: Verify that FFI development tools function correctly.

**Tools Tested**:
- ✅ Contract validation script (`validate_ffi_contracts.sh`)
- ✅ Breaking change detector (`detect_ffi_breaking_changes.sh`)
- ✅ Module splitter (`module_splitter`)
- ✅ Contract metrics generator (`generate_contract_metrics.sh`)
- ✅ Module size analyzer (`analyze_module_sizes.sh`)
- ✅ FFI implementation logger (`log_ffi_implementation.sh`)
- ✅ Watch mode (`ffi_watch.sh`)
- ✅ Dart bindings generator (`generate_dart_bindings`)
- ✅ Incremental generation cache

**Example**:
```rust
#[test]
fn test_module_size_analysis() {
    let output = Command::new("bash")
        .arg("scripts/analyze_module_sizes.sh")
        .output()
        .expect("Failed to run analyzer");

    assert!(output.status.success());

    let json_path = ".spec-workflow/analysis/module-sizes.json";
    let content = std::fs::read_to_string(json_path).unwrap();
    let json: serde_json::Value = serde_json::from_str(&content).unwrap();

    assert!(json.is_object() || json.is_array());
}
```

## Running Tests

### Run All FFI Tests

```bash
# Run all unit tests including FFI tests
cd core
cargo test unit::ffi

# Run with verbose output
cargo test unit::ffi -- --nocapture

# Run specific test
cargo test unit::ffi::contract_derivation_tests
```

### Run Example Projects

```bash
# Navigate to an example
cd examples/ffi_development/minimal_ffi_function

# Build the example
cargo build

# Run tests
cargo test

# Run with verbose output
cargo test -- --nocapture
```

### Run Tool Integration Tests

```bash
# These tests verify FFI tools work correctly
cargo test unit::ffi::tool_integration_tests
```

### Run in CI

The FFI tests are automatically included in the CI pipeline:

```bash
# CI runs all tests including FFI tests
# See .github/workflows/ci.yml

# Locally simulate CI test run
cargo nextest run --all-features --filter-expr 'test(unit)'
```

## Coverage Requirements

### Overall Coverage Target

**Minimum**: 80% line coverage for all tool code

### Coverage by Component

| Component | Target | Priority |
|-----------|--------|----------|
| Contract derivation macro | 90% | High |
| Contract validation | 85% | High |
| Breaking change detector | 85% | High |
| Module splitter | 80% | Medium |
| Dart bindings generator | 85% | High |
| Watch mode | 75% | Medium |
| Metrics tools | 70% | Low |

### Checking Coverage

```bash
# Generate coverage report
cd core
cargo llvm-cov --all-features --html

# View HTML report
open target/llvm-cov/html/index.html

# Check coverage threshold (fails if <80%)
cargo llvm-cov --all-features --fail-under-lines 80

# Coverage for specific module
cargo llvm-cov --lib --html -- unit::ffi
```

## Testing Tools

### Test Frameworks Used

- **Rust built-in test framework** - Unit and integration tests
- **Nextest** - Fast test runner with parallel execution
- **cargo-llvm-cov** - Coverage reporting
- **Trybuild** - Compile-fail tests for macros
- **Serde** - JSON testing and validation

### Helper Tools

```bash
# Install test tools
cargo install nextest
cargo install cargo-llvm-cov

# Run tests with nextest (faster)
cargo nextest run

# Watch tests
cargo watch -x test
```

## Best Practices

### Writing FFI Tests

1. **Test contracts, not implementation details**
   ```rust
   // Good: Test contract structure
   assert_eq!(contract["name"], "get_value");

   // Bad: Test internal implementation
   assert!(macro_expands_correctly());
   ```

2. **Use realistic examples**
   ```rust
   // Good: Real-world function signature
   pub fn get_profile(id: &str) -> Result<Profile, Error>

   // Bad: Artificial test case
   pub fn test_fn(x: i32) -> i32
   ```

3. **Test error cases**
   ```rust
   #[test]
   fn test_validation_empty_input() {
       let result = validate_input("");
       assert!(result.is_err());
   }
   ```

4. **Test boundary conditions**
   ```rust
   #[test]
   fn test_max_length_boundary() {
       validate_input("x".repeat(100)); // Max
       validate_input("x".repeat(101)); // Over limit
   }
   ```

5. **Use property-based testing for invariants**
   ```rust
   #[test]
   fn property_all_functions_have_contracts() {
       // Verify invariant holds for all functions
   }
   ```

### Test Organization

- Keep tests close to the code they test
- Use descriptive test names: `test_<action>_<scenario>_<expected>`
- Group related tests in modules
- Use helper functions to reduce duplication
- Document complex test setups

### Continuous Integration

- All tests must pass before merging
- Coverage must not decrease
- Breaking changes require version bump
- Run tests on multiple platforms (Linux, macOS, Windows)

## Examples

### Example 1: Testing Contract Derivation

```rust
use keyrx_core::ffi::contract::FfiContract;

#[test]
fn test_derive_contract_from_function() {
    // Given: A Rust function with #[ffi_export]
    // #[ffi_export(domain = "config", version = "1.0")]
    // pub fn get_value(key: &str) -> Result<String, Error>

    // When: Contract is derived
    let contract = load_contract("config");

    // Then: Contract matches function signature
    let func = contract.functions.iter()
        .find(|f| f.name == "get_value")
        .expect("Function not found in contract");

    assert_eq!(func.parameters.len(), 1);
    assert_eq!(func.parameters[0].name, "key");
    assert_eq!(func.parameters[0].type_, "string");
}
```

### Example 2: Testing Breaking Changes

```rust
#[test]
fn test_detect_parameter_type_change() {
    let old = json!({
        "functions": [{
            "name": "update",
            "parameters": [{"name": "count", "type": "number"}]
        }]
    });

    let new = json!({
        "functions": [{
            "name": "update",
            "parameters": [{"name": "count", "type": "string"}]
        }]
    });

    let changes = detect_breaking_changes(&old, &new);

    assert!(!changes.is_empty());
    assert!(changes[0].contains("BREAKING"));
    assert!(changes[0].contains("type"));
}
```

### Example 3: Testing Property Invariants

```rust
#[test]
fn property_function_names_are_unique() {
    let contracts = load_all_contracts();

    let mut seen = HashSet::new();
    let mut duplicates = Vec::new();

    for contract in contracts {
        for func in &contract.functions {
            let key = format!("{}::{}", contract.domain, func.name);
            if !seen.insert(key.clone()) {
                duplicates.push(key);
            }
        }
    }

    assert!(
        duplicates.is_empty(),
        "Found duplicate function names: {:?}",
        duplicates
    );
}
```

## Troubleshooting

### Tests Failing

1. **Contract mismatch errors**
   - Run `just gen-dart-bindings` to regenerate contracts
   - Check that Rust signatures match contract JSON

2. **Module not found errors**
   - Verify `mod.rs` includes the test module
   - Check file paths are correct

3. **Compilation errors**
   - Ensure all dependencies are installed
   - Run `cargo clean && cargo build`

### Coverage Not Meeting Threshold

1. **Identify untested code**
   ```bash
   cargo llvm-cov --html
   # Open HTML report and find red lines
   ```

2. **Add tests for untested paths**
   - Focus on error cases
   - Test boundary conditions
   - Add property tests

3. **Exclude non-critical code**
   ```rust
   #[cfg(not(tarpaulin_include))]
   fn debug_helper() { }
   ```

## References

- [FFI Development Guide](./FFI_DEVELOPMENT.md)
- [FFI Migration Guide](./FFI_MIGRATION_GUIDE.md)
- [Contract Schema](../contracts/schema/)
- [AI Agent Guide](./AI_AGENT_GUIDE.md)
