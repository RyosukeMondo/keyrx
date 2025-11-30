# Tasks Document: advanced-remapping-engine

## Phase 1: State Management (Layer 1)

- [x] 1. Create KeyStateTracker
  - File: `core/src/engine/state/key_state.rs` (new)
  - Implement `KeyStateTracker` struct with HashMap<KeyCode, u64>
  - Methods: `key_down()`, `key_up()`, `is_pressed()`, `press_time()`, `pressed_keys()`
  - Handle duplicate key-down (is_repeat) correctly
  - Add comprehensive unit tests
  - Purpose: Track which physical keys are currently held with timestamps
  - _Leverage: InputEvent.timestamp_us, KeyCode from keycodes.rs_
  - _Requirements: REQ-1_
  - _Prompt: Implement the task for spec advanced-remapping-engine, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Developer | Task: Create KeyStateTracker in core/src/engine/state/key_state.rs that tracks pressed keys with timestamps, handles duplicates, and provides iteration | Restrictions: No heap allocation on key_down/key_up hot path, use HashMap with reserved capacity | Success: All unit tests pass, benchmarks show <100ns per operation | Instructions: Mark task [-] in tasks.md before starting, use log-implementation tool after completion with artifacts, mark [x] when done_

- [x] 2. Create ModifierState with virtual modifiers
  - File: `core/src/engine/state/modifiers.rs` (new)
  - Implement `VirtualModifiers` bitmap (256 bits = [u64; 4])
  - Implement `StandardModifiers` for OS modifiers
  - Implement `OneShotState` for sticky modifiers
  - Implement `ModifierState` combining all three
  - Methods: `activate()`, `deactivate()`, `is_active()`, `arm_one_shot()`, `consume_one_shot()`
  - Add unit tests for all modifier operations
  - Purpose: Track standard and virtual modifier state
  - _Leverage: Pattern from MockState_
  - _Requirements: REQ-3, REQ-7_
  - _Prompt: Implement the task for spec advanced-remapping-engine, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Developer specializing in bit manipulation | Task: Create ModifierState in core/src/engine/state/modifiers.rs with 256-bit virtual modifier bitmap and one-shot state machine | Restrictions: Use bitwise ops for speed, no heap allocation | Success: Supports 255 virtual modifiers, one-shot cycle works, tests pass | Instructions: Mark task [-] in tasks.md before starting, use log-implementation tool after completion with artifacts, mark [x] when done_

- [x] 3. Create LayerStack system
  - File: `core/src/engine/state/layers.rs` (new)
  - Define `Layer` struct with name, mappings, transparent flag
  - Define `LayerAction` enum (Remap, Block, TapHold, LayerPush, etc.)
  - Implement `LayerStack` with push/pop/toggle operations
  - Implement `lookup()` with top-to-bottom priority and transparency
  - Add unit tests for layer operations
  - Purpose: Manage keyboard layers with priority
  - _Leverage: RemapRegistry pattern_
  - _Requirements: REQ-4_
  - _Prompt: Implement the task for spec advanced-remapping-engine, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Developer | Task: Create LayerStack in core/src/engine/state/layers.rs with Layer struct, LayerAction enum, and priority-based lookup with transparency | Restrictions: Lookup must be O(layers * 1), no deep cloning | Success: Push/pop/toggle work, transparent fallthrough works, tests pass | Instructions: Mark task [-] in tasks.md before starting, use log-implementation tool after completion with artifacts, mark [x] when done_

- [x] 4. Create state module with re-exports
  - File: `core/src/engine/state/mod.rs` (new)
  - Re-export KeyStateTracker, ModifierState, LayerStack
  - Update `core/src/engine/mod.rs` to include state module
  - Ensure all public types are accessible
  - Purpose: Organize state management code
  - _Requirements: REQ-1, REQ-3, REQ-4_
  - _Prompt: Implement the task for spec advanced-remapping-engine, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Developer | Task: Create state/mod.rs with proper re-exports, update engine/mod.rs | Restrictions: Maintain backward compatibility | Success: All state types importable from keyrx::engine::state | Instructions: Mark task [-] in tasks.md before starting, use log-implementation tool after completion with artifacts, mark [x] when done_

## Phase 2: Decision System (Layer 2)

- [x] 5. Create TimingConfig
  - File: `core/src/engine/decision/timing.rs` (new)
  - Implement `TimingConfig` struct with all timing parameters
  - Implement `Default` with values from tech.md spec
  - Add `Serialize`/`Deserialize` for config files
  - Add builder pattern for fluent configuration
  - Purpose: Configurable timing parameters
  - _Requirements: REQ-8_
  - _Prompt: Implement the task for spec advanced-remapping-engine, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Developer | Task: Create TimingConfig in core/src/engine/decision/timing.rs with tap_timeout_ms, combo_timeout_ms, hold_delay_ms, eager_tap, permissive_hold, retro_tap | Restrictions: All fields must have sensible defaults | Success: Default matches tech.md, serialization works | Instructions: Mark task [-] in tasks.md before starting, use log-implementation tool after completion with artifacts, mark [x] when done_

- [x] 6. Create PendingDecision and DecisionQueue
  - File: `core/src/engine/decision/pending.rs` (new)
  - Define `PendingDecision` enum (TapHold, Combo variants)
  - Define `DecisionResolution` enum for outcomes
  - Implement `DecisionQueue` with add, check_event, check_timeouts
  - Handle permissive_hold interrupt tracking
  - Handle eager_tap immediate emission
  - Add comprehensive unit tests for all resolution scenarios
  - Purpose: Track and resolve timing-based decisions
  - _Leverage: TimingConfig from task 5_
  - _Requirements: REQ-2, REQ-5, REQ-10_
  - _Prompt: Implement the task for spec advanced-remapping-engine, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Developer | Task: Create DecisionQueue in core/src/engine/decision/pending.rs with tap-hold resolution (tap on early release, hold on timeout), permissive_hold, eager_tap support | Restrictions: O(n) check is acceptable for small queue, cap at 32 pending | Success: Tap/hold/interrupt scenarios all tested and working | Instructions: Mark task [-] in tasks.md before starting, use log-implementation tool after completion with artifacts, mark [x] when done_

- [x] 7. Create ComboRegistry
  - File: `core/src/engine/decision/combos.rs` (new)
  - Define `ComboDef` struct with keys and action
  - Implement `ComboRegistry` with register and find methods
  - Implement order-independent key matching
  - Add unit tests for combo matching
  - Purpose: Store and match combo definitions
  - _Requirements: REQ-6_
  - _Prompt: Implement the task for spec advanced-remapping-engine, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Developer | Task: Create ComboRegistry in core/src/engine/decision/combos.rs with order-independent key set matching | Restrictions: Use SmallVec for keys to avoid allocation | Success: 2-key and 3-key combos work regardless of press order | Instructions: Mark task [-] in tasks.md before starting, use log-implementation tool after completion with artifacts, mark [x] when done_

- [x] 8. Create decision module with re-exports
  - File: `core/src/engine/decision/mod.rs` (new)
  - Re-export TimingConfig, DecisionQueue, ComboRegistry
  - Update `core/src/engine/mod.rs` to include decision module
  - Purpose: Organize decision system code
  - _Requirements: REQ-2, REQ-5, REQ-6_
  - _Prompt: Implement the task for spec advanced-remapping-engine, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Developer | Task: Create decision/mod.rs with proper re-exports, update engine/mod.rs | Restrictions: Maintain backward compatibility | Success: All decision types importable from keyrx::engine::decision | Instructions: Mark task [-] in tasks.md before starting, use log-implementation tool after completion with artifacts, mark [x] when done_

## Phase 3: Advanced Engine Integration

- [x] 9. Create AdvancedEngine
  - File: `core/src/engine/advanced.rs` (new)
  - Implement `AdvancedEngine<I, S>` struct with all state components
  - Implement `process_event()` with full 9-step flow from design
  - Implement `tick()` for timeout checking
  - Implement safe mode toggle (Ctrl+Alt+Shift+Escape)
  - Wire up all components (KeyState, Modifiers, Layers, Decisions)
  - Purpose: Orchestrate all components into cohesive engine
  - _Leverage: Existing Engine pattern, all state/decision components_
  - _Requirements: REQ-1 through REQ-11_
  - _Prompt: Implement the task for spec advanced-remapping-engine, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Developer | Task: Create AdvancedEngine in core/src/engine/advanced.rs implementing the 9-step event processing flow with all state components | Restrictions: Keep process_event under 50 lines by delegating to helpers | Success: Full event flow works, safe mode works, all unit tests pass | Instructions: Mark task [-] in tasks.md before starting, use log-implementation tool after completion with artifacts, mark [x] when done_

- [x] 10. Add engine state inspection methods
  - File: `core/src/engine/advanced.rs` (extend)
  - Add `key_state()`, `modifiers()`, `layers()`, `pending()` accessors
  - Create `EngineState` struct for serializable state snapshot
  - Implement `snapshot()` method returning EngineState
  - Purpose: Enable GUI/FFI state inspection
  - _Requirements: REQ-10_
  - _Prompt: Implement the task for spec advanced-remapping-engine, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Developer | Task: Add state inspection methods to AdvancedEngine and create EngineState struct for serializable snapshots | Restrictions: EngineState must be Serialize for FFI | Success: All state accessible, snapshot serializes to JSON | Instructions: Mark task [-] in tasks.md before starting, use log-implementation tool after completion with artifacts, mark [x] when done_

## Phase 4: Rhai Script Integration

- [x] 11. Add tap_hold() Rhai function
  - File: `core/src/scripting/runtime.rs` (extend)
  - Register `tap_hold(key, tap_key, hold_key)` function
  - Register `tap_hold_mod(key, tap_key, hold_modifier)` variant
  - Store tap-hold configs in registry for engine access
  - Add tests for tap-hold registration
  - Purpose: Enable tap-hold configuration via Rhai scripts
  - _Leverage: Existing remap() registration pattern_
  - _Requirements: REQ-5, REQ-9_
  - _Prompt: Implement the task for spec advanced-remapping-engine, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Developer | Task: Add tap_hold() and tap_hold_mod() Rhai functions to runtime.rs, store in TapHoldRegistry | Restrictions: Use existing pending ops pattern | Success: Scripts can define tap-hold, tests pass | Instructions: Mark task [-] in tasks.md before starting, use log-implementation tool after completion with artifacts, mark [x] when done_

- [x] 12. Add combo() Rhai function
  - File: `core/src/scripting/runtime.rs` (extend)
  - Register `combo(keys_array, action_key)` function
  - Parse key array from Rhai Array type
  - Store combos in ComboRegistry
  - Add tests for combo registration
  - Purpose: Enable combo configuration via Rhai scripts
  - _Leverage: Existing remap() registration pattern_
  - _Requirements: REQ-6, REQ-9_
  - _Prompt: Implement the task for spec advanced-remapping-engine, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Developer | Task: Add combo() Rhai function that accepts array of keys and action | Restrictions: Validate key array has 2-4 keys | Success: Scripts can define combos, tests pass | Instructions: Mark task [-] in tasks.md before starting, use log-implementation tool after completion with artifacts, mark [x] when done_

- [x] 13. Add layer control Rhai functions
  - File: `core/src/scripting/runtime.rs` (extend)
  - Register `layer_define(name, transparent)` function
  - Register `layer_map(layer_name, key, action)` function
  - Register `layer_push(name)`, `layer_pop()`, `layer_toggle(name)` functions
  - Register `is_layer_active(name)` query function
  - Add tests for layer operations
  - Purpose: Enable layer configuration via Rhai scripts
  - _Leverage: LayerStack from task 3_
  - _Requirements: REQ-4, REQ-9_
  - _Prompt: Implement the task for spec advanced-remapping-engine, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Developer | Task: Add layer_define(), layer_map(), layer_push(), layer_pop(), layer_toggle(), is_layer_active() Rhai functions | Restrictions: Layer names must be validated | Success: Scripts can define and control layers, tests pass | Instructions: Mark task [-] in tasks.md before starting, use log-implementation tool after completion with artifacts, mark [x] when done_

- [x] 14. Add modifier control Rhai functions
  - File: `core/src/scripting/runtime.rs` (extend)
  - Register `define_modifier(name)` function (returns modifier ID)
  - Register `modifier_on(name)`, `modifier_off(name)` functions
  - Register `one_shot(name)` function
  - Register `is_modifier_active(name)` query function
  - Add tests for modifier operations
  - Purpose: Enable virtual modifier control via Rhai scripts
  - _Leverage: ModifierState from task 2_
  - _Requirements: REQ-3, REQ-7, REQ-9_
  - _Prompt: Implement the task for spec advanced-remapping-engine, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Developer | Task: Add define_modifier(), modifier_on(), modifier_off(), one_shot(), is_modifier_active() Rhai functions | Restrictions: Max 255 modifiers, validate names | Success: Scripts can define and control virtual modifiers, one-shot works, tests pass | Instructions: Mark task [-] in tasks.md before starting, use log-implementation tool after completion with artifacts, mark [x] when done_

- [x] 15. Add timing config Rhai functions
  - File: `core/src/scripting/runtime.rs` (extend)
  - Register `set_tap_timeout(ms)` function
  - Register `set_combo_timeout(ms)` function
  - Register `set_hold_delay(ms)` function
  - Register `set_eager_tap(bool)`, `set_permissive_hold(bool)`, `set_retro_tap(bool)` functions
  - Add tests for timing configuration
  - Purpose: Enable timing configuration via Rhai scripts
  - _Leverage: TimingConfig from task 5_
  - _Requirements: REQ-8, REQ-9_
  - _Prompt: Implement the task for spec advanced-remapping-engine, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Developer | Task: Add timing configuration Rhai functions | Restrictions: Validate ranges (e.g., timeout 1-5000ms) | Success: Scripts can configure timing, tests pass | Instructions: Mark task [-] in tasks.md before starting, use log-implementation tool after completion with artifacts, mark [x] when done_

## Phase 5: CLI Integration

- [x] 16. Update simulate command for advanced features
  - File: `core/src/cli/commands/simulate.rs` (extend)
  - Support `--hold-ms` flag for simulating key holds
  - Support `--combo` flag for simultaneous keys
  - Show pending decisions in output
  - Show layer state in output
  - Purpose: Enable testing advanced behaviors via CLI
  - _Leverage: AdvancedEngine from task 9_
  - _Requirements: REQ-9_
  - _Prompt: Implement the task for spec advanced-remapping-engine, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Developer | Task: Update simulate command to support hold duration and combo simulation, show pending decisions | Restrictions: Maintain backward compatibility with existing simulate | Success: keyrx simulate --input "CapsLock:hold:300" works | Instructions: Mark task [-] in tasks.md before starting, use log-implementation tool after completion with artifacts, mark [x] when done_

- [x] 17. Add state command for debugging
  - File: `core/src/cli/commands/state.rs` (new or extend)
  - Add `--pending` flag to show pending decisions
  - Add `--layers` flag to show active layers
  - Add `--modifiers` flag to show active modifiers
  - Add `--json` output format
  - Purpose: Enable debugging of engine state via CLI
  - _Leverage: AdvancedEngine.snapshot() from task 10_
  - _Requirements: REQ-10_
  - _Prompt: Implement the task for spec advanced-remapping-engine, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Developer | Task: Add/extend state command with flags for pending, layers, modifiers, and JSON output | Restrictions: JSON format must match EngineState struct | Success: keyrx state --pending --json shows pending decisions | Instructions: Mark task [-] in tasks.md before starting, use log-implementation tool after completion with artifacts, mark [x] when done_

## Phase 6: Testing & Verification

- [x] 18. Create integration tests for tap-hold
  - File: `core/tests/tap_hold_test.rs` (new)
  - Test tap scenario (quick release)
  - Test hold scenario (timeout expires)
  - Test permissive_hold (interrupt resolves as hold)
  - Test eager_tap (immediate emit + correction)
  - Test retro_tap (tap on release after hold)
  - Purpose: Verify tap-hold behavior end-to-end
  - _Requirements: REQ-5_
  - _Prompt: Implement the task for spec advanced-remapping-engine, first run spec-workflow-guide to get the workflow guide then implement the task: Role: QA Engineer | Task: Create comprehensive tap-hold integration tests covering all scenarios | Restrictions: Use mock clock for deterministic timing | Success: All 5 scenarios tested and passing | Instructions: Mark task [-] in tasks.md before starting, use log-implementation tool after completion with artifacts, mark [x] when done_

- [x] 19. Create integration tests for combos
  - File: `core/tests/combo_test.rs` (new)
  - Test 2-key combo (simultaneous)
  - Test 3-key combo
  - Test combo with different press order
  - Test combo timeout (partial keys pass through)
  - Purpose: Verify combo behavior end-to-end
  - _Requirements: REQ-6_
  - _Prompt: Implement the task for spec advanced-remapping-engine, first run spec-workflow-guide to get the workflow guide then implement the task: Role: QA Engineer | Task: Create comprehensive combo integration tests | Restrictions: Test order-independence | Success: All combo scenarios tested and passing | Instructions: Mark task [-] in tasks.md before starting, use log-implementation tool after completion with artifacts, mark [x] when done_

- [x] 20. Create integration tests for layers
  - File: `core/tests/layer_test.rs` (new)
  - Test layer push/pop
  - Test layer toggle
  - Test transparent layer fallthrough
  - Test layer with tap-hold
  - Purpose: Verify layer behavior end-to-end
  - _Requirements: REQ-4_
  - _Prompt: Implement the task for spec advanced-remapping-engine, first run spec-workflow-guide to get the workflow guide then implement the task: Role: QA Engineer | Task: Create comprehensive layer integration tests | Restrictions: Test interaction with tap-hold | Success: All layer scenarios tested and passing | Instructions: Mark task [-] in tasks.md before starting, use log-implementation tool after completion with artifacts, mark [x] when done_

- [x] 21. Run benchmarks and verify latency
  - Run `cargo bench` on all new code paths
  - Benchmark `process_event()` with pending decisions
  - Benchmark layer lookup with 5+ layers
  - Benchmark combo matching with 10+ combos
  - Document results, ensure < 1ms total
  - Purpose: Verify performance requirements
  - _Requirements: REQ-8 (non-functional)_
  - _Prompt: Implement the task for spec advanced-remapping-engine, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Performance Engineer | Task: Run benchmarks on all new code paths, document results | Restrictions: Fail if any operation > 100 microseconds | Success: All benchmarks pass, documented in implementation log | Instructions: Mark task [-] in tasks.md before starting, use log-implementation tool after completion with artifacts, mark [x] when done_

- [x] 22. Create example scripts demonstrating features
  - File: `scripts/examples/tap_hold.rhai` (new)
  - File: `scripts/examples/combos.rhai` (new)
  - File: `scripts/examples/layers.rhai` (new)
  - File: `scripts/examples/home_row_mods.rhai` (new)
  - Document each with comments explaining usage
  - Purpose: Provide users with working examples
  - _Requirements: All_
  - _Prompt: Implement the task for spec advanced-remapping-engine, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Technical Writer | Task: Create 4 example Rhai scripts demonstrating tap-hold, combos, layers, and home row mods | Restrictions: Must be copy-paste usable | Success: All scripts work when run with keyrx run --script | Instructions: Mark task [-] in tasks.md before starting, use log-implementation tool after completion with artifacts, mark [x] when done_
