# KeyRx

The Ultimate Input Remapping Engine - a cross-platform keyboard remapper powered by Rhai scripting.

## Overview

**KeyRx** (pronounced "Key-Rex" or "Key-Rx") carries a triple meaning:

1. **"Rex" (The King)**: Designed to be the dominant, powerful force in input remapping
2. **"Rx" (The Prescription)**: You are the doctor prescribing a script to fix your input
3. **"Rx" (Reactive)**: Built on reactive programming principles with instant event reactions

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

Runs diagnostics to verify your system is ready.

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

| Command | Description |
|---------|-------------|
| `check` | Validate and lint a Rhai script |
| `run` | Run the engine with optional script |
| `simulate` | Simulate key events without real keyboard |
| `devices` | List available keyboard devices |
| `doctor` | Run self-diagnostics |
| `bench` | Run latency benchmark |
| `state` | Inspect current engine state |
| `repl` | Start interactive REPL |
| `uat` | Run User Acceptance Tests |
| `golden` | Manage golden sessions for regression testing |
| `regression` | Verify golden sessions for regressions |
| `ci-check` | Run complete CI test suite with gates |

Use `--format json` for machine-readable output.

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

**Note**: Some enterprise security software may block low-level keyboard hooks.

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

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for development setup, testing, and contribution guidelines.

## License

See LICENSE file for details.
