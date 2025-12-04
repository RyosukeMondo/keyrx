# Tasks Document

## Phase 1: Infrastructure

- [x] 1. Create Detector trait and common types
  - File: `core/src/validation/detectors/mod.rs`
  - Define `Detector` trait with name(), detect(), is_skippable()
  - Create `DetectorContext`, `DetectorResult`, `DetectorStats` types
  - Purpose: Common interface for all detectors
  - _Leverage: Rust trait patterns_
  - _Requirements: 1.4, 2.4, 3.4_
  - _Prompt: Implement the task for spec validation-decomposition, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Developer with trait design expertise | Task: Create Detector trait and supporting types in core/src/validation/detectors/mod.rs | Restrictions: Trait must be object-safe, support Send+Sync, include timing stats | _Leverage: Rust trait patterns | Success: Trait compiles, types are ergonomic, supports async if needed | After completion: Mark task [-] as in-progress before starting, use log-implementation tool to record artifacts, then mark [x] when complete_

- [x] 2. Create ValidationIssue and Severity types
  - File: `core/src/validation/common/issue.rs`
  - Define `ValidationIssue` with severity, detector, message, locations
  - Create `Severity` enum (Error, Warning, Info)
  - Add serialization support
  - Purpose: Unified issue reporting
  - _Leverage: Existing error types_
  - _Requirements: 1.3, 2.2, 3.2_
  - _Prompt: Implement the task for spec validation-decomposition, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Developer creating error types | Task: Create ValidationIssue and Severity in core/src/validation/common/issue.rs | Restrictions: Serde serializable, include source locations, clear Display impl | _Leverage: Existing validation error patterns | Success: Types compile, serialize correctly, Display is informative | After completion: Mark task [-] as in-progress before starting, use log-implementation tool to record artifacts, then mark [x] when complete_

- [x] 3. Create OperationVisitor utility
  - File: `core/src/validation/common/visitor.rs`
  - Define `OperationVisitor` trait
  - Implement `visit_all()` helper function
  - Purpose: Shared operation traversal
  - _Leverage: Visitor pattern_
  - _Requirements: 5.1_
  - _Prompt: Implement the task for spec validation-decomposition, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Developer implementing visitor pattern | Task: Create OperationVisitor in core/src/validation/common/visitor.rs | Restrictions: Handle all PendingOp variants, efficient traversal | _Leverage: Visitor pattern, PendingOp type | Success: Visitor works with all operations, reusable by detectors | After completion: Mark task [-] as in-progress before starting, use log-implementation tool to record artifacts, then mark [x] when complete_

## Phase 2: Detector Extraction

- [x] 4. Extract ConflictDetector
  - File: `core/src/validation/detectors/conflicts.rs`
  - Move remap/block conflict detection from original conflicts.rs
  - Implement Detector trait
  - Build key→operations map for O(n) detection
  - Purpose: Isolated conflict detection
  - _Leverage: Original conflicts.rs logic_
  - _Requirements: 1.1, 1.2, 1.3, 1.4_
  - _Prompt: Implement the task for spec validation-decomposition, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Developer extracting module | Task: Extract ConflictDetector from conflicts.rs into core/src/validation/detectors/conflicts.rs | Restrictions: Preserve existing behavior, implement Detector trait, O(n) complexity | _Leverage: Current conflict detection in conflicts.rs | Success: ConflictDetector works identically, < 300 LOC, tests pass | After completion: Mark task [-] as in-progress before starting, use log-implementation tool to record artifacts, then mark [x] when complete_

- [-] 5. Extract ShadowingDetector
  - File: `core/src/validation/detectors/shadowing.rs`
  - Move combo shadowing detection from original conflicts.rs
  - Implement Detector trait with is_skippable() = true
  - Purpose: Isolated shadowing detection
  - _Leverage: Original shadowing logic_
  - _Requirements: 2.1, 2.2, 2.3, 2.4_
  - _Prompt: Implement the task for spec validation-decomposition, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Developer extracting module | Task: Extract ShadowingDetector from conflicts.rs into core/src/validation/detectors/shadowing.rs | Restrictions: Preserve existing behavior, mark as skippable, handle key ordering | _Leverage: Current shadowing detection | Success: ShadowingDetector works identically, skippable flag works | After completion: Mark task [-] as in-progress before starting, use log-implementation tool to record artifacts, then mark [x] when complete_

- [ ] 6. Extract CycleDetector
  - File: `core/src/validation/detectors/cycles.rs`
  - Move circular dependency detection from original conflicts.rs
  - Implement DFS-based cycle detection
  - Report full cycle paths
  - Purpose: Isolated cycle detection
  - _Leverage: Original cycle detection, graph algorithms_
  - _Requirements: 3.1, 3.2, 3.3, 3.4_
  - _Prompt: Implement the task for spec validation-decomposition, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Developer with graph algorithm expertise | Task: Extract CycleDetector from conflicts.rs into core/src/validation/detectors/cycles.rs | Restrictions: O(V+E) complexity, report full paths, use DFS with coloring | _Leverage: Current cycle detection, Tarjan's algorithm | Success: CycleDetector finds all cycles, reports paths, efficient | After completion: Mark task [-] as in-progress before starting, use log-implementation tool to record artifacts, then mark [x] when complete_

## Phase 3: Integration

- [ ] 7. Create DetectorOrchestrator
  - File: `core/src/validation/orchestrator.rs`
  - Register and run all detectors in sequence
  - Aggregate results into ValidationReport
  - Support skipping detectors
  - Purpose: Detector coordination
  - _Leverage: Detector trait_
  - _Requirements: 5.1_
  - _Prompt: Implement the task for spec validation-decomposition, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Developer creating orchestrator | Task: Create DetectorOrchestrator in core/src/validation/orchestrator.rs | Restrictions: Run detectors in order, aggregate results, support skip flags | _Leverage: Detector trait, existing ValidationEngine | Success: Orchestrator runs all detectors, aggregates correctly | After completion: Mark task [-] as in-progress before starting, use log-implementation tool to record artifacts, then mark [x] when complete_

- [ ] 8. Update ValidationEngine to use orchestrator
  - File: `core/src/validation/engine.rs`
  - Replace inline conflict detection with orchestrator
  - Maintain backward-compatible API
  - Purpose: Integration with new architecture
  - _Leverage: DetectorOrchestrator_
  - _Requirements: Non-functional (compatibility)_
  - _Prompt: Implement the task for spec validation-decomposition, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Developer integrating components | Task: Update ValidationEngine to use DetectorOrchestrator | Restrictions: Maintain backward compatibility, same public API, same behavior | _Leverage: DetectorOrchestrator, existing engine.rs | Success: Engine uses new detectors, all tests pass, API unchanged | After completion: Mark task [-] as in-progress before starting, use log-implementation tool to record artifacts, then mark [x] when complete_

- [ ] 9. Extract test helpers
  - File: `core/src/validation/common/test_helpers.rs`
  - Create operation builders for tests
  - Add assertion helpers for issues
  - Purpose: Shared test utilities
  - _Leverage: Existing test patterns_
  - _Requirements: 4.2_
  - _Prompt: Implement the task for spec validation-decomposition, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Test Developer | Task: Create test helpers in core/src/validation/common/test_helpers.rs | Restrictions: Builder pattern for operations, assertion helpers, #[cfg(test)] | _Leverage: Existing test patterns in conflicts.rs | Success: Helpers reduce test boilerplate, clear API | After completion: Mark task [-] as in-progress before starting, use log-implementation tool to record artifacts, then mark [x] when complete_

## Phase 4: Test Migration

- [ ] 10. Move conflict tests to dedicated file
  - File: `core/src/validation/tests/conflict_tests.rs`
  - Extract ~150 conflict-related tests from conflicts.rs
  - Use test helpers
  - Purpose: Test organization
  - _Leverage: Test helpers, existing tests_
  - _Requirements: 4.1, 4.3_
  - _Prompt: Implement the task for spec validation-decomposition, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Test Developer | Task: Move conflict tests to core/src/validation/tests/conflict_tests.rs | Restrictions: All tests must pass, use new test helpers, organize by scenario | _Leverage: Existing inline tests, test_helpers | Success: All conflict tests moved, passing, well-organized | After completion: Mark task [-] as in-progress before starting, use log-implementation tool to record artifacts, then mark [x] when complete_

- [ ] 11. Move shadowing tests to dedicated file
  - File: `core/src/validation/tests/shadowing_tests.rs`
  - Extract shadowing-related tests
  - Use test helpers
  - Purpose: Test organization
  - _Leverage: Test helpers, existing tests_
  - _Requirements: 4.1, 4.3_
  - _Prompt: Implement the task for spec validation-decomposition, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Test Developer | Task: Move shadowing tests to core/src/validation/tests/shadowing_tests.rs | Restrictions: All tests must pass, use new test helpers | _Leverage: Existing inline tests | Success: All shadowing tests moved, passing | After completion: Mark task [-] as in-progress before starting, use log-implementation tool to record artifacts, then mark [x] when complete_

- [ ] 12. Move cycle tests to dedicated file
  - File: `core/src/validation/tests/cycle_tests.rs`
  - Extract cycle detection tests
  - Add edge cases for complex cycles
  - Purpose: Test organization
  - _Leverage: Test helpers, existing tests_
  - _Requirements: 4.1, 4.3_
  - _Prompt: Implement the task for spec validation-decomposition, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Test Developer | Task: Move cycle tests to core/src/validation/tests/cycle_tests.rs | Restrictions: All tests must pass, add complex cycle cases | _Leverage: Existing inline tests | Success: All cycle tests moved, passing, edge cases covered | After completion: Mark task [-] as in-progress before starting, use log-implementation tool to record artifacts, then mark [x] when complete_

## Phase 5: Cleanup

- [ ] 13. Remove original conflicts.rs
  - Files: `core/src/validation/conflicts.rs` (delete or minimize)
  - Replace with re-exports from new modules
  - Update mod.rs
  - Purpose: Complete migration
  - _Leverage: New module structure_
  - _Requirements: Non-functional (cleanup)_
  - _Prompt: Implement the task for spec validation-decomposition, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Developer completing cleanup | Task: Remove or minimize original conflicts.rs, update module structure | Restrictions: All imports must resolve, no dead code, clean module tree | _Leverage: New detector modules | Success: Old file removed, imports work, no warnings | After completion: Mark task [-] as in-progress before starting, use log-implementation tool to record artifacts, then mark [x] when complete_

- [ ] 14. Verify all existing behavior preserved
  - Files: Run full test suite, integration tests
  - Compare validation output before/after
  - Purpose: Regression verification
  - _Leverage: Existing test suite_
  - _Requirements: Non-functional (reliability)_
  - _Prompt: Implement the task for spec validation-decomposition, first run spec-workflow-guide to get the workflow guide then implement the task: Role: QA Developer | Task: Verify all validation behavior preserved through full test suite | Restrictions: No behavior changes, same outputs, same error messages | _Leverage: Full test suite, integration tests | Success: All tests pass, no regressions detected | After completion: Mark task [-] as in-progress before starting, use log-implementation tool to record artifacts, then mark [x] when complete_

- [ ] 15. Add detector documentation
  - File: `docs/validation-architecture.md`
  - Document detector pattern
  - Explain how to add new detectors
  - Purpose: Developer documentation
  - _Leverage: Implementation details_
  - _Requirements: Non-functional (usability)_
  - _Prompt: Implement the task for spec validation-decomposition, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Technical Writer | Task: Create validation architecture documentation | Restrictions: Cover detector pattern, adding new detectors, module structure | _Leverage: Implementation from previous tasks | Success: Documentation complete, examples work | After completion: Mark task [-] as in-progress before starting, use log-implementation tool to record artifacts, then mark [x] when complete_
