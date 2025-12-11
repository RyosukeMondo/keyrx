# Function Length Audit Report

**Date**: 2025-12-12
**Spec**: misc-improvements
**Task**: 1.1 - Audit function lengths across codebase

## Executive Summary

| Metric | Value |
|--------|-------|
| Total functions analyzed | 6,222 |
| Non-test functions | 3,490 |
| Functions > 50 lines | 90 |
| Compliance rate | 97.4% |

## Summary by Severity

| Severity | Line Count | Count |
|----------|------------|-------|
| **Critical** | 100+ lines | 14 |
| **High** | 70-99 lines | 30 |
| **Medium** | 51-69 lines | 46 |

## Top 20 Violations (Ranked by Size)

| Rank | Lines | File | Function | Line |
|------|-------|------|----------|------|
| 1 | 261 | scripting/docs/generators/html/templates.rs | `html_header` | 7 |
| 2 | 199 | validation/engine/rhai_engine.rs | `create_validation_engine` | 20 |
| 3 | 172 | validation/coverage.rs | `render_ascii_keyboard` | 144 |
| 4 | 163 | engine/state/mod.rs | `apply` | 640 |
| 5 | 143 | bin/keyrx/dispatch.rs | `run_command` | 22 |
| 6 | 131 | engine/transitions/graph.rs | `validate_session_transition` | 231 |
| 7 | 124 | scripting/sandbox/function_capabilities.rs | `build_function_registry` | 69 |
| 8 | 114 | cli/commands/check.rs | `print_human_result` | 170 |
| 9 | 112 | migration/v1_to_v2.rs | `migrate` | 40 |
| 10 | 112 | scripting/docs/generators/html/templates.rs | `html_scripts` | 316 |
| 11 | 108 | cli/commands/hardware.rs | `calibrate` | 398 |
| 12 | 101 | drivers/windows/device.rs | `list_keyboards` | 28 |
| 13 | 100 | metrics/alerts.rs | `evaluate` | 211 |
| 14 | 100 | engine/replay.rs | `from_streaming_file` | 297 |
| 15 | 98 | metrics/grafana.rs | `panels` | 48 |
| 16 | 97 | observability/logger.rs | `init` | 149 |
| 17 | 93 | profiling/flamegraph_diff.rs | `render_legend` | 266 |
| 18 | 90 | observability/metrics_bridge.rs | `check_thresholds` | 278 |
| 19 | 90 | cli/commands/uat.rs | `output_human_results` | 425 |
| 20 | 90 | cli/commands/keymap.rs | `map` | 146 |

## All Violations by Severity

### Critical (100+ lines) - 14 functions

| Lines | Function | Location |
|-------|----------|----------|
| 261 | `html_header` | scripting/docs/generators/html/templates.rs:7 |
| 199 | `create_validation_engine` | validation/engine/rhai_engine.rs:20 |
| 172 | `render_ascii_keyboard` | validation/coverage.rs:144 |
| 163 | `apply` | engine/state/mod.rs:640 |
| 143 | `run_command` | bin/keyrx/dispatch.rs:22 |
| 131 | `validate_session_transition` | engine/transitions/graph.rs:231 |
| 124 | `build_function_registry` | scripting/sandbox/function_capabilities.rs:69 |
| 114 | `print_human_result` | cli/commands/check.rs:170 |
| 112 | `migrate` | migration/v1_to_v2.rs:40 |
| 112 | `html_scripts` | scripting/docs/generators/html/templates.rs:316 |
| 108 | `calibrate` | cli/commands/hardware.rs:398 |
| 101 | `list_keyboards` | drivers/windows/device.rs:28 |
| 100 | `evaluate` | metrics/alerts.rs:211 |
| 100 | `from_streaming_file` | engine/replay.rs:297 |

### High (70-99 lines) - 30 functions

| Lines | Function | Location |
|-------|----------|----------|
| 98 | `panels` | metrics/grafana.rs:48 |
| 97 | `init` | observability/logger.rs:149 |
| 93 | `render_legend` | profiling/flamegraph_diff.rs:266 |
| 90 | `check_thresholds` | observability/metrics_bridge.rs:278 |
| 90 | `output_human_results` | cli/commands/uat.rs:425 |
| 90 | `map` | cli/commands/keymap.rs:146 |
| 88 | `process_event_traced` | engine/advanced/processing.rs:131 |
| 88 | `conflict_message` | validation/detectors/conflicts.rs:134 |
| 87 | `analyze` | validation/coverage.rs:21 |
| 86 | `evaluate_with_context` | uat/gates_evaluation.rs:32 |
| 86 | `add_slot` | cli/commands/runtime.rs:225 |
| 85 | `run` | uat/perf.rs:68 |
| 85 | `build_metrics_exporter` | observability/otel/metrics.rs:115 |
| 83 | `run_perf_test` | uat/perf_runner.rs:27 |
| 80 | `spawn_thread` | drivers/windows/raw_input.rs:55 |
| 79 | `html_performance_section` | uat/report_html_sections.rs:87 |
| 78 | `parse_layer_action` | scripting/builtins.rs:381 |
| 78 | `output_human_results` | cli/commands/regression.rs:245 |
| 77 | `combo_rc_impl` | scripting/bindings/row_col.rs:276 |
| 76 | `load_from_content` | config/loader/parsing.rs:79 |
| 76 | `run_internal` | cli/commands/uat.rs:219 |
| 75 | `handle_event` | discovery/session.rs:198 |
| 74 | `search_type` | scripting/docs/search.rs:197 |
| 72 | `message_loop` | discovery/watcher_windows.rs:93 |
| 72 | `run` | cli/commands/regression.rs:156 |
| 71 | `set_slot_active` | cli/commands/runtime.rs:399 |
| 71 | `wire` | cli/commands/hardware.rs:196 |
| 70 | `load_script` | ffi/domains/script.rs:63 |
| 70 | `process_raw_input` | drivers/windows/raw_input.rs:236 |
| 70 | `search_function` | scripting/docs/search.rs:111 |

### Medium (51-69 lines) - 46 functions

| Lines | Function | Location |
|-------|----------|----------|
| 69 | `new` | metrics/collector.rs:169 |
| 69 | `validate_system_transition` | engine/transitions/graph.rs:135 |
| 68 | `find_operation_line` | validation/engine/context.rs:119 |
| 68 | `generate_function_html` | scripting/docs/generators/html/rendering.rs:143 |
| 67 | `html_coverage_section` | uat/report_html_sections.rs:11 |
| 67 | `run_loop` | engine/event_loop.rs:259 |
| 67 | `render_frame` | profiling/flamegraph_diff.rs:368 |
| 67 | `decode` | engine/recording/format.rs:159 |
| 67 | `remove_slot` | cli/commands/runtime.rs:322 |
| 66 | `run_regression_tests` | cli/commands/ci_check/ci_check_phases.rs:136 |
| 64 | `render_latency_metrics` | metrics/prometheus.rs:55 |
| 64 | `run_internal` | uat/runner.rs:115 |
| 64 | `record_result` | cli/commands/simulate.rs:292 |
| 62 | `new` | drivers/linux/reader.rs:110 |
| 62 | `new_with_injector_and_metrics` | drivers/linux/mod.rs:96 |
| 61 | `send_output` | drivers/resilient.rs:225 |
| 61 | `simulate` | ffi/domains/testing.rs:201 |
| 61 | `next_state` | engine/transitions/graph.rs:423 |
| 60 | `check_timing_bounds` | validation/semantic.rs:81 |
| 60 | `load` | registry/bindings.rs:151 |
| 60 | `list_devices` | cli/commands/runtime.rs:159 |
| 60 | `profile` | cli/commands/hardware.rs:332 |
| 57 | `stop_recording` | ffi/domains/recording.rs:192 |
| 57 | `check_event` | engine/decision/pending.rs:173 |
| 57 | `from_code` | cli/commands/exit_codes.rs:103 |
| 56 | `run_linux_diagnostics` | ffi/domains/diagnostics.rs:176 |
| 56 | `retry_with_backoff` | drivers/common/recovery.rs:128 |
| 55 | `log_profile_validation_error` | discovery/storage.rs:126 |
| 55 | `check_emergency_combo_keys` | validation/safety.rs:100 |
| 55 | `retry_with_backoff_sync` | drivers/common/recovery.rs:225 |
| 55 | `generate_type_html` | scripting/docs/generators/html/rendering.rs:49 |
| 55 | `log_config_validation_error` | config/loader/parsing.rs:188 |
| 55 | `run` | cli/commands/repl.rs:28 |
| 55 | `process_command` | cli/commands/repl.rs:96 |
| 54 | `record` | uat/golden.rs:118 |
| 54 | `compare_baseline_with_threshold` | uat/perf.rs:196 |
| 54 | `render_frame` | profiling/flamegraph.rs:228 |
| 54 | `run` | cli/commands/runtime.rs:104 |
| 53 | `from_driver_error` | errors/critical.rs:328 |
| 53 | `normalize_array` | cli/output/table.rs:49 |
| 52 | `builtin_profiles` | hardware/database.rs:93 |
| 52 | `from_file` | engine/replay.rs:131 |
| 52 | `run_single_test` | scripting/test_runner.rs:103 |
| 52 | `validate_discovery_transition` | engine/transitions/graph.rs:366 |
| 51 | `execute_hardware` | bin/keyrx/commands_config.rs:101 |
| 51 | `run_verify` | cli/commands/golden.rs:139 |

## Refactoring Recommendations

### Priority 1: Critical Functions (100+ lines)

These functions are over twice the limit and should be refactored first:

1. **`html_header` (261 lines)** - Template function, split into section generators
2. **`create_validation_engine` (199 lines)** - Split by registration phase
3. **`render_ascii_keyboard` (172 lines)** - Split by keyboard section
4. **`apply` (163 lines)** - Split by action type handling
5. **`run_command` (143 lines)** - Split by command group

### Priority 2: High Functions (70-99 lines)

These are moderately above the limit and should be addressed after critical:

- Focus on CLI command handlers (multiple functions in cli/commands/)
- Metrics/observability functions
- Engine processing functions

### Priority 3: Medium Functions (51-69 lines)

These are slightly above the limit:

- May be acceptable with minor extraction
- Lower priority, address if time permits

## Notes

- Test functions (#[test], #[cfg(test)]) were excluded from this analysis
- Line counts are logical lines (excluding blank lines and comments)
- All paths are relative to `core/src/`
