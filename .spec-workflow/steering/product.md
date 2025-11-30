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

2. **True Blank Canvas**: The keyboard is treated as a pure grid of buttons with physical positions, not OS-defined key names. Through device discovery, each physical key is mapped by its position (row, column), eliminating assumptions about what a key "should" be. Modifiers are just buttons. Layouts don't exist until you create them.

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

6. **Progressive Complexity**: Simple things should be simple, complex things should be possible. Never overwhelm beginners, never limit experts.

7. **Testable Configs**: Users can write tests for their keyboard configurations. Refactor with confidence.

## Foundational Pillars

### Safe Mode / Emergency Exit

**Problem**: Users fear keyboard remappers because a bad config can lock them out.

**Solution**: A hardcoded, never-remapped emergency exit:

```
Ctrl + Alt + Shift + Escape = Instantly disable all remapping
```

**Guarantees**:
- This combo is NEVER intercepted by KeyRx, regardless of config
- Works even if the engine is stuck or crashed
- Visual indicator (system tray turns red) confirms disabled state
- Same combo re-enables KeyRx
- On crash/panic, keyboard automatically returns to normal

**Why foundational**: Trust is the foundation. Users must never fear losing control.

### True Blank Canvas (Hardware-Level Abstraction)

**Problem**: Traditional remappers inherit OS assumptions about keys. "A" is always "A", "Shift" is always a modifier. This limits what's possible and ties configurations to specific keyboard layouts (ANSI, ISO, JIS).

**Solution**: Treat the keyboard as a pure grid of physical buttons:

```
┌─────────────────────────────────────────────────────────────┐
│ TRADITIONAL VIEW (OS-centric)                               │
│                                                             │
│  [Esc] [F1] [F2] ...  ← Keys have predefined "meaning"     │
│  [Caps] [A ] [S ] ...  ← "Modifier" keys are special       │
└─────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────┐
│ KEYRX VIEW (Hardware-centric)                               │
│                                                             │
│  [0,0] [0,1] [0,2] ...  ← Just buttons with positions      │
│  [3,0] [3,1] [3,2] ...  ← No "special" keys exist          │
└─────────────────────────────────────────────────────────────┘
```

**Device Discovery Flow**:
1. On first connection, detect keyboard by (vendor_id, product_id)
2. If no profile exists, prompt user to discover physical layout
3. User presses keys in order: top-left → right, then next row
4. Each key is registered by physical position: `KEY_0_0`, `KEY_0_1`, etc.
5. Profile is saved per-device for future sessions

**Per-Keyboard Profiles**:
```
~/.config/keyrx/devices/
├── 046d_c52b.json    # Logitech K270 - discovered layout
├── 1234_5678.json    # Custom ortholinear
└── default.json      # Fallback to OS key names
```

**Modifier Keys as Pure Buttons**:
- At hardware level, Shift/Ctrl/Alt send key down/up like any other key
- OS adds "modifier" behavior on top
- KeyRx intercepts BEFORE OS modifier processing
- Result: Shift is just `KEY_4_0`, a button like any other

**Benefits**:
- Works with ANY keyboard layout (ANSI, ISO, JIS, ortholinear, custom)
- No assumptions about key meanings
- Truly layout-agnostic configurations
- Multiple keyboards can have different profiles
- Share configs by position, not by key name

**Why foundational**: This is what makes KeyRx a "blank canvas" rather than a "remap layer on top of OS".

### Progressive Complexity

**Problem**: Power users need full scripting, but beginners just want "CapsLock → Escape".

**Solution**: Three-tier complexity model:

```
┌─────────────────────────────────────────────────────────────┐
│ TIER 1: SIMPLE MODE (Visual, no code)                       │
│                                                             │
│   [CapsLock] ──────────→ [Escape]                          │
│   [+ Add Another Remap]                                     │
├─────────────────────────────────────────────────────────────┤
│ TIER 2: ADVANCED MODE (Declarative syntax)                  │
│                                                             │
│   tap_hold("CapsLock", tap: "Escape", hold: "Ctrl");       │
│   layer("Navigation", "Space + HJKL → Arrows");            │
├─────────────────────────────────────────────────────────────┤
│ TIER 3: EXPERT MODE (Full Rhai scripting)                   │
│                                                             │
│   on_key("CapsLock", |ctx| {                               │
│       if ctx.held_ms > 200 { ctx.activate_mod("Ctrl") }    │
│       else { ctx.emit("Escape") }                          │
│   });                                                       │
└─────────────────────────────────────────────────────────────┘
```

**Implementation**:
- Tier 1 generates Tier 2 code under the hood
- Tier 2 is syntactic sugar for Tier 3
- Users can "eject" to see generated code at any time
- GUI supports all three tiers with appropriate UI

**Why foundational**: Adoption depends on approachability. Retention depends on depth.

### Script Testing Framework

**Problem**: Users can't verify their configs work correctly. Refactoring is scary.

**Solution**: Built-in testing primitives:

```rhai
// In config file or separate test file
#[test]
fn capslock_tap_produces_escape() {
    simulate_tap("CapsLock");
    assert_output("Escape");
}

#[test]
fn capslock_hold_activates_ctrl() {
    simulate_hold("CapsLock", 300);
    assert_modifier_active("Ctrl");
}

#[test]
fn vim_navigation_layer() {
    simulate_hold("Space");
    simulate_tap("H");
    assert_output("Left");
}
```

**CLI Integration**:
```bash
# Run all tests
keyrx test --script config.rhai

# Run specific test
keyrx test --script config.rhai --filter "capslock*"

# Watch mode (re-run on change)
keyrx test --watch --script config.rhai

# Output for AI parsing
keyrx test --script config.rhai --json
```

**Exit Codes**:
- 0: All tests passed
- 1: Test execution error
- 2: Test assertion failed (with diff)
- 3: Test timeout

**Why foundational**:
- Users can refactor with confidence
- AI agents can self-verify their changes
- Community scripts can include tests for trust

### Legendary Backlog (Differentiators to Track)
- Conflict detection with resolution strategies (tap-hold vs swap) before deploy.
- Importers for Karabiner/AutoHotkey/keyd/kanata for frictionless migration.
- Progressive tiers (Simple/Advanced/Expert) with “eject to code” visibility.
- Auto-generated printable cheat sheets (layout + layer legend) from configs.
- Module system (official/community/local) with override parameters.
- Typing analytics with ergonomic suggestions; always-on emergency exit combo.
- Layout awareness: logical-position remaps independent of physical layout selection.
- Config history with undo/restore and diffs.

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

### Phase 2.5: The Brain (Advanced Remapping Engine)
Engine-level primitives for advanced keyboard customization. All timing and behavior parameters are configurable, with the GUI visualizing trade-offs.

**State Management (Layer 1):**
- Key state tracking (which physical keys are currently held)
- Timer system (for timing-based decisions)
- Virtual modifier state (255 user-defined modifiers like Mod_Thumb, Mod_Gaming)
- Layer stack (multiple layouts with priority)

**Decision Primitives (Layer 2):**
- Tap vs Hold detection (configurable timing threshold)
- Simultaneous key detection (for chords/combos)
- Sequence detection (for leader key patterns)
- Interrupt detection (key pressed while another held)

**Action Primitives (Layer 3):**
- Emit key (output press/release)
- Block (suppress key)
- Modifier control (activate/deactivate virtual modifier)
- Layer control (push/pop/toggle layers)
- Macro (emit key sequence with timing)

**Composed Behaviors (Layer 4 - Engine optimized):**
- **Tap-Hold**: Different action for tap vs hold (e.g., CapsLock = Esc tap, Ctrl hold)
- **One-Shot (Sticky)**: Modifier active for next key only
- **Combos**: Simultaneous keys produce different output (A+S → Ctrl)

**Scriptable Behaviors (Layer 4 - Rhai):**
- **Tap-Dance**: Different actions for 1/2/3 taps
- **Leader Key**: Sequence triggers action (Leader→W→S → save)
- **Layer Toggle/Hold/Lock**: Various layer activation modes

**Configuration Philosophy:**
Unlike existing tools with fixed behaviors, KeyRx exposes all timing parameters:
- `tap_timeout_ms`: How long until tap becomes hold (default: 200)
- `combo_timeout_ms`: Window for simultaneous detection (default: 50)
- `hold_delay_ms`: Prevent accidental holds during fast typing (default: 0)
- `eager_tap`: Output tap immediately, correct if becomes hold (default: false)
- `permissive_hold`: Consider hold if interrupted by other key (default: true)

The Flutter GUI visualizes these trade-offs in real-time, showing timing diagrams and prediction of behavior changes.

### Phase 3: The Face (Flutter)
Beautiful GUI with FFI bindings, Visual Layer Editor, and REPL console.

**Key Features:**
- **Config Trade-off Visualizer**: Interactive timing diagrams showing tap/hold thresholds
- **Real-time State Inspector**: See active layers, held modifiers, pending decisions
- **Behavior Simulator**: Test configurations without real keyboard
- **Visual Layer Editor**: Drag-and-drop keymap design

### Phase 4: The Ecosystem
Community sharing via standard library (`std/layouts`, `std/modifiers`), enabling users to build on each other's work.

### Potential Enhancements
- **Mouse/MIDI Support**: Extend beyond keyboard to unified input handling
- **Cloud Sync**: Sync configurations across machines
- **Profile System**: Per-application or per-device profiles
- **Hardware Integration**: Direct firmware communication with programmable keyboards
