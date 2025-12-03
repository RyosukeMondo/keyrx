# Tasks Document

## Phase 1: Core Types

- [x] 1. Create Mutation enum
  - File: `core/src/engine/state/mutation.rs`
  - Define all mutation variants (KeyDown, KeyUp, PushLayer, etc.)
  - Add Debug, Clone derives
  - Purpose: Explicit state change operations
  - _Leverage: Existing state change patterns_
  - _Requirements: 2.1, 2.2_
  - _Prompt: Implement the task for spec engine-state-unification, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Developer designing state mutations | Task: Create Mutation enum in core/src/engine/state/mutation.rs | Restrictions: Cover all state changes, include timestamps, be serializable | _Leverage: Existing state change patterns in engine | Success: Enum covers all mutations, clear semantics | After completion: Mark task [-] as in-progress before starting, use log-implementation tool to record artifacts, then mark [x] when complete_

- [x] 2. Create StateChange and Effect types
  - File: `core/src/engine/state/change.rs`
  - Define StateChange with version, mutation, effects
  - Define Effect enum for secondary changes
  - Add Serialize support
  - Purpose: Record state changes for events
  - _Leverage: Event pattern_
  - _Requirements: 2.3, 4.4_
  - _Prompt: Implement the task for spec engine-state-unification, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Developer creating event types | Task: Create StateChange and Effect in core/src/engine/state/change.rs | Restrictions: Serializable, include version, track all effects | _Leverage: Event sourcing patterns | Success: Changes capture all state effects, serializes correctly | After completion: Mark task [-] as in-progress before starting, use log-implementation tool to record artifacts, then mark [x] when complete_

- [x] 3. Create StateError type
  - File: `core/src/engine/state/error.rs`
  - Define error variants for invalid mutations
  - Implement std::error::Error
  - Purpose: Error handling for state operations
  - _Leverage: thiserror_
  - _Requirements: 2.4_
  - _Prompt: Implement the task for spec engine-state-unification, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Developer creating error types | Task: Create StateError in core/src/engine/state/error.rs | Restrictions: Use thiserror, cover all invalid mutations | _Leverage: thiserror patterns | Success: Errors are descriptive, help debugging | After completion: Mark task [-] as in-progress before starting, use log-implementation tool to record artifacts, then mark [x] when complete_

## Phase 2: State Components

- [x] 4. Refactor KeyState component
  - File: `core/src/engine/state/keys.rs`
  - Extract from KeyStateTracker
  - Implement is_pressed, press, release methods
  - Add timestamp tracking
  - Purpose: Key state as unified component
  - _Leverage: Existing KeyStateTracker_
  - _Requirements: 1.2, 3.1_
  - _Prompt: Implement the task for spec engine-state-unification, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Developer refactoring state | Task: Create KeyState in core/src/engine/state/keys.rs from KeyStateTracker | Restrictions: Same functionality, timestamp tracking, efficient lookups | _Leverage: Existing KeyStateTracker | Success: KeyState works identically, cleaner API | After completion: Mark task [-] as in-progress before starting, use log-implementation tool to record artifacts, then mark [x] when complete_

- [x] 5. Refactor LayerState component
  - File: `core/src/engine/state/layers.rs`
  - Extract from LayerStack
  - Implement push, pop, active_layers methods
  - Handle layer priorities
  - Purpose: Layer state as unified component
  - _Leverage: Existing LayerStack_
  - _Requirements: 1.2, 3.2_
  - _Prompt: Implement the task for spec engine-state-unification, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Developer refactoring state | Task: Create LayerState in core/src/engine/state/layers.rs from LayerStack | Restrictions: Same functionality, priority handling, efficient operations | _Leverage: Existing LayerStack | Success: LayerState works identically, cleaner API | After completion: Mark task [-] as in-progress before starting, use log-implementation tool to record artifacts, then mark [x] when complete_

- [ ] 6. Refactor ModifierState component
  - File: `core/src/engine/state/modifiers.rs`
  - Extract from ModifierState
  - Handle 255 custom modifiers
  - Implement activate, deactivate, is_active methods
  - Purpose: Modifier state as unified component
  - _Leverage: Existing ModifierState_
  - _Requirements: 1.2, 3.2_
  - _Prompt: Implement the task for spec engine-state-unification, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Developer refactoring state | Task: Create ModifierState in core/src/engine/state/modifiers.rs | Restrictions: Support 255 modifiers, efficient bitmap, clear API | _Leverage: Existing ModifierState | Success: ModifierState supports all modifiers, efficient | After completion: Mark task [-] as in-progress before starting, use log-implementation tool to record artifacts, then mark [x] when complete_

- [ ] 7. Refactor PendingState component
  - File: `core/src/engine/state/pending.rs`
  - Extract from PendingDecisionQueue
  - Implement add, resolve, clear, timeout methods
  - Track decision types and timings
  - Purpose: Pending decisions as unified component
  - _Leverage: Existing PendingDecisionQueue_
  - _Requirements: 1.2, 3.4_
  - _Prompt: Implement the task for spec engine-state-unification, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Developer refactoring state | Task: Create PendingState in core/src/engine/state/pending.rs | Restrictions: Track all decision types, efficient resolution, timing aware | _Leverage: Existing PendingDecisionQueue | Success: PendingState handles all decisions, timing works | After completion: Mark task [-] as in-progress before starting, use log-implementation tool to record artifacts, then mark [x] when complete_

## Phase 3: Unified State

- [ ] 8. Create EngineState container
  - File: `core/src/engine/state/mod.rs`
  - Combine all state components
  - Implement query methods (is_key_pressed, active_layers, etc.)
  - Add version tracking
  - Purpose: Single state container
  - _Leverage: All state components_
  - _Requirements: 1.1, 1.3, 1.4_
  - _Prompt: Implement the task for spec engine-state-unification, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Developer creating unified state | Task: Create EngineState in core/src/engine/state/mod.rs | Restrictions: Own all components, version tracking, Clone-able | _Leverage: All state components | Success: EngineState provides unified access to all state | After completion: Mark task [-] as in-progress before starting, use log-implementation tool to record artifacts, then mark [x] when complete_

- [ ] 9. Implement apply() mutation method
  - File: `core/src/engine/state/mod.rs`
  - Apply single mutation atomically
  - Update affected components
  - Return StateChange
  - Purpose: Atomic state mutations
  - _Leverage: Mutation enum, state components_
  - _Requirements: 2.1, 2.3_
  - _Prompt: Implement the task for spec engine-state-unification, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Developer implementing mutations | Task: Implement apply() in EngineState for atomic mutations | Restrictions: Atomic updates, produce StateChange, handle errors | _Leverage: Mutation enum, components | Success: apply() works for all mutations, changes tracked | After completion: Mark task [-] as in-progress before starting, use log-implementation tool to record artifacts, then mark [x] when complete_

- [ ] 10. Implement apply_batch() for atomic batches
  - File: `core/src/engine/state/mod.rs`
  - Apply multiple mutations atomically
  - Rollback on failure
  - Return all StateChanges
  - Purpose: Batch mutations
  - _Leverage: apply() method_
  - _Requirements: 2.2, 2.4_
  - _Prompt: Implement the task for spec engine-state-unification, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Developer implementing batch operations | Task: Implement apply_batch() with rollback on failure | Restrictions: Atomic semantics, full rollback, preserve state on error | _Leverage: apply() method | Success: Batches are atomic, rollback works | After completion: Mark task [-] as in-progress before starting, use log-implementation tool to record artifacts, then mark [x] when complete_

- [ ] 11. Implement state synchronization
  - File: `core/src/engine/state/mod.rs`
  - Sync modifiers when keys released
  - Sync pending when layers change
  - Validate invariants after mutations
  - Purpose: State consistency
  - _Leverage: State components_
  - _Requirements: 3.1, 3.2, 3.3, 3.4_
  - _Prompt: Implement the task for spec engine-state-unification, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Developer implementing synchronization | Task: Add state synchronization logic to EngineState mutations | Restrictions: Sync all affected components, validate invariants | _Leverage: Component APIs | Success: State always consistent, invariants enforced | After completion: Mark task [-] as in-progress before starting, use log-implementation tool to record artifacts, then mark [x] when complete_

## Phase 4: Snapshots and Inspection

- [ ] 12. Create StateSnapshot type
  - File: `core/src/engine/state/snapshot.rs`
  - Serializable state representation
  - Implement From<&EngineState>
  - Purpose: State export for FFI/debugging
  - _Leverage: serde_
  - _Requirements: 4.1, 4.2_
  - _Prompt: Implement the task for spec engine-state-unification, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Developer creating snapshots | Task: Create StateSnapshot in core/src/engine/state/snapshot.rs | Restrictions: Serializable, efficient conversion, include all relevant state | _Leverage: serde | Success: Snapshot captures state, serializes correctly | After completion: Mark task [-] as in-progress before starting, use log-implementation tool to record artifacts, then mark [x] when complete_

- [ ] 13. Implement state history tracking
  - File: `core/src/engine/state/history.rs`
  - Ring buffer of recent StateChanges
  - Configurable history depth
  - Purpose: Debugging and replay
  - _Leverage: StateChange type_
  - _Requirements: 4.3_
  - _Prompt: Implement the task for spec engine-state-unification, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Developer implementing history | Task: Create StateHistory with ring buffer of changes | Restrictions: Configurable depth, efficient storage, easy iteration | _Leverage: StateChange type | Success: History tracks changes, bounded memory | After completion: Mark task [-] as in-progress before starting, use log-implementation tool to record artifacts, then mark [x] when complete_

- [ ] 14. Implement state persistence
  - File: `core/src/engine/state/persistence.rs`
  - Save/load state to disk
  - Handle version migration
  - Fallback to clean state on corruption
  - Purpose: State persistence across sessions
  - _Leverage: serde, StateSnapshot_
  - _Requirements: 5.1, 5.2, 5.3, 5.4_
  - _Prompt: Implement the task for spec engine-state-unification, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Developer implementing persistence | Task: Create state persistence in core/src/engine/state/persistence.rs | Restrictions: Handle migration, detect corruption, safe fallback | _Leverage: serde, StateSnapshot | Success: State persists correctly, handles errors | After completion: Mark task [-] as in-progress before starting, use log-implementation tool to record artifacts, then mark [x] when complete_

## Phase 5: Integration

- [ ] 15. Update Engine to use EngineState
  - Files: `core/src/engine/{advanced,processing}.rs`
  - Replace separate state structs with EngineState
  - Use apply() for all state changes
  - Purpose: Engine integration
  - _Leverage: EngineState_
  - _Requirements: 1.1_
  - _Prompt: Implement the task for spec engine-state-unification, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Developer integrating state | Task: Update Engine core to use EngineState exclusively | Restrictions: Same behavior, use mutations, maintain performance | _Leverage: EngineState | Success: Engine uses unified state, all tests pass | After completion: Mark task [-] as in-progress before starting, use log-implementation tool to record artifacts, then mark [x] when complete_

- [ ] 16. Update FFI state exports
  - File: `core/src/ffi/exports_engine.rs`
  - Use StateSnapshot for state queries
  - Emit StateChange events
  - Purpose: FFI integration
  - _Leverage: StateSnapshot, StateChange_
  - _Requirements: 4.1, 4.4_
  - _Prompt: Implement the task for spec engine-state-unification, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Developer updating FFI | Task: Update FFI exports to use StateSnapshot and StateChange | Restrictions: Same API surface, emit events, serialize correctly | _Leverage: StateSnapshot, FFI patterns | Success: FFI provides state snapshots, emits changes | After completion: Mark task [-] as in-progress before starting, use log-implementation tool to record artifacts, then mark [x] when complete_

- [ ] 17. Remove legacy state structs
  - Files: Delete or deprecate old state files
  - Update imports throughout codebase
  - Verify no dead code
  - Purpose: Cleanup
  - _Leverage: New state module_
  - _Requirements: Non-functional (cleanup)_
  - _Prompt: Implement the task for spec engine-state-unification, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Developer completing cleanup | Task: Remove legacy state structs, update imports | Restrictions: No dead code, clean imports, all tests pass | _Leverage: New state module | Success: Old structs removed, codebase cleaner | After completion: Mark task [-] as in-progress before starting, use log-implementation tool to record artifacts, then mark [x] when complete_

- [ ] 18. Add state invariant tests
  - File: `core/tests/unit/engine/state_invariants_test.rs`
  - Property tests for state consistency
  - Fuzz mutation sequences
  - Purpose: Verify invariants
  - _Leverage: proptest_
  - _Requirements: Non-functional (reliability)_
  - _Prompt: Implement the task for spec engine-state-unification, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Test Developer | Task: Create property tests for state invariants | Restrictions: Use proptest, fuzz mutations, verify consistency | _Leverage: proptest | Success: Property tests pass, invariants verified | After completion: Mark task [-] as in-progress before starting, use log-implementation tool to record artifacts, then mark [x] when complete_
