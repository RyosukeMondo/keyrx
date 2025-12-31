# Tasks Document

## Phase 1: Core Runtime Data Structures

- [x] 1. Create runtime module structure
  - Files: `keyrx_core/src/runtime/mod.rs`, `state.rs`, `lookup.rs`, `event.rs`
  - Set up module exports and public API
  - Add `bitvec` dependency to keyrx_core/Cargo.toml
  - Purpose: Establish module structure for runtime components
  - _Leverage: keyrx_core/src/config (existing)_
  - _Requirements: Non-Functional: Code Architecture and Modularity_
  - _Prompt: Role: Rust Software Architect with expertise in module design and workspace organization | Task: Create keyrx_core/src/runtime module with mod.rs, state.rs, lookup.rs, event.rs files, set up public API exports in mod.rs, and add bitvec crate dependency to Cargo.toml | Restrictions: Must follow existing keyrx_core module patterns, maintain no_std compatibility design (don't use no_std yet, but design for it), ensure clear module boundaries with no circular dependencies | Success: Module compiles without errors, mod.rs exports DeviceState, KeyLookup, KeyEvent, process_event function, all files have proper documentation comments, bitvec dependency added to Cargo.toml_

- [x] 2. Implement DeviceState with bit vectors
  - File: `keyrx_core/src/runtime/state.rs`
  - Define `DeviceState` struct with 255-bit modifiers and locks (using `BitVec<u8, Lsb0>`)
  - Implement `new()`, `set_modifier()`, `clear_modifier()`, `toggle_lock()`
  - Implement `is_modifier_active()`, `is_lock_active()`
  - Purpose: Track runtime state (255 modifiers + 255 locks) with sub-microsecond updates
  - _Leverage: bitvec crate for efficient bit manipulation_
  - _Requirements: 2.1, 2.2, 2.3, 2.4_
  - _Prompt: Role: Rust Systems Programmer with expertise in bit manipulation and high-performance data structures | Task: Implement DeviceState struct using BitVec<u8, Lsb0> for 255-bit modifier and lock state vectors, following requirements 2.1-2.4, with methods for setting/clearing/toggling bits and querying state | Restrictions: Must validate bit indexes (0-254 only, reject 255+), no panics on invalid input (log error and return), use BitVec::set/get for clarity, design for <10μs update time (measure later), add comprehensive doc comments explaining bit layout | Success: DeviceState::new() creates zeroed 255-bit vectors, set_modifier(0) and set_modifier(254) work correctly, set_modifier(255) logs error and returns without panic, is_modifier_active returns correct boolean, toggle_lock flips bit state (first call ON, second OFF), all methods have doc comments with examples_

- [x] 3. Implement condition evaluation in DeviceState
  - File: `keyrx_core/src/runtime/state.rs` (continue)
  - Implement `evaluate_condition(&self, condition: &Condition) -> bool`
  - Handle `Condition::ModifierActive`, `LockActive`, `AllActive`, `NotActive`
  - Purpose: Enable conditional mapping evaluation during event processing
  - _Leverage: keyrx_core::config::Condition, ConditionItem enums (existing)_
  - _Requirements: 2.5, 2.6, 2.7, 2.8_
  - _Prompt: Role: Rust Developer with expertise in pattern matching and enum handling | Task: Implement evaluate_condition method in DeviceState that evaluates Condition enum variants (ModifierActive, LockActive, AllActive, NotActive) following requirements 2.5-2.8, using existing Condition and ConditionItem enums from keyrx_core::config | Restrictions: Must handle all Condition variants exhaustively with match, AllActive returns true only if all items evaluate to true, NotActive returns true only if all items evaluate to false, use is_modifier_active and is_lock_active helpers (don't duplicate bit checking logic), add doc comments with examples for each variant | Success: evaluate_condition(Condition::ModifierActive(0)) returns true when modifier 0 is set, AllActive([MD_00, LK_01]) returns true only when both are active, NotActive([MD_00]) returns true when MD_00 is NOT active, handles all variants without panics_

- [x] 4. Write DeviceState unit tests
  - File: `keyrx_core/src/runtime/state.rs` (add tests module)
  - Test set/clear modifier for IDs 0, 127, 254
  - Test toggle_lock behavior (OFF→ON→OFF)
  - Test evaluate_condition for all variants
  - Test invalid ID handling (255+)
  - Purpose: Ensure DeviceState reliability and correct behavior
  - _Leverage: None (unit tests)_
  - _Requirements: 2.1-2.8_
  - _Prompt: Role: QA Engineer with Rust testing expertise and focus on edge case coverage | Task: Create comprehensive unit tests for DeviceState covering requirements 2.1-2.8, testing all methods with valid inputs (0, 127, 254), invalid inputs (255+), boundary conditions, and condition evaluation logic | Restrictions: Must test each public method independently, use assert_eq! for state checks, verify toggle_lock behavior (first press ON, second press OFF, third press ON), test AllActive with multiple conditions, test NotActive with single condition, ensure tests are isolated (each test creates fresh DeviceState) | Success: Tests cover set_modifier/clear_modifier/toggle_lock for IDs 0, 127, 254, test set_modifier(255) doesn't panic (logs error), test toggle_lock cycles OFF→ON→OFF, test evaluate_condition for all Condition variants, all tests pass_

## Phase 2: Key Lookup Implementation

- [x] 5. Implement KeyLookup with HashMap
  - File: `keyrx_core/src/runtime/lookup.rs`
  - Define `KeyLookup` struct with `HashMap<KeyCode, Vec<BaseKeyMapping>>`
  - Implement `from_device_config(config: &DeviceConfig) -> Self`
  - Build lookup table: iterate mappings, group by input key
  - Order: conditional mappings first (registration order), unconditional last
  - Purpose: O(1) average-case key→mapping resolution
  - _Leverage: keyrx_core::config::{DeviceConfig, BaseKeyMapping, KeyCode} (existing)_
  - _Requirements: 3.1, 3.7_
  - _Prompt: Role: Rust Developer with expertise in HashMap optimization and data structure design | Task: Implement KeyLookup struct using HashMap<KeyCode, Vec<BaseKeyMapping>> to build lookup table from DeviceConfig, following requirements 3.1 and 3.7, grouping mappings by input key and ordering conditionals before unconditional | Restrictions: Must iterate DeviceConfig.mappings, extract input key from each BaseKeyMapping variant (Simple→from, Modifier→key, Lock→key, TapHold→key, ModifiedOutput→from, Conditional→extract from nested mappings), insert into HashMap using entry API, push conditional mappings before unconditional, use Vec::with_capacity for efficiency if possible, add doc comments explaining ordering | Success: from_device_config builds HashMap correctly, mappings for same key are grouped in Vec, conditional mappings appear before unconditional in Vec, handles empty DeviceConfig (returns empty HashMap), compiles without warnings_

- [x] 6. Implement find_mapping with condition evaluation
  - File: `keyrx_core/src/runtime/lookup.rs` (continue)
  - Implement `find_mapping(&self, key: KeyCode, state: &DeviceState) -> Option<&BaseKeyMapping>`
  - Iterate Vec<BaseKeyMapping> for key
  - For Conditional mappings: call `state.evaluate_condition()`, return if true
  - Return first matching conditional, or unconditional if no conditionals match
  - Purpose: Find correct mapping based on current runtime state
  - _Leverage: DeviceState::evaluate_condition (new, same spec)_
  - _Requirements: 3.2, 3.3, 3.4, 3.5, 3.6_
  - _Prompt: Role: Rust Developer with expertise in Option handling and iterator patterns | Task: Implement find_mapping method in KeyLookup that searches HashMap for key, iterates Vec<BaseKeyMapping>, evaluates conditional mappings using DeviceState::evaluate_condition, and returns first matching mapping following requirements 3.2-3.6 | Restrictions: Must use HashMap::get to retrieve Vec, return None if key not in table (passthrough case), iterate Vec in order (conditionals first), for Conditional variant extract condition and call state.evaluate_condition, return Some(&mapping) on first match, return unconditional mapping if no conditionals match, never clone BaseKeyMapping (return reference), add doc comments with examples | Success: find_mapping returns None for unmapped key, returns &BaseKeyMapping for simple mapping, evaluates conditionals in order and returns first match, falls back to unconditional if no conditionals match, handles Vec with only conditionals (returns None if none match), compiles without warnings_

- [x] 7. Write KeyLookup unit tests
  - File: `keyrx_core/src/runtime/lookup.rs` (add tests module)
  - Test from_device_config with simple, conditional, mixed mappings
  - Test find_mapping with no mapping (passthrough)
  - Test find_mapping with conditional true/false
  - Test ordering: conditionals before unconditional
  - Purpose: Ensure lookup correctness and performance
  - _Leverage: keyrx_core::config::mappings helper functions (simple, conditional)_
  - _Requirements: 3.1-3.7_
  - _Prompt: Role: QA Engineer with Rust unit testing expertise and focus on data structure validation | Task: Create comprehensive unit tests for KeyLookup covering requirements 3.1-3.7, testing from_device_config table building and find_mapping resolution with various scenarios (simple, conditional, mixed, no mapping) | Restrictions: Must use KeyMapping helper functions (simple, conditional) from keyrx_core::config::mappings to create test configs, test ordering by verifying conditional checked before unconditional, use DeviceState with specific modifier state for conditional tests, verify find_mapping returns correct &BaseKeyMapping reference, ensure tests are isolated | Success: Tests cover from_device_config with empty config, simple mapping (A→B), conditional mapping (MD_00 active → H→Left), mixed (conditional + unconditional), find_mapping returns None for unmapped key, find_mapping returns conditional when condition true, falls back to unconditional when condition false, all tests pass_

## Phase 3: Event Processing Logic

- [x] 8. Define KeyEvent enum
  - File: `keyrx_core/src/runtime/event.rs`
  - Define `KeyEvent` enum: `Press(KeyCode)`, `Release(KeyCode)`
  - Derive: `Debug, Clone, Copy, PartialEq, Eq, Hash`
  - Add `keycode(&self) -> KeyCode` helper method
  - Purpose: Represent keyboard events in type-safe way
  - _Leverage: keyrx_core::config::KeyCode (existing)_
  - _Requirements: 4.1, 4.2_
  - _Prompt: Role: Rust Type System Expert with focus on ergonomic API design | Task: Define KeyEvent enum with Press and Release variants holding KeyCode, derive all useful traits (Debug, Clone, Copy, PartialEq, Eq, Hash), and add keycode helper method following requirements 4.1-4.2 | Restrictions: Must use #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)] for maximum utility, keycode method must extract KeyCode from both variants (use match), add doc comments explaining usage and why Copy is safe (KeyCode is Copy), include examples in doc comments | Success: KeyEvent::Press and KeyEvent::Release defined, all derives compile, keycode() returns correct KeyCode for both variants, enum is Copy (can pass by value efficiently), doc comments include usage examples_

- [x] 9. Implement process_event for simple mappings
  - File: `keyrx_core/src/runtime/event.rs` (continue)
  - Implement `process_event(event: KeyEvent, lookup: &KeyLookup, state: &mut DeviceState) -> Vec<KeyEvent>`
  - Handle `BaseKeyMapping::Simple`: map input key to output key
  - Handle passthrough: if no mapping, return original event
  - Purpose: Core event processing logic for simple 1:1 remapping
  - _Leverage: KeyLookup::find_mapping, KeyEvent enum_
  - _Requirements: 4.1, 4.2, 4.3_
  - _Prompt: Role: Rust Developer with expertise in event-driven systems and pattern matching | Task: Implement process_event function that takes KeyEvent, uses KeyLookup::find_mapping to resolve mapping, handles BaseKeyMapping::Simple by remapping key, and handles passthrough (no mapping) by returning original event, following requirements 4.1-4.3 | Restrictions: Must call lookup.find_mapping(event.keycode(), state), match on Option returned, if None return vec![event] (passthrough), if Some match on BaseKeyMapping variants, for Simple variant replace keycode with output key while preserving Press/Release, return Vec<KeyEvent> for consistency (even single event), use match event { Press(k) => Press(output_k), Release(k) => Release(output_k) } pattern, add TODO comments for unimplemented variants (Modifier, Lock, etc.), add doc comments with examples | Success: process_event with no mapping returns original event unchanged, process_event with Simple A→B mapping returns Press(B) for Press(A), returns Release(B) for Release(A), returns Vec with one element, compiles with warnings for unimplemented variants (expected at this stage)_

- [x] 10. Implement process_event for Modifier and Lock mappings
  - File: `keyrx_core/src/runtime/event.rs` (continue)
  - Handle `BaseKeyMapping::Modifier`: Press→set bit, Release→clear bit, no output
  - Handle `BaseKeyMapping::Lock`: Press→toggle bit, Release→no output
  - Purpose: State management mappings (modifiers and locks)
  - _Leverage: DeviceState::set_modifier, clear_modifier, toggle_lock_
  - _Requirements: 4.4, 4.5_
  - _Prompt: Role: Rust Developer with expertise in stateful event processing and side effects | Task: Extend process_event to handle BaseKeyMapping::Modifier and Lock variants, updating DeviceState and returning empty Vec (no output events), following requirements 4.4-4.5 | Restrictions: For Modifier variant on Press call state.set_modifier(id), on Release call state.clear_modifier(id), return Vec::new() (no output), for Lock variant on Press call state.toggle_lock(id), on Release return Vec::new() (do nothing), use match on event inside Modifier/Lock arms to distinguish Press/Release, add doc comments explaining state update behavior | Success: Modifier mapping on Press sets modifier bit and returns empty Vec, on Release clears modifier bit and returns empty Vec, Lock mapping on Press toggles lock bit and returns empty Vec, on Release returns empty Vec (no toggle), DeviceState is correctly mutated, no output events for Modifier or Lock mappings_

- [x] 11. Implement process_event for ModifiedOutput and Conditional mappings
  - File: `keyrx_core/src/runtime/event.rs` (continue)
  - Handle `BaseKeyMapping::ModifiedOutput`: Press→output modifiers then key, Release→reverse order
  - Handle `BaseKeyMapping::Conditional`: already handled by find_mapping (conditions evaluated there)
  - Add TapHold stub: return `Vec::new()` with TODO comment
  - Purpose: Complete event processing for all mapping types
  - _Leverage: KeyEvent enum for building output sequences_
  - _Requirements: 4.6, 4.7, 4.8_
  - _Prompt: Role: Rust Developer with expertise in complex event sequencing and pattern matching | Task: Extend process_event to handle BaseKeyMapping::ModifiedOutput by generating multiple KeyEvents (modifier presses, then key press) and BaseKeyMapping::Conditional (delegate to find_mapping), add TapHold stub with TODO, following requirements 4.6-4.8 | Restrictions: For ModifiedOutput on Press: build Vec starting with modifier presses (if shift: Press(LShift), if ctrl: Press(LCtrl), etc.) then Press(output_key), on Release: reverse order (Release(output_key) then modifier releases), for Conditional just return result (condition already evaluated in find_mapping, so this branch may not be reached directly), for TapHold return Vec::new() and add TODO comment "TapHold deferred to advanced-input-logic spec", add doc comments with examples showing multi-event output | Success: ModifiedOutput with shift=true outputs [Press(LShift), Press(key)] on Press, [Release(key), Release(LShift)] on Release, Conditional mappings work via find_mapping (no special handling needed in process_event), TapHold returns empty Vec with TODO, compiles without errors_

- [x] 12. Write process_event unit tests
  - File: `keyrx_core/src/runtime/event.rs` (add tests module)
  - Test passthrough (no mapping)
  - Test Simple mapping (A→B)
  - Test Modifier mapping (state update, no output)
  - Test Lock mapping (toggle, no output)
  - Test ModifiedOutput (Shift+1 sequence)
  - Test Conditional mapping (with state)
  - Purpose: Ensure process_event handles all mapping types correctly
  - _Leverage: KeyMapping helpers, DeviceState, KeyLookup_
  - _Requirements: 4.1-4.8_
  - _Prompt: Role: QA Engineer with Rust testing expertise and focus on event processing validation | Task: Create comprehensive unit tests for process_event covering all BaseKeyMapping variants and requirements 4.1-4.8, testing passthrough, simple remapping, modifier/lock state updates, modified output sequences, and conditional evaluation | Restrictions: Must create DeviceConfig with specific mappings using KeyMapping helpers, build KeyLookup from config, create DeviceState, call process_event with test events, assert output Vec<KeyEvent> matches expected, verify DeviceState changes for Modifier/Lock, test both Press and Release events, ensure tests are isolated | Success: Tests cover passthrough (unmapped key returns original event), Simple A→B (Press(A) returns [Press(B)]), Modifier (Press sets state, returns empty Vec), Lock (Press toggles, returns empty Vec), ModifiedOutput (Shift+1 returns multi-event sequence), Conditional (evaluates condition using state), all tests pass_

## Phase 4: Platform Abstraction Layer

- [x] 13. Define platform traits
  - File: `keyrx_daemon/src/platform/mod.rs` (NEW)
  - Define `InputDevice` trait: `next_event()`, `grab()`, `release()`
  - Define `OutputDevice` trait: `inject_event()`
  - Define `DeviceError` enum: `NotFound`, `PermissionDenied`, `EndOfStream`, `InjectionFailed`, `Io`
  - Add `thiserror` dependency to keyrx_daemon/Cargo.toml
  - Purpose: Platform-agnostic contracts for input/output devices
  - _Leverage: keyrx_core::runtime::event::KeyEvent (new, same spec)_
  - _Requirements: 5.1_
  - _Prompt: Role: Rust Software Architect with expertise in trait design and platform abstraction | Task: Define InputDevice and OutputDevice traits with method signatures for event retrieval and injection, define DeviceError enum using thiserror crate, following requirement 5.1 | Restrictions: InputDevice must have next_event() -> Result<KeyEvent, DeviceError>, grab() -> Result<(), DeviceError>, release() -> Result<(), DeviceError>, OutputDevice must have inject_event(event: KeyEvent) -> Result<(), DeviceError>, DeviceError variants must use #[error("...")] attributes from thiserror, include Io variant with #[from] std::io::Error, add comprehensive doc comments explaining trait contracts and when each error variant occurs | Success: Traits compile with correct signatures, DeviceError has all required variants (NotFound, PermissionDenied, EndOfStream, InjectionFailed, Io), thiserror derives Display and Error traits, doc comments explain grab/release semantics (exclusive device access), traits are public and exported from platform module_

- [x] 14. Implement MockInput device
  - File: `keyrx_daemon/src/platform/mock.rs` (NEW)
  - Define `MockInput` struct: `events: VecDeque<KeyEvent>`, `grabbed: bool`
  - Implement `InputDevice` trait: next_event pops from queue, grab sets flag, release clears flag
  - Add `new(events: Vec<KeyEvent>)` constructor
  - Add `is_grabbed() -> bool` helper for tests
  - Purpose: Zero-dependency input simulation for testing
  - _Leverage: std::collections::VecDeque for FIFO queue_
  - _Requirements: 5.2, 5.5_
  - _Prompt: Role: Rust Developer with expertise in mock implementations and test infrastructure | Task: Implement MockInput struct with VecDeque for event queue, implement InputDevice trait where next_event pops events and returns EndOfStream when empty, grab/release update grabbed flag, following requirements 5.2 and 5.5 | Restrictions: Must use VecDeque::pop_front for next_event (FIFO order), return Ok(event) if queue not empty, Err(DeviceError::EndOfStream) if empty, grab sets grabbed = true, release sets grabbed = false, new() converts Vec to VecDeque, is_grabbed() returns grabbed field, must have zero OS dependencies (no libc, no evdev, pure Rust), add doc comments with usage examples | Success: MockInput::new creates instance with preloaded events, next_event returns events in FIFO order, returns EndOfStream when exhausted, grab/release update grabbed field, is_grabbed allows test verification, compiles without OS-specific dependencies_

- [x] 15. Implement MockOutput device
  - File: `keyrx_daemon/src/platform/mock.rs` (continue)
  - Define `MockOutput` struct: `events: Vec<KeyEvent>`
  - Implement `OutputDevice` trait: inject_event appends to Vec
  - Add `new() -> Self` constructor
  - Add `events() -> &[KeyEvent]` getter for test verification
  - Purpose: Zero-dependency output capture for testing
  - _Leverage: Vec for append-only event collection_
  - _Requirements: 5.3, 5.5_
  - _Prompt: Role: Rust Developer with expertise in mock implementations and test utilities | Task: Implement MockOutput struct with Vec<KeyEvent> for capturing injected events, implement OutputDevice trait where inject_event appends to Vec, following requirements 5.3 and 5.5 | Restrictions: Must use Vec::push for inject_event, always return Ok(()) (mock never fails unless explicitly configured), new() creates empty Vec, events() returns slice reference for test assertions, must have zero OS dependencies, add optional fail_mode field (bool) to simulate InjectionFailed error if needed for error testing, add doc comments | Success: MockOutput::new creates empty instance, inject_event appends to Vec and returns Ok(()), events() returns all injected events in order, compiles without OS-specific dependencies, can be used in tests to verify output event sequences_

- [x] 16. Write platform mock tests
  - File: `keyrx_daemon/src/platform/mock.rs` (add tests module)
  - Test MockInput: event sequence, grab/release, EndOfStream
  - Test MockOutput: event capture, ordering
  - Purpose: Verify mock implementations behave correctly
  - _Leverage: KeyEvent enum_
  - _Requirements: 5.2, 5.3_
  - _Prompt: Role: QA Engineer with focus on mock validation and test infrastructure | Task: Create unit tests for MockInput and MockOutput covering all trait methods and edge cases following requirements 5.2-5.3 | Restrictions: For MockInput test event sequence matches input order, test grab sets flag, test release clears flag, test next_event returns EndOfStream after events exhausted, for MockOutput test inject_event appends events, test events() returns correct sequence, ensure tests are isolated, use assert_eq! for event comparisons | Success: Tests cover MockInput with 3 events (returns all 3 then EndOfStream), MockInput grab/release updates grabbed flag correctly, MockOutput captures all injected events, events() returns correct order, all tests pass_

## Phase 5: Event Processor and Config Loader

- [x] 17. Implement ConfigLoader
  - File: `keyrx_daemon/src/config_loader.rs` (NEW)
  - Define `load_config<P: AsRef<Path>>(path: P) -> Result<ConfigRoot, ConfigError>`
  - Define `ConfigError` enum: `Io`, `Deserialize` (wraps DeserializeError)
  - Read file bytes, call `keyrx_compiler::serialize::deserialize`
  - Purpose: Load and validate .krx binary files
  - _Leverage: keyrx_compiler::serialize::deserialize (existing)_
  - _Requirements: 1.1-1.6_
  - _Prompt: Role: Rust Developer with expertise in file I/O and error handling | Task: Implement load_config function that reads .krx file and calls keyrx_compiler::serialize::deserialize to load ConfigRoot, define ConfigError enum wrapping I/O and deserialization errors, following requirements 1.1-1.6 | Restrictions: Must use std::fs::read to load file bytes, call deserialize(bytes), use ? operator to propagate errors, ConfigError must have #[from] conversions for io::Error and DeserializeError using thiserror, add doc comments explaining error scenarios (file not found, corrupted hash, invalid magic), include example usage in doc comments | Success: load_config reads valid .krx and returns ConfigRoot, load_config with missing file returns ConfigError::Io, load_config with corrupted .krx returns ConfigError::Deserialize with specific variant (HashMismatch, InvalidMagic, etc.), compiles and links against keyrx_compiler crate_

- [x] 18. Implement EventProcessor orchestrator
  - File: `keyrx_daemon/src/processor.rs` (NEW)
  - Define `EventProcessor<I: InputDevice, O: OutputDevice>` struct
  - Implement `new(config: &DeviceConfig, input: I, output: O) -> Self`
  - Implement `process_one(&mut self) -> Result<(), ProcessorError>`
  - Implement `run(&mut self) -> Result<(), ProcessorError>`
  - Define `ProcessorError` enum: `Input`, `Output`
  - Purpose: Main event loop orchestrator (input → process → output)
  - _Leverage: KeyLookup, DeviceState, process_event, InputDevice, OutputDevice traits_
  - _Requirements: 4.9, 6.1_
  - _Prompt: Role: Rust Developer with expertise in event-driven architectures and generic programming | Task: Implement EventProcessor struct generic over InputDevice and OutputDevice, with new constructor that builds KeyLookup and DeviceState from config, process_one that reads one event and processes it, run that loops until EndOfStream, define ProcessorError wrapping device errors, following requirements 4.9 and 6.1 | Restrictions: new() must call KeyLookup::from_device_config and DeviceState::new, store input, output, lookup, state as fields, process_one must call input.next_event(), call keyrx_core::runtime::event::process_event, iterate output events and call output.inject_event for each, return Result<(), ProcessorError>, run() must loop calling process_one until Err(ProcessorError::Input(EndOfStream)), ProcessorError must have Input(DeviceError) and Output(DeviceError) variants using thiserror, add doc comments with usage examples | Success: EventProcessor compiles with generic parameters, new() builds lookup and state correctly, process_one reads event, processes it, outputs results, run() loops until EndOfStream, ProcessorError propagates device errors, compiles without warnings_

- [x] 19. Add structured logging to EventProcessor
  - File: `keyrx_daemon/src/processor.rs` (continue)
  - Add `log` crate dependency to keyrx_daemon/Cargo.toml
  - Log JSON events: config_loaded, key_processed, state_transition, platform_error
  - Use DEBUG level for per-event logging, INFO for lifecycle, ERROR for failures
  - Purpose: Machine-readable observability for AI agents
  - _Leverage: serde_json for JSON formatting (if needed)_
  - _Requirements: 1.6, 4.10, 6.4, 6.5, 6.6, Observability_
  - _Prompt: Role: Rust Developer with expertise in structured logging and observability | Task: Add structured logging to EventProcessor using log crate (with JSON format), logging config_loaded, key_processed, state_transition, and platform_error events following requirements 1.6, 4.10, 6.4-6.6 and Observability section | Restrictions: Must use log::debug!, log::info!, log::error! macros, log JSON strings manually (format as "{\"timestamp\":\"...\",\"level\":\"...\",\"service\":\"keyrx_daemon\",\"event_type\":\"...\",\"context\":{...}}"), use chrono or time crate for timestamps if needed, log key_processed at DEBUG level with input/output keys and latency_us (measure with std::time::Instant), log config_loaded at INFO level with device count, log platform_error at ERROR level with error details, never log PII or secrets, add doc comments explaining log format | Success: EventProcessor logs config_loaded on new(), logs key_processed on each process_one with latency measurement, logs platform_error on device errors, all logs are valid JSON with required schema fields, compiles with log crate dependency_

- [x] 20. Write EventProcessor integration tests
  - File: `keyrx_daemon/tests/processor_tests.rs` (NEW)
  - Test end-to-end: load config → create processor → process events → verify output
  - Test passthrough, simple remap, modifier activation, lock toggle
  - Test error handling (EndOfStream)
  - Purpose: Verify complete event processing pipeline
  - _Leverage: MockInput, MockOutput, load_config (or manually construct ConfigRoot)_
  - _Requirements: 4.1-4.9, 5.4_
  - _Prompt: Role: Integration Test Engineer with Rust testing expertise | Task: Create end-to-end integration tests for EventProcessor using mock platform, testing complete workflows from config loading to event output following requirements 4.1-4.9 and 5.4 | Restrictions: Must create test configs using KeyMapping helpers or load from test .krx files, create MockInput with predefined event sequences, create MockOutput to capture results, instantiate EventProcessor with config and mocks, call run() or process_one repeatedly, assert output events match expected sequences, test passthrough (unmapped key), simple remap (A→B), modifier activation (CapsLock→MD_00 then conditional mapping), lock toggle (ScrollLock→LK_01), verify EndOfStream handling (run() returns Ok after stream ends), use #[test] for each scenario | Success: Test end_to_end_simple_remap loads config with A→B, processes [Press(A), Release(A)], outputs [Press(B), Release(B)], test conditional_with_modifier activates MD_00 then applies conditional mapping, test lock_toggle toggles LK_01 on/off correctly, test passthrough returns original events, all tests pass_

## Phase 6: Testing and Validation

- [x] 21. Add property-based tests for DeviceState
  - File: `keyrx_core/src/runtime/state.rs` (add proptest tests)
  - Add `proptest` dev-dependency to keyrx_core/Cargo.toml
  - Test: modifier state always valid (bits 0-254, never 255)
  - Test: lock toggle cycles correctly (OFF→ON→OFF→...)
  - Purpose: Verify state management invariants hold for random inputs
  - _Leverage: proptest crate_
  - _Requirements: 2.1-2.4, Testability: Property-Based Testing_
  - _Prompt: Role: Property-Based Testing Expert with Rust and proptest expertise | Task: Add proptest-based property tests to DeviceState verifying state invariants hold for random inputs following requirements 2.1-2.4 and Testability section | Restrictions: Must add proptest to dev-dependencies, use proptest! macro, generate random modifier IDs (0..=255u8), test that IDs >254 don't set bits (invalid IDs rejected), generate random set/clear sequences and verify state matches expected, test lock toggle with random number of toggles (verify final state = toggles % 2), run tests with at least 100 cases (default), add comments explaining invariants being tested | Success: Property test prop_modifier_state_valid generates random IDs, verifies only bits 0-254 can be set, prop_lock_toggle_cycles generates random toggle counts, verifies final state matches parity, tests run 100+ iterations, all pass_

- [x] 22. Add property-based tests for event processing
  - File: `keyrx_core/src/runtime/event.rs` (add proptest tests)
  - Test: no event loss (input count == output count for Simple mappings)
  - Test: deterministic execution (same input → same output)
  - Purpose: Verify event processing invariants
  - _Leverage: proptest crate, KeyEvent enum_
  - _Requirements: 4.1-4.3, Testability: Property-Based Testing_
  - _Prompt: Role: Property-Based Testing Expert with focus on event-driven systems | Task: Add proptest-based property tests to process_event verifying no event loss and deterministic execution following requirements 4.1-4.3 and Testability section | Restrictions: Must generate random Vec<KeyEvent> using proptest strategies, create simple config (A→B mapping), process all events, count input and output events (for Simple mapping 1:1), verify counts match, run same event sequence twice and verify outputs are identical (deterministic), test with at least 100 cases, add comments explaining invariants | Success: Property test prop_no_event_loss generates random event sequences, processes with Simple mapping, verifies input count == output count, prop_deterministic processes same sequence twice, verifies outputs are byte-for-byte identical, tests run 100+ iterations, all pass_

- [x] 23. Add criterion benchmarks for performance claims
  - File: `keyrx_core/benches/core_bench.rs` (UPDATED)
  - Add `criterion` dev-dependency to keyrx_core/Cargo.toml
  - Benchmark: key lookup (<100μs target)
  - Benchmark: state update (<10μs target)
  - Benchmark: process_event end-to-end (<1ms target)
  - Purpose: Verify performance requirements with measurements
  - _Leverage: criterion crate_
  - _Requirements: Performance: Event Processing Latency, Lookup Time, State Update Time_
  - _Prompt: Role: Performance Engineer with Rust benchmarking expertise | Task: Create criterion benchmarks measuring key lookup time, state update time, and end-to-end event processing time, verifying they meet performance targets (<100μs, <10μs, <1ms respectively) following Performance requirements | Restrictions: Must add criterion to dev-dependencies with harness = false in Cargo.toml [[bench]] section, create benches/runtime_benchmarks.rs, use criterion::Criterion and criterion::black_box to prevent optimization, benchmark KeyLookup::find_mapping with realistic config (100 mappings), benchmark DeviceState::set_modifier, benchmark process_event with simple mapping, run with `cargo bench`, add comments showing target times | Success: Benchmarks compile and run with `cargo bench`, output shows lookup <100μs, state update <10μs, process_event <1ms on typical hardware (logged for future optimization), criterion generates report in target/criterion/_

- [x] 24. Create end-to-end integration test suite
  - File: `keyrx_daemon/tests/integration_tests.rs` (NEW)
  - Test realistic scenarios: Vim navigation, lock persistence, multi-device
  - Load test .krx files from tests/fixtures/
  - Verify output matches expected for complex workflows
  - Purpose: Validate complete system behavior
  - _Leverage: load_config, EventProcessor, mock platform_
  - _Requirements: 5.4, Testing Strategy: End-to-End Testing_
  - _Prompt: Role: E2E Test Engineer with focus on realistic user scenarios | Task: Create end-to-end integration tests using realistic configurations (Vim navigation layer, lock toggle persistence, multi-device) following requirement 5.4 and Testing Strategy E2E section | Restrictions: Must create or load realistic test configs (e.g., CapsLock→MD_00, when(MD_00){H→Left, J→Down, K→Up, L→Right}), create event sequences simulating user input (Press(CapsLock), Press(H), Release(H), Release(CapsLock)), process through EventProcessor, verify output matches expected navigation keys, test lock persistence (toggle on, use, toggle off, use), test multiple devices with different configs (requires multiple DeviceConfig and EventProcessor instances), use descriptive test names | Success: Test test_vim_navigation_layer activates CapsLock, presses H, outputs Left, test_lock_persistence toggles ScrollLock, verifies subsequent keys use conditional mapping, then toggles off and verifies passthrough, test_multi_device processes events for 2 different devices with independent state, all tests pass_

- [x] 25. Add fuzzing infrastructure (optional but recommended)
  - File: `keyrx_core/fuzz/fuzz_targets/fuzz_runtime.rs`
  - Use `cargo-fuzz` to generate random events and configs
  - Verify: no panics, no infinite loops, no crashes
  - Purpose: Discover edge cases and undefined behavior
  - _Leverage: cargo-fuzz (libFuzzer)_
  - _Requirements: Reliability: No Panics, Security: No Panics_
  - _Prompt: Role: Security Engineer with fuzzing expertise and Rust safety focus | Task: Set up cargo-fuzz infrastructure for keyrx_core runtime components, create fuzz target that generates random KeyEvents and DeviceConfigs, verifies no panics or crashes, following Reliability and Security requirements | Restrictions: Must use `cargo install cargo-fuzz` and `cargo fuzz init` to set up, create fuzz_targets/runtime_fuzzing.rs, use arbitrary crate to derive Arbitrary for KeyEvent if needed, fuzz process_event with random inputs, run for at least 60 seconds (`cargo fuzz run runtime_fuzzing -- -max_total_time=60`), document any crashes found and fixed, add README.md in fuzz/ explaining how to run fuzz tests | Success: Fuzz infrastructure compiles, fuzz_targets/runtime_fuzzing.rs generates random events, fuzzer runs without crashes for 60 seconds, README.md documents fuzzing setup and usage, any crashes discovered are fixed and documented_

- [x] 26. Final integration, benchmarking, and documentation
  - Files: All modified files
  - Run full test suite (unit + integration + property + benchmarks)
  - Verify benchmarks meet performance targets
  - Update CHANGELOG.md with new runtime features
  - Write examples/runtime_example.rs demonstrating usage
  - Purpose: Polish and finalize Phase 2 implementation
  - _Leverage: Existing CI tools (cargo test, cargo bench)_
  - _Requirements: All_
  - _Prompt: Role: Senior Rust Developer with focus on code quality and documentation | Task: Complete final integration by running full test suite, verifying benchmarks meet targets, updating CHANGELOG, and creating usage examples, ensuring all requirements are met | Restrictions: Must run `cargo test --workspace` and verify all tests pass, run `cargo bench` and verify performance targets met (lookup <100μs, state update <10μs, process_event <1ms), update CHANGELOG.md with sections for Added (runtime module, platform traits, mock platform, EventProcessor, etc.), run `cargo clippy --workspace -- -D warnings` and fix all warnings, run `cargo fmt --all`, create examples/runtime_example.rs showing how to load config and process events with mock platform, ensure all public APIs have doc comments, verify keyrx_core and keyrx_daemon compile independently | Success: All 50+ tests pass (unit + integration + property), benchmarks meet performance targets (documented in CHANGELOG), clippy produces zero warnings, all code formatted, examples/runtime_example.rs compiles and runs successfully, CHANGELOG.md comprehensively documents Phase 2 features, both crates compile without errors_
