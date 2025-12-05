# KeyRx Test Organization

This directory contains all tests for the KeyRx core library, organized by test type and domain to support fast, parallel test execution and easy navigation.

## Directory Structure

```
core/tests/
├── fixtures/           # Shared test utilities and builders
├── unit/               # Fast, isolated unit tests
├── integration/        # Cross-module integration tests
└── e2e/                # End-to-end workflow tests
```

## Test Categories

### Unit Tests (`unit/`)

Fast, isolated tests that verify individual components without I/O or external dependencies.

**Location:** `core/tests/unit/<module>/`

**Characteristics:**
- No I/O operations
- No network calls
- Minimal dependencies
- Fast execution (< 100ms per test)
- Can run in parallel

**Structure:**
```
unit/
├── engine/           # Engine component tests
├── validation/       # Validation logic tests
├── scripting/        # Scripting engine tests
├── metrics/          # Metrics collection tests
└── observability/    # Logging and tracing tests
```

**When to add unit tests:**
- Testing pure functions
- Testing single component behavior
- Testing error conditions
- Testing edge cases

### Integration Tests (`integration/`)

Tests that verify interactions between multiple components, may involve I/O.

**Location:** `core/tests/integration/<domain>/`

**Characteristics:**
- Cross-module interactions
- May involve file I/O
- May involve subprocess spawning
- Moderate execution time (< 1s per test)
- Most can run in parallel

**Structure:**
```
integration/
├── drivers/          # Driver integration tests
├── phases/           # Phase processing tests
└── validation/       # Validation engine integration tests
```

**When to add integration tests:**
- Testing CLI commands
- Testing FFI interfaces
- Testing complete processing flows
- Testing driver integrations

### E2E Tests (`e2e/`)

Full system tests that verify complete user workflows and scenarios.

**Location:** `core/tests/e2e/`

**Characteristics:**
- Complete user workflows
- Minimal mocking
- May require hardware access
- Slower execution (> 1s per test)
- May need sequential execution

**When to add e2e tests:**
- Testing critical user scenarios
- Testing full input-to-output flows
- Testing real device interactions
- Testing complex state transitions

### Test Fixtures (`fixtures/`)

Shared utilities, builders, and test data to reduce boilerplate across tests.

**Available fixtures:**

#### `operations.rs` - Operation Builders
```rust
use crate::fixtures::operations::OperationBuilder;

let ops = OperationBuilder::new()
    .remap(KeyCode::A, KeyCode::B)
    .block(KeyCode::C)
    .build();
```

#### `scripts.rs` - Script Snippets
```rust
use crate::fixtures::scripts::{minimal_script, layer_script};

let script = layer_script();
```

#### `engine.rs` - Test Engine Helper
```rust
use crate::fixtures::engine::TestEngine;

let mut engine = TestEngine::new().with_script(script);
let result = engine.process(event);
```

**When to add fixtures:**
- Common test setup patterns used 3+ times
- Complex object builders
- Shared test data
- Mock implementations

## Naming Conventions

### File Names

**Pattern:** `<component>_<scenario>_tests.rs`

**Examples:**
- `conflict_integration_tests.rs` - Conflict detection integration tests
- `layer_actions_test.rs` - Layer action unit tests
- `cli_integration_tests.rs` - CLI integration tests

**Guidelines:**
- Use snake_case
- End with `_tests.rs` or `_test.rs`
- Be descriptive but concise
- Group related tests in one file
- Keep files under 500 lines

### Test Function Names

**Pattern:** `test_<what>_<scenario>`

**Examples:**
```rust
#[test]
fn test_validates_simple_remap_script() { ... }

#[test]
fn test_detects_cycle_in_layer_definitions() { ... }

#[test]
fn test_cli_check_with_invalid_script_fails() { ... }
```

**Guidelines:**
- Start with `test_` prefix (Rust convention)
- Use snake_case
- Describe what is tested and expected outcome
- Be specific about the scenario

### Module Names

**Pattern:** Match the component or domain being tested

**Examples:**
```rust
// In unit/engine/mod.rs
pub mod decision;
pub mod layer_actions_test;
pub mod state_transitions_test;

// In integration/validation/mod.rs
pub mod basic_validation_tests;
pub mod conflict_integration_tests;
pub mod safety_tests;
```

## Where to Add New Tests

### Decision Flow

1. **Is it testing a single function/method in isolation?**
   → Add to `unit/<module>/<component>_test.rs`

2. **Is it testing interaction between 2+ modules?**
   → Add to `integration/<domain>/<scenario>_tests.rs`

3. **Is it testing a complete user workflow?**
   → Add to `e2e/<workflow>_tests.rs`

4. **Is it testing validation logic?**
   - Single validation rule → `unit/validation/`
   - Complete validation flow → `integration/validation/`

5. **Is it testing engine behavior?**
   - State transitions/decisions → `unit/engine/`
   - Full processing pipeline → `integration/phases/`

### Examples

**Example 1: Testing a new validation rule**
```
Component: validation::rules::no_self_remap
Location: core/tests/unit/validation/rules_tests.rs
```

**Example 2: Testing CLI simulate command**
```
Component: CLI simulate with device selection
Location: core/tests/integration/cli/simulate_tests.rs
```

**Example 3: Testing a combo key workflow**
```
Component: Full combo key press and release flow
Location: core/tests/e2e/combo_workflow_tests.rs
```

## Running Tests

### Run All Tests
```bash
cargo test
```

### Run Specific Category
```bash
# Unit tests only
cargo test --lib

# Integration tests only
cargo test --test '*' integration

# E2E tests only
cargo test --test '*' e2e

# Specific module
cargo test --test '*' validation
```

### Run with Coverage
```bash
cargo llvm-cov --html
```

### Run in Parallel
```bash
# Default: parallel execution
cargo test

# Force parallel with specific thread count
cargo test -- --test-threads=4

# Force sequential (for debugging)
cargo test -- --test-threads=1
```

### Run Specific Test
```bash
cargo test test_validates_simple_remap_script
```

## Code Quality Standards

### File Size Limits
- Maximum 500 lines per test file
- Split larger files into focused test modules
- Use fixtures to reduce boilerplate

### Test Coverage
- Minimum 80% coverage overall
- 90% coverage for critical paths
- Coverage is measured and enforced in CI

### Test Structure
```rust
#[test]
fn test_descriptive_name() {
    // Arrange: Set up test data
    let input = setup_test_input();

    // Act: Execute the code under test
    let result = function_under_test(input);

    // Assert: Verify expected outcomes
    assert_eq!(result.status, ExpectedStatus);
    assert!(result.is_valid);
}
```

### Common Patterns

#### Testing with Fixtures
```rust
use crate::fixtures::operations::OperationBuilder;
use crate::fixtures::scripts::minimal_script;

#[test]
fn test_with_fixtures() {
    let ops = OperationBuilder::new()
        .remap(KeyCode::A, KeyCode::B)
        .build();

    let script = minimal_script();
    // ... test logic
}
```

#### Testing Error Conditions
```rust
#[test]
fn test_error_on_invalid_input() {
    let result = validate_script("invalid syntax");

    assert!(!result.is_valid);
    assert!(!result.errors.is_empty());
    assert!(result.errors[0].message.contains("syntax"));
}
```

#### Testing with Proptest
```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_property_holds(key_code in 0u16..65535u16) {
        let result = process_key_code(key_code);
        prop_assert!(result.is_valid());
    }
}
```

## CI Integration

Tests are organized to support efficient CI execution:

### Fast Feedback
- Unit tests run first (fastest)
- Integration tests run in parallel
- E2E tests run last or on-demand

### Coverage Enforcement
- Coverage thresholds enforced per category
- Coverage reports generated automatically
- Failed coverage fails the build

### Parallel Execution
- Most tests can run in parallel
- Use `#[serial]` attribute for tests that must run sequentially
- Tests are grouped for optimal parallelization

## Best Practices

### DO
✅ Use descriptive test names that explain the scenario
✅ Use fixtures for common setup patterns
✅ Keep test files focused and under 500 lines
✅ Test both success and error paths
✅ Use arrange-act-assert pattern
✅ Add comments for complex test scenarios
✅ Group related tests in the same file

### DON'T
❌ Put integration tests in unit test directories
❌ Create duplicate test fixtures
❌ Write tests that depend on execution order
❌ Use hardcoded paths (use relative paths or fixtures)
❌ Skip cleanup in tests with side effects
❌ Add tests that require specific hardware without `#[ignore]`
❌ Let test files grow beyond 500 lines

## Troubleshooting

### Test Discovery Issues
If tests aren't being discovered:
- Ensure file ends with `_test.rs` or `_tests.rs`
- Verify module is declared in parent `mod.rs`
- Check for `#[cfg(test)]` in the right place

### Parallel Test Failures
If tests fail when run in parallel:
- Use `#[serial]` attribute from `serial_test` crate
- Ensure tests don't share mutable global state
- Use unique file paths for tests that create files

### Coverage Gaps
If coverage is lower than expected:
- Check that test files are in the correct location
- Verify tests are actually running (not `#[ignore]`)
- Look for untested error paths

## Contributing

When adding new tests:

1. **Choose the right location** using the decision flow above
2. **Use existing fixtures** where possible, create new ones if needed
3. **Follow naming conventions** for files, modules, and functions
4. **Keep files focused** - split if approaching 500 lines
5. **Verify tests pass** both individually and with the full suite
6. **Update this README** if adding new test patterns or fixtures

## References

- [Rust Testing Documentation](https://doc.rust-lang.org/book/ch11-00-testing.html)
- [cargo-llvm-cov](https://github.com/taiki-e/cargo-llvm-cov) - Coverage tool
- [proptest](https://github.com/proptest-rs/proptest) - Property-based testing
- [serial_test](https://github.com/palfrey/serial_test) - Sequential test execution
