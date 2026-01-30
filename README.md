# KeyRx2

[![License: AGPL-3.0-or-later](https://img.shields.io/badge/License-AGPL%203.0+-blue.svg)](https://www.gnu.org/licenses/agpl-3.0)
[![Rust](https://img.shields.io/badge/rust-1.70%2B-orange.svg)](https://www.rust-lang.org/)
[![Platform: Linux](https://img.shields.io/badge/Linux-Supported-green.svg)](docs/user-guide/linux-setup.md)
[![Platform: Windows](https://img.shields.io/badge/Windows-Supported-green.svg)](docs/user-guide/windows-setup.md)

**Advanced keyboard remapping with layer support, tap-hold behavior, and conditional mappings.**

KeyRx2 is a cross-platform keyboard remapping system that provides powerful customization beyond basic key rebinding. It features custom modifiers (layers), tap-hold dual-function keys, conditional mappings, and device-specific configurations.

## Features

- **Custom Modifiers (Layers)**: Create up to 255 custom modifiers for layer-based layouts (Vim-style navigation, symbol layers, etc.)
- **Tap-Hold Behavior**: Keys that behave differently when tapped vs. held (e.g., Space tap=space, hold=navigation layer)
- **Conditional Mappings**: Different behaviors based on modifier/lock state
- **Custom Locks (Toggles)**: Toggle states like gaming mode, numpad mode, etc. (255 available)
- **Device-Specific Configs**: Different mappings for different keyboards
- **Multi-File Configurations**: Organize complex configurations with imports
- **Deterministic Compilation**: Same input always produces identical output
- **Binary Format**: Fast, zero-copy deserialization with integrity checking
- **WASM Support**: Test configurations in browser before deploying

## Architecture

KeyRx2 is a Rust-based workspace with four crates:

- **keyrx_core**: Platform-agnostic remapping logic (no_std, WASM-compatible)
- **keyrx_compiler**: Rhai DSL → .krx binary compiler (CLI tool)
- **keyrx_daemon**: OS-level keyboard interception daemon (Linux, Windows)
- **keyrx_ui**: React + WASM web interface for configuration testing

## Quickstart

### 1. Install the Compiler

Build the compiler from source:

```bash
cargo build --release -p keyrx_compiler
```

The binary will be at `target/release/keyrx_compiler`.

Optionally, install it to your PATH:

```bash
cargo install --path keyrx_compiler
```

### 2. Write a Configuration

Create a file `my-config.rhai` with your key mappings:

```rhai
// Simple example: CapsLock → Escape
device_start("*");  // "*" matches all devices

    map("CapsLock", "VK_Escape");

device_end();
```

More complex example with Vim navigation layer:

```rhai
device_start("*");

    // CapsLock becomes custom modifier MD_00
    map("CapsLock", "MD_00");

    // When CapsLock (MD_00) is held, enable Vim navigation
    when_start("MD_00");
        map("H", "VK_Left");
        map("J", "VK_Down");
        map("K", "VK_Up");
        map("L", "VK_Right");
    when_end();

device_end();
```

See [examples/](examples/) for more configurations:
- [01-simple-remap.rhai](examples/01-simple-remap.rhai) - Basic remapping
- [02-capslock-escape.rhai](examples/02-capslock-escape.rhai) - Classic CapsLock→Escape
- [03-vim-navigation.rhai](examples/03-vim-navigation.rhai) - Vim-style HJKL navigation
- [04-dual-function-keys.rhai](examples/04-dual-function-keys.rhai) - Tap-hold behavior
- [05-multiple-devices.rhai](examples/05-multiple-devices.rhai) - Device-specific configs
- [06-advanced-layers.rhai](examples/06-advanced-layers.rhai) - Complex multi-layer setup

### 3. Compile the Configuration

Compile your Rhai script to a binary `.krx` file:

```bash
keyrx_compiler compile my-config.rhai -o my-config.krx
```

If you omit `-o`, the output will be `my-config.krx` (same name with `.krx` extension).

### 4. Verify the Output

Check that the compiled file is valid:

```bash
keyrx_compiler verify my-config.krx
```

Output:
```
✓ Magic bytes valid
✓ Version: 1
✓ SHA256 hash matches
✓ rkyv deserialization successful
✓ Configuration valid (1 device, 5 mappings)
✓ Verification passed
```

### 5. Run the Daemon (Linux)

Build and run the daemon:

```bash
# Build the daemon
cargo build --release -p keyrx_daemon --features linux

# List available keyboard devices
./target/release/keyrx_daemon list-devices

# Validate your configuration (dry-run)
./target/release/keyrx_daemon validate --config my-config.krx

# Run the daemon (requires root or proper permissions)
sudo ./target/release/keyrx_daemon run --config my-config.krx
```

For non-root operation and systemd integration, see the [Linux Setup Guide](docs/user-guide/linux-setup.md).

### 6. Run the Daemon (Windows)

Build and run the daemon:

```powershell
# Build the daemon with windows feature
cargo build --release -p keyrx_daemon --features windows

# Run the daemon with your configuration
.\target\release\keyrx_daemon.exe run --config my-config.krx
```

The daemon will appear in your system tray. You can right-click the icon to reload the configuration or exit the application. For more details, see the [Windows Setup Guide](docs/user-guide/windows-setup.md).

### 7. Test in Browser (Optional)

You can also test configurations in the browser using the WASM simulator:

```bash
# Build and run the UI
cd keyrx_ui
npm install
npm run dev
```

Then open http://localhost:5173 and load your `.krx` file.

## Documentation

### User Guides
- **[DSL Manual](docs/user-guide/dsl-manual.md)** - Complete reference for the KeyRx DSL with syntax, functions, and examples
- **[Linux Setup Guide](docs/user-guide/linux-setup.md)** - Linux installation, permissions, and systemd integration
- **[Windows Setup Guide](docs/user-guide/windows-setup.md)** - Windows installation, tray icon usage, and troubleshooting
- **[Examples](examples/)** - Six example configurations from basic to advanced

### Developer Guides
- **[Version Management](docs/version-management.md)** - Step-by-step version update procedures and SSOT enforcement
- **[Troubleshooting Installer](docs/troubleshooting-installer.md)** - Comprehensive guide to diagnosing installation issues
- **[Windows VM Setup (Vagrant)](docs/development/windows-vm-setup.md)** - Setting up a Windows development environment on Linux
- **[Compiler README](keyrx_compiler/README.md)** - CLI commands and usage
- **[Core README](keyrx_core/README.md)** - Architecture and library API

## CLI Reference

### compile

Compile a Rhai script to a `.krx` binary:

```bash
keyrx_compiler compile input.rhai -o output.krx
```

### verify

Verify a `.krx` file's integrity:

```bash
keyrx_compiler verify config.krx
```

### parse

Parse and inspect a Rhai script without compiling:

```bash
# Human-readable output
keyrx_compiler parse input.rhai

# JSON output
keyrx_compiler parse input.rhai --json
```

### hash

Extract and verify the SHA256 hash from a `.krx` file:

```bash
# Extract hash
keyrx_compiler hash config.krx

# Verify hash matches data
keyrx_compiler hash config.krx --verify
```

## DSL Quick Reference

### Key Prefixes

All output keys require a prefix to indicate their type:

- **VK_** - Virtual Key (standard key output): `VK_A`, `VK_Enter`, `VK_Escape`
- **MD_** - Custom Modifier (layer switching): `MD_00` through `MD_FE` (0-254)
- **LK_** - Custom Lock (toggle state): `LK_00` through `LK_FE` (0-254)

Input keys (the `from` parameter) have no prefix: `CapsLock`, `Space`, `A`.

### Core Functions

```rhai
// Simple remapping
map("CapsLock", "VK_Escape");

// Custom modifier (layer)
map("CapsLock", "MD_00");

// Custom lock (toggle)
map("ScrollLock", "LK_00");

// Tap-hold dual function
tap_hold("Space", "VK_Space", "MD_01", 200);
// tap=space, hold=MD_01, threshold=200ms

// Physical modifier output
map("F1", with_shift("VK_F1"));     // Output Shift+F1
map("F2", with_ctrl("VK_F2"));      // Output Ctrl+F2
map("F3", with_alt("VK_F3"));       // Output Alt+F3

// Conditional mappings
when_start("MD_00");
    map("H", "VK_Left");  // Only when MD_00 active
when_end();

when_not_start("LK_00");
    map("W", "VK_W");     // Only when LK_00 NOT active
when_not_end();

// Multiple conditions (AND logic)
when_start(["MD_00", "MD_01"]);
    map("1", "VK_F1");    // When both active
when_end();

// Device-specific configuration
device_start("USB Keyboard");
    map("Enter", "VK_Space");
device_end();

device_start("*");  // Wildcard matches all devices
    map("CapsLock", "VK_Escape");
device_end();
```

See [DSL Manual](docs/user-guide/dsl-manual.md) for complete documentation.

## Installation

### Windows MSI Installer

For Windows users, we provide an MSI installer:

```powershell
# Download the latest release from GitHub
# Then install:
msiexec /i KeyRx-0.1.5-x64.msi /qn

# Or double-click the MSI file
```

The installer will:
- Install KeyRx to `C:\Program Files\KeyRx`
- Automatically start the daemon with administrator rights
- Add system tray icon for easy control

**Troubleshooting Installation:**

If you encounter issues:

```powershell
# Run pre-install diagnostics
.\scripts\installer-health-check.ps1 -PreInstall

# Run post-install diagnostics
.\scripts\installer-health-check.ps1 -PostInstall

# Comprehensive troubleshooting
.\scripts\diagnose-installation.ps1
```

See the **[Troubleshooting Installer](docs/troubleshooting-installer.md)** guide for detailed help.

### Version Management

KeyRx uses Single Source of Truth (SSOT) for version management. To update the version:

```bash
# Update all version sources automatically
./scripts/sync-version.sh 0.2.0

# Verify consistency
./scripts/sync-version.sh --check

# Build and install
cargo build --release
```

See the **[Version Management](docs/version-management.md)** guide for complete procedures.

## Troubleshooting

### Installation Issues

See **[Troubleshooting Installer](docs/troubleshooting-installer.md)** for comprehensive installation troubleshooting, including:
- Access denied errors
- File lock issues
- Version mismatches
- Port conflicts
- Daemon won't start

### Common Errors

**Missing prefix error:**
```
Error: Missing prefix: expected VK_/MD_/LK_, got 'A'
help: Output keys must have a prefix. Did you mean 'VK_A'?
```
**Fix**: Add the appropriate prefix to your output key: `VK_A`, `MD_00`, or `LK_00`.

**Physical modifier in MD_ error:**
```
Error: Physical modifier name in MD_: MD_LShift
help: Custom modifiers must use hex IDs (MD_00-MD_FE), not physical names.
```
**Fix**: Use a hex ID like `MD_00` instead of `MD_LShift`. Physical modifiers (LShift, RCtrl, etc.) cannot be used as custom modifier IDs.

**Out of range error:**
```
Error: Modifier ID out of range: 0xFF
help: Valid range is 0x00-0xFE (0-254)
```
**Fix**: Use IDs from `MD_00` to `MD_FE`. The value `FF` (255) is reserved.

**Syntax error:**
```
Error: Syntax error at line 10, column 5
help: Check for missing semicolons, quotes, or parentheses
```
**Fix**: Review your Rhai syntax. Common issues:
- Missing semicolons at end of statements
- Unmatched quotes or parentheses
- Calling `device_start()` without matching `device_end()`

**Permission denied (Linux):**
```
Error: Permission denied when accessing /dev/input/eventX
```
**Fix**: Set up proper permissions for non-root operation:
```bash
# Install udev rules
sudo cp keyrx_daemon/udev/99-keyrx.rules /etc/udev/rules.d/
sudo udevadm control --reload-rules && sudo udevadm trigger

# Add user to required groups
sudo groupadd -f uinput
sudo usermod -aG input,uinput $USER

# Log out and back in for changes to take effect
```
Or run with sudo: `sudo ./target/release/keyrx_daemon run --config config.krx`

See [Linux Setup Guide](docs/user-guide/linux-setup.md) for complete instructions.

### Getting Help

- Check the [DSL Manual](docs/user-guide/dsl-manual.md) for function reference
- Look at [examples/](examples/) for working configurations
- Run `keyrx_compiler parse your-config.rhai` to inspect the parsed configuration
- Use `--json` flag for detailed output: `keyrx_compiler parse --json your-config.rhai`

## Development

### Build All Crates

```bash
make build
# Or: scripts/build.sh
```

### Run Tests

```bash
make test
# Or: scripts/test.sh
```

### Run Quality Checks

```bash
make verify
# Or: scripts/verify.sh
```

This runs:
- Cargo build (clean workspace build)
- Clippy linting (treats warnings as errors)
- Rustfmt check (code formatting)
- Cargo test (all tests)
- Coverage analysis (80% minimum required)

### Setup Development Environment

Install required tools:

```bash
make setup
```

This installs:
- cargo-watch (continuous build)
- cargo-tarpaulin (code coverage)
- cargo-fuzz (fuzzing)
- wasm-pack (WASM compilation)
- Git pre-commit hooks (automated quality gates)

## Contributing

Contributions are welcome! Please follow these guidelines:

### Code Quality Standards

All code must meet these requirements before merging:

1. **Clippy**: No warnings (`cargo clippy -- -D warnings`)
2. **Rustfmt**: Properly formatted (`cargo fmt`)
3. **Tests**: All tests pass (`cargo test`)
4. **Coverage**: Minimum 80% code coverage (90% for critical paths)
5. **File Size**: Maximum 500 lines per file (excluding comments/blanks)
6. **Function Size**: Maximum 50 lines per function

### Workflow

1. Fork the repository
2. Create a feature branch: `git checkout -b feature/my-feature`
3. Make your changes following code quality standards
4. Run `make verify` to ensure all checks pass
5. Commit your changes: `git commit -m "feat: add feature"`
6. Push to your fork: `git push origin feature/my-feature`
7. Open a pull request

### Pre-Commit Hooks

Pre-commit hooks automatically run quality checks before each commit. To install:

```bash
make setup
```

To bypass hooks (not recommended):

```bash
git commit --no-verify
```

### Commit Message Format

Follow conventional commits:

- `feat:` - New feature
- `fix:` - Bug fix
- `docs:` - Documentation changes
- `test:` - Test changes
- `refactor:` - Code refactoring
- `chore:` - Build process or tooling changes

## Architecture Overview

### Configuration Flow

```
.rhai script
    ↓
[keyrx_compiler]
    ↓
.krx binary
    ↓
[keyrx_daemon] ← loads config
    ↓
OS keyboard events
    ↓
[keyrx_core] ← remapping logic
    ↓
Modified events
```

### Key Components

1. **Compiler** (`keyrx_compiler`):
   - Parses Rhai DSL scripts
   - Validates key names and prefixes
   - Resolves imports and detects circular dependencies
   - Serializes to `.krx` binary format with SHA256 hash

2. **Core** (`keyrx_core`):
   - Zero-copy deserialization of `.krx` files
   - O(1) key lookup using MPHF
   - DFA state machine for tap-hold behavior
   - 255 custom modifiers + 255 custom locks

3. **Daemon** (`keyrx_daemon`):
   - OS-level keyboard event interception
   - Linux: evdev/uinput
   - Windows: Low-level keyboard hooks
   - Embedded web server for UI

4. **UI** (`keyrx_ui`):
   - React frontend
   - WASM-based configuration simulator
   - Real-time testing without hardware

## Platform Support

### Linux ✅ Fully Supported
- **evdev** for input capture
- **uinput** for output injection
- Comprehensive setup guide: [docs/user-guide/linux-setup.md](docs/user-guide/linux-setup.md)
- Tested on Ubuntu 20.04+, Arch Linux, Fedora

### Windows ✅ Supported
- **Low-level Keyboard Hooks** for input capture and suppression
- **SendInput API** for output injection
- **System Tray Icon** for daemon control (Reload, Exit)
- Comprehensive setup guide: [docs/user-guide/windows-setup.md](docs/user-guide/windows-setup.md)
- Tested on Windows 10 and Windows 11

### macOS ❌ Not Planned
- Not currently on the roadmap
- Contributions welcome if there's community interest

## License

Copyright © 2024 keyrx contributors

This program is free software: you can redistribute it and/or modify it under the terms of the **GNU Affero General Public License** as published by the Free Software Foundation, either version 3 of the License, or (at your option) any later version.

This program is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the [GNU Affero General Public License](https://www.gnu.org/licenses/agpl-3.0.html) for more details.

**Why AGPL-3.0?**
The AGPL-3.0 license ensures that if you run a modified version of this software as a network service, you must make the source code available to users of that service. This prevents proprietary forks while encouraging contributions back to the community.

See [LICENSE](LICENSE) for the full license text.

## Credits

Built with:
- [Rust](https://www.rust-lang.org/) - Systems programming language
- [Rhai](https://rhai.rs/) - Embedded scripting language
- [rkyv](https://rkyv.org/) - Zero-copy serialization
- [boomphf](https://github.com/10XGenomics/rust-boomphf) - Minimal Perfect Hash Functions
- [clap](https://github.com/clap-rs/clap) - CLI argument parsing
