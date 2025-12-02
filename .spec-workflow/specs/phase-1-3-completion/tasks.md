# Tasks Document

## Phase 1: Iron Core Completion

### Refactoring (Code Quality Compliance)

- [x] 1. Refactor scripting/runtime.rs into modular structure
  - Files: core/src/scripting/runtime.rs → core/src/scripting/runtime.rs, core/src/scripting/bindings.rs, core/src/scripting/builtins.rs
  - Split 1654-line file into 3 modules: runtime.rs (core setup, ~400 lines), bindings.rs (function registration, ~500 lines), builtins.rs (standard functions, ~500 lines)
  - Maintain existing API; internal refactor only
  - _Leverage: existing module structure in core/src/scripting/_
  - _Requirements: NFR Code Architecture_
  - _Prompt: Implement the task for spec phase-1-3-completion, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust systems engineer specializing in module organization | Task: Split scripting/runtime.rs (1654 lines) into 3 modules: runtime.rs (Engine/AST setup, ~400 lines), bindings.rs (register_* functions for Rhai, ~500 lines), builtins.rs (layer/modifier/timing helpers, ~500 lines) | Restrictions: No API changes; all existing tests must pass; maintain re-exports in runtime.rs mod.rs; each file ≤500 lines | _Leverage: existing module patterns in cli/commands/ | _Requirements: NFR Code Architecture | Success: All 3 files ≤500 lines, cargo test passes, no breaking changes to public API, clippy clean._

- [x] 2. Refactor editor.dart into modular structure
  - Files: ui/lib/pages/editor.dart → ui/lib/pages/editor_page.dart, ui/lib/pages/editor_widgets.dart
  - Split 844-line file into 2 files: editor_page.dart (state/layout, ~400 lines), editor_widgets.dart (reusable widgets, ~400 lines)
  - _Leverage: existing widget patterns in ui/lib/pages/_
  - _Requirements: NFR Code Architecture_
  - _Prompt: Implement the task for spec phase-1-3-completion, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Flutter/Dart engineer specializing in widget architecture | Task: Split editor.dart (844 lines) into editor_page.dart (EditorPage stateful widget, state management, ~400 lines) and editor_widgets.dart (KeyMappingRow, LayerSelector, KeyPicker widgets, ~400 lines) | Restrictions: No functionality changes; all existing imports must resolve; each file ≤500 lines | _Leverage: existing widget patterns in debugger.dart, console.dart | _Requirements: NFR Code Architecture | Success: Both files ≤500 lines, flutter test passes, no UI regressions._

- [x] 2.5. Replace panic!() calls with proper assertions in test code
  - Files: core/src/discovery/session.rs, core/src/engine/decision/pending.rs, core/src/engine/advanced.rs, core/src/cli/commands/discover.rs, core/src/mocks/mock_state.rs, core/tests/integration/channel_tests.rs
  - Replace 10 identified panic!() calls with assert!, assert_eq!, or .expect() with descriptive messages
  - _Leverage: existing test patterns in core/src/_
  - _Requirements: NFR Code Architecture (CLAUDE.md panic cleanup)_
  - _Prompt: Implement the task for spec phase-1-3-completion, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust code quality engineer | Task: Replace all panic!() calls in test code with proper assertions: use assert_eq!() for value comparisons, assert!() for boolean conditions, .expect("descriptive message") for Option/Result unwrapping; files to modify: discovery/session.rs (3 panics), engine/decision/pending.rs (2 panics), engine/advanced.rs (1 panic), cli/commands/discover.rs (1 panic), mocks/mock_state.rs (1 panic), tests/integration/channel_tests.rs (2 panics) | Restrictions: No behavior changes; preserve test intent; add descriptive messages | _Leverage: existing assertion patterns in codebase | _Requirements: NFR Code Architecture | Success: `cargo test` passes, no panic!() in test code, clippy clean._

### Script Testing Framework

- [x] 3. Implement test harness with Rhai test primitives
  - Files: core/src/scripting/test_harness.rs (new)
  - Create TestHarness struct; implement register_test_functions() to add simulate_tap, simulate_hold, assert_output, assert_mapping to Rhai runtime
  - _Leverage: core/src/scripting/runtime.rs (RhaiRuntime), core/src/engine/types.rs (InputEvent)_
  - _Requirements: 1.1_
  - _Prompt: Implement the task for spec phase-1-3-completion, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust/Rhai integration engineer | Task: Create test_harness.rs with TestHarness struct; implement register_test_functions(runtime: &mut RhaiRuntime) adding simulate_tap(key), simulate_hold(key, ms), assert_output(key), assert_mapping(from, to) functions; store test outputs in thread-local Vec for assertions | Restrictions: ≤500 lines; no panics across FFI; use existing event injection patterns; thread-safe | _Leverage: RhaiRuntime function registration, InputEvent::key_down/key_up | _Requirements: 1.1 | Success: Rhai scripts can call simulate_tap("KeyA") and assert_output("KeyB"), assertions report pass/fail with message._

- [x] 4. Implement test discovery and runner
  - Files: core/src/scripting/test_runner.rs (new)
  - Parse Rhai AST for functions with `test_` prefix (Rhai doesn't support attributes); execute each in isolated context; collect TestResult structs
  - _Leverage: core/src/scripting/runtime.rs, core/src/scripting/test_harness.rs_
  - _Requirements: 1.1_
  - _Prompt: Implement the task for spec phase-1-3-completion, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust test framework engineer | Task: Create test_runner.rs with discover_tests(ast: &rhai::AST) -> Vec<String> to find functions with `test_` prefix (NOT #[test] - Rhai doesn't support attributes); run_tests(runtime, tests) -> Vec<TestResult>; TestResult { name, passed, message, duration_µs, line_number } | Restrictions: ≤400 lines; iterate AST functions checking name.starts_with("test_"); catch panics per-test; report line numbers on failure | _Leverage: rhai::AST::iter_functions() for discovery, test_harness for primitives | _Requirements: 1.1 | Success: Discovers `fn test_*` functions, runs each independently, returns structured results._

- [x] 5. Implement keyrx test CLI command
  - Files: core/src/cli/commands/test.rs (new), core/src/bin/keyrx.rs (modify to add command)
  - Add `keyrx test --script <path>` command with --filter, --watch, --json flags; load script, discover tests, run, report results with exit code per product.md (0=pass, 1=error, 2=assertion fail, 3=timeout)
  - _Leverage: core/src/cli/commands/check.rs (script loading pattern), core/src/scripting/test_runner.rs, notify crate for watch_
  - _Requirements: 1.1_
  - _Prompt: Implement the task for spec phase-1-3-completion, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust CLI engineer | Task: Create test.rs with TestCommand struct; implement execute() to load script, call discover_tests(), run_tests(), format results as table (name, status, duration); support --filter "pattern*" to run subset, --watch to re-run on file change (use notify crate), --json for machine output; exit codes: 0=all pass, 1=execution error, 2=assertion fail, 3=timeout | Restrictions: ≤400 lines; add to Commands enum in bin/keyrx.rs; add notify = "6.0" to Cargo.toml for watch mode | _Leverage: check.rs for script path validation, cli/output.rs for formatting, notify crate | _Requirements: 1.1 | Success: `keyrx test --script tests/example.rhai --filter "capslock*" --watch` runs filtered tests, re-runs on change._

### REPL Console

- [x] 6. Implement keyrx repl CLI command
  - Files: core/src/cli/commands/repl.rs (new/replace stub), core/Cargo.toml (add rustyline)
  - Create interactive REPL with prompt; support load_script, simulate, state, layers, eval, help, exit commands
  - Add `rustyline = "14.0"` to Cargo.toml dependencies
  - _Leverage: core/src/scripting/runtime.rs (shared runtime), core/src/engine/state.rs (state snapshot)_
  - _Requirements: 1.2_
  - _Prompt: Implement the task for spec phase-1-3-completion, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust CLI/REPL engineer | Task: Replace stub in repl.rs with ReplCommand; FIRST add `rustyline = "14.0"` to Cargo.toml; use rustyline for readline with history; implement commands: load_script(path), simulate(keys), state() -> JSON, layers -> list, eval "code", help, exit; maintain shared RhaiRuntime instance | Restrictions: ≤400 lines; handle Ctrl+C gracefully (return to prompt, not exit); print errors without exiting; no panics | _Leverage: rustyline crate for readline with history/completion, existing runtime and state accessors | _Requirements: 1.2 | Success: `keyrx repl` opens prompt with history, commands work interactively, state displays correctly, Ctrl+C returns to prompt._

### Session Recording & Replay

- [x] 7. Implement EventRecord and SessionFile data structures
  - Files: core/src/engine/event_recording.rs (new)
  - Define EventRecord, SessionFile structs with serde Serialize/Deserialize; implement to_json/from_json
  - _Leverage: core/src/engine/types.rs (InputEvent, OutputAction), core/src/engine/state.rs (EngineSnapshot)_
  - _Requirements: 1.3_
  - _Prompt: Implement the task for spec phase-1-3-completion, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust data modeling engineer | Task: Create event_recording.rs with EventRecord { seq, timestamp_µs, input, output, decision_type, active_layers, modifiers_state, latency_µs } and SessionFile { version, created_at, script_used, timing_config, initial_state, events }; derive Serialize/Deserialize; implement from_json/to_json | Restrictions: ≤200 lines; use serde_json; compact serialization for events array | _Leverage: existing InputEvent, OutputAction, EngineSnapshot types | _Requirements: 1.3 | Success: Structures serialize/deserialize correctly, version field for schema evolution._

- [x] 8. Implement EventRecorder middleware
  - Files: core/src/engine/event_recording.rs (extend)
  - Create EventRecorder struct; implement record_event(), finish() methods; write to .krx file
  - _Leverage: core/src/engine/event_loop.rs (intercept point), std::fs::File_
  - _Requirements: 1.3_
  - _Prompt: Implement the task for spec phase-1-3-completion, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust I/O engineer | Task: Add EventRecorder { file, session_start, event_count } to event_recording.rs; implement new(path) -> Result, record_event(EventRecord) -> Result (append to buffer), finish(final_state) -> Result (write SessionFile JSON to .krx file) | Restrictions: ≤200 lines combined with task 7; buffer events in memory, flush on finish; handle I/O errors gracefully | _Leverage: std::fs, serde_json::to_writer_pretty | _Requirements: 1.3 | Success: Recording session creates valid .krx JSON file with all events._

- [x] 9. Integrate recording into keyrx run command
  - Files: core/src/cli/commands/run.rs (modify)
  - Add --record <path.krx> flag; wrap engine loop with EventRecorder; call finish on shutdown
  - _Leverage: core/src/engine/event_recording.rs (EventRecorder), existing run.rs structure_
  - _Requirements: 1.3_
  - _Prompt: Implement the task for spec phase-1-3-completion, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust CLI integration engineer | Task: Modify run.rs to add --record Option<PathBuf> argument; if present, create EventRecorder before engine loop; after each process_event, call recorder.record_event(); on SIGINT/shutdown, call recorder.finish() with final state | Restrictions: ≤50 lines added; maintain existing run behavior when --record not specified; handle recorder errors without crashing engine | _Leverage: existing RunCommand structure, EventRecorder API | _Requirements: 1.3 | Success: `keyrx run --record session.krx` creates valid .krx file on exit._

- [x] 10. Implement session replay as InputSource
  - Files: core/src/engine/replay.rs (new)
  - Create ReplaySession struct implementing InputSource trait; read .krx file, inject events with timing
  - _Leverage: core/src/traits/input_source.rs (InputSource trait), core/src/engine/event_recording.rs (SessionFile)_
  - _Requirements: 1.3_
  - _Prompt: Implement the task for spec phase-1-3-completion, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust async/timing engineer | Task: Create replay.rs with ReplaySession { events: VecDeque<EventRecord>, start_time }; impl InputSource for ReplaySession with start() loading events, poll() returning next event when timestamp elapsed, stop() clearing queue | Restrictions: ≤250 lines; use std::time for timing; deterministic replay (same inter-event delays) | _Leverage: InputSource trait, SessionFile::from_json | _Requirements: 1.3 | Success: ReplaySession injects events with correct timing, implements InputSource trait._

- [x] 11. Implement keyrx replay CLI command
  - Files: core/src/cli/commands/replay.rs (new), core/src/bin/keyrx.rs (add command)
  - Add `keyrx replay <session.krx>` command; load session, create ReplaySession, run engine, compare outputs
  - _Leverage: core/src/engine/replay.rs (ReplaySession), core/src/cli/commands/run.rs (engine setup pattern)_
  - _Requirements: 1.3_
  - _Prompt: Implement the task for spec phase-1-3-completion, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust CLI engineer | Task: Create replay.rs with ReplayCommand; implement execute() to load .krx file, create ReplaySession as InputSource, run engine, collect outputs, optionally verify against original recording | Restrictions: ≤300 lines; add to Commands enum; support --verify flag to compare outputs | _Leverage: ReplaySession, run.rs engine initialization pattern | _Requirements: 1.3 | Success: `keyrx replay session.krx` replays events deterministically, --verify reports match/mismatch._

- [x] 12. Implement keyrx analyze timing diagram command
  - Files: core/src/cli/commands/analyze.rs (new), core/src/bin/keyrx.rs (add command)
  - Add `keyrx analyze <session.krx> --diagram` command; parse .krx, generate ASCII timing table
  - _Leverage: core/src/engine/event_recording.rs (SessionFile parsing)_
  - _Requirements: 1.3_
  - _Prompt: Implement the task for spec phase-1-3-completion, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust visualization engineer | Task: Create analyze.rs with AnalyzeCommand; implement execute() to load .krx, generate ASCII table: Event | Input | Decision | Output | Latency (µs); format with box-drawing characters | Restrictions: ≤200 lines; support --json for machine-readable; add to Commands enum | _Leverage: SessionFile, cli/output.rs formatting patterns | _Requirements: 1.3 | Success: `keyrx analyze session.krx --diagram` outputs readable timing table._

### FFI Script Loading Fix

- [x] 13. Complete keyrx_load_script FFI function
  - Files: core/src/ffi/exports.rs (modify line ~72)
  - Replace TODO comment with actual script loading; use shared RhaiRuntime; return proper error codes
  - _Leverage: core/src/scripting/runtime.rs (load_file method), existing FFI patterns in exports.rs_
  - _Requirements: 1.4_
  - _Prompt: Implement the task for spec phase-1-3-completion, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust FFI engineer | Task: Complete keyrx_load_script() in exports.rs; replace TODO at line ~72 with: get shared runtime lock, call runtime.load_file(path), return 0 on success, -3 on syntax error, log errors with tracing | Restrictions: ≤30 lines change; no panics across FFI; maintain existing return codes (-1 null, -2 utf8) | _Leverage: SHARED_RUNTIME pattern, RhaiRuntime::load_file | _Requirements: 1.4 | Success: Flutter can load scripts via FFI, syntax errors return -3 with logged message._

### Phase 1 Tests

- [x] 14. Add unit tests for test harness and runner
  - Files: core/src/scripting/test_harness.rs (add #[cfg(test)] module), core/src/scripting/test_runner.rs (add tests)
  - Test simulate_tap generates events, assert_output validates correctly, discover_tests finds #[test] functions
  - _Leverage: existing test patterns in core/src/scripting/runtime.rs tests_
  - _Requirements: 1.1, NFR Test Coverage_
  - _Prompt: Implement the task for spec phase-1-3-completion, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust test engineer | Task: Add #[cfg(test)] modules to test_harness.rs and test_runner.rs; test: simulate_tap adds event to output queue, assert_output passes/fails correctly, discover_tests finds #[test] fn, run_tests returns correct results | Restrictions: ≤150 lines per file; deterministic tests; no I/O dependencies | _Leverage: existing test patterns, mock engine state | _Requirements: 1.1, NFR Test Coverage | Success: `cargo test` passes, ≥85% coverage on test harness/runner._

- [x] 15. Add integration tests for session recording/replay
  - Files: core/tests/session_recording_test.rs (new)
  - Test record 100 events, replay, verify outputs match; test corrupted file handling
  - _Leverage: core/src/engine/event_recording.rs, core/src/engine/replay.rs_
  - _Requirements: 1.3, NFR Test Coverage_
  - _Prompt: Implement the task for spec phase-1-3-completion, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust integration test engineer | Task: Create session_recording_test.rs; tests: record_100_events_and_replay_matches (record session, replay, compare outputs byte-for-byte), corrupted_file_returns_error (malformed JSON rejected), empty_session_handles_gracefully | Restrictions: ≤200 lines; use tempfile for .krx files; deterministic | _Leverage: EventRecorder, ReplaySession, tempfile crate | _Requirements: 1.3, NFR Test Coverage | Success: Integration tests pass, verify deterministic replay._

## Phase 2: Nervous System - Driver Integration

- [x] 16. Add OpenTelemetry dependencies and EngineTracer struct
  - Files: core/Cargo.toml (add deps), core/src/engine/tracing.rs (new)
  - Add opentelemetry, opentelemetry-otlp crates; create EngineTracer struct with span methods
  - _Leverage: opentelemetry crate documentation_
  - _Requirements: 2.1_
  - _Prompt: Implement the task for spec phase-1-3-completion, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust observability engineer | Task: Add opentelemetry = "0.21", opentelemetry-otlp = "0.14" to Cargo.toml (optional feature "tracing"); create tracing.rs with EngineTracer { tracer }; implement span_input_received(event), span_decision_made(decision, latency), span_output_generated(action) | Restrictions: ≤200 lines; feature-gated (compile without tracing); no runtime overhead when disabled | _Leverage: opentelemetry docs, tracing crate patterns | _Requirements: 2.1 | Success: EngineTracer compiles, spans emittable when feature enabled._

- [x] 17. Integrate tracing into engine process_event
  - Files: core/src/engine/advanced.rs (modify process_event)
  - Wrap process_event with trace spans; emit on input, decision, output
  - _Leverage: core/src/engine/tracing.rs (EngineTracer), existing process_event structure_
  - _Requirements: 2.1_
  - _Prompt: Implement the task for spec phase-1-3-completion, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust instrumentation engineer | Task: Modify process_event in advanced.rs to optionally accept &EngineTracer; wrap input processing in span_input_received, decision in span_decision_made, output in span_output_generated; pass latency_µs to spans | Restrictions: ≤50 lines added; no overhead when tracer is None; maintain existing function signature compatibility | _Leverage: EngineTracer API, existing latency measurement | _Requirements: 2.1 | Success: Trace spans emitted for each event when tracer provided._

- [x] 18. Add trace export to keyrx run command
  - Files: core/src/cli/commands/run.rs (modify)
  - Add --trace <file> flag; initialize OpenTelemetry exporter; export traces on shutdown
  - _Leverage: core/src/engine/tracing.rs, opentelemetry-otlp crate_
  - _Requirements: 2.1_
  - _Prompt: Implement the task for spec phase-1-3-completion, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust CLI/observability engineer | Task: Modify run.rs to add --trace Option<PathBuf>; if present, initialize OTLP file exporter, create EngineTracer, pass to engine loop; on shutdown, flush and export traces | Restrictions: ≤40 lines added; graceful degradation if export fails; feature-gated with "tracing" | _Leverage: opentelemetry::sdk::export, EngineTracer | _Requirements: 2.1 | Success: `keyrx run --trace events.otlp` exports valid OpenTelemetry traces._

- [x] 19. Add tracing unit tests
  - Files: core/src/engine/tracing.rs (add #[cfg(test)] module)
  - Test span creation, attribute setting, no-op when disabled
  - _Leverage: opentelemetry testing utilities_
  - _Requirements: 2.1, NFR Test Coverage_
  - _Prompt: Implement the task for spec phase-1-3-completion, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust test engineer | Task: Add #[cfg(test)] module to tracing.rs; tests: span_input_has_correct_attributes, span_decision_includes_latency, tracer_disabled_does_not_panic, multiple_spans_linked_correctly | Restrictions: ≤100 lines; use in-memory exporter for testing; deterministic | _Leverage: opentelemetry::testing, mock tracer | _Requirements: 2.1, NFR Test Coverage | Success: Tracing tests pass, ≥80% coverage._

## Phase 3: Flutter GUI Completion

### Debugger Enhancement

- [x] 20. Enhance debugger page with live state subscription
  - Files: ui/lib/pages/debugger.dart (modify)
  - Subscribe to EngineStateStream; update UI on each snapshot; display layers, held keys, modifiers, latency
  - _Leverage: ui/lib/ffi/bridge.dart (stateStream), existing debugger layout_
  - _Requirements: 3.1_
  - _Prompt: Implement the task for spec phase-1-3-completion, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Flutter state management engineer | Task: Modify debugger.dart to subscribe to bridge.stateStream; on each EngineSnapshot, update: activeLayers list, heldKeys chips, modifiers toggle display, latencyMicroseconds meter; animate value changes with AnimatedContainer | Restrictions: ≤100 lines added; maintain existing layout; dispose stream subscription properly | _Leverage: existing bridge.stateStream, StreamBuilder widget | _Requirements: 3.1 | Success: Debugger updates within 50ms of key press, shows all state fields._

- [x] 21. Add pending decision visualization to debugger
  - Files: ui/lib/pages/debugger.dart (extend)
  - Display pending tap-hold countdown timer; highlight combo keys in progress
  - _Leverage: EngineSnapshot.pending field, existing debugger widgets_
  - _Requirements: 3.1_
  - _Prompt: Implement the task for spec phase-1-3-completion, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Flutter animation engineer | Task: Add PendingDecisionWidget to debugger; when snapshot.pending contains tap-hold, show countdown CircularProgressIndicator with remaining ms; when combo in progress, highlight matched keys with pulsing border | Restrictions: ≤80 lines; smooth animations; no jank on rapid updates | _Leverage: EngineSnapshot.pending, Timer for countdown | _Requirements: 3.1 | Success: Tap-hold shows countdown, combo shows key highlights._

### Training Screen

- [x] 22. Implement TrainingScreen page with lesson framework
  - Files: ui/lib/pages/training_screen.dart (new/extend existing)
  - Create lesson data structure; implement step-by-step progression; validate user actions
  - _Leverage: ui/lib/services/engine_service.dart (eval, simulate), ui/lib/ffi/bridge.dart_
  - _Requirements: 3.2_
  - _Prompt: Implement the task for spec phase-1-3-completion, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Flutter instructional design engineer | Task: Create/extend training_screen.dart with TrainingLesson { title, steps } and TrainingStep { instruction, validator, hint }; implement lessonCarousel showing current step, validator checking user action via stateStream, hint button revealing guidance | Restrictions: ≤400 lines; 5 lessons minimum (remap, layer, modifier, tap-hold, combo); persist progress in SharedPreferences | _Leverage: bridge.stateStream for validation, SharedPreferences for progress | _Requirements: 3.2 | Success: User completes 5 lessons with step validation, progress persisted._

- [x] 23. Add training exercises with feedback
  - Files: ui/lib/pages/training_screen.dart (extend)
  - Implement interactive exercises; show success/failure feedback; track completion
  - _Leverage: existing training_screen structure, EngineService_
  - _Requirements: 3.2_
  - _Prompt: Implement the task for spec phase-1-3-completion, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Flutter UX engineer | Task: Add interactive exercises to each lesson: "Press A to see B" validates via stateStream output; show green checkmark on success, red X with explanation on failure; Certificate modal on all lessons complete | Restrictions: ≤150 lines added; animated feedback; accessible (screen reader friendly) | _Leverage: stateStream, AlertDialog for certificate | _Requirements: 3.2 | Success: Exercises validate correctly, feedback is clear, certificate appears._

### Trade-off Visualizer

- [x] 24. Implement trade-off visualizer page
  - Files: ui/lib/pages/trade_off_visualizer.dart (new), ui/pubspec.yaml (add fl_chart)
  - Create interactive chart showing tap-hold timeout vs miss rate; slider to adjust thresholds
  - Add `fl_chart: ^0.68.0` to pubspec.yaml dependencies, run `flutter pub get`
  - _Leverage: fl_chart package, ui/lib/ffi/bridge.dart (timing config)_
  - _Requirements: 3.3_
  - _Prompt: Implement the task for spec phase-1-3-completion, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Flutter data visualization engineer | Task: FIRST add `fl_chart: ^0.68.0` to pubspec.yaml and run `flutter pub get`; create trade_off_visualizer.dart with LineChart showing X=tap_hold_timeout_ms (100-1000), Y=estimated_miss_rate (0-30%); implement miss rate calculation using normal CDF (P(miss) = normalCdf(threshold, mean, stddev)); add Slider to adjust timeout, update chart point; show preset regions (Gaming: <150ms, Typing: 200ms, Relaxed: >300ms) | Restrictions: ≤350 lines; responsive layout; include statistical model for miss rate | _Leverage: fl_chart LineChart, bridge for timing config | _Requirements: 3.3 | Success: Chart renders with statistical curve, slider updates threshold, presets highlighted._

- [x] 25. Add typing speed simulation to trade-off visualizer
  - Files: ui/lib/pages/trade_off_visualizer.dart (extend)
  - Implement "Simulate my typing speed" button; measure user's inter-key delays; overlay on chart
  - _Leverage: existing trade_off_visualizer, keyboard event handling_
  - _Requirements: 3.3_
  - _Prompt: Implement the task for spec phase-1-3-completion, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Flutter measurement engineer | Task: Add "Simulate" button that prompts user to type sample text; measure inter-key delays; calculate mean/stddev; overlay UserTypingProfile on chart as vertical band; recommend threshold based on profile | Restrictions: ≤100 lines added; 30-second max simulation; cancel button available | _Leverage: RawKeyboardListener, statistics calculation | _Requirements: 3.3 | Success: Simulation measures typing speed, recommendation displayed._

### Console Enhancement

- [x] 26. Add error styling to console page
  - Files: ui/lib/pages/console.dart (modify)
  - Style ok: responses green, error: responses red; add icon indicators; quick action for "Engine not initialized"
  - _Leverage: existing console.dart output handling_
  - _Requirements: 3.4_
  - _Prompt: Implement the task for spec phase-1-3-completion, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Flutter UI engineer | Task: Modify console.dart to parse ok:/error: prefixes; style ok: with green text + checkmark icon, error: with red text + warning icon; if error contains "not initialized", show "Initialize Engine" ElevatedButton | Restrictions: ≤50 lines added; maintain existing scroll behavior; copyable text without prefix | _Leverage: existing _handleResponse method, Theme colors | _Requirements: 3.4 | Success: Console visually distinguishes success/error, quick action works._

### Flutter Tests

- [x] 27. Add widget tests for debugger enhancements
  - Files: ui/test/debugger_test.dart (new/extend)
  - Test state stream subscription, latency display, pending visualization
  - _Leverage: flutter_test, mockito for bridge mocking_
  - _Requirements: 3.1, NFR Test Coverage_
  - _Prompt: Implement the task for spec phase-1-3-completion, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Flutter test engineer | Task: Create/extend debugger_test.dart; tests: debugger_subscribes_to_state_stream, latency_meter_updates_on_snapshot, pending_tap_hold_shows_countdown, combo_keys_highlighted; use mock stateStream | Restrictions: ≤200 lines; deterministic; no real FFI calls | _Leverage: flutter_test, StreamController for mock stream | _Requirements: 3.1, NFR Test Coverage | Success: Widget tests pass, ≥75% coverage on debugger._

- [x] 28. Add widget tests for training screen
  - Files: ui/test/training_screen_test.dart (new)
  - Test lesson progression, exercise validation, completion tracking
  - _Leverage: flutter_test, SharedPreferences mock_
  - _Requirements: 3.2, NFR Test Coverage_
  - _Prompt: Implement the task for spec phase-1-3-completion, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Flutter test engineer | Task: Create training_screen_test.dart; tests: lesson_displays_current_step, exercise_validates_correct_input, exercise_shows_error_on_wrong_input, completion_shows_certificate, progress_persisted; mock SharedPreferences | Restrictions: ≤200 lines; deterministic; isolated tests | _Leverage: flutter_test, shared_preferences mock | _Requirements: 3.2, NFR Test Coverage | Success: Training tests pass, ≥75% coverage._

- [x] 29. Add widget tests for trade-off visualizer and console
  - Files: ui/test/trade_off_test.dart (new), ui/test/console_styling_test.dart (new)
  - Test chart rendering, slider interaction, console ok/error styling
  - _Leverage: flutter_test_
  - _Requirements: 3.3, 3.4, NFR Test Coverage_
  - _Prompt: Implement the task for spec phase-1-3-completion, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Flutter test engineer | Task: Create trade_off_test.dart (chart renders, slider updates value, presets visible) and console_styling_test.dart (ok: shows green, error: shows red, quick action button appears); | Restrictions: ≤150 lines each; deterministic | _Leverage: flutter_test, find.byType for chart/widgets | _Requirements: 3.3, 3.4, NFR Test Coverage | Success: Tests pass, ≥75% coverage on visualizer and console._

## Final Integration

- [x] 30. Final integration test and cleanup
  - Files: core/tests/phase_1_3_integration_test.rs (new), ui/integration_test/ (new)
  - End-to-end test: test script → run → record → replay → analyze; Flutter training → debugger flow
  - _Leverage: all implemented components_
  - _Requirements: All, NFR Test Coverage_
  - _Prompt: Implement the task for spec phase-1-3-completion, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Integration test engineer | Task: Create phase_1_3_integration_test.rs with test: write_test_script_run_and_verify, record_replay_deterministic_match, analyze_outputs_timing_diagram; create Flutter integration test: training_to_debugger_flow; verify all commands work end-to-end | Restrictions: ≤300 lines Rust, ≤200 lines Dart; use tempdir for artifacts; cleanup after tests | _Leverage: all Phase 1-3 components, tempfile | _Requirements: All, NFR Test Coverage | Success: Integration tests pass, demonstrates full feature chain._

## Phase 4: Hardening & Compliance

### Emergency Exit Implementation (Critical Safety)

- [x] 31. Implement emergency exit handler module
  - Files: core/src/drivers/emergency_exit.rs (new)
  - Create EmergencyExit module with BYPASS_MODE atomic flag; implement check_emergency_exit(), activate_bypass_mode(), deactivate_bypass_mode(), is_bypass_active()
  - _Leverage: std::sync::atomic::AtomicBool, existing ModifierState_
  - _Requirements: 4.1_
  - _Prompt: Implement the task for spec phase-1-3-completion, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust systems safety engineer | Task: Create emergency_exit.rs with static BYPASS_MODE: AtomicBool; implement check_emergency_exit(key, mods) -> bool that returns true when Ctrl+Alt+Shift+Escape pressed; implement activate_bypass_mode() that sets flag and logs warning; implement deactivate_bypass_mode() and is_bypass_active() | Restrictions: ≤150 lines; thread-safe with SeqCst ordering; no panics; include callback hook for UI notification | _Leverage: std::sync::atomic, tracing crate | _Requirements: 4.1 | Success: Emergency exit detection works, bypass mode toggles correctly, thread-safe._

- [x] 32. Integrate emergency exit into Windows driver
  - Files: core/src/drivers/windows/hook.rs (modify)
  - Add emergency exit check at start of keyboard_proc, before any processing; if triggered, activate bypass and return pass-through
  - _Leverage: core/src/drivers/emergency_exit.rs, existing Windows hook structure_
  - _Requirements: 4.1_
  - _Prompt: Implement the task for spec phase-1-3-completion, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Windows driver engineer | Task: Modify keyboard_proc in hook.rs to check emergency_exit::check_emergency_exit() FIRST before any other processing; if true, call activate_bypass_mode() and return CallNextHookEx immediately; if bypass_mode active, always pass through | Restrictions: ≤30 lines added; must be FIRST check in callback; no performance regression | _Leverage: emergency_exit module, existing hook structure | _Requirements: 4.1 | Success: Ctrl+Alt+Shift+Escape disables remapping on Windows, all keys pass through._

- [x] 33. Integrate emergency exit into Linux driver
  - Files: core/src/drivers/linux/evdev.rs (modify)
  - Add emergency exit check at start of event processing; if triggered, ungrab device and activate bypass mode
  - _Leverage: core/src/drivers/emergency_exit.rs, existing evdev handler_
  - _Requirements: 4.1_
  - _Prompt: Implement the task for spec phase-1-3-completion, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Linux driver engineer | Task: Modify evdev event loop to check emergency_exit::check_emergency_exit() FIRST; if true, call device.ungrab() to release exclusive access, then activate_bypass_mode(); if bypass active, skip all processing | Restrictions: ≤30 lines added; must ungrab device so keys flow to OS; no panics on ungrab failure | _Leverage: emergency_exit module, evdev::Device::ungrab | _Requirements: 4.1 | Success: Ctrl+Alt+Shift+Escape disables remapping on Linux, device ungrabbed._

- [x] 34. Add emergency exit FFI exports and Flutter UI indicator
  - Files: core/src/ffi/exports.rs (modify), ui/lib/widgets/bypass_indicator.dart (new)
  - Export keyrx_is_bypass_active(), keyrx_set_bypass(); create Flutter widget showing bypass status
  - _Leverage: existing FFI patterns, Flutter Provider state_
  - _Requirements: 4.1_
  - _Prompt: Implement the task for spec phase-1-3-completion, first run spec-workflow-guide to get the workflow guide then implement the task: Role: FFI/Flutter engineer | Task: Add keyrx_is_bypass_active() -> bool and keyrx_set_bypass(active: bool) to exports.rs; create bypass_indicator.dart with BypassIndicator widget that shows red "REMAPPING DISABLED" banner when bypass active, with "Re-enable" button | Restrictions: ≤50 lines Rust FFI, ≤100 lines Dart; poll bypass status every 500ms or use callback | _Leverage: existing FFI export patterns, emergency_exit module | _Requirements: 4.1 | Success: Flutter shows bypass status, user can re-enable remapping._

- [x] 35. Add emergency exit unit and integration tests
  - Files: core/src/drivers/emergency_exit.rs (add tests), core/tests/emergency_exit_test.rs (new)
  - Test combo detection, bypass mode activation/deactivation, thread safety
  - _Leverage: existing test patterns_
  - _Requirements: 4.1, NFR Test Coverage_
  - _Prompt: Implement the task for spec phase-1-3-completion, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust test engineer | Task: Add #[cfg(test)] module to emergency_exit.rs with tests: combo_detected_with_all_mods, combo_not_detected_partial_mods, bypass_mode_toggles, thread_safety_concurrent_access; create emergency_exit_test.rs with integration test simulating key sequence | Restrictions: ≤150 lines; deterministic; test thread safety with multiple threads | _Leverage: std::thread, existing test patterns | _Requirements: 4.1, NFR Test Coverage | Success: All tests pass, ≥90% coverage on emergency exit module._

### CI Performance Regression Detection

- [x] 36. Implement CI benchmark workflow and regression check
  - Files: .github/workflows/benchmark.yml (new), scripts/check_bench_regression.py (new)
  - Create GitHub Actions workflow running Criterion benchmarks; Python script to parse output and fail on >100µs regression
  - _Leverage: Criterion benchmark output format, existing core/benches/_
  - _Requirements: 4.3_
  - _Prompt: Implement the task for spec phase-1-3-completion, first run spec-workflow-guide to get the workflow guide then implement the task: Role: DevOps/CI engineer | Task: Create benchmark.yml workflow that runs on PRs: checkout, cargo bench --bench latency, compare with main baseline; create check_bench_regression.py that parses Criterion JSON output, compares with baseline, fails if any benchmark >100µs slower; output detailed report | Restrictions: benchmark.yml ≤50 lines; Python script ≤150 lines; cache Cargo for speed | _Leverage: Criterion --save-baseline, --baseline flags, JSON output | _Requirements: 4.3 | Success: CI runs benchmarks on PRs, fails on regression, reports delta._

### Code Quality Refactoring (File Size Compliance)

- [x] 37. Refactor test_harness.rs into modular structure
  - Files: core/src/scripting/test_harness.rs → test_harness.rs + test_primitives.rs
  - Split 802-line file: test_harness.rs (TestHarness, register_test_functions ~400 lines), test_primitives.rs (simulate_*, assert_* implementations ~400 lines)
  - _Leverage: existing module patterns in scripting/_
  - _Requirements: 4.4, NFR Code Architecture_
  - _Prompt: Implement the task for spec phase-1-3-completion, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust refactoring engineer | Task: Split test_harness.rs (802 lines) into test_harness.rs (TestHarness struct, register_test_functions, public API ~400 lines) and test_primitives.rs (simulate_tap, simulate_hold, assert_output, assert_mapping implementations ~400 lines); maintain re-exports in test_harness.rs | Restrictions: No API changes; all tests must pass; each file ≤500 lines | _Leverage: existing module patterns | _Requirements: 4.4 | Success: Both files ≤500 lines, cargo test passes, API unchanged._

- [x] 38. Refactor test_runner.rs into modular structure
  - Files: core/src/scripting/test_runner.rs → test_runner.rs + test_discovery.rs
  - Split 741-line file: test_runner.rs (TestRunner, run_tests ~350 lines), test_discovery.rs (discover_tests, AST parsing ~350 lines)
  - _Leverage: existing module patterns_
  - _Requirements: 4.4, NFR Code Architecture_
  - _Prompt: Implement the task for spec phase-1-3-completion, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust refactoring engineer | Task: Split test_runner.rs (741 lines) into test_runner.rs (TestRunner struct, run_tests, result formatting ~350 lines) and test_discovery.rs (discover_tests, AST iteration, test_ prefix detection ~350 lines); maintain re-exports | Restrictions: No API changes; all tests must pass; each file ≤500 lines | _Leverage: existing patterns | _Requirements: 4.4 | Success: Both files ≤500 lines, cargo test passes._

- [x] 39. Refactor event_recording.rs into modular structure
  - Files: core/src/engine/event_recording.rs → event_recording.rs + event_recorder.rs
  - Split 732-line file: event_recording.rs (EventRecord, SessionFile, serialization ~350 lines), event_recorder.rs (EventRecorder middleware ~350 lines)
  - _Leverage: existing engine module patterns_
  - _Requirements: 4.4, NFR Code Architecture_
  - _Prompt: Implement the task for spec phase-1-3-completion, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust refactoring engineer | Task: Split event_recording.rs (732 lines) into event_recording.rs (EventRecord, SessionFile structs, serde impls, to_json/from_json ~350 lines) and event_recorder.rs (EventRecorder struct, record_event, finish ~350 lines) | Restrictions: No API changes; all tests must pass; each file ≤500 lines | _Leverage: existing patterns | _Requirements: 4.4 | Success: Both files ≤500 lines, cargo test passes._

- [x] 40. Refactor advanced.rs into modular structure
  - Files: core/src/engine/advanced.rs → advanced.rs + decision_engine.rs
  - Split 706-line file: advanced.rs (process_event core ~350 lines), decision_engine.rs (tap-hold, combo algorithms ~350 lines)
  - _Leverage: existing engine module patterns_
  - _Requirements: 4.4, NFR Code Architecture_
  - _Prompt: Implement the task for spec phase-1-3-completion, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust refactoring engineer | Task: Split advanced.rs (706 lines) into advanced.rs (process_event, state management ~350 lines) and decision_engine.rs (tap_hold_decision, combo_decision, timing logic ~350 lines) | Restrictions: No API changes; all tests must pass; each file ≤500 lines; decision_engine should be pure functions where possible | _Leverage: existing patterns | _Requirements: 4.4 | Success: Both files ≤500 lines, cargo test passes._

- [ ] 41. Refactor debugger.dart into modular structure
  - Files: ui/lib/pages/debugger.dart → debugger_page.dart + debugger_widgets.dart + debugger_meters.dart
  - Split 969-line file into 3 focused files each ≤400 lines
  - _Leverage: existing Flutter widget patterns_
  - _Requirements: 4.4, NFR Code Architecture_
  - _Prompt: Implement the task for spec phase-1-3-completion, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Flutter refactoring engineer | Task: Split debugger.dart (969 lines) into debugger_page.dart (DebuggerPage widget, state management ~400 lines), debugger_widgets.dart (TimelineWidget, LayerPanelWidget ~300 lines), debugger_meters.dart (LatencyMeter, PendingDecisionWidget ~270 lines) | Restrictions: No functionality changes; all tests must pass; each file ≤400 lines | _Leverage: existing widget extraction patterns | _Requirements: 4.4 | Success: All files ≤400 lines, flutter test passes._

- [ ] 42. Refactor trade_off_visualizer.dart into modular structure
  - Files: ui/lib/pages/trade_off_visualizer.dart → trade_off_page.dart + trade_off_chart.dart + typing_simulator.dart
  - Split 949-line file into 3 focused files each ≤400 lines
  - _Leverage: existing Flutter widget patterns_
  - _Requirements: 4.4, NFR Code Architecture_
  - _Prompt: Implement the task for spec phase-1-3-completion, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Flutter refactoring engineer | Task: Split trade_off_visualizer.dart (949 lines) into trade_off_page.dart (main page, state ~400 lines), trade_off_chart.dart (fl_chart config, rendering ~300 lines), typing_simulator.dart (speed measurement, statistics ~250 lines) | Restrictions: No functionality changes; all tests must pass; each file ≤400 lines | _Leverage: existing patterns | _Requirements: 4.4 | Success: All files ≤400 lines, flutter test passes._

### Visual Editor Tier 1 (No-Code Experience)

- [ ] 43. Implement visual keyboard layout widget
  - Files: ui/lib/widgets/visual_keyboard.dart (new), ui/lib/models/keyboard_layout.dart (new)
  - Create interactive keyboard widget with ANSI layout; render keys with proper sizing and positioning
  - _Leverage: existing key registry, Flutter CustomPaint or Stack layout_
  - _Requirements: 4.2_
  - _Prompt: Implement the task for spec phase-1-3-completion, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Flutter UI engineer | Task: Create keyboard_layout.dart with KeyboardLayout, KeyDefinition classes for ANSI 104-key layout; create visual_keyboard.dart with VisualKeyboard StatefulWidget rendering keys as tappable Container widgets with proper row/column positioning and width (1.0u = standard key, 1.5u = Tab, 2.0u = Shift, etc.) | Restrictions: ≤300 lines each; responsive sizing; highlight on tap | _Leverage: existing key names from scripts/std/layouts/ansi.rhai | _Requirements: 4.2 | Success: ANSI keyboard renders correctly, keys are tappable._

- [ ] 44. Implement drag-and-drop key mapping interaction
  - Files: ui/lib/widgets/visual_keyboard.dart (extend), ui/lib/widgets/mapping_overlay.dart (new)
  - Add Draggable/DragTarget to keys; draw mapping arrows between connected keys
  - _Leverage: Flutter Draggable widget, CustomPaint for arrows_
  - _Requirements: 4.2_
  - _Prompt: Implement the task for spec phase-1-3-completion, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Flutter interaction engineer | Task: Extend VisualKeyboard to wrap each key in Draggable; add DragTarget to accept drops; create mapping_overlay.dart with MappingOverlay CustomPainter that draws arrows from source to target keys; maintain List<RemapConfig> state; show "X" button on mapping to delete | Restrictions: ≤200 lines added to visual_keyboard, ≤150 lines mapping_overlay; smooth drag feedback | _Leverage: Draggable, DragTarget, CustomPaint | _Requirements: 4.2 | Success: Drag key A to B creates mapping with arrow, deletable._

- [ ] 45. Implement Rhai code generator service
  - Files: ui/lib/services/rhai_generator.dart (new)
  - Generate Rhai script from visual config; parse simple scripts back to visual config (best-effort)
  - _Leverage: Rhai syntax knowledge, existing script examples_
  - _Requirements: 4.2_
  - _Prompt: Implement the task for spec phase-1-3-completion, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Flutter/Rhai engineer | Task: Create rhai_generator.dart with RhaiGenerator class; implement generateScript(VisualConfig) returning valid Rhai code with remap(), tap_hold() calls; implement parseScript(String) that regex-parses simple remap/tap_hold patterns back to VisualConfig; set hasAdvancedFeatures=true if unparseable constructs found | Restrictions: ≤250 lines; handle edge cases (special chars in key names); add header comment with generation timestamp | _Leverage: RegExp for parsing, StringBuffer for generation | _Requirements: 4.2 | Success: Generated scripts are valid Rhai, simple scripts parse back correctly._

- [ ] 46. Implement visual editor page with "Eject to Code" feature
  - Files: ui/lib/pages/visual_editor_page.dart (new)
  - Create complete visual editor page combining keyboard, mappings, and code view; "Eject to Code" shows generated Rhai
  - _Leverage: VisualKeyboard, RhaiGenerator, existing page patterns_
  - _Requirements: 4.2_
  - _Prompt: Implement the task for spec phase-1-3-completion, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Flutter page engineer | Task: Create visual_editor_page.dart with VisualEditorPage; layout: VisualKeyboard (top), MappingList (side), "Show Code" toggle button; when toggled, show generated Rhai in syntax-highlighted CodeEditor; "Save" button writes .rhai file; warn if manually edited code loses visual sync | Restrictions: ≤400 lines; clean layout with proper spacing; include file picker for save location | _Leverage: VisualKeyboard, RhaiGenerator, file_picker package | _Requirements: 4.2 | Success: Full visual editing workflow works, code ejection shows valid Rhai._

- [ ] 47. Add visual editor tests
  - Files: ui/test/visual_keyboard_test.dart (new), ui/test/rhai_generator_test.dart (new)
  - Test keyboard rendering, drag-drop interaction, code generation/parsing
  - _Leverage: flutter_test, existing widget test patterns_
  - _Requirements: 4.2, NFR Test Coverage_
  - _Prompt: Implement the task for spec phase-1-3-completion, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Flutter test engineer | Task: Create visual_keyboard_test.dart with tests: keyboard_renders_all_keys, key_tap_triggers_callback, drag_creates_mapping; create rhai_generator_test.dart with tests: generates_valid_remap_syntax, generates_tap_hold, parses_simple_script, detects_advanced_features | Restrictions: ≤150 lines each; deterministic; no real file I/O | _Leverage: flutter_test, find.byKey | _Requirements: 4.2, NFR Test Coverage | Success: All tests pass, ≥80% coverage on visual editor components._

## Phase 4 Final Integration

- [ ] 48. Phase 4 integration test and documentation
  - Files: core/tests/phase_4_integration_test.rs (new), ui/integration_test/visual_editor_flow_test.dart (new)
  - End-to-end test: emergency exit flow, visual editor → code generation → load script
  - _Leverage: all Phase 4 components_
  - _Requirements: All Phase 4, NFR Test Coverage_
  - _Prompt: Implement the task for spec phase-1-3-completion, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Integration test engineer | Task: Create phase_4_integration_test.rs testing: emergency_exit_activates_and_deactivates, bypass_mode_passes_all_keys; create visual_editor_flow_test.dart testing: create_mapping_visually_eject_to_code_load_script; update README with visual editor and emergency exit documentation | Restrictions: ≤200 lines each; use mock input for safety tests | _Leverage: all Phase 4 components | _Requirements: All Phase 4 | Success: Integration tests pass, documentation complete._

