# Tasks Document: Code Quality Fixes

## Phase 1: Fix Failing Test

- [x] 1. Fix device_profiles_dir_prefers_xdg_config_home test
  - File: `src/discovery/types.rs`
  - Add `#[serial]` attribute from `serial_test` crate to isolate env var tests
  - Ensure proper cleanup of environment variables
  - Purpose: Fix the 1 failing unit test
  - _Leverage: `serial_test` crate already in Cargo.toml_
  - _Requirements: REQ-1_
  - _Prompt: Implement the task for spec code-quality-fixes, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Developer | Task: Fix the failing test `device_profiles_dir_prefers_xdg_config_home` in src/discovery/types.rs by adding `#[serial]` attribute to prevent race conditions with parallel tests that modify environment variables. Also add it to `device_profiles_dir_falls_back_to_home` test. Import `use serial_test::serial;` | Restrictions: Do not change test logic, only add isolation | Success: `cargo test device_profiles_dir` passes. Mark task [-] in_progress before starting, use log-implementation after completion, then mark [x] complete._

## Phase 2: FFI Module Refactoring

- [x] 2. Create exports_script.rs module
  - File: `src/ffi/exports_script.rs`
  - Move: `keyrx_load_script`, `keyrx_check_script`, `keyrx_eval` from exports_session.rs
  - Move: `ValidationError`, `ValidationResult` structs
  - Purpose: Isolate script loading/validation FFI functions
  - _Leverage: Existing pattern in exports.rs, exports_device.rs_
  - _Requirements: REQ-2_
  - _Prompt: Implement the task for spec code-quality-fixes, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust FFI Developer | Task: Create src/ffi/exports_script.rs by extracting keyrx_load_script (lines 26-77), keyrx_check_script (lines 103-164), keyrx_eval (lines 533-557), and ValidationError/ValidationResult structs from exports_session.rs. Add proper module documentation and imports. | Restrictions: Keep function signatures identical, add `#![allow(unsafe_code)]` | Success: Functions compile and are accessible. Mark task [-] before starting, log-implementation after, mark [x] when complete._

- [x] 3. Create exports_testing.rs module
  - File: `src/ffi/exports_testing.rs`
  - Move: `keyrx_discover_tests`, `keyrx_run_tests`, `keyrx_simulate` from exports_session.rs
  - Move: Related structs (DiscoveredTestJson, TestResultJson, etc.)
  - Purpose: Isolate testing/simulation FFI functions
  - _Leverage: Existing FFI patterns_
  - _Requirements: REQ-2_
  - _Prompt: Implement the task for spec code-quality-fixes, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust FFI Developer | Task: Create src/ffi/exports_testing.rs by extracting keyrx_discover_tests (lines 183-259), keyrx_run_tests (lines 261-404), keyrx_simulate (lines 406-531) and related structs from exports_session.rs | Restrictions: Maintain all function signatures | Success: Test-related FFI functions compile. Mark [-] before, log after, mark [x] complete._

- [x] 4. Create exports_discovery.rs module
  - File: `src/ffi/exports_discovery.rs`
  - Move: `keyrx_on_discovery_*`, `keyrx_start_discovery`, `keyrx_process_discovery_event`, `keyrx_cancel_discovery`, `keyrx_get_discovery_progress`
  - Move: Discovery session state management, callback setup
  - Purpose: Isolate device discovery FFI functions
  - _Leverage: Existing callback patterns_
  - _Requirements: REQ-2_
  - _Prompt: Implement the task for spec code-quality-fixes, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust FFI Developer | Task: Create src/ffi/exports_discovery.rs with all discovery-related functions: keyrx_on_discovery_progress/duplicate/summary (lines 559-578), refresh_discovery_sink, discovery_sink, discovery_session_slot, keyrx_start_discovery (lines 1036-1207), keyrx_process_discovery_event, keyrx_cancel_discovery, keyrx_get_discovery_progress, and DiscoverySessionState struct | Restrictions: Keep OnceLock patterns | Success: Discovery FFI works. Mark [-] before, log after, mark [x] complete._

- [x] 5. Create exports_recording.rs module
  - File: `src/ffi/exports_recording.rs`
  - Move: `keyrx_start_recording`, `keyrx_stop_recording`, `keyrx_is_recording`, `keyrx_get_recording_path`
  - Move: RecordingState, recording_state_slot, set_active_recorder, with_active_recorder
  - Purpose: Isolate session recording FFI functions
  - _Requirements: REQ-2_
  - _Prompt: Implement the task for spec code-quality-fixes, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust FFI Developer | Task: Create src/ffi/exports_recording.rs with recording functions: RecordingState struct, RECORDING_STATE OnceLock, recording_state_slot, keyrx_start_recording (lines 1551-1657), keyrx_stop_recording, keyrx_is_recording, keyrx_get_recording_path, set_active_recorder, with_active_recorder, get_recording_request, clear_recording_state | Restrictions: Maintain state management patterns | Success: Recording FFI works. Mark [-] before, log after, mark [x] complete._

- [x] 6. Create exports_analysis.rs module
  - File: `src/ffi/exports_analysis.rs`
  - Move: `keyrx_list_sessions`, `keyrx_analyze_session`, `keyrx_replay_session`
  - Move: Related JSON output structs
  - Purpose: Isolate session analysis FFI functions
  - _Requirements: REQ-2_
  - _Prompt: Implement the task for spec code-quality-fixes, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust FFI Developer | Task: Create src/ffi/exports_analysis.rs with analysis functions: keyrx_list_sessions (lines 629-730), keyrx_analyze_session (lines 732-829), keyrx_replay_session (lines 831-929), and their related JSON structs (SessionListJson, AnalysisResultJson, ReplayResultJson) | Restrictions: Keep JSON serialization patterns | Success: Analysis FFI works. Mark [-] before, log after, mark [x] complete._

- [x] 7. Create exports_diagnostics.rs module
  - File: `src/ffi/exports_diagnostics.rs`
  - Move: `keyrx_run_benchmark`, `keyrx_run_doctor`
  - Move: run_linux_diagnostics, run_windows_diagnostics helpers
  - Purpose: Isolate diagnostic FFI functions
  - _Requirements: REQ-2_
  - _Prompt: Implement the task for spec code-quality-fixes, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust FFI Developer | Task: Create src/ffi/exports_diagnostics.rs with keyrx_run_benchmark (lines 931-993), keyrx_run_doctor (lines 1333-1399), run_linux_diagnostics (lines 1401-1466), run_windows_diagnostics (lines 1468-1497) | Restrictions: Keep platform-specific #[cfg] attributes | Success: Diagnostic FFI works. Mark [-] before, log after, mark [x] complete._

- [x] 8. Update ffi/mod.rs with re-exports
  - File: `src/ffi/mod.rs`
  - Add module declarations for new files
  - Add re-exports to maintain public API
  - Remove exports_session.rs or keep minimal
  - Purpose: Maintain backward compatibility
  - _Requirements: REQ-2_
  - _Prompt: Implement the task for spec code-quality-fixes, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Developer | Task: Update src/ffi/mod.rs to declare new modules (exports_script, exports_testing, exports_discovery, exports_recording, exports_analysis, exports_diagnostics) and re-export all public FFI functions. Delete or minimize exports_session.rs. | Restrictions: All existing FFI function names must remain accessible | Success: `cargo build` succeeds, FFI tests pass. Mark [-] before, log after, mark [x] complete._

## Phase 3: UAT Report Module Refactoring

- [x] 9. Create uat/report_data.rs module
  - File: `src/uat/report_data.rs`
  - Move: `ReportData`, `CategoryStats` structs and their impl blocks
  - Purpose: Isolate report data structures
  - _Requirements: REQ-3_
  - _Prompt: Implement the task for spec code-quality-fixes, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Developer | Task: Create src/uat/report_data.rs by extracting ReportData struct (line 17), CategoryStats struct (line 126), and all their impl blocks from report.rs. Add proper imports and module documentation. | Restrictions: Keep all method signatures | Success: Structs compile. Mark [-] before, log after, mark [x] complete._

- [ ] 10. Create uat/report_markdown.rs module
  - File: `src/uat/report_markdown.rs`
  - Move: Markdown generation methods from ReportGenerator
  - Purpose: Isolate markdown report generation
  - _Requirements: REQ-3_
  - _Prompt: Implement the task for spec code-quality-fixes, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Developer | Task: Create src/uat/report_markdown.rs by extracting markdown generation methods (generate_markdown, generate_markdown_summary, etc.) from ReportGenerator impl in report.rs. Create a MarkdownReportGenerator struct or use free functions. | Restrictions: Maintain output format | Success: Markdown generation works. Mark [-] before, log after, mark [x] complete._

- [ ] 11. Create uat/report_html.rs module
  - File: `src/uat/report_html.rs`
  - Move: HTML generation methods from ReportGenerator
  - Move: escape_html helper function
  - Purpose: Isolate HTML report generation
  - _Requirements: REQ-3_
  - _Prompt: Implement the task for spec code-quality-fixes, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Developer | Task: Create src/uat/report_html.rs by extracting HTML generation methods and escape_html helper from report.rs. | Restrictions: Maintain HTML output format | Success: HTML generation works. Mark [-] before, log after, mark [x] complete._

- [ ] 12. Update uat/report.rs and mod.rs
  - Files: `src/uat/report.rs`, `src/uat/mod.rs`
  - Keep ReportGenerator as coordinator
  - Add re-exports for backward compatibility
  - Purpose: Complete report module refactoring
  - _Requirements: REQ-3_
  - _Prompt: Implement the task for spec code-quality-fixes, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Developer | Task: Update src/uat/report.rs to import from new modules and keep ReportGenerator as facade. Update src/uat/mod.rs with new module declarations and re-exports. Verify report.rs is under 500 lines. | Restrictions: Maintain public API | Success: UAT report tests pass, file under 500 lines. Mark [-] before, log after, mark [x] complete._

## Phase 4: UAT Golden Module Refactoring

- [ ] 13. Create uat/golden_types.rs module
  - File: `src/uat/golden_types.rs`
  - Move: GoldenSession, GoldenEvent, GoldenVerifyResult, GoldenDifference, ExpectedOutput structs
  - Move: GoldenSessionError, RecordResult, UpdateResult
  - Purpose: Isolate golden session type definitions
  - _Requirements: REQ-3_
  - _Prompt: Implement the task for spec code-quality-fixes, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Developer | Task: Create src/uat/golden_types.rs with all golden-related type definitions from golden.rs: GoldenSession, GoldenSessionMetadata, GoldenEvent, ExpectedOutput, GoldenVerifyResult, GoldenDifference, DifferenceType, GoldenSessionError, RecordResult, UpdateResult | Restrictions: Keep serde derives | Success: Types compile. Mark [-] before, log after, mark [x] complete._

- [ ] 14. Create uat/golden_comparison.rs module
  - File: `src/uat/golden_comparison.rs`
  - Move: compare_outputs, outputs_match, remove_timestamps, find_number_end helpers
  - Purpose: Isolate comparison logic
  - _Requirements: REQ-3_
  - _Prompt: Implement the task for spec code-quality-fixes, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Developer | Task: Create src/uat/golden_comparison.rs with comparison functions: compare_outputs (line 604), outputs_match (line 665), remove_timestamps (line 684), find_number_end (line 706) | Restrictions: Keep function signatures | Success: Comparison logic works. Mark [-] before, log after, mark [x] complete._

- [ ] 15. Update uat/golden.rs and mod.rs
  - Files: `src/uat/golden.rs`, `src/uat/mod.rs`
  - Keep GoldenSessionManager in golden.rs
  - Add re-exports
  - Purpose: Complete golden module refactoring
  - _Requirements: REQ-3_
  - _Prompt: Implement the task for spec code-quality-fixes, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Developer | Task: Update src/uat/golden.rs to import from golden_types.rs and golden_comparison.rs, keeping GoldenSessionManager implementation. Update mod.rs with re-exports. Verify golden.rs is under 500 lines. | Restrictions: Maintain public API | Success: Golden tests pass, file under 500 lines. Mark [-] before, log after, mark [x] complete._

## Phase 5: UAT Performance Module Refactoring

- [ ] 16. Create uat/perf_types.rs module
  - File: `src/uat/perf_types.rs`
  - Move: Performance-related structs and enums
  - Purpose: Isolate performance type definitions
  - _Requirements: REQ-3_
  - _Prompt: Implement the task for spec code-quality-fixes, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Developer | Task: Create src/uat/perf_types.rs with performance data structures from perf.rs (PerformanceResult, PerformanceStats, RegressionCheck, etc.) | Restrictions: Keep serde derives | Success: Types compile. Mark [-] before, log after, mark [x] complete._

- [ ] 17. Create uat/perf_analysis.rs module
  - File: `src/uat/perf_analysis.rs`
  - Move: Statistical analysis and regression detection functions
  - Purpose: Isolate analysis logic
  - _Requirements: REQ-3_
  - _Prompt: Implement the task for spec code-quality-fixes, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Developer | Task: Create src/uat/perf_analysis.rs with statistical analysis functions from perf.rs (calculate_stats, detect_regression, compare_baselines, etc.) | Restrictions: Keep calculation accuracy | Success: Analysis functions work. Mark [-] before, log after, mark [x] complete._

- [ ] 18. Update uat/perf.rs and mod.rs
  - Files: `src/uat/perf.rs`, `src/uat/mod.rs`
  - Keep test runner coordination in perf.rs
  - Add re-exports
  - Purpose: Complete perf module refactoring
  - _Requirements: REQ-3_
  - _Prompt: Implement the task for spec code-quality-fixes, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Developer | Task: Update src/uat/perf.rs to import from perf_types.rs and perf_analysis.rs. Update mod.rs with re-exports. Verify perf.rs is under 500 lines. | Restrictions: Maintain public API | Success: Perf tests pass, file under 500 lines. Mark [-] before, log after, mark [x] complete._

## Phase 6: UAT Runner/Gates Refactoring

- [ ] 19. Refactor uat/runner.rs
  - File: `src/uat/runner.rs`
  - Split into: runner_discovery.rs (test discovery), runner_execution.rs (test execution)
  - Keep runner.rs as coordinator under 500 lines
  - Purpose: Split runner module
  - _Requirements: REQ-3_
  - _Prompt: Implement the task for spec code-quality-fixes, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Developer | Task: Split src/uat/runner.rs (1079 lines) into runner_discovery.rs (test file discovery, metadata parsing) and runner_execution.rs (test execution logic). Keep UatRunner struct and coordination in runner.rs. Update mod.rs. | Restrictions: Maintain UatRunner public API | Success: Runner tests pass, all files under 500 lines. Mark [-] before, log after, mark [x] complete._

- [ ] 20. Refactor uat/gates.rs
  - File: `src/uat/gates.rs`
  - Split into: gates_definitions.rs (gate types), gates_evaluation.rs (gate checking)
  - Keep gates.rs as coordinator under 500 lines
  - Purpose: Split gates module
  - _Requirements: REQ-3_
  - _Prompt: Implement the task for spec code-quality-fixes, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Developer | Task: Split src/uat/gates.rs (1011 lines) into gates_definitions.rs (Gate struct, GateResult, GateType enums) and gates_evaluation.rs (check_gate, evaluate functions). Keep coordination in gates.rs. Update mod.rs. | Restrictions: Maintain Gate public API | Success: Gates tests pass, all files under 500 lines. Mark [-] before, log after, mark [x] complete._

## Phase 7: Engine/CLI Refactoring

- [ ] 21. Refactor engine/tracing.rs
  - File: `src/engine/tracing.rs`
  - Split into: tracing_types.rs (span types), tracing_formatters.rs (output formatters)
  - Keep tracing.rs as coordinator under 500 lines
  - Purpose: Split tracing module
  - _Requirements: REQ-4_
  - _Prompt: Implement the task for spec code-quality-fixes, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Developer | Task: Split src/engine/tracing.rs (621 lines) into tracing_types.rs (TraceSpan, TraceEvent structs) and tracing_formatters.rs (formatting/display logic). Keep TracingEngine in tracing.rs. Update engine/mod.rs. | Restrictions: Maintain tracing API | Success: Engine tests pass, all files under 500 lines. Mark [-] before, log after, mark [x] complete._

- [ ] 22. Refactor cli/commands/ci_check.rs
  - File: `src/cli/commands/ci_check.rs`
  - Split into: ci_check_phases.rs (phase runners), ci_check_summary.rs (summary generation)
  - Keep ci_check.rs as main command under 500 lines
  - Purpose: Split CI check command
  - _Requirements: REQ-4_
  - _Prompt: Implement the task for spec code-quality-fixes, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Developer | Task: Split src/cli/commands/ci_check.rs (789 lines) into ci_check_phases.rs (individual phase check functions) and ci_check_summary.rs (CiCheckSummary, CheckPhaseResult, output generation). Keep CiCheckCommand in ci_check.rs. Update commands/mod.rs. | Restrictions: Maintain CLI interface | Success: CI check command works, all files under 500 lines. Mark [-] before, log after, mark [x] complete._

## Phase 8: Verification

- [ ] 23. Run full test suite
  - Run: `cargo test --lib` - verify 0 failures
  - Run: `cargo test --test '*'` - verify integration tests pass
  - Purpose: Ensure all tests pass after refactoring
  - _Requirements: REQ-5_
  - _Prompt: Implement the task for spec code-quality-fixes, first run spec-workflow-guide to get the workflow guide then implement the task: Role: QA Engineer | Task: Run full test suite: `cargo test --lib` and `cargo test --test '*'`. Fix any failures introduced by refactoring. | Restrictions: Do not skip tests | Success: All tests pass. Mark [-] before, log after, mark [x] complete._

- [ ] 24. Verify file sizes and coverage
  - Run: `find src -name "*.rs" -exec wc -l {} \; | awk '$1 > 500'` - verify no violations
  - Run: `cargo llvm-cov --lib --summary-only` - verify >= 80% coverage
  - Purpose: Final verification of code quality metrics
  - _Requirements: REQ-5_
  - _Prompt: Implement the task for spec code-quality-fixes, first run spec-workflow-guide to get the workflow guide then implement the task: Role: QA Engineer | Task: Verify all files are under 500 lines and test coverage is >= 80%. Run file size check and coverage report. Document results. | Restrictions: All metrics must pass | Success: No files over 500 lines, coverage >= 80%. Mark [-] before, log after, mark [x] complete._
