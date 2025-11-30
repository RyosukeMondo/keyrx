# Technology Stack

## Project Type
Desktop application with a hybrid architecture: native Rust core engine communicating with a Flutter GUI via FFI bridge. The core can also run headless as a daemon.

## Core Technologies

### Primary Languages
- **Rust (stable)**: Core engine, event loop, scripting runtime, OS drivers
- **Dart/Flutter**: Cross-platform GUI application
- **Rhai**: Embedded scripting language for user configuration

### Runtime/Build Tools
- **Tokio**: Async runtime for concurrent event handling
- **Cargo**: Rust package manager and build system
- **Flutter SDK**: UI framework with hot reload

### Key Dependencies/Libraries

#### Rust Core
- **tokio**: Async runtime for event loop and concurrent I/O
- **rhai**: Embedded scripting engine (sandboxed, Rust-native)
- **proptest**: Property-based testing and fuzzing
- **criterion**: Latency benchmarking

#### Platform Drivers
- **windows-rs**: Windows API bindings for WH_KEYBOARD_LL hooks
- **evdev/uinput**: Linux input device handling

#### Flutter UI
- **dart:ffi**: Foreign Function Interface to Rust
- **Skia/Impeller**: Hardware-accelerated rendering

### Application Architecture

**3-Layer Hybrid Architecture**:

```
┌─────────────────────────────────────────┐
│           UI Layer (Flutter)            │
│  Visual Editor │ Debugger │ REPL        │
└────────────────┬────────────────────────┘
                 │ FFI (C-ABI)
┌────────────────┴────────────────────────┐
│           Core Layer (Rust)             │
│  Tokio Event Loop │ Rhai │ State Machine│
└────────────────┬────────────────────────┘
                 │ Trait Abstraction
┌────────────────┴────────────────────────┐
│           OS Layer (Native Adapters)    │
│  Windows Hook │ Linux uinput/evdev      │
└─────────────────────────────────────────┘
```

**Key Patterns**:
- **Event Sourcing**: Input treated as immutable event stream
- **No Global State**: All instances are self-contained structs
- **Modular Drivers**: OS adapters implement generic `InputSource` trait
- **Dependency Injection**: All external dependencies injected for testability
- **CLI First**: All features exercisable via CLI before GUI implementation

### Data Storage
- **Primary storage**: Rhai script files (user configurations)
- **Runtime state**: In-memory layer state machine
- **Data formats**: Rhai scripts (text), JSON for state export/import

### External Integrations
- **Protocols**: OS-level keyboard hooks (WH_KEYBOARD_LL, evdev)
- **FFI**: Dart-Rust communication via C-compatible ABI

## Dependency Injection Architecture

All external dependencies are injected via traits, enabling testability and mock substitution:

```rust
// Trait definition (interface)
pub trait InputSource: Send + Sync {
    fn poll_events(&mut self) -> Vec<InputEvent>;
    fn send_output(&mut self, action: OutputAction) -> Result<()>;
}

pub trait ScriptRuntime: Send + Sync {
    fn execute(&mut self, script: &str) -> Result<ScriptResult>;
    fn call_hook(&mut self, hook: &str, args: &[Value]) -> Result<Value>;
}

pub trait StateStore: Send + Sync {
    fn get_layer(&self, name: &str) -> Option<&Layer>;
    fn set_active_modifiers(&mut self, mods: ModifierSet);
}

// Engine with injected dependencies
pub struct Engine<I: InputSource, S: ScriptRuntime, St: StateStore> {
    input: I,
    script: S,
    state: St,
}

// Production usage
let engine = Engine::new(
    WindowsInputSource::new()?,
    RhaiRuntime::new()?,
    InMemoryState::new(),
);

// Test usage with mocks
let engine = Engine::new(
    MockInputSource::new(),
    MockScriptRuntime::new(),
    MockState::new(),
);
```

## Advanced Remapping Engine Architecture

The engine implements a layered architecture for advanced keyboard customization. All timing parameters are configurable, enabling users to tune behavior for their typing style.

### Engine Layer Model

```
┌─────────────────────────────────────────────────────────────────┐
│ Layer 5: CONFIG PATTERNS (Rhai scripts - user presets)         │
│   Home Row Mods, Caps Word, Auto Shift, Application-Aware      │
├─────────────────────────────────────────────────────────────────┤
│ Layer 4: COMPOSED BEHAVIORS                                    │
│   ┌──────────────────────┐  ┌──────────────────────┐           │
│   │ Engine (perf critical)│  │ Script (flexibility) │           │
│   │ • Tap-Hold           │  │ • Tap-Dance          │           │
│   │ • One-Shot           │  │ • Leader Key         │           │
│   │ • Combos             │  │ • Layer modes        │           │
│   └──────────────────────┘  └──────────────────────┘           │
├─────────────────────────────────────────────────────────────────┤
│ Layer 3: ACTION PRIMITIVES (Engine - What to output)           │
│   emit_key, block, modifier_on/off, layer_push/pop, macro      │
├─────────────────────────────────────────────────────────────────┤
│ Layer 2: DECISION PRIMITIVES (Engine - When to act)            │
│   tap_or_hold, simultaneous, sequence, interrupt               │
├─────────────────────────────────────────────────────────────────┤
│ Layer 1: STATE MANAGEMENT (Engine - What we track)             │
│   key_states, timers, modifier_states, layer_stack             │
├─────────────────────────────────────────────────────────────────┤
│ Layer 0: EVENT METADATA (Driver - from REQ-9)                  │
│   timestamp_us, is_repeat, is_synthetic, scan_code, device_id  │
└─────────────────────────────────────────────────────────────────┘
```

### Configurable Timing Parameters

All timing decisions are parameterized, not hardcoded:

```rust
pub struct TimingConfig {
    /// Duration (ms) to distinguish tap from hold. Default: 200
    pub tap_timeout_ms: u32,

    /// Window (ms) for detecting simultaneous keypresses. Default: 50
    pub combo_timeout_ms: u32,

    /// Delay (ms) before considering a hold, to prevent misfires. Default: 0
    pub hold_delay_ms: u32,

    /// If true, emit tap immediately and correct if becomes hold. Default: false
    pub eager_tap: bool,

    /// If true, consider as hold if another key pressed during hold. Default: true
    pub permissive_hold: bool,

    /// If true, release tap even if interrupted (retro tap). Default: false
    pub retro_tap: bool,
}
```

### Virtual Modifiers

Up to 255 user-defined virtual modifiers exist only in the engine:

```rust
pub struct ModifierState {
    /// Bitmap for 255 custom modifiers (Mod1-Mod255)
    custom: [u64; 4],  // 256 bits

    /// Standard modifiers (Ctrl, Shift, Alt, Win)
    standard: StandardModifiers,
}

// In Rhai scripts:
// define_modifier("Mod_Thumb");
// on_key("Space", hold: activate_modifier("Mod_Thumb"));
// on_key("J", when: modifier_active("Mod_Thumb"), emit: "Left");
```

### Layer System

Layers are stacked with configurable priority:

```rust
pub struct LayerStack {
    /// Active layers in priority order (highest first)
    stack: Vec<LayerId>,

    /// Layer definitions with transparency support
    layers: HashMap<LayerId, Layer>,
}

pub struct Layer {
    pub name: String,
    pub mappings: HashMap<KeyCode, LayerAction>,
    pub transparent: bool,  // Fall through to layer below
}
```

### Decision State Machine

The engine tracks pending decisions for timing-based behaviors:

```rust
pub enum PendingDecision {
    TapOrHold {
        key: KeyCode,
        pressed_at: Instant,
        tap_action: Action,
        hold_action: Action,
    },
    Combo {
        keys: Vec<KeyCode>,
        started_at: Instant,
        combo_action: Action,
    },
    Sequence {
        keys_so_far: Vec<KeyCode>,
        started_at: Instant,
    },
}
```

### GUI Visualization Contract

The engine exposes state for GUI visualization:

```rust
pub struct EngineState {
    /// Current pending decisions (for timing visualization)
    pub pending: Vec<PendingDecision>,

    /// Active layers (for layer inspector)
    pub active_layers: Vec<LayerId>,

    /// Held modifiers (for modifier display)
    pub modifiers: ModifierState,

    /// Recent events with timing (for event log)
    pub event_history: VecDeque<TimedEvent>,

    /// Current config (for trade-off visualizer)
    pub timing_config: TimingConfig,
}
```

## CLI-First Development

Every feature must be CLI-exercisable before GUI implementation:

```bash
# Script validation and linting
keyrx check scripts/user_config.rhai

# Headless engine with debug output
keyrx run --debug --script scripts/user_config.rhai

# Event simulation (no real keyboard)
keyrx simulate --input "A,B,Ctrl+C" --expect "B,A,copy"

# State inspection
keyrx state --layers --modifiers --json

# Self-diagnostics
keyrx doctor --verbose

# Latency benchmarking
keyrx bench --iterations 10000

# REPL for interactive testing
keyrx repl
```

**Benefits**:
- **Rapid Trial**: Instant feedback without GUI overhead
- **Self-Check**: Automated validation in CI/CD pipelines
- **AI Agent Autonomy**: AI tools can test, validate, and iterate without human intervention
- **Debug Mode**: JSON output for programmatic parsing

## Development Environment

### Build & Development Tools
- **Build System**: Cargo (Rust), Flutter CLI
- **Package Management**: Cargo (crates.io), pub (pub.dev)
- **Development workflow**: Hot reload (Flutter), cargo watch (Rust)

### Code Quality Tools
- **Static Analysis**: clippy (Rust), dart analyze
- **Formatting**: rustfmt, dart format
- **Testing Framework**:
  - cargo test (unit tests)
  - proptest (property-based fuzzing)
  - criterion (benchmarks)
  - flutter_test (widget tests)
- **Documentation**: rustdoc, dartdoc

### Version Control & Collaboration
- **VCS**: Git
- **Branching Strategy**: Feature branches with PR reviews
- **Code Review Process**: PR-based with CI checks

## Deployment & Distribution
- **Target Platforms**: Windows 10+, Linux (X11/Wayland)
- **Distribution Method**: Binary releases, package managers (planned)
- **Installation Requirements**: No external runtime dependencies
- **Update Mechanism**: In-app update check (planned)

## Technical Requirements & Constraints

### Performance Requirements
- **Input latency**: < 1ms processing overhead (hard requirement)
- **Memory usage**: < 50MB idle
- **Startup time**: < 500ms to fully operational
- **Benchmark threshold**: CI fails if latency increases > 100 microseconds

### Compatibility Requirements
- **Platform Support**: Windows 10+, Linux (kernel 5.0+)
- **Architecture**: x86_64, ARM64 (planned)
- **Standards Compliance**: HID specification compliance

### Security & Compliance
- **Script Sandboxing**: Rhai scripts cannot access filesystem, network, or system calls
- **No Elevated Privileges**: Runs in user space where possible
- **Safe Script Sharing**: Community scripts are inherently safe due to sandbox

### Scalability & Reliability
- **Reliability Target**: Zero crashes under any input sequence
- **Fuzz Testing**: Must survive 100,000+ random key combinations
- **Deterministic Replay**: Event sourcing enables bug reproduction

## Technical Decisions & Rationale

### Decision Log

1. **Rust over C++**: Chosen for memory safety, fearless concurrency, and modern tooling. Eliminates entire classes of bugs (use-after-free, data races) common in input handling.

2. **Rhai over Lua/Python**: Rhai compiles into Rust binary (no external VM), provides sandboxed safety by default, and shares types with Rust reducing conversion overhead.

3. **Flutter over Electron/Tauri**:
   - Hot reload for rapid UI iteration
   - Skia/Impeller provides 60fps+ rendering
   - Direct FFI to Rust (no HTTP/JSON serialization overhead)

4. **Tokio Async Runtime**: Input handling is inherently async. Enables concurrent handling of keyboard, mouse, MIDI, and timers without complex threading.

5. **Event Sourcing Pattern**: Enables "Replay Debugging" - record sessions to reproduce flaky bugs deterministically.

6. **Trait-based OS Abstraction**: Core logic never imports OS headers directly. Drivers are plugins implementing `InputSource` trait, enabling mock testing.

## Observability & Debuggability Architecture

Industry-standard observability infrastructure serving both AI coding agents and human developers.

### Design Goals

| Audience | Primary Needs |
|----------|---------------|
| **AI Agents** | Machine-parseable output, deterministic replay, self-verification, rapid iteration |
| **Humans** | Visual timing diagrams, state flow animation, real-time overlay, historical playback |

### Build Mode Behavior

| Feature | Debug Build | Release Build |
|---------|-------------|---------------|
| Session Recording | All sessions (auto) | Opt-in (`--record`) |
| Debug Overlay | Available | Hidden |
| Trace Level | TRACE | INFO |
| Assertions | Enabled | Disabled |
| Performance Checks | Enabled | Disabled |

### Event Log Infrastructure

```rust
/// Complete event record for replay and analysis
#[derive(Serialize, Deserialize)]
pub struct EventRecord {
    /// Monotonic sequence number
    pub seq: u64,
    /// Microseconds since session start
    pub timestamp_us: u64,
    /// The input event with all metadata
    pub input: InputEvent,
    /// Processing span ID (for tracing correlation)
    pub span_id: String,
    /// Decision made (if any)
    pub decision: Option<Decision>,
    /// Output actions produced
    pub outputs: Vec<OutputAction>,
    /// State snapshot (optional, for keyframes)
    pub state_snapshot: Option<EngineState>,
}
```

**Storage Configuration:**
- Retention: Configurable by hours (default: 24h debug, 1h release)
- Rolling update: Oldest sessions pruned automatically
- Format: JSON for interchange, CBOR/MessagePack for high-frequency timing data

### Tracing & OpenTelemetry

```rust
// Span hierarchy for each event
#[tracing::instrument(skip(self), fields(key = %event.key, seq = %event.seq))]
fn process_event(&mut self, event: InputEvent) -> Vec<OutputAction> {
    let _decision_span = tracing::info_span!("decision").entered();
    // Decision logic...

    let _output_span = tracing::info_span!("output").entered();
    // Output generation...
}
```

**Integration:**
- `tracing` crate for structured spans
- `tracing-opentelemetry` for OTLP export
- Compatible with Jaeger, Grafana, Datadog

### For AI Agents: Self-Verification

```bash
# Structured JSON output for parsing
keyrx simulate --input "CapsLock:tap" --json

# Deterministic replay with assertions
keyrx replay session.krx --assert-output expected.json

# Watch mode for rapid iteration
keyrx watch --script config.rhai --on-change "keyrx test"

# Semantic exit codes
# 0=success, 1=error, 2=assertion fail, 3=timeout
```

**Invariant Checks (debug build):**
- State consistency after each event
- Modifier state matches key state
- Layer stack integrity
- No orphaned pending decisions

### For Humans: Visual Feedback (Flutter GUI)

**Timing Diagram:**
```
Time (ms)  0    50   100  150  200  250
Key A      ████████████████░░░░░░░░░░░  (held)
Decision   │ pending... │ → HOLD      │
Threshold  │──── tap ───│─── hold ────│
                     200ms
```

**State Flow Visualization:**
- Event → Decision → Output trace
- Layer stack with transparency
- Active modifiers (custom + standard)
- Animated state transitions

**Real-time Debug Overlay (developer only):**
- Floating always-on-top panel
- Key heatmap, pending queue, latency display

### Performance Metrics

| Metric | Description |
|--------|-------------|
| `keyrx.event.latency_us` | Per-event processing time |
| `keyrx.decision.pending_count` | Undecided tap-holds |
| `keyrx.layer.active_count` | Active layer count |
| `keyrx.event.throughput` | Events per second |

### CLI Commands

```bash
# Record session (debug: auto, release: explicit)
keyrx run --record session.krx --script config.rhai

# Replay for debugging
keyrx replay session.krx --step-through

# Export to OTLP endpoint
keyrx run --otlp-endpoint http://localhost:4317

# Generate timing diagram
keyrx analyze session.krx --timing-diagram --output diagram.svg

# State inspection
keyrx state --json --include-pending --include-history
```

## Known Limitations

- **macOS Support**: Not currently targeted (different input architecture)
- **Wayland**: Limited support due to security model restrictions
- **32-bit Systems**: Not supported
- **Real-time Guarantees**: Best-effort low latency, not hard real-time
