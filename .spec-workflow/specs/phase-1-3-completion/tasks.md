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

- [ ] 17. Integrate tracing into engine process_event
  - Files: core/src/engine/advanced.rs (modify process_event)
  - Wrap process_event with trace spans; emit on input, decision, output
  - _Leverage: core/src/engine/tracing.rs (EngineTracer), existing process_event structure_
  - _Requirements: 2.1_
  - _Prompt: Implement the task for spec phase-1-3-completion, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust instrumentation engineer | Task: Modify process_event in advanced.rs to optionally accept &EngineTracer; wrap input processing in span_input_received, decision in span_decision_made, output in span_output_generated; pass latency_µs to spans | Restrictions: ≤50 lines added; no overhead when tracer is None; maintain existing function signature compatibility | _Leverage: EngineTracer API, existing latency measurement | _Requirements: 2.1 | Success: Trace spans emitted for each event when tracer provided._

- [ ] 18. Add trace export to keyrx run command
  - Files: core/src/cli/commands/run.rs (modify)
  - Add --trace <file> flag; initialize OpenTelemetry exporter; export traces on shutdown
  - _Leverage: core/src/engine/tracing.rs, opentelemetry-otlp crate_
  - _Requirements: 2.1_
  - _Prompt: Implement the task for spec phase-1-3-completion, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust CLI/observability engineer | Task: Modify run.rs to add --trace Option<PathBuf>; if present, initialize OTLP file exporter, create EngineTracer, pass to engine loop; on shutdown, flush and export traces | Restrictions: ≤40 lines added; graceful degradation if export fails; feature-gated with "tracing" | _Leverage: opentelemetry::sdk::export, EngineTracer | _Requirements: 2.1 | Success: `keyrx run --trace events.otlp` exports valid OpenTelemetry traces._

- [ ] 19. Add tracing unit tests
  - Files: core/src/engine/tracing.rs (add #[cfg(test)] module)
  - Test span creation, attribute setting, no-op when disabled
  - _Leverage: opentelemetry testing utilities_
  - _Requirements: 2.1, NFR Test Coverage_
  - _Prompt: Implement the task for spec phase-1-3-completion, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust test engineer | Task: Add #[cfg(test)] module to tracing.rs; tests: span_input_has_correct_attributes, span_decision_includes_latency, tracer_disabled_does_not_panic, multiple_spans_linked_correctly | Restrictions: ≤100 lines; use in-memory exporter for testing; deterministic | _Leverage: opentelemetry::testing, mock tracer | _Requirements: 2.1, NFR Test Coverage | Success: Tracing tests pass, ≥80% coverage._

## Phase 3: Flutter GUI Completion

### Debugger Enhancement

- [ ] 20. Enhance debugger page with live state subscription
  - Files: ui/lib/pages/debugger.dart (modify)
  - Subscribe to EngineStateStream; update UI on each snapshot; display layers, held keys, modifiers, latency
  - _Leverage: ui/lib/ffi/bridge.dart (stateStream), existing debugger layout_
  - _Requirements: 3.1_
  - _Prompt: Implement the task for spec phase-1-3-completion, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Flutter state management engineer | Task: Modify debugger.dart to subscribe to bridge.stateStream; on each EngineSnapshot, update: activeLayers list, heldKeys chips, modifiers toggle display, latencyMicroseconds meter; animate value changes with AnimatedContainer | Restrictions: ≤100 lines added; maintain existing layout; dispose stream subscription properly | _Leverage: existing bridge.stateStream, StreamBuilder widget | _Requirements: 3.1 | Success: Debugger updates within 50ms of key press, shows all state fields._

- [ ] 21. Add pending decision visualization to debugger
  - Files: ui/lib/pages/debugger.dart (extend)
  - Display pending tap-hold countdown timer; highlight combo keys in progress
  - _Leverage: EngineSnapshot.pending field, existing debugger widgets_
  - _Requirements: 3.1_
  - _Prompt: Implement the task for spec phase-1-3-completion, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Flutter animation engineer | Task: Add PendingDecisionWidget to debugger; when snapshot.pending contains tap-hold, show countdown CircularProgressIndicator with remaining ms; when combo in progress, highlight matched keys with pulsing border | Restrictions: ≤80 lines; smooth animations; no jank on rapid updates | _Leverage: EngineSnapshot.pending, Timer for countdown | _Requirements: 3.1 | Success: Tap-hold shows countdown, combo shows key highlights._

### Training Screen

- [ ] 22. Implement TrainingScreen page with lesson framework
  - Files: ui/lib/pages/training_screen.dart (new/extend existing)
  - Create lesson data structure; implement step-by-step progression; validate user actions
  - _Leverage: ui/lib/services/engine_service.dart (eval, simulate), ui/lib/ffi/bridge.dart_
  - _Requirements: 3.2_
  - _Prompt: Implement the task for spec phase-1-3-completion, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Flutter instructional design engineer | Task: Create/extend training_screen.dart with TrainingLesson { title, steps } and TrainingStep { instruction, validator, hint }; implement lessonCarousel showing current step, validator checking user action via stateStream, hint button revealing guidance | Restrictions: ≤400 lines; 5 lessons minimum (remap, layer, modifier, tap-hold, combo); persist progress in SharedPreferences | _Leverage: bridge.stateStream for validation, SharedPreferences for progress | _Requirements: 3.2 | Success: User completes 5 lessons with step validation, progress persisted._

- [ ] 23. Add training exercises with feedback
  - Files: ui/lib/pages/training_screen.dart (extend)
  - Implement interactive exercises; show success/failure feedback; track completion
  - _Leverage: existing training_screen structure, EngineService_
  - _Requirements: 3.2_
  - _Prompt: Implement the task for spec phase-1-3-completion, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Flutter UX engineer | Task: Add interactive exercises to each lesson: "Press A to see B" validates via stateStream output; show green checkmark on success, red X with explanation on failure; Certificate modal on all lessons complete | Restrictions: ≤150 lines added; animated feedback; accessible (screen reader friendly) | _Leverage: stateStream, AlertDialog for certificate | _Requirements: 3.2 | Success: Exercises validate correctly, feedback is clear, certificate appears._

### Trade-off Visualizer

- [ ] 24. Implement trade-off visualizer page
  - Files: ui/lib/pages/trade_off_visualizer.dart (new), ui/pubspec.yaml (add fl_chart)
  - Create interactive chart showing tap-hold timeout vs miss rate; slider to adjust thresholds
  - Add `fl_chart: ^0.68.0` to pubspec.yaml dependencies, run `flutter pub get`
  - _Leverage: fl_chart package, ui/lib/ffi/bridge.dart (timing config)_
  - _Requirements: 3.3_
  - _Prompt: Implement the task for spec phase-1-3-completion, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Flutter data visualization engineer | Task: FIRST add `fl_chart: ^0.68.0` to pubspec.yaml and run `flutter pub get`; create trade_off_visualizer.dart with LineChart showing X=tap_hold_timeout_ms (100-1000), Y=estimated_miss_rate (0-30%); implement miss rate calculation using normal CDF (P(miss) = normalCdf(threshold, mean, stddev)); add Slider to adjust timeout, update chart point; show preset regions (Gaming: <150ms, Typing: 200ms, Relaxed: >300ms) | Restrictions: ≤350 lines; responsive layout; include statistical model for miss rate | _Leverage: fl_chart LineChart, bridge for timing config | _Requirements: 3.3 | Success: Chart renders with statistical curve, slider updates threshold, presets highlighted._

- [ ] 25. Add typing speed simulation to trade-off visualizer
  - Files: ui/lib/pages/trade_off_visualizer.dart (extend)
  - Implement "Simulate my typing speed" button; measure user's inter-key delays; overlay on chart
  - _Leverage: existing trade_off_visualizer, keyboard event handling_
  - _Requirements: 3.3_
  - _Prompt: Implement the task for spec phase-1-3-completion, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Flutter measurement engineer | Task: Add "Simulate" button that prompts user to type sample text; measure inter-key delays; calculate mean/stddev; overlay UserTypingProfile on chart as vertical band; recommend threshold based on profile | Restrictions: ≤100 lines added; 30-second max simulation; cancel button available | _Leverage: RawKeyboardListener, statistics calculation | _Requirements: 3.3 | Success: Simulation measures typing speed, recommendation displayed._

### Console Enhancement

- [ ] 26. Add error styling to console page
  - Files: ui/lib/pages/console.dart (modify)
  - Style ok: responses green, error: responses red; add icon indicators; quick action for "Engine not initialized"
  - _Leverage: existing console.dart output handling_
  - _Requirements: 3.4_
  - _Prompt: Implement the task for spec phase-1-3-completion, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Flutter UI engineer | Task: Modify console.dart to parse ok:/error: prefixes; style ok: with green text + checkmark icon, error: with red text + warning icon; if error contains "not initialized", show "Initialize Engine" ElevatedButton | Restrictions: ≤50 lines added; maintain existing scroll behavior; copyable text without prefix | _Leverage: existing _handleResponse method, Theme colors | _Requirements: 3.4 | Success: Console visually distinguishes success/error, quick action works._

### Flutter Tests

- [ ] 27. Add widget tests for debugger enhancements
  - Files: ui/test/debugger_test.dart (new/extend)
  - Test state stream subscription, latency display, pending visualization
  - _Leverage: flutter_test, mockito for bridge mocking_
  - _Requirements: 3.1, NFR Test Coverage_
  - _Prompt: Implement the task for spec phase-1-3-completion, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Flutter test engineer | Task: Create/extend debugger_test.dart; tests: debugger_subscribes_to_state_stream, latency_meter_updates_on_snapshot, pending_tap_hold_shows_countdown, combo_keys_highlighted; use mock stateStream | Restrictions: ≤200 lines; deterministic; no real FFI calls | _Leverage: flutter_test, StreamController for mock stream | _Requirements: 3.1, NFR Test Coverage | Success: Widget tests pass, ≥75% coverage on debugger._

- [ ] 28. Add widget tests for training screen
  - Files: ui/test/training_screen_test.dart (new)
  - Test lesson progression, exercise validation, completion tracking
  - _Leverage: flutter_test, SharedPreferences mock_
  - _Requirements: 3.2, NFR Test Coverage_
  - _Prompt: Implement the task for spec phase-1-3-completion, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Flutter test engineer | Task: Create training_screen_test.dart; tests: lesson_displays_current_step, exercise_validates_correct_input, exercise_shows_error_on_wrong_input, completion_shows_certificate, progress_persisted; mock SharedPreferences | Restrictions: ≤200 lines; deterministic; isolated tests | _Leverage: flutter_test, shared_preferences mock | _Requirements: 3.2, NFR Test Coverage | Success: Training tests pass, ≥75% coverage._

- [ ] 29. Add widget tests for trade-off visualizer and console
  - Files: ui/test/trade_off_test.dart (new), ui/test/console_styling_test.dart (new)
  - Test chart rendering, slider interaction, console ok/error styling
  - _Leverage: flutter_test_
  - _Requirements: 3.3, 3.4, NFR Test Coverage_
  - _Prompt: Implement the task for spec phase-1-3-completion, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Flutter test engineer | Task: Create trade_off_test.dart (chart renders, slider updates value, presets visible) and console_styling_test.dart (ok: shows green, error: shows red, quick action button appears); | Restrictions: ≤150 lines each; deterministic | _Leverage: flutter_test, find.byType for chart/widgets | _Requirements: 3.3, 3.4, NFR Test Coverage | Success: Tests pass, ≥75% coverage on visualizer and console._

## Final Integration

- [ ] 30. Final integration test and cleanup
  - Files: core/tests/phase_1_3_integration_test.rs (new), ui/integration_test/ (new)
  - End-to-end test: test script → run → record → replay → analyze; Flutter training → debugger flow
  - _Leverage: all implemented components_
  - _Requirements: All, NFR Test Coverage_
  - _Prompt: Implement the task for spec phase-1-3-completion, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Integration test engineer | Task: Create phase_1_3_integration_test.rs with test: write_test_script_run_and_verify, record_replay_deterministic_match, analyze_outputs_timing_diagram; create Flutter integration test: training_to_debugger_flow; verify all commands work end-to-end | Restrictions: ≤300 lines Rust, ≤200 lines Dart; use tempdir for artifacts; cleanup after tests | _Leverage: all Phase 1-3 components, tempfile | _Requirements: All, NFR Test Coverage | Success: Integration tests pass, demonstrates full feature chain._

