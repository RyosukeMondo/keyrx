# Phase 1-3 Completion: Design Document

## Overview

This design document details the technical implementation of Phase 1-3 completion features: script testing, REPL console, session recording/replay, FFI fixes, event tracing, and Flutter UI enhancements. The implementation follows KeyRx's CLI-first philosophy, modular architecture, and CLAUDE.md code quality standards.

**Key Design Principles:**
- Reuse existing Rhai runtime infrastructure (no reimplementation)
- CLI commands separate from core logic (injectable dependencies)
- Event recording as optional middleware to engine loop
- Flutter UI as thin client over FFI bridge
- Strict adherence to file/function size limits (≤500/50 lines)

## Steering Document Alignment

### Technical Standards (tech.md)
- **CLI-First Development**: All features implemented as CLI commands first (§158-192)
- **Rhai Scripting**: Leverage existing RhaiRuntime for all script evaluation (§193-240)
- **FFI Bridge**: Extend existing keyrx_* FFI exports for new functionality (§241-292)
- **Error Handling**: Structured JSON logging with timestamp, level, service, event (§449-508)
- **Observability**: OpenTelemetry integration for tracing (§529-610)

### Project Structure (structure.md)
- **core/src/**: Rust engine and CLI commands
  - `cli/commands/`: New `test.rs`, `repl.rs`, `replay.rs`, `record.rs`, `analyze.rs`
  - `scripting/`: Extend `runtime.rs` → split into `runtime.rs`, `bindings.rs`, `builtins.rs`, `test_harness.rs`
  - `engine/`: Add `event_recording.rs` module
  - `ffi/`: Complete `exports.rs` script loading
- **ui/lib/**: Flutter application
  - `pages/`: Refactor `editor.dart` → `editor_page.dart` + `editor_widgets.dart`
  - New pages: `training_screen.dart`, `trade_off_visualizer.dart`

## Code Reuse Analysis

### Existing Components to Leverage

1. **RhaiRuntime** (`core/src/scripting/runtime.rs`)
   - **How reused**: REPL, script testing, and FFI load_script all use the same runtime instance via shared Arc<Mutex<>>
   - **Extension needed**: Export test primitives (simulate_tap, assert_output) as Rhai functions

2. **Engine State & Decision** (`core/src/engine/`)
   - **How reused**: Event recording intercepts process_event calls; state snapshots reuse existing state accessors
   - **Extension needed**: Add `to_snapshot_json()` method to EngineState

3. **Input Source Trait** (`core/src/traits/input_source.rs`)
   - **How reused**: Session replay implements InputSource trait to inject recorded events
   - **Extension needed**: None; replay_session will be a mock InputSource

4. **FFI Exports** (`core/src/ffi/exports.rs`)
   - **How reused**: Complete keyrx_load_script; add keyrx_eval_rhai for REPL/Flutter
   - **Extension needed**: Add new FFI functions for recording/replay status

5. **Flutter Bridge** (`ui/lib/ffi/bridge.dart`, `bindings.dart`)
   - **How reused**: Extend BridgeState to include new fields from state snapshots
   - **Extension needed**: Add bridge methods for training, trade-off data

6. **Debugger Page** (`ui/lib/pages/debugger.dart`)
   - **How reused**: Replace static demo data with live state stream
   - **Extension needed**: Subscribe to keyrx_on_state callback

### Integration Points

- **Engine Event Loop** → Event Recording Middleware
- **Rhai Runtime** → Test Harness (inject test functions)
- **FFI Boundary** → REPL/Script Loading (string marshaling)
- **Flutter UI** → State Snapshots (via keyrx_on_state callback stream)
- **OpenTelemetry** → Engine Decision Points (emit spans on tap-hold, combo, layer changes)

## Architecture

### Phase 1: Iron Core - CLI-First Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                    RhaiRuntime (Arc<Mutex>)                      │
│  - Shared instance across test, repl, and runtime commands      │
└───────────┬──────────────────┬──────────────────┬────────────────┘
            │                  │                  │
    ┌───────v──────┐   ┌──────v──────┐   ┌──────v─────┐
    │ keyrx test   │   │ keyrx repl   │   │ keyrx run  │
    │ --script     │   │              │   │ --record   │
    └──────┬───────┘   └──────┬───────┘   └──────┬─────┘
           │                  │                  │
    ┌──────v──────────────────v──────────────────v──────┐
    │      Test Harness API                             │
    │  - simulate_tap(key)                              │
    │  - simulate_hold(key, duration_ms)                │
    │  - assert_output(key)                             │
    │  - assert_mapping(from, to)                       │
    └─────────────────────────────────────────────────────┘
           │                  │
    ┌──────v──────────┐  ┌────v─────────────┐
    │ Engine           │  │ Event Recording  │
    │ process_event()  │  │ write_session.krx│
    └──────┬───────────┘  └────┬─────────────┘
           │                   │
    ┌──────v───────────────────v──────┐
    │ Output Generation                │
    │ (keyrx_inject_key, etc.)         │
    └──────────────────────────────────┘
```

### Phase 2: Event Tracing Integration

```
┌────────────────────────────────────────────┐
│ engine::process_event()                     │
└────────────┬───────────────────────────────┘
             │
             v
    ┌────────────────────────┐
    │ OpenTelemetry Span     │
    │ - event_id             │
    │ - input_key            │
    │ - timestamp            │
    └────────────┬───────────┘
                 │
    ┌────────────v──────────────────┐
    │ Decision Making                │
    │ (tap-hold, combo, layer)       │
    │ emit decision span             │
    └────────────┬───────────────────┘
                 │
    ┌────────────v──────────────────┐
    │ Output + Span Export           │
    │ send to OpenTelemetry backend  │
    └────────────────────────────────┘
```

### Phase 3: Flutter UI Architecture

```
┌────────────────────────────────────┐
│ Flutter App Shell (main.dart)       │
└────────────┬───────────────────────┘
             │
    ┌────────v──────────────┐
    │ Page Router            │
    ├───────────────────────┤
    │ debugger_page.dart    │
    │ training_screen.dart  │
    │ trade_off_visual.dart │
    │ editor_page.dart      │
    │ console_page.dart     │
    └────────┬──────────────┘
             │
    ┌────────v──────────────────────┐
    │ FFI Bridge (bridge.dart)       │
    │ - keyrx_on_state stream        │
    │ - keyrx_eval_rhai              │
    │ - keyrx_load_script            │
    │ - keyrx_list_keys              │
    └────────┬───────────────────────┘
             │
    ┌────────v──────────────────────┐
    │ Rust FFI Layer (exports.rs)    │
    │ - keyrx_on_state_callback      │
    │ - keyrx_eval (eval_rhai)       │
    │ - keyrx_load_script (FIXED)    │
    └────────────────────────────────┘
```

## Components and Interfaces

### 1. Script Test Harness (`core/src/scripting/test_harness.rs`)

**Purpose:** Provide test functions (simulate_*, assert_*) to Rhai scripts; collect test results

**Test Discovery Convention:** Since Rhai doesn't support attributes like `#[test]`, tests are discovered by function name prefix `test_` (e.g., `fn test_capslock_tap()`). This follows Python/JavaScript testing conventions.

**Interfaces:**
```rust
pub fn register_test_functions(runtime: &mut RhaiRuntime) -> Result<()>
pub fn discover_tests(ast: &rhai::AST) -> Vec<String>  // Find fn test_*
pub fn run_test_function(runtime: &RhaiRuntime, fn_name: &str) -> TestResult
pub struct TestResult {
    pub name: String,
    pub passed: bool,
    pub error: Option<String>,
    pub duration_µs: u64,
    pub line_number: Option<usize>,
}
```

**Dependencies:** RhaiRuntime, Engine (for simulate operations)

**Reuses:** Existing event injection, state getters

### 2. REPL Command (`core/src/cli/commands/repl.rs`)

**Purpose:** Interactive command-line interface for script evaluation and state inspection

**Interfaces:**
```rust
pub struct ReplCommand;
impl ReplCommand {
    pub fn execute() -> Result<()>
}

// REPL supports:
// > load_script(path)
// > simulate(key_sequence)
// > state()
// > layers
// > eval("rhai_code")
// > help
// > exit
```

**Dependencies:** RhaiRuntime (shared), Engine, StandardInput, **rustyline crate** (add to Cargo.toml)

**New Cargo.toml dependency:**
```toml
[dependencies]
rustyline = "14.0"  # For readline with history and completion
```

**Reuses:** Existing script loading, event simulation

### 3. Session Recording Middleware (`core/src/engine/event_recording.rs`)

**Purpose:** Intercept events and outputs; serialize to .krx file

**Interfaces:**
```rust
pub struct EventRecorder {
    file: File,
    session_start: Instant,
    event_count: u64,
}

pub struct EventRecord {
    pub seq: u64,
    pub timestamp_µs: u64,
    pub input: InputEvent,
    pub output: Option<OutputAction>,
    pub decision_type: String,  // "pass", "remap", "tap", "hold", "combo"
    pub latency_µs: u64,
}

impl EventRecorder {
    pub fn record_event(&mut self, record: EventRecord) -> Result<()>
    pub fn finish(&mut self, final_state: EngineSnapshot) -> Result<()>
}
```

**Dependencies:** File I/O, Engine (state snapshots)

**Reuses:** Existing event types, state serialization

### 4. Session Replay (`core/src/cli/commands/replay.rs`)

**Purpose:** Read .krx file and inject events deterministically

**Interfaces:**
```rust
pub struct ReplaySession {
    events: Vec<EventRecord>,
    timing_offset_µs: u64,
}

impl InputSource for ReplaySession {
    fn start(&mut self) -> Result<()>
    fn poll(&mut self) -> Result<Option<InputEvent>>
    fn stop(&mut self) -> Result<()>
}

pub fn verify_replay(original: &str, replayed: &str) -> Result<ReplayDiff>
```

**Dependencies:** File I/O, InputSource trait

**Reuses:** Existing input injection mechanisms

### 5. Timing Diagram Generator (`core/src/cli/commands/analyze.rs`)

**Purpose:** Parse .krx file and generate ASCII timing diagrams

**Interfaces:**
```rust
pub struct TimingAnalyzer;
impl TimingAnalyzer {
    pub fn generate_diagram(session_path: &str) -> Result<String>
}

// Output format:
// Event │ Input │ Decision │ Output │ Latency (µs)
// ──────┼───────┼──────────┼────────┼──────────────
// 1     │ KeyA  │ Remap    │ KeyB   │ 142
// 2     │ KeyC  │ Hold...  │ (wait) │ 2500 (pending)
// 3     │ KeyC↑ │ Hold→Tap │ KeyC   │ 2501
```

**Dependencies:** File I/O, EventRecord structures

**Reuses:** Existing serialization

### 6. FFI Script Loading Complete (`core/src/ffi/exports.rs`)

**Purpose:** Load script file into active runtime via FFI

**Interfaces:**
```rust
#[no_mangle]
pub unsafe extern "C" fn keyrx_load_script(path: *const c_char) -> i32
// Returns: 0 (success), -1 (invalid path), -2 (invalid UTF8), -3 (syntax error)
// Side effect: script loaded into runtime, hooks registered
```

**Dependencies:** RhaiRuntime (shared), CStr marshaling

**Reuses:** Existing script loading logic from keyrx check command

### 7. OpenTelemetry Tracing (`core/src/engine/tracing.rs`)

**Purpose:** Emit spans for input→decision→output flow

**Interfaces:**
```rust
pub struct EngineTracer {
    tracer: opentelemetry::trace::Tracer,
}

impl EngineTracer {
    pub fn span_input_received(&self, event: &InputEvent) -> Span
    pub fn span_decision_made(&self, decision: &str, latency_µs: u64) -> Span
    pub fn span_output_generated(&self, action: &OutputAction) -> Span
}
```

**Dependencies:** opentelemetry crate, optional at runtime

**Reuses:** None (new feature)

### 8. Flutter State Subscription (`ui/lib/ffi/state_stream.dart`)

**Purpose:** Receive state snapshots via FFI callback; emit Dart Stream

**Interfaces:**
```dart
class EngineStateStream {
  Stream<EngineSnapshot> get snapshot stream
  void _onStateCallback(String jsonPayload) // invoked from FFI
}

class EngineSnapshot {
  List<String> activeLayers
  List<String> heldKeys
  Map<String, bool> modifiers
  Map<String, dynamic> pending
  int latencyMicroseconds
  Map<String, dynamic> timing
}
```

**Dependencies:** bridge.dart, FFI bindings

**Reuses:** Existing keyrx_on_state callback

### 9. Debugger Page Enhancement (`ui/lib/pages/debugger.dart`)

**Purpose:** Display real-time engine state with live updates

**Separation Plan:** Split into:
- `debugger_page.dart` (440 lines) - State management, layout
- `debugger_widgets.dart` (300 lines) - Component widgets (timeline, layer panel, modifier display)

**Key Widgets:**
- `DebuggerTimelineWidget` - Animated timeline of recent events
- `LayerPanelWidget` - Active layer stack display
- `ModifierDisplayWidget` - Current modifier state
- `LatencyMeterWidget` - Real-time latency visualization

**Dependencies:** state_stream, bridge

**Reuses:** Existing timeline components

### 10. Training Screen (`ui/lib/pages/training_screen.dart`)

**Purpose:** Interactive guided lessons for KeyRx configuration

**Lesson Structure:**
```dart
class TrainingLesson {
  String title
  String description
  String objective
  List<TrainingStep> steps
}

class TrainingStep {
  String instruction
  Function validator  // returns bool
  String hint
  String feedbackSuccess
  String feedbackFailure
}
```

**Dependencies:** engine service, key registry, state stream

**Reuses:** Existing key validation logic

### 11. Trade-off Visualizer (`ui/lib/pages/trade_off_visualizer.dart`)

**Purpose:** Interactive chart showing timing threshold vs. miss rate trade-offs

**Chart Data:**
```dart
class TradeOffPoint {
  double tapHoldTimeoutMs
  double missRatePercent
  String label  // "Gaming", "Typing", etc.
}

class UserTypingProfile {
  double avgInterKeyDelayMs
  double stdDevMs
  int samplesCollected
}

// Miss rate calculation (statistical model)
double calculateMissRate(double threshold, double userMean, double userStdDev) {
  // CDF of normal distribution - probability of inter-key delay < threshold
  // Lower threshold = higher miss rate for fast typists
  return normalCdf(threshold, userMean, userStdDev);
}
```

**New pubspec.yaml dependency:**
```yaml
dependencies:
  fl_chart: ^0.68.0  # For interactive line charts
```

**Dependencies:** fl_chart package, bridge

**Reuses:** None (new)

## Data Models

### EventRecord (Rust)
```rust
#[derive(Serialize, Deserialize)]
pub struct EventRecord {
    pub seq: u64,
    pub timestamp_µs: u64,
    pub input: InputEvent,
    pub output: Option<OutputAction>,
    pub decision_type: String,
    pub active_layers: Vec<String>,
    pub modifiers_state: serde_json::Value,
    pub latency_µs: u64,
}

#[derive(Serialize, Deserialize)]
pub struct SessionFile {
    pub version: String,
    pub created_at: SystemTime,
    pub script_used: String,
    pub timing_config: TimingConfiguration,
    pub initial_state: EngineSnapshot,
    pub events: Vec<EventRecord>,
}
```

### EngineSnapshot (Dart/Rust)
```rust
#[derive(Serialize, Deserialize, Clone)]
pub struct EngineSnapshot {
    pub active_layers: Vec<String>,
    pub held_keys: Vec<String>,
    pub modifiers: Map<String, bool>,
    pub pending_decisions: Vec<PendingDecision>,
    pub event_summary: String,
    pub latency_µs: u64,
    pub timing: TimingConfiguration,
}
```

### TestResult
```rust
pub struct TestResult {
    pub name: String,
    pub passed: bool,
    pub message: String,
    pub duration_µs: u64,
    pub line_number: Option<usize>,
}
```

## Error Handling

### Phase 1 Error Scenarios

1. **Script Syntax Error in Test**
   - **Handling:** Catch RhaiError, extract location (file:line:col)
   - **User Impact:** `keyrx test` exits code 1, prints error with context
   - **Example:** `config.rhai:45: Unknown variable 'simulate_hold'`

2. **Test Function Not Found**
   - **Handling:** Check runtime registry before executing
   - **User Impact:** `keyrx test` exits code 1, suggests available test functions
   - **Example:** `No test function named 'test_remap_a_to_b' found`

3. **REPL Runtime Not Initialized**
   - **Handling:** Lazy-initialize on first REPL command
   - **User Impact:** User sees `keyrx> ` prompt; first command initializes runtime
   - **Example:** `(Loading runtime...)`

4. **Session File Corrupted**
   - **Handling:** Validate JSON schema before replay
   - **User Impact:** `keyrx replay` exits code 1, reports corruption location
   - **Example:** `session.krx: Invalid event at record 42 (missing 'timestamp_µs')`

### Phase 2 Error Scenarios

5. **OpenTelemetry Backend Unavailable**
   - **Handling:** Skip span emission; continue running
   - **User Impact:** No visible failure; tracing simply unavailable
   - **Example:** (silent; tracing is optional)

### Phase 3 Error Scenarios

6. **State Snapshot Stream Disconnected**
   - **Handling:** Debugger detects empty stream; show error banner
   - **User Impact:** Debugger shows "Disconnected. Reconnect?" button
   - **Example:** Reconnect trigger: user presses button or refreshes page

7. **Key Registry Fetch Failed**
   - **Handling:** Training/editor use fallback set; warn user
   - **User Impact:** Editor shows "Some keys unavailable" warning
   - **Example:** Still allows mapping but without validation

## Testing Strategy

### Unit Testing

**Phase 1:**
- Test harness: `#[test] fn test_simulate_tap_generates_event()` → verify event injection
- REPL: `#[test] fn repl_parse_command_load_script()` → verify command parsing
- Recording: `#[test] fn event_record_serializes_to_json()` → verify serialization
- Replay: `#[test] fn replay_injects_events_in_order()` → verify event sequence
- Test discovery: `#[test] fn discover_test_functions_from_script()` → verify #[test] parsing

**Coverage Target:** 85%

### Integration Testing

**Phase 1:**
- End-to-end test: `keyrx test tests/example_test.rhai` → full test cycle
- Recording + Replay: Record 100 events, replay, verify outputs match
- REPL + Script Load: Load script in REPL, verify mappings active

**Phase 2:**
- Tracing + Engine: Generate 1000 events, verify spans emitted
- OpenTelemetry export: Verify spans exported to mock backend

**Phase 3:**
- Flutter + State Stream: Debugger connected, verify state updates within 50ms
- Training completion: Complete all lessons, verify state transitions

**Coverage Target:** 80%

### End-to-End Testing

**User Scenarios:**
1. Developer writes test in script, runs `keyrx test`, sees results
2. User records session with `keyrx run --record`, replays with `keyrx replay`
3. User opens Flutter debugger, sees live state updating as they press keys
4. User completes training lessons, applies knowledge to editor
5. User adjusts timing thresholds, sees trade-off visualization update

**Coverage Target:** 75%

## Implementation Sequence

**Phase 1 (Weeks 1-2):**
1. Refactor scripting/runtime.rs → split into 4 modules (500 line limit)
2. Implement test harness (register_test_functions, test discovery)
3. Implement keyrx test command
4. Implement keyrx repl command
5. Implement event recording middleware
6. Implement keyrx replay command
7. Implement keyrx analyze command
8. Fix keyrx_load_script FFI function
9. Unit + integration tests (85% coverage)

**Phase 2 (Week 3):**
1. Add OpenTelemetry dependency
2. Implement EngineTracer
3. Integrate tracing into engine::process_event
4. Implement trace export
5. Tests (80% coverage)

**Phase 3 (Weeks 4-5):**
1. Refactor editor.dart → split into 2 files (500 line limit)
2. Enhance debugger page with state stream subscription
3. Implement training_screen.dart with lesson framework
4. Implement trade_off_visualizer.dart with chart
5. Add console error styling
6. Flutter tests (75% coverage)

## Phase 4: Hardening & Compliance

### Architecture Overview

```
┌─────────────────────────────────────────────────────────────────────────┐
│                    EMERGENCY EXIT LAYER (Driver Level)                   │
│  ┌─────────────────────────────────────────────────────────────────┐    │
│  │ Windows: WH_KEYBOARD_LL Hook                                     │    │
│  │   → check_emergency_exit(vk, mods) BEFORE any processing         │    │
│  │   → if Ctrl+Alt+Shift+Escape: set BYPASS_MODE, return pass-thru  │    │
│  └─────────────────────────────────────────────────────────────────┘    │
│  ┌─────────────────────────────────────────────────────────────────┐    │
│  │ Linux: evdev Handler                                             │    │
│  │   → check_emergency_exit(key, mods) BEFORE any processing        │    │
│  │   → if Ctrl+Alt+Shift+Escape: set BYPASS_MODE, ungrab device     │    │
│  └─────────────────────────────────────────────────────────────────┘    │
└─────────────────────────────────────────────────────────────────────────┘
                                    │
                                    ▼
┌─────────────────────────────────────────────────────────────────────────┐
│                         ENGINE PROCESSING                                │
│   (Only runs if BYPASS_MODE == false)                                   │
│   → Rhai script execution                                               │
│   → Tap-hold, combo, layer processing                                   │
└─────────────────────────────────────────────────────────────────────────┘
```

### Component 12: Emergency Exit Handler (`core/src/drivers/emergency_exit.rs`)

**Purpose:** Provide hardcoded emergency exit detection at driver level, guaranteed to work regardless of script/engine state.

**Interfaces:**
```rust
/// Global bypass mode flag - atomic for thread safety
static BYPASS_MODE: AtomicBool = AtomicBool::new(false);

/// Check if emergency exit combo is pressed
/// Returns true if Ctrl+Alt+Shift+Escape detected
pub fn check_emergency_exit(key: KeyCode, modifiers: &ModifierState) -> bool {
    key == KeyCode::Escape
        && modifiers.ctrl
        && modifiers.alt
        && modifiers.shift
}

/// Activate bypass mode - all keys pass through unmodified
pub fn activate_bypass_mode() {
    BYPASS_MODE.store(true, Ordering::SeqCst);
    tracing::warn!("Emergency exit activated - remapping disabled");
    // Notify UI via callback
    notify_bypass_activated();
}

/// Deactivate bypass mode - resume remapping
pub fn deactivate_bypass_mode() {
    BYPASS_MODE.store(false, Ordering::SeqCst);
    tracing::info!("Remapping re-enabled");
}

/// Check if bypass mode is active
pub fn is_bypass_active() -> bool {
    BYPASS_MODE.load(Ordering::SeqCst)
}
```

**Integration Points:**
- Windows driver: Check in `keyboard_proc` before `process_event`
- Linux driver: Check in `evdev_read_loop` before `process_event`
- FFI: Export `keyrx_is_bypass_active()` and `keyrx_set_bypass()` for UI

### Component 13: Visual Keyboard Layout (`ui/lib/widgets/visual_keyboard.dart`)

**Purpose:** Render interactive keyboard layout for drag-and-drop mapping.

**Interfaces:**
```dart
class VisualKeyboard extends StatefulWidget {
  final KeyboardLayout layout; // ANSI, ISO, JIS, etc.
  final Map<String, String> mappings; // current remappings
  final Function(String from, String to) onMappingCreated;
  final Function(String key) onMappingDeleted;
}

class KeyboardLayout {
  final String name;
  final List<KeyRow> rows;
  final Map<String, KeyDefinition> keys;
}

class KeyDefinition {
  final String code;
  final String label;
  final double width; // in key units (1.0 = standard)
  final Offset position;
}
```

### Component 14: Rhai Code Generator (`ui/lib/services/rhai_generator.dart`)

**Purpose:** Generate Rhai script from visual mappings.

**Interfaces:**
```dart
class RhaiGenerator {
  /// Generate Rhai script from visual configuration
  String generateScript(VisualConfig config) {
    final buffer = StringBuffer();
    buffer.writeln('// Generated by KeyRx Visual Editor');
    buffer.writeln('// ${DateTime.now().toIso8601String()}');
    buffer.writeln();

    for (final mapping in config.remaps) {
      buffer.writeln('remap("${mapping.from}", "${mapping.to}");');
    }

    for (final tapHold in config.tapHolds) {
      buffer.writeln('tap_hold("${tapHold.key}", "${tapHold.tap}", "${tapHold.hold}");');
    }

    return buffer.toString();
  }

  /// Parse simple Rhai script back to visual config (best-effort)
  VisualConfig? parseScript(String script);
}

class VisualConfig {
  List<RemapConfig> remaps;
  List<TapHoldConfig> tapHolds;
  List<LayerConfig> layers;
  bool hasAdvancedFeatures; // true if script has unsupported constructs
}
```

### Component 15: CI Benchmark Workflow (`.github/workflows/benchmark.yml`)

**Purpose:** Run latency benchmarks on PRs and detect regressions.

**Workflow:**
```yaml
name: Performance Benchmarks

on:
  pull_request:
    branches: [main]

jobs:
  benchmark:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Run Criterion benchmarks
        run: cargo bench --bench latency -- --save-baseline pr-${{ github.event.number }}

      - name: Compare with main baseline
        run: |
          cargo bench --bench latency -- --baseline main --load-baseline pr-${{ github.event.number }}

      - name: Check regression threshold
        run: |
          # Parse criterion output, fail if any benchmark >100µs regression
          python scripts/check_bench_regression.py --threshold 100
```

### Data Models for Phase 4

**BypassState (Rust):**
```rust
#[derive(Debug, Clone, Serialize)]
pub struct BypassState {
    pub active: bool,
    pub activated_at: Option<SystemTime>,
    pub reason: String,
}
```

**VisualConfig (Dart):**
```dart
class VisualConfig {
  final List<RemapConfig> remaps;
  final List<TapHoldConfig> tapHolds;
  final List<ComboConfig> combos;
  final List<LayerConfig> layers;
  final bool hasAdvancedFeatures;
  final String? parseWarning;
}

class RemapConfig {
  final String fromKey;
  final String toKey;
  final String? layer; // null = base layer
}
```

### Refactoring Strategy for Oversized Files

**test_harness.rs (802 lines) → Split:**
- `test_harness.rs` (~400 lines): TestHarness struct, register_test_functions
- `test_primitives.rs` (~400 lines): simulate_*, assert_* implementations

**test_runner.rs (741 lines) → Split:**
- `test_runner.rs` (~350 lines): TestRunner struct, run_tests
- `test_discovery.rs` (~350 lines): discover_tests, AST parsing

**event_recording.rs (732 lines) → Split:**
- `event_recording.rs` (~350 lines): EventRecord, SessionFile, serialization
- `event_recorder.rs` (~350 lines): EventRecorder middleware implementation

**advanced.rs (706 lines) → Split:**
- `advanced.rs` (~350 lines): Core process_event logic
- `decision_engine.rs` (~350 lines): Tap-hold, combo decision algorithms

**debugger.dart (969 lines) → Split:**
- `debugger_page.dart` (~400 lines): Main page, state management
- `debugger_widgets.dart` (~300 lines): Timeline, layer panel widgets
- `debugger_meters.dart` (~270 lines): Latency meter, pending visualization

**trade_off_visualizer.dart (949 lines) → Split:**
- `trade_off_page.dart` (~400 lines): Main page, state management
- `trade_off_chart.dart` (~300 lines): fl_chart configuration, rendering
- `typing_simulator.dart` (~250 lines): Typing speed measurement

### Testing Strategy for Phase 4

**Emergency Exit Tests:**
```rust
#[test]
fn emergency_exit_detected_when_all_mods_pressed() {
    let mods = ModifierState { ctrl: true, alt: true, shift: true, ..Default::default() };
    assert!(check_emergency_exit(KeyCode::Escape, &mods));
}

#[test]
fn emergency_exit_not_triggered_without_all_mods() {
    let mods = ModifierState { ctrl: true, alt: true, shift: false, ..Default::default() };
    assert!(!check_emergency_exit(KeyCode::Escape, &mods));
}

#[test]
fn bypass_mode_passes_all_keys_through() {
    activate_bypass_mode();
    // Verify no remapping occurs
    deactivate_bypass_mode();
}
```

**Visual Editor Tests (Flutter):**
```dart
testWidgets('drag key A to B creates remap', (tester) async {
  await tester.pumpWidget(VisualKeyboard(...));
  await tester.drag(find.byKey(Key('KeyA')), Offset(100, 0));
  await tester.pumpAndSettle();
  expect(createdMappings, contains(RemapConfig('KeyA', 'KeyB')));
});

test('RhaiGenerator creates valid script', () {
  final config = VisualConfig(remaps: [RemapConfig('A', 'B')]);
  final script = RhaiGenerator().generateScript(config);
  expect(script, contains('remap("A", "B")'));
});
```

### Implementation Sequence for Phase 4

**Emergency Exit (Tasks 31-33):**
1. Implement emergency_exit.rs with bypass mode logic
2. Integrate into Windows driver (keyboard_proc)
3. Integrate into Linux driver (evdev handler)
4. Add FFI exports for UI
5. Add system tray indicator (Flutter)
6. Unit + integration tests

**CI Benchmark (Task 34):**
1. Create GitHub Actions workflow
2. Implement regression check script
3. Set up baseline storage
4. Add PR comment reporting

**Code Refactoring (Tasks 35-40):**
1. Refactor Rust files one at a time
2. Refactor Flutter files
3. Verify all tests pass after each refactor

**Visual Editor (Tasks 41-43):**
1. Implement VisualKeyboard widget with layout rendering
2. Add drag-and-drop interaction
3. Implement RhaiGenerator
4. Add "Eject to Code" feature
5. Integration tests

