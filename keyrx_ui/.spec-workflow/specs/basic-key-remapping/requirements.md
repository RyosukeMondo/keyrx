# Requirements Document

## Introduction

The Basic Key Remapping spec implements the runtime engine for keyrx, enabling real-time keyboard event processing with firmware-class performance. This phase focuses on core event processing, state management, and lookup mechanisms using a platform-agnostic mock layer for testing. The implementation provides the foundation for actual OS integration in subsequent phases.

**Current State:** The compiler (keyrx_compiler) can parse Rhai DSL and generate binary .krx files. The core data structures (KeyMapping, ConfigRoot, Condition) are defined in keyrx_core. The runtime execution layer does not yet exist.

**This Spec Delivers:** A complete event processing pipeline that loads .krx configs, maintains runtime state (255 modifiers + 255 locks), performs key lookup, applies mappings, and outputs remapped events—all with <1ms latency on mock devices.

## Alignment with Product Vision

This spec directly supports the core goals outlined in product.md:

**From product.md - Core Value Proposition:**
> "keyrx delivers firmware-class performance (<1ms latency) with software-level flexibility"

**How This Spec Delivers:**
- **Sub-millisecond latency:** Event processing pipeline designed for <1ms end-to-end latency
- **Deterministic execution:** Same input sequence always produces same output (no randomness, no time dependencies)
- **Zero-copy processing:** Efficient state management with minimal allocations in hot path
- **Testable architecture:** Mock platform layer enables comprehensive testing without OS dependencies

**From product.md - Key Features:**
> "Support for 255 custom modifier keys (vs. standard 8)"
> "Support for 255 custom lock keys"

**How This Spec Serves Them:**
- **255-bit state vectors:** Efficient tracking of all modifier/lock states using bit vectors
- **Conditional mapping evaluation:** Runtime checks modifier/lock state before applying mappings
- **State isolation:** Each device has independent modifier/lock state

**From product.md - Product Principles:**
> "AI Coding Agent First: deterministic behavior, machine-verifiable configuration"

**How This Spec Enables:**
- **Deterministic event processing:** No undefined behavior, reproducible results for automated testing
- **Structured logging:** JSON-formatted events for machine parsing
- **Mock platform layer:** AI agents can test without physical keyboards

## Requirements

### Requirement 1: Configuration Loading and Deserialization

**User Story:** As a keyrx daemon, I want to load and deserialize .krx binary configuration files efficiently, so that I can prepare the runtime state for event processing without blocking.

#### Acceptance Criteria

1. **WHEN** daemon calls `load_config(path)` with valid .krx file **THEN** system **SHALL** deserialize ConfigRoot using rkyv
   - Verify magic bytes (KRX\n)
   - Verify format version
   - Validate SHA256 hash
   - Deserialize rkyv structure
   - Return ConfigRoot ready for processing

2. **WHEN** .krx file has corrupted hash **THEN** system **SHALL** return `ConfigError::HashMismatch` with expected and actual hashes

3. **WHEN** .krx file has invalid magic bytes **THEN** system **SHALL** return `ConfigError::InvalidMagic`

4. **WHEN** .krx file has unsupported version **THEN** system **SHALL** return `ConfigError::UnsupportedVersion` with version number

5. **WHEN** .krx file is truncated or corrupted **THEN** system **SHALL** return `ConfigError::DeserializationFailed`

6. **WHEN** config loads successfully **THEN** system **SHALL** log structured JSON event:
   ```json
   {"timestamp":"...","level":"INFO","service":"keyrx_daemon","event_type":"config_loaded","context":{"path":"config.krx","hash":"3a7f8c...","devices":2,"mappings":42}}
   ```

### Requirement 2: Runtime State Management

**User Story:** As the runtime engine, I want to maintain per-device state (active modifiers, active locks) with sub-microsecond update times, so that I can evaluate conditional mappings efficiently.

#### Acceptance Criteria

1. **WHEN** DeviceState is initialized **THEN** system **SHALL** create:
   - 255-bit modifier state vector (all bits = 0, all modifiers inactive)
   - 255-bit lock state vector (all bits = 0, all locks inactive)
   - Lookup table built from device mappings

2. **WHEN** key mapping activates modifier MD_XX **THEN** system **SHALL** set bit XX in modifier state vector

3. **WHEN** key mapping releases modifier MD_XX **THEN** system **SHALL** clear bit XX in modifier state vector

4. **WHEN** key mapping toggles lock LK_XX **THEN** system **SHALL** flip bit XX in lock state vector
   - First press: set bit (lock ON)
   - Second press: clear bit (lock OFF)

5. **WHEN** evaluating conditional mapping `Condition::ModifierActive(XX)` **THEN** system **SHALL** return true if bit XX is set in modifier state vector

6. **WHEN** evaluating conditional mapping `Condition::LockActive(XX)` **THEN** system **SHALL** return true if bit XX is set in lock state vector

7. **WHEN** evaluating `Condition::AllActive([A, B, C])` **THEN** system **SHALL** return true only if all conditions A, B, C evaluate to true

8. **WHEN** evaluating `Condition::NotActive([A])` **THEN** system **SHALL** return true only if condition A evaluates to false

9. **ALL** state updates **SHALL** complete in <10μs (measured via benchmarks)

### Requirement 3: Key Lookup Implementation

**User Story:** As the event processor, I want to find the correct mapping for an input key in <100μs, so that total event processing stays under 1ms.

#### Acceptance Criteria

1. **WHEN** lookup is initialized from DeviceConfig **THEN** system **SHALL** build HashMap<KeyCode, Vec<BaseKeyMapping>>
   - Key: input KeyCode
   - Value: vector of all mappings for that key (unconditional first, conditionals ordered)

2. **WHEN** looking up key with no mappings **THEN** system **SHALL** return None (passthrough)

3. **WHEN** looking up key with simple mapping **THEN** system **SHALL** return mapping in <100μs
   - Verified via criterion benchmarks

4. **WHEN** looking up key with conditional mappings **THEN** system **SHALL**:
   - Evaluate conditions in order
   - Return first mapping where condition evaluates to true
   - Return None if no conditions match

5. **WHEN** looking up key with both unconditional and conditional mappings **THEN** system **SHALL**:
   - Check conditional mappings first (in registration order)
   - Fall back to unconditional mapping if no conditions match

6. **WHEN** multiple conditional mappings match **THEN** system **SHALL** return the first matching mapping

7. **ALL** lookup operations **SHALL** be O(log n) or better (HashMap provides O(1) average case)

### Requirement 4: Event Processing Pipeline

**User Story:** As the runtime engine, I want to process key events (press/release) and apply mappings to produce output events, so that users see their configured remapping behavior.

#### Acceptance Criteria

1. **WHEN** input event is `Press(KeyCode::A)` with mapping A→B **THEN** system **SHALL** output `Press(KeyCode::B)`

2. **WHEN** input event is `Release(KeyCode::A)` with mapping A→B **THEN** system **SHALL** output `Release(KeyCode::B)`

3. **WHEN** input key has no mapping **THEN** system **SHALL** output original event unchanged (passthrough)

4. **WHEN** input key maps to modifier MD_XX **THEN** system **SHALL**:
   - On Press: Set modifier bit XX, output nothing
   - On Release: Clear modifier bit XX, output nothing

5. **WHEN** input key maps to lock LK_XX **THEN** system **SHALL**:
   - On Press: Toggle lock bit XX, output nothing
   - On Release: Output nothing (lock toggles only on press)

6. **WHEN** input key has ModifiedOutput mapping (e.g., Shift+1) **THEN** system **SHALL**:
   - On Press: Output Press(LShift), then Press(Num1)
   - On Release: Output Release(Num1), then Release(LShift)

7. **WHEN** input key matches conditional mapping **THEN** system **SHALL**:
   - Evaluate condition using current modifier/lock state
   - Apply mapping if condition is true
   - Passthrough if condition is false and no fallback mapping exists

8. **WHEN** processing TapHold mapping **THEN** system **SHALL** return stub result with TODO comment
   - Note: Full TapHold implementation deferred to advanced-input-logic spec

9. **ALL** event processing (lookup + state update + output generation) **SHALL** complete in <1ms
   - Verified via criterion benchmarks with 10,000 event sequences

10. **WHEN** processing completes **THEN** system **SHALL** log JSON event:
    ```json
    {"timestamp":"...","level":"DEBUG","service":"keyrx_daemon","event_type":"key_processed","context":{"input":"KeyA","output":"KeyB","latency_us":42,"modifiers":0,"locks":0}}
    ```

### Requirement 5: Mock Platform Layer

**User Story:** As a developer, I want to test the event processing pipeline without OS dependencies, so that I can run comprehensive integration tests in CI without physical keyboards.

#### Acceptance Criteria

1. **WHEN** defining platform abstraction **THEN** system **SHALL** provide traits:
   ```rust
   pub trait InputDevice {
       fn next_event(&mut self) -> Result<KeyEvent, DeviceError>;
       fn grab(&mut self) -> Result<(), DeviceError>;
       fn release(&mut self) -> Result<(), DeviceError>;
   }

   pub trait OutputDevice {
       fn inject_event(&mut self, event: KeyEvent) -> Result<(), DeviceError>;
   }
   ```

2. **WHEN** mock input device is created with event sequence **THEN** mock **SHALL**:
   - Return events from sequence via next_event()
   - Track grab() and release() calls
   - Return DeviceError::EndOfStream when sequence exhausted

3. **WHEN** mock output device receives injected event **THEN** mock **SHALL**:
   - Append event to internal Vec<KeyEvent> for verification
   - Return Ok(()) if not configured to fail
   - Return DeviceError::InjectionFailed if configured to simulate failure

4. **WHEN** integration test runs end-to-end workflow **THEN** test **SHALL**:
   - Load .krx config
   - Create DeviceState with mock platform
   - Feed input sequence via mock input device
   - Collect output events from mock output device
   - Assert output matches expected remapped sequence

5. **ALL** mock implementations **SHALL** have zero OS dependencies (no evdev, no Windows API, pure Rust)

### Requirement 6: Error Handling and Observability

**User Story:** As a developer debugging the runtime, I want comprehensive error handling and structured logging, so that I can diagnose issues quickly without manual instrumentation.

#### Acceptance Criteria

1. **WHEN** any error occurs **THEN** system **SHALL** return Result<T, E> (never panic on runtime errors)

2. **WHEN** device not found **THEN** system **SHALL** return `DeviceError::NotFound` with device identifier

3. **WHEN** permission denied (future: requires root for evdev grab) **THEN** system **SHALL** return `DeviceError::PermissionDenied`

4. **WHEN** platform operation fails **THEN** system **SHALL** log error with context:
   ```json
   {"timestamp":"...","level":"ERROR","service":"keyrx_daemon","event_type":"platform_error","context":{"operation":"grab","device":"mock0","error":"PermissionDenied"}}
   ```

5. **ALL** log entries **SHALL** follow schema: `{timestamp, level, service, event_type, context}`
   - Levels: DEBUG, INFO, WARN, ERROR
   - No PII or secrets in logs
   - Machine-parseable JSON format

6. **WHEN** state transition occurs **THEN** system **SHALL** log at DEBUG level:
   ```json
   {"timestamp":"...","level":"DEBUG","service":"keyrx_daemon","event_type":"state_transition","context":{"type":"modifier","id":0,"state":"active","bit_vector":"0x01"}}
   ```

## Non-Functional Requirements

### Code Architecture and Modularity

- **Single Responsibility Principle**:
  - DeviceState: Manages runtime state only (no I/O, no lookup logic)
  - Lookup: Builds and queries mapping tables only
  - EventProcessor: Orchestrates processing only (delegates to DeviceState and Lookup)
  - Platform traits: Define contracts only (no business logic)

- **Modular Design**:
  - keyrx_core: OS-agnostic data structures and state management
  - keyrx_daemon: Platform layer and orchestration
  - Clear separation: core logic is `no_std` compatible (future WASM support)

- **Dependency Management**:
  - No circular dependencies between modules
  - Platform traits use dependency injection (mock or real implementation injected)
  - State and lookup are isolated (state doesn't know about I/O, lookup doesn't know about state)

**File Organization:**
```
keyrx_core/src/
├── runtime/
│   ├── mod.rs              # Public API
│   ├── state.rs            # DeviceState (255-bit vectors, condition evaluation)
│   ├── lookup.rs           # HashMap-based key lookup
│   └── event.rs            # KeyEvent enum, event processing logic

keyrx_daemon/src/
├── platform/
│   ├── mod.rs              # Platform trait definitions
│   └── mock.rs             # Mock implementations for testing
├── processor.rs            # EventProcessor (main orchestrator)
└── config_loader.rs        # Load .krx files (reuses keyrx_compiler deserialize)
```

### Performance

- **Event Processing Latency**: <1ms end-to-end (input → lookup → state update → output)
- **Lookup Time**: <100μs per key lookup (measured via criterion)
- **State Update Time**: <10μs per modifier/lock state change
- **Memory Usage**: <10MB per device (including lookup table and state)
- **Throughput**: Handle 1000 events/second sustained load (typing speed: ~10 keys/sec, so 100x headroom)

**Benchmark Suite:**
- Lookup performance: 1000 random key lookups
- State update: 1000 modifier toggles
- End-to-end: 10,000 event sequences (press + release)
- Conditional evaluation: 1000 lookups with 10-condition chains

### Security

- **Input Validation**: Validate all KeyCode values are in valid range (prevent out-of-bounds access)
- **No Panics**: All error paths return Result<> (no unwrap() in hot path)
- **State Isolation**: Each device has independent state (no cross-device interference)
- **Fail-Safe Passthrough**: If lookup/processing fails, pass input unchanged (don't drop keys)

### Reliability

- **Deterministic Execution**: Same input + same config → same output (100% reproducible)
- **No Event Loss**: All input events produce output (even if passthrough)
- **Graceful Degradation**: If conditional evaluation fails, fall back to unconditional mapping
- **Error Recovery**: Platform errors don't crash daemon (log error, attempt recovery)

### Testability

- **Mock Platform Layer**: 100% of core logic testable without OS dependencies
- **Property-Based Testing**: Use proptest to verify:
  - No events are dropped (input count == output count)
  - Modifier state always valid (bits 0-254 only)
  - Deterministic behavior (same input → same output across runs)
- **Benchmark-Driven Development**: All performance claims verified via criterion benchmarks
- **Integration Tests**: End-to-end workflows (load config → process events → verify output)

### Observability

- **Structured Logging**: All events logged as JSON (timestamp, level, service, event_type, context)
- **Performance Metrics**: Latency histogram exposed (p50, p95, p99, max)
- **State Inspection**: Debug mode allows reading modifier/lock state at any time
- **Event Tracing**: Correlation IDs track events through pipeline (input → lookup → state → output)
