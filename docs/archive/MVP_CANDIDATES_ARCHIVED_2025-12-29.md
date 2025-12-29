# MVP Candidates - Prioritized Feature Specs

## Overview

This document outlines the next specs to implement after completing the **ai-dev-foundation**. Each candidate is prioritized based on:
- **Business Value**: Impact on core product functionality
- **Technical Dependencies**: What must exist before this can be built
- **Risk**: Technical complexity and unknowns
- **Effort**: Estimated implementation complexity

## Priority Matrix

| Spec | Business Value | Dependencies | Risk | Effort | Priority |
|------|----------------|--------------|------|--------|----------|
| Core Configuration System | Critical | ai-dev-foundation | Medium | High | **P0** |
| Basic Key Remapping | Critical | Core Config | Low | Medium | **P0** |
| Linux Platform Integration | Critical | Basic Remapping | Medium | High | **P0** |
| MPHF Lookup System | High | Core Config | Medium | Medium | **P1** |
| DFA State Machine | High | MPHF Lookup | High | High | **P1** |
| Extended Modifiers (255) | Medium | DFA State Machine | Low | Medium | **P2** |
| Web UI Simulator | Medium | Core Config | Medium | High | **P2** |
| Windows Platform Integration | Medium | Basic Remapping | Medium | High | **P2** |

---

## P0 Specs (MVP - Must Have)

### 1. Core Configuration System

**Spec Name**: `core-config-system`

**Business Goal**: Enable users to write Rhai scripts that compile to deterministic .krx binary configuration files.

**Why P0**:
- Foundation for all other features
- Without this, no configuration can be loaded or executed
- Blocks all downstream work

**Key Features**:
- Rhai script parser and evaluator
- Configuration DSL (device selection, key mapping definitions)
- Binary serialization with rkyv (deterministic, zero-copy)
- Hash-based verification (SSOT mechanism)
- CLI compiler: `keyrx_compiler input.rhai -o output.krx`

**Acceptance Criteria**:
- Parse Rhai script with device() and map() functions
- Generate .krx binary with rkyv serialization
- Verify binary deserialization is deterministic (same input → same hash)
- Compiler outputs errors for invalid scripts
- Support single entry point with imports (as per structure.md)

**Technical Design Highlights**:
- `keyrx_compiler/src/parser.rs`: Rhai AST evaluation
- `keyrx_compiler/src/serialize.rs`: rkyv binary output
- `keyrx_core/src/config.rs`: Configuration data structures

**Example Usage**:
```bash
# Compile Rhai to binary
keyrx_compiler main.rhai -o config.krx

# Verify compilation
keyrx_compiler --verify config.krx
# Output: SHA256: abc123... (deterministic)
```

**Dependencies**: ai-dev-foundation (workspace, scripts, CI/CD)

**Effort Estimate**: 2-3 weeks

---

### 2. Basic Key Remapping

**Spec Name**: `basic-key-remapping`

**Business Goal**: Enable simple 1:1 key remapping (e.g., CapsLock → Escape) with <1ms latency.

**Why P0**:
- Core value proposition of the product
- Validates end-to-end architecture (config → lookup → inject)
- Required for all advanced features (modifiers, layers, tap/hold)

**Key Features**:
- Simple 1:1 key mapping (no modifiers, no tap/hold yet)
- Linear lookup (HashMap-based, not MPHF yet)
- Input event capture (abstract interface for platform layer)
- Output event injection (abstract interface for platform layer)
- Basic logging with structured JSON

**Acceptance Criteria**:
- Load .krx config file
- Map one key to another (e.g., `map(Key::CapsLock, Key::Escape)`)
- Inject remapped key events
- <1ms processing latency (measured with benchmarks)
- No key repeat issues
- Works with mock input/output (for testing without OS integration)

**Technical Design Highlights**:
- `keyrx_core/src/lookup.rs`: HashMap<KeyCode, KeyCode>
- `keyrx_core/src/state.rs`: Basic event processor
- `keyrx_daemon/src/platform/mod.rs`: Abstract input/output traits
- Mock implementation for testing

**Example Configuration**:
```rhai
// Simple 1:1 remapping
let kbd = device("USB\\VID_AAAA&PID_1111\\SERIAL_ABC");

kbd.map(Key::CapsLock, Key::Escape);
kbd.map(Key::A, Key::B);  // A → B
```

**Dependencies**: core-config-system (load .krx files)

**Effort Estimate**: 1-2 weeks

---

### 3. Linux Platform Integration

**Spec Name**: `linux-evdev-integration`

**Business Goal**: Enable real keyboard interception and injection on Linux using evdev/uinput.

**Why P0**:
- Without platform integration, the system only works in tests
- Linux is the primary development/testing platform
- Required to validate real-world latency and behavior

**Key Features**:
- evdev input capture with EVIOCGRAB (exclusive device access)
- uinput output injection
- Device identification by serial number (via /sys/class/input/)
- Permission handling (require root or udev rules)
- Graceful error handling for missing permissions

**Acceptance Criteria**:
- Capture real keyboard events via evdev
- Grab device exclusively (no events leak to OS)
- Inject remapped events via uinput
- Identify devices by USB serial number
- Daemon starts without errors on valid config
- Clear error messages for permission issues
- Release device gracefully on daemon shutdown

**Technical Design Highlights**:
- `keyrx_daemon/src/platform/linux.rs`: evdev + uinput implementation
- Implement input/output traits from basic-key-remapping
- Use `evdev` and `uinput` crates
- Serial number lookup via sysfs

**Example Daemon Launch**:
```bash
# Run as root (or with udev rules)
sudo ./keyrx_daemon --config main.krx

# Output:
# [INFO] Found device: Keyboard ABC (USB\VID_AAAA&PID_1111\SERIAL_ABC)
# [INFO] Grabbed device: /dev/input/event5
# [INFO] Created virtual device: /dev/input/event10
# [INFO] Daemon running (PID: 12345)
```

**Dependencies**: basic-key-remapping (remapping logic)

**Effort Estimate**: 2-3 weeks

---

## P1 Specs (Enhanced MVP - Should Have)

### 4. MPHF Lookup System

**Spec Name**: `mphf-key-lookup`

**Business Goal**: Achieve O(1) constant-time key lookup using Minimal Perfect Hash Functions (MPHF).

**Why P1**:
- Performance optimization (critical for <100μs target)
- Enables 255 modifiers + 255 locks without lookup degradation
- Proves "firmware-class performance" claim

**Key Features**:
- Generate MPHF with boomphf crate at compile-time
- Replace HashMap with MPHF-based lookup
- Benchmark comparison: HashMap vs MPHF
- Compiler embeds MPHF table in .krx binary

**Acceptance Criteria**:
- MPHF generated from key set during compilation
- Lookup time <100ns (measured with Criterion)
- Binary size increase <10KB for typical configs
- Backwards compatible with basic-key-remapping

**Technical Design Highlights**:
- `keyrx_compiler/src/mphf_gen.rs`: boomphf table generation
- `keyrx_core/src/lookup.rs`: MPHF-based lookup implementation
- Embed MPHF table in rkyv-serialized config

**Benchmark Target**:
```
Key Lookup Benchmark:
  HashMap:  ~50ns per lookup
  MPHF:     ~20ns per lookup (2.5x faster)
```

**Dependencies**: core-config-system, basic-key-remapping

**Effort Estimate**: 1-2 weeks

---

### 5. DFA State Machine (Tap/Hold)

**Spec Name**: `dfa-tap-hold`

**Business Goal**: Enable Tap/Hold behavior (e.g., Tap for Escape, Hold for Control) with deterministic state transitions.

**Why P1**:
- Flagship feature (QMK-style Tap/Hold in software)
- Differentiator from simple remappers
- Foundation for advanced features (layers, combos)

**Key Features**:
- Deterministic Finite Automaton (DFA) for state management
- States: Pending → Held → Tapped → Released
- Configurable timeouts (tap threshold, hold threshold)
- Retroactive state correction (Permissive Hold mode)
- Virtual clock for deterministic testing

**Acceptance Criteria**:
- Tap behavior: Press & release <200ms → output tap action
- Hold behavior: Press >200ms → output hold action
- Deterministic simulation: same input sequence → same output
- No race conditions (single-threaded event processing)
- Property-based testing with proptest (100K+ random sequences)

**Technical Design Highlights**:
- `keyrx_core/src/dfa.rs`: State machine implementation
- `keyrx_core/src/simulator.rs`: Deterministic Simulation Testing (DST)
- Virtual clock abstraction (no wall-clock dependencies in core)

**Example Configuration**:
```rhai
// Tap for Escape, Hold for Control
kbd.tap_hold(
    Key::CapsLock,
    on_tap: Key::Escape,
    on_hold: Modifier::Ctrl,
    tap_threshold_ms: 200
);
```

**Dependencies**: mphf-key-lookup, basic-key-remapping

**Effort Estimate**: 3-4 weeks

---

## P2 Specs (Future Enhancements - Nice to Have)

### 6. Extended Modifiers (255)

**Spec Name**: `extended-modifiers`

**Business Goal**: Support 255 custom modifiers + 255 custom lock keys (beyond standard 8 modifiers).

**Why P2**:
- Unique differentiator (no other software supports this)
- Enables power users to create complex workflows
- Requires DFA + MPHF to be performant

**Key Features**:
- 255-bit modifier state vector (keyrx_core/src/state.rs)
- 255-bit lock state vector
- Cross-device modifier sharing (global state)
- Modifier conflict resolution

**Acceptance Criteria**:
- Define custom modifiers in Rhai (e.g., `Modifier::Custom(42)`)
- Conditional key output based on modifier state
- Cross-device state sharing (Shift on Keyboard A affects Keyboard B)
- State vector serialization for debugging

**Example Configuration**:
```rhai
// Define custom modifier
let vim_mode = Modifier::Custom(1);

kbd.map(Key::Space, vim_mode);  // Space acts as modifier

kbd.conditional(
    Key::H,
    when_held: [vim_mode],
    output: Key::Left
);
```

**Dependencies**: dfa-tap-hold, mphf-key-lookup

**Effort Estimate**: 2-3 weeks

---

### 7. Web UI Simulator

**Spec Name**: `web-ui-simulator`

**Business Goal**: Provide browser-based WASM simulator for testing configurations without hardware.

**Why P2**:
- Enables configuration testing without device
- Validates WASM compatibility of keyrx_core
- Edit-and-preview workflow for users

**Key Features**:
- React 18+ UI with TypeScript
- WASM-compiled keyrx_core
- Virtual keyboard input (browser key events)
- State visualization (current layer, active modifiers)
- Live configuration editing with Rhai Monaco editor
- WebSocket connection to daemon for live updates (optional)

**Acceptance Criteria**:
- Load .krx config in browser
- Simulate key presses (click virtual keyboard)
- Display remapped output in real-time
- Visualize DFA state transitions
- Export .krx config from browser

**Technical Design Highlights**:
- `keyrx_ui/`: React + Vite + TypeScript
- `keyrx_core/`: Compile to WASM with wasm-pack
- `keyrx_daemon/src/web/`: Serve static UI files
- WebSocket API for daemon ↔ UI communication

**Dependencies**: core-config-system, dfa-tap-hold (for state visualization)

**Effort Estimate**: 3-4 weeks

---

### 8. Windows Platform Integration

**Spec Name**: `windows-hooks-integration`

**Business Goal**: Enable keyboard interception on Windows using Low-Level Hooks.

**Why P2**:
- Expands platform support (large user base)
- Validates cross-platform architecture
- Not critical for MVP (Linux-first strategy)

**Key Features**:
- WH_KEYBOARD_LL hooks for input capture
- SendInput for output injection
- Raw Input API for device identification
- Service mode (run as Windows service)

**Acceptance Criteria**:
- Capture keyboard events via hooks
- Inject remapped events via SendInput
- Identify devices by serial number (Raw Input)
- Run as Windows service or console app
- Handle UAC restrictions gracefully

**Dependencies**: basic-key-remapping

**Effort Estimate**: 2-3 weeks

---

## Recommended MVP Roadmap

### Phase 1: Core Foundation (4-6 weeks)
1. ✅ **ai-dev-foundation** - COMPLETED
2. **core-config-system** - Rhai → .krx compiler
3. **basic-key-remapping** - Simple 1:1 remapping
4. **linux-evdev-integration** - Real device support

**Deliverable**: Working prototype on Linux with simple remapping

---

### Phase 2: Performance & Advanced Features (5-7 weeks)
5. **mphf-key-lookup** - O(1) lookup optimization
6. **dfa-tap-hold** - Tap/Hold state machine

**Deliverable**: QMK-competitive feature set with sub-millisecond latency

---

### Phase 3: Enhancements & Polish (5-7 weeks)
7. **extended-modifiers** - 255 modifiers/locks
8. **web-ui-simulator** - Browser-based testing
9. **windows-hooks-integration** - Windows support

**Deliverable**: Production-ready v1.0 with multi-platform support

---

## Selection Criteria for Next Spec

**Recommended: Start with `core-config-system` (P0)**

**Rationale**:
- Blocks all other features (critical path)
- Establishes Rhai DSL and .krx format
- Validates deterministic serialization (SSOT)
- Enables testing infrastructure (load configs in tests)
- Manageable complexity (no platform-specific code)

**Success Metrics**:
- Compiler can parse Rhai and generate .krx
- Binary deserialization is deterministic (hash verification)
- Tests can load .krx configs programmatically
- Documentation and examples for DSL

**Next After core-config-system**: `basic-key-remapping` → `linux-evdev-integration`

This gives you a working end-to-end system on Linux within 6-8 weeks.

---

## Questions for Prioritization

Before starting, clarify:
1. **Platform priority**: Linux-first or multi-platform from day 1?
   - Recommendation: Linux-first (simpler, faster MVP)
2. **Performance target**: Is <1ms acceptable for MVP, or must we hit <100μs immediately?
   - Recommendation: <1ms for MVP (HashMap), optimize to <100μs in Phase 2 (MPHF)
3. **Feature scope**: Simple remapping only, or Tap/Hold in MVP?
   - Recommendation: Simple remapping in MVP, Tap/Hold in Phase 2

Let me know which spec to start with, and I'll create the requirements document!
