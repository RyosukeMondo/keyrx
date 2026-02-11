# WASM Simulation Guide

## Introduction

KeyRX's **WASM Simulation** feature enables you to test keyboard remapping configurations directly in your browser without reloading the daemon or using real keyboard input. This provides instant, deterministic feedback for configuration development, making it easier to perfect tap-hold timings, layer switches, modifier combinations, and macro sequences.

### Key Benefits

- **Instant Testing**: No daemon reload required - test configurations in milliseconds
- **Deterministic Results**: Same event sequence always produces identical output
- **Visual Feedback**: See exactly how the DFA state machine processes each event
- **Performance Insights**: Identify bottlenecks with microsecond-precision latency stats
- **Safe Experimentation**: Test configurations without affecting your live keyboard setup

### Why WASM Simulation?

Traditional keyboard remapping testing requires:
1. Edit Rhai configuration
2. Compile to .krx binary
3. Reload daemon
4. Test with real keyboard input
5. Repeat for each change

With WASM Simulation:
1. Edit configuration in browser
2. Click "Test Configuration"
3. See results instantly

This **100x faster feedback loop** dramatically accelerates configuration development.

## Getting Started

### Opening the Simulator

1. Launch the KeyRX web interface (default: http://localhost:8080)
2. Click **"Simulator"** in the navigation menu
3. The simulator panel will load with the configuration loader

### Quick Start Tutorial

This 5-minute tutorial will help you run your first simulation.

#### Step 1: Load a Configuration

**Option A: Paste Rhai Configuration**

```rhai
// Example: Simple tap-hold configuration
let config = #{
    tap_hold_threshold: 200_000, // 200ms in microseconds
    keys: #{
        "A": #{
            tap: "A",
            hold: "LShift"
        }
    }
};
```

1. Paste your Rhai configuration into the **Config Loader** textarea
2. Click **"Load Configuration"**
3. Wait for "Configuration loaded successfully" message

**Option B: Upload .rhai File**

1. Click **"Choose File"** in the Config Loader
2. Select your .rhai file (max 1MB)
3. The configuration will load automatically

**Option C: Upload .krx Binary**

1. Click **"Choose File"** and select a pre-compiled .krx file
2. The binary will be deserialized using zero-copy validation

#### Step 2: Select a Built-in Scenario

The simulator includes pre-built scenarios for common patterns:

1. Open the **Scenario Selector** dropdown
2. Choose **"Tap-Hold Under Threshold"**
3. Read the scenario description:
   ```
   Simulates a key press and release within 200ms threshold
   to test tap behavior. Expects output: A press → A release.
   ```
4. Click **"Run Scenario"**

#### Step 3: View Results

The **Simulation Results** timeline displays:

```
Timeline (horizontally scrollable):

0μs      100μs     200μs     300μs
●────────●─────────●─────────●
^        ^         ^         ^
A↓       StateChg  A↑        Complete
(blue)   (orange)  (green)
```

**Legend:**
- **●** Event marker
- **Blue** = Input event (key press/release)
- **Orange** = State change (modifier/lock/layer)
- **Green** = Output event
- **Red** = Mismatch between input and output

**Hover over any event** to see:
```
┌─────────────────────────────┐
│ Timestamp: 100μs            │
│ Input: A press              │
│ Output: A press             │
│ Active Modifiers: []        │
│ Active Locks: []            │
│ Active Layer: base          │
└─────────────────────────────┘
```

#### Step 4: Check Performance

The **Latency Stats** panel shows:

```
┌───────────────────────────┐
│ Performance Metrics       │
├───────────────────────────┤
│ Min Latency:    12μs      │
│ Avg Latency:    45μs      │
│ Max Latency:    89μs  ✓   │
│ P95 Latency:    78μs      │
│ P99 Latency:    85μs      │
└───────────────────────────┘

✓ All events processed in <1ms
```

**Warning Thresholds:**
- **Red highlight** if max latency >5ms
- **Green checkmark** if all events <1ms

## Built-in Test Scenarios

The simulator includes 4 pre-built scenarios for common remapping patterns.

### 1. Tap-Hold Under Threshold

**Purpose:** Test tap behavior when key is pressed and released quickly.

**Event Sequence:**
```
0μs:     A press
150μs:   A release  (within 200ms threshold)
```

**Expected Output:**
```
0μs:     A press
150μs:   A release
```

**Use Case:** Verify tap-hold key acts as normal key when tapped quickly.

---

### 2. Tap-Hold Over Threshold

**Purpose:** Test hold behavior when key is pressed beyond threshold.

**Event Sequence:**
```
0μs:     A press
250μs:   A release  (beyond 200ms threshold)
```

**Expected Output:**
```
0μs:     LShift press   (hold action activated)
250μs:   LShift release
```

**Use Case:** Verify tap-hold key switches to hold action after threshold.

---

### 3. Layer Switch

**Purpose:** Test layer activation and key remapping on different layers.

**Event Sequence:**
```
0μs:     Space press     (layer switch key)
100μs:   A press         (while layer active)
200μs:   A release
300μs:   Space release   (deactivate layer)
```

**Expected Output:**
```
0μs:     Layer "symbols" activated
100μs:   "1" press       (A remapped to 1 on symbols layer)
200μs:   "1" release
300μs:   Layer "base" activated
```

**Use Case:** Verify layer switching and layer-specific remapping.

---

### 4. Modifier Combination

**Purpose:** Test modifier key combinations (e.g., Shift+Ctrl+A).

**Event Sequence:**
```
0μs:     LShift press
50μs:    LCtrl press
100μs:   A press
150μs:   A release
200μs:   LCtrl release
250μs:   LShift release
```

**Expected Output:**
```
0μs:     LShift press
50μs:    LCtrl press
100μs:   A press        (with both modifiers active)
150μs:   A release
200μs:   LCtrl release
250μs:   LShift release
```

**Use Case:** Verify modifier stacking and correct event ordering.

## Creating Custom Event Sequences

For precise testing, create custom event sequences in the **Event Sequence Editor**.

### Event Sequence Format

An event sequence is a list of keyboard events with microsecond-precision timestamps:

```typescript
interface EventSequence {
  events: KeyEvent[];
}

interface KeyEvent {
  timestamp: number;    // Microseconds since sequence start
  key_code: number;     // Linux evdev key code (e.g., 30 = KEY_A)
  event_type: "press" | "release";
}
```

### Common Key Codes

| Key | Code | Key | Code | Key | Code |
|-----|------|-----|------|-----|------|
| A | 30 | J | 36 | S | 31 |
| B | 48 | K | 37 | T | 20 |
| C | 46 | L | 38 | U | 22 |
| D | 32 | M | 50 | V | 47 |
| E | 18 | N | 49 | W | 17 |
| F | 33 | O | 24 | X | 45 |
| G | 34 | P | 25 | Y | 21 |
| H | 35 | Q | 16 | Z | 44 |
| I | 23 | R | 19 | Space | 57 |

| Modifier | Code | Modifier | Code |
|----------|------|----------|------|
| LShift | 42 | RShift | 54 |
| LCtrl | 29 | RCtrl | 97 |
| LAlt | 56 | RAlt | 100 |
| LMeta | 125 | RMeta | 126 |

Full key code list: [Linux Input Event Codes](https://github.com/torvalds/linux/blob/master/include/uapi/linux/input-event-codes.h)

### Example: Custom Tap-Hold Sequence

Test a 175ms tap (just under 200ms threshold):

1. Click **"Add Event"** in Event Sequence Editor
2. Enter event details:
   - **Timestamp:** 0
   - **Key Code:** 30 (A)
   - **Event Type:** press
3. Click **"Add Event"** again
4. Enter release event:
   - **Timestamp:** 175000 (175ms in microseconds)
   - **Key Code:** 30 (A)
   - **Event Type:** release
5. Click **"Simulate Custom Sequence"**

### Example: Double-Tap Detection

Test rapid double-tap (two taps within 500ms):

```
Event 1: 0μs        - A press
Event 2: 80μs       - A release
Event 3: 200μs      - A press   (second tap)
Event 4: 280μs      - A release
```

**Use Case:** Verify double-tap detection triggers correctly.

### Validation Rules

The Event Sequence Editor validates:

- **Timestamps:** Must be non-negative integers
- **Ordering:** Timestamps must be in ascending order
- **Key Codes:** Must be valid Linux evdev codes (0-255)
- **Event Types:** Must be "press" or "release"

Validation errors appear inline with red highlighting.

## Advanced Features

### Exporting/Importing Custom Sequences

**Export Sequence (Future Feature):**
1. Create a custom event sequence
2. Click **"Export Sequence"**
3. Save JSON file for reuse

**Import Sequence (Future Feature):**
1. Click **"Import Sequence"**
2. Select JSON file
3. Sequence loads into editor

### Keyboard Shortcuts (Future Feature)

Speed up event creation with keyboard shortcuts:

- **Ctrl+Enter:** Add event quickly
- **Delete:** Remove selected event
- **Arrow Keys:** Adjust timestamps (±10μs per press)

### Timeline Visualization Controls (Future Feature)

**Toggle Options:**
- **"Show State Changes"** - Hide/show modifier and layer changes
- **"Show Differences Only"** - Filter timeline to input/output mismatches

**Performance Optimization:**
- Timeline uses virtualization for 1000+ events
- Smooth 60fps scrolling guaranteed

## Integration with Configuration Editor

### "Test Configuration" Button

When editing configurations in the Config Editor:

1. Make changes to your Rhai configuration
2. Click **"Test Configuration"** (next to Save button)
3. Browser navigates to Simulator with config pre-loaded
4. Run scenarios or custom sequences
5. Return to editor with original state preserved

**Configuration Flow:**
```
Config Editor → Test Button → Simulator Panel
                                    ↓
                            Auto-load config
                                    ↓
                            Run simulation
                                    ↓
                            View results
```

### Passing Configurations

Configurations are passed via:
- **URL Parameters:** `?config=<encoded-rhai-source>`
- **Session Storage:** Fallback for large configs (>2KB)

The simulator auto-loads and clears the storage after loading to prevent stale data.

## Understanding Simulation Output

### Timeline Events

The timeline displays 4 event types:

1. **Input Events** (Blue)
   - User-generated key press/release events
   - Positioned by timestamp

2. **State Changes** (Orange)
   - Modifier activation/deactivation
   - Lock key state changes (CapsLock, NumLock, etc.)
   - Layer switches

3. **Output Events** (Green)
   - Remapped key events sent to system
   - May differ from input (that's the point of remapping!)

4. **Mismatches** (Red)
   - Highlights where output differs from input
   - Useful for debugging unexpected behavior

### State Inspection

Hover over any event to see full state:

```
┌────────────────────────────────────┐
│ Timestamp: 250μs                   │
│ Input: A press                     │
│ Output: LShift press               │
│                                    │
│ Active Modifiers: [MD_00]          │
│ Active Locks: []                   │
│ Active Layer: base                 │
│                                    │
│ Raw State Vector:                  │
│ 0x0000000000000001...              │
└────────────────────────────────────┘
```

**Fields:**
- **Timestamp:** Event time in microseconds
- **Input:** Original keyboard event
- **Output:** Remapped event (empty if suppressed)
- **Active Modifiers:** List of active modifier IDs
- **Active Locks:** List of active lock keys
- **Active Layer:** Current layer name
- **Raw State Vector:** 255-bit state for advanced debugging

### Latency Statistics

The simulator tracks per-event processing latency:

```
┌────────────────────────────────────┐
│ Latency Statistics                 │
├────────────────────────────────────┤
│ Min:     12μs   (fastest event)    │
│ Avg:     45μs   (mean latency)     │
│ Max:     89μs   (slowest event)    │
│ P95:     78μs   (95th percentile)  │
│ P99:     85μs   (99th percentile)  │
└────────────────────────────────────┘
```

**Interpreting Results:**

- **Min/Avg/Max:** Basic statistics
- **P95:** 95% of events processed faster than this
- **P99:** 99% of events processed faster than this

**Performance Targets:**

- ✅ **<1ms (1000μs):** Excellent - imperceptible latency
- ⚠️ **1-5ms:** Acceptable - may notice slight delay
- ❌ **>5ms:** Warning - configuration may have performance issues

**Common Performance Issues:**

1. **Large Layer Count (>100 layers)**
   - Symptom: High avg/p95 latency
   - Solution: Reduce layer count or simplify layer structure

2. **Complex DFA (>1000 states)**
   - Symptom: High max latency spikes
   - Solution: Simplify key mappings or reduce modifier combinations

3. **Heavy Macro Processing**
   - Symptom: High latency on specific events
   - Solution: Optimize macro sequences or reduce macro count

## WASM API Reference (Advanced)

For developers integrating the simulator or building custom tools, the WASM module exposes a low-level JavaScript API.

### WasmCore Class

```typescript
class WasmCore {
  /**
   * Initialize WASM module.
   * Must be called before any other methods.
   */
  static async init(): Promise<void>;

  /**
   * Load Rhai configuration from source code.
   * Compiles to in-memory DFA.
   *
   * @param rhaiSource - Rhai configuration source code
   * @returns ConfigHandle for use in simulate()
   * @throws Error with line numbers if parse fails
   */
  static async loadConfig(rhaiSource: string): Promise<ConfigHandle>;

  /**
   * Load pre-compiled .krx binary.
   * Uses zero-copy rkyv deserialization.
   *
   * @param krxBinary - .krx file contents as Uint8Array
   * @returns ConfigHandle for use in simulate()
   * @throws Error if deserialization fails
   */
  static async loadKrx(krxBinary: Uint8Array): Promise<ConfigHandle>;

  /**
   * Simulate event sequence with loaded configuration.
   *
   * @param config - ConfigHandle from loadConfig() or loadKrx()
   * @param eventSequence - JSON event sequence
   * @returns Simulation results with timeline and latency stats
   * @throws Error if ConfigHandle invalid or simulation fails
   */
  static async simulate(
    config: ConfigHandle,
    eventSequence: EventSequence
  ): Promise<SimulationResult>;

  /**
   * Get current simulation state.
   * Returns state in same format as daemon IPC.
   *
   * @param config - ConfigHandle from loadConfig() or loadKrx()
   * @returns Current DaemonState
   * @throws Error if ConfigHandle invalid
   */
  static async getState(config: ConfigHandle): Promise<DaemonState>;
}
```

### Type Definitions

```typescript
/** Opaque reference to loaded configuration */
type ConfigHandle = number;

/** Event sequence input */
interface EventSequence {
  events: KeyEvent[];
}

/** Single keyboard event */
interface KeyEvent {
  timestamp: number;       // Microseconds since sequence start
  key_code: number;        // Linux evdev key code (0-255)
  event_type: "press" | "release";
}

/** Simulation output */
interface SimulationResult {
  timeline: TimelineEvent[];
  latency_stats: LatencyStats;
}

/** Timeline event (input, state change, or output) */
interface TimelineEvent {
  timestamp: number;
  event_type: "input" | "state_change" | "output";

  // For input/output events
  key_event?: KeyEvent;

  // For state change events
  state_change?: {
    modifiers?: number[];    // Active modifier IDs
    locks?: number[];        // Active lock IDs
    layer?: string;          // Active layer name
  };
}

/** Latency statistics */
interface LatencyStats {
  min_us: number;    // Minimum latency (microseconds)
  avg_us: number;    // Average latency (microseconds)
  max_us: number;    // Maximum latency (microseconds)
  p95_us: number;    // 95th percentile (microseconds)
  p99_us: number;    // 99th percentile (microseconds)
}

/** Current daemon state (matches daemon IPC format) */
interface DaemonState {
  active_modifiers: number[];
  active_locks: number[];
  active_layer: string;
  raw_state: number[];    // 255-bit state vector
}
```

### Example Usage

```typescript
import { WasmCore } from '@/wasm/core';

// Initialize WASM module
await WasmCore.init();

// Load Rhai configuration
const rhaiSource = `
  let config = #{
    tap_hold_threshold: 200_000,
    keys: #{ "A": #{ tap: "A", hold: "LShift" } }
  };
`;
const config = await WasmCore.loadConfig(rhaiSource);

// Create event sequence
const sequence: EventSequence = {
  events: [
    { timestamp: 0, key_code: 30, event_type: "press" },
    { timestamp: 150000, key_code: 30, event_type: "release" }
  ]
};

// Run simulation
const result = await WasmCore.simulate(config, sequence);

// Display results
console.log("Timeline:", result.timeline);
console.log("Latency Stats:", result.latency_stats);

// Get current state
const state = await WasmCore.getState(config);
console.log("Active Layer:", state.active_layer);
```

## Troubleshooting

### Common Issues

#### 1. WASM Module Fails to Load

**Symptom:** "Failed to initialize WASM module" error

**Causes:**
- Browser doesn't support WASM
- WASM file not found (404 error)
- CORS policy blocking WASM file

**Solutions:**
- Use modern browser (Chrome 90+, Firefox 88+, Safari 15+)
- Check browser console for network errors
- Ensure WASM file served with correct MIME type: `application/wasm`
- Check vite.config.ts has correct WASM plugin configuration

---

#### 2. Configuration Parse Errors

**Symptom:** "Parse error at line X" when loading config

**Causes:**
- Syntax error in Rhai code
- Missing semicolons or braces
- Invalid key codes or threshold values

**Solutions:**
- Check error message for line number
- Validate Rhai syntax (use Config Editor with validation)
- Common mistakes:
  ```rhai
  // Wrong: Missing semicolon
  let config = #{ threshold: 200 }

  // Correct:
  let config = #{ threshold: 200 };
  ```

---

#### 3. Simulation Produces Unexpected Results

**Symptom:** Output events don't match expected behavior

**Debugging Steps:**
1. Check timeline for state changes
2. Hover over events to inspect state
3. Verify threshold values in microseconds (not milliseconds)
4. Ensure event timestamps in correct order
5. Check for modifier/layer activation issues

**Common Mistakes:**
```
Wrong: 200ms = 200 (should be 200000 microseconds)
Wrong: Events out of order (press at 100μs, release at 50μs)
Wrong: Missing layer activation event before layer-specific key
```

---

#### 4. Performance Warnings

**Symptom:** Red highlight on max latency (>5ms)

**Causes:**
- Too many layers (>100)
- Complex DFA state machine (>1000 states)
- Large modifier combinations

**Solutions:**
- Reduce layer count
- Simplify key mappings
- Profile configuration with different scenarios
- Consider splitting complex configs into multiple profiles

---

#### 5. Browser Compatibility Issues

**Symptom:** Feature works in Chrome but not Safari/Firefox

**Known Issues:**
- Safari 15 requires top-level await flag enabled
- Firefox may have stricter CORS policies

**Solutions:**
- Update to latest browser version
- Check browser console for specific errors
- Report browser-specific issues to KeyRX GitHub

## Performance Considerations

### WASM Module Size

**Current Optimized Size:**
- Uncompressed: ~1.7MB
- Gzipped: ~510KB

**Optimization Techniques Used:**
- Cargo release profile: `opt-level = "z"` (optimize for size)
- Link-Time Optimization (LTO)
- wasm-opt with `-Oz` flag (aggressive size optimization)

**Impact:**
- Initial load: ~500ms on broadband connection
- Module cached by browser after first load
- Subsequent loads: <50ms

### Simulation Performance

**Performance Targets:**

| Metric | Target | Typical | Notes |
|--------|--------|---------|-------|
| 100-event sequence | <10ms | ~5ms | Well within target |
| 1000-event sequence | <100ms | ~45ms | Excellent performance |
| Config load (Rhai) | <200ms | ~80ms | Fast compilation |
| Config load (.krx) | <50ms | ~15ms | Zero-copy deserialization |

**Memory Usage:**
- WASM module: ~20MB baseline
- 1000-event simulation: ~5MB additional
- Total: <100MB (well within browser limits)

### Optimization Tips

1. **Use .krx binaries for large configs**
   - Pre-compiled .krx loads 5x faster than Rhai source
   - Reduced parse overhead

2. **Limit event sequence size**
   - <1000 events recommended for real-time feedback
   - Larger sequences still work but may take longer

3. **Browser caching**
   - WASM module cached automatically
   - Clear cache only when debugging module issues

## Frequently Asked Questions

### General Questions

**Q: Does simulation require the daemon to be running?**

A: No. Simulation runs entirely in the browser using WASM. The daemon is not involved.

**Q: Can I test configurations on platforms without KeyRX daemon support?**

A: Yes! WASM simulation works on Windows, macOS, and Linux, even if the daemon only supports Linux.

**Q: Are simulation results identical to daemon behavior?**

A: Yes. The WASM module uses the exact same keyrx_core engine as the daemon, ensuring 100% behavioral parity.

**Q: Can I share simulation scenarios with others?**

A: Export/import features are planned. Currently, you can manually share event sequence JSON.

### Technical Questions

**Q: What's the difference between load_config() and load_krx()?**

A:
- `load_config()`: Parses Rhai source and compiles to DFA (slower, for development)
- `load_krx()`: Deserializes pre-compiled binary (faster, for production configs)

**Q: Why microsecond timestamps instead of milliseconds?**

A: Keyboard remapping requires microsecond precision for accurate tap-hold detection and low-latency processing. Millisecond precision is insufficient.

**Q: Can I modify the WASM module source code?**

A: Yes. The WASM module is built from keyrx_core/src/wasm.rs. Rebuild with `npm run build:wasm`.

**Q: How do I debug WASM panics?**

A: WASM panics are caught and converted to JavaScript exceptions with stack traces. Check browser console for details.

### Configuration Questions

**Q: My tap-hold doesn't work as expected. How do I debug?**

A:
1. Run "Tap-Hold Under Threshold" and "Tap-Hold Over Threshold" scenarios
2. Check timeline for state changes at threshold boundary
3. Verify threshold value in microseconds (200ms = 200000μs)
4. Hover over events to inspect modifier activation timing

**Q: How do I test layer switching?**

A: Use the "Layer Switch" scenario or create a custom sequence with layer activation key held while pressing layer-specific keys.

**Q: Can I test macros with simulation?**

A: Yes. Create event sequences with precise timing for macro playback. Future versions will support macro recording.

## Additional Resources

### Documentation

- [Configuration Validation Guide](./config-validation.md) - Real-time validation in Config Editor
- [User Guide](./user-guide/) - KeyRX usage tutorials
- [Development Guide](./development/) - Contributing to KeyRX

### Code References

- [WASM Module Source](../keyrx_core/src/wasm.rs) - Rust WASM bindings
- [TypeScript API Wrapper](../keyrx_ui/src/wasm/core.ts) - JavaScript API
- [React Simulator Components](../keyrx_ui/src/components/Simulator/) - UI implementation

### Examples

Example configurations with test scenarios:

1. **Basic Tap-Hold** (`examples/tap-hold.rhai`)
2. **Layer Switching** (`examples/layers.rhai`)
3. **Modifier Combinations** (`examples/modifiers.rhai`)
4. **Complex Macros** (`examples/macros.rhai`)

### Getting Help

- **GitHub Issues:** [Report bugs or request features](https://github.com/keyrx/keyrx/issues)
- **Discussions:** [Ask questions or share configurations](https://github.com/keyrx/keyrx/discussions)
- **Discord:** Join the KeyRX community (link in README)

---

**Last Updated:** 2025-12-30
**WASM Module Version:** keyrx_core 0.1.0
**Supported Browsers:** Chrome 90+, Firefox 88+, Safari 15+, Edge 90+
