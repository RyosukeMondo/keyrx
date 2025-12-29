# Requirements: Low-Priority Improvements

## Introduction

This spec addresses low-priority technical debt and improvements identified during the TODO audit (2025-12-29). These items don't block functionality but improve code quality, test coverage, and metadata completeness.

## Scope

**In Scope:**
- BUG #37 hot-unplug regression test implementation
- Source hash calculation for metadata traceability
- Signal handling test completion or deferral
- Documentation of CheckBytes security improvement (separate feature candidate)

**Out of Scope:**
- CheckBytes implementation (moved to feature candidate - requires significant effort)
- Breaking changes or new features
- Performance optimizations

## Requirements

### Requirement 1: Hot-Unplug Regression Test

**User Story:** As a developer, I want a regression test for BUG #37 (hot-unplug crash) so that we prevent this bug from reoccurring.

**Context:** BUG #37 was a real bug where unplugging a device during operation caused a daemon crash. The fix is implemented, but the regression test is stubbed out.

**Acceptance Criteria:**
1. Test SHALL simulate device hot-unplug using VirtualLinuxKeyboard
2. Test SHALL verify daemon continues running after device removal
3. Test SHALL verify remaining devices continue to function
4. Test SHALL run automatically in CI (not marked `#[ignore]`)

**Priority:** Medium

---

### Requirement 2: Source Hash Metadata

**User Story:** As a user, I want the .krx file to include a hash of the source Rhai file so I can trace which source generated each binary.

**Context:** The `source_hash` field in metadata currently contains placeholder "TODO" string.

**Acceptance Criteria:**
1. Compiler SHALL calculate SHA256 hash of source Rhai file
2. Hash SHALL be stored in Metadata.source_hash field
3. Hash SHALL be displayed by `keyrx_compiler verify` command
4. Hash SHALL enable tracing .krx → source.rhai

**Priority:** Low

---

### Requirement 3: Signal Handling Tests

**User Story:** As a developer, I want signal handling tests to be either implemented or explicitly deferred with documentation.

**Context:** Three signal tests (SIGTERM, SIGINT, SIGHUP) have `todo!()` stubs.

**Acceptance Criteria:**
1. Tests SHALL either:
   - **Option A:** Be implemented using subprocess spawning, OR
   - **Option B:** Be marked `#[ignore]` with clear explanation of why deferred
2. If deferred, documentation SHALL explain that signal handling is tested in integration tests
3. CI SHALL not have failing/panicking tests

**Priority:** Low

---

## Non-Functional Requirements

### Code Quality
- All implementations SHALL follow existing code style
- No new `unsafe` code unless absolutely necessary
- All code SHALL pass clippy with `-D warnings`

### Testing
- All new code SHALL have unit tests
- Test coverage SHALL not decrease
- Tests SHALL be deterministic (no flaky tests)

### Documentation
- All changes SHALL be documented in code comments
- CHANGELOG.md SHALL be updated

## Success Criteria

**Definition of Done:**
- ✅ All `todo!()` macros removed or explained
- ✅ All tests pass in CI
- ✅ Source hash appears in .krx metadata
- ✅ Hot-unplug regression test passes
- ✅ Workspace compiles with no warnings

## Out of Scope (Future Work)

**CheckBytes Security Improvement** - Deferred to feature candidate
- Requires implementing CheckBytes trait for all config types
- ~1-2 days effort
- See: `docs/features_candidate/checkbytes-security.md` (to be created)
