# Tasks Document

## Phase 1: Core Types

- [x] 1. Create ErrorCode type
  - File: `core/src/errors/code.rs`
  - Define ErrorCode with category and number
  - Implement Display as KRX-XXXX
  - Purpose: Unique error identifier
  - _Leverage: Rust type patterns_
  - _Requirements: 1.1, 1.3_
  - _Prompt: Implement the task for spec error-code-registry, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Developer creating types | Task: Create ErrorCode with KRX-XXXX format | Restrictions: Category prefix, 3-digit number, Display impl | _Leverage: Type patterns | Success: ErrorCode displays correctly | After completion: Mark task [-] as in-progress before starting, use log-implementation tool to record artifacts, then mark [x] when complete_

- [x] 2. Create ErrorCategory enum
  - File: `core/src/errors/category.rs`
  - Define categories: Config, Runtime, Driver, Validation, Ffi, Internal
  - Assign number ranges to each category
  - Purpose: Error categorization
  - _Leverage: Enum patterns_
  - _Requirements: 4.1, 4.2_
  - _Prompt: Implement the task for spec error-code-registry, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Developer creating enums | Task: Create ErrorCategory with prefix and range | Restrictions: Clear prefixes, no overlap in ranges | _Leverage: Enum patterns | Success: Categories cover all error types | After completion: Mark task [-] as in-progress before starting, use log-implementation tool to record artifacts, then mark [x] when complete_

- [x] 3. Create ErrorDef type
  - File: `core/src/errors/definition.rs`
  - Define error with code, template, hint, severity
  - Add message formatting with args
  - Purpose: Error definition
  - _Leverage: Template patterns_
  - _Requirements: 2.2, 3.1, 3.2_
  - _Prompt: Implement the task for spec error-code-registry, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Developer creating definitions | Task: Create ErrorDef with message templating | Restrictions: Template args, hint support, severity | _Leverage: Template patterns | Success: Definitions format correctly | After completion: Mark task [-] as in-progress before starting, use log-implementation tool to record artifacts, then mark [x] when complete_

- [x] 4. Create KeyrxError type
  - File: `core/src/errors/error.rs`
  - Implement std::error::Error
  - Add context and source chaining
  - JSON serialization
  - Purpose: Runtime error type
  - _Leverage: thiserror_
  - _Requirements: 1.3, 1.4, 3.3_
  - _Prompt: Implement the task for spec error-code-registry, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Developer creating errors | Task: Create KeyrxError with thiserror | Restrictions: Error trait, context chain, JSON output | _Leverage: thiserror | Success: Errors chain and serialize | After completion: Mark task [-] as in-progress before starting, use log-implementation tool to record artifacts, then mark [x] when complete_

## Phase 2: Registry Infrastructure

- [x] 5. Create define_errors! macro
  - File: `core/src/errors/macros.rs`
  - Macro for defining error sets
  - Compile-time duplicate detection
  - Purpose: Error definition convenience
  - _Leverage: Rust macro patterns_
  - _Requirements: 2.1, 2.2_
  - _Prompt: Implement the task for spec error-code-registry, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Macro Developer | Task: Create define_errors! macro with duplicate detection | Restrictions: Compile-time checks, clear syntax | _Leverage: Macro patterns | Success: Macro defines errors cleanly | After completion: Mark task [-] as in-progress before starting, use log-implementation tool to record artifacts, then mark [x] when complete_

- [x] 6. Create keyrx_err! and bail_keyrx! macros
  - File: `core/src/errors/macros.rs`
  - Error creation macros
  - Context injection
  - Purpose: Convenient error creation
  - _Leverage: anyhow patterns_
  - _Requirements: 3.3_
  - _Prompt: Implement the task for spec error-code-registry, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Macro Developer | Task: Create keyrx_err! and bail_keyrx! macros | Restrictions: Ergonomic, context support | _Leverage: anyhow patterns | Success: Errors easy to create | After completion: Mark task [-] as in-progress before starting, use log-implementation tool to record artifacts, then mark [x] when complete_

- [ ] 7. Create ErrorRegistry
  - File: `core/src/errors/registry.rs`
  - Static registry of all errors
  - Lookup by code and category
  - Purpose: Central error repository
  - _Leverage: Static patterns_
  - _Requirements: 2.1, 4.3_
  - _Prompt: Implement the task for spec error-code-registry, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Developer creating registry | Task: Create ErrorRegistry with static storage | Restrictions: Thread-safe, efficient lookup | _Leverage: Static patterns | Success: Registry provides all errors | After completion: Mark task [-] as in-progress before starting, use log-implementation tool to record artifacts, then mark [x] when complete_

## Phase 3: Define Errors

- [ ] 8. Define Config errors (1xxx)
  - File: `core/src/errors/config.rs`
  - Config loading, parsing, validation errors
  - Purpose: Configuration error codes
  - _Leverage: define_errors! macro_
  - _Requirements: 2.2, 4.2_
  - _Prompt: Implement the task for spec error-code-registry, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Developer defining errors | Task: Define Config category errors (KRX-C1xx) | Restrictions: Cover all config failures, helpful hints | _Leverage: define_errors! | Success: Config errors comprehensive | After completion: Mark task [-] as in-progress before starting, use log-implementation tool to record artifacts, then mark [x] when complete_

- [ ] 9. Define Runtime errors (2xxx)
  - File: `core/src/errors/runtime.rs`
  - Engine, processing, state errors
  - Purpose: Runtime error codes
  - _Leverage: define_errors! macro_
  - _Requirements: 2.2, 4.2_
  - _Prompt: Implement the task for spec error-code-registry, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Developer defining errors | Task: Define Runtime category errors (KRX-R2xx) | Restrictions: Cover all runtime failures, hints | _Leverage: define_errors! | Success: Runtime errors comprehensive | After completion: Mark task [-] as in-progress before starting, use log-implementation tool to record artifacts, then mark [x] when complete_

- [ ] 10. Define Driver errors (3xxx)
  - File: `core/src/errors/driver.rs`
  - Windows and Linux driver errors
  - Purpose: Driver error codes
  - _Leverage: define_errors! macro_
  - _Requirements: 2.2, 4.2_
  - _Prompt: Implement the task for spec error-code-registry, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Developer defining errors | Task: Define Driver category errors (KRX-D3xx) | Restrictions: Platform-specific hints, cover all drivers | _Leverage: define_errors! | Success: Driver errors comprehensive | After completion: Mark task [-] as in-progress before starting, use log-implementation tool to record artifacts, then mark [x] when complete_

- [ ] 11. Define Validation errors (4xxx)
  - File: `core/src/errors/validation.rs`
  - Config validation, conflict detection errors
  - Purpose: Validation error codes
  - _Leverage: define_errors! macro_
  - _Requirements: 2.2, 4.2_
  - _Prompt: Implement the task for spec error-code-registry, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Developer defining errors | Task: Define Validation category errors (KRX-V4xx) | Restrictions: Cover all validation failures | _Leverage: define_errors! | Success: Validation errors comprehensive | After completion: Mark task [-] as in-progress before starting, use log-implementation tool to record artifacts, then mark [x] when complete_

## Phase 4: Migration

- [ ] 12. Replace println errors in CLI
  - Files: `core/src/bin/keyrx.rs`, CLI modules
  - Replace raw println with keyrx_err!
  - Purpose: Consistent CLI errors
  - _Leverage: keyrx_err! macro_
  - _Requirements: 1.1, 1.3_
  - _Prompt: Implement the task for spec error-code-registry, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Developer migrating errors | Task: Replace println errors in CLI with keyrx_err! | Restrictions: All user-facing errors, preserve context | _Leverage: keyrx_err! | Success: CLI uses error codes | After completion: Mark task [-] as in-progress before starting, use log-implementation tool to record artifacts, then mark [x] when complete_

- [ ] 13. Replace anyhow errors in core
  - Files: Core engine and processing
  - Migrate anyhow::anyhow! to keyrx_err!
  - Purpose: Structured core errors
  - _Leverage: keyrx_err! macro_
  - _Requirements: 1.1, 3.3_
  - _Prompt: Implement the task for spec error-code-registry, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Developer migrating errors | Task: Replace anyhow! in core with keyrx_err! | Restrictions: Preserve error context, maintain chains | _Leverage: keyrx_err! | Success: Core uses error codes | After completion: Mark task [-] as in-progress before starting, use log-implementation tool to record artifacts, then mark [x] when complete_

- [ ] 14. Update FFI error exports
  - File: `core/src/ffi/exports_*.rs`
  - Export error codes through FFI
  - Purpose: Flutter error access
  - _Leverage: FFI patterns_
  - _Requirements: 1.3_
  - _Prompt: Implement the task for spec error-code-registry, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Developer updating FFI | Task: Update FFI to export error codes | Restrictions: C-compatible, include hints | _Leverage: FFI patterns | Success: Flutter receives error codes | After completion: Mark task [-] as in-progress before starting, use log-implementation tool to record artifacts, then mark [x] when complete_

## Phase 5: Documentation

- [ ] 15. Create ErrorDocGenerator
  - File: `core/src/errors/doc_generator.rs`
  - Generate markdown from registry
  - Per-category documentation
  - Purpose: Auto-generate docs
  - _Leverage: ErrorRegistry_
  - _Requirements: 1.2, 2.3_
  - _Prompt: Implement the task for spec error-code-registry, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Developer creating generator | Task: Create ErrorDocGenerator for markdown | Restrictions: Per-category, include hints, examples | _Leverage: ErrorRegistry | Success: Docs generate from registry | After completion: Mark task [-] as in-progress before starting, use log-implementation tool to record artifacts, then mark [x] when complete_

- [ ] 16. Generate initial error documentation
  - Files: `docs/errors/{index,config,runtime,driver,validation}.md`
  - Run generator for all categories
  - Purpose: User-facing error docs
  - _Leverage: ErrorDocGenerator_
  - _Requirements: 1.2_
  - _Prompt: Implement the task for spec error-code-registry, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Technical Writer | Task: Generate error documentation from registry | Restrictions: All categories, searchable format | _Leverage: ErrorDocGenerator | Success: Comprehensive error docs exist | After completion: Mark task [-] as in-progress before starting, use log-implementation tool to record artifacts, then mark [x] when complete_

- [ ] 17. Add doc generation to build
  - File: `core/build.rs` or script
  - Regenerate docs on error changes
  - Purpose: Keep docs in sync
  - _Leverage: Build script patterns_
  - _Requirements: 2.3_
  - _Prompt: Implement the task for spec error-code-registry, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Build Developer | Task: Add error doc generation to build | Restrictions: Only regenerate on changes, fast | _Leverage: Build scripts | Success: Docs auto-update | After completion: Mark task [-] as in-progress before starting, use log-implementation tool to record artifacts, then mark [x] when complete_

## Phase 6: Testing

- [ ] 18. Add error code tests
  - File: `core/tests/unit/errors/`
  - Test formatting, serialization
  - Verify no duplicate codes
  - Purpose: Error correctness
  - _Leverage: Test fixtures_
  - _Requirements: Non-functional (reliability)_
  - _Prompt: Implement the task for spec error-code-registry, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Test Developer | Task: Create error code tests | Restrictions: Test formatting, duplicates, serialization | _Leverage: Test fixtures | Success: All error codes valid | After completion: Mark task [-] as in-progress before starting, use log-implementation tool to record artifacts, then mark [x] when complete_
