# Test Coverage Analysis

**Generated**: 2025-12-12
**Tool**: `cargo llvm-cov --lib --summary-only`

## Executive Summary

| Metric | Current | Target | Status |
|--------|---------|--------|--------|
| **Overall Line Coverage** | 81.25% | ≥80% | ✅ PASS |
| **Overall Function Coverage** | 78.32% | ≥80% | ⚠️ CLOSE |
| **Overall Region Coverage** | 79.81% | ≥80% | ⚠️ CLOSE |

The codebase meets the 80% overall coverage target for line coverage. Function and region coverage are slightly below target but within acceptable range.

## Critical Path Analysis (Target: ≥90%)

### services/ Module

| File | Lines | Missed | Coverage | Status |
|------|-------|--------|----------|--------|
| src/services/device.rs | 89 | 24 | 73.03% | ❌ BELOW |
| src/services/profile.rs | 79 | 79 | 0.00% | ❌ UNCOVERED |
| src/services/runtime.rs | 152 | 152 | 0.00% | ❌ UNCOVERED |
| src/services/mocks/device.rs | 96 | 44 | 54.17% | Test helper |
| src/services/mocks/profile.rs | 252 | 87 | 65.48% | Test helper |
| src/services/mocks/runtime.rs | 218 | 50 | 77.06% | Test helper |
| src/services/mocks/tests.rs | 590 | 3 | 99.49% | ✅ PASS |

**Analysis**: The core service interfaces (profile.rs, runtime.rs) are completely uncovered. These are trait definitions likely used in production code but not exercised in tests due to being real implementations rather than mocks.

### api.rs Module

| File | Lines | Missed | Coverage | Status |
|------|-------|--------|----------|--------|
| src/api.rs | 194 | 194 | 0.00% | ❌ UNCOVERED |

**Analysis**: The API module has zero coverage. This appears to be external API code that may require integration tests or mock setups.

### engine/ Module

| File | Lines | Missed | Coverage | Status |
|------|-------|--------|----------|--------|
| src/engine/event_loop.rs | 520 | 213 | 59.04% | ❌ BELOW |
| src/engine/decision/timing.rs | 39 | 36 | 7.69% | ❌ CRITICAL |
| src/engine/layer_actions.rs | 109 | 57 | 47.71% | ❌ BELOW |
| src/engine/state/key_state.rs | 81 | 40 | 50.62% | ❌ BELOW |
| src/engine/advanced/mod.rs | 183 | 72 | 60.66% | ❌ BELOW |
| src/engine/replay.rs | 819 | 140 | 82.91% | ⚠️ CLOSE |
| src/engine/recording/recorder.rs | 331 | 53 | 83.99% | ⚠️ CLOSE |
| Most other engine files | - | - | >90% | ✅ PASS |

**Analysis**: Most engine modules have excellent coverage. Key gaps:
- `decision/timing.rs` is critically low (7.69%)
- `event_loop.rs`, `layer_actions.rs`, `key_state.rs` need attention

### ffi/ Module

| File | Lines | Missed | Coverage | Status |
|------|-------|--------|----------|--------|
| src/ffi/domains/config.rs | 160 | 160 | 0.00% | ❌ UNCOVERED |
| src/ffi/domains/discovery.rs | 222 | 222 | 0.00% | ❌ UNCOVERED |
| src/ffi/domains/validation.rs | 14 | 14 | 0.00% | ❌ UNCOVERED |
| src/ffi/exports.rs | 166 | 100 | 39.76% | ❌ BELOW |
| src/ffi/exports_compat.rs | 452 | 452 | 0.00% | ❌ UNCOVERED |
| src/ffi/exports_runtime.rs | 307 | 307 | 0.00% | ❌ UNCOVERED |
| src/ffi/logging.rs | 37 | 37 | 0.00% | ❌ UNCOVERED |
| src/ffi/contract.rs | 340 | 126 | 62.94% | ❌ BELOW |
| src/ffi/domains/device.rs | 211 | 73 | 65.40% | ❌ BELOW |
| src/ffi/domains/device_registry.rs | 709 | 208 | 70.66% | ❌ BELOW |
| src/ffi/domains/migration.rs | 431 | 121 | 71.93% | ❌ BELOW |
| Most other FFI files | - | - | >85% | ⚠️/✅ |

**Analysis**: FFI layer has significant gaps, particularly:
- Several completely uncovered files (config, discovery, validation, exports_compat, exports_runtime, logging)
- FFI code often requires integration tests with actual FFI calls

## Other Notable Coverage Gaps

### validation/ Module

| File | Lines | Missed | Coverage | Status |
|------|-------|--------|----------|--------|
| src/validation/common/issue.rs | 125 | 125 | 0.00% | ❌ UNCOVERED |
| src/validation/common/visitor.rs | 23 | 23 | 0.00% | ❌ UNCOVERED |
| src/validation/conflicts.rs | 56 | 35 | 37.50% | ❌ BELOW |
| src/validation/engine/rhai_engine.rs | 347 | 156 | 55.04% | ❌ BELOW |
| Most other validation files | - | - | >95% | ✅ PASS |

### scripting/ Module

| File | Lines | Missed | Coverage | Status |
|------|-------|--------|----------|--------|
| src/scripting/api/executor.rs | 371 | 267 | 28.03% | ❌ CRITICAL |
| src/scripting/api/function_tests.rs | 377 | 230 | 38.99% | ❌ BELOW |
| src/scripting/api/functions/layers.rs | 227 | 144 | 36.56% | ❌ BELOW |
| src/scripting/api/functions/keys.rs | 157 | 96 | 38.85% | ❌ BELOW |
| src/scripting/api/functions/core.rs | 149 | 116 | 22.15% | ❌ CRITICAL |
| src/scripting/api/functions/keys_keycode.rs | 163 | 142 | 12.88% | ❌ CRITICAL |
| Many other scripting files | - | - | <70% | ❌ BELOW |

**Analysis**: The scripting module has significant coverage gaps across many files. This is a key area needing improvement.

### uat/ Module

| File | Lines | Missed | Coverage | Status |
|------|-------|--------|----------|--------|
| src/uat/golden.rs | 472 | 247 | 47.67% | ❌ BELOW |
| src/uat/perf.rs | 406 | 88 | 78.33% | ⚠️ CLOSE |
| Most other UAT files | - | - | >95% | ✅ PASS |

## Priority Recommendations

### High Priority (Critical Path <90%)

1. **src/api.rs** (0.00%) - Add integration tests for API endpoints
2. **src/services/profile.rs** (0.00%) - Add trait implementation tests
3. **src/services/runtime.rs** (0.00%) - Add trait implementation tests
4. **src/engine/decision/timing.rs** (7.69%) - Critical timing logic needs tests
5. **src/scripting/api/functions/keys_keycode.rs** (12.88%) - Core scripting needs tests
6. **src/scripting/api/functions/core.rs** (22.15%) - Core scripting needs tests
7. **src/scripting/api/executor.rs** (28.03%) - Script execution needs tests

### Medium Priority (50-80% coverage)

1. **src/engine/event_loop.rs** (59.04%) - Event loop logic
2. **src/engine/layer_actions.rs** (47.71%) - Layer action handling
3. **src/engine/state/key_state.rs** (50.62%) - Key state tracking
4. **src/ffi/domains/device_registry.rs** (70.66%) - Device registry FFI
5. **src/validation/engine/rhai_engine.rs** (55.04%) - Rhai validation engine
6. **src/uat/golden.rs** (47.67%) - Golden test utilities

### Low Priority (Uncovered but not critical)

1. FFI compatibility/export files (deprecated or seldom-used paths)
2. CLI tooling (generate_dart_bindings cli.rs)
3. Platform-specific code paths

## Test Infrastructure Notes

1. **Flaky Tests Found**: Two tests (`test_generate_markdown_integration`, `test_search_partial_match`) are flaky when run in parallel with coverage instrumentation. They pass with `--test-threads=1`.

2. **Mock Infrastructure**: The `src/services/mocks/` directory has good test coverage (99.49% for tests.rs), indicating the mock infrastructure is well-tested and can be used for improving service coverage.

3. **Coverage Target Assessment**:
   - Overall 80% target: ✅ Met (81.25%)
   - Critical path 90% target: ❌ Not met (many critical files below target)

## Conclusion

The codebase meets the overall 80% coverage target but falls short on critical path coverage. Key areas needing attention:
1. **Scripting API** - Multiple files under 40% coverage
2. **Services traits** - Core interfaces completely untested
3. **FFI exports** - Several completely uncovered files
4. **Engine timing/events** - Critical logic paths under-tested

Recommendation: Focus on high-priority items first, particularly the scripting API and engine timing modules, as these represent core functionality with critical coverage gaps.
