# Requirements: WebSocket Event Notification Reliability

## 1. Overview

Fix 2 failing WebSocket event notification tests by ensuring proper event stream connection and reliable event delivery from daemon to WebSocket clients.

## 2. Problem Statement

**Current Status:** 3/5 WebSocket tests passing (60%)
**Failing Tests:**
- `websocket-003` - Device update events not received
- `websocket-004` - Profile activation events not received

**Root Cause:** WebSocket connection established but daemon events not properly propagated through event notification system.

## 3. Functional Requirements

### 3.1 Event Stream Connection
- **REQ-3.1.1**: WebSocket clients must subscribe to daemon event stream
- **REQ-3.1.2**: Events must propagate from daemon → event bus → WebSocket handler → clients
- **REQ-3.1.3**: Event delivery must complete within 1 second of trigger

### 3.2 Device Update Events
- **REQ-3.2.1**: PATCH /api/devices/:id must publish device_updated event
- **REQ-3.2.2**: Event payload must include device ID and updated fields
- **REQ-3.2.3**: Subscribed clients must receive event via WebSocket

### 3.3 Profile Activation Events
- **REQ-3.3.1**: Profile activation must publish profile_activated event
- **REQ-3.3.2**: Event payload must include profile name
- **REQ-3.3.3**: Subscribed clients must receive event via WebSocket

## 4. Non-Functional Requirements

### 4.1 Performance
- **REQ-4.1.1**: Event delivery latency < 100ms (p95)
- **REQ-4.1.2**: Support 100+ concurrent WebSocket connections

### 4.2 Reliability
- **REQ-4.2.1**: Zero missed events under normal conditions
- **REQ-4.2.2**: Events queued if client temporarily unavailable
- **REQ-4.2.3**: Graceful degradation on event bus failure

## 5. Acceptance Criteria

- ✅ `websocket-003` test passes (device update events)
- ✅ `websocket-004` test passes (profile activation events)
- ✅ Event delivery latency < 100ms
- ✅ Zero flaky test failures (10 consecutive runs)

## 6. Out of Scope

- ❌ Event persistence/replay
- ❌ Event filtering/transformation
- ❌ Multi-node event distribution
- ❌ Custom event types beyond device/profile

## 7. Success Metrics

- ✅ 100% WebSocket test pass rate (5/5)
- ✅ < 100ms event delivery latency
- ✅ Zero missed events in 1000-event stress test
