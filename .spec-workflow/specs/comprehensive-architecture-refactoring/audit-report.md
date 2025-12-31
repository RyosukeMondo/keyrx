# Comprehensive Technical Debt Audit Report

**Spec**: comprehensive-architecture-refactoring
**Date**: 2025-12-31
**Audit Tool**: find-and-fix (autonomous technical debt discovery)

---

## Executive Summary

Comprehensive audit of keyrx2 codebase identified **34 violations** across 9 categories:
- **🔴 CRITICAL**: 5 issues (global state, missing implementations, file sizes)
- **🟠 HIGH**: 8 issues (SOLID violations, architecture anti-patterns)
- **🟡 MEDIUM**: 12 issues (test coverage gaps, documentation)
- **⚪ LOW**: 9 issues (code quality improvements)

**Total Files Scanned**: 127 source files
**Violations Found**: 34 distinct issues
**Spec Generated**: comprehensive-architecture-refactoring (18 tasks, 5 phases)

---

## 1. FILE SIZE VIOLATIONS (Critical)

### Summary
- **Count**: 23 files exceed 500-line limit
- **Severity**: CRITICAL (14 files), HIGH (9 files)
- **Impact**: Violates Single Responsibility Principle, reduces maintainability

### Critical Files (>1000 lines)

| File | Lines | Ratio | Recommendation |
|------|-------|-------|----------------|
| keyrx_core/src/runtime/tap_hold.rs | 3,614 | 7.2x | Split into 8 modules (state_machine, event_processor, timeout_handler, types, testing/) |
| keyrx_daemon/tests/e2e_harness.rs | 3,523 | 7.0x | Extract to test_utils/e2e/ (7 modules: base, linux, windows, simulation, injection, assertions) |
| keyrx_compiler/tests/parser_function_tests.rs | 2,864 | 5.7x | Split by feature (maps, taps, modifiers, macros, layers, validation) |
| keyrx_daemon/src/platform/linux/mod.rs | 1,952 | 3.9x | Extract input_capture.rs, output_injection.rs, device_discovery.rs |
| keyrx_core/src/runtime/event.rs | 1,733 | 3.5x | Extract event types to dedicated module |
| keyrx_daemon/src/test_utils/output_capture.rs | 1,664 | 3.3x | Extract virtual_keyboard_setup.rs, event_capture.rs |
| keyrx_daemon/src/daemon/mod.rs | 1,591 | 3.2x | Extract signal_handler.rs, event_loop.rs |
| keyrx_core/src/runtime/state.rs | 1,224 | 2.4x | Extract modifiers logic |
| keyrx_daemon/src/web/api.rs | 1,206 | 2.4x | Split by domain (devices, profiles, config, macros, metrics) |
| keyrx_daemon/src/cli/config.rs | 914 | 1.8x | Extract handlers by category (map, tap, layer) |

**Total Lines in Violations**: ~21,000 lines need refactoring

---

## 2. GLOBAL STATE VIOLATIONS (Critical)

### Summary
- **Count**: 6 static variables with interior mutability
- **Severity**: CRITICAL (3), HIGH (3)
- **Impact**: Prevents testability, causes thread-safety issues

### Critical Instances

**1. MACRO_RECORDER Singleton** (web/api.rs:16)
```rust
static MACRO_RECORDER: OnceLock<MacroRecorder> = OnceLock::new();
```
- **Problem**: Not injectable, blocks unit testing of macro endpoints
- **Fix**: Move to AppState with Arc<MacroRecorder>

**2. Windows Bridge State** (platform/windows/rawinput.rs:39-40)
```rust
static BRIDGE_CONTEXT: RwLock<Option<BridgeContext>> = RwLock::new(None);
static BRIDGE_HOOK: RwLock<Option<isize>> = RwLock::new(None);
```
- **Problem**: Global RwLock synchronization overhead, prevents multiple instances
- **Fix**: Move to WindowsPlatform struct fields

**3. Test Utility Global** (test_utils/output_capture.rs:16)
```rust
static SENDER: RwLock<Option<Sender<KeyEvent>>> = RwLock::new(None);
```
- **Problem**: Prevents concurrent test execution, test flakiness
- **Fix**: Return OutputCapture handle instead of global

---

## 3. SOLID PRINCIPLE VIOLATIONS (High)

### Single Responsibility Principle (SRP)

**Violations**:
1. **linux/mod.rs** (1,952 lines): Device management + Input capture + Output injection + System tray
2. **daemon/mod.rs** (1,591 lines): Signal handling + Event loop + Device coordination + Config loading
3. **web/api.rs** (1,206 lines): Routing + Error handling + Business logic + Response formatting

**Fix**: Extract each responsibility to focused module

### Open/Closed Principle (OCP)

**Violations**:
1. **cli/config.rs** (914 lines): Hard-coded match dispatch for commands
```rust
match command {
    ConfigCommand::MapKey { ... } => handle_map_key(...),
    // ... dozens more cases
}
```
**Fix**: Trait-based command handler pattern

### Dependency Inversion Principle (DIP)

**Violations**:
1. **daemon/mod.rs**: Imports concrete platform types instead of traits
```rust
#[cfg(target_os = "linux")]
use crate::platform::linux::LinuxPlatform;
```
**Fix**: Use `Box<dyn Platform>` trait abstraction

---

## 4. DEPENDENCY INJECTION ISSUES (High)

### Hard-Coded Values

| Location | Value | Severity | Fix |
|----------|-------|----------|-----|
| ipc/mod.rs:14 | `/tmp/keyrx-daemon.sock` | CRITICAL | Environment variable / config |
| main.rs:359 | `localhost:9867` | MEDIUM | Config injection |
| useDaemonWebSocket.ts:40 | `ws://localhost:9867/ws` | MEDIUM | ApiContext |

### Non-Injectable Dependencies

- **MacroRecorder**: Global OnceLock (18 access sites)
- **Platform**: Direct imports instead of trait injection
- **ProfileManager**: No interface abstraction for mocking

---

## 5. CODE DUPLICATION (Medium)

### Duplicated Patterns

1. **Error Handling**: Lock failure handling duplicated across 3+ modules
2. **Validation**: Profile validation in both CLI and API layers
3. **Output Formatting**: JSON/text output in multiple CLI handlers
4. **Error Conversion**: From implementations duplicated in multiple modules

**Total Duplication**: ~400 lines estimated

---

## 6. ERROR HANDLING GAPS (Medium)

### unwrap/expect Abuse

- **Total**: 297 instances across keyrx_daemon
- **Production Code**: ~20 critical instances
- **Test Code**: ~277 instances (acceptable)

**Critical Instances**:
- main.rs:206 - `Runtime::new().expect(...)`
- ipc/unix_socket.rs:70 - `.unwrap()` on mutex

---

## 7. DOCUMENTATION GAPS (Medium)

### Missing rustdoc

| Module | Public Items | Documented | Coverage |
|--------|--------------|------------|----------|
| platform/mod.rs | 15 items | 6 items | 40% |
| daemon/mod.rs | 8 items | 2 items | 25% |
| services/* | 0 items (new) | 0 items | N/A |
| web/api.rs | 25 endpoints | 5 endpoints | 20% |

**Overall Documentation Coverage**: ~40% of public APIs

---

## 8. TEST COVERAGE GAPS (Medium)

### Coverage by Crate

| Crate | Current Coverage | Target | Gap |
|-------|------------------|--------|-----|
| keyrx_core | ~60% | ≥90% | +30% |
| keyrx_daemon | ~65% | ≥80% | +15% |
| keyrx_compiler | ~75% | ≥80% | +5% |
| keyrx_ui | ~70% | ≥80% | +10% |

### Critical Untested Paths

1. **main.rs** (991 lines): No unit tests for CLI parsing, shutdown
2. **daemon/mod.rs**: Only 19 tests for complex state machine
3. **platform/windows/**: ~10% coverage estimated
4. **platform/linux/**: ~5% coverage estimated

---

## 9. ARCHITECTURE ISSUES (High)

### Missing Service Layer

**Current**: CLI/Web API → Manager → Platform → OS
**Problem**: Business logic duplicated across transport layers

**Recommended**: CLI/Web API → Service → Manager → Platform → OS
**Benefit**: Single source of truth for business logic

### Tight Coupling

1. **Platform-specific code**: daemon/mod.rs uses #[cfg(target_os)] throughout
2. **IPC hard-wired**: Unix sockets on all platforms (Windows should use named pipes)
3. **No abstraction**: Platform code directly imported instead of trait dispatch

### Anti-Patterns

1. **Test code in production**: test_utils/output_capture.rs (1,664 lines) in main library
2. **Global state**: 6 static variables with interior mutability
3. **God objects**: linux/mod.rs doing everything for Linux platform

---

## 10. MIGRATION/REFACTORING DEBT

### Blockers

1. **CheckBytes Missing** (CRITICAL)
   - fuzz/fuzz_targets/fuzz_deserialize.rs - TODO comment
   - compiler/src/serialize.rs - Security concern noted
   - Impact: WASM deserialization unsafe from untrusted input

2. **Layer System** (MEDIUM)
   - wasm/simulation.rs - TODO for active layer extraction
   - Impact: Layer-based features incomplete

3. **E2E Test Skips** (MEDIUM)
   - tests/e2e_linux_multidevice.rs - 2x #[ignore]
   - Reason: "daemon can't find virtual keyboards"

---

## Remediation Plan

### Phase 1: Critical (Week 1-2)
1. Implement CheckBytes for security
2. Remove global MACRO_RECORDER
3. Remove Windows BRIDGE_CONTEXT/BRIDGE_HOOK
4. Remove test utility global SENDER
5. Create ProfileService foundation

### Phase 2: File Size (Week 3-5)
6. Split tap_hold.rs (3,614 → <500)
7. Split e2e_harness.rs (3,523 → <500)
8. Split parser_function_tests.rs (2,864 → <500)
9. Split linux/mod.rs (1,952 → <500)
10. Split web/api.rs (1,206 → <500)
11. Split remaining 13 files

### Phase 3: Architecture (Week 6-8)
12. Create Platform trait
13. Implement LinuxPlatform with trait
14. Implement WindowsPlatform with trait
15. Update Daemon to use trait abstraction
16. Wire CLI to services
17. Wire Web API to services

### Phase 4: Quality (Week 9-10)
18. Platform code tests (≥70% coverage)
19. Service layer tests (≥100% coverage)
20. Document all public APIs
21. Add integration tests

### Phase 5: Validation (Week 11)
22. Run comprehensive quality validation
23. Fix any remaining issues

---

## Quality Metrics

### Before Remediation
- Files >500 lines: 23 (FAIL)
- Global state: 6 instances (FAIL)
- Test coverage: 60-75% (BELOW TARGET)
- Documentation: 40% (BELOW TARGET)
- Clippy warnings: Unknown
- unwrap in production: ~20 instances (FAIL)

### After Remediation (Target)
- Files >500 lines: 0 (PASS)
- Global state: 0 instances (PASS)
- Test coverage: 80-90% (PASS)
- Documentation: 100% public APIs (PASS)
- Clippy warnings: 0 (PASS)
- unwrap in production: 0 instances (PASS)

---

## Risk Assessment

### High Risk
- **Breaking platform code during refactoring**
  - Mitigation: Comprehensive integration tests before/after each change

### Medium Risk
- **Test coverage reveals existing bugs**
  - Mitigation: Fix bugs as separate tasks, prioritize by severity

### Low Risk
- **Performance regression from trait dispatch**
  - Mitigation: Benchmark before/after, <5% overhead acceptable

---

This audit provides a complete roadmap from current technical debt to clean, testable, maintainable architecture following Rust best practices and SOLID principles.
