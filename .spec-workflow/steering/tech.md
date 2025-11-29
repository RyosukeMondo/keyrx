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

## Known Limitations

- **macOS Support**: Not currently targeted (different input architecture)
- **Wayland**: Limited support due to security model restrictions
- **32-bit Systems**: Not supported
- **Real-time Guarantees**: Best-effort low latency, not hard real-time
