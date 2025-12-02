# Phase 1-3 Completion: Core Features & GUI Requirements

## Introduction

This specification addresses the critical gaps between KeyRx's documented architecture (product.md, tech.md) and current implementation. It covers three implementation phases to complete the Rust backend (Phase 1), driver integration (Phase 2), and Flutter UI (Phase 3). The focus is on developer tooling, observability, and user-facing visualization of the key remapping engine's state and capabilities.

## Alignment with Product Vision

This spec directly enables the product vision outlined in `product.md`:
- **Phase 1**: Completes "Iron Core - Headless" with script testing and REPL for developer iteration
- **Phase 2**: Implements "Nervous System - Drivers" observability through event tracing
- **Phase 3**: Delivers "Flutter GUI" features for real-time debugging and configuration trade-offs

These features are critical for the "Developer-Friendly Remapping" differentiation stated in product.md §10-40.

## Requirements

### Phase 1: Iron Core Completion

#### Requirement 1.1: Script Testing Framework

**User Story:** As a Rhai script developer, I want to write deterministic tests for my key remapping scripts, so that I can validate behavior without manual testing.

**Description:** Implement a test framework that allows users to write tests in Rhai scripts using a simple test harness with assertions.

#### Acceptance Criteria

1. WHEN a user creates a `.rhai` file with functions prefixed `test_`, THEN `keyrx test --script <file>` SHALL discover and execute all test functions (Note: Rhai doesn't support attributes, so `test_` prefix convention is used)
2. WHEN a test function calls `simulate_tap("KeyA")`, THEN the system SHALL inject a keydown+keyup event pair with current timestamp
3. WHEN a test function calls `assert_output("KeyB")`, THEN the assertion SHALL pass if the last output event is KeyB, else fail with clear message
4. WHEN a test calls `assert_mapping("A", "B")`, THEN it SHALL verify the configured mapping by checking the registry
5. WHEN a test suite has failures, THEN `keyrx test` SHALL exit with code per product.md: 0=all pass, 1=execution error, 2=assertion fail, 3=timeout
6. WHEN a test calls `assert_duration(100..200)`, THEN it SHALL verify the last action completed within ±100-200µs (assuming deterministic timing)
7. IF a user's script has a syntax error in a test function, THEN `keyrx test` SHALL report the error location with line number
8. WHEN user runs `keyrx test --filter "capslock*"`, THEN only tests matching the pattern SHALL execute
9. WHEN user runs `keyrx test --watch`, THEN tests SHALL re-run automatically when script file changes
10. WHEN user runs `keyrx test --json`, THEN output SHALL be machine-parseable JSON for AI agent consumption

**Test File Example:**
```rhai
// Test functions use test_ prefix (Rhai doesn't support #[test] attributes)
fn test_capslock_tap_produces_escape() {
    simulate_tap("CapsLock");
    assert_output("Escape");
}

fn test_ctrl_a_selects_all() {
    simulate_key_with_mods("KeyA", ["Control"]);
    assert_output("SelectAll");
}
```

#### Requirement 1.2: REPL Console (Interactive Script Evaluation)

**User Story:** As a developer, I want an interactive REPL where I can execute Rhai scripts and inspect engine state in real-time, so that I can iterate quickly during development.

**Description:** Implement a command-line REPL that allows users to load the active runtime and execute Rhai code interactively, with support for inspecting state.

#### Acceptance Criteria

1. WHEN a user runs `keyrx repl`, THEN a prompt `keyrx> ` SHALL appear and accept Rhai input
2. WHEN a user types `load_script("path/to/script.rhai")`, THEN the script SHALL be loaded into the runtime and any hooks/remaps registered
3. WHEN a user types `simulate("a")` in REPL, THEN the input SHALL be processed by the engine and output shown
4. WHEN a user types `state()`, THEN the REPL SHALL display current layers, modifiers, held keys as JSON
5. WHEN a user types `eval "remap('a', 'b')"`, THEN the Rhai code SHALL execute in the current runtime context
6. WHEN a user types `layers`, THEN the REPL SHALL list active layer stack with names and priorities
7. WHEN a user types `help`, THEN REPL SHALL show available REPL commands
8. WHEN a user types `exit`, THEN REPL SHALL cleanly shut down and return to shell
9. IF a user inputs invalid Rhai syntax, THEN REPL SHALL show error with line/column and keep prompt open

#### Requirement 1.3: Session Recording & Replay Infrastructure

**User Story:** As a developer debugging a key remapping issue, I want to record a session of key events, then replay them deterministically to reproduce the issue, so that I can verify fixes without manual retesting.

**Description:** Implement event recording to `.krx` files and a replay mechanism to deterministically re-execute sequences with full state snapshots.

#### Acceptance Criteria

1. WHEN `keyrx run --record session.krx` is executed, THEN all input events SHALL be serialized to `.krx` file with timestamps and input data
2. WHEN the session completes, THEN the `.krx` file SHALL contain: event sequence, initial engine state, timing configuration, script used
3. WHEN `keyrx replay session.krx` is executed, THEN events SHALL be injected in order with accurate inter-event timing
4. WHEN a replay completes, THEN outputs SHALL match the original recorded outputs byte-for-byte (deterministic)
5. WHEN a user modifies a script and replays an old session, THEN the modified behavior SHALL be visible in the new outputs
6. WHEN `keyrx analyze session.krx --diagram`, THEN ASCII timing diagram SHALL show input→decision→output latency for each event
7. WHEN a `.krx` file is corrupted, THEN `keyrx replay` SHALL report the corruption and refuse to load
8. IF a user records a session with 1000+ events, THEN `.krx` file size SHALL not exceed 500KB (compressed or optimized format)

#### Requirement 1.4: FFI Script Loading Fix

**User Story:** As a Flutter UI user, I want to load custom Rhai scripts through the UI without reimplementation, so that the engine-UI integration works end-to-end.

**Description:** Complete the incomplete `keyrx_load_script` FFI function to actually load and execute scripts from file paths.

#### Acceptance Criteria

1. WHEN Flutter calls `keyrx_load_script("path/to/config.rhai")`, THEN the script SHALL be loaded into the active runtime
2. WHEN the script loads successfully, THEN the function SHALL return 0 (success) and any hooks/remaps are registered
3. WHEN the script path is invalid, THEN the function SHALL return -1 with error logged
4. WHEN the script has syntax errors, THEN the function SHALL return -3 and log the error location
5. WHEN the script loads after engine initialization, THEN previously registered mappings SHALL still be active

### Phase 2: Nervous System - Driver Integration

#### Requirement 2.1: Event Tracing & Observability

**User Story:** As a user experiencing issues with key remapping on complex setups, I want the engine to emit structured observability data (traces, spans) that integration developers can analyze, so that debugging difficult driver interactions is possible.

**Description:** Integrate OpenTelemetry-compatible event tracing to emit detailed spans for input processing, decision-making, and output generation.

#### Acceptance Criteria

1. WHEN an input event is processed, THEN a trace span SHALL record: event type, timestamp, duration from ingress to output
2. WHEN a tap-hold decision completes, THEN a span SHALL be emitted with: input key, timeout decision, output action, total latency_µs
3. WHEN combo matching occurs, THEN a span SHALL record: combo name, matched keys, decision timestamp, whether combo fired
4. WHEN a decision is made, THEN the span SHALL include: active layers, modifiers state, script execution time (if applicable)
5. WHEN `keyrx run --trace <trace-file>` is used, THEN all spans SHALL be exported to trace-file in OpenTelemetry format
6. IF OpenTelemetry is unavailable, THEN the engine SHALL continue operating normally (observability is optional)
7. WHEN a user examines a trace in compatible tool (Jaeger, etc.), THEN they SHALL see full causal chain: input→decision→output

### Phase 3: Flutter GUI Completion

#### Requirement 3.1: Enhanced Debugger with Real-Time State

**User Story:** As a user testing my key remapping configuration, I want to see the engine's real-time state (layers, modifiers, timing) updating as I press keys, so that I can understand what the engine is doing.

**Description:** Integrate the state snapshot stream (from live-ui-ffi-hardening spec) into the debugger page to display live engine state with thresholds and latency visualization.

#### Acceptance Criteria

1. WHEN the debugger page opens, THEN it SHALL fetch initial state snapshot via FFI and display: active layers, held keys, modifiers
2. WHEN a key is pressed, THEN the debugger SHALL update within 50ms to show: key in "held" state, any triggered layers/modifiers
3. WHEN a key is released, THEN the debugger SHALL show: output event generated, timing latency_µs, decision type (pass/remap/tap/hold/combo)
4. WHEN a tap-hold is pending, THEN the debugger SHALL display countdown timer and highlight affected keys
5. WHEN tap-hold timeout occurs, THEN the debugger SHALL show transition: "pending" → "hold" with timestamp
6. WHEN a combo is matched, THEN the debugger SHALL highlight all combo keys and show matched combo name
7. WHEN the state updates, THEN animation SHALL fade in new values (no jarring jumps)
8. IF state snapshot stream disconnects, THEN debugger SHALL show error and offer "Reconnect" button

#### Requirement 3.2: Training Screen Implementation

**User Story:** As a user new to KeyRx, I want a guided tutorial/training mode that teaches me how to configure key remappings, so that I can learn the system interactively.

**Description:** Create an interactive training screen with guided exercises and immediate feedback for learning KeyRx configuration.

#### Acceptance Criteria

1. WHEN training screen opens, THEN it SHALL present step-by-step lessons on: remapping, layers, modifiers, tap-hold, combos
2. WHEN a user completes a "remap A→B" exercise, THEN they SHALL be prompted to press 'A' and see 'B' output
3. WHEN the user presses 'A' and 'B' is correctly output, THEN the exercise SHALL mark as complete with checkmark
4. IF the user presses 'A' and gets wrong output, THEN feedback SHALL explain why (e.g., "Layer not active")
5. WHEN all lessons are complete, THEN training screen SHALL offer "Start with blank canvas" to apply knowledge
6. WHEN a user clicks "Show console output during exercise", THEN the console panel SHALL appear inline showing eval results
7. IF a user is stuck, THEN a "Hint" button SHALL provide guidance without spoiling the answer
8. WHEN training completes, THEN a "Certificate" modal SHALL appear (visual achievement, can be dismissed)

#### Requirement 3.3: Configuration Trade-off Visualizer

**User Story:** As a user configuring timing-sensitive features (tap-hold thresholds), I want to see how my timing choices affect latency and reliability, so that I can make informed trade-offs.

**Description:** Create a visualization showing the relationship between timing configuration choices (thresholds, timeouts) and resulting latency/responsiveness characteristics.

#### Acceptance Criteria

1. WHEN user opens "Settings → Timing & Trade-offs" page, THEN an interactive chart SHALL show: X-axis (tap timeout ms), Y-axis (miss rate %)
2. WHEN user adjusts tap_hold_timeout slider, THEN the chart SHALL update to show predicted miss rate based on statistical typing speed models (derived from published research on inter-key intervals)
3. WHEN user hovers over a point on the curve, THEN a tooltip SHALL show: "500ms timeout → 8% miss rate on fast typists, 2% on average"
4. WHEN a user loads a timing preset (e.g., "Gaming", "Typing"), THEN the chart SHALL highlight the recommended region
5. WHEN tap_hold_timeout threshold is < 100ms, THEN UI SHALL warn: "Very tight timing - may cause accidental holds"
6. WHEN user clicks "Simulate my typing speed", THEN a test sequence SHALL run and measure their actual typing patterns
7. WHEN simulation completes, THEN the chart SHALL overlay: user's typing speed distribution and recommended thresholds
8. WHEN user exports config, THEN a comment SHALL be added: "Optimized for X typing speed, see trade-offs at line Y"

**Data Source for Predictions:** Miss rate curves are derived from published research on typing speed distributions (average inter-key interval ~150-200ms for skilled typists). The model assumes: P(miss) ≈ CDF(threshold, mean=user_mean, stddev=user_stddev) where threshold is tap_hold_timeout.

#### Requirement 3.4: Console Error Styling Enhancement

**User Story:** As a user, I want the console to visually distinguish between successful commands and errors, so that I can quickly identify issues.

**Description:** Update console page to style responses based on ok:/error: prefix from eval results.

#### Acceptance Criteria

1. WHEN a console command returns `ok:<value>`, THEN the response SHALL appear in green/success styling
2. WHEN a console command returns `error:<message>`, THEN the response SHALL appear in red/error styling with icon
3. WHEN an error is `error:Engine not initialized`, THEN console SHALL offer "Initialize Engine" quick action
4. WHEN error contains file path, THEN the path SHALL be a link to open in editor (if applicable)
5. WHEN user types `remap('a', 'b')` and it succeeds, THEN console SHALL show: `✓ Remapping created: a → b`
6. WHEN user selects an error message, THEN pressing Ctrl+C SHALL copy just the message without prefix

## Non-Functional Requirements

### Code Architecture and Modularity

- **CLAUDE.md Compliance**:
  - All files shall remain ≤ 500 lines (excluding comments/blank lines)
  - All functions shall remain ≤ 50 lines
  - Panic macros in test code shall use proper assertions (assert!, assert_eq!, expect)
  - `core/src/scripting/runtime.rs` (currently 1654 lines) must be refactored into smaller modules
  - `ui/lib/pages/editor.dart` (currently 844 lines) must be refactored into editor_widgets.dart + editor_page.dart
  - **Panic Cleanup**: The 10 identified panic!() calls in test code (discovery/session.rs, engine/decision/pending.rs, engine/advanced.rs, cli/commands/discover.rs, mocks/mock_state.rs, tests/integration/) must be replaced with proper assertions

- **Single Responsibility**: Script testing, REPL, and replay shall be separate CLI commands with reusable internal APIs
- **Dependency Injection**: All external dependencies (FFI, file I/O, tracing) shall be injectable for testing
- **Error Handling**: Structured JSON logging with timestamp, level, service, event, context per CLAUDE.md

### Session File Format

- **Compression**: .krx files may use gzip compression for sessions >100 events to meet ≤500KB target for 1000+ events
- **Schema Versioning**: SessionFile includes version field for forward/backward compatibility

### Performance

- **REPL Responsiveness**: REPL commands shall execute within 100ms (excluding I/O)
- **State Snapshot Latency**: State updates shall reach debugger within 50ms
- **Session Recording Overhead**: Recording shall add <5% CPU overhead
- **Trace Export**: Exporting 10K events to OpenTelemetry format shall complete in <2 seconds

### Reliability

- **Deterministic Replay**: Replayed sessions shall produce byte-identical outputs to original recording
- **Error Recovery**: Session loading errors shall not crash the engine; graceful degradation required
- **Script Syntax Error Handling**: All syntax errors shall be reported with file:line:column location
- **FFI Safety**: All FFI functions shall prevent panics across boundary; return error codes instead

### Test Coverage

- **Phase 1**: Script testing framework and REPL code shall have ≥85% coverage
- **Phase 2**: Event tracing shall have ≥80% coverage (non-critical path)
- **Phase 3**: Flutter UI pages shall have ≥75% widget/integration test coverage
- **Integration**: End-to-end tests for session record→replay cycle

### Usability

- **Error Messages**: All errors shall explain the problem and suggest corrective action
- **REPL Help**: Built-in `help` command shall list all REPL functions with examples
- **Training**: Guided training shall take <15 minutes to complete all lessons
- **Visualization**: Charts/diagrams shall be readable at 1024x768 resolution and mobile-friendly

### Security

- **Session File Validation**: `.krx` files shall be validated before replay (prevent injection)
- **REPL Isolation**: REPL shall not expose internal Rust APIs or private engine state unintentionally
- **FFI Boundary**: All FFI string pointers shall be bounds-checked; prevent buffer overflows

## Phase 4: Hardening & Compliance

### Requirement 4.1: Emergency Exit (Critical Safety Feature)

**User Story:** As a user who may have misconfigured their key remapping, I want a guaranteed escape hatch (Ctrl+Alt+Shift+Escape) that instantly disables all remapping, so that I can always recover control of my keyboard.

**Description:** Implement a hardcoded emergency exit hotkey that bypasses all script processing and immediately disables the remapping engine. This is a critical safety feature mentioned in product.md but not yet implemented.

#### Acceptance Criteria

1. WHEN user presses Ctrl+Alt+Shift+Escape simultaneously, THEN all key remapping SHALL be instantly disabled
2. WHEN emergency exit is triggered, THEN all keys SHALL pass through unmodified to the OS
3. WHEN emergency exit is triggered, THEN the engine SHALL emit a clear notification (system tray, console log)
4. WHEN emergency exit is active, THEN a visual indicator (system tray icon change) SHALL show "remapping disabled" state
5. WHEN user wants to re-enable remapping, THEN they SHALL be able to via CLI (`keyrx enable`) or GUI toggle
6. WHEN the engine crashes or hangs, THEN the emergency exit SHALL still function (hardcoded at driver level, not script level)
7. IF the engine is in any state (tap-hold pending, combo in progress, etc.), THEN emergency exit SHALL immediately cancel and disable
8. WHEN emergency exit is triggered, THEN the system SHALL log the event with timestamp for debugging

**Implementation Note:** The emergency exit check MUST be implemented at the driver level (before any script processing) in both Windows hook and Linux evdev handlers to guarantee it always works regardless of script state.

### Requirement 4.2: Visual Editor Tier 1 (No-Code Experience)

**User Story:** As a non-technical user, I want to configure key remappings using a visual drag-and-drop interface without writing code, so that I can customize my keyboard without learning Rhai scripting.

**Description:** Implement a visual editor that generates Rhai scripts automatically from user interactions, fulfilling the "Tier 1: Simple visual mode" described in product.md's three-tier complexity model.

#### Acceptance Criteria

1. WHEN user opens the visual editor, THEN they SHALL see a graphical keyboard layout with draggable keys
2. WHEN user drags key A onto key B, THEN a remap(A, B) SHALL be created visually with connecting line/arrow
3. WHEN user creates mappings visually, THEN the underlying Rhai script SHALL be auto-generated in real-time
4. WHEN user clicks "Show Code" / "Eject to Code", THEN the generated Rhai script SHALL be displayed and editable
5. WHEN user modifies the Rhai code manually, THEN they SHALL be warned that visual sync may be lost
6. WHEN user saves visual configuration, THEN a `.rhai` file SHALL be created with the generated script
7. WHEN visual editor loads an existing simple script, THEN it SHALL reverse-parse and display visually (best-effort)
8. IF a script contains advanced features (conditionals, custom functions), THEN visual editor SHALL show "Code-only sections" indicator
9. WHEN user hovers over a visual mapping, THEN tooltip SHALL show the equivalent Rhai code

### Requirement 4.3: CI Performance Regression Detection

**User Story:** As a maintainer, I want the CI pipeline to automatically detect and fail on performance regressions, so that latency increases are caught before merge.

**Description:** Implement CI workflow that runs latency benchmarks and fails if performance degrades beyond threshold, enforcing the <1ms latency guarantee from product.md.

#### Acceptance Criteria

1. WHEN CI runs on a PR, THEN latency benchmarks SHALL execute automatically
2. WHEN any benchmark shows >100µs regression from baseline, THEN CI SHALL fail with detailed report
3. WHEN benchmarks pass, THEN results SHALL be stored as new baseline for future comparisons
4. WHEN CI fails due to regression, THEN report SHALL show: benchmark name, baseline (µs), current (µs), delta (%)
5. WHEN a legitimate performance change is expected, THEN maintainer SHALL be able to update baseline via PR comment
6. WHEN benchmark results are collected, THEN they SHALL be published to a metrics dashboard (optional)

### Requirement 4.4: Code Quality Compliance (File Size Refactoring)

**User Story:** As a maintainer following CLAUDE.md guidelines, I want all source files to comply with the ≤500 lines limit, so that the codebase remains maintainable and navigable.

**Description:** Refactor the 6 files currently exceeding the 500-line guideline into smaller, focused modules.

#### Files Requiring Refactoring

| File | Current Lines | Target |
|------|---------------|--------|
| `core/src/scripting/test_harness.rs` | 802 | Split into 2 modules |
| `core/src/scripting/test_runner.rs` | 741 | Split into 2 modules |
| `core/src/engine/event_recording.rs` | 732 | Split into 2 modules |
| `core/src/engine/advanced.rs` | 706 | Split into 2 modules |
| `ui/lib/pages/debugger.dart` | 969 | Split into 2-3 files |
| `ui/lib/pages/trade_off_visualizer.dart` | 949 | Split into 2-3 files |

#### Acceptance Criteria

1. WHEN refactoring is complete, THEN all source files SHALL be ≤500 lines (excluding comments/blank lines)
2. WHEN modules are split, THEN public API SHALL remain unchanged (no breaking changes)
3. WHEN modules are split, THEN each new module SHALL have clear single responsibility
4. WHEN refactoring is complete, THEN all existing tests SHALL pass without modification
5. WHEN new modules are created, THEN re-exports SHALL maintain existing import paths where possible

