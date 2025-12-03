# KeyRx

The Ultimate Input Remapping Engine - a cross-platform keyboard remapper powered by Rhai scripting.

- **core/**: The Rust backend (Logic, Rhai Scripting).
- **ui/**: The Flutter frontend (GUI, Visualizer).
- **docs/**: Documentation and Architecture.

## Origin of the Name

**KeyRx** (pronounced "Key-Rex" or "Key-Rx") carries a triple meaning:

1. **"Rex" (The King/Dinosaur)**: Like *Tyrannosaurus Rex*, this tool is designed to be the dominant, powerful force in input remapping.
2. **"Rx" (The Prescription)**: Default keyboard layouts are often "broken" or inefficient. You are the doctor prescribing a script to fix your input and make it healthy for your hands.
3. **"Rx" (Reactive)**: The engine is built on *Reactive Programming* principles. Input is treated as a stream of events that flows through your logic to trigger instant reactions.

## Installation

```bash
cd core
cargo build --release
```

The binary will be at `core/target/release/keyrx`.

## Development Setup

### Prerequisites

- [Rust](https://rustup.rs/) (stable toolchain)
- [Flutter](https://docs.flutter.dev/get-started/install) (for UI development)
- [just](https://github.com/casey/just#installation) (task runner)

### Quick Setup

```bash
just setup
```

This installs all toolchain components, development tools (cargo-nextest, cargo-watch), dependencies, and git hooks.

### Available Commands

Run `just` to see all available commands:

| Command | Description |
|---------|-------------|
| `just setup` | Install tools, dependencies, and git hooks |
| `just dev` | Run core with auto-reload (cargo watch) |
| `just ui` | Run Flutter UI in development mode |
| `just check` | Run all quality checks (fmt, clippy, test) |
| `just fmt` | Format Rust code |
| `just clippy` | Run clippy linter |
| `just test` | Run tests with nextest |
| `just bench` | Run benchmarks |
| `just clean` | Clean build artifacts |

## Building

### Current Platform

```bash
just build
```

Creates an optimized release binary at `core/target/release/keyrx`.

### All Platforms

```bash
just build-all
```

Builds for Linux and Windows x86_64. Windows builds require [cross](https://github.com/cross-rs/cross).

### Individual Platforms

```bash
just build-linux    # Linux x86_64
just build-windows  # Windows x86_64 (requires cross)
```

## Contributing

### Code Quality Checks

Before submitting changes, run the full quality suite:

```bash
just check
```

This runs formatting checks, clippy linting, and all tests.

### Pre-commit Hooks

Git hooks are automatically installed with `just setup`. The pre-commit hook runs:

1. `cargo fmt --check` - Code formatting
2. `cargo clippy -- -D warnings` - Linting
3. `cargo test --lib` - Unit tests

Commits are blocked if any check fails. Run `just fmt` to fix formatting issues.

### Creating a Release

```bash
just release 1.0.0
```

This updates version numbers, generates the changelog, and creates a git tag.

## Quick Start

### 1. Check your system

```bash
keyrx doctor
```

This runs diagnostics to verify your system is ready (uinput on Linux, keyboard hooks on Windows).

### 2. Validate a script

```bash
keyrx check scripts/std/example.rhai
```

### 3. Run with a script

```bash
keyrx run --script scripts/std/example.rhai
```

Use `--mock` flag to test without real keyboard capture:

```bash
keyrx run --script scripts/std/example.rhai --mock
```

### 4. Simulate key events

Test your remapping without running the full engine:

```bash
keyrx simulate --input "CapsLock,A,B,Insert" --script scripts/std/example.rhai
```

### 5. Benchmark latency

```bash
keyrx bench --iterations 10000
```

## Example Script

Create a file `my-config.rhai`:

```javascript
fn on_init() {
    print_debug("My custom config loaded!");

    // Remap CapsLock to Escape (Vim users love this!)
    remap("CapsLock", "Escape");

    // Block the Insert key (stop accidental overwrites)
    block("Insert");

    // Pass Enter through unchanged (explicit, default behavior)
    pass("Enter");
}
```

Run it:

```bash
keyrx run --script my-config.rhai
```

## CLI Commands

| Command    | Description                                    |
|------------|------------------------------------------------|
| `check`    | Validate and lint a Rhai script                |
| `run`      | Run the engine with optional script            |
| `simulate` | Simulate key events without real keyboard      |
| `devices`  | List available keyboard devices                |
| `doctor`   | Run self-diagnostics                           |
| `bench`    | Run latency benchmark                          |
| `state`    | Inspect current engine state                   |
| `repl`     | Start interactive REPL (not yet implemented)   |
| `uat`      | Run User Acceptance Tests                      |
| `golden`   | Manage golden sessions for regression testing  |
| `regression` | Verify golden sessions for regressions       |
| `ci-check` | Run complete CI test suite with gates          |

Use `--format json` for machine-readable output.

## Testing

### User Acceptance Tests (UAT)

Run the UAT test suite:

```bash
keyrx uat
```

Filter tests by category or priority:

```bash
keyrx uat --category core --priority P0
keyrx uat --category layers
```

Apply a quality gate:

```bash
keyrx uat --gate default  # 95% pass rate required
keyrx uat --gate alpha    # Relaxed (80% pass rate)
keyrx uat --gate ga       # Strictest (100% pass rate)
```

### Golden Sessions (Regression Testing)

Record a golden session:

```bash
keyrx golden record my_session --script path/to/script.rhai
```

Verify a golden session:

```bash
keyrx golden verify my_session
```

Run all regression tests:

```bash
keyrx regression
```

### CI Check

Run the complete CI test suite:

```bash
keyrx ci-check                  # Run all tests
keyrx ci-check --gate beta      # With quality gate enforcement
keyrx ci-check --skip-perf      # Skip performance tests
keyrx ci-check --json           # JSON output for CI parsing
```

### Quality Gates

Quality gates are defined in `.keyrx/quality-gates.toml`:

| Gate | Pass Rate | Max P0 | Max P1 | Max Latency | Min Coverage |
|------|-----------|--------|--------|-------------|--------------|
| alpha | 80% | 0 | 5 | 2000µs | 60% |
| beta | 90% | 0 | 2 | 1000µs | 75% |
| default | 95% | 0 | 2 | 1000µs | 80% |
| rc | 98% | 0 | 0 | 500µs | 85% |
| ga | 100% | 0 | 0 | 500µs | 90% |

### Writing UAT Tests

Create Rhai test files in `tests/uat/`:

```javascript
// @category: core
// @priority: P0
// @requirement: REQ-001
// @latency: 1000
fn uat_basic_mapping() {
    let result = 1 + 1;
    if result != 2 {
        throw "Basic math failed";
    }
}
```

Metadata tags:
- `@category`: Test category for filtering (e.g., core, layers, performance)
- `@priority`: P0 (critical), P1 (high), P2 (normal)
- `@requirement`: Traceability to requirements
- `@latency`: Maximum allowed execution time in microseconds

## Key Reference

**Modifiers**: `LeftShift`, `RightShift`, `LeftCtrl`, `RightCtrl`, `LeftAlt`, `RightAlt`, `LeftMeta`, `RightMeta`

**Navigation**: `Up`, `Down`, `Left`, `Right`, `Home`, `End`, `PageUp`, `PageDown`

**Function Keys**: `F1` - `F12`

**Special**: `Escape`, `Tab`, `CapsLock`, `Space`, `Enter`, `Backspace`, `Insert`, `Delete`

See `scripts/std/example.rhai` for a complete key reference.

## Platform Setup

### Linux

KeyRx uses **evdev** for reading keyboard events and **uinput** for injecting remapped keys.

#### 1. Add your user to the `input` group

```bash
sudo usermod -aG input $USER
```

**Important**: Log out and back in for group changes to take effect.

#### 2. Load the uinput kernel module

```bash
sudo modprobe uinput
```

To make this persistent across reboots:

```bash
echo "uinput" | sudo tee /etc/modules-load.d/uinput.conf
```

#### 3. Set up udev rules (recommended)

Create `/etc/udev/rules.d/99-keyrx.rules`:

```
# Allow input group to access input devices
KERNEL=="event*", SUBSYSTEM=="input", MODE="0660", GROUP="input"

# Allow input group to create uinput devices
KERNEL=="uinput", SUBSYSTEM=="misc", MODE="0660", GROUP="input"
```

Reload udev rules:

```bash
sudo udevadm control --reload-rules
sudo udevadm trigger
```

#### 4. List available keyboards

```bash
keyrx devices
```

#### 5. Run with a specific device (optional)

```bash
keyrx run --script my-config.rhai --device /dev/input/event3
```

### Windows

KeyRx uses **low-level keyboard hooks** for capturing events and **SendInput** for key injection.

#### Requirements

- No administrator privileges required for basic operation
- Antivirus software may flag keyboard hooks - add an exception if needed

#### Running

```bash
keyrx.exe run --script my-config.rhai
```

**Note**: Some enterprise security software may block low-level keyboard hooks. Contact your IT department if you encounter issues.

## Troubleshooting

### Linux

| Issue | Solution |
|-------|----------|
| "Permission denied" accessing `/dev/input/event*` | Add user to `input` group and re-login |
| "Failed to create uinput device" | Run `sudo modprobe uinput` and check udev rules |
| "Device not found" | Run `keyrx devices` to list available keyboards |
| Keyboard stays grabbed after crash | Run `sudo killall keyrx` or unplug/replug keyboard |

### Windows

| Issue | Solution |
|-------|----------|
| "Failed to install keyboard hook" | Check for conflicting keyboard software |
| Antivirus blocking KeyRx | Add KeyRx to antivirus exclusions |
| Keys not being remapped | Ensure KeyRx is running in foreground |
| Keyboard unresponsive | Press Ctrl+C to stop KeyRx cleanly |

### General

- Use `keyrx doctor` to run diagnostics
- Use `--mock` flag to test scripts without capturing real keyboard
- Press **Ctrl+C** to stop KeyRx and release the keyboard

## Flutter UI

KeyRx includes a Flutter-based GUI for visual configuration and debugging.

### Running the UI

```bash
just ui            # Development mode
just ui-build      # Production build
```

### User Interface (4 Main Screens)

| Screen | Description |
|--------|-------------|
| **Editor** | Visual script editor with syntax highlighting and validation |
| **Devices** | Keyboard device selection and profile management |
| **Run Controls** | Start/stop engine with recording toggle and status indicators |
| **Training** | Typing pattern analysis for optimal tap-hold timeout |

### Developer Tools

Access developer tools via the wrench icon in the app bar (developer mode).

| Tool | Description |
|------|-------------|
| **Debugger** | Real-time state visualization with layers, modifiers, pending |
| **Console** | Interactive REPL for Rhai commands |
| **Test Runner** | Discover and execute Rhai test functions |
| **Simulator** | Build key sequences visually and test mappings |
| **Analyzer** | Session analysis with statistics and decision breakdown |
| **Benchmark** | Latency performance testing (min/mean/p99/max) |
| **Doctor** | System diagnostics with remediation steps |
| **Replay** | Session replay with verification mode |
| **Discovery** | Guided wizard for keyboard layout discovery |

### Screenshots

<!-- TODO: Add screenshots -->

## License

See LICENSE file for details.
