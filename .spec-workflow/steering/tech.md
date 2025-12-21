# Technology Stack

## Project Type

**System-level input remapping daemon with browser-based UI**

keyrx is a hybrid system combining:
- **Low-level system daemon**: OS-specific input interception and injection
- **Cross-platform core library**: Platform-agnostic remapping logic
- **CLI compiler**: Rhai script → static binary transformation
- **Web-based UI**: React + WASM for configuration and simulation

## Core Technologies

### Primary Language(s)
- **Language**: Rust (edition 2021, stable channel)
- **Compiler**: rustc 1.70+ with multiple compilation targets
  - `x86_64-unknown-linux-gnu` (Linux daemon)
  - `x86_64-pc-windows-msvc` (Windows daemon)
  - `wasm32-unknown-unknown` (browser simulation)
- **Language-specific tools**:
  - **Cargo**: Build system and package manager
  - **cargo-fuzz**: Fuzzing infrastructure (libFuzzer backend)
  - **wasm-pack**: WASM build and npm packaging

### Key Dependencies/Libraries

#### Core Runtime (keyrx_core - no_std)
- **rkyv** (0.7+): Zero-copy deserialization for configuration files
  - Deterministic binary serialization
  - Validation via bytecheck
- **boomphf** (0.6+): Minimal Perfect Hash Function (MPHF) generation
  - CHD algorithm for O(1) key lookup
- **fixedbitset** (0.4+): Compact bitset for 255 modifiers/locks state
- **arrayvec** (0.7+): Fixed-capacity vectors (no heap allocation)

#### Scripting & Compilation (keyrx_compiler)
- **rhai** (1.15+): Embedded scripting language for configuration DSL
  - Compile-time evaluation only (not runtime)
- **serde** (1.0+): Intermediate serialization before rkyv conversion

#### OS Integration & Web Server (keyrx_daemon)
**Linux**:
- **evdev** (0.12+): Input device handling via `/dev/input/event*`
- **uinput** (0.1+): Virtual device creation via `/dev/uinput`
- **nix** (0.26+): Safe wrappers for ioctl, epoll

**Windows**:
- **windows-sys** (0.48+): Raw Windows API bindings
  - `SetWindowsHookEx` (WH_KEYBOARD_LL)
  - `GetRawInputDeviceInfo` (device identification)
  - `SendInput` (event injection)

**Embedded Web Server** (optional feature, enabled by default):
- **axum** (0.7+): Lightweight async web framework
  - Serves static UI files (embedded at compile-time via `include_dir!`)
  - WebSocket endpoint for real-time UI communication
  - REST API for daemon control (status, config upload)
- **tower-http** (0.5+): HTTP middleware (CORS, static file serving)
- **tokio** (1.35+): Async runtime for web server + event processing

#### Concurrency & IPC
- **crossbeam-channel** (0.5+): Lock-free MPMC channels for event queues
- **memmap2** (0.7+): Memory-mapped files for zero-copy .krx loading
- **parking_lot** (0.12+): Faster synchronization primitives (when needed outside hot path)

#### Frontend (keyrx_ui)
- **React** (18+): UI framework
- **TypeScript** (5+): Type-safe JavaScript
- **wasm-bindgen** (0.2+): Rust ↔ JavaScript FFI
- **serde-wasm-bindgen** (0.5+): Serde serialization across WASM boundary

#### Testing & Verification
- **proptest** (1.2+): Property-based testing framework
- **criterion** (0.5+): Benchmarking with statistical rigor
- **cargo-fuzz**: Fuzz testing (integrated with libFuzzer)

### Application Architecture

**Four-Crate Architecture (KISS Principle)**:

```
┌─────────────────────────────────────────────┐
│  keyrx_compiler (CLI)                       │
│  - Rhai DSL parser                          │
│  - MPHF generation (boomphf)                │
│  - Outputs: .krx binary (rkyv)              │
└─────────────────┬───────────────────────────┘
                  │ (compile time only)
                  ▼
┌─────────────────────────────────────────────┐
│  keyrx_core (no_std library)                │
│  - Pure logic, no OS dependencies           │
│  - DFA state machine (Tap/Hold)             │
│  - MPHF-based O(1) lookup                   │
│  - Compilable to WASM                       │
└─────────────────┬───────────────────────────┘
                  │ (embedded by)
          ┌───────┴───────┐
          ▼               ▼
┌──────────────────────────┐  ┌─────────────────────┐
│ keyrx_daemon             │  │ keyrx_ui            │
│ - OS hooks (evdev/WinLL) │  │ - React + WASM      │
│ - .krx loader (mmap)     │  │ - Browser simulator │
│ - Embedded web server:   │  │ - Static files only │
│   * axum HTTP/WebSocket  │  │                     │
│   * Serves UI (embedded) │◄─┤ (compiled output)   │
│   * REST API             │  │                     │
└──────────────────────────┘  └─────────────────────┘
         │
         │ (User accesses)
         ▼
   http://localhost:9876
   (Daemon serves UI + API)
```

**Key Architectural Principles**:
- **no_std Core**: keyrx_core has zero OS dependencies, enabling WASM compilation
- **Compile-Time Code Generation**: Rhai scripts → static Rust structures (MPHF tables, DFA)
- **Single Source of Truth**: Both daemon and UI consume identical .krx binary
- **Lock-Free Hot Path**: Input processing uses lock-free ring buffers, no mutexes

### Crate Organization

**Four-crate workspace** (see `structure.md` for detailed folder structure):

- **`keyrx_core`**: Platform-agnostic remapping logic (no_std, WASM-compatible)
- **`keyrx_compiler`**: Rhai → .krx compiler (standalone CLI tool)
- **`keyrx_daemon`**: OS-specific daemon with embedded web server (evdev/Windows hooks + axum)
- **`keyrx_ui`**: React + WASM frontend (compiled to static assets, embedded in daemon)

### Multi-Device Architecture (QMK-Inspired)

**Inspired by [QMK Split Keyboard](https://docs.qmk.fm/features/split_keyboard)** | **Fills gap left by [Karabiner-Elements](https://github.com/pqrs-org/Karabiner-Elements/issues/2007)**

keyrx adopts a **global state model** similar to QMK's master/slave architecture, enabling cross-device modifier sharing that existing OS-level remappers cannot achieve.

#### Global State Model (Single Daemon)

- **Single `ExtendedState`**: 255 modifiers + 255 locks shared across ALL connected devices
- **Cross-device modifier support**: Holding Shift on Device A affects keys on Device B
- **Single-threaded event processing**: QMK-style sequential processing (no race conditions)
- **KISS architecture**: One daemon process, no IPC complexity

#### Device Identification

**Linux** ([evdev](https://linux.die.net/man/4/evdev)):
```bash
# Persistent device IDs (recommended over location-based)
/dev/input/by-id/usb-Vendor_Keyboard_SerialXXX-event-kbd

# Query device info via ioctl
struct input_id device_id;
ioctl(fd, EVIOCGID, &device_id);
// Serial from sysfs: /sys/class/input/eventX/device/../../serial
```

**Windows**:
```cpp
// GetRawInputDeviceInfo for serial number
UINT size;
GetRawInputDeviceInfo(hDevice, RIDI_DEVICENAME, buffer, &size);
// Example: \\?\HID#VID_AAAA&PID_1111#SerialXXX#{...}
```

#### Configuration Model: Single Entry Point with Imports

**System loads ONE file** (`main.rhai`), which can import other files for modularity:

```rhai
// main.rhai - Single entry point
import "devices/left_hand.rhai";   // Device-specific configs
import "devices/right_hand.rhai";
import "layers/vim_mode.rhai";     // Shared layers
import "macros/common.rhai";       // Shared utilities

// Conditional imports
if device_exists("USB\\SERIAL_GAMING") {
    import "devices/gaming.rhai";
}
```

**User's Config Structure**:
```
~/.config/keyrx/
├── main.rhai              # Entry point (daemon loads this)
├── devices/
│   ├── left_hand.rhai     # Left keyboard config
│   └── right_hand.rhai    # Right keyboard config
├── layers/
│   ├── vim_mode.rhai      # Shared layer (used by multiple devices)
│   └── gaming.rhai
└── macros/
    └── common.rhai        # Utility functions
```

**Example: Cross-Device Configuration**

```rhai
// devices/left_hand.rhai
let left = device("USB\\VID_AAAA&PID_1111\\SERIAL_LEFT");

// Define global modifiers (shared state)
left.map(Key::LShift, Modifier(1));
left.map(Key::RShift, Modifier(2));

// devices/right_hand.rhai
let right = device("USB\\VID_BBBB&PID_2222\\SERIAL_RIGHT");

// Respond to left keyboard's modifiers
right.map(Key::A, conditional(
    Modifier(1),  // If left Shift is held
    Key::ShiftA,  // Output uppercase A
    Key::A        // Otherwise lowercase a
));
```

#### Event Processing Pipeline

```
┌─────────────────────────────────────────────────┐
│ OS Layer (Async Capture)                       │
│ ┌──────────┐ ┌──────────┐ ┌──────────┐         │
│ │ evdev A  │ │ evdev B  │ │ evdev C  │         │
│ │ (Shift)  │ │  (A key) │ │(Numpad)  │         │
│ └────┬─────┘ └────┬─────┘ └────┬─────┘         │
│      │ Lock-free  │ Ring        │ Buffers       │
└──────┼────────────┼─────────────┼───────────────┘
       ▼            ▼             ▼
┌─────────────────────────────────────────────────┐
│ Daemon (Single-Threaded Sequential Processing) │
│                                                 │
│ Event Router: Serial# → CompiledConfig (.krx)  │
│      │                                          │
│      ▼                                          │
│ ┌─────────────────────────────────┐            │
│ │ Global ExtendedState            │            │
│ │ - 255 modifiers (bitset)        │            │
│ │ - 255 locks (bitset)            │            │
│ └─────────────────────────────────┘            │
│      │                                          │
│      ▼                                          │
│ Process: Device A Shift Down → Set Modifier(1) │
│ Process: Device B A Down → Lookup with Mod(1)  │
│      │                                          │
│      ▼                                          │
│ Output: uinput inject Shift+A                  │
└─────────────────────────────────────────────────┘
```

**No Timing Issues**:
- Events from all devices enter single queue
- Processed in arrival order (FIFO)
- State updates are immediate and synchronous
- No cross-device race conditions

#### Comparison to Existing Tools

| Feature | keyrx | [Karabiner](https://karabiner-elements.pqrs.org) | [kmonad](https://github.com/kmonad/kmonad) | [QMK](https://docs.qmk.fm) |
|---------|-------|-----------|--------|-----|
| **Cross-device modifiers** | ✅ Yes | ❌ No | ❌ No | ✅ Yes |
| **Serial number config** | ✅ Yes | ❌ No | ❌ No | N/A |
| **Modular imports** | ✅ Yes | ❌ No | ❌ No | ✅ `#include` |
| **Global state** | ✅ Yes | ❌ Per-device | ❌ Per-instance | ✅ Yes |
| **Software-based** | ✅ Yes | ✅ Yes | ✅ Yes | ❌ Firmware |

### Data Storage

#### Primary Storage
- **Configuration Files (.krx)**:
  - Format: rkyv-serialized binary (deterministic, zero-copy)
  - Location: `~/.config/keyrx/` (Linux), `%APPDATA%\keyrx\` (Windows)
  - Loading: Memory-mapped via memmap2 (no deserialization overhead)

- **State Storage**:
  - In-memory only: 255-bit modifier state, 255-bit lock state (fixedbitset)
  - No persistent state (stateless daemon restart)

#### Caching
- **No explicit caching layer**: MPHF tables and memory-mapping provide O(1) access
- **Kernel-level buffering**: evdev/Raw Input queues managed by OS

#### Data Formats
- **Configuration**: rkyv binary (internal), Rhai source (human-editable)
- **Logs**: JSON (structured logging)
- **IPC**: JSON over WebSocket (control channel), shared memory ring buffer (debug events)
- **Frontend State**: React state (ephemeral), WASM linear memory (simulation)

### External Integrations

#### APIs
- **OS Input Subsystem APIs** (core integration):
  - Linux: evdev ioctl, uinput write
  - Windows: SetWindowsHookEx, SendInput, GetRawInputDeviceInfo

#### Protocols
- **WebSocket**: Daemon ↔ UI real-time communication
- **HTTP/REST**: Optional daemon control API (future)
- **IPC**: Named pipes (Windows) / Unix sockets (Linux) for CLI ↔ daemon

#### Authentication
- **Not applicable**: Local system daemon (no network authentication)
- **OS-level permissions**: Managed via udev rules (Linux) or admin privileges (Windows)

### Monitoring & Dashboard Technologies

#### Dashboard Framework
- **React 18+**: Component-based UI
- **TypeScript**: Type safety for complex state management
- **Vite**: Build tool with hot module replacement (HMR)

#### Real-time Communication
- **WebSocket**: Daemon → UI event streaming
  - Event types: configuration updates, input events (debug mode), latency metrics
- **WASM Direct Calls**: UI → keyrx_core simulation (no network latency)

#### Visualization Libraries
- **Custom Canvas Rendering**: Keyboard layout visualization with state highlighting
- **SVG-based State Diagrams**: DFA state transitions (Pending → Held → Tapped)
- **Chart.js** (optional): Latency histograms, usage heatmaps

#### State Management
- **React useState/useReducer**: UI state (current layer, active devices)
- **WASM Memory as SSOT**: Simulation state lives in keyrx_core (WASM), not JavaScript
- **.krx File as Authoritative Source**: Both daemon and UI read same binary

## Development Environment

### Build & Development Tools

#### Build System
- **Cargo**: Primary build tool
  - Workspace configuration for 4 crates
  - Feature flags: `linux`, `windows`, `wasm`, `debug_ui`
- **npm/pnpm**: Frontend build (keyrx_ui)
- **Makefile**: Top-level orchestration (delegates to AI-friendly scripts)

#### AI-Coding-Agent-Friendly Scripts
All scripts follow strict conventions for machine parseability (per AI Coding Agent First principle):

**Output Format**:
- **Consistent markers**: `=== accomplished ===`, `=== failed ===`, `=== warning ===`
- **Structured logs**: `[YYYY-MM-DD HH:MM:SS] [LEVEL] message`
- **Exit codes**: 0 (success), 1 (error), 2 (warning)
- **Timestamped log files**: `logs/build_$(date +%s).log` (epoch timestamp)

**Flag Support** (all scripts):
- `--error`: Output errors only (filter INFO/DEBUG for AI focus)
- `--json`: Machine-readable JSON output
- `--quiet`: Suppress all output except final marker
- `--log-file <path>`: Override default log location

**Example Usage** (for AI agents):
```bash
# Build and check for errors only
./scripts/build.sh --error --log-file logs/build_$(date +%s).log
if grep -q "=== accomplished ===" logs/build_*.log; then
  echo "Build verified by AI agent"
fi
```

**CLAUDE.md Documentation**:
- Located at `scripts/CLAUDE.md`
- Minimal, structured documentation for each script
- AI agents read this first before executing commands
- Includes examples, expected outputs, failure scenarios

#### Package Management
- **Cargo**: Rust dependencies via Cargo.toml
- **npm/pnpm**: JavaScript dependencies via package.json
- **System packages**: evdev headers (Linux), Windows SDK (Windows)

#### Development Workflow
- **Daemon with UI**: `cargo run --bin keyrx_daemon --features web`
  - Serves UI on http://localhost:9876
  - Automatically embeds compiled WASM + React frontend (from ui_dist/)
  - Hot-reloads when .krx config changes
- **Frontend Development**: `npm run dev` (inside keyrx_ui/) with Vite HMR
  - Development mode: http://localhost:5173 (connects to daemon's WebSocket at :9876)
  - Production mode: `npm run build` → outputs to daemon's ui_dist/ → embedded at compile-time
- **Compiler**: `cargo run --bin keyrx_compiler -- config.rhai -o output.krx`
- **Headless Daemon** (no UI): `cargo run --bin keyrx_daemon` (web feature disabled)
- **Build All**: `make build` (compiles daemon + UI, embeds assets)

### Code Quality Tools

#### Pre-Commit Hooks (Mandatory)
Per CLAUDE.md requirements:
- **clippy**: Linting (`cargo clippy -- -D warnings`)
- **rustfmt**: Formatting (`cargo fmt --check`)
- **Tests**: `cargo test --all` must pass
- **Coverage**: `cargo tarpaulin` (80% minimum, 90% for keyrx_core)

#### Static Analysis
- **clippy**: Rust linter (pedantic mode enabled)
- **cargo-deny**: License and dependency auditing
- **cargo-audit**: Security vulnerability scanning

#### Formatting
- **rustfmt**: Rust code formatting (enforced in CI)
- **prettier**: JavaScript/TypeScript formatting

#### Testing Framework
- **Built-in Rust test harness**: Unit tests
- **proptest**: Property-based testing (1M+ generated test cases)
- **cargo-fuzz**: Fuzzing with coverage-guided mutation
- **criterion**: Performance benchmarks with regression detection
- **Deterministic Simulation Testing (DST)**: Custom framework with virtual clock

#### Documentation
- **rustdoc**: API documentation (`cargo doc`)
- **mdBook**: User guide and architecture documentation (future)

### Version Control & Collaboration

#### VCS
- **Git**: Primary version control
- **GitHub**: Hosting, CI/CD, issue tracking

#### Branching Strategy
- **Trunk-Based Development** (per CLAUDE.md: no backward compatibility required)
- **Feature branches**: Short-lived (<3 days), merged to main after CI passes
- **No release branches**: Rolling releases with semantic versioning

#### Code Review Process
- **Pull requests**: Required for all changes
- **Automated checks**: clippy, rustfmt, tests, coverage, benchmarks
- **Review focus**: Architecture, performance implications, test coverage

### Dashboard Development

#### Live Reload
- **Vite HMR**: Instant frontend updates during development (http://localhost:5173)
- **WASM watch mode**: Auto-rebuild on keyrx_core changes (integrated with Vite)
- **Production**: UI assets embedded in daemon binary via `include_dir!` macro

#### Port Management
- **Daemon Web Server**: Default 9876 (serves UI + WebSocket + REST API)
- **Vite Dev Server**: Default 5173 (development only, proxies to daemon)
- **Configuration**: Ports configurable via `~/.config/keyrx/daemon.toml`

#### Multi-Instance Support
- **Single daemon per user session**: OS input hooks are exclusive (evdev GRAB, Windows LL Hook)
- **Multiple UI connections**: Supported (read-only observers via WebSocket)

## Deployment & Distribution

### Target Platform(s)
- **Linux**: x86_64, kernel 5.10+ (evdev requirement)
  - Tested: Ubuntu 22.04+, Fedora 38+, Arch Linux
- **Windows**: x86_64, Windows 10 1903+ (Low-Level Hooks stability)
- **WASM**: Any modern browser (Chrome 90+, Firefox 88+, Safari 15+)

### Distribution Method
- **Binary releases**: GitHub Releases with pre-built binaries
- **Package managers** (future):
  - Linux: AUR (Arch), PPA (Ubuntu), Flatpak
  - Windows: winget, Chocolatey
- **Source builds**: `cargo install keyrx` (crates.io publication)

### Installation Requirements

**Linux**:
- `udev` rules setup (auto-generated by installer)
- Input group membership (non-root access to `/dev/input/event*`)

**Windows**:
- Administrator privileges for initial setup (Low-Level Hook registration)
- User-mode execution after setup

### Update Mechanism
- **Manual**: Download new binary, restart daemon
- **Auto-update** (future): Background downloader with signature verification

## Technical Requirements & Constraints

### Performance Requirements

#### Hard Requirements
- **Latency**: <1ms end-to-end (OS hook → processing → injection)
  - Target: <100μs for 95th percentile
- **Lookup**: O(1) constant-time key lookup (MPHF guarantee)
- **Memory**: <50MB resident set size (daemon + loaded config)
- **CPU**: <1% on idle, <5% under sustained input (1000 keys/sec)

#### Benchmarks
- **MPHF lookup**: <50ns (verified via criterion)
- **DFA state transition**: <50ns
- **rkyv deserialization**: <1μs (zero-copy validation)
- **Event pipeline**: <100μs total (measured via tracing)

### Compatibility Requirements

#### Platform Support
- **OS**: Linux 5.10+, Windows 10 1903+
- **Architecture**: x86_64 (ARM64 future consideration)
- **Desktop Environments**: X11, Wayland, Windows Desktop

#### Dependency Versions
- **Rust**: 1.70+ (MSRV - Minimum Supported Rust Version)
- **glibc**: 2.31+ (Linux)
- **WASM**: wasm32-unknown-unknown target (stable Rust)

#### Standards Compliance
- **Linux Input Subsystem**: evdev protocol compliance
- **USB HID**: Standard HID usage tables (USB.org specification)
- **Windows Input**: Windows Input Architecture compliance

### Security & Compliance

#### Security Requirements
- **No secret logging**: PII/credentials excluded from logs (per CLAUDE.md)
- **Memory safety**: Rust guarantees + `forbid(unsafe_code)` in keyrx_core
- **Input validation**: All external input (Rhai scripts, IPC) validated before execution

#### Threat Model
- **Untrusted configurations**: Malicious .rhai scripts cannot execute arbitrary code (sandboxed Rhai engine)
- **Privilege escalation**: Daemon runs with minimal required privileges (input group, not root)
- **Log injection**: Structured JSON logging prevents injection attacks

#### Compliance Standards
- **Not applicable**: No PII collection, no network transmission (local-only system)

### Scalability & Reliability

#### Expected Load
- **Input rate**: Up to 1000 events/sec (competitive gaming scenario)
- **Configuration size**: Up to 10,000 remapping rules (MPHF handles efficiently)
- **Concurrent users**: Single user per daemon instance

#### Availability Requirements
- **Uptime**: 99.9% (daemon restarts in <100ms)
- **Graceful degradation**: If daemon crashes, input passes through unmodified (no stuck keys)
- **State recovery**: Stateless design enables instant recovery after crash

#### Growth Projections
- **Not applicable**: Single-user system tool (no multi-tenancy)

## Technical Decisions & Rationale

### Decision Log

#### 1. Rust Language Choice
**Decision**: Use Rust for all performance-critical code (core, daemon, compiler)

**Rationale**:
- Memory safety without GC (sub-1ms latency requirement)
- Zero-cost abstractions (high-level code → low-level performance)
- WASM compilation support (browser simulation)
- Strong type system (AI agent verification via compile-time checks)

**Alternatives Considered**:
- C++: Rejected due to undefined behavior risks, harder AI verification
- Go: Rejected due to GC pauses (incompatible with <1ms latency)

#### 2. Rhai for Configuration DSL
**Decision**: Use Rhai scripting language, evaluated at compile-time only

**Rationale**:
- Rust-native (easy FFI, no C dependencies)
- Sandboxed execution (no filesystem/network access)
- Familiar syntax (JavaScript-like, low learning curve)
- Compile-time evaluation eliminates runtime overhead

**Alternatives Considered**:
- Lua: Rejected (C dependency, harder WASM integration)
- YAML/TOML: Rejected (insufficient expressiveness for 255 modifiers)
- JavaScript (Deno): Rejected (heavy runtime, latency concerns)

#### 3. rkyv Zero-Copy Serialization
**Decision**: Use rkyv for .krx binary format

**Rationale**:
- Zero-copy deserialization (no parsing overhead)
- Deterministic output (hash-based verification for AI agents)
- Validation without full deserialization (bytecheck crate)
- Memory-map friendly (direct access to mmap'd files)

**Alternatives Considered**:
- bincode: Rejected (requires deserialization, not zero-copy)
- Protocol Buffers: Rejected (schema compilation complexity, not zero-copy)
- MessagePack: Rejected (parsing overhead)

#### 4. MPHF (Minimal Perfect Hash Function) for Lookup
**Decision**: Use boomphf CHD algorithm for O(1) key lookup

**Rationale**:
- Guaranteed O(1) lookup (no hash collisions)
- Compact representation (sparse keyspace → dense array)
- Build time negligible (compile phase only)
- Cache-friendly (sequential array access)

**Alternatives Considered**:
- std::HashMap: Rejected (non-deterministic, worst-case O(n), resize overhead)
- Direct array indexing: Rejected (sparse keyspace wastes memory)
- B-tree: Rejected (O(log n) lookup, not constant-time)

#### 5. no_std Core Design
**Decision**: keyrx_core is `#![no_std]` (no standard library)

**Rationale**:
- WASM compilation without std (smaller binary, faster)
- Eliminates accidental heap allocation in hot path
- Forces explicit dependency management (better for AI code analysis)
- Proves core logic is OS-agnostic

**Alternatives Considered**:
- Full std support: Rejected (couples core to OS, harder to verify)

#### 6. React + WASM Frontend (Not Electron/Tauri Initially)
**Decision**: Web-based UI with WASM, packaged as local HTML later

**Rationale**:
- WASM simulation shares exact core code with daemon (no drift)
- Faster iteration (web dev tools, HMR)
- Cross-platform by default (same UI on Linux/Windows)
- Lighter weight than Electron (future Tauri packaging possible)

**Alternatives Considered**:
- Native GUI (GTK/Qt): Rejected (platform-specific, no WASM simulation)
- Electron: Rejected (bloat, slower startup)
- TUI (terminal UI): Rejected (insufficient visualization for DFA/state)

## Known Limitations

### 1. Windows Hook Timeout Risk
**Impact**: If keyrx_daemon processing exceeds ~300ms, Windows may silently unhook

**Mitigation**: Lock-free event queue + immediate CallNextHookEx return

**Future Solution**: Kernel driver (requires signing, deployment complexity)

### 2. macOS Not Supported
**Impact**: macOS users cannot use keyrx

**Why**: Requires CGEventTap API research and implementation

**Timeline**: Post-1.0 (after Linux/Windows stabilization)

### 3. WASM Simulation Cannot Test OS-Specific Quirks
**Impact**: evdev/Windows hook edge cases not testable in browser

**Mitigation**: E2E tests on real OS (GitHub Actions matrix)

**Future Solution**: Record/replay of OS events for deterministic testing

### 4. 255 Modifiers/Locks May Exceed OS Virtual Key Limits
**Impact**: Some OS key codes may conflict with custom modifier IDs

**Mitigation**: Namespace separation (custom IDs start at 0x8000, above standard range)

**Future Solution**: Virtual key remapping table (if needed)

### 5. No Multi-User Support
**Impact**: One keyrx_daemon per system (not per user session)

**Why**: Global input hooks are system-wide (Windows) or require device grab (Linux)

**Future Solution**: User-session isolation via systemd user services (Linux) or per-session hooks (Windows, complex)
