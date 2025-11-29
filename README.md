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
| `doctor`   | Run self-diagnostics                           |
| `bench`    | Run latency benchmark                          |
| `state`    | Inspect current engine state                   |
| `repl`     | Start interactive REPL (not yet implemented)   |

Use `--format json` for machine-readable output.

## Key Reference

**Modifiers**: `LeftShift`, `RightShift`, `LeftCtrl`, `RightCtrl`, `LeftAlt`, `RightAlt`, `LeftMeta`, `RightMeta`

**Navigation**: `Up`, `Down`, `Left`, `Right`, `Home`, `End`, `PageUp`, `PageDown`

**Function Keys**: `F1` - `F12`

**Special**: `Escape`, `Tab`, `CapsLock`, `Space`, `Enter`, `Backspace`, `Insert`, `Delete`

See `scripts/std/example.rhai` for a complete key reference.

## Platform Support

- **Linux**: Uses evdev/uinput (requires `input` group membership)
- **Windows**: Uses low-level keyboard hooks

## License

See LICENSE file for details.
