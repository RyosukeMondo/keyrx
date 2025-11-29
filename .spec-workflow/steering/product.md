# Product Overview

## Product Purpose
KeyRx is an advanced input remapping engine that empowers users to fully control their keyboard and input behavior through programmable scripts. Unlike traditional remappers that simply map key A to key B, KeyRx treats input as a programmable stream of events, enabling complex behaviors, layer systems, and context-aware remapping.

**Core Problem Solved**: Default keyboard layouts are often inefficient or broken for power users. Existing remapping tools are either too limited (static config files) or too complex (legacy C++ codebases). KeyRx bridges this gap with a scriptable, visualizable, high-performance solution.

## Target Users

### Primary Users
1. **Power Users / Keyboard Enthusiasts**: Users who want complete control over their input, including custom modifiers, layers, and context-aware behaviors.
2. **Developers / Vim Users**: Those who need application-specific keybindings and modal editing support.
3. **Accessibility Users**: People who need to remap keys for ergonomic or physical accessibility reasons.
4. **Gamers**: Users requiring custom macros, layer switching, and low-latency input handling.

### Pain Points Addressed
- Static YAML/JSON configs are insufficient for complex behaviors
- Existing tools have high latency or crash-prone behavior
- Complex layer systems are impossible to debug or visualize
- Sharing configurations safely between users is risky

## Key Features

1. **Rhai Scripting Engine**: Full programmable logic instead of static configuration files. Define behaviors, conditions, and reactions in a sandboxed scripting language.

2. **Blank Slate Data Model**: Start from nothing and define only what you need. No fighting against OS defaults or existing layouts.

3. **255 Custom Modifiers**: Create unlimited virtual modifiers (Mod_Thumb, Mod_Gaming, Mod_Photoshop) that exist only in the engine.

4. **Real-time Visual Debugger**: See exactly which layer is active, which modifiers are held, and why a key was blocked.

5. **Visual Keymap Editor**: Drag-and-drop interface that generates the underlying Rhai script automatically.

6. **Rhai REPL Console**: Type commands directly into the running engine for testing and experimentation.

7. **Cross-Platform**: Consistent behavior on Windows and Linux while respecting native input paradigms.

## Business Objectives

- Become the dominant, professional-grade input remapping tool for power users
- Build a community ecosystem of shared, safe Rhai scripts
- Achieve sub-1ms latency for "invisible" performance feel
- Replace legacy tools (Yamy, AutoHotkey, Karabiner) with a modern alternative

## Success Metrics

- **Latency**: < 1ms input-to-output processing time (measured via criterion benchmarks)
- **Reliability**: Zero crashes under fuzz testing with 100,000+ random key combinations
- **Adoption**: Community standard library with 50+ shared scripts
- **Performance Regression**: CI fails if any PR increases latency > 100 microseconds

## Product Principles

1. **Logic > Configuration**: Static config files are insufficient. Scripting (Rhai) enables true programmable behavior.

2. **Visual > Abstract**: Complex state machines must be visible to be understood. The UI is a window into the engine's brain.

3. **Performance > Features**: Latency is the enemy. The tool must feel invisible. No feature is worth perceptible lag.

4. **Safety First**: Scripts are sandboxed. Community scripts cannot access filesystem, network, or crash the app.

5. **CLI First, GUI Later**: Every feature must be exercisable via CLI before GUI implementation. This enables rapid trial, automated testing, and autonomous AI agent development.

## Monitoring & Visibility

- **Dashboard Type**: Flutter desktop application with integrated debugger
- **Real-time Updates**: FFI bridge provides instant state reflection (no HTTP/JSON overhead)
- **Key Metrics Displayed**: Active layers, held modifiers, blocked keys, script execution trace
- **Sharing Capabilities**: Export/import Rhai scripts, shareable layer configurations

## CLI-First Development

All core features are CLI-exercisable for rapid development and AI agent autonomy:

```bash
# Load and validate a script
keyrx check scripts/user_config.rhai

# Run engine in headless/debug mode
keyrx run --debug --script scripts/user_config.rhai

# Simulate key events without real input
keyrx simulate --input "A,B,Ctrl+C" --script scripts/user_config.rhai

# Inspect current state
keyrx state --layers --modifiers

# Run self-check / health diagnostics
keyrx doctor

# Benchmark latency
keyrx bench
```

This enables:
- **Rapid Trial**: Test scripts without GUI overhead
- **Self-Check**: Automated validation and diagnostics
- **AI Agent Development**: Autonomous testing and iteration by AI tools

## Future Vision

### Phase 1: The Iron Core (Headless)
Rust binary that accepts input, runs Rhai scripts, and outputs results.

### Phase 2: The Nervous System (Drivers)
Platform-specific drivers for Windows (WH_KEYBOARD_LL) and Linux (uinput/evdev).

### Phase 3: The Face (Flutter)
Beautiful GUI with FFI bindings, Visual Layer Editor, and REPL console.

### Phase 4: The Ecosystem
Community sharing via standard library (`std/layouts`, `std/modifiers`), enabling users to build on each other's work.

### Potential Enhancements
- **Mouse/MIDI Support**: Extend beyond keyboard to unified input handling
- **Cloud Sync**: Sync configurations across machines
- **Profile System**: Per-application or per-device profiles
- **Hardware Integration**: Direct firmware communication with programmable keyboards
