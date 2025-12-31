# Requirements Document: Tap-Hold Functionality

## Introduction

Tap-Hold (also known as Dual-Function Keys) is a fundamental keyboard remapping feature that enables a single physical key to perform different actions based on press duration. This feature is essential for power users who want to maximize keyboard efficiency without adding physical keys.

**Example Use Case:**
- **Tap** CapsLock (< 200ms) → sends Escape (for Vim users)
- **Hold** CapsLock (>= 200ms) → activates Ctrl modifier (for shortcuts)

This transforms an underutilized key into a dual-purpose powerhouse.

## Alignment with Product Vision

From `product.md`:
- **Advanced Input Logic (Section 4)**: "Deterministic Finite Automaton (DFA) for Tap/Hold behavior" - this is a core planned feature
- **AI-First Verification**: Tap-Hold must be deterministically testable with virtual clock
- **Sub-Millisecond Latency**: Timing logic must not introduce latency spikes
- **Browser-based WASM simulation**: Users must preview Tap-Hold behavior before deployment

## Requirements

### Requirement 1: Basic Tap-Hold Behavior

**User Story:** As a power user, I want to configure a key with dual tap/hold behavior, so that I can perform different actions based on how long I press the key.

#### Acceptance Criteria

1. WHEN a tap-hold key is pressed AND released before threshold_ms THEN system SHALL output the tap action (virtual key)
2. WHEN a tap-hold key is pressed AND held for >= threshold_ms THEN system SHALL activate the hold action (custom modifier)
3. WHEN a tap-hold key with active hold modifier is released THEN system SHALL deactivate the hold modifier
4. IF threshold_ms is not specified THEN system SHALL use default value of 200ms
5. WHEN threshold_ms < 50 THEN system SHALL reject configuration with helpful error message
6. WHEN threshold_ms > 1000 THEN system SHALL warn user but accept configuration

### Requirement 2: DSL Configuration

**User Story:** As a user, I want to configure tap-hold keys using the Rhai DSL, so that I can define dual-function keys in my configuration.

#### Acceptance Criteria

1. WHEN user writes `tap_hold("CapsLock", "VK_Escape", "MD_00", 200)` THEN system SHALL parse and compile correctly
2. WHEN tap parameter lacks VK_ prefix THEN system SHALL reject with error: "tap parameter must have VK_ prefix"
3. WHEN hold parameter lacks MD_ prefix THEN system SHALL reject with error: "hold parameter must have MD_ prefix"
4. WHEN user combines tap_hold with when_start conditional THEN system SHALL honor the conditional context
5. WHEN device_start pattern matches multiple devices THEN system SHALL apply tap-hold to all matched devices

### Requirement 3: State Machine Behavior

**User Story:** As a user, I want predictable and deterministic tap-hold behavior, so that my muscle memory remains consistent.

#### Acceptance Criteria

1. WHEN tap-hold key is in Pending state AND another key is pressed THEN system SHALL immediately confirm Hold (Permissive Hold mode)
2. WHEN tap-hold key is in Pending state AND timer expires THEN system SHALL activate hold modifier
3. WHEN multiple tap-hold keys are pressed simultaneously THEN system SHALL track each independently
4. IF system restarts THEN tap-hold state SHALL reset to idle (no stuck modifiers)
5. WHEN key repeat events occur (OS auto-repeat) THEN system SHALL ignore repeats while in Pending/Hold state

### Requirement 4: Timing and Determinism

**User Story:** As an AI coding agent, I want tap-hold behavior to be fully deterministic, so that I can verify configurations without manual testing.

#### Acceptance Criteria

1. WHEN same input sequence is replayed THEN system SHALL produce identical output (deterministic)
2. WHEN virtual clock is used in tests THEN system SHALL use virtual time instead of wall clock
3. WHEN event timestamps are provided by OS THEN system SHALL use those timestamps (not query wall clock)
4. IF event arrives without timestamp THEN system SHALL use monotonic clock reading
5. WHEN processing latency exceeds 10ms THEN system SHALL log warning but continue functioning

### Requirement 5: Integration with Existing Features

**User Story:** As a user, I want tap-hold to work seamlessly with other keyrx features, so that I can build complex configurations.

#### Acceptance Criteria

1. WHEN tap-hold hold modifier (MD_XX) is active THEN conditional mappings using that modifier SHALL activate
2. WHEN tap-hold is configured inside when_start block THEN tap-hold SHALL only activate when parent condition is true
3. WHEN tap action outputs a key THEN that key SHALL respect active modifiers (physical Shift, Ctrl, etc.)
4. WHEN multiple devices have different tap-hold configs THEN each device SHALL use its own configuration
5. WHEN cross-device modifier sharing is active THEN tap-hold modifier from Device A SHALL affect Device B

## Non-Functional Requirements

### Code Architecture and Modularity
- **Single Responsibility Principle**: Tap-hold state machine in separate module from event processing
- **Modular Design**: PendingKeyState, TapHoldStateMachine as distinct, testable components
- **Dependency Management**: Timing abstraction injected via trait (not hardcoded)
- **Clear Interfaces**: `TapHoldProcessor` trait with `process_event(&mut self, event: KeyEvent, time: Instant) -> Vec<KeyEvent>`

### Performance
- State machine transition: < 100ns (no heap allocation)
- Timer check: < 50ns (simple comparison)
- No additional latency on non-tap-hold keys
- Memory: < 1KB per active tap-hold key (max 32 concurrent)

### Security
- No timing side-channels (constant-time operations where possible)
- No logging of key content (only state transitions in debug mode)

### Reliability
- No stuck modifiers after daemon restart
- Graceful handling of OS event drops (timeout-based recovery)
- No deadlock possible in state machine

### Usability
- Clear error messages for invalid configurations
- Debug mode shows state transitions: `[DEBUG] Key=CapsLock State=Pending→Hold elapsed=205ms`
- WASM simulator shows real-time state visualization
