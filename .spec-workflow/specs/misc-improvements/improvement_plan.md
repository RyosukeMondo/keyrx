# Prioritized Improvement Plan

**Date:** 2025-12-12
**Spec:** misc-improvements
**Task:** 2.1 - Create prioritized improvement plan

## Executive Summary

This plan consolidates findings from Phase 1 analysis tasks (1.1-1.5) into a prioritized list of the top 20 improvements with estimated effort and expected impact.

### Phase 1 Summary

| Analysis | Key Finding |
|----------|-------------|
| **Function Length (1.1)** | 90 functions > 50 lines; 14 critical (100+) |
| **Test Coverage (1.2)** | 81.25% overall (PASS); critical paths <90% (FAIL) |
| **Logging (1.3)** | Mostly compliant; minor naming differences |
| **Documentation (1.4)** | 1,069 missing items; 28 warnings |
| **Complexity (1.5)** | 6 functions >15 complexity; 12 functions 11-15 |

## Prioritization Matrix

| Priority | Criteria |
|----------|----------|
| **P1 (Critical)** | Critical path <90% coverage, functions >100 lines, missing API docs, logging violations |
| **P2 (High)** | Overall coverage gaps, functions 70-100 lines, important docs, complexity >15 |
| **P3 (Medium)** | Functions 50-70 lines, nice-to-have docs, complexity 11-15 |
| **P4 (Low)** | Optional improvements, polish items |

## Top 20 Prioritized Improvements

### P1: Critical (Must Do)

| # | Item | Category | Impact | Effort | Notes |
|---|------|----------|--------|--------|-------|
| 1 | Add tests for `engine/decision/timing.rs` (7.69% coverage) | Coverage | High | Medium | Critical timing logic |
| 2 | Refactor `apply()` - 163 lines, complexity 26 | Function Length | High | High | Core engine function |
| 3 | Add tests for `api.rs` (0% coverage) | Coverage | High | Medium | Critical API path |
| 4 | Refactor `create_validation_engine()` - 199 lines | Function Length | High | High | Validation core |
| 5 | Add tests for `services/profile.rs` (0% coverage) | Coverage | High | Low | Service interface |
| 6 | Add tests for `services/runtime.rs` (0% coverage) | Coverage | High | Low | Service interface |
| 7 | Document services module (24 items) | Documentation | High | Medium | External API |
| 8 | Refactor `html_header()` - 261 lines | Function Length | Medium | Medium | Template function |
| 9 | Refactor `render_ascii_keyboard()` - 172 lines | Function Length | Medium | Medium | Output function |
| 10 | Document FFI contracts (71 items) | Documentation | High | High | External interface |

### P2: High (Should Do)

| # | Item | Category | Impact | Effort | Notes |
|---|------|----------|--------|--------|-------|
| 11 | Add tests for `scripting/api/executor.rs` (28% coverage) | Coverage | High | High | Script execution |
| 12 | Refactor `run_command()` - 143 lines | Function Length | Medium | Medium | CLI dispatcher |
| 13 | Refactor `validate_session_transition()` - 131 lines, complexity 19 | Function Length | High | Medium | Transition logic |
| 14 | Add tests for `engine/event_loop.rs` (59% coverage) | Coverage | Medium | High | Event handling |
| 15 | Document error types (31 items) | Documentation | Medium | Medium | Error troubleshooting |
| 16 | Fix logging field names (target→service, message→event) | Logging | Low | Low | Compliance |

### P3: Medium (Nice to Have)

| # | Item | Category | Impact | Effort | Notes |
|---|------|----------|--------|--------|-------|
| 17 | Refactor remaining 10 functions (70-99 lines) | Function Length | Medium | High | Phase 3.2 |
| 18 | Add tests for `validation/engine/rhai_engine.rs` (55% coverage) | Coverage | Medium | Medium | Validation |
| 19 | Fix 28 documentation warnings | Documentation | Low | Low | Clean builds |
| 20 | Document engine state types (~100 items) | Documentation | Medium | High | Internal docs |

### P4: Low (If Time Permits)

| # | Item | Category | Impact | Effort | Notes |
|---|------|----------|--------|--------|-------|
| - | Refactor 46 functions (51-69 lines) | Function Length | Low | Very High | Diminishing returns |
| - | Change timestamp to ISO 8601 | Logging | Low | Low | Nice to have |
| - | Change log level to UPPERCASE | Logging | Low | Low | Cosmetic |
| - | Document CLI commands (~100 items) | Documentation | Low | High | Internal |
| - | Document config models (40 items) | Documentation | Low | Medium | Config types |

## Detailed Breakdown

### Function Length Improvements (from Task 1.1)

**Critical (100+ lines) - 14 functions to refactor:**

| Priority | Function | Lines | Complexity | File |
|----------|----------|-------|------------|------|
| P1 | `html_header` | 261 | 3 | scripting/docs/generators/html/templates.rs |
| P1 | `create_validation_engine` | 199 | 16 | validation/engine/rhai_engine.rs |
| P1 | `render_ascii_keyboard` | 172 | 8 | validation/coverage.rs |
| P1 | `apply` | 163 | 26 | engine/state/mod.rs |
| P1 | `run_command` | 143 | 14 | bin/keyrx/dispatch.rs |
| P2 | `validate_session_transition` | 131 | 19 | engine/transitions/graph.rs |
| P2 | `build_function_registry` | 124 | 8 | scripting/sandbox/function_capabilities.rs |
| P2 | `print_human_result` | 114 | 10 | cli/commands/check.rs |
| P2 | `migrate` | 112 | 13 | migration/v1_to_v2.rs |
| P2 | `html_scripts` | 112 | 2 | scripting/docs/generators/html/templates.rs |
| P2 | `calibrate` | 108 | 9 | cli/commands/hardware.rs |
| P2 | `list_keyboards` | 101 | 7 | drivers/windows/device.rs |
| P2 | `evaluate` | 100 | 15 | metrics/alerts.rs |
| P2 | `from_streaming_file` | 100 | 9 | engine/replay.rs |

### Coverage Improvements (from Task 1.2)

**Critical Path Files (target: 90%):**

| Priority | File | Current | Gap | Notes |
|----------|------|---------|-----|-------|
| P1 | `engine/decision/timing.rs` | 7.69% | 82.31% | Critical timing logic |
| P1 | `api.rs` | 0.00% | 90.00% | API endpoints |
| P1 | `services/profile.rs` | 0.00% | 90.00% | Service interface |
| P1 | `services/runtime.rs` | 0.00% | 90.00% | Service interface |
| P2 | `scripting/api/executor.rs` | 28.03% | 61.97% | Script execution |
| P2 | `scripting/api/functions/core.rs` | 22.15% | 67.85% | Scripting API |
| P2 | `engine/event_loop.rs` | 59.04% | 30.96% | Event handling |
| P3 | `validation/engine/rhai_engine.rs` | 55.04% | 34.96% | Validation |
| P3 | `engine/layer_actions.rs` | 47.71% | 42.29% | Layer actions |

### Logging Improvements (from Task 1.3)

| Priority | Item | Status | Action |
|----------|------|--------|--------|
| P2 | Rename `target` → `service` | PARTIAL | Add serde rename |
| P2 | Rename `message` → `event` | PARTIAL | Add serde rename |
| P4 | ISO 8601 timestamp | PARTIAL | Optional - Unix ms widely used |
| P4 | UPPERCASE log levels | PARTIAL | Cosmetic change |

**Overall:** Logging is functionally compliant. Serde renames are low effort if strict compliance needed.

### Documentation Improvements (from Task 1.4)

| Priority | Items | Count | Focus |
|----------|-------|-------|-------|
| P1 | Services API | 24 | External consumers |
| P1 | FFI contracts | 71 | Dart/Flutter bindings |
| P2 | Error types | 31 | User troubleshooting |
| P2 | Module-level docs | 21 | Entry points |
| P3 | Engine state types | ~100 | Internal understanding |
| P3 | Fix warnings | 28 | Clean cargo doc |
| P4 | CLI commands | ~100 | Internal tooling |
| P4 | Config models | 40 | Configuration |

### Complexity Improvements (from Task 1.5)

**Very High Complexity (>15) - 6 functions:**

| Priority | Function | Complexity | Lines | Strategy |
|----------|----------|------------|-------|----------|
| P1 | `apply` | 26 | 163 | Extract per-mutation handlers |
| P2 | `validate_session_transition` | 19 | 131 | Extract validation helpers |
| P2 | `analyze` | 17 | 87 | Extract pending op processing |
| P2 | `create_validation_engine` | 16 | 199 | Group registrations |
| P2 | `evaluate` | 15 | 100 | Create threshold helpers |
| P3 | `process_event_traced` | 15 | 88 | Extract build helpers |

## Effort Estimates

| Effort | Definition | Examples |
|--------|------------|----------|
| **Low** | < 2 hours | Serde renames, small docs, simple tests |
| **Medium** | 2-4 hours | Medium function refactoring, module docs, integration tests |
| **High** | 4-8 hours | Large refactoring, complex tests, comprehensive docs |
| **Very High** | 8+ hours | Major rewrites, full module documentation |

## Dependency Considerations

1. **DI Spec**: Already completed - mocks available for testing
2. **Split Large Files Spec**: Some overlap - coordinate refactoring
3. **Fix Failing Tests Spec**: Completed - coverage measurement reliable

## Expected Impact

After completing all P1 and P2 items:

| Metric | Before | After | Status |
|--------|--------|-------|--------|
| Functions > 50 lines | 90 | ~45 | Improved |
| Overall coverage | 81.25% | ~85% | Improved |
| Critical path coverage | <50% | ≥80% | Improved |
| Missing critical docs | 95+ | ~0 | Resolved |
| Logging compliance | Partial | Full | Resolved |

## Recommended Execution Order

### Phase 3: Function Length (Tasks 3.1, 3.2)
1. `html_header` (261 lines) - Template extraction
2. `create_validation_engine` (199 lines) - Group registrations
3. `render_ascii_keyboard` (172 lines) - Section extraction
4. `apply` (163 lines, complexity 26) - Handler extraction
5. `run_command` (143 lines) - Command grouping

### Phase 4: Test Coverage (Tasks 4.1, 4.2)
1. `engine/decision/timing.rs` - Critical timing
2. `api.rs` - API endpoints
3. `services/profile.rs` + `services/runtime.rs` - Service traits
4. `scripting/api/executor.rs` - Script execution
5. `engine/event_loop.rs` - Event handling

### Phase 5: Logging and Documentation (Tasks 5.1, 5.2, 5.3)
1. Fix logging field names (serde renames)
2. Document services module (24 items)
3. Document FFI contracts (71 items)
4. Document error types (31 items)
5. Fix 28 documentation warnings

## Summary

This plan provides a clear path forward with prioritized improvements:

- **14 functions** need length refactoring (critical path)
- **7 files** need critical coverage improvements
- **~125 items** need critical documentation
- **2 logging fields** need renaming for compliance

Total estimated effort: **40-60 hours** for P1+P2 items.

Recommendation: Execute phases sequentially, committing improvements incrementally. Each phase builds on the previous, with function refactoring making code more testable, and tests validating refactoring correctness.
