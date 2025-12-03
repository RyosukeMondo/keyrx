# Tasks Document

## Phase 1: Foundation

- [ ] 1. Create FfiError and FfiResult types
  - File: `core/src/ffi/error.rs`
  - Define `FfiError` struct with code, message, details fields
  - Implement `FfiResult<T>` type alias and serialization
  - Add helper constructors: `invalid_input`, `internal`, `not_found`
  - Purpose: Standardized error handling across all FFI exports
  - _Leverage: Existing error patterns in `exports_discovery.rs`_
  - _Requirements: 4.1, 4.2_
  - _Prompt: Implement the task for spec ffi-architecture-overhaul, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Developer specializing in error handling and FFI | Task: Create FfiError and FfiResult types in core/src/ffi/error.rs following requirements 4.1 and 4.2, with JSON serialization producing "ok:{...}" or "error:{code, message, details}" format | Restrictions: Do not modify existing exports yet, maintain backward compatibility, use serde for serialization | _Leverage: Review exports_discovery.rs for current error patterns | Success: Types compile, serialize correctly to expected JSON format, unit tests pass for all error constructors | After completion: Mark task [-] as in-progress before starting, use log-implementation tool to record artifacts, then mark [x] when complete_

- [ ] 2. Create FfiContext handle-based state management
  - File: `core/src/ffi/context.rs`
  - Define `FfiContext` struct with handle, domain state storage
  - Implement `new()`, `get_domain<T>()`, `get_domain_mut<T>()`
  - Add handle registry for safe pointer management
  - Purpose: Replace global OnceLock statics with instance-scoped state
  - _Leverage: Current state patterns in `exports_discovery.rs` (lines 68-78)_
  - _Requirements: 3.1, 3.2, 3.3_
  - _Prompt: Implement the task for spec ffi-architecture-overhaul, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Developer with FFI and memory safety expertise | Task: Create FfiContext in core/src/ffi/context.rs implementing handle-based state per requirements 3.1-3.3, replacing global statics pattern | Restrictions: Must be thread-safe, no global mutable state, use Arc<RwLock> for shared access | _Leverage: Study DISCOVERY_SESSION static in exports_discovery.rs:68-78 for current pattern | Success: Context can store/retrieve typed domain state, handles are unique, tests run in parallel without interference | After completion: Mark task [-] as in-progress before starting, use log-implementation tool to record artifacts, then mark [x] when complete_

- [ ] 3. Create EventRegistry for unified callbacks
  - File: `core/src/ffi/events.rs`
  - Define `EventType` enum covering all callback categories
  - Implement `EventRegistry` with register/invoke methods
  - Add callback type definition: `extern "C" fn(*const u8, usize)`
  - Purpose: Single callback registration system replacing per-domain functions
  - _Leverage: `CallbackRegistry` in `ffi/callbacks.rs`_
  - _Requirements: 2.1, 2.2, 2.3, 2.4_
  - _Prompt: Implement the task for spec ffi-architecture-overhaul, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Developer specializing in callback systems | Task: Create EventRegistry in core/src/ffi/events.rs implementing unified callback management per requirements 2.1-2.4 | Restrictions: Must handle null callbacks gracefully, thread-safe invocation, no memory leaks | _Leverage: Study CallbackRegistry in ffi/callbacks.rs for current pattern | Success: Single registration point for all event types, callbacks invoked with JSON payloads, replaces 3+ individual callback functions | After completion: Mark task [-] as in-progress before starting, use log-implementation tool to record artifacts, then mark [x] when complete_

- [ ] 4. Create FfiExportable trait definition
  - File: `core/src/ffi/traits.rs`
  - Define `FfiExportable` trait with DOMAIN const, init(), cleanup()
  - Add documentation with usage examples
  - Create `FfiDomain` marker for domain type storage
  - Purpose: Contract for FFI-exportable domain modules
  - _Leverage: `InputSource` trait pattern from `core/src/traits/`_
  - _Requirements: 1.1, 1.4_
  - _Prompt: Implement the task for spec ffi-architecture-overhaul, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Developer with trait design expertise | Task: Create FfiExportable trait in core/src/ffi/traits.rs following requirements 1.1 and 1.4 | Restrictions: Trait must be object-safe where possible, clear documentation required, follow existing trait patterns | _Leverage: Review InputSource trait in core/src/traits/ for design patterns | Success: Trait compiles, can be implemented by domain modules, documentation is complete | After completion: Mark task [-] as in-progress before starting, use log-implementation tool to record artifacts, then mark [x] when complete_

## Phase 2: Procedural Macro

- [ ] 5. Set up procedural macro crate
  - Files: `core/ffi-macros/Cargo.toml`, `core/ffi-macros/src/lib.rs`
  - Create new crate `keyrx-ffi-macros` as proc-macro
  - Add dependencies: syn, quote, proc-macro2
  - Wire up to main Cargo.toml as workspace member
  - Purpose: Foundation for #[ffi_export] macro
  - _Leverage: Rust proc-macro best practices_
  - _Requirements: 1.1_
  - _Prompt: Implement the task for spec ffi-architecture-overhaul, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Developer with procedural macro experience | Task: Set up keyrx-ffi-macros crate in core/ffi-macros/ with proper workspace configuration | Restrictions: Must be proc-macro crate type, use stable Rust features only, follow workspace conventions | _Leverage: Standard Rust proc-macro crate structure | Success: Crate compiles, is recognized by workspace, can define proc-macro attributes | After completion: Mark task [-] as in-progress before starting, use log-implementation tool to record artifacts, then mark [x] when complete_

- [ ] 6. Implement #[ffi_export] attribute macro
  - File: `core/ffi-macros/src/lib.rs`
  - Parse method signatures with syn
  - Generate C-ABI wrapper functions with error handling
  - Handle string parameters (null check, UTF-8 validation)
  - Purpose: Auto-generate FFI wrappers from Rust methods
  - _Leverage: Current wrapper patterns in `exports_discovery.rs:114-270`_
  - _Requirements: 1.1, 1.2, 1.3_
  - _Prompt: Implement the task for spec ffi-architecture-overhaul, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Developer with deep proc-macro expertise | Task: Implement #[ffi_export] macro generating C-ABI wrappers per requirements 1.1-1.3 | Restrictions: Generated code must handle all error cases, match existing wrapper quality, compile without warnings | _Leverage: Study keyrx_start_discovery in exports_discovery.rs:114-270 for target output pattern | Success: Macro generates correct wrappers, null checks work, UTF-8 validation works, JSON serialization works | After completion: Mark task [-] as in-progress before starting, use log-implementation tool to record artifacts, then mark [x] when complete_

- [ ] 7. Add panic catching to generated wrappers
  - File: `core/ffi-macros/src/lib.rs`
  - Wrap method calls in `std::panic::catch_unwind`
  - Convert panics to FfiError with INTERNAL_ERROR code
  - Log panic details for debugging
  - Purpose: Prevent panics from crossing FFI boundary
  - _Leverage: catch_unwind patterns_
  - _Requirements: 4.2 (error handling)_
  - _Prompt: Implement the task for spec ffi-architecture-overhaul, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Developer with FFI safety expertise | Task: Add panic catching to #[ffi_export] generated wrappers using catch_unwind | Restrictions: Must catch all panics, convert to error responses, log for debugging, not affect performance significantly | _Leverage: std::panic::catch_unwind documentation | Success: Panics in domain code don't crash FFI caller, error response returned instead, panic logged | After completion: Mark task [-] as in-progress before starting, use log-implementation tool to record artifacts, then mark [x] when complete_

## Phase 3: First Domain Migration (Discovery)

- [ ] 8. Create DiscoveryFfi implementing FfiExportable
  - File: `core/src/ffi/domains/discovery.rs`
  - Implement FfiExportable trait for DiscoveryFfi
  - Move state from global statics to domain struct
  - Implement init() and cleanup() methods
  - Purpose: First domain using new pattern
  - _Leverage: Current `exports_discovery.rs` implementation_
  - _Requirements: 5.1, 3.1_
  - _Prompt: Implement the task for spec ffi-architecture-overhaul, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Developer migrating FFI code | Task: Create DiscoveryFfi in core/src/ffi/domains/discovery.rs implementing FfiExportable trait | Restrictions: Must maintain same functionality as exports_discovery.rs, no global statics, state in struct | _Leverage: exports_discovery.rs as source, FfiExportable trait from task 4 | Success: DiscoveryFfi has same capabilities as current exports, uses instance state, passes existing tests | After completion: Mark task [-] as in-progress before starting, use log-implementation tool to record artifacts, then mark [x] when complete_

- [ ] 9. Apply #[ffi_export] to Discovery methods
  - File: `core/src/ffi/domains/discovery.rs`
  - Add #[ffi_export] to start_discovery, process_event, cancel, get_progress
  - Remove manual C-ABI wrapper code
  - Verify generated exports match existing signatures
  - Purpose: Validate macro with real domain
  - _Leverage: Existing FFI function signatures_
  - _Requirements: 1.1, 1.2, 1.3_
  - _Prompt: Implement the task for spec ffi-architecture-overhaul, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Developer applying new patterns | Task: Apply #[ffi_export] macro to DiscoveryFfi methods, removing manual wrapper code | Restrictions: Generated exports must match existing function signatures exactly, all tests must pass | _Leverage: Current exports_discovery.rs function signatures | Success: Discovery FFI works with macro-generated wrappers, code reduced by ~200 lines, tests pass | After completion: Mark task [-] as in-progress before starting, use log-implementation tool to record artifacts, then mark [x] when complete_

- [ ] 10. Migrate Discovery callbacks to EventRegistry
  - File: `core/src/ffi/domains/discovery.rs`
  - Replace keyrx_on_discovery_progress/duplicate/summary with EventRegistry
  - Add DiscoveryProgress, DiscoveryDuplicate, DiscoverySummary event types
  - Update callback invocation to use EventRegistry
  - Purpose: Unified callback system for Discovery
  - _Leverage: `callbacks.rs` and `EventRegistry` from task 3_
  - _Requirements: 2.1, 2.2, 2.3_
  - _Prompt: Implement the task for spec ffi-architecture-overhaul, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Developer refactoring callbacks | Task: Migrate Discovery callbacks to EventRegistry, replacing 3 separate callback functions | Restrictions: Callback behavior must remain identical, existing Flutter code must work with shim | _Leverage: EventRegistry from task 3, current callback pattern in callbacks.rs | Success: Discovery uses EventRegistry, backward-compatible shims exist, Flutter works unchanged | After completion: Mark task [-] as in-progress before starting, use log-implementation tool to record artifacts, then mark [x] when complete_

- [ ] 11. Create backward-compatible shims for Discovery
  - File: `core/src/ffi/compat/discovery_compat.rs`
  - Create thin wrappers: keyrx_on_discovery_progress → EventRegistry.register
  - Mark as #[deprecated] with migration guidance
  - Keep existing function signatures exactly
  - Purpose: Allow incremental Flutter migration
  - _Leverage: Existing exports_discovery.rs functions_
  - _Requirements: 5.3 (deprecation warnings)_
  - _Prompt: Implement the task for spec ffi-architecture-overhaul, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Developer creating compatibility layer | Task: Create backward-compatible shims in core/src/ffi/compat/discovery_compat.rs for existing callback functions | Restrictions: Must not change existing function signatures, must emit deprecation warnings, must forward to EventRegistry | _Leverage: Current exports_discovery.rs callback functions | Success: Existing Flutter code works unchanged, deprecation warnings appear, new code can use EventRegistry directly | After completion: Mark task [-] as in-progress before starting, use log-implementation tool to record artifacts, then mark [x] when complete_

## Phase 4: Additional Domain Migrations

- [ ] 12. Migrate ValidationFfi
  - File: `core/src/ffi/domains/validation.rs`
  - Implement FfiExportable for ValidationFfi
  - Apply #[ffi_export] to validation functions
  - Migrate validation callbacks to EventRegistry
  - Purpose: Validate pattern with second domain
  - _Leverage: `exports_validation.rs`, DiscoveryFfi pattern_
  - _Requirements: 5.1, 1.1_
  - _Prompt: Implement the task for spec ffi-architecture-overhaul, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Developer migrating FFI code | Task: Migrate exports_validation.rs to ValidationFfi following DiscoveryFfi pattern | Restrictions: Maintain same functionality, use #[ffi_export] macro, migrate to EventRegistry | _Leverage: exports_validation.rs as source, DiscoveryFfi as pattern | Success: ValidationFfi works with new architecture, code reduced, tests pass | After completion: Mark task [-] as in-progress before starting, use log-implementation tool to record artifacts, then mark [x] when complete_

- [ ] 13. Migrate EngineFfi
  - File: `core/src/ffi/domains/engine.rs`
  - Implement FfiExportable for EngineFfi
  - Apply #[ffi_export] to engine functions
  - Migrate engine callbacks to EventRegistry
  - Purpose: Core engine FFI migration
  - _Leverage: `exports_engine.rs`, established pattern_
  - _Requirements: 5.1, 1.1_
  - _Prompt: Implement the task for spec ffi-architecture-overhaul, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Developer migrating FFI code | Task: Migrate exports_engine.rs to EngineFfi following established pattern | Restrictions: Maintain same functionality, use #[ffi_export] macro, handle engine state carefully | _Leverage: exports_engine.rs as source, DiscoveryFfi as pattern | Success: EngineFfi works with new architecture, state management correct, tests pass | After completion: Mark task [-] as in-progress before starting, use log-implementation tool to record artifacts, then mark [x] when complete_

- [ ] 14. Migrate remaining domains (Device, Testing, Analysis, Diagnostics, Script)
  - Files: `core/src/ffi/domains/{device,testing,analysis,diagnostics,script}.rs`
  - Apply same pattern to all remaining exports_*.rs files
  - Consolidate smaller domains if < 3 functions
  - Create compat shims as needed
  - Purpose: Complete FFI migration
  - _Leverage: Established pattern, remaining exports_*.rs files_
  - _Requirements: 5.1, 5.2_
  - _Prompt: Implement the task for spec ffi-architecture-overhaul, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Developer completing FFI migration | Task: Migrate remaining exports_*.rs files to new domain pattern | Restrictions: Apply consistent pattern, consolidate small domains, maintain all functionality | _Leverage: All remaining exports_*.rs files, established pattern | Success: All FFI domains migrated, consistent architecture, all tests pass | After completion: Mark task [-] as in-progress before starting, use log-implementation tool to record artifacts, then mark [x] when complete_

## Phase 5: Dart Binding Generation

- [ ] 15. Create Dart binding generator script
  - File: `scripts/generate_dart_bindings.py`
  - Parse Rust FFI exports (from macro annotations or cbindgen output)
  - Generate Dart FFI bindings with type conversions
  - Output to `ui/lib/ffi/generated/`
  - Purpose: Automated Dart binding synchronization
  - _Leverage: ffigen or custom parsing_
  - _Requirements: 6.1, 6.2, 6.3_
  - _Prompt: Implement the task for spec ffi-architecture-overhaul, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Python/Dart Developer with FFI experience | Task: Create binding generator script producing Dart FFI bindings from Rust exports | Restrictions: Must generate type-safe bindings, handle all parameter types, output to ui/lib/ffi/generated/ | _Leverage: dart:ffi documentation, current KeyrxBridge patterns | Success: Script generates correct Dart bindings, types match Rust, integration works | After completion: Mark task [-] as in-progress before starting, use log-implementation tool to record artifacts, then mark [x] when complete_

- [ ] 16. Integrate binding generation into build
  - Files: `core/Cargo.toml`, `ui/pubspec.yaml`, `scripts/build.sh`
  - Add build.rs or Makefile step to run generator
  - Add Flutter pre-build hook to verify bindings
  - Fail build if bindings out of sync
  - Purpose: Automatic binding synchronization
  - _Leverage: Cargo build scripts, Flutter build hooks_
  - _Requirements: 6.4_
  - _Prompt: Implement the task for spec ffi-architecture-overhaul, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Build Engineer with Rust/Flutter expertise | Task: Integrate binding generation into build pipeline with sync verification | Restrictions: Must fail build on mismatch, integrate with existing build, minimal build time impact | _Leverage: Cargo build.rs, Flutter build hooks | Success: Build runs generator automatically, fails on mismatch, CI catches binding drift | After completion: Mark task [-] as in-progress before starting, use log-implementation tool to record artifacts, then mark [x] when complete_

- [ ] 17. Update KeyrxBridge to use generated bindings
  - File: `ui/lib/ffi/bridge.dart`
  - Replace manual FFI function lookups with generated bindings
  - Add unified callback registration via EventRegistry
  - Maintain backward compatibility during transition
  - Purpose: Flutter integration with new FFI
  - _Leverage: Current KeyrxBridge implementation, generated bindings_
  - _Requirements: 2.1, 6.1_
  - _Prompt: Implement the task for spec ffi-architecture-overhaul, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Flutter Developer with FFI expertise | Task: Update KeyrxBridge to use generated bindings and unified callback registration | Restrictions: Maintain backward compatibility, all existing functionality must work, gradual migration | _Leverage: Current bridge.dart, generated bindings from task 15 | Success: KeyrxBridge uses generated bindings, callbacks work with EventRegistry, tests pass | After completion: Mark task [-] as in-progress before starting, use log-implementation tool to record artifacts, then mark [x] when complete_

## Phase 6: Testing & Cleanup

- [ ] 18. Add parallel FFI tests
  - File: `core/src/ffi/tests/parallel_tests.rs`
  - Create tests that run FFI operations in parallel
  - Verify no state interference between contexts
  - Use cargo nextest for true parallelism
  - Purpose: Validate isolated state management
  - _Leverage: FfiContext, existing FFI tests_
  - _Requirements: 3.4_
  - _Prompt: Implement the task for spec ffi-architecture-overhaul, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Test Developer | Task: Create parallel FFI tests verifying isolated state management | Restrictions: Tests must actually run in parallel, verify no state leakage, use nextest | _Leverage: FfiContext from task 2, existing FFI tests | Success: Parallel tests pass with nextest, no state interference, CI runs them parallel | After completion: Mark task [-] as in-progress before starting, use log-implementation tool to record artifacts, then mark [x] when complete_

- [ ] 19. Add FFI fuzz tests
  - File: `core/src/ffi/tests/fuzz_tests.rs`
  - Use proptest to generate random FFI inputs
  - Verify no panics escape FFI boundary
  - Test edge cases: empty strings, max lengths, null bytes
  - Purpose: Robustness testing for FFI layer
  - _Leverage: proptest, FfiError_
  - _Requirements: Non-functional (reliability)_
  - _Prompt: Implement the task for spec ffi-architecture-overhaul, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Security/Test Developer | Task: Create FFI fuzz tests using proptest for robustness validation | Restrictions: Must catch all panics, test realistic edge cases, integrate with CI | _Leverage: proptest documentation, FfiError from task 1 | Success: Fuzz tests run, no panics escape, edge cases handled gracefully | After completion: Mark task [-] as in-progress before starting, use log-implementation tool to record artifacts, then mark [x] when complete_

- [ ] 20. Remove deprecated exports and clean up
  - Files: Delete `core/src/ffi/exports_*.rs` (old files)
  - Remove backward-compat shims after Flutter migration
  - Update mod.rs to export new structure
  - Update documentation
  - Purpose: Complete migration, reduce code
  - _Leverage: New domain structure_
  - _Requirements: 5.3_
  - _Prompt: Implement the task for spec ffi-architecture-overhaul, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Developer completing cleanup | Task: Remove deprecated FFI exports after Flutter migration complete | Restrictions: Only remove after all Flutter code migrated, update all imports, update docs | _Leverage: New ffi/domains/ structure | Success: Old exports_*.rs deleted, code compiles, tests pass, docs updated | After completion: Mark task [-] as in-progress before starting, use log-implementation tool to record artifacts, then mark [x] when complete_

- [ ] 21. Update FFI documentation
  - File: `docs/ffi-architecture.md`
  - Document new trait-based architecture
  - Add migration guide for adding new FFI exports
  - Include examples for common patterns
  - Purpose: Developer documentation
  - _Leverage: Implementation from previous tasks_
  - _Requirements: Non-functional (usability)_
  - _Prompt: Implement the task for spec ffi-architecture-overhaul, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Technical Writer with Rust expertise | Task: Create comprehensive FFI architecture documentation | Restrictions: Cover all new patterns, include examples, migration guide for new exports | _Leverage: Implementation details from all previous tasks | Success: Documentation is complete, examples work, developers can add new exports easily | After completion: Mark task [-] as in-progress before starting, use log-implementation tool to record artifacts, then mark [x] when complete_
