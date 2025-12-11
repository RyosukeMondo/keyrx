# Tasks Document

## Implementation Tasks

### Phase 1: Runtime Library

- [x] 1. Create keyrx_ffi_runtime crate
  - Files: `core/keyrx_ffi_runtime/Cargo.toml`, `core/keyrx_ffi_runtime/src/lib.rs`
  - Create new library crate for FFI runtime helpers
  - Add dependencies: serde_json, libc
  - Purpose: Provide reusable FFI utilities for generated code
  - _Leverage: Existing FFI patterns from core/src/ffi_
  - _Requirements: REQ-2_
  - _Prompt: Implement the task for spec rust-ffi-codegen, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Developer specializing in crate architecture and FFI | Task: Create new keyrx_ffi_runtime library crate with proper Cargo.toml configuration following requirement REQ-2 | Restrictions: Must use stable Rust, add appropriate dependencies, follow crate naming conventions | Success: Crate compiles successfully, dependencies are correctly configured, follows Rust 2021 edition | Instructions: Mark this task as in-progress in tasks.md before starting. After completion, log the implementation with detailed artifacts using the log-implementation tool (include crate structure, dependencies), then mark the task as completed in tasks.md_

- [x] 2. Implement C string parsing helpers
  - File: `core/keyrx_ffi_runtime/src/string.rs`
  - Implement `parse_c_string(ptr: *const c_char, name: &str) -> Result<String, String>`
  - Add null pointer checking and UTF-8 validation
  - Purpose: Safely convert C strings to Rust strings
  - _Leverage: Existing parse_c_string patterns from core/src/ffi/exports.rs_
  - _Requirements: REQ-2_
  - _Prompt: Implement the task for spec rust-ffi-codegen, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Developer with FFI and string handling expertise | Task: Implement safe C string parsing with null checks and UTF-8 validation following requirement REQ-2 | Restrictions: Must handle null pointers safely, provide descriptive errors, function under 50 lines | Success: Parses valid C strings correctly, returns errors for null/invalid UTF-8, safe and tested | Instructions: Mark this task as in-progress in tasks.md before starting. After completion, log the implementation with detailed artifacts using the log-implementation tool (include function signature, safety checks), then mark the task as completed in tasks.md_

- [x] 3. Implement JSON serialization helpers
  - File: `core/keyrx_ffi_runtime/src/json.rs`
  - Implement `serialize_to_c_string<T: Serialize>(value: &T) -> Result<*const c_char, String>`
  - Handle serialization errors gracefully
  - Purpose: Convert Rust types to C strings via JSON
  - _Leverage: serde_json for serialization_
  - _Requirements: REQ-2_
  - _Prompt: Implement the task for spec rust-ffi-codegen, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Developer with expertise in serialization and FFI | Task: Implement JSON to C string serialization following requirement REQ-2, handling errors and memory allocation | Restrictions: Must allocate C string correctly, handle serialization failures, function under 50 lines | Success: Serializes Rust types to C strings, proper memory management, error handling works | Instructions: Mark this task as in-progress in tasks.md before starting. After completion, log the implementation with detailed artifacts using the log-implementation tool (include function signature, memory management), then mark the task as completed in tasks.md_

- [x] 4. Implement panic catching wrapper
  - File: `core/keyrx_ffi_runtime/src/panic.rs`
  - Implement `handle_panic<F>(f: F) -> Result<F::Output, String>` using catch_unwind
  - Convert panic payload to error message
  - Purpose: Catch panics at FFI boundary
  - _Leverage: std::panic::catch_unwind_
  - _Requirements: REQ-3_
  - _Prompt: Implement the task for spec rust-ffi-codegen, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Developer with expertise in panic handling and FFI safety | Task: Implement panic catching wrapper following requirement REQ-3, using catch_unwind to prevent panics crossing FFI boundary | Restrictions: Must handle all panic types, extract meaningful error messages, function under 50 lines | Success: Catches all panics, converts to error strings, prevents undefined behavior at FFI boundary | Instructions: Mark this task as in-progress in tasks.md before starting. After completion, log the implementation with detailed artifacts using the log-implementation tool (include function signature, panic handling logic), then mark the task as completed in tasks.md_

- [x] 5. Implement complete FFI wrapper
  - File: `core/keyrx_ffi_runtime/src/wrapper.rs`
  - Implement `ffi_wrapper<F, T>(error: *mut *mut c_char, f: F) -> *const c_char`
  - Combine panic catching, error handling, and serialization
  - Purpose: Provide single wrapper for all generated FFI functions
  - _Leverage: panic.rs, json.rs, string.rs modules_
  - _Requirements: REQ-2, REQ-3_
  - _Prompt: Implement the task for spec rust-ffi-codegen, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Developer with expertise in FFI and error handling | Task: Implement comprehensive FFI wrapper following requirements REQ-2 and REQ-3, integrating panic catching, Result handling, and JSON serialization | Restrictions: Must handle all error paths, set error pointer correctly, function under 50 lines | Success: Wrapper handles all scenarios (success, error, panic), memory is managed correctly, error pointer set appropriately | Instructions: Mark this task as in-progress in tasks.md before starting. After completion, log the implementation with detailed artifacts using the log-implementation tool (include function signature, error handling flow), then mark the task as completed in tasks.md_

- [x] 6. Add runtime library unit tests
  - File: `core/keyrx_ffi_runtime/src/tests.rs`
  - Test all helper functions with success and error cases
  - Test panic catching, null handling, invalid UTF-8
  - Purpose: Ensure runtime reliability
  - _Leverage: None (new test module)_
  - _Requirements: REQ-2, REQ-3_
  - _Prompt: Implement the task for spec rust-ffi-codegen, first run spec-workflow-guide to get the workflow guide then implement the task: Role: QA Engineer with Rust testing expertise | Task: Create comprehensive unit tests for FFI runtime library following requirements REQ-2 and REQ-3, testing all helper functions and error paths | Restrictions: Test both success and failure scenarios, test panic catching, maintain test isolation | Success: All runtime functions tested, edge cases covered, tests run reliably | Instructions: Mark this task as in-progress in tasks.md before starting. After completion, log the implementation with detailed artifacts using the log-implementation tool, then mark the task as completed in tasks.md_

### Phase 2: Procedural Macro Crate

- [ ] 7. Create keyrx_ffi_macro crate
  - Files: `core/keyrx_ffi_macro/Cargo.toml`, `core/keyrx_ffi_macro/src/lib.rs`
  - Create proc-macro crate with `proc-macro = true`
  - Add dependencies: syn, quote, proc-macro2, serde_json
  - Purpose: Provide procedural macro for FFI code generation
  - _Leverage: None (new crate)_
  - _Requirements: REQ-1_
  - _Prompt: Implement the task for spec rust-ffi-codegen, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Developer specializing in procedural macros | Task: Create keyrx_ffi_macro proc-macro crate following requirement REQ-1, configuring Cargo.toml for proc-macro type | Restrictions: Must set proc-macro = true, add correct dependencies, follow macro crate conventions | Success: Proc-macro crate compiles, dependencies correct, ready for macro implementation | Instructions: Mark this task as in-progress in tasks.md before starting. After completion, log the implementation with detailed artifacts using the log-implementation tool (include crate structure), then mark the task as completed in tasks.md_

- [ ] 8. Implement macro attribute parsing
  - File: `core/keyrx_ffi_macro/src/parse.rs`
  - Parse `#[keyrx_ffi(domain = "...")]` attribute
  - Extract domain parameter
  - Purpose: Parse macro invocation and extract configuration
  - _Leverage: syn for attribute parsing_
  - _Requirements: REQ-1_
  - _Prompt: Implement the task for spec rust-ffi-codegen, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust macro developer with syn expertise | Task: Implement attribute parsing for keyrx_ffi macro following requirement REQ-1, extracting domain parameter from macro attributes | Restrictions: Must validate domain parameter exists, provide clear errors for invalid usage, function under 50 lines | Success: Parses domain parameter correctly, validates input, provides helpful error messages | Instructions: Mark this task as in-progress in tasks.md before starting. After completion, log the implementation with detailed artifacts using the log-implementation tool (include function signature, parsing logic), then mark the task as completed in tasks.md_

- [ ] 9. Implement contract loader
  - File: `core/keyrx_ffi_macro/src/contract_loader.rs`
  - Implement `load_contract_for_domain(domain: &str) -> Result<FfiContract, syn::Error>`
  - Load and parse JSON contract files at compile time
  - Purpose: Read contracts during macro expansion
  - _Leverage: FfiContract from core/src/ffi/contract.rs, serde_json_
  - _Requirements: REQ-1_
  - _Prompt: Implement the task for spec rust-ffi-codegen, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Developer with compile-time file I/O and macro expertise | Task: Implement contract loader following requirement REQ-1, reading and parsing JSON contracts during macro expansion | Restrictions: Must use CARGO_MANIFEST_DIR, handle file not found, provide clear compile errors, function under 50 lines | Success: Loads contracts at compile time, parses JSON correctly, provides helpful errors for missing/invalid files | Instructions: Mark this task as in-progress in tasks.md before starting. After completion, log the implementation with detailed artifacts using the log-implementation tool (include function signature, file loading logic), then mark the task as completed in tasks.md_

- [ ] 10. Create type mapping for code generation
  - File: `core/keyrx_ffi_macro/src/type_mapper.rs`
  - Implement `map_contract_type_to_ffi(contract_type: &str) -> TokenStream`
  - Generate Rust FFI type tokens from contract types
  - Purpose: Convert contract types to Rust code tokens
  - _Leverage: quote crate for token generation_
  - _Requirements: REQ-2, REQ-4_
  - _Prompt: Implement the task for spec rust-ffi-codegen, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust macro developer with quote and FFI expertise | Task: Implement type mapper for code generation following requirements REQ-2 and REQ-4, converting contract types to Rust token streams | Restrictions: Must handle all contract types, generate correct FFI types, function under 50 lines | Success: Generates correct Rust FFI types for all contract types, token streams compile correctly | Instructions: Mark this task as in-progress in tasks.md before starting. After completion, log the implementation with detailed artifacts using the log-implementation tool (include type mappings, token generation), then mark the task as completed in tasks.md_

- [ ] 11. Implement parameter parser generation
  - File: `core/keyrx_ffi_macro/src/codegen.rs`
  - Generate code for parsing FFI parameters
  - Handle string, int, bool, and JSON types
  - Purpose: Generate parameter parsing code
  - _Leverage: type_mapper.rs, keyrx_ffi_runtime helpers_
  - _Requirements: REQ-2_
  - _Prompt: Implement the task for spec rust-ffi-codegen, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust code generation expert | Task: Generate parameter parsing code following requirement REQ-2, creating code that converts FFI parameters to Rust types | Restrictions: Must use runtime helpers, handle all parameter types, generate safe code, function under 50 lines | Success: Generated parsing code compiles and works correctly, handles all parameter types, uses runtime helpers appropriately | Instructions: Mark this task as in-progress in tasks.md before starting. After completion, log the implementation with detailed artifacts using the log-implementation tool (include code generation logic), then mark the task as completed in tasks.md_

- [ ] 12. Implement result serializer generation
  - File: `core/keyrx_ffi_macro/src/codegen.rs`
  - Generate code for serializing return values
  - Handle void, primitives, and custom types
  - Purpose: Generate result serialization code
  - _Leverage: type_mapper.rs, keyrx_ffi_runtime::serialize_to_c_string_
  - _Requirements: REQ-2_
  - _Prompt: Implement the task for spec rust-ffi-codegen, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust code generation expert | Task: Generate result serialization code following requirement REQ-2, creating code that converts Rust return values to FFI types | Restrictions: Must handle void returns, use serialization helpers, generate correct code, function under 50 lines | Success: Generated serialization code works for all return types, handles void correctly, uses runtime helpers | Instructions: Mark this task as in-progress in tasks.md before starting. After completion, log the implementation with detailed artifacts using the log-implementation tool (include code templates), then mark the task as completed in tasks.md_

- [ ] 13. Implement FFI function generation
  - File: `core/keyrx_ffi_macro/src/codegen.rs`
  - Generate complete `extern "C"` function wrapper
  - Include #[no_mangle], error pointer, panic handling
  - Purpose: Generate complete FFI function from contract
  - _Leverage: All codegen components_
  - _Requirements: REQ-1, REQ-2, REQ-3, REQ-4_
  - _Prompt: Implement the task for spec rust-ffi-codegen, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust macro developer with comprehensive FFI expertise | Task: Implement complete FFI function generation following all requirements, creating extern C wrappers with panic handling and error management | Restrictions: Must use ffi_wrapper from runtime, follow contract signatures exactly, function under 50 lines | Success: Generated FFI functions compile correctly, match contract signatures, handle all error scenarios | Instructions: Mark this task as in-progress in tasks.md before starting. After completion, log the implementation with detailed artifacts using the log-implementation tool (include generated function template, key features), then mark the task as completed in tasks.md_

- [ ] 14. Implement macro entry point
  - File: `core/keyrx_ffi_macro/src/lib.rs`
  - Implement `#[proc_macro_attribute]` for keyrx_ffi
  - Orchestrate parsing, loading, and code generation
  - Purpose: Main entry point for the macro
  - _Leverage: All previous macro components_
  - _Requirements: REQ-1_
  - _Prompt: Implement the task for spec rust-ffi-codegen, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust macro developer | Task: Implement main macro entry point following requirement REQ-1, orchestrating all components to generate FFI code | Restrictions: Must handle errors gracefully with syn::Error, validate impl structure, function under 50 lines | Success: Macro compiles and expands correctly, generates valid Rust code, provides clear errors | Instructions: Mark this task as in-progress in tasks.md before starting. After completion, log the implementation with detailed artifacts using the log-implementation tool (include macro flow, error handling), then mark the task as completed in tasks.md_

### Phase 3: Testing & Integration

- [ ] 15. Add trybuild tests for macro
  - File: `core/keyrx_ffi_macro/tests/compile_tests.rs`
  - Create trybuild tests for successful expansions
  - Create trybuild tests for error cases
  - Purpose: Test macro compilation behavior
  - _Leverage: trybuild crate_
  - _Requirements: REQ-1, REQ-6_
  - _Prompt: Implement the task for spec rust-ffi-codegen, first run spec-workflow-guide to get the workflow guide then implement the task: Role: QA Engineer with Rust macro testing expertise | Task: Create trybuild tests for macro following requirements REQ-1 and REQ-6, testing both successful and failing compilations | Restrictions: Test valid and invalid macro usage, verify error messages, organize test files clearly | Success: Macro behavior is fully tested, error cases produce expected errors, tests catch regressions | Instructions: Mark this task as in-progress in tasks.md before starting. After completion, log the implementation with detailed artifacts using the log-implementation tool, then mark the task as completed in tasks.md_

- [ ] 16. Create integration test with real contract
  - File: `core/tests/ffi_macro_integration.rs`
  - Use macro with config domain contract
  - Verify generated code compiles and works
  - Purpose: Test end-to-end macro functionality
  - _Leverage: Existing config.ffi-contract.json_
  - _Requirements: All_
  - _Prompt: Implement the task for spec rust-ffi-codegen, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Integration test engineer with Rust expertise | Task: Create integration test using real contracts, verifying generated FFI functions work correctly end-to-end | Restrictions: Must use real contracts, test actual FFI calls, verify results, test under 100 lines | Success: Generated code works with real contracts, FFI calls succeed, demonstrates full functionality | Instructions: Mark this task as in-progress in tasks.md before starting. After completion, log the implementation with detailed artifacts using the log-implementation tool (include test scenarios), then mark the task as completed in tasks.md_

- [ ] 17. Apply macro to config domain
  - File: `core/src/ffi/domains/config.rs`
  - Replace manual FFI functions with macro-generated ones
  - Verify all config functions work identically
  - Purpose: Migrate first domain to use macro
  - _Leverage: config.ffi-contract.json, generated code_
  - _Requirements: All_
  - _Prompt: Implement the task for spec rust-ffi-codegen, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Developer with migration expertise | Task: Migrate config domain to use keyrx_ffi macro, replacing manual FFI with generated code while maintaining identical behavior | Restrictions: Must not break existing functionality, test thoroughly, keep implementation clean | Success: Config domain uses macro, all functions work identically, code is cleaner and shorter | Instructions: Mark this task as in-progress in tasks.md before starting. After completion, log the implementation with detailed artifacts using the log-implementation tool (include migration changes, before/after comparison), then mark the task as completed in tasks.md_

- [ ] 18. Add documentation and examples
  - Files: `core/keyrx_ffi_macro/README.md`, inline docs
  - Document macro usage with examples
  - Create migration guide from manual FFI
  - Purpose: Enable developers to use the macro
  - _Leverage: None (new documentation)_
  - _Requirements: REQ-5_
  - _Prompt: Implement the task for spec rust-ffi-codegen, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Technical Writer with Rust FFI expertise | Task: Create comprehensive documentation for FFI macro following requirement REQ-5, including usage examples and migration guide | Restrictions: Provide clear examples, explain contract requirements, document error cases | Success: Documentation is complete, examples are clear, migration guide is helpful | Instructions: Mark this task as in-progress in tasks.md before starting. After completion, log the implementation with detailed artifacts using the log-implementation tool, then mark the task as completed in tasks.md_
