# Coverage Gaps Analysis

**Date**: 2025-12-11  
**Overall Coverage**: 81.28% (Target: 80%)  
**Critical Path Target**: 90%

## Summary

The codebase exceeds the 80% minimum coverage target. This document identifies areas for future improvement.

## Modules Below 80% Coverage

| Module | Line Coverage | Priority | Notes |
|--------|--------------|----------|-------|
| `src/bin/keyrx/` | 0% | Low | CLI entry point - difficult to unit test |
| `src/validation/common/issue.rs` | 0% | Medium | Utility module |
| `src/validation/common/visitor.rs` | 0% | Medium | Utility module |
| `src/validation/conflicts.rs` | 37.50% | Medium | Conflict detection logic |
| `src/uat/golden.rs` | 47.67% | Low | Test infrastructure |
| `src/validation/engine/rhai_engine.rs` | 55.04% | High | Script engine validation |
| `src/ffi/domains/scripting.rs` | 69.38% | High | FFI scripting interface |
| `src/ffi/domains/observability.rs` | 73.67% | Medium | FFI observability |
| `src/validation/engine/context.rs` | 75.74% | Medium | Validation context |
| `src/uat/perf.rs` | 78.33% | Low | Performance testing |

## Critical Paths Below 90%

Critical paths include FFI, services, API, and engine modules.

| Module | Coverage | Gap to 90% | Priority |
|--------|----------|------------|----------|
| `src/ffi/domains/scripting.rs` | 69.38% | -20.62% | High |
| `src/ffi/domains/observability.rs` | 73.67% | -16.33% | Medium |
| `src/engine/state.rs` | ~75% | -15% | High |
| `src/validation/engine/rhai_engine.rs` | 55.04% | -34.96% | High |
| `src/validation/engine/context.rs` | 75.74% | -14.26% | Medium |

## Well-Covered Critical Areas (>90%)

- `src/api.rs` - ~95%
- `src/engine/processor.rs` - High coverage
- `src/services/` - High coverage
- `src/ffi/contract/` - High coverage
- `src/validation/semantic.rs` - 100%
- `src/validation/coverage.rs` - 97.72%

## Recommendations

### High Priority
1. **Rhai Engine Validation** (`src/validation/engine/rhai_engine.rs`)
   - Currently at 55% - significant gap
   - Add tests for error paths and edge cases

2. **FFI Scripting** (`src/ffi/domains/scripting.rs`)
   - Critical FFI path at 69%
   - Add tests for script evaluation scenarios

### Medium Priority
3. **Validation Utilities** (`src/validation/common/`)
   - 0% coverage on issue.rs and visitor.rs
   - Add basic unit tests for utilities

4. **FFI Observability** (`src/ffi/domains/observability.rs`)
   - At 73%, near target
   - Focus on untested branches

### Low Priority
5. **CLI Entry Points** (`src/bin/keyrx/`)
   - CLI main functions are hard to unit test
   - Integration tests cover this path

6. **Test Infrastructure** (`src/uat/golden.rs`)
   - Test infrastructure has lower priority
   - Self-testing is less critical

## Action Items

These items should be tracked in a separate spec/task list:

- [ ] Add tests for `validation/engine/rhai_engine.rs` error paths
- [ ] Add tests for `ffi/domains/scripting.rs` edge cases
- [ ] Create unit tests for `validation/common/issue.rs`
- [ ] Create unit tests for `validation/common/visitor.rs`
- [ ] Add missing test cases for `validation/conflicts.rs`

## Conclusion

The codebase is in good health with 81.28% coverage exceeding the 80% target. The identified gaps are primarily in:
- Validation engine utilities
- Some FFI domain modules
- Test infrastructure code

None of these gaps block the current spec objectives. Coverage improvement should be tracked as a separate initiative.
