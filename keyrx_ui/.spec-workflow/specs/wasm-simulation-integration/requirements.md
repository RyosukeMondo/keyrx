# Requirements Document

## Introduction

The WASM Simulation Integration feature enables users to test keyboard remapping configurations directly in the web UI without reloading the daemon. Currently, users must edit a Rhai config, compile it to a .krx binary, reload the daemon, and then test with real keyboard input to verify their configuration works correctly. This creates a slow feedback loop that hinders rapid iteration and experimentation.

By compiling keyrx_core's simulation engine to WebAssembly (WASM), we can provide instant, deterministic testing of configurations in the browser. Users can simulate tap-hold timings, layer switches, modifier combinations, and macro sequences with millisecond-precision event sequences, seeing exactly how their configuration will behave before applying it to the daemon.

This is critical for:
- **Configuration Development**: Test tap-hold thresholds and timing-sensitive logic without trial-and-error on real hardware
- **Debugging**: Reproduce exact event sequences that trigger unexpected behavior
- **Education**: Understand how the remapping engine processes events through interactive visualization
- **Confidence**: Verify configuration correctness before committing changes to production

## Alignment with Product Vision

This feature aligns with the KeyRx vision of providing a comprehensive, user-friendly keyboard remapping solution by:

- **Developer Experience**: Instant feedback loop accelerates configuration development and reduces frustration
- **Transparency**: Users see exactly how the DFA state machine processes each event, building mental models of the system
- **Code Reuse**: 100% reuse of keyrx_core engine ensures WASM simulation matches daemon behavior exactly
- **Modern UX**: Browser-based testing matches user expectations for web applications in 2025
- **Accessibility**: No daemon required - users can experiment with configurations on any platform with a web browser

## Requirements

### Requirement 1: WASM Build Pipeline

**User Story:** As a KeyRx developer, I want keyrx_core to compile to WASM, so that I can run the simulation engine in the browser.

#### Acceptance Criteria

1. WHEN keyrx_core is compiled with `wasm-pack build` THEN the build SHALL succeed without errors
2. WHEN the WASM module is loaded in a browser THEN it SHALL be <10MB in size (optimized release build)
3. WHEN the WASM module is initialized THEN it SHALL expose a JavaScript API for simulation functions
4. WHEN keyrx_core has no_std-incompatible dependencies THEN the build SHALL use feature flags to exclude them from WASM builds

### Requirement 2: Configuration Loading in WASM

**User Story:** As a web UI user, I want to load my Rhai configuration into the simulator, so that I can test it without compiling to .krx format.

#### Acceptance Criteria

1. WHEN a user uploads a .rhai file in the web UI THEN the WASM module SHALL parse it and build an in-memory DFA
2. WHEN parsing fails THEN the system SHALL return detailed error messages with line numbers
3. WHEN a user uploads a .krx binary THEN the WASM module SHALL deserialize it using rkyv
4. WHEN configuration exceeds 1MB THEN the system SHALL warn the user about potential performance impact
5. WHEN configuration is loaded THEN the system SHALL validate it for correctness (no invalid key codes, no circular dependencies)

### Requirement 3: Event Sequence Simulation

**User Story:** As a user testing tap-hold timings, I want to simulate precise key press/release sequences with microsecond timestamps, so that I can verify my threshold settings are correct.

#### Acceptance Criteria

1. WHEN a user creates an event sequence THEN they SHALL be able to specify press/release events with timestamps in microseconds
2. WHEN a user simulates a sequence THEN the WASM engine SHALL process events in timestamp order
3. WHEN simulation completes THEN the system SHALL return the output events with latency statistics
4. WHEN a sequence contains 1000 events THEN simulation SHALL complete in <100ms
5. WHEN a user simulates a built-in scenario (e.g., "tap-hold-under-threshold") THEN the system SHALL generate the appropriate event sequence automatically

### Requirement 4: Built-in Test Scenarios

**User Story:** As a user learning KeyRx, I want pre-built test scenarios for common patterns (tap-hold, layer switching, modifiers), so that I can quickly verify my configuration handles standard cases.

#### Acceptance Criteria

1. WHEN a user selects "Tap-Hold Under Threshold" scenario THEN the system SHALL simulate a key press and release within the configured threshold
2. WHEN a user selects "Tap-Hold Over Threshold" scenario THEN the system SHALL simulate a key press held beyond the threshold
3. WHEN a user selects "Layer Switch" scenario THEN the system SHALL simulate layer activation and key presses on that layer
4. WHEN a user selects "Modifier Combination" scenario THEN the system SHALL simulate modifier keys pressed in sequence
5. WHEN a scenario completes THEN the system SHALL display the expected vs. actual output for comparison

### Requirement 5: Simulation Output Visualization

**User Story:** As a user debugging my configuration, I want to see each step of event processing (DFA state transitions, modifier state changes, layer activations), so that I can understand where my configuration behaves unexpectedly.

#### Acceptance Criteria

1. WHEN simulation runs THEN the system SHALL capture each DFA state transition
2. WHEN simulation runs THEN the system SHALL capture modifier and lock state changes
3. WHEN simulation runs THEN the system SHALL capture layer activation/deactivation events
4. WHEN simulation completes THEN the UI SHALL display a timeline of state changes
5. WHEN a user hovers over a timeline event THEN the UI SHALL show the full state at that moment (active modifiers, locks, layer)
6. WHEN output events differ from input events THEN the UI SHALL highlight the differences

### Requirement 6: Performance Monitoring

**User Story:** As a performance-conscious user, I want to see latency statistics for simulated event processing, so that I can identify if my configuration has performance bottlenecks.

#### Acceptance Criteria

1. WHEN simulation completes THEN the system SHALL report min/avg/max/p95/p99 latency for event processing
2. WHEN any event processing exceeds 5ms THEN the system SHALL flag it as a performance warning
3. WHEN configuration has more than 100 layers THEN the system SHALL warn about potential lookup performance impact
4. WHEN simulation processes 1000 events/second THEN the WASM module SHALL use <100MB memory

### Requirement 7: Integration with Web UI Configuration Editor

**User Story:** As a user editing my configuration in the web UI, I want to test my changes immediately without saving or reloading, so that I can iterate quickly.

#### Acceptance Criteria

1. WHEN a user edits a key mapping in the web UI THEN they SHALL see a "Test Configuration" button
2. WHEN a user clicks "Test Configuration" THEN the system SHALL compile the current UI state to a configuration and load it into the WASM simulator
3. WHEN a user creates a custom event sequence THEN the system SHALL save it for reuse
4. WHEN a user runs simulation THEN the UI SHALL display results alongside the configuration editor
5. WHEN simulation reveals an error THEN the UI SHALL highlight the problematic mapping

## Non-Functional Requirements

### Code Architecture and Modularity

- **Single Responsibility Principle**:
  - WASM bindings isolated in keyrx_core/src/wasm.rs
  - JavaScript API wrapper isolated in keyrx_ui/src/wasm/core.ts
  - Simulation UI components isolated in keyrx_ui/src/components/Simulator/
- **Modular Design**:
  - WASM module exposes minimal API (load_config, simulate_events, get_state)
  - No direct DOM manipulation in WASM (output via return values only)
  - Simulation logic reuses existing keyrx_core::simulator module
- **Dependency Management**:
  - WASM build uses feature flags to exclude daemon-specific dependencies
  - No web-sys or js-sys dependencies in core simulation logic
  - UI depends on WASM module via TypeScript type definitions
- **Clear Interfaces**:
  - WASM API documented with JSDoc comments
  - Event types defined in shared schema (Rust + TypeScript)
  - Error types clearly distinguish parse errors, validation errors, and simulation errors

### Performance

- **WASM Module Size**: Optimized build SHALL be <10MB (gzipped)
- **Initialization Time**: WASM module load + initialization SHALL complete in <500ms
- **Simulation Latency**: 1000-event sequence SHALL simulate in <100ms
- **Memory Usage**: WASM module SHALL use <100MB memory during simulation
- **Configuration Load**: 1000-line Rhai config SHALL parse and compile in <200ms

### Security

- **Sandboxing**: WASM code runs in browser sandbox with no filesystem or network access
- **Input Validation**: All user-provided configurations validated before simulation
- **Memory Safety**: Rust's ownership system prevents buffer overflows in WASM
- **No Eval**: Configuration parsing does NOT use JavaScript eval()

### Reliability

- **Error Handling**: WASM panics caught and converted to JavaScript exceptions
- **Graceful Degradation**: If WASM fails to load, UI shows error message and disables simulation features
- **Browser Compatibility**: WASM module SHALL work in Chrome 90+, Firefox 88+, Safari 15+, Edge 90+
- **Deterministic**: Same configuration + same event sequence SHALL always produce identical output

### Usability

- **Discovery**: "Test Configuration" button prominently displayed in configuration editor
- **Feedback**: Simulation results displayed within 1 second of clicking "Run Simulation"
- **Error Messages**: Parse errors include line numbers and helpful suggestions
- **Examples**: Web UI includes 5+ example configurations with pre-built test scenarios
- **Documentation**: Inline help text explains how to create custom event sequences

## Dependencies

### Existing Infrastructure

This feature builds on existing KeyRx components:

1. **keyrx_core** (no_std library):
   - Already supports WASM (no_std compatible)
   - Simulator module already implemented (keyrx_core/src/simulator.rs)
   - rkyv serialization for .krx binaries
   - DFA state machine and event processing

2. **keyrx_compiler**:
   - Rhai parser (can be adapted for WASM use)
   - DFA generation logic (needed for Rhai → DFA compilation in browser)

3. **keyrx_ui**:
   - React components already set up
   - Configuration editor UI exists
   - Need to add Simulator panel and WASM integration

### New Dependencies

- **wasm-pack**: Build tool for Rust → WASM compilation
- **wasm-bindgen**: Rust ↔ JavaScript interop
- **serde-wasm-bindgen**: Serialize Rust types to JavaScript (optional, for complex return values)

### Build Pipeline Changes

- Add WASM build target to CI/CD (GitHub Actions)
- Add `npm run build:wasm` script to compile keyrx_core to WASM
- Integrate WASM build into UI build process (vite.config.ts)
