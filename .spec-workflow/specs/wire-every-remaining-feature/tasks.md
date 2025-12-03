# Tasks Document

## FFI Layer (Rust)

- [x] 1. Add device listing FFI export
  - Files: core/src/ffi/mod.rs (modify)
  - Export `keyrx_list_devices()` returning JSON array of devices
  - _Leverage: core/src/cli/commands/devices.rs_
  - _Requirements: 1_
  - _Prompt: Implement the task for spec wire-every-remaining-feature, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust FFI engineer | Task: Add `keyrx_list_devices` FFI export to core/src/ffi/mod.rs that returns JSON array [{name, vendorId, productId, path, hasProfile}] using existing devices.rs logic | Restrictions: ≤50 lines; return CString; handle errors as JSON | _Leverage: core/src/cli/commands/devices.rs | _Requirements: 1 | Success: FFI function callable, returns valid JSON device list._

- [x] 2. Add device selection FFI export
  - Files: core/src/ffi/mod.rs (modify)
  - Export `keyrx_select_device(path)` to set active device
  - _Leverage: existing device registry_
  - _Requirements: 1_
  - _Prompt: Implement the task for spec wire-every-remaining-feature, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust FFI engineer | Task: Add `keyrx_select_device` FFI export that takes device path as CString, sets it as active device in engine config, returns 0 on success | Restrictions: ≤30 lines; validate path exists | _Leverage: device registry | _Requirements: 1 | Success: Selected device persists and is used on engine start._

- [x] 3. Add script validation FFI export
  - Files: core/src/ffi/mod.rs (modify)
  - Export `keyrx_check_script(path)` returning validation result JSON
  - _Leverage: core/src/cli/commands/check.rs_
  - _Requirements: 3_
  - _Prompt: Implement the task for spec wire-every-remaining-feature, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust FFI engineer | Task: Add `keyrx_check_script` FFI export that validates Rhai script syntax, returns JSON {valid: bool, errors: [{line, column, message}]} | Restrictions: ≤60 lines; reuse check.rs parser | _Leverage: core/src/cli/commands/check.rs | _Requirements: 3 | Success: Validation errors include line/column for UI highlighting._

- [x] 4. Add test discovery and execution FFI exports
  - Files: core/src/ffi/mod.rs (modify)
  - Export `keyrx_discover_tests(path)` and `keyrx_run_tests(path, filter)`
  - _Leverage: core/src/cli/commands/test.rs_
  - _Requirements: 5_
  - _Prompt: Implement the task for spec wire-every-remaining-feature, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust FFI engineer | Task: Add `keyrx_discover_tests` returning JSON array of test_* functions [{name, file, line}], and `keyrx_run_tests` returning JSON results [{name, passed, error, durationMs}] | Restrictions: ≤100 lines; support filter pattern | _Leverage: core/src/cli/commands/test.rs | _Requirements: 5 | Success: Tests discoverable and runnable from FFI._

- [x] 5. Add simulation FFI export
  - Files: core/src/ffi/mod.rs (modify)
  - Export `keyrx_simulate(keys_json, combo_mode)` returning simulation results
  - _Leverage: core/src/cli/commands/simulate.rs_
  - _Requirements: 6_
  - _Prompt: Implement the task for spec wire-every-remaining-feature, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust FFI engineer | Task: Add `keyrx_simulate` FFI export taking JSON keys array [{code, holdMs}] and comboMode bool, returning JSON {mappings: [{input, output, decision}], activeLayers, pending} | Restrictions: ≤80 lines; reuse simulate.rs | _Leverage: core/src/cli/commands/simulate.rs | _Requirements: 6 | Success: Key sequences processed with full state output._

- [x] 6. Add session management FFI exports
  - Files: core/src/ffi/mod.rs (modify)
  - Export `keyrx_list_sessions()`, `keyrx_analyze_session(path)`, `keyrx_replay_session(path, verify)`
  - _Leverage: core/src/cli/commands/analyze.rs, replay.rs_
  - _Requirements: 7, 10_
  - _Prompt: Implement the task for spec wire-every-remaining-feature, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust FFI engineer | Task: Add session FFI exports: list_sessions returns [{path, name, created, eventCount, durationMs}], analyze_session returns full analysis JSON, replay_session streams events or returns verification result | Restrictions: ≤150 lines; scan sessions/ directory | _Leverage: analyze.rs, replay.rs | _Requirements: 7, 10 | Success: Sessions listable, analyzable, replayable from FFI._

- [x] 7. Add benchmark FFI export
  - Files: core/src/ffi/mod.rs (modify)
  - Export `keyrx_run_benchmark(iterations)` returning results JSON
  - _Leverage: core/src/cli/commands/bench.rs_
  - _Requirements: 8_
  - _Prompt: Implement the task for spec wire-every-remaining-feature, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust FFI engineer | Task: Add `keyrx_run_benchmark` FFI export taking iteration count, returning JSON {minNs, maxNs, meanNs, p99Ns, hasWarning} | Restrictions: ≤60 lines; reuse bench.rs | _Leverage: core/src/cli/commands/bench.rs | _Requirements: 8 | Success: Benchmark results match CLI output format._

- [x] 8. Add diagnostics FFI export
  - Files: core/src/ffi/mod.rs (modify)
  - Export `keyrx_run_doctor()` returning diagnostics JSON
  - _Leverage: core/src/cli/commands/doctor.rs_
  - _Requirements: 9_
  - _Prompt: Implement the task for spec wire-every-remaining-feature, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust FFI engineer | Task: Add `keyrx_run_doctor` FFI export returning JSON {checks: [{name, status, details, remediation}], passed, failed, warned} | Restrictions: ≤80 lines; platform-specific checks | _Leverage: core/src/cli/commands/doctor.rs | _Requirements: 9 | Success: All diagnostic checks run and report correctly._

- [x] 9. Add discovery FFI exports
  - Files: core/src/ffi/mod.rs (modify)
  - Export `keyrx_start_discovery(device_id)` and progress callback
  - _Leverage: core/src/cli/commands/discover.rs_
  - _Requirements: 11_
  - _Prompt: Implement the task for spec wire-every-remaining-feature, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust FFI engineer | Task: Add discovery FFI: `keyrx_start_discovery` taking device ID, `keyrx_on_discovery_progress` for callback registration. Return JSON progress updates and final result | Restrictions: ≤100 lines; handle cancellation | _Leverage: core/src/cli/commands/discover.rs | _Requirements: 11 | Success: Discovery wizard controllable from Flutter._

- [x] 10. Add recording control FFI exports
  - Files: core/src/ffi/mod.rs (modify)
  - Export `keyrx_start_recording(path)` and `keyrx_stop_recording()`
  - _Leverage: existing session recording_
  - _Requirements: 2_
  - _Prompt: Implement the task for spec wire-every-remaining-feature, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust FFI engineer | Task: Add recording FFI: `keyrx_start_recording` taking output path, `keyrx_stop_recording` returning session path. Integrate with engine run loop | Restrictions: ≤50 lines; thread-safe state | _Leverage: session recording module | _Requirements: 2 | Success: Recording starts/stops cleanly, produces valid .krx files._

## FFI Layer (Dart)

- [x] 11. Add FFI bindings for device management
  - Files: ui/lib/ffi/bindings.dart (modify), ui/lib/ffi/bridge.dart (modify)
  - Add Dart signatures and bridge methods for device functions
  - _Leverage: existing binding patterns_
  - _Requirements: 1_
  - _Prompt: Implement the task for spec wire-every-remaining-feature, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Flutter FFI engineer | Task: Add to bindings.dart: KeyrxListDevices, KeyrxSelectDevice typedefs. Add to bridge.dart: listDevices(), selectDevice(path) methods with JSON parsing | Restrictions: ≤60 lines; follow existing patterns | _Leverage: existing bindings.dart | _Requirements: 1 | Success: Dart can call device FFI functions._

- [x] 12. Add FFI bindings for validation, tests, simulation
  - Files: ui/lib/ffi/bindings.dart (modify), ui/lib/ffi/bridge.dart (modify)
  - Add Dart signatures and bridge methods
  - _Leverage: existing binding patterns_
  - _Requirements: 3, 5, 6_
  - _Prompt: Implement the task for spec wire-every-remaining-feature, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Flutter FFI engineer | Task: Add bindings for: checkScript, discoverTests, runTests, simulate. Add bridge methods with proper JSON parsing and error handling | Restrictions: ≤100 lines; return typed Dart objects | _Leverage: existing bindings.dart | _Requirements: 3, 5, 6 | Success: All new FFI functions callable from Dart._

- [x] 13. Add FFI bindings for sessions, benchmark, doctor, discovery
  - Files: ui/lib/ffi/bindings.dart (modify), ui/lib/ffi/bridge.dart (modify)
  - Add remaining Dart signatures and bridge methods
  - _Leverage: existing binding patterns_
  - _Requirements: 7, 8, 9, 10, 11_
  - _Prompt: Implement the task for spec wire-every-remaining-feature, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Flutter FFI engineer | Task: Add bindings for: listSessions, analyzeSession, replaySession, runBenchmark, runDoctor, startDiscovery, startRecording, stopRecording. Bridge methods with JSON parsing | Restrictions: ≤150 lines; consistent error handling | _Leverage: existing bindings.dart | _Requirements: 7, 8, 9, 10, 11 | Success: Full FFI coverage for all CLI commands._

## Services Layer

- [x] 14. Create DeviceService
  - Files: ui/lib/services/device_service.dart (new)
  - Implement device listing, selection, profile checking
  - _Leverage: ui/lib/services/engine_service.dart patterns_
  - _Requirements: 1_
  - _Prompt: Implement the task for spec wire-every-remaining-feature, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Flutter service engineer | Task: Create DeviceService with: listDevices() → List<KeyboardDevice>, selectDevice(path), hasProfile(deviceId) → bool. Use KeyrxBridge | Restrictions: ≤80 lines; follow EngineService patterns | _Leverage: engine_service.dart | _Requirements: 1 | Success: Device operations abstracted behind clean service interface._

- [x] 15. Create TestService
  - Files: ui/lib/services/test_service.dart (new)
  - Implement test discovery and execution
  - _Leverage: ui/lib/services/engine_service.dart patterns_
  - _Requirements: 5_
  - _Prompt: Implement the task for spec wire-every-remaining-feature, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Flutter service engineer | Task: Create TestService with: discoverTests(scriptPath) → List<TestCase>, runTests(scriptPath, filter) → TestResults. Parse JSON from bridge | Restrictions: ≤100 lines; typed models | _Leverage: engine_service.dart | _Requirements: 5 | Success: Test execution abstracted with progress streaming._

- [x] 16. Create SimulationService
  - Files: ui/lib/services/simulation_service.dart (new)
  - Implement key sequence simulation
  - _Leverage: ui/lib/services/engine_service.dart patterns_
  - _Requirements: 6_
  - _Prompt: Implement the task for spec wire-every-remaining-feature, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Flutter service engineer | Task: Create SimulationService with: simulate(keys, comboMode) → SimulationResult. Models for KeyInput, KeyMapping, SimulationResult | Restrictions: ≤80 lines; clean API | _Leverage: engine_service.dart | _Requirements: 6 | Success: Simulation callable with typed inputs/outputs._

- [x] 17. Create SessionService
  - Files: ui/lib/services/session_service.dart (new)
  - Implement session listing, analysis, replay
  - _Leverage: ui/lib/services/engine_service.dart patterns_
  - _Requirements: 7, 10_
  - _Prompt: Implement the task for spec wire-every-remaining-feature, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Flutter service engineer | Task: Create SessionService with: listSessions() → List<SessionInfo>, analyze(path) → SessionAnalysis, replay(path, speed, verify) → Stream<ReplayEvent> | Restrictions: ≤120 lines; stream for replay | _Leverage: engine_service.dart | _Requirements: 7, 10 | Success: Full session management through service._

- [x] 18. Create BenchmarkService and DoctorService
  - Files: ui/lib/services/benchmark_service.dart (new), ui/lib/services/doctor_service.dart (new)
  - Implement benchmarking and diagnostics
  - _Leverage: ui/lib/services/engine_service.dart patterns_
  - _Requirements: 8, 9_
  - _Prompt: Implement the task for spec wire-every-remaining-feature, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Flutter service engineer | Task: Create BenchmarkService with runBenchmark(iterations) → BenchmarkResults. Create DoctorService with runDiagnostics() → DiagnosticReport | Restrictions: ≤100 lines total; typed models | _Leverage: engine_service.dart | _Requirements: 8, 9 | Success: Benchmark and diagnostics abstracted cleanly._

## User Interface Pages

- [x] 19. Create DevicesPage
  - Files: ui/lib/pages/devices_page.dart (new)
  - Device list with selection and refresh
  - _Leverage: ui/lib/pages/debugger_page.dart layout patterns_
  - _Requirements: 1_
  - _Prompt: Implement the task for spec wire-every-remaining-feature, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Flutter UI engineer | Task: Create DevicesPage showing device list with ListTile for each (name, path, profile badge). Tap to select, pull-to-refresh, empty state with troubleshooting | Restrictions: ≤150 lines; Material 3 design | _Leverage: debugger_page.dart | _Requirements: 1 | Success: Users can see and select their keyboard device._

- [x] 20. Create RunControlsPage
  - Files: ui/lib/pages/run_controls_page.dart (new)
  - Central engine control panel
  - _Leverage: ui/lib/pages/training_screen.dart patterns_
  - _Requirements: 2_
  - _Prompt: Implement the task for spec wire-every-remaining-feature, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Flutter UI engineer | Task: Create RunControlsPage with large Start/Stop FAB, device dropdown, script selector, recording toggle, status indicators (running, device, script, recording) | Restrictions: ≤200 lines; prominent controls | _Leverage: training_screen.dart | _Requirements: 2 | Success: One-tap engine start with clear status._

- [x] 21. Add script validation to EditorPage
  - Files: ui/lib/pages/editor_page.dart (modify)
  - Add real-time validation and error display
  - _Leverage: existing editor_page.dart_
  - _Requirements: 3_
  - _Prompt: Implement the task for spec wire-every-remaining-feature, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Flutter UI engineer | Task: Modify EditorPage to call checkScript on script changes (debounced 500ms). Show error banner when invalid, highlight line on tap. Block Load if invalid | Restrictions: ≤80 lines added; non-intrusive UX | _Leverage: existing editor_page.dart | _Requirements: 3 | Success: Syntax errors visible before loading script._

- [ ] 22. Update main navigation for User screens
  - Files: ui/lib/main.dart (modify)
  - Add Devices and Run Controls to NavigationRail
  - _Leverage: existing main.dart navigation_
  - _Requirements: 1, 2_
  - _Prompt: Implement the task for spec wire-every-remaining-feature, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Flutter UI engineer | Task: Add Devices (icon: keyboard) and Run Controls (icon: play_circle) to NavigationRail after Editor. Update selectedIndex handling | Restrictions: ≤30 lines changed; maintain existing order | _Leverage: main.dart | _Requirements: 1, 2 | Success: 4-tab user navigation working._

## Developer Tools Navigation

- [ ] 23. Create DeveloperDrawer widget
  - Files: ui/lib/widgets/developer_drawer.dart (new)
  - Navigation drawer for developer tools
  - _Leverage: ui/lib/main.dart NavigationRail patterns_
  - _Requirements: 4_
  - _Prompt: Implement the task for spec wire-every-remaining-feature, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Flutter UI engineer | Task: Create DeveloperDrawer with NavigationDrawer containing destinations: Debugger, Console, Test Runner, Simulator, Analyzer, Benchmark, Doctor, Replay, Discovery. Include header and close button | Restrictions: ≤100 lines; consistent styling | _Leverage: main.dart navigation | _Requirements: 4 | Success: Drawer shows all 9 developer tools._

- [ ] 24. Add developer mode to AppState
  - Files: ui/lib/state/app_state.dart (modify)
  - Add developer mode flag with persistence
  - _Leverage: existing app_state.dart_
  - _Requirements: 4_
  - _Prompt: Implement the task for spec wire-every-remaining-feature, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Flutter state engineer | Task: Add isDeveloperMode bool to AppState with SharedPreferences persistence. Add toggleDeveloperMode() method | Restrictions: ≤30 lines added; persist across restarts | _Leverage: app_state.dart | _Requirements: 4 | Success: Developer mode persists and controls drawer visibility._

- [ ] 25. Integrate DeveloperDrawer into main app
  - Files: ui/lib/main.dart (modify)
  - Add developer tools button and drawer integration
  - _Leverage: existing main.dart_
  - _Requirements: 4_
  - _Prompt: Implement the task for spec wire-every-remaining-feature, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Flutter UI engineer | Task: Add developer tools IconButton to AppBar. When tapped, show DeveloperDrawer. Handle navigation to developer pages. Show developer page content when selected | Restrictions: ≤60 lines added; clean integration | _Leverage: main.dart | _Requirements: 4 | Success: Developer tools accessible from app bar button._

## Developer Tool Pages

- [ ] 26. Create TestRunnerPage
  - Files: ui/lib/pages/developer/test_runner_page.dart (new)
  - Test discovery, execution, and results display
  - _Leverage: ui/lib/pages/debugger_page.dart patterns_
  - _Requirements: 5_
  - _Prompt: Implement the task for spec wire-every-remaining-feature, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Flutter UI engineer | Task: Create TestRunnerPage with: test list showing name/status, Run All button, filter TextField, individual test tap-to-run, expandable error details, watch mode toggle | Restrictions: ≤250 lines; real-time status updates | _Leverage: debugger_page.dart | _Requirements: 5 | Success: Full test runner UI with live results._

- [ ] 27. Create SimulatorPage
  - Files: ui/lib/pages/developer/simulator_page.dart (new)
  - Key sequence simulation with virtual keyboard
  - _Leverage: ui/lib/widgets/keyboard.dart_
  - _Requirements: 6_
  - _Prompt: Implement the task for spec wire-every-remaining-feature, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Flutter UI engineer | Task: Create SimulatorPage with: virtual keyboard for key selection, key sequence chips with hold duration editor, combo mode toggle, Simulate button, results showing input→output mappings, layer/pending state | Restrictions: ≤300 lines; interactive keyboard | _Leverage: keyboard.dart | _Requirements: 6 | Success: Users can build and test key sequences visually._

- [ ] 28. Create AnalyzerPage
  - Files: ui/lib/pages/developer/analyzer_page.dart (new)
  - Session analysis with statistics and timeline
  - _Leverage: ui/lib/pages/debugger_page.dart patterns_
  - _Requirements: 7_
  - _Prompt: Implement the task for spec wire-every-remaining-feature, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Flutter UI engineer | Task: Create AnalyzerPage with: session file picker, statistics cards (events, duration, latency), decision breakdown pie chart, timeline view with event details on tap | Restrictions: ≤250 lines; use fl_chart for visualization | _Leverage: debugger_page.dart, trade_off_chart.dart | _Requirements: 7 | Success: Sessions analyzable with visual timeline._

- [ ] 29. Create BenchmarkPage
  - Files: ui/lib/pages/developer/benchmark_page.dart (new)
  - Latency benchmark configuration and results
  - _Leverage: ui/lib/pages/trade_off_page.dart patterns_
  - _Requirements: 8_
  - _Prompt: Implement the task for spec wire-every-remaining-feature, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Flutter UI engineer | Task: Create BenchmarkPage with: iterations slider (1K-100K), Run Benchmark button with progress, results cards (min/max/mean/p99), warning banner if >1ms, optional history chart | Restrictions: ≤180 lines; clear results display | _Leverage: trade_off_page.dart | _Requirements: 8 | Success: Benchmarks runnable with clear latency reporting._

- [ ] 30. Create DoctorPage
  - Files: ui/lib/pages/developer/doctor_page.dart (new)
  - System diagnostics with remediation
  - _Leverage: ui/lib/pages/ patterns_
  - _Requirements: 9_
  - _Prompt: Implement the task for spec wire-every-remaining-feature, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Flutter UI engineer | Task: Create DoctorPage with: auto-run on open, check list with pass/fail/warn icons, expandable details, remediation steps for failures, Re-run button, summary counts | Restrictions: ≤180 lines; helpful remediation | _Leverage: existing page patterns | _Requirements: 9 | Success: Users can diagnose setup issues from UI._

- [ ] 31. Create ReplayPage
  - Files: ui/lib/pages/developer/replay_page.dart (new)
  - Session replay with playback controls
  - _Leverage: ui/lib/pages/debugger_page.dart patterns_
  - _Requirements: 10_
  - _Prompt: Implement the task for spec wire-every-remaining-feature, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Flutter UI engineer | Task: Create ReplayPage with: session list, metadata display, Play/Pause/Stop controls, speed slider (0x/1x/2x), verify mode toggle, event visualization during playback, mismatch highlighting | Restrictions: ≤250 lines; real-time playback | _Leverage: debugger_page.dart | _Requirements: 10 | Success: Sessions replayable with verification._

- [ ] 32. Create DiscoveryPage
  - Files: ui/lib/pages/developer/discovery_page.dart (new)
  - Device profile discovery wizard
  - _Leverage: ui/lib/pages/ patterns_
  - _Requirements: 11_
  - _Prompt: Implement the task for spec wire-every-remaining-feature, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Flutter UI engineer | Task: Create DiscoveryPage with: Start Discovery button, step-by-step prompts, key press feedback, progress indicator, confirmation screen showing detected layout, Save/Cancel buttons | Restrictions: ≤250 lines; wizard flow | _Leverage: existing page patterns | _Requirements: 11 | Success: Device profiles creatable through guided UI._

## Integration & Testing

- [ ] 33. Create developer pages directory structure
  - Files: ui/lib/pages/developer/ (new directory)
  - Ensure proper directory structure for developer pages
  - _Leverage: existing ui/lib/pages/ structure_
  - _Requirements: 4_
  - _Prompt: Implement the task for spec wire-every-remaining-feature, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Flutter engineer | Task: Create ui/lib/pages/developer/ directory. Add barrel file (developer.dart) exporting all developer pages | Restrictions: ≤20 lines for barrel file | _Leverage: existing structure | _Requirements: 4 | Success: Developer pages properly organized._

- [ ] 34. Add data models for new features
  - Files: ui/lib/models/ (new directory with model files)
  - Create typed models for all new data structures
  - _Leverage: existing model patterns_
  - _Requirements: All_
  - _Prompt: Implement the task for spec wire-every-remaining-feature, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Flutter data engineer | Task: Create models: KeyboardDevice, TestCase, TestResult, SimulationResult, SessionInfo, SessionAnalysis, BenchmarkResult, DiagnosticCheck. Use freezed or manual immutable classes | Restrictions: ≤200 lines total; JSON serialization | _Leverage: existing patterns | _Requirements: All | Success: All data types properly modeled._

- [ ] 35. Write widget tests for new pages
  - Files: ui/test/pages/ (new test files)
  - Test key UI interactions for each new page
  - _Leverage: existing ui/test/ patterns_
  - _Requirements: All_
  - _Prompt: Implement the task for spec wire-every-remaining-feature, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Flutter test engineer | Task: Create widget tests for: DevicesPage (list, select), RunControlsPage (start/stop), TestRunnerPage (run, filter), SimulatorPage (add keys, simulate). Mock services | Restrictions: ≤300 lines total; mock FFI | _Leverage: existing tests | _Requirements: All | Success: Core UI flows have test coverage._

- [ ] 36. Write integration tests for FFI round-trip
  - Files: ui/integration_test/ffi_test.dart (new)
  - Test Dart→Rust→Dart data flow
  - _Leverage: Flutter integration test patterns_
  - _Requirements: All_
  - _Prompt: Implement the task for spec wire-every-remaining-feature, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Flutter integration test engineer | Task: Create integration tests verifying FFI round-trip for: listDevices, checkScript, simulate, runBenchmark. Test on real library | Restrictions: ≤150 lines; skip if library unavailable | _Leverage: Flutter integration_test | _Requirements: All | Success: FFI calls work end-to-end._

- [ ] 37. Update README with new UI features
  - Files: README.md (modify)
  - Document new User and Developer tool screens
  - _Leverage: existing README structure_
  - _Requirements: All_
  - _Prompt: Implement the task for spec wire-every-remaining-feature, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Technical writer | Task: Add "## Flutter UI" section to README explaining: User interface (4 screens), Developer Tools (how to access, 8 tools), feature overview | Restrictions: ≤60 lines added; include screenshots placeholder | _Leverage: README.md | _Requirements: All | Success: New contributors understand UI capabilities._
