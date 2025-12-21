# Next Steps - keyrx Development Roadmap

**Generated:** 2025-12-22
**Status:** Active Planning

---

## ✅ Recently Completed

### BaseKeyMapping/KeyMapping Split Finalization
**Completed:** 2025-12-22

**What was done:**
- ✅ Resolved rkyv recursion limitations by keeping BaseKeyMapping/KeyMapping split
- ✅ Added ergonomic helper functions to hide implementation details
- ✅ Updated all parser functions to use helpers
- ✅ Fixed all tests (103 tests passing, 0 failures)
- ✅ Cleaned up unused imports
- ✅ Updated spec documentation (design.md, RECURSIVE_DATA_STRUCTURES_DESIGN.md)

**Technical Outcome:**
- rkyv 0.7 with `validation` feature
- Deterministic serialization confirmed
- Zero-copy deserialization working
- 1-level conditional nesting (users can combine conditions with `when([A, B])`)
- Custom KeyCode discriminants preserved (0x00, 0x100, 0x200)

**Key Files Modified:**
- `keyrx_core/src/config/mappings.rs` - Added helpers, simplified tests
- `keyrx_core/src/config/conditions.rs` - Fixed tests
- `keyrx_compiler/src/parser/functions/*.rs` - Updated to use helpers
- `keyrx_compiler/src/serialize.rs` - Full implementation with validation
- `keyrx_compiler/tests/property_tests.rs` - Comprehensive property tests

---

## 🎯 Immediate Next Steps (Week 1-2)

### Priority 1: Complete core-config-system Spec

**Status:** ~60% complete

**What's Done:**
- ✅ Data structures (BaseKeyMapping, KeyMapping, Condition, ConfigRoot)
- ✅ Binary serialization/deserialization with rkyv
- ✅ Hash verification (SHA256)
- ✅ Import resolver with circular detection
- ✅ Property-based testing framework
- ✅ Fuzzing infrastructure
- ✅ Basic CLI structure

**What's Remaining:**

#### 1.1: Complete Rhai DSL Parser (3-5 days)
**Files:** `keyrx_compiler/src/parser/`

**Tasks:**
- [ ] Implement prefix validation (VK_, MD_, LK_) with user-friendly errors
- [ ] Complete `map()` function with all 3 output types (VK_, MD_, LK_)
- [ ] Implement `tap_hold()` with validation
- [ ] Implement `when()` for conditional mappings
- [ ] Implement `when_not()` for negated conditions
- [ ] Implement helper functions: `with_shift()`, `with_ctrl()`, `with_alt()`, `with_mods()`
- [ ] Implement `device()` function for device selection
- [ ] Add comprehensive error messages for common mistakes

**Acceptance Criteria:**
```rhai
// This should parse successfully
device("*", || {
    map("VK_A", "VK_B");                    // Simple mapping
    map("VK_CapsLock", "MD_00");            // Modifier assignment
    map("VK_ScrollLock", "LK_01");          // Lock assignment

    tap_hold("VK_Space", "VK_Space", "MD_00", 200);  // Tap/hold

    map("VK_1", with_shift("VK_1"));        // Physical modifier output

    when("MD_00", || {                      // Conditional block
        map("VK_H", "VK_Left");
        map("VK_J", "VK_Down");
    });

    when_not("LK_01", || {                  // Negated conditional
        map("VK_K", "VK_Up");
    });
});
```

**Error Handling Examples:**
```
Error: Missing prefix on output key
  --> main.rhai:5:15
   |
 5 |     map("VK_A", "B");
   |                  ^^^ Output must have VK_, MD_, or LK_ prefix
   |
   = help: Use "VK_B" for virtual key output
   = help: Use "MD_XX" for modifier assignment (e.g., "MD_00")
   = help: Use "LK_XX" for lock assignment (e.g., "LK_01")

Error: Invalid modifier ID in tap_hold
  --> main.rhai:8:45
   |
 8 |     tap_hold("VK_Space", "VK_Space", "MD_LShift", 200);
   |                                       ^^^^^^^^^^^ Cannot use physical modifier names in MD_
   |
   = help: Use hex IDs MD_00 through MD_FE
   = note: Physical modifiers (Shift, Ctrl, Alt, Win) are reserved for output only
```

#### 1.2: CLI Polish & Integration Tests (2-3 days)
**Files:** `keyrx_compiler/src/main.rs`, `keyrx_compiler/tests/`

**Tasks:**
- [ ] Complete all 4 CLI subcommands:
  - `compile input.rhai -o output.krx` - Full compilation
  - `verify config.krx` - Validate binary structure
  - `hash config.krx` - Extract embedded hash
  - `parse input.rhai --json` - Parse and show AST (debugging)
- [ ] Add integration tests for complete workflows
- [ ] Test all error scenarios (syntax errors, invalid prefixes, circular imports)
- [ ] Add progress indicators for large compilations
- [ ] Improve error formatting (colors, suggestions, context)

**Example Output:**
```bash
$ keyrx_compiler main.rhai -o config.krx
[INFO] Parsing main.rhai
[INFO] Resolving imports: common/vim-nav.rhai, common/locks.rhai
[INFO] Generating configuration
[INFO] Serializing to binary (rkyv)
[INFO] Computing SHA256 hash: 3a7f8c2e1b9d...
[INFO] Output written to config.krx (12.5 KB)
✓ Compilation successful

$ keyrx_compiler verify config.krx
✓ Magic bytes valid (KRX\n)
✓ Version: 1
✓ SHA256 hash matches: 3a7f8c2e1b9d...
✓ Data size: 12,442 bytes
✓ rkyv structure valid
✓ Verification passed
```

#### 1.3: Documentation & Examples (2-3 days)
**Files:** `docs/DSL_MANUAL.md`, `examples/`, `README.md`

**Tasks:**
- [ ] Write comprehensive DSL manual with all functions
- [ ] Create example configurations:
  - `examples/simple-remap.rhai` - Basic A→B remapping
  - `examples/vim-navigation.rhai` - Vim-style arrow keys with modifiers
  - `examples/dual-function-keys.rhai` - Tap/hold for space (tap=space, hold=ctrl)
  - `examples/multi-device.rhai` - Multiple keyboards with different configs
  - `examples/advanced-layers.rhai` - 255 modifiers showcase
- [ ] Update README with quickstart guide
- [ ] Add troubleshooting section
- [ ] Document all error codes and fixes

**Deliverable:** User can read DSL_MANUAL.md and write their own configurations in 30 minutes.

---

## 🚀 Phase 2: Basic Runtime (Week 3-5)

### Priority 2: Implement basic-key-remapping Spec

**Goal:** Load .krx config and perform simple 1:1 key remapping with <1ms latency.

**Prerequisites:** core-config-system (complete)

**Key Components:**

#### 2.1: Runtime State Management (4-5 days)
**New Files:** `keyrx_daemon/src/state.rs`, `keyrx_daemon/src/event.rs`

**Tasks:**
- [ ] Define `DeviceState` struct:
  - 255-bit modifier state vector (which MD_XX are active)
  - 255-bit lock state vector (which LK_XX are toggled)
  - Current active mappings
  - Tap/hold state machine (future, stub for now)
- [ ] Implement event processing pipeline:
  - Input event → Lookup mapping → Apply modifiers → Output event
- [ ] Add structured logging (JSON format)
- [ ] Implement mock input/output for testing (no OS dependencies yet)

**Acceptance Criteria:**
```rust
// Load config
let config = load_krx("config.krx")?;
let mut state = DeviceState::new(config);

// Process key event (mock)
let input = KeyEvent::Press(KeyCode::A);
let output = state.process_event(input)?;

assert_eq!(output, vec![KeyEvent::Press(KeyCode::B)]);  // A remapped to B
```

#### 2.2: Simple Lookup Implementation (2-3 days)
**New Files:** `keyrx_core/src/lookup.rs`

**Tasks:**
- [ ] Implement HashMap-based lookup (O(log n), acceptable for MVP)
- [ ] Build lookup table from ConfigRoot at load time
- [ ] Handle conditional mappings (check modifier state before applying)
- [ ] Add benchmarks (target: <1ms processing latency)

**Note:** MPHF optimization (O(1) lookup) deferred to Phase 3 (Priority P1).

#### 2.3: Mock Platform Layer (2-3 days)
**New Files:** `keyrx_daemon/src/platform/mod.rs`, `keyrx_daemon/src/platform/mock.rs`

**Tasks:**
- [ ] Define platform traits:
  ```rust
  pub trait InputDevice {
      fn next_event(&mut self) -> Result<KeyEvent, Error>;
      fn grab(&mut self) -> Result<(), Error>;
      fn release(&mut self) -> Result<(), Error>;
  }

  pub trait OutputDevice {
      fn inject_event(&mut self, event: KeyEvent) -> Result<(), Error>;
  }
  ```
- [ ] Implement mock platform for testing
- [ ] Add integration tests with mock devices
- [ ] Verify no key events are dropped or duplicated

**Deliverable:** End-to-end test: Load .krx → Process mock events → Verify correct remapping

---

## 🐧 Phase 3: Linux Platform Integration (Week 6-8)

### Priority 3: Implement linux-evdev-integration Spec

**Goal:** Real keyboard interception and injection on Linux.

**Prerequisites:** basic-key-remapping (complete)

**Key Components:**

#### 3.1: evdev Input Capture (3-4 days)
**New Files:** `keyrx_daemon/src/platform/linux.rs`

**Tasks:**
- [ ] Implement `InputDevice` trait using `evdev` crate
- [ ] Device discovery via `/sys/class/input/`
- [ ] Device identification by serial number pattern matching
- [ ] Exclusive grab with EVIOCGRAB
- [ ] Error handling for permission issues
- [ ] Add udev rules template for non-root usage

#### 3.2: uinput Output Injection (2-3 days)
**Continue:** `keyrx_daemon/src/platform/linux.rs`

**Tasks:**
- [ ] Implement `OutputDevice` trait using `uinput` crate
- [ ] Create virtual keyboard device
- [ ] Event injection with proper timing
- [ ] Handle key repeat correctly
- [ ] Graceful cleanup on daemon shutdown

#### 3.3: Daemon Service (2-3 days)
**New Files:** `keyrx_daemon/src/main.rs`, `keyrx_daemon/src/daemon.rs`

**Tasks:**
- [ ] CLI for daemon: `keyrx_daemon --config config.krx`
- [ ] Signal handling (SIGTERM, SIGINT for graceful shutdown)
- [ ] Logging to syslog or journald
- [ ] systemd service file template
- [ ] Installation script

**Example Usage:**
```bash
# Compile config
keyrx_compiler main.rhai -o config.krx

# Run daemon (requires root or udev rules)
sudo keyrx_daemon --config config.krx

# Output:
# [INFO] Loading config: config.krx (SHA256: 3a7f8c2e...)
# [INFO] Found 1 device: USB Keyboard (pattern: "USB\\VID_04D9*")
# [INFO] Grabbed device: /dev/input/event5
# [INFO] Created virtual device: /dev/input/event10
# [INFO] Daemon running (PID: 1234)
# [INFO] Press Ctrl+C to stop

# Test remapping
# (Press CapsLock, should output Escape)
# (Press A, should output B if configured)
```

**Deliverable:** Working keyboard remapper on Linux with real hardware!

---

## 📊 Success Metrics

### For core-config-system (Phase 1):
- [ ] Users can write .rhai configs following DSL_MANUAL.md
- [ ] Compiler produces deterministic .krx files (same input → same hash)
- [ ] All error messages are actionable with clear suggestions
- [ ] 90%+ code coverage
- [ ] Property tests pass 1000+ iterations
- [ ] Fuzzer runs 60+ seconds without panics

### For basic-key-remapping (Phase 2):
- [ ] <1ms event processing latency (measured)
- [ ] No dropped or duplicated events in tests
- [ ] Mock platform tests pass
- [ ] State machine correctly tracks 255 modifiers + 255 locks

### For linux-evdev-integration (Phase 3):
- [ ] Daemon successfully grabs real keyboard
- [ ] Key remapping works with physical device
- [ ] No events leak to OS
- [ ] Graceful error messages for permission issues
- [ ] systemd service starts without errors
- [ ] User can map CapsLock → Escape end-to-end

---

## 🔮 Future Phases (Weeks 9+)

### Phase 4: Performance Optimization (P1)
- Implement MPHF lookup (O(1) constant time)
- Target: <100μs processing latency
- Benchmark against QMK firmware

### Phase 5: Advanced Features (P1)
- DFA state machine for Tap/Hold behavior
- Deterministic simulation testing
- Retroactive state correction (Permissive Hold mode)

### Phase 6: Extended Modifiers (P2)
- 255 custom modifiers in practice
- Cross-device modifier sharing
- Advanced layer system

### Phase 7: Multi-Platform (P2)
- Windows support (WH_KEYBOARD_LL hooks)
- macOS support (IOKit or DriverKit)
- WASM simulator for web UI

### Phase 8: Web UI (P2)
- React-based configuration editor
- Live simulation without hardware
- Visual layer designer
- WebSocket connection to daemon for real-time updates

---

## 🤔 Decisions Needed

Before starting Phase 1 completion:

1. **Error Message Style:**
   - Use colored output in terminal? (requires `colored` crate)
   - Include suggestions automatically or only with `--help` flag?
   - **Recommendation:** Colored by default, disable with `--no-color`

2. **Import System Scope:**
   - Support relative imports: `import "common/vim.rhai"`?
   - Support absolute imports: `import "/usr/share/keyrx/stdlib/vim.rhai"`?
   - **Recommendation:** Relative only for MVP, absolute in Phase 2

3. **DSL Function Names:**
   - Current: `map()`, `tap_hold()`, `when()`, `when_not()`
   - Alternative: `remap()`, `dual_function()`, `layer()`, `not_when()`?
   - **Recommendation:** Keep current names (match QMK/Kanata conventions)

4. **Platform Priority:**
   - Linux-first (get MVP working end-to-end)?
   - Multi-platform from day 1 (slower initial progress)?
   - **Recommendation:** Linux-first (faster to MVP, validate architecture)

---

## 📝 Recommended Next Action

**START HERE:** Complete Phase 1 (core-config-system)

**Estimated Time:** 7-11 days (1.5-2 weeks)

**Steps:**
1. Implement Rhai DSL parser with all functions (3-5 days)
2. Polish CLI and add integration tests (2-3 days)
3. Write documentation and examples (2-3 days)

**Deliverable:** Users can write .rhai configs, compile to .krx, and verify deterministic output.

**After Phase 1:** Move to Phase 2 (basic-key-remapping) for runtime implementation.

---

## 📚 References

- **Current Spec:** `.spec-workflow/specs/core-config-system/`
- **MVP Candidates:** `.spec-workflow/MVP_CANDIDATES.md`
- **Technical Design:** `RECURSIVE_DATA_STRUCTURES_DESIGN.md`
- **Existing Tests:** `keyrx_compiler/tests/`, `keyrx_core/src/config/`
- **Fuzzing Results:** `keyrx_core/fuzz/FUZZING_RESULTS.md`

Let me know which phase to start with! I recommend Phase 1 completion first.
