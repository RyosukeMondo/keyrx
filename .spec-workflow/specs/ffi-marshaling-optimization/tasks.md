# Tasks Document

## Phase 1: Core Types

- [ ] 1. Create FfiMarshaler trait
  - File: `core/src/ffi/marshal/traits.rs`
  - Define trait with CRepr associated type
  - Add streaming support
  - Purpose: Unified marshaling interface
  - _Leverage: Trait patterns_
  - _Requirements: 1.1, 1.2_
  - _Prompt: Implement the task for spec ffi-marshaling-optimization, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Developer creating traits | Task: Create FfiMarshaler trait with CRepr | Restrictions: Generic, streaming-aware, type-safe | _Leverage: Trait patterns | Success: Trait defines marshaling contract | After completion: Mark task [-] as in-progress before starting, use log-implementation tool to record artifacts, then mark [x] when complete_

- [ ] 2. Create FfiResult type
  - File: `core/src/ffi/marshal/result.rs`
  - C-compatible result representation
  - Error pointer management
  - Purpose: Safe result passing
  - _Leverage: C ABI_
  - _Requirements: 1.3, 2.1_
  - _Prompt: Implement the task for spec ffi-marshaling-optimization, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Developer creating types | Task: Create FfiResult with C ABI | Restrictions: repr(C), memory-safe, error support | _Leverage: C ABI | Success: Results cross FFI safely | After completion: Mark task [-] as in-progress before starting, use log-implementation tool to record artifacts, then mark [x] when complete_

- [ ] 3. Create FfiError type
  - File: `core/src/ffi/marshal/error.rs`
  - Error with code, message, hint, context
  - C-compatible allocation
  - Purpose: Error marshaling
  - _Leverage: Error patterns_
  - _Requirements: 2.1, 2.2, 2.3, 2.4_
  - _Prompt: Implement the task for spec ffi-marshaling-optimization, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Developer creating errors | Task: Create FfiError with C-compatible layout | Restrictions: Preserve codes, context, safe free | _Leverage: Error patterns | Success: Errors cross FFI with details | After completion: Mark task [-] as in-progress before starting, use log-implementation tool to record artifacts, then mark [x] when complete_

## Phase 2: Common Marshalers

- [ ] 4. Implement primitive marshalers
  - File: `core/src/ffi/marshal/impls/primitives.rs`
  - u8, u16, u32, u64, bool, f32, f64
  - Direct C representation
  - Purpose: Basic type support
  - _Leverage: FfiMarshaler trait_
  - _Requirements: 1.1_
  - _Prompt: Implement the task for spec ffi-marshaling-optimization, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Developer implementing traits | Task: Implement FfiMarshaler for primitives | Restrictions: Zero-copy where possible | _Leverage: FfiMarshaler | Success: Primitives marshal efficiently | After completion: Mark task [-] as in-progress before starting, use log-implementation tool to record artifacts, then mark [x] when complete_

- [ ] 5. Implement string marshaler
  - File: `core/src/ffi/marshal/impls/string.rs`
  - String and &str support
  - Null-terminated C strings
  - Purpose: String support
  - _Leverage: FfiMarshaler trait_
  - _Requirements: 1.1_
  - _Prompt: Implement the task for spec ffi-marshaling-optimization, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Developer implementing marshalers | Task: Implement FfiMarshaler for strings | Restrictions: Null-terminated, UTF-8 safe | _Leverage: FfiMarshaler | Success: Strings marshal safely | After completion: Mark task [-] as in-progress before starting, use log-implementation tool to record artifacts, then mark [x] when complete_

- [ ] 6. Implement array/Vec marshaler
  - File: `core/src/ffi/marshal/impls/array.rs`
  - Generic Vec<T> support
  - FfiArray C representation
  - Purpose: Collection support
  - _Leverage: FfiMarshaler trait_
  - _Requirements: 1.1, 3.3_
  - _Prompt: Implement the task for spec ffi-marshaling-optimization, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Developer implementing marshalers | Task: Implement FfiMarshaler for Vec<T> | Restrictions: Length-prefixed, batch encoding | _Leverage: FfiMarshaler | Success: Arrays marshal efficiently | After completion: Mark task [-] as in-progress before starting, use log-implementation tool to record artifacts, then mark [x] when complete_

- [ ] 7. Implement JSON marshaler
  - File: `core/src/ffi/marshal/impls/json.rs`
  - JsonWrapper<T> for complex types
  - Fallback for non-C-compatible types
  - Purpose: Complex type support
  - _Leverage: serde_json_
  - _Requirements: 1.4_
  - _Prompt: Implement the task for spec ffi-marshaling-optimization, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Developer implementing marshalers | Task: Implement JSON marshaler for complex types | Restrictions: Efficient JSON, streaming for large | _Leverage: serde | Success: Complex types marshal via JSON | After completion: Mark task [-] as in-progress before starting, use log-implementation tool to record artifacts, then mark [x] when complete_

## Phase 3: Large Data Support

- [ ] 8. Create FfiStreamMarshaler trait
  - File: `core/src/ffi/marshal/stream.rs`
  - Chunked transfer for large data
  - Iterator-based API
  - Purpose: Large data transfer
  - _Leverage: Trait patterns_
  - _Requirements: 3.1, 3.4_
  - _Prompt: Implement the task for spec ffi-marshaling-optimization, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Developer creating traits | Task: Create FfiStreamMarshaler for chunked transfer | Restrictions: Fixed chunk size, resumable | _Leverage: Trait patterns | Success: Large data streams efficiently | After completion: Mark task [-] as in-progress before starting, use log-implementation tool to record artifacts, then mark [x] when complete_

- [ ] 9. Implement streaming for session data
  - File: `core/src/ffi/marshal/impls/session.rs`
  - Stream recording/replay data
  - Efficient chunking
  - Purpose: Session data transfer
  - _Leverage: FfiStreamMarshaler_
  - _Requirements: 3.1, 3.2_
  - _Prompt: Implement the task for spec ffi-marshaling-optimization, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Developer implementing streaming | Task: Implement streaming for session data | Restrictions: Efficient chunks, minimal copies | _Leverage: FfiStreamMarshaler | Success: Sessions transfer efficiently | After completion: Mark task [-] as in-progress before starting, use log-implementation tool to record artifacts, then mark [x] when complete_

## Phase 4: Callback System

- [ ] 10. Create FfiCallback trait
  - File: `core/src/ffi/marshal/callback.rs`
  - Unified callback interface
  - Type-safe invocation
  - Purpose: Callback abstraction
  - _Leverage: Trait patterns_
  - _Requirements: 4.1, 4.3_
  - _Prompt: Implement the task for spec ffi-marshaling-optimization, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Developer creating traits | Task: Create FfiCallback trait | Restrictions: Type-safe, async-compatible | _Leverage: Trait patterns | Success: Callbacks have unified interface | After completion: Mark task [-] as in-progress before starting, use log-implementation tool to record artifacts, then mark [x] when complete_

- [ ] 11. Create CallbackRegistry
  - File: `core/src/ffi/marshal/callback.rs`
  - Register/unregister callbacks
  - Thread-safe invocation
  - Purpose: Callback management
  - _Leverage: DashMap_
  - _Requirements: 4.1, 4.2_
  - _Prompt: Implement the task for spec ffi-marshaling-optimization, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Developer creating registry | Task: Create CallbackRegistry for FFI callbacks | Restrictions: Thread-safe, ID-based | _Leverage: DashMap | Success: Callbacks managed centrally | After completion: Mark task [-] as in-progress before starting, use log-implementation tool to record artifacts, then mark [x] when complete_

- [ ] 12. Add callback error handling
  - File: `core/src/ffi/marshal/callback.rs`
  - Fallback on callback failure
  - Error reporting
  - Purpose: Robust callbacks
  - _Leverage: FfiError_
  - _Requirements: 4.4_
  - _Prompt: Implement the task for spec ffi-marshaling-optimization, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Developer adding error handling | Task: Add fallback and error handling for callbacks | Restrictions: Never panic, log failures | _Leverage: FfiError | Success: Callback failures handled gracefully | After completion: Mark task [-] as in-progress before starting, use log-implementation tool to record artifacts, then mark [x] when complete_

## Phase 5: Derive Macro

- [ ] 13. Create FfiMarshaler derive macro
  - File: `core-macros/src/ffi_marshaler.rs`
  - Auto-generate implementations
  - Strategy attribute (json/c_struct/auto)
  - Purpose: Reduce boilerplate
  - _Leverage: proc-macro_
  - _Requirements: 1.2_
  - _Prompt: Implement the task for spec ffi-marshaling-optimization, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Macro Developer | Task: Create FfiMarshaler derive macro | Restrictions: Strategy selection, correct generation | _Leverage: proc-macro | Success: Derive generates correct impl | After completion: Mark task [-] as in-progress before starting, use log-implementation tool to record artifacts, then mark [x] when complete_

## Phase 6: Migration

- [ ] 14. Migrate exports_engine.rs
  - File: `core/src/ffi/exports_engine.rs`
  - Use FfiMarshaler for all exports
  - Consolidate error handling
  - Purpose: Engine FFI migration
  - _Leverage: FfiMarshaler_
  - _Requirements: 1.1, 2.1_
  - _Prompt: Implement the task for spec ffi-marshaling-optimization, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Developer migrating code | Task: Migrate exports_engine.rs to FfiMarshaler | Restrictions: Same API surface, better internals | _Leverage: FfiMarshaler | Success: Engine exports use marshaler | After completion: Mark task [-] as in-progress before starting, use log-implementation tool to record artifacts, then mark [x] when complete_

- [ ] 15. Migrate exports_discovery.rs
  - File: `core/src/ffi/exports_discovery.rs`
  - Use FfiMarshaler for device data
  - Consolidate callbacks
  - Purpose: Discovery FFI migration
  - _Leverage: FfiMarshaler_
  - _Requirements: 1.1, 4.1_
  - _Prompt: Implement the task for spec ffi-marshaling-optimization, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Developer migrating code | Task: Migrate exports_discovery.rs to FfiMarshaler | Restrictions: Same API, unified callbacks | _Leverage: FfiMarshaler | Success: Discovery exports use marshaler | After completion: Mark task [-] as in-progress before starting, use log-implementation tool to record artifacts, then mark [x] when complete_

- [ ] 16. Migrate remaining exports
  - Files: `core/src/ffi/exports_*.rs`
  - Use FfiMarshaler throughout
  - Remove duplicate marshaling code
  - Purpose: Complete migration
  - _Leverage: FfiMarshaler_
  - _Requirements: 1.1_
  - _Prompt: Implement the task for spec ffi-marshaling-optimization, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Developer migrating code | Task: Migrate remaining FFI exports to FfiMarshaler | Restrictions: All exports, consistent patterns | _Leverage: FfiMarshaler | Success: All FFI uses marshaler | After completion: Mark task [-] as in-progress before starting, use log-implementation tool to record artifacts, then mark [x] when complete_

- [ ] 17. Add FFI benchmarks
  - File: `core/benches/ffi_bench.rs`
  - Benchmark marshaling overhead
  - Compare JSON vs C struct
  - Purpose: Performance verification
  - _Leverage: criterion_
  - _Requirements: Non-functional (performance)_
  - _Prompt: Implement the task for spec ffi-marshaling-optimization, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Benchmark Developer | Task: Create FFI marshaling benchmarks | Restrictions: Compare strategies, realistic data | _Leverage: criterion | Success: Performance targets met | After completion: Mark task [-] as in-progress before starting, use log-implementation tool to record artifacts, then mark [x] when complete_
