# Design: WebSocket Event Notification Reliability

## 1. Problem Analysis

### Current Architecture
```
Daemon Service (PATCH /api/devices/:id)
    ↓
Updates device state
    ↓ (MISSING LINK)
Event Bus
    ↓
WebSocket Handler
    ↓
WebSocket Clients
```

**Gap:** Device/profile updates don't publish events to event bus.

## 2. Solution Design

### Event Publishing Integration

```rust
// In device service handler
pub async fn update_device_config(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Json(updates): Json<DeviceConfigUpdate>,
) -> Result<Json<Value>, ApiError> {
    // 1. Update device
    let result = state.device_service.update_device(&id, updates).await?;

    // 2. Publish event to event bus
    state.event_tx.send(DaemonEvent::DeviceUpdated {
        device_id: id.clone(),
        updates: updates.clone(),
    }).await;

    Ok(Json(result))
}
```

### Event Flow
```
1. REST API Call (PATCH /api/devices/:id)
   ↓
2. Service Layer Updates State
   ↓
3. Publish Event to Event Bus (mpsc channel)
   ↓
4. WebSocket Handler Receives Event
   ↓
5. Broadcast to Subscribed Clients
   ↓
6. Client Receives Event (< 100ms)
```

## 3. Implementation Plan

### 3.1 Add Event Publishing to Device Updates
**File:** `keyrx_daemon/src/web/api/devices.rs`
**Change:** Add `event_tx.send()` after successful device update

### 3.2 Add Event Publishing to Profile Activation
**File:** `keyrx_daemon/src/web/api/profiles.rs`
**Change:** Add `event_tx.send()` after successful profile activation

### 3.3 Verify Event Bus Connectivity
**File:** `keyrx_daemon/src/web/ws.rs`
**Change:** Ensure WebSocket handler properly receives from event_rx channel

## 4. Testing Strategy

### 4.1 Unit Tests
- Test event publishing in isolation
- Mock event bus channel
- Verify event payload structure

### 4.2 Integration Tests
- Start daemon with event bus
- Make REST API call
- Verify WebSocket client receives event
- Measure end-to-end latency

### 4.3 E2E Tests
- Use existing `websocket-003` and `websocket-004` tests
- Verify tests pass after implementation
- Run 10 times to ensure no flakiness

## 5. Performance Considerations

### 5.1 Event Channel Capacity
- Use bounded channel with capacity 1000
- Monitor channel fullness
- Log warnings if 80% full

### 5.2 Event Delivery Timeout
- Set 1-second timeout for event send
- Log error if timeout exceeded
- Don't block API response on event delivery

## 6. Rollback Plan

If event publishing causes issues:
1. Remove `event_tx.send()` calls
2. Tests will fail but REST API continues working
3. No user impact (events are optional feature)
