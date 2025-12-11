# Code Complexity Analysis Report

**Date**: 2025-12-12
**Spec**: misc-improvements
**Task**: 1.5 - Calculate code complexity (optional)

## Methodology

Cyclomatic complexity was estimated by counting decision points in each function:
- `if` / `else if` statements
- `match` arms (each non-wildcard arm adds 1)
- `for` / `while` / `loop` constructs
- `?` error propagation (implicit branch)
- `&&` / `||` boolean operators (short-circuit evaluation)

Formula: **Complexity = 1 + decision_points**

**Target**: Functions with complexity >10 are flagged for simplification.

## Executive Summary

| Metric | Count |
|--------|-------|
| Functions analyzed | 44 (from 1.1 audit: Critical + High severity) |
| Functions >15 complexity (Very High) | 6 |
| Functions 11-15 complexity (High) | 12 |
| Functions ≤10 complexity (Acceptable) | 26 |

## High Complexity Functions (>10)

### Very High Complexity (>15) - Priority 1

| Function | Location | Est. Complexity | Decision Points |
|----------|----------|-----------------|-----------------|
| `apply` | engine/state/mod.rs:640 | **26** | 12 match arms + 14 nested if/else |
| `validate_session_transition` | engine/transitions/graph.rs:231 | **19** | 9 match arms + 10 if/else conditions |
| `analyze` | validation/coverage.rs:21 | **17** | 7 match arms + 5 nested match arms + 5 iteration |
| `create_validation_engine` | validation/engine/rhai_engine.rs:20 | **16** | 15+ function registrations with closures containing conditionals |
| `evaluate` | metrics/alerts.rs:211 | **15** | 1 loop + 7 if/else-if pairs + 7 conditions |
| `process_event_traced` | engine/advanced/processing.rs:131 | **15** | 5 if/else + 4 let-else/? + 6 match conditionals |

### High Complexity (11-15) - Priority 2

| Function | Location | Est. Complexity | Decision Points |
|----------|----------|-----------------|-----------------|
| `run_command` | bin/keyrx/dispatch.rs:22 | **14** | 13 match arms + 1 nested match |
| `migrate` | migration/v1_to_v2.rs:40 | **13** | 5 if/else + 4 match + 4 loops/nested |
| `evaluate_with_context` | uat/gates_evaluation.rs:32 | **12** | 6 if conditions + 3 let-if + 3 iterations |
| `conflict_message` | validation/detectors/conflicts.rs:134 | **12** | Multiple match arms with conditions |
| `check_thresholds` | observability/metrics_bridge.rs:278 | **12** | 6 threshold comparisons with conditions |
| `output_human_results` | cli/commands/uat.rs:425 | **12** | Multiple conditional output formatting |
| `map` | cli/commands/keymap.rs:146 | **11** | Various conditional branches |
| `parse_layer_action` | scripting/builtins.rs:381 | **11** | Multiple match/if combinations |
| `combo_rc_impl` | scripting/bindings/row_col.rs:276 | **11** | Nested conditionals and loops |
| `load_from_content` | config/loader/parsing.rs:79 | **11** | Multiple validation branches |
| `handle_event` | discovery/session.rs:198 | **11** | Event type matching with conditions |
| `validate_discovery_transition` | engine/transitions/graph.rs:366 | **11** | Multiple transition type checks |

## Complexity Distribution

```
Complexity     Count     |  Bar Chart
-------------------------------------------
 1-5           10        |  ████████████████████
 6-10          16        |  ████████████████████████████████
11-15          12        |  ████████████████████████
16-20           4        |  ████████
21-30           2        |  ████
```

## Simplification Strategies

### 1. `apply` (complexity: 26)
**Location**: engine/state/mod.rs:640
**Problem**: Giant match statement handling 15+ mutation types
**Strategy**: Extract each match arm into a dedicated handler method:
```rust
// Before: 163-line apply() with massive match
// After: apply() dispatches to:
fn apply_key_down(&mut self, ...) -> StateResult<StateChange>
fn apply_key_up(&mut self, ...) -> StateResult<StateChange>
fn apply_push_layer(&mut self, ...) -> StateResult<StateChange>
// ... etc
```

### 2. `validate_session_transition` (complexity: 19)
**Location**: engine/transitions/graph.rs:231
**Problem**: 9 match arms with similar error construction patterns
**Strategy**:
- Create helper `validate_or_err()` to reduce error boilerplate
- Extract state validation predicates

### 3. `analyze` (complexity: 17)
**Location**: validation/coverage.rs:21
**Problem**: Nested match within loop
**Strategy**:
- Extract inner match to `process_pending_op(op, &mut coverage_state)`
- Use a dedicated struct for accumulating coverage data

### 4. `create_validation_engine` (complexity: 16)
**Location**: validation/engine/rhai_engine.rs:20
**Problem**: Repetitive function registrations (199 lines)
**Strategy**:
- Group related registrations into helper methods:
  - `register_key_operations(&mut engine, &ops)`
  - `register_layer_operations(&mut engine, &ops)`
  - `register_modifier_operations(&mut engine, &ops)`
  - `register_timing_operations(&mut engine, &ops)`

### 5. `evaluate` (complexity: 15)
**Location**: metrics/alerts.rs:211
**Problem**: Repetitive threshold checking with similar alert construction
**Strategy**:
- Create `check_threshold()` helper that returns optional Alert
- Reduce code duplication for warn/critical pairs

### 6. `process_event_traced` (complexity: 15)
**Location**: engine/advanced/processing.rs:131
**Problem**: Multiple processing steps with conditionals
**Strategy**: Already reasonably structured, could extract:
- `build_key_mutation()` helper
- `handle_device_passthrough()` helper

## Acceptable Complexity Functions

The following long functions have acceptable complexity (≤10) despite their length:

| Function | Lines | Complexity | Reason |
|----------|-------|------------|--------|
| `html_header` | 261 | 3 | Template with minimal branching |
| `html_scripts` | 112 | 2 | Static string generation |
| `render_ascii_keyboard` | 172 | 8 | Visual output with loops |
| `render_legend` | 93 | 5 | Formatting output |
| `panels` | 98 | 4 | Dashboard configuration |
| `init` | 97 | 9 | Logger setup with feature flags |

These functions are long due to **data/configuration**, not logic complexity. They could still benefit from extraction into smaller units for readability, but complexity reduction is not the primary concern.

## Recommendations

### Priority 1 (Immediate - Very High Complexity)
1. **`apply`** - Break into per-mutation-type handlers
2. **`validate_session_transition`** - Extract validation helpers
3. **`analyze`** - Extract pending op processing

### Priority 2 (Soon - High Complexity)
4. **`create_validation_engine`** - Group function registrations
5. **`evaluate`** - Create threshold checking helpers
6. **`run_command`** - Already dispatches to subcommands (acceptable)

### Priority 3 (Nice to Have)
- Functions with complexity 11-15 are borderline
- Focus on those that are also >50 lines
- Skip if time-constrained

## Correlation with Function Length

| Complexity | Avg Lines | Correlation |
|------------|-----------|-------------|
| >15 | 143 | Strong - complex = long |
| 11-15 | 82 | Moderate |
| ≤10 | 94 | Weak - long can be simple |

High complexity strongly correlates with long functions in the critical range (100+ lines), suggesting that refactoring for length will also reduce complexity.

## Conclusion

This analysis is **optional** per task definition. Key findings:

1. **6 functions** have very high complexity (>15) requiring simplification
2. **12 functions** have high complexity (11-15), borderline
3. Most complexity is in the engine core and validation modules
4. Refactoring for length (task 3.x) will likely reduce complexity as well
5. Template/output functions are long but not complex

The highest priority targets are `apply`, `validate_session_transition`, and `analyze` due to their combination of high complexity AND high line count.
