# Tasks Document: dev-tooling

## Task 1: Create Configuration Files

- [x] 1.1 Create rustfmt.toml
  - File: `rustfmt.toml` (repository root)
  - Configure: max_width=100, edition=2021, imports_granularity=Crate
  - Enable: format_code_in_doc_comments, format_strings
  - Purpose: Consistent code formatting across team
  - _Requirements: REQ-6_
  - _Prompt: Implement the task for spec dev-tooling, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust DevOps Engineer | Task: Create rustfmt.toml at repository root with max_width=100, edition="2021", imports_granularity="Crate", format_code_in_doc_comments=true, format_strings=true, group_imports="StdExternalCrate" | Restrictions: Use only stable rustfmt options, no nightly-only features | Success: cargo fmt --check passes without changes needed. Mark task [-] in tasks.md before starting, log implementation with log-implementation tool after completion, then mark [x] when complete._

- [x] 1.2 Create clippy.toml
  - File: `clippy.toml` (repository root)
  - Configure: cognitive-complexity-threshold=25, too-many-arguments-threshold=7
  - Purpose: Consistent lint levels
  - _Requirements: REQ-6_
  - _Prompt: Implement the task for spec dev-tooling, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust DevOps Engineer | Task: Create clippy.toml with cognitive-complexity-threshold=25, too-many-arguments-threshold=7, type-complexity-threshold=250 | Restrictions: Only use stable clippy configuration options | Success: cargo clippy uses the configuration. Mark task [-] in tasks.md before starting, log implementation with log-implementation tool after completion, then mark [x] when complete._

- [x] 1.3 Update Cargo.toml with dev profile
  - File: `core/Cargo.toml`
  - Add [profile.dev] with opt-level=0, debug=true
  - Add [profile.dev.package."*"] with opt-level=2 for faster deps
  - Add [lints.rust] and [lints.clippy] sections
  - Purpose: Fast iteration builds
  - _Requirements: REQ-6_
  - _Prompt: Implement the task for spec dev-tooling, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust DevOps Engineer | Task: Update core/Cargo.toml to add [profile.dev] with opt-level=0, debug=true. Add [profile.dev.package."*"] with opt-level=2. Add [lints.rust] with unsafe_code="warn". Add [lints.clippy] with unwrap_used="warn", expect_used="warn", panic="warn" | Restrictions: Keep existing dependencies unchanged | Success: cargo build uses optimized profile. Mark task [-] in tasks.md before starting, log implementation with log-implementation tool after completion, then mark [x] when complete._

## Task 2: Pre-commit Hooks

- [x] 2.1 Create install-hooks script
  - File: `scripts/install-hooks.sh`
  - Create script that copies .githooks/* to .git/hooks/
  - Make executable and cross-platform (bash)
  - Print success message with hook list
  - Purpose: One-command hook installation
  - _Requirements: REQ-1_
  - _Prompt: Implement the task for spec dev-tooling, first run spec-workflow-guide to get the workflow guide then implement the task: Role: DevOps Engineer | Task: Create scripts/install-hooks.sh that: 1) Creates .git/hooks if missing, 2) Copies all files from .githooks/ to .git/hooks/, 3) Makes them executable with chmod +x, 4) Prints "Hooks installed: pre-commit". Use bash shebang, handle errors gracefully | Restrictions: No external dependencies beyond bash, support Linux/macOS | Success: Running ./scripts/install-hooks.sh installs hooks successfully. Mark task [-] in tasks.md before starting, log implementation with log-implementation tool after completion, then mark [x] when complete._

- [x] 2.2 Create pre-commit hook
  - File: `.githooks/pre-commit`
  - Run: cargo fmt --check (fail fast)
  - Run: cargo clippy -- -D warnings (deny warnings)
  - Run: cargo test --lib (unit tests only for speed)
  - Exit non-zero on any failure with clear message
  - Purpose: Enforce quality before commit
  - _Requirements: REQ-1_
  - _Prompt: Implement the task for spec dev-tooling, first run spec-workflow-guide to get the workflow guide then implement the task: Role: DevOps Engineer | Task: Create .githooks/pre-commit bash script that: 1) Runs cargo fmt --check, exits 1 on failure with "Format check failed. Run cargo fmt", 2) Runs cargo clippy -- -D warnings, exits 1 on failure, 3) Runs cargo test --lib, exits 1 on failure. Print status for each step. Use set -e for fail-fast | Restrictions: Keep total runtime under 30 seconds, test only lib not integration tests | Success: git commit is blocked when checks fail. Mark task [-] in tasks.md before starting, log implementation with log-implementation tool after completion, then mark [x] when complete._

## Task 3: GitHub Actions CI

- [x] 3.1 Create CI workflow
  - File: `.github/workflows/ci.yml`
  - Trigger on: push to main, pull_request
  - Jobs: check (fmt, clippy), test, build-linux, build-windows
  - Use: actions/checkout@v4, dtolnay/rust-toolchain@stable
  - Cache: Swatinem/rust-cache@v2
  - Purpose: Automated PR validation
  - _Requirements: REQ-2_
  - _Prompt: Implement the task for spec dev-tooling, first run spec-workflow-guide to get the workflow guide then implement the task: Role: GitHub Actions Engineer | Task: Create .github/workflows/ci.yml with: 1) name "CI", 2) triggers on push to main and pull_request, 3) Job "check" running cargo fmt --check and cargo clippy -- -D warnings, 4) Job "test" running cargo test --all-features, 5) Job "build-linux" building release binary, 6) Job "build-windows" cross-compiling with target x86_64-pc-windows-gnu. Use actions/checkout@v4, dtolnay/rust-toolchain@stable, Swatinem/rust-cache@v2. Set working-directory to ./core for cargo commands | Restrictions: Use only stable Rust, cache dependencies, keep workflow under 10 minutes | Success: PR triggers CI and all jobs pass. Mark task [-] in tasks.md before starting, log implementation with log-implementation tool after completion, then mark [x] when complete._

## Task 4: Custom Error Types

- [x] 4.1 Create error module
  - File: `core/src/error.rs`
  - Define KeyRxError enum with thiserror derive
  - Variants: UnknownKey, ScriptCompileError, ScriptRuntimeError, InvalidPath, Io, PlatformError
  - Implement Display with actionable messages
  - Purpose: Structured error handling
  - _Requirements: REQ-3_
  - _Prompt: Implement the task for spec dev-tooling, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Developer | Task: Create core/src/error.rs with KeyRxError enum using thiserror::Error derive. Variants: UnknownKey { key: String }, ScriptCompileError { message: String, line: Option<usize>, column: Option<usize> }, ScriptRuntimeError { message: String }, InvalidPath { path: String, reason: String }, Io(#[from] std::io::Error), PlatformError { message: String }. Add impl From<Box<rhai::EvalAltResult>> for KeyRxError | Restrictions: All variants must have actionable error messages, use #[error("...")] attribute | Success: Error types compile, can be converted from common error sources. Mark task [-] in tasks.md before starting, log implementation with log-implementation tool after completion, then mark [x] when complete._

- [x] 4.2 Export error module from lib.rs
  - File: `core/src/lib.rs`
  - Add `pub mod error;`
  - Add `pub use error::KeyRxError;`
  - Purpose: Make errors available to consumers
  - _Requirements: REQ-3_
  - _Prompt: Implement the task for spec dev-tooling, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Developer | Task: Update core/src/lib.rs to add pub mod error and pub use error::KeyRxError to re-exports | Restrictions: Preserve existing exports | Success: KeyRxError is importable from keyrx_core. Mark task [-] in tasks.md before starting, log implementation with log-implementation tool after completion, then mark [x] when complete._

## Task 5: Refactor RhaiRuntime Error Handling

- [x] 5.1 Remove Rc<RefCell> pattern
  - File: `core/src/scripting/runtime.rs`
  - Replace `Rc<RefCell<RemapRegistry>>` with owned `RemapRegistry`
  - Store reference in Engine instead of cloning Rc
  - Remove RefCell borrow_mut calls
  - Purpose: Simpler ownership, no runtime borrow panics
  - _Requirements: REQ-7_
  - _Prompt: Implement the task for spec dev-tooling, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Developer | Task: Refactor core/src/scripting/runtime.rs to remove Rc<RefCell<RemapRegistry>>. Store RemapRegistry directly in RhaiRuntime struct. For Rhai function registration, use a different approach: store pending operations in a Vec and apply them after script execution, OR use rhai's NativeCallContext to access engine state. The lookup_remap method should return from owned registry | Restrictions: Must not introduce runtime panics, maintain thread safety | Success: No Rc<RefCell> in runtime.rs, all tests pass. Mark task [-] in tasks.md before starting, log implementation with log-implementation tool after completion, then mark [x] when complete._

- [ ] 5.2 Add error returns to Rhai functions
  - File: `core/src/scripting/runtime.rs`
  - Change remap/block/pass to return Result via Rhai's error mechanism
  - Use rhai::EvalAltResult for script-visible errors
  - Log warnings AND return error to script
  - Purpose: No silent failures in scripts
  - _Requirements: REQ-3_
  - _Prompt: Implement the task for spec dev-tooling, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Developer with Rhai experience | Task: Update remap/block/pass function registrations in core/src/scripting/runtime.rs to return errors to the script. Use engine.register_result_fn instead of register_fn. Return Err(Box::new(EvalAltResult::ErrorRuntime(...))) for unknown keys. Keep tracing::warn for logging but ALSO return the error | Restrictions: Errors must be catchable in Rhai scripts with try/catch, maintain backwards compatibility for valid key names | Success: Invalid key names cause script errors that can be caught. Mark task [-] in tasks.md before starting, log implementation with log-implementation tool after completion, then mark [x] when complete._

- [ ] 5.3 Fix RhaiRuntime Default impl
  - File: `core/src/scripting/runtime.rs`
  - Remove expect() from Default impl
  - Either: remove Default impl, or return a "null" runtime that errors on use
  - Purpose: No panics on initialization failure
  - _Requirements: REQ-3_
  - _Prompt: Implement the task for spec dev-tooling, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Developer | Task: Remove or fix Default impl for RhaiRuntime in core/src/scripting/runtime.rs. Option 1: Remove Default impl entirely (callers use new()). Option 2: Return runtime with error state that returns Err on any operation. Prefer Option 1 for simplicity | Restrictions: Must not panic, update any code that relies on Default | Success: No expect() calls in runtime.rs. Mark task [-] in tasks.md before starting, log implementation with log-implementation tool after completion, then mark [x] when complete._

## Task 6: Refactor CLI Error Handling

- [ ] 6.1 Remove process::exit from check command
  - File: `core/src/cli/commands/check.rs`
  - Replace std::process::exit(2) with return Err(...)
  - Use anyhow::bail! or return KeyRxError
  - Let main.rs handle exit codes
  - Purpose: Testable CLI commands
  - _Requirements: REQ-3_
  - _Prompt: Implement the task for spec dev-tooling, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Developer | Task: Update core/src/cli/commands/check.rs to remove std::process::exit(2). Replace with returning anyhow::Error or KeyRxError::ScriptCompileError. The run() method should return Result<()> and propagate errors | Restrictions: Exit codes should be handled in main.rs only | Success: CheckCommand::run() returns Err on invalid scripts, no process::exit calls. Mark task [-] in tasks.md before starting, log implementation with log-implementation tool after completion, then mark [x] when complete._

- [ ] 6.2 Fix path handling in run command
  - File: `core/src/cli/commands/run.rs`
  - Replace path.to_str().unwrap_or_default() with proper error
  - Return KeyRxError::InvalidPath for non-UTF8 paths
  - Purpose: Clear error messages for path issues
  - _Requirements: REQ-3_
  - _Prompt: Implement the task for spec dev-tooling, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Developer | Task: Update core/src/cli/commands/run.rs to fix path handling. Replace to_str().unwrap_or_default() with: path.to_str().ok_or_else(|| anyhow::anyhow!("Invalid UTF-8 in path: {:?}", path))? | Restrictions: Provide clear error message including the problematic path | Success: Non-UTF8 paths produce clear error message. Mark task [-] in tasks.md before starting, log implementation with log-implementation tool after completion, then mark [x] when complete._

- [ ] 6.3 Update main.rs error handling
  - File: `core/src/bin/keyrx.rs`
  - Centralize exit code logic in main
  - Exit 1 for general errors, 2 for validation errors
  - Print errors to stderr with context
  - Purpose: Consistent exit code handling
  - _Requirements: REQ-3_
  - _Prompt: Implement the task for spec dev-tooling, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Developer | Task: Update core/src/bin/keyrx.rs main() to handle errors centrally. Wrap command execution in match, print errors to stderr with eprintln!, exit with code 1 for runtime errors. If error is KeyRxError::ScriptCompileError, exit with code 2 | Restrictions: Keep main() simple, delegate to commands | Success: CLI returns proper exit codes, errors printed to stderr. Mark task [-] in tasks.md before starting, log implementation with log-implementation tool after completion, then mark [x] when complete._

## Task 7: Enhanced Mock Infrastructure

- [ ] 7.1 Add call tracking to MockInput
  - File: `core/src/mocks/mock_input.rs`
  - Add CallTracker field with Vec<MockCall>
  - Record all method calls (start, stop, poll_events, send_output)
  - Add call_history() -> &[MockCall] method
  - Add with_error_on_start(error) builder method
  - Purpose: Enable behavior verification in tests
  - _Requirements: REQ-5_
  - _Prompt: Implement the task for spec dev-tooling, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Developer | Task: Update core/src/mocks/mock_input.rs to add call tracking. Add MockCall enum with variants Start, Stop, PollEvents, SendOutput(OutputAction). Add calls: Vec<MockCall> field. Record each method call. Add call_history(&self) -> &[MockCall]. Add with_error_on_start(anyhow::Error) -> Self builder that causes start() to return error | Restrictions: Keep existing functionality working | Success: Tests can verify which methods were called and in what order. Mark task [-] in tasks.md before starting, log implementation with log-implementation tool after completion, then mark [x] when complete._

- [ ] 7.2 Add call tracking to MockRuntime
  - File: `core/src/mocks/mock_runtime.rs`
  - Record calls to execute, call_hook, load_file, lookup_remap
  - Add configurable return values for lookup_remap
  - Add with_remap(from, to) builder for test setup
  - Purpose: Enable script runtime verification
  - _Requirements: REQ-5_
  - _Prompt: Implement the task for spec dev-tooling, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Developer | Task: Update core/src/mocks/mock_runtime.rs to add call tracking. Add MockRuntimeCall enum. Record all calls. Add with_remap(from: KeyCode, to: KeyCode) -> Self builder that configures lookup_remap to return Remap(to) for from. Add with_block(key: KeyCode) builder. Add call_history() method | Restrictions: Maintain ScriptRuntime trait compliance | Success: Tests can configure mock responses and verify calls. Mark task [-] in tasks.md before starting, log implementation with log-implementation tool after completion, then mark [x] when complete._

- [ ] 7.3 Add history tracking to MockState
  - File: `core/src/mocks/mock_state.rs`
  - Track all state mutations with timestamps
  - Add state_history() -> &[StateChange] method
  - Add assert_layer_activated(name) helper
  - Purpose: Enable state change verification
  - _Requirements: REQ-5_
  - _Prompt: Implement the task for spec dev-tooling, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Developer | Task: Update core/src/mocks/mock_state.rs to add history tracking. Add StateChange enum with variants like LayerActivated(String), ModifierSet(ModifierSet). Record all mutations. Add state_history() method. Add assert_layer_activated(&self, name: &str) that panics if layer wasn't activated | Restrictions: Keep StateStore trait compliance | Success: Tests can verify state mutations occurred. Mark task [-] in tasks.md before starting, log implementation with log-implementation tool after completion, then mark [x] when complete._

## Task 8: Documentation

- [ ] 8.1 Create SCRIPTING.md
  - File: `docs/SCRIPTING.md`
  - Document: remap(from, to), block(key), pass(key)
  - Document: on_init() hook lifecycle
  - Document: error handling (try/catch)
  - Include: complete examples
  - Purpose: Enable autonomous script development
  - _Requirements: REQ-4_
  - _Prompt: Implement the task for spec dev-tooling, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Technical Writer | Task: Create docs/SCRIPTING.md with: 1) Overview of Rhai scripting in KeyRx, 2) Function reference for remap(from, to), block(key), pass(key) with parameters, return values, errors, 3) Hook lifecycle explaining on_init(), 4) Error handling with try/catch examples, 5) Complete example scripts. Use markdown with code blocks | Restrictions: Keep examples runnable, document all error conditions | Success: AI agent can write valid scripts from documentation alone. Mark task [-] in tasks.md before starting, log implementation with log-implementation tool after completion, then mark [x] when complete._

- [ ] 8.2 Create KEYS.md
  - File: `docs/KEYS.md`
  - List all valid key names organized by category
  - Include all aliases (e.g., Esc = Escape)
  - Add examples of common remappings
  - Purpose: Complete key name reference
  - _Requirements: REQ-4_
  - _Prompt: Implement the task for spec dev-tooling, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Technical Writer | Task: Create docs/KEYS.md listing all valid key names from KeyCode enum. Organize by category: Letters (A-Z), Numbers (0-9), Function Keys (F1-F12), Modifiers, Navigation, Editing, Whitespace, Locks, Punctuation, Numpad, Media. For each key, list all accepted aliases. Extract this information from core/src/engine/types.rs FromStr implementation | Restrictions: Must match actual implementation exactly | Success: All key names and aliases documented. Mark task [-] in tasks.md before starting, log implementation with log-implementation tool after completion, then mark [x] when complete._

## Task 9: Update Trait Documentation

- [ ] 9.1 Document InputSource trait
  - File: `core/src/traits/input_source.rs`
  - Add module-level //! documentation
  - Document thread safety requirements (Send bound)
  - Document method contracts and error conditions
  - Purpose: Clear interface contracts
  - _Requirements: REQ-4_
  - _Prompt: Implement the task for spec dev-tooling, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Developer | Task: Update core/src/traits/input_source.rs with comprehensive documentation. Add //! module doc explaining purpose. For each method, document: what it does, when to call it, error conditions, thread safety. Explain why Send bound is required. Add example implementation sketch in doc comments | Restrictions: Keep docs accurate to actual behavior | Success: Trait is self-documenting for implementors. Mark task [-] in tasks.md before starting, log implementation with log-implementation tool after completion, then mark [x] when complete._

- [ ] 9.2 Document ScriptRuntime trait
  - File: `core/src/traits/script_runtime.rs`
  - Document method call order requirements
  - Document error handling expectations
  - Add example usage in doc comments
  - Purpose: Clear interface contracts
  - _Requirements: REQ-4_
  - _Prompt: Implement the task for spec dev-tooling, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Developer | Task: Update core/src/traits/script_runtime.rs with comprehensive documentation. Document: 1) Expected call order (load_file before run_script), 2) Error handling for each method, 3) What lookup_remap should return for unmapped keys, 4) Thread safety considerations. Add usage example in module docs | Restrictions: Document actual behavior | Success: Trait implementation requirements are clear. Mark task [-] in tasks.md before starting, log implementation with log-implementation tool after completion, then mark [x] when complete._

## Task 10: Verification and Cleanup

- [ ] 10.1 Run cargo fmt on entire codebase
  - Run: `cargo fmt` in core/
  - Verify: `cargo fmt --check` passes
  - Purpose: Establish clean formatting baseline
  - _Requirements: REQ-6_
  - _Prompt: Implement the task for spec dev-tooling, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Developer | Task: Run cargo fmt in core/ directory to format all code. Then run cargo fmt --check to verify. Fix any issues that prevent formatting | Restrictions: None | Success: cargo fmt --check exits 0. Mark task [-] in tasks.md before starting, log implementation with log-implementation tool after completion, then mark [x] when complete._

- [ ] 10.2 Fix all clippy warnings
  - Run: `cargo clippy -- -D warnings`
  - Fix: All warnings (derive suggestions, etc.)
  - Purpose: Clean lint baseline
  - _Requirements: REQ-6_
  - _Prompt: Implement the task for spec dev-tooling, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Developer | Task: Run cargo clippy -- -D warnings in core/ and fix all warnings. Common fixes: add #[derive(Default)] where suggested, use clippy::derive_partial_eq_without_eq if needed, fix any other suggestions | Restrictions: Don't suppress warnings, fix them | Success: cargo clippy -- -D warnings exits 0. Mark task [-] in tasks.md before starting, log implementation with log-implementation tool after completion, then mark [x] when complete._

- [ ] 10.3 Run all tests and verify
  - Run: `cargo test`
  - Verify: All 59+ tests pass
  - Run: `cargo test --release`
  - Purpose: Confirm no regressions
  - _Requirements: REQ-1, REQ-2_
  - _Prompt: Implement the task for spec dev-tooling, first run spec-workflow-guide to get the workflow guide then implement the task: Role: QA Engineer | Task: Run cargo test and cargo test --release in core/. All tests must pass. If any fail, investigate and fix. Document any test changes needed due to error handling refactors | Restrictions: All existing tests must pass or be updated appropriately | Success: All tests pass in both debug and release modes. Mark task [-] in tasks.md before starting, log implementation with log-implementation tool after completion, then mark [x] when complete._

- [ ] 10.4 Install and test pre-commit hooks
  - Run: `./scripts/install-hooks.sh`
  - Test: Make a commit with formatting issue, verify blocked
  - Test: Make a clean commit, verify accepted
  - Purpose: End-to-end hook verification
  - _Requirements: REQ-1_
  - _Prompt: Implement the task for spec dev-tooling, first run spec-workflow-guide to get the workflow guide then implement the task: Role: QA Engineer | Task: Run ./scripts/install-hooks.sh. Create test file with bad formatting, attempt commit (should fail). Fix formatting, commit again (should succeed). Delete test file. Verify hooks work end-to-end | Restrictions: Clean up test artifacts | Success: Pre-commit hooks block bad code, allow good code. Mark task [-] in tasks.md before starting, log implementation with log-implementation tool after completion, then mark [x] when complete._
