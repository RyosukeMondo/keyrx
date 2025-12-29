# Test Scenario Library

This directory contains JSON test scenarios for deterministic keyboard event simulation.

## Scenario Format

Each scenario is a JSON file with the following structure:

```json
{
  "description": "Human-readable description of the test case",
  "seed": 42,
  "events": [
    {
      "device_id": null,
      "timestamp_us": 0,
      "key": "KeyName",
      "event_type": "press"
    }
  ],
  "expected_behavior": "What should happen when this scenario is replayed",
  "test_config": "Required configuration for this test to work"
}
```

### Field Descriptions

- **description**: Explains what this scenario tests
- **seed**: Random seed for deterministic replay (use same seed for consistent results)
- **events**: Array of keyboard events in chronological order
  - **device_id**: Optional device identifier for multi-device tests (null for single device)
  - **timestamp_us**: Timestamp in microseconds from scenario start
  - **key**: Key identifier (e.g., "A", "CapsLock", "Shift", "F13")
  - **event_type**: Either "press" or "release"
- **expected_behavior**: What the output should be (for documentation/validation)
- **test_config**: Required configuration in the .rhai profile for this test

## Available Scenarios

### 1. tap-hold-under-threshold.json
Tests tap behavior when key is released before the tap-hold threshold (typically 200ms).

**Expected**: Tap action triggers
**Config needed**: CapsLock configured as tap-hold

### 2. tap-hold-over-threshold.json
Tests hold behavior when key is held beyond the threshold.

**Expected**: Hold action triggers (e.g., layer activation)
**Config needed**: CapsLock configured as tap-hold

### 3. permissive-hold.json
Tests permissive-hold: when another key is pressed during tap-hold, it should trigger hold.

**Expected**: Hold action triggers even if released before threshold
**Config needed**: CapsLock with permissive-hold enabled

### 4. cross-device-modifiers.json
Tests that modifiers on one device affect keys on another device.

**Expected**: Shift+A produces uppercase A across devices
**Config needed**: Global modifier scope (not device-specific)

### 5. macro-sequence.json
Tests macro playback when a macro key is pressed.

**Expected**: Macro sequence is output
**Config needed**: F13 configured as macro key

## Usage

### With CLI
```bash
# Replay a scenario
keyrx simulate --events-file tests/scenarios/tap-hold-under-threshold.json

# Run all scenarios
keyrx test --scenario all
```

### In Tests
```rust
use keyrx_daemon::config::simulation_engine::{EventSequence, SimulationEngine};
use std::fs;

let scenario_json = fs::read_to_string("tests/scenarios/tap-hold-under-threshold.json")?;
let sequence: EventSequence = serde_json::from_str(&scenario_json)?;
let engine = SimulationEngine::new(config_path);
let result = engine.replay(&sequence)?;
```

## Adding New Scenarios

1. Create a new JSON file following the format above
2. Include a deterministic seed (e.g., 42)
3. Provide clear description and expected behavior
4. Document required configuration
5. Ensure timestamps are in microseconds and in chronological order
6. Test the scenario with: `keyrx simulate --events-file your-scenario.json`

## Determinism

All scenarios use seeds for deterministic replay. Running the same scenario with the same seed and configuration should **always** produce identical output. This is critical for automated testing.

## Event Timing Guidelines

- **Microseconds**: All timestamps are in microseconds (1 second = 1,000,000 μs)
- **Typical timings**:
  - Key press duration: 50-100ms (50,000-100,000 μs)
  - Tap-hold threshold: 200ms (200,000 μs)
  - Macro delays: 10-50ms between keys
  - Human-like variation: ±20ms

## Edge Cases to Test

Consider creating scenarios for:
- Simultaneous key presses (within 1ms)
- Very fast taps (<10ms)
- Very long holds (>1 second)
- Modifier combinations (Ctrl+Shift+Alt+key)
- Rapid repeated presses (key bounce)
- Cross-layer key interactions
- Device switching mid-sequence
