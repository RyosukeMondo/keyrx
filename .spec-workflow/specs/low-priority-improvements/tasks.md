# Tasks: Low-Priority Improvements

## Task List

- [x] 1. Implement BUG #37 hot-unplug regression test
  - File: keyrx_daemon/tests/bug_regression_tests.rs
  - Replace `todo!()` at line 474 with actual test implementation
  - Use VirtualLinuxKeyboard to simulate device removal
  - Verify daemon continues running and other devices work
  - Purpose: Prevent regression of hot-unplug crash bug
  - _Leverage: Existing VirtualLinuxKeyboard test utility, BUG #37 fix in daemon_
  - _Requirements: Requirement 1_
  - _Prompt: Role: Rust test engineer with Linux evdev expertise | Task: Implement regression test for BUG #37 hot-unplug crash following Requirement 1, using VirtualLinuxKeyboard to simulate device removal during daemon operation | Restrictions: Test must be deterministic (no race conditions), must verify daemon continues running after device removal, must verify remaining devices still function, must run in CI (not #[ignore]), use existing test infrastructure | Success: Test creates 2 virtual keyboards, daemon grabs both, removes one keyboard mid-operation, daemon doesn't crash, second keyboard still processes events, test passes consistently_

- [x] 2. Calculate actual source hash in compiler
  - File: keyrx_compiler/src/parser/core.rs
  - Replace `source_hash: "TODO".to_string()` at line 131 with SHA256 calculation
  - Read source file bytes and calculate hash
  - Store hash in hex format (consistent with binary hash)
  - Purpose: Enable tracing .krx files back to source Rhai files
  - _Leverage: Existing SHA256 usage in serialize.rs, std::fs::read for file loading_
  - _Requirements: Requirement 2_
  - _Prompt: Role: Rust developer with cryptography experience | Task: Calculate SHA256 hash of source Rhai file and store in Metadata.source_hash following Requirement 2, replacing "TODO" placeholder | Restrictions: Use sha2 crate (already in workspace), read source file from PathBuf, calculate hash same way as binary hash, store as lowercase hex string, handle file read errors gracefully (return error, don't panic) | Success: Metadata.source_hash contains SHA256 hex string, `keyrx_compiler verify` shows source hash, hash changes when source file changes, hash is deterministic (same file → same hash)_

- [x] 3. Handle signal test stubs (defer with documentation)
  - File: keyrx_daemon/tests/daemon_tests.rs
  - Mark tests at lines 591, 599, 607 as `#[ignore]` with explanation
  - Add doc comment explaining signal handling is tested in integration tests
  - Document why subprocess-based testing is deferred
  - Purpose: Remove `todo!()` panics from test suite
  - _Leverage: Existing #[ignore] pattern from flaky tests_
  - _Requirements: Requirement 3 (Option B)_
  - _Prompt: Role: Rust test maintainer | Task: Mark signal handling test stubs as ignored following Requirement 3 Option B, documenting why subprocess testing is deferred | Restrictions: Use #[ignore] attribute, add clear doc comment explaining: (1) signal handling IS tested in integration tests, (2) unit tests would require subprocess spawning, (3) deferred due to complexity vs benefit, tests must compile but not run in CI | Success: Three signal tests marked #[ignore], doc comments explain deferral rationale, `cargo test` doesn't panic on todo!(), CI passes_

- [x] 4. Document CheckBytes as feature candidate
  - File: docs/features_candidate/checkbytes-security.md
  - Create feature candidate document for CheckBytes security improvement
  - Document requirements, effort estimate, security benefits
  - Reference TODOs in serialize.rs and fuzz_deserialize.rs
  - Purpose: Preserve CheckBytes improvement for future consideration
  - _Leverage: Existing feature candidate template (mphf-lookup-system.md)_
  - _Requirements: Out of Scope section from requirements.md_
  - _Prompt: Role: Technical documentation writer with security expertise | Task: Create feature candidate document for CheckBytes security improvement, following feature candidate template, documenting why deferred and when to revisit | Restrictions: Follow structure of mphf-lookup-system.md, explain CheckBytes trait purpose (validates rkyv deserialization), document security benefit (prevents malformed .krx from causing UB), estimate 1-2 days effort, reference existing TODOs, provide conditions for revisiting | Success: Document created, explains what CheckBytes is, why deferred (not urgent since SHA256 verified), effort estimate, security rationale, linked from feature candidate README_

- [x] 5. Update CHANGELOG.md
  - File: CHANGELOG.md
  - Add entry for low-priority improvements
  - Document hot-unplug test, source hash, signal test deferral
  - Purpose: Record technical debt resolution
  - _Leverage: Existing CHANGELOG format_
  - _Requirements: Non-Functional: Documentation_
  - _Prompt: Role: Release notes writer | Task: Add CHANGELOG entry for low-priority improvements following existing format | Restrictions: Add to [Unreleased] section, use appropriate categories (Added/Changed/Fixed), mention specific changes (hot-unplug regression test, source hash metadata, signal tests marked ignored), keep concise, link to spec | Success: CHANGELOG updated, entry lists all changes, follows Keep a Changelog format_

- [x] 6. Verify all improvements
  - File: (workspace-wide verification)
  - Run `cargo test --workspace` to verify all tests pass
  - Run `cargo clippy --workspace -- -D warnings` to verify no warnings
  - Verify no `todo!()` macros remain in test code (except ignored tests)
  - Purpose: Ensure all improvements are complete and working
  - _Leverage: Existing CI verification scripts_
  - _Requirements: Success Criteria from requirements.md_
  - _Prompt: Role: QA engineer | Task: Verify all low-priority improvements are complete and working following Success Criteria | Restrictions: Run full test suite, run clippy with warnings-as-errors, grep for remaining todo!() in non-ignored tests, verify workspace compiles, check CHANGELOG updated | Success: All tests pass, no clippy warnings, no unhandled todo!() macros, workspace compiles successfully_
