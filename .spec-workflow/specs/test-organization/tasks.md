# Tasks Document

## Phase 1: Infrastructure

- [x] 1. Create test directory structure
  - Files: `core/tests/{fixtures,unit,integration,e2e}/mod.rs`
  - Create directory hierarchy as designed
  - Add mod.rs files with documentation
  - Purpose: Foundation for test organization
  - _Leverage: Rust test conventions_
  - _Requirements: 2.1, 2.2, 2.4_
  - _Prompt: Implement the task for spec test-organization, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Developer setting up test infrastructure | Task: Create test directory structure in core/tests/ with unit/integration/e2e/fixtures subdirectories | Restrictions: Follow Rust conventions, add mod.rs with docs | _Leverage: Rust test organization patterns | Success: Directory structure created, mod.rs files present, cargo test discovers structure | After completion: Mark task [-] as in-progress before starting, use log-implementation tool to record artifacts, then mark [x] when complete_

- [x] 2. Create OperationBuilder fixture
  - File: `core/tests/fixtures/operations.rs`
  - Builder for creating PendingOp instances
  - Methods: remap(), block(), combo(), pass(), build()
  - Purpose: Reduce test setup boilerplate
  - _Leverage: Builder pattern_
  - _Requirements: 4.1, 4.3_
  - _Prompt: Implement the task for spec test-organization, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Developer creating test utilities | Task: Create OperationBuilder in core/tests/fixtures/operations.rs | Restrictions: Builder pattern, chainable methods, clear API | _Leverage: PendingOp types | Success: Builder works for all operation types, reduces test LOC | After completion: Mark task [-] as in-progress before starting, use log-implementation tool to record artifacts, then mark [x] when complete_

- [x] 3. Create script fixtures
  - File: `core/tests/fixtures/scripts.rs`
  - Common Rhai script snippets for testing
  - Functions: minimal_script(), layer_script(), error_script()
  - Purpose: Consistent test scripts
  - _Leverage: Existing test scripts_
  - _Requirements: 4.1_
  - _Prompt: Implement the task for spec test-organization, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Developer creating test utilities | Task: Create script fixtures in core/tests/fixtures/scripts.rs | Restrictions: Cover common scenarios, valid and invalid scripts | _Leverage: Existing test scripts | Success: Fixtures cover testing needs, scripts are valid Rhai | After completion: Mark task [-] as in-progress before starting, use log-implementation tool to record artifacts, then mark [x] when complete_

- [x] 4. Create TestEngine fixture
  - File: `core/tests/fixtures/engine.rs`
  - Wrapper for testing engine with mock dependencies
  - Methods: new(), with_script(), process()
  - Purpose: Simplify engine testing
  - _Leverage: Existing mock patterns_
  - _Requirements: 4.1, 4.2_
  - _Prompt: Implement the task for spec test-organization, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Developer creating test utilities | Task: Create TestEngine fixture in core/tests/fixtures/engine.rs | Restrictions: Use mock dependencies, easy setup, proper cleanup | _Leverage: Existing mock patterns | Success: TestEngine simplifies engine tests, handles cleanup | After completion: Mark task [-] as in-progress before starting, use log-implementation tool to record artifacts, then mark [x] when complete_

## Phase 2: Split Validation Tests

- [x] 5. Split validation_integration.rs - conflict tests
  - File: `core/tests/integration/validation/conflict_integration_tests.rs`
  - Extract conflict-related tests from 27K file
  - Use fixtures for setup
  - Purpose: First validation test split
  - _Leverage: OperationBuilder, existing tests_
  - _Requirements: 1.1, 1.2_
  - _Prompt: Implement the task for spec test-organization, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Developer splitting tests | Task: Extract conflict tests from validation_integration.rs to conflict_integration_tests.rs | Restrictions: Keep tests passing, use fixtures, < 500 LOC | _Leverage: OperationBuilder, existing tests | Success: Tests pass, file < 500 LOC, coverage maintained | After completion: Mark task [-] as in-progress before starting, use log-implementation tool to record artifacts, then mark [x] when complete_

- [x] 6. Split validation_integration.rs - cycle tests
  - File: `core/tests/integration/validation/cycle_integration_tests.rs`
  - Extract cycle detection tests
  - Purpose: Continue validation split
  - _Leverage: Fixtures, existing tests_
  - _Requirements: 1.1_
  - _Prompt: Implement the task for spec test-organization, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Developer splitting tests | Task: Extract cycle tests from validation_integration.rs | Restrictions: Keep tests passing, < 500 LOC | _Leverage: Fixtures, existing tests | Success: Tests pass, organized by cycle scenarios | After completion: Mark task [-] as in-progress before starting, use log-implementation tool to record artifacts, then mark [x] when complete_

- [ ] 7. Split validation_integration.rs - remaining categories
  - Files: `core/tests/integration/validation/{shadowing,safety,coverage,edge_case}_tests.rs`
  - Extract remaining test categories
  - Delete or minimize original file
  - Purpose: Complete validation split
  - _Leverage: Fixtures, pattern from previous tasks_
  - _Requirements: 1.1, 1.4_
  - _Prompt: Implement the task for spec test-organization, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Developer completing test split | Task: Extract remaining tests from validation_integration.rs | Restrictions: All tests passing, original file removed, < 500 LOC each | _Leverage: Pattern from previous splits | Success: All validation tests organized, original file gone | After completion: Mark task [-] as in-progress before starting, use log-implementation tool to record artifacts, then mark [x] when complete_

## Phase 3: Split Phase Tests

- [ ] 8. Split phase_1_3_integration_test.rs - Phase 1 tests
  - Files: `core/tests/integration/phases/phase1_{basic_remap,block}_tests.rs`
  - Extract Phase 1 functionality tests
  - Purpose: First phase test split
  - _Leverage: Fixtures, existing tests_
  - _Requirements: 1.1_
  - _Prompt: Implement the task for spec test-organization, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Developer splitting tests | Task: Extract Phase 1 tests from phase_1_3_integration_test.rs | Restrictions: Keep tests passing, < 600 LOC per file | _Leverage: Fixtures, existing tests | Success: Phase 1 tests organized, passing | After completion: Mark task [-] as in-progress before starting, use log-implementation tool to record artifacts, then mark [x] when complete_

- [ ] 9. Split phase_1_3_integration_test.rs - Phase 2 tests
  - Files: `core/tests/integration/phases/phase2_{layer,modifier}_tests.rs`
  - Extract Phase 2 functionality tests
  - Purpose: Continue phase split
  - _Leverage: Fixtures, pattern from previous_
  - _Requirements: 1.1_
  - _Prompt: Implement the task for spec test-organization, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Developer splitting tests | Task: Extract Phase 2 tests from phase_1_3_integration_test.rs | Restrictions: Keep tests passing, < 700 LOC per file | _Leverage: Pattern from Phase 1 split | Success: Phase 2 tests organized, passing | After completion: Mark task [-] as in-progress before starting, use log-implementation tool to record artifacts, then mark [x] when complete_

- [ ] 10. Split phase_1_3_integration_test.rs - Phase 3 and remaining
  - Files: `core/tests/integration/phases/phase3_{combo,timing}_tests.rs`, `regression_tests.rs`
  - Extract Phase 3 and cleanup original
  - Purpose: Complete phase split
  - _Leverage: Fixtures, established pattern_
  - _Requirements: 1.1_
  - _Prompt: Implement the task for spec test-organization, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Developer completing test split | Task: Extract Phase 3 and remaining tests, remove original file | Restrictions: All tests passing, original file removed | _Leverage: Established pattern | Success: All phase tests organized, original file gone | After completion: Mark task [-] as in-progress before starting, use log-implementation tool to record artifacts, then mark [x] when complete_

## Phase 4: Inline Test Extraction

- [ ] 11. Extract inline tests from validation modules
  - Files: Move tests from `validation/*.rs` to `tests/unit/validation/`
  - Add #[cfg(test)] helpers as needed
  - Purpose: Clean source files
  - _Leverage: Test helpers_
  - _Requirements: 3.1, 3.2, 3.3_
  - _Prompt: Implement the task for spec test-organization, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Developer extracting tests | Task: Move inline tests from validation modules to tests/unit/validation/ | Restrictions: Expose helpers if needed, maintain coverage, keep < 100 LOC inline | _Leverage: cfg(test) patterns | Success: Source files cleaner, tests still pass | After completion: Mark task [-] as in-progress before starting, use log-implementation tool to record artifacts, then mark [x] when complete_

- [ ] 12. Extract inline tests from engine modules
  - Files: Move tests from `engine/*.rs` to `tests/unit/engine/`
  - Preserve internal behavior tests inline
  - Purpose: Clean source files
  - _Leverage: Test helpers_
  - _Requirements: 3.1, 3.4_
  - _Prompt: Implement the task for spec test-organization, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Developer extracting tests | Task: Move inline tests from engine modules to tests/unit/engine/ | Restrictions: Keep truly internal tests inline, extract module behavior tests | _Leverage: cfg(test) patterns | Success: Engine source cleaner, tests organized | After completion: Mark task [-] as in-progress before starting, use log-implementation tool to record artifacts, then mark [x] when complete_

- [ ] 13. Extract inline tests from scripting modules
  - Files: Move tests from `scripting/*.rs` to `tests/unit/scripting/`
  - Purpose: Clean source files
  - _Leverage: Test helpers_
  - _Requirements: 3.1_
  - _Prompt: Implement the task for spec test-organization, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Developer extracting tests | Task: Move inline tests from scripting modules to tests/unit/scripting/ | Restrictions: Maintain coverage, extract binding tests | _Leverage: cfg(test) patterns | Success: Scripting source cleaner, tests organized | After completion: Mark task [-] as in-progress before starting, use log-implementation tool to record artifacts, then mark [x] when complete_

## Phase 5: Documentation & CI

- [ ] 14. Create test README documentation
  - File: `core/tests/README.md`
  - Document test organization strategy
  - Explain where to add new tests
  - List naming conventions
  - Purpose: Developer guidance
  - _Leverage: Implementation details_
  - _Requirements: 5.1, 5.2, 5.4_
  - _Prompt: Implement the task for spec test-organization, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Technical Writer | Task: Create test README in core/tests/README.md | Restrictions: Cover organization, naming, where to add tests | _Leverage: New test structure | Success: README is helpful, developers can find right location | After completion: Mark task [-] as in-progress before starting, use log-implementation tool to record artifacts, then mark [x] when complete_

- [ ] 15. Update CI to run test categories
  - File: `.github/workflows/ci.yml`
  - Add separate jobs for unit/integration/e2e
  - Configure parallel execution
  - Purpose: CI optimization
  - _Leverage: GitHub Actions, cargo nextest_
  - _Requirements: Non-functional (performance)_
  - _Prompt: Implement the task for spec test-organization, first run spec-workflow-guide to get the workflow guide then implement the task: Role: DevOps Engineer | Task: Update CI workflow to run test categories separately | Restrictions: Unit tests fast, integration can be slower, e2e optional | _Leverage: GitHub Actions, cargo nextest | Success: CI runs tests efficiently, clear feedback per category | After completion: Mark task [-] as in-progress before starting, use log-implementation tool to record artifacts, then mark [x] when complete_

- [ ] 16. Verify coverage maintained
  - Run coverage before and after reorganization
  - Ensure no coverage regression
  - Add coverage thresholds to CI
  - Purpose: Quality assurance
  - _Leverage: cargo-llvm-cov_
  - _Requirements: Non-functional (reliability)_
  - _Prompt: Implement the task for spec test-organization, first run spec-workflow-guide to get the workflow guide then implement the task: Role: QA Developer | Task: Verify test coverage maintained after reorganization | Restrictions: No coverage regression, add CI thresholds | _Leverage: cargo-llvm-cov | Success: Coverage same or better, CI enforces threshold | After completion: Mark task [-] as in-progress before starting, use log-implementation tool to record artifacts, then mark [x] when complete_
