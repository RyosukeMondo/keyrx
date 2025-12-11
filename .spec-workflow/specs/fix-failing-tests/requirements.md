# Requirements Document

## Introduction

The KeyRx test suite currently has **2 failing tests** that block CI/CD pipeline and prevent code coverage measurement. This spec addresses the #3 priority improvement: fixing these test failures to unblock quality metrics and ensure build stability.

**Problem Statement:**
- 2 tests failing out of 2,440 total tests
- CI/CD pipeline broken (`just ci-check` fails)
- Cannot measure code coverage with `cargo llvm-cov`
- Blocks confident PR merges
- Prevents quality improvements
- Failing tests:
  1. `ffi::domains::device_registry::tests::test_c_api_null_label_clears` (line 571)
  2. `scripting::docs::test_example::tests::test_macro_generates_doc` (line 46)

**Value Proposition:**
- **Unblocks CI/CD pipeline** - enables automated quality checks
- **Enables coverage measurement** - can verify 80%/90% targets
- **Confident merges** - tests validate all changes
- **Fast fix** - 1-2 hours estimated
- **High impact** - unblocks all other quality improvements

## Alignment with Product Vision

This feature supports the core development principles outlined in `~/.claude/CLAUDE.md`:

- **Code Quality Enforcement**: Pre-commit hooks require passing tests
- **80% test coverage minimum**: Can't measure without passing test suite
- **Development velocity**: Broken tests slow down all development

## Requirements

### Requirement 1: Fix FFI Device Registry Test

**User Story:** As a developer, I want the FFI device registry test to pass, so that FFI layer functionality is validated.

**Test Location:** `core/src/ffi/domains/device_registry.rs:571`
**Test Name:** `test_c_api_null_label_clears`
**Error:** `assertion failed: msg.starts_with("ok:")`

#### Acceptance Criteria

1. WHEN test is run THEN it SHALL pass without assertion failures
2. WHEN null label is passed to C API THEN it SHALL clear the label correctly
3. WHEN response is returned THEN it SHALL have correct format (starts with "ok:")
4. IF response format changed THEN test SHALL be updated to match new format
5. WHEN fix is implemented THEN it SHALL not break other device registry tests

**Possible Root Causes:**
- Response format changed but test not updated
- Null handling changed in implementation
- Serialization format changed
- Test expectation incorrect

**Success Metrics:**
- Test passes consistently (not flaky)
- Other device registry tests still pass
- FFI C API null handling validated

### Requirement 2: Fix Scripting Documentation Test

**User Story:** As a developer, I want the scripting documentation test to pass, so that doc generation is validated.

**Test Location:** `core/src/scripting/docs/test_example.rs:46`
**Test Name:** `test_macro_generates_doc`
**Error:** `Documentation should be registered`

#### Acceptance Criteria

1. WHEN test is run THEN it SHALL pass without assertion failures
2. WHEN `#[rhai_doc]` macro is used THEN documentation SHALL be registered in registry
3. WHEN registry is queried THEN documentation SHALL be found
4. IF registry initialization changed THEN test setup SHALL be updated
5. WHEN fix is implemented THEN it SHALL not break other doc tests

**Possible Root Causes:**
- Doc registry not initialized in test
- Macro expansion changed
- Registration logic changed
- Test setup incomplete

**Success Metrics:**
- Test passes consistently
- Other documentation tests still pass
- Doc registration validated

### Requirement 3: Verify No Regressions

**User Story:** As a QA engineer, I want comprehensive testing after fixes, so that I can verify no regressions were introduced.

#### Acceptance Criteria

1. WHEN fixes are complete THEN full test suite SHALL pass (2,440 tests)
2. WHEN `cargo test --lib` is run THEN all library tests SHALL pass
3. WHEN `cargo test --all` is run THEN all tests SHALL pass
4. IF other tests fail THEN they SHALL be investigated and fixed
5. WHEN tests pass THEN coverage measurement SHALL be enabled

**Testing Checklist:**
- [ ] `cargo test --lib` passes
- [ ] `cargo test --all` passes
- [ ] `cargo clippy` passes
- [ ] `cargo build --release` succeeds
- [ ] `just ci-check` passes

**Success Metrics:**
- 100% test pass rate (2,440/2,440)
- Zero test failures
- CI pipeline green
- Coverage measurement works

### Requirement 4: Enable Code Coverage Measurement

**User Story:** As a project maintainer, I want to measure code coverage after tests pass, so that I can verify compliance with 80%/90% targets.

#### Acceptance Criteria

1. WHEN tests pass THEN `cargo llvm-cov` SHALL run successfully
2. WHEN coverage is measured THEN overall coverage percentage SHALL be reported
3. WHEN coverage report is generated THEN it SHALL identify gaps
4. IF coverage is below 80% THEN gaps SHALL be documented
5. WHEN critical paths are measured THEN coverage SHALL be checked against 90% target

**Coverage Targets:**
- **Overall:** ≥80% line coverage
- **Critical paths:** ≥90% line coverage (services, API, engine, FFI)
- **New code:** 100% coverage for new implementations

**Success Metrics:**
- Coverage measurement runs successfully
- Overall coverage percentage known
- Critical path coverage verified
- Gaps identified for future work

## Non-Functional Requirements

### Reliability

- **Test Stability**: Fixed tests SHALL pass consistently (not flaky)
- **Deterministic**: Test failures SHALL be reproducible
- **Isolated**: Tests SHALL not depend on external state
- **Fast**: Test fixes SHALL not slow down test suite

### Maintainability

- **Clear Fixes**: Root cause SHALL be documented
- **Prevent Recurrence**: Fix SHALL prevent similar issues
- **Understandable**: Code changes SHALL be clear and well-commented
- **Minimal**: Fixes SHALL be minimal and focused

### Performance

- **No Slowdown**: Fixes SHALL not increase test runtime
- **Quick Execution**: Fixed tests SHALL complete in <100ms
- **CI Impact**: Total test suite time SHALL remain <3s

### Documentation

- **Root Cause**: Document why test failed
- **Fix Explanation**: Explain what changed
- **Prevention**: Note how to prevent similar issues
- **Test Purpose**: Clarify what test validates
