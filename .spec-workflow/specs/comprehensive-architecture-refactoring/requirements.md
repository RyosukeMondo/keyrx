# Requirements Document

## 1. Functional Requirements

### FR1: File Size Compliance
- **Description**: All source files must comply with the 500-line limit (excluding comments and blank lines) to improve maintainability and enforce Single Responsibility Principle
- **Acceptance Criteria**:
  - [ ] All files in keyrx_core ≤500 lines
  - [ ] All files in keyrx_daemon ≤500 lines
  - [ ] All files in keyrx_compiler ≤500 lines
  - [ ] All test files ≤500 lines (or split into logical groups)
  - [ ] Verification script confirms compliance
- **Priority**: CRITICAL
- **Related Issues**:
  - tap_hold.rs (3614 lines)
  - e2e_harness.rs (3523 lines)
  - parser_function_tests.rs (2864 lines)
  - linux/mod.rs (1952 lines)
  - event.rs (1733 lines)
  - output_capture.rs (1664 lines)
  - daemon/mod.rs (1591 lines)
  - state.rs (1224 lines)
  - web/api.rs (1206 lines)
  - cli/config.rs (914 lines)
  - Plus 13 more files in 500-1000 line range

### FR2: Eliminate Global State
- **Description**: Remove all global static state to enable testability, thread safety, and dependency injection
- **Acceptance Criteria**:
  - [ ] MACRO_RECORDER OnceLock converted to injected dependency
  - [ ] Windows BRIDGE_CONTEXT/BRIDGE_HOOK converted to instance state
  - [ ] Test utilities SENDER converted to dependency injection
  - [ ] All platform-specific static variables eliminated
  - [ ] Zero `static` declarations with interior mutability in production code
- **Priority**: CRITICAL
- **Related Issues**:
  - web/api.rs:16 - MACRO_RECORDER OnceLock
  - platform/windows/rawinput.rs - BRIDGE_CONTEXT, BRIDGE_HOOK RwLock globals
  - test_utils/output_capture.rs - SENDER RwLock global

### FR3: Implement SOLID Architecture Principles
- **Description**: Refactor modules to follow Single Responsibility, Open/Closed, Liskov Substitution, Interface Segregation, and Dependency Inversion principles
- **Acceptance Criteria**:
  - [ ] Each module has one clear responsibility
  - [ ] Platform abstraction uses trait-based extension (not hard-coded dispatch)
  - [ ] All platform implementations are substitutable
  - [ ] Interfaces are focused and minimal
  - [ ] All dependencies inverted through trait abstractions
- **Priority**: HIGH
- **Related Issues**:
  - SRP violations: linux/mod.rs, daemon/mod.rs, web/api.rs
  - OCP violations: cli/config.rs hard-coded dispatch
  - ISP violations: ConfigManager fat interface
  - DIP violations: Hard-coded concrete platform types

### FR4: Create Service Layer Abstraction
- **Description**: Implement a service layer to deduplicate business logic between CLI and Web API interfaces
- **Acceptance Criteria**:
  - [ ] ProfileService handles all profile operations
  - [ ] ConfigService handles all configuration operations
  - [ ] DeviceService handles all device operations
  - [ ] MacroService handles all macro operations
  - [ ] CLI and Web API both delegate to services (no duplication)
  - [ ] Services are fully testable in isolation
- **Priority**: HIGH
- **Related Issues**:
  - CLI and Web API duplicate profile management logic
  - CLI and Web API duplicate configuration logic
  - No clear service boundary

### FR5: Dependency Injection Infrastructure
- **Description**: All external dependencies (APIs, file systems, platform code) must be injectable for testing
- **Acceptance Criteria**:
  - [ ] Hard-coded socket paths configurable via environment or config
  - [ ] Hard-coded URLs/ports injected via configuration
  - [ ] Platform implementations injectable via traits
  - [ ] File system operations mockable for tests
  - [ ] All services accept dependencies via constructor injection
- **Priority**: HIGH
- **Related Issues**:
  - ipc/mod.rs: Hard-coded /tmp/keyrx-daemon.sock
  - main.rs: Hard-coded localhost:9867
  - Non-injectable dependencies throughout

### FR6: Error Handling Standards
- **Description**: Implement consistent error handling with proper propagation and no panic-on-error in production code
- **Acceptance Criteria**:
  - [ ] Zero unwrap() calls in production code (test code acceptable)
  - [ ] Zero expect() calls in production code (test code acceptable)
  - [ ] All errors propagate with ? operator or proper Result handling
  - [ ] Custom error types with proper Display implementation
  - [ ] Error context preserved through error chain
- **Priority**: MEDIUM
- **Related Issues**:
  - 297 unwrap/expect instances across keyrx_daemon
  - ~20 instances in production code paths

### FR7: Code Duplication Elimination
- **Description**: Extract all duplicated logic into shared utility modules
- **Acceptance Criteria**:
  - [ ] Error conversion logic centralized
  - [ ] Validation logic shared between CLI and API
  - [ ] Output formatting utilities shared across CLI commands
  - [ ] Device discovery logic not duplicated in tests
  - [ ] Zero copy-pasted functions across modules
- **Priority**: MEDIUM
- **Related Issues**:
  - Error conversion duplicated in multiple modules
  - Validation duplicated in CLI and API
  - Output formatting duplicated in CLI handlers

### FR8: Test Coverage Improvement
- **Description**: Increase test coverage to meet project standards (≥80% overall, ≥90% critical paths)
- **Acceptance Criteria**:
  - [ ] keyrx_core coverage ≥90% (critical path)
  - [ ] keyrx_daemon coverage ≥80%
  - [ ] keyrx_compiler coverage ≥80%
  - [ ] Platform-specific code coverage ≥70%
  - [ ] All public APIs have unit tests
  - [ ] Integration tests cover E2E scenarios
- **Priority**: MEDIUM
- **Related Issues**:
  - main.rs: No unit tests
  - daemon/mod.rs: Insufficient signal handling tests
  - platform/windows: ~10% coverage
  - platform/linux: ~5% coverage

### FR9: Documentation Completeness
- **Description**: All public APIs must have comprehensive rustdoc documentation with examples
- **Acceptance Criteria**:
  - [ ] All public functions have rustdoc comments
  - [ ] All public traits have rustdoc with examples
  - [ ] All public structs/enums have field documentation
  - [ ] Complex algorithms have inline documentation
  - [ ] Architecture patterns documented in module-level docs
  - [ ] cargo doc builds without warnings
- **Priority**: MEDIUM
- **Related Issues**:
  - ~40% public API coverage currently
  - Missing: Daemon::run(), LinuxPlatform::init(), Platform trait examples
  - All CLI handler functions undocumented

### FR10: CheckBytes Implementation
- **Description**: Implement safe deserialization with CheckBytes for WASM to prevent security vulnerabilities from untrusted input
- **Acceptance Criteria**:
  - [ ] CheckBytes implemented for all serialized types
  - [ ] Fuzzing tests validate no panics on malformed input
  - [ ] WASM deserialization safe from untrusted sources
  - [ ] Security audit passes
- **Priority**: CRITICAL
- **Related Issues**:
  - fuzz/fuzz_targets/fuzz_deserialize.rs - TODO comment
  - compiler/src/serialize.rs - Security concern noted

## 2. Non-Functional Requirements

### NFR1: Code Quality Metrics
- **File size limit**: ≤500 lines (excluding comments/blanks)
- **Function size limit**: ≤50 lines
- **Test coverage**: ≥80% overall, ≥90% critical paths
- **Clippy warnings**: 0 (treat warnings as errors)
- **Rustfmt**: All code formatted (enforce in CI)

### NFR2: Performance
- **Build time**: No regression (must remain ≤current)
- **Test execution**: No regression (must remain ≤current)
- **Runtime performance**: No measurable regression in event processing latency
- **Memory usage**: No increase from current baseline

### NFR3: Backward Compatibility
- **Public APIs**: No breaking changes unless explicitly documented
- **CLI behavior**: Identical output format and exit codes
- **Configuration format**: .krx files remain compatible
- **IPC protocol**: No breaking changes to socket/pipe protocol

### NFR4: Platform Support
- **Linux**: All functionality maintained
- **Windows**: All functionality maintained
- **Cross-platform**: Trait abstractions support future platforms (macOS, BSD)

## 3. Technical Requirements

### TR1: Dependencies
**Existing (no changes required):**
- rkyv - Binary serialization
- evdev (Linux) - Input device handling
- windows-sys (Windows) - Win32 API bindings
- axum - Web server framework
- clap - CLI argument parsing
- serde - JSON serialization
- tokio - Async runtime

**To be added:**
- None (use existing dependencies)

**To be removed:**
- None

### TR2: Development Tools
- cargo-tarpaulin - Code coverage analysis
- cargo-clippy - Linting (already installed)
- cargo-fmt - Code formatting (already installed)
- cargo-fuzz - Fuzzing for CheckBytes validation

### TR3: Build System Changes
- Add file size verification to CI pipeline
- Add coverage threshold checks to CI
- Add CheckBytes validation to security checks

## 4. Constraints

### C1: Refactoring Strategy
- **Incremental changes only**: No "big bang" rewrites
- **Test-driven**: All tests must pass before and after each change
- **Backward compatible**: No breaking changes to public APIs

### C2: Timeline
- **Phase 1 (Critical)**: 2-3 weeks - Global state removal, CheckBytes
- **Phase 2 (High)**: 3-4 weeks - File size compliance, SOLID refactoring
- **Phase 3 (Medium)**: 2-3 weeks - Documentation, test coverage

### C3: Resource Constraints
- **No new external dependencies** unless critical
- **No changes to .krx binary format** (maintain compatibility)
- **No UI changes** (backend refactoring only)

## 5. Success Metrics

### Code Quality
- [ ] Zero files exceed 500-line limit
- [ ] Zero clippy warnings in production code
- [ ] Zero rustfmt check failures
- [ ] Code coverage ≥80% (≥90% for keyrx_core)

### Architecture
- [ ] Zero global static state in production code
- [ ] All modules follow Single Responsibility Principle
- [ ] All platform code behind trait abstractions
- [ ] Service layer successfully abstracts business logic

### Testing
- [ ] All critical paths have unit tests
- [ ] All platform-specific code has integration tests
- [ ] Fuzzing tests validate CheckBytes implementation
- [ ] Zero #[ignore] tests (all tests enabled and passing)

### Documentation
- [ ] cargo doc builds without warnings
- [ ] All public APIs documented with examples
- [ ] Architecture decision records (ADRs) created for major changes

## 6. Out of Scope

The following are explicitly NOT included in this refactoring:

- ❌ New features or functionality
- ❌ Performance optimizations (unless fixing regressions)
- ❌ UI/UX changes
- ❌ Configuration format changes
- ❌ Binary protocol changes
- ❌ Algorithm improvements (tap-hold, DFA, etc.)
- ❌ Platform expansion (macOS, BSD support)

## 7. Risk Assessment

### High Risk
- **Risk**: Breaking changes to platform-specific code during refactoring
- **Mitigation**: Comprehensive integration tests before and after each change

### Medium Risk
- **Risk**: Test coverage reveals existing bugs
- **Mitigation**: Fix bugs as separate tasks, prioritize by severity

### Low Risk
- **Risk**: File splits create merge conflicts with other work
- **Mitigation**: Coordinate with team, use feature branches
