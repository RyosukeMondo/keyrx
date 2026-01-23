# Design: Simulator-Macro Recorder Integration

## 1. Current Architecture

### Simulator (Isolated)
```
POST /api/simulator/events
    ↓
SimulatorService::simulate_events()
    ↓
Generate KeyEvent structs
    ↓ (NOWHERE - events discarded)
```

### Macro Recorder (Separate)
```
Physical Keyboard
    ↓
Platform Layer (evdev/Windows)
    ↓
Event Loop
    ↓
MacroRecorder::record_event()
```

**Problem:** Two separate event paths with no connection.

## 2. Solution Architecture

### Integrated Event Flow
```
POST /api/simulator/events
    ↓
SimulatorService::simulate_events()
    ↓
Generate KeyEvent structs
    ↓
event_tx.send(KeyEvent) ──┐
                          │
Physical Keyboard         │
    ↓                     │
Platform Layer            │
    ↓                     │
event_tx.send(KeyEvent) ──┤
                          ↓
                    Event Bus (mpsc channel)
                          ↓
                    MacroRecorder::record_event()
```

**Solution:** Route simulator events through same event bus as physical keyboard.

## 3. Implementation Design

### 3.1 Add Event Bus to Simulator

```rust
// keyrx_daemon/src/services/simulator_service.rs
pub struct SimulatorService {
    event_tx: tokio::sync::mpsc::Sender<KeyEvent>, // Add this field
}

impl SimulatorService {
    pub async fn simulate_events(&self, events: Vec<SimulatorEvent>) -> Result<()> {
        for event in events {
            let key_event = KeyEvent {
                key: event.key,
                event_type: event.event_type,
                timestamp_us: SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_micros() as u64,
            };

            // Send to event bus (macro recorder will receive)
            self.event_tx.send(key_event).await?;
        }
        Ok(())
    }
}
```

### 3.2 Connect Macro Recorder to Event Bus

```rust
// keyrx_daemon/src/macro_recorder.rs
pub struct MacroRecorder {
    recording: Arc<AtomicBool>,
    events: Arc<Mutex<Vec<MacroEvent>>>,
}

pub async fn start_recording_loop(
    recorder: Arc<MacroRecorder>,
    mut event_rx: tokio::sync::mpsc::Receiver<KeyEvent>
) {
    while let Some(event) = event_rx.recv().await {
        if recorder.is_recording() {
            recorder.record_event(event);
        }
    }
}
```

### 3.3 Wire Up in Daemon Initialization

```rust
// keyrx_daemon/src/daemon/mod.rs
pub async fn run_daemon() -> Result<()> {
    // Create event bus channel
    let (event_tx, event_rx) = tokio::sync::mpsc::channel(1000);

    // Create simulator with event bus
    let simulator = SimulatorService::new(event_tx.clone());

    // Create macro recorder
    let macro_recorder = Arc::new(MacroRecorder::new());

    // Start macro recorder event loop
    tokio::spawn(start_recording_loop(
        Arc::clone(&macro_recorder),
        event_rx
    ));

    // ... rest of daemon initialization
}
```

## 4. Event Format Standardization

### Current Simulator Event
```rust
pub struct SimulatorEvent {
    pub key: String,           // "VK_A"
    pub event_type: String,    // "press" or "release"
}
```

### Required Macro Event
```rust
pub struct MacroEvent {
    pub key: String,           // "VK_A"
    pub event_type: String,    // "press" or "release"
    pub timestamp_us: u64,     // microseconds since epoch
}
```

**Conversion:** Add timestamp when converting SimulatorEvent → KeyEvent → MacroEvent

## 5. Testing Strategy

### 5.1 Unit Tests
- Test simulator event generation
- Test macro recorder event capture
- Test event format conversion

### 5.2 Integration Tests
- Start macro recording
- Simulate events via SimulatorService
- Verify events recorded
- Check event ordering and timestamps

### 5.3 E2E Tests
- Use existing `workflow-006` test
- Verify test passes after integration
- Run 10 times to ensure no flakiness

## 6. Performance Considerations

### 6.1 Channel Capacity
- Use bounded channel with capacity 1000
- Monitor channel fullness
- Log warnings if 80% full

### 6.2 Event Processing Speed
- MacroRecorder::record_event() is fast (just appends to Vec)
- No blocking operations
- Expected throughput: 10,000+ events/second

### 6.3 Memory Usage
- Each MacroEvent: ~32 bytes
- 1000 events: ~32 KB
- Acceptable memory overhead

## 7. Edge Cases

### 7.1 Recording Not Active
- Events sent to macro recorder but discarded
- No impact on performance

### 7.2 Macro Recorder Full
- Implement max event limit (e.g., 10,000 events)
- Return error if limit exceeded
- Clear error message to user

### 7.3 Simulator and Physical Keyboard Together
- Both send events to same channel
- Macro recorder receives both
- Events timestamped and ordered correctly

## 8. Rollback Plan

If integration causes issues:
1. Remove event_tx from SimulatorService
2. Simulator events no longer recorded
3. workflow-006 test still fails but no other impact
4. Macro recorder still works for physical keyboard
