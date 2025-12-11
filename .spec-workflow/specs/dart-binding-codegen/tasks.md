# Tasks Document

## Implementation Tasks

### Phase 1: CLI Tool Setup

- [ ] 1. Create generate_dart_bindings binary crate
  - Files: `core/tools/generate_dart_bindings/Cargo.toml`, `core/tools/generate_dart_bindings/src/main.rs`
  - Create new binary crate for code generator
  - Add dependencies: clap, serde_json, walkdir
  - Purpose: Provide CLI tool for generating Dart bindings
  - _Leverage: Existing ContractRegistry from core/src/ffi/contract.rs_
  - _Requirements: REQ-1, REQ-6_
  - _Prompt: Implement the task for spec dart-binding-codegen, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Developer specializing in CLI tools | Task: Create generate_dart_bindings binary crate following requirements REQ-1 and REQ-6, setting up project structure and dependencies | Restrictions: Must use clap for CLI, add necessary dependencies, follow tool naming conventions | Success: Binary crate compiles, dependencies configured, ready for implementation | Instructions: Mark this task as in-progress in tasks.md before starting. After completion, log the implementation with detailed artifacts using the log-implementation tool (include crate structure, dependencies), then mark the task as completed in tasks.md_

- [ ] 2. Implement CLI argument parsing
  - File: `core/tools/generate_dart_bindings/src/cli.rs`
  - Parse command-line arguments (--domain, --check, --verbose)
  - Define CLI structure with clap
  - Purpose: Handle command-line interface
  - _Leverage: clap crate_
  - _Requirements: REQ-6_
  - _Prompt: Implement the task for spec dart-binding-codegen, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust CLI developer | Task: Implement CLI argument parsing following requirement REQ-6, using clap to define command structure and options | Restrictions: Must support all required flags, provide help text, validate arguments | Success: CLI parses all arguments correctly, help text is clear, validation works | Instructions: Mark this task as in-progress in tasks.md before starting. After completion, log the implementation with detailed artifacts using the log-implementation tool (include CLI structure, arguments), then mark the task as completed in tasks.md_

- [ ] 3. Implement contract loading
  - File: `core/tools/generate_dart_bindings/src/loader.rs`
  - Load all contracts from core/src/ffi/contracts directory
  - Handle missing files and parse errors
  - Purpose: Read contract files for code generation
  - _Leverage: ContractRegistry from core_
  - _Requirements: REQ-1_
  - _Prompt: Implement the task for spec dart-binding-codegen, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Developer with file I/O expertise | Task: Implement contract loading following requirement REQ-1, reading and parsing all FFI contracts from directory | Restrictions: Must handle file errors gracefully, validate JSON, provide clear errors, function under 50 lines | Success: Loads all contracts successfully, handles errors appropriately, returns structured data | Instructions: Mark this task as in-progress in tasks.md before starting. After completion, log the implementation with detailed artifacts using the log-implementation tool (include function signature, error handling), then mark the task as completed in tasks.md_

### Phase 2: Type Mapping

- [ ] 4. Create DartType enum
  - File: `core/tools/generate_dart_bindings/src/types.rs`
  - Define enum for Dart FFI types (Pointer<Utf8>, Int32, Bool, etc.)
  - Include methods for code generation
  - Purpose: Model Dart FFI type system
  - _Leverage: None (new module)_
  - _Requirements: REQ-2_
  - _Prompt: Implement the task for spec dart-binding-codegen, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Developer with type system expertise | Task: Create DartType enum following requirement REQ-2, modeling all Dart FFI types with code generation methods | Restrictions: Must cover all FFI types, include to_string() for code gen, derive Debug/Clone | Success: Enum covers all Dart FFI types, code generation methods work, type-safe representation | Instructions: Mark this task as in-progress in tasks.md before starting. After completion, log the implementation with detailed artifacts using the log-implementation tool, then mark the task as completed in tasks.md_

- [ ] 5. Implement contract to Dart type mapper
  - File: `core/tools/generate_dart_bindings/src/type_mapper.rs`
  - Implement `map_to_dart_ffi_type(contract_type: &str) -> DartType`
  - Implement `map_to_dart_native_type(contract_type: &str) -> String`
  - Purpose: Convert contract types to Dart types
  - _Leverage: TypeDefinition from contracts, DartType enum_
  - _Requirements: REQ-2_
  - _Prompt: Implement the task for spec dart-binding-codegen, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Developer with Dart FFI knowledge | Task: Implement type mapper following requirement REQ-2, converting contract types to Dart FFI and native types | Restrictions: Must handle all contract types, return clear errors for unknown types, function under 50 lines | Success: All contract types map correctly to Dart types, mappings are accurate, errors are helpful | Instructions: Mark this task as in-progress in tasks.md before starting. After completion, log the implementation with detailed artifacts using the log-implementation tool (include type mapping table), then mark the task as completed in tasks.md_

- [ ] 6. Add unit tests for type mapper
  - File: `core/tools/generate_dart_bindings/src/type_mapper_tests.rs`
  - Test all type mappings
  - Test nullable type handling
  - Purpose: Ensure type mapping reliability
  - _Leverage: None (new test module)_
  - _Requirements: REQ-2_
  - _Prompt: Implement the task for spec dart-binding-codegen, first run spec-workflow-guide to get the workflow guide then implement the task: Role: QA Engineer with Rust testing expertise | Task: Create unit tests for type mapper following requirement REQ-2, testing all contract type to Dart type conversions | Restrictions: Test all primitive and custom types, test nullable variants, maintain test clarity | Success: All type mappings tested, edge cases covered, tests verify correctness | Instructions: Mark this task as in-progress in tasks.md before starting. After completion, log the implementation with detailed artifacts using the log-implementation tool, then mark the task as completed in tasks.md_

### Phase 3: Code Generation

- [ ] 7. Create code generation templates
  - File: `core/tools/generate_dart_bindings/src/templates.rs`
  - Define string templates for FFI signatures, wrappers, and classes
  - Include placeholder replacement logic
  - Purpose: Provide templates for code generation
  - _Leverage: None (new module)_
  - _Requirements: REQ-1, REQ-3_
  - _Prompt: Implement the task for spec dart-binding-codegen, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Code generation expert | Task: Create code templates following requirements REQ-1 and REQ-3, defining Dart code templates with placeholder replacement | Restrictions: Templates must produce valid Dart code, placeholders clearly marked, keep templates readable | Success: Templates generate valid Dart code, placeholders work correctly, code is properly formatted | Instructions: Mark this task as in-progress in tasks.md before starting. After completion, log the implementation with detailed artifacts using the log-implementation tool (include template examples), then mark the task as completed in tasks.md_

- [ ] 8. Implement FFI signature generator
  - File: `core/tools/generate_dart_bindings/src/bindings_gen.rs`
  - Generate typedef declarations for native and Dart signatures
  - Generate function pointer lookups
  - Purpose: Generate FFI type definitions
  - _Leverage: templates.rs, type_mapper.rs_
  - _Requirements: REQ-1_
  - _Prompt: Implement the task for spec dart-binding-codegen, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Dart FFI code generator expert | Task: Generate FFI signature declarations following requirement REQ-1, creating typedef and lookup code | Restrictions: Must generate valid Dart FFI code, follow naming conventions, function under 50 lines | Success: Generated signatures are valid Dart, typedef naming is correct, lookups work | Instructions: Mark this task as in-progress in tasks.md before starting. After completion, log the implementation with detailed artifacts using the log-implementation tool (include generated code examples), then mark the task as completed in tasks.md_

- [ ] 9. Implement wrapper function generator
  - File: `core/tools/generate_dart_bindings/src/bindings_gen.rs`
  - Generate high-level wrapper functions with error handling
  - Add parameter marshaling and result deserialization
  - Purpose: Generate user-friendly Dart functions
  - _Leverage: templates.rs, type_mapper.rs_
  - _Requirements: REQ-3, REQ-4_
  - _Prompt: Implement the task for spec dart-binding-codegen, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Dart developer with FFI and error handling expertise | Task: Generate wrapper functions following requirements REQ-3 and REQ-4, creating functions with automatic error handling and marshaling | Restrictions: Must handle error pointers, free memory correctly, follow Dart conventions, function under 50 lines | Success: Generated wrappers handle errors, memory management is correct, code is idiomatic Dart | Instructions: Mark this task as in-progress in tasks.md before starting. After completion, log the implementation with detailed artifacts using the log-implementation tool (include wrapper template, error handling), then mark the task as completed in tasks.md_

- [ ] 10. Implement model class generator
  - File: `core/tools/generate_dart_bindings/src/models_gen.rs`
  - Generate Dart classes from contract type definitions
  - Generate fromJson and toJson methods
  - Purpose: Generate Dart models for custom types
  - _Leverage: templates.rs, TypeDefinition from contracts_
  - _Requirements: REQ-5_
  - _Prompt: Implement the task for spec dart-binding-codegen, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Dart model generator expert | Task: Generate Dart model classes following requirement REQ-5, creating classes with JSON serialization | Restrictions: Must handle nested types, generate valid JSON methods, follow Dart class conventions | Success: Generated classes are valid Dart, fromJson/toJson work correctly, nested types supported | Instructions: Mark this task as in-progress in tasks.md before starting. After completion, log the implementation with detailed artifacts using the log-implementation tool (include class template, JSON methods), then mark the task as completed in tasks.md_

- [ ] 11. Add file header generator
  - File: `core/tools/generate_dart_bindings/src/header.rs`
  - Generate warning header for generated files
  - Include generation timestamp and source info
  - Purpose: Mark files as generated
  - _Leverage: None_
  - _Requirements: All_
  - _Prompt: Implement the task for spec dart-binding-codegen, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Code generation utility developer | Task: Generate file headers for generated code, including warnings and metadata | Restrictions: Must warn not to edit manually, include timestamp, list source contracts | Success: Headers are clear, include all necessary information, properly formatted | Instructions: Mark this task as in-progress in tasks.md before starting. After completion, log the implementation with detailed artifacts using the log-implementation tool, then mark the task as completed in tasks.md_

### Phase 4: File Writing & Formatting

- [ ] 12. Implement file writer
  - File: `core/tools/generate_dart_bindings/src/writer.rs`
  - Write generated code to files
  - Check if regeneration is needed (timestamps)
  - Purpose: Write generated code to disk
  - _Leverage: std::fs for file I/O_
  - _Requirements: REQ-6_
  - _Prompt: Implement the task for spec dart-binding-codegen, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust file I/O developer | Task: Implement file writer following requirement REQ-6, writing generated code only when needed | Restrictions: Must check timestamps, handle write errors, create directories if needed, function under 50 lines | Success: Writes files correctly, skips unnecessary writes, handles errors gracefully | Instructions: Mark this task as in-progress in tasks.md before starting. After completion, log the implementation with detailed artifacts using the log-implementation tool (include file writing logic), then mark the task as completed in tasks.md_

- [ ] 13. Implement Dart formatter integration
  - File: `core/tools/generate_dart_bindings/src/formatter.rs`
  - Run `dart format` on generated files
  - Handle formatting errors
  - Purpose: Format generated Dart code
  - _Leverage: std::process::Command_
  - _Requirements: All_
  - _Prompt: Implement the task for spec dart-binding-codegen, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Process integration developer | Task: Integrate dart format command to format generated code, handling success and failure | Restrictions: Must check if dart command exists, handle stderr, continue on format failure with warning | Success: Runs dart format correctly, handles errors, formatted code is valid | Instructions: Mark this task as in-progress in tasks.md before starting. After completion, log the implementation with detailed artifacts using the log-implementation tool (include command execution), then mark the task as completed in tasks.md_

- [ ] 14. Orchestrate full generation pipeline
  - File: `core/tools/generate_dart_bindings/src/main.rs`
  - Connect all components (load → generate → write → format)
  - Handle errors at each stage
  - Purpose: Complete the generation tool
  - _Leverage: All previous components_
  - _Requirements: All_
  - _Prompt: Implement the task for spec dart-binding-codegen, first run spec-workflow-guide to get the workflow guide then implement the task: Role: System integration developer | Task: Orchestrate full generation pipeline, connecting all components and handling errors appropriately | Restrictions: Must handle errors at each stage, provide progress feedback in verbose mode, exit with correct codes | Success: Generator works end-to-end, generates valid Dart code, handles all scenarios | Instructions: Mark this task as in-progress in tasks.md before starting. After completion, log the implementation with detailed artifacts using the log-implementation tool (include pipeline flow), then mark the task as completed in tasks.md_

### Phase 5: Testing & Integration

- [ ] 15. Add unit tests for code generators
  - File: `core/tools/generate_dart_bindings/src/tests.rs`
  - Test FFI signature generation
  - Test wrapper function generation
  - Test model class generation
  - Purpose: Ensure code generation correctness
  - _Leverage: None (new test module)_
  - _Requirements: All_
  - _Prompt: Implement the task for spec dart-binding-codegen, first run spec-workflow-guide to get the workflow guide then implement the task: Role: QA Engineer with code generation testing expertise | Task: Create unit tests for all code generators, verifying generated Dart code is correct | Restrictions: Test with various contract types, verify generated code structure, maintain test clarity | Success: All generators tested, output verified correct, edge cases covered | Instructions: Mark this task as in-progress in tasks.md before starting. After completion, log the implementation with detailed artifacts using the log-implementation tool, then mark the task as completed in tasks.md_

- [ ] 16. Create integration test
  - File: `core/tools/generate_dart_bindings/tests/integration_test.rs`
  - Run generator with real contracts
  - Verify output files exist and are valid
  - Run `dart analyze` on generated code
  - Purpose: Test end-to-end generation
  - _Leverage: Real contracts from core/src/ffi/contracts_
  - _Requirements: All_
  - _Prompt: Implement the task for spec dart-binding-codegen, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Integration test engineer | Task: Create integration test running full generation pipeline with real contracts, verifying output | Restrictions: Must use real contracts, verify files created, run dart analyze, clean up after test | Success: Generator works with real contracts, output is valid Dart, analyze passes | Instructions: Mark this task as in-progress in tasks.md before starting. After completion, log the implementation with detailed artifacts using the log-implementation tool (include test scenarios), then mark the task as completed in tasks.md_

- [ ] 17. Add justfile recipes for generation
  - File: `justfile` (project root)
  - Add `gen-dart-bindings` recipe
  - Update `build` recipe to include generation
  - Purpose: Integrate with build system
  - _Leverage: Existing justfile structure_
  - _Requirements: REQ-7_
  - _Prompt: Implement the task for spec dart-binding-codegen, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Build system engineer | Task: Add justfile recipes for Dart binding generation following requirement REQ-7, integrating with existing build workflow | Restrictions: Must follow existing justfile patterns, add clear comments, make recipes reusable | Success: Recipes work correctly, integrate with build, easy to use | Instructions: Mark this task as in-progress in tasks.md before starting. After completion, log the implementation with detailed artifacts using the log-implementation tool (include recipes added), then mark the task as completed in tasks.md_

- [ ] 18. Add CI check for up-to-date bindings
  - File: `.github/workflows/ci.yml` (or CI config)
  - Add step to check if bindings are current
  - Fail CI if regeneration needed
  - Purpose: Ensure bindings stay in sync with contracts
  - _Leverage: Existing CI pipeline_
  - _Requirements: REQ-7_
  - _Prompt: Implement the task for spec dart-binding-codegen, first run spec-workflow-guide to get the workflow guide then implement the task: Role: CI/CD engineer | Task: Add CI check following requirement REQ-7, verifying Dart bindings are up-to-date with contracts | Restrictions: Must run generator and check git diff, fail if bindings outdated, provide clear error | Success: CI catches stale bindings, error message is helpful, check is reliable | Instructions: Mark this task as in-progress in tasks.md before starting. After completion, log the implementation with detailed artifacts using the log-implementation tool (include CI step), then mark the task as completed in tasks.md_

- [ ] 19. Generate bindings for all domains
  - Files: `ui/lib/ffi/generated_bindings.dart`, `ui/lib/models/generated_models.dart`
  - Run generator on all existing contracts
  - Verify generated code compiles
  - Purpose: Generate initial bindings for production use
  - _Leverage: All contracts in core/src/ffi/contracts_
  - _Requirements: All_
  - _Prompt: Implement the task for spec dart-binding-codegen, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Dart developer | Task: Generate bindings for all domains and verify they compile with Flutter, fixing any issues discovered | Restrictions: Must generate for all contracts, fix compilation errors, test bindings work | Success: Bindings generated successfully, Flutter compiles, bindings are usable | Instructions: Mark this task as in-progress in tasks.md before starting. After completion, log the implementation with detailed artifacts using the log-implementation tool (include generated files, any fixes), then mark the task as completed in tasks.md_

- [ ] 20. Add documentation and examples
  - Files: `core/tools/generate_dart_bindings/README.md`, inline docs
  - Document generator usage
  - Provide examples of using generated bindings
  - Document integration with build system
  - Purpose: Enable developers to use and maintain the generator
  - _Leverage: None (new documentation)_
  - _Requirements: REQ-6_
  - _Prompt: Implement the task for spec dart-binding-codegen, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Technical Writer with Dart and Rust expertise | Task: Create comprehensive documentation for Dart binding generator following requirement REQ-6, including usage, examples, and build integration | Restrictions: Provide clear usage examples, document CLI options, explain build integration | Success: Documentation is complete, examples are helpful, developers can use generator effectively | Instructions: Mark this task as in-progress in tasks.md before starting. After completion, log the implementation with detailed artifacts using the log-implementation tool, then mark the task as completed in tasks.md_
