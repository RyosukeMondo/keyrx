# Tasks Document

## Phase 1: Audit

- [x] 1. Catalog all state types
  - Files: Entire codebase
  - Document each state type, location, purpose
  - Identify overlaps and duplicates
  - Purpose: Complete inventory
  - _Leverage: Code search_
  - _Requirements: 1.1, 1.2_
  - _Prompt: Implement the task for spec state-machine-audit, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Developer auditing code | Task: Catalog all state types in codebase | Restrictions: Document location, purpose, overlaps | _Leverage: Code search | Success: Complete state inventory | After completion: Mark task [-] as in-progress before starting, use log-implementation tool to record artifacts, then mark [x] when complete_

- [x] 2. Document state ownership
  - File: `docs/state-audit.md`
  - Map state types to owning components
  - Document lifecycle of each state
  - Purpose: Clear ownership
  - _Leverage: Audit results_
  - _Requirements: 1.3_
  - _Prompt: Implement the task for spec state-machine-audit, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Technical Writer | Task: Document state ownership and lifecycle | Restrictions: Clear ownership boundaries, lifecycle docs | _Leverage: Audit | Success: Ownership is clear | After completion: Mark task [-] as in-progress before starting, use log-implementation tool to record artifacts, then mark [x] when complete_

- [x] 3. Create consolidation plan
  - File: `docs/state-audit.md`
  - Plan for merging duplicates
  - Plan for extracting common patterns
  - Purpose: Roadmap for cleanup
  - _Leverage: Audit results_
  - _Requirements: 1.4_
  - _Prompt: Implement the task for spec state-machine-audit, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Architect | Task: Create state consolidation plan | Restrictions: Minimize breaking changes, clear migration | _Leverage: Audit | Success: Clear plan for consolidation | After completion: Mark task [-] as in-progress before starting, use log-implementation tool to record artifacts, then mark [x] when complete_

## Phase 2: Core Types

- [x] 4. Create StateTransition enum
  - File: `core/src/engine/transitions/transition.rs`
  - Define all valid transitions
  - Add metadata (timestamp, category)
  - Purpose: Explicit transitions
  - _Leverage: Audit results_
  - _Requirements: 2.1, 2.2_
  - _Prompt: Implement the task for spec state-machine-audit, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Developer creating types | Task: Create StateTransition enum with all variants | Restrictions: Cover all transitions, serializable | _Leverage: Audit | Success: All transitions enumerated | After completion: Mark task [-] as in-progress before starting, use log-implementation tool to record artifacts, then mark [x] when complete_

- [x] 5. Create StateKind enum
  - File: `core/src/engine/transitions/state_kind.rs`
  - Define high-level state categories
  - Map to transition validity
  - Purpose: State categorization
  - _Leverage: Audit results_
  - _Requirements: 2.1_
  - _Prompt: Implement the task for spec state-machine-audit, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Developer creating types | Task: Create StateKind enum for state categories | Restrictions: Cover all states, clear semantics | _Leverage: Audit | Success: States categorized | After completion: Mark task [-] as in-progress before starting, use log-implementation tool to record artifacts, then mark [x] when complete_

- [x] 6. Create StateGraph
  - File: `core/src/engine/transitions/graph.rs`
  - Define valid transition rules
  - Implement is_valid and apply
  - Purpose: Transition enforcement
  - _Leverage: StateTransition, StateKind_
  - _Requirements: 2.2, 2.3, 2.4_
  - _Prompt: Implement the task for spec state-machine-audit, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Developer creating state machine | Task: Create StateGraph with transition rules | Restrictions: Enforce validity, reject invalid | _Leverage: StateTransition | Success: Invalid transitions rejected | After completion: Mark task [-] as in-progress before starting, use log-implementation tool to record artifacts, then mark [x] when complete_

## Phase 3: Validation

- [x] 7. Create Invariant trait
  - File: `core/src/engine/transitions/invariant.rs`
  - Define invariant interface
  - Add InvariantViolation type
  - Purpose: Pluggable validation
  - _Leverage: Trait patterns_
  - _Requirements: 3.1, 3.2_
  - _Prompt: Implement the task for spec state-machine-audit, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Developer creating traits | Task: Create Invariant trait for validation | Restrictions: Pluggable, clear errors | _Leverage: Traits | Success: Invariant interface defined | After completion: Mark task [-] as in-progress before starting, use log-implementation tool to record artifacts, then mark [x] when complete_

- [x] 8. Implement core invariants
  - File: `core/src/engine/transitions/invariants/`
  - NoOrphanedModifiers, LayerStackNotEmpty, etc.
  - Purpose: Common validations
  - _Leverage: Invariant trait_
  - _Requirements: 3.1_
  - _Prompt: Implement the task for spec state-machine-audit, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Developer implementing invariants | Task: Implement core state invariants | Restrictions: Cover all critical invariants | _Leverage: Invariant trait | Success: Key invariants enforced | After completion: Mark task [-] as in-progress before starting, use log-implementation tool to record artifacts, then mark [x] when complete_

- [x] 9. Create StateValidator
  - File: `core/src/engine/transitions/validator.rs`
  - Combine invariants
  - Add debug-only extra validation
  - Purpose: Comprehensive validation
  - _Leverage: Invariant implementations_
  - _Requirements: 3.1, 3.3, 3.4_
  - _Prompt: Implement the task for spec state-machine-audit, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Developer creating validator | Task: Create StateValidator combining invariants | Restrictions: Debug extra checks, clear errors | _Leverage: Invariants | Success: All invariants checked | After completion: Mark task [-] as in-progress before starting, use log-implementation tool to record artifacts, then mark [x] when complete_

## Phase 4: Logging

- [x] 10. Create TransitionEntry type
  - File: `core/src/engine/transitions/log.rs`
  - State before/after, transition, timing
  - Serializable for export
  - Purpose: Log entry structure
  - _Leverage: serde_
  - _Requirements: 4.1, 4.2_
  - _Prompt: Implement the task for spec state-machine-audit, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Developer creating types | Task: Create TransitionEntry for logging | Restrictions: Complete state capture, serializable | _Leverage: serde | Success: Entries capture all info | After completion: Mark task [-] as in-progress before starting, use log-implementation tool to record artifacts, then mark [x] when complete_

- [x] 11. Create TransitionLog
  - File: `core/src/engine/transitions/log.rs`
  - Ring buffer implementation
  - Search and export
  - Purpose: Transition history
  - _Leverage: Ring buffer patterns_
  - _Requirements: 4.1, 4.2, 4.3_
  - _Prompt: Implement the task for spec state-machine-audit, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Developer creating logger | Task: Create TransitionLog with ring buffer | Restrictions: Bounded memory, searchable, exportable | _Leverage: Ring buffer | Success: History tracked efficiently | After completion: Mark task [-] as in-progress before starting, use log-implementation tool to record artifacts, then mark [x] when complete_

- [x] 12. Add zero-cost disable
  - File: `core/src/engine/transitions/log.rs`
  - Feature flag for logging
  - Compile-time removal when disabled
  - Purpose: Zero overhead option
  - _Leverage: Feature flags_
  - _Requirements: 4.4_
  - _Prompt: Implement the task for spec state-machine-audit, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Developer optimizing | Task: Add feature flag for zero-cost log disable | Restrictions: Compile-time removal, zero overhead | _Leverage: Feature flags | Success: No overhead when disabled | After completion: Mark task [-] as in-progress before starting, use log-implementation tool to record artifacts, then mark [x] when complete_

## Phase 5: Integration

- [x] 13. Integrate StateGraph into Engine
  - Files: `core/src/engine/mod.rs`
  - Route all changes through graph
  - Add validation on transitions
  - Purpose: Engine integration
  - _Leverage: StateGraph_
  - _Requirements: 2.4, 3.1_
  - _Prompt: Implement the task for spec state-machine-audit, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Developer integrating | Task: Integrate StateGraph into Engine | Restrictions: All changes through graph, validate | _Leverage: StateGraph | Success: Engine uses state graph | After completion: Mark task [-] as in-progress before starting, use log-implementation tool to record artifacts, then mark [x] when complete_

- [x] 14. Add transition logging
  - Files: `core/src/engine/mod.rs`
  - Log all transitions when enabled
  - Add FFI export for log
  - Purpose: Debugging support
  - _Leverage: TransitionLog_
  - _Requirements: 4.1, 4.3_
  - _Prompt: Implement the task for spec state-machine-audit, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Developer adding logging | Task: Add transition logging to engine | Restrictions: Configurable, FFI export | _Leverage: TransitionLog | Success: Transitions logged for debugging | After completion: Mark task [-] as in-progress before starting, use log-implementation tool to record artifacts, then mark [x] when complete_

## Phase 6: Consolidation

- [x] 15. Merge duplicate EngineState definitions
  - Files: `core/src/engine/advanced.rs`, `core/src/engine/mod.rs`
  - Create single canonical definition
  - Update all references
  - Purpose: Remove duplication
  - _Leverage: Consolidation plan_
  - _Requirements: 1.4_
  - _Prompt: Implement the task for spec state-machine-audit, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Developer consolidating | Task: Merge duplicate EngineState definitions | Restrictions: Single definition, update all refs | _Leverage: Plan | Success: One EngineState definition | After completion: Mark task [-] as in-progress before starting, use log-implementation tool to record artifacts, then mark [x] when complete_

- [x] 16. Extract common SessionState
  - Files: `core/src/engine/recording.rs`, `core/src/engine/replay.rs`
  - Create shared SessionState base
  - Compose into Recording/ReplayState
  - Purpose: Remove duplication
  - _Leverage: Consolidation plan_
  - _Requirements: 1.4_
  - _Prompt: Implement the task for spec state-machine-audit, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Developer consolidating | Task: Extract common SessionState for recording/replay | Restrictions: Composition over inheritance | _Leverage: Plan | Success: Shared session state | After completion: Mark task [-] as in-progress before starting, use log-implementation tool to record artifacts, then mark [x] when complete_

- [x] 17. Add state transition tests
  - File: `core/tests/unit/engine/state_transitions_test.rs`
  - Test all valid transitions
  - Test invalid transition rejection
  - Purpose: Verify state machine
  - _Leverage: StateGraph_
  - _Requirements: 2.3_
  - _Prompt: Implement the task for spec state-machine-audit, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Test Developer | Task: Create state transition tests | Restrictions: All transitions, valid and invalid | _Leverage: StateGraph | Success: All transitions tested | After completion: Mark task [-] as in-progress before starting, use log-implementation tool to record artifacts, then mark [x] when complete_
