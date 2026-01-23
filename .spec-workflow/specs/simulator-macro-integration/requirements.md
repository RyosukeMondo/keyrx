# Requirements: Simulator-Macro Recorder Integration

## 1. Overview

Fix 1 failing workflow test by connecting simulator event output to macro recorder input, enabling macro recording of simulated keyboard events.

## 2. Problem Statement

**Current Status:** 1 test failing due to architectural limitation
**Failing Test:** `workflow-006` - Macro record → simulate → playback

**Root Cause:** Simulator generates keyboard events but doesn't feed them to macro recorder. Events are isolated in simulator subsystem.

**Expected Behavior:**
1. Start macro recording
2. Simulate keyboard events (POST /api/simulator/events)
3. Stop macro recording
4. Get recorded events - should include simulated events

**Actual Behavior:**
1. Start macro recording ✅
2. Simulate keyboard events ✅
3. Stop macro recording ✅
4. Get recorded events - **EMPTY** ❌

## 3. Functional Requirements

### 3.1 Event Flow Integration
- **REQ-3.1.1**: Simulator must publish events to event bus
- **REQ-3.1.2**: Macro recorder must subscribe to event bus
- **REQ-3.1.3**: Simulated events must flow: Simulator → Event Bus → Macro Recorder

### 3.2 Event Format
- **REQ-3.2.1**: Simulator events must match macro recorder event format
- **REQ-3.2.2**: Events must include: key, event_type (press/release), timestamp_us
- **REQ-3.2.3**: Event type must be lowercase string ('press', 'release')

### 3.3 Recording Control
- **REQ-3.3.1**: Macro recorder only captures events when recording active
- **REQ-3.3.2**: Simulated events captured same as physical keyboard events
- **REQ-3.3.3**: Recording state managed independently of simulator

## 4. Non-Functional Requirements

### 4.1 Performance
- **REQ-4.1.1**: Event delivery latency < 1ms (simulator → macro recorder)
- **REQ-4.1.2**: Support 1000+ events/second throughput

### 4.2 Reliability
- **REQ-4.2.1**: Zero missed events during recording
- **REQ-4.2.2**: Event ordering preserved
- **REQ-4.2.3**: Graceful handling if macro recorder full

## 5. Acceptance Criteria

- ✅ `workflow-006` test passes
- ✅ Simulated events recorded by macro recorder
- ✅ Event format matches expected schema
- ✅ Event ordering preserved
- ✅ Zero flaky test failures (10 consecutive runs)

## 6. Out of Scope

- ❌ Event replay through simulator
- ❌ Macro recording of real keyboard events (already works)
- ❌ Event filtering/transformation
- ❌ Recording limits/throttling

## 7. Success Metrics

- ✅ 100% E2E test pass rate (84/84 with this + IPC fixes)
- ✅ Event delivery latency < 1ms
- ✅ Zero missed events in 1000-event test
