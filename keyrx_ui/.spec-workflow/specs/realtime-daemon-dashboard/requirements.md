# Requirements Document

## Introduction

The Real-Time Daemon Dashboard provides live visualization of KeyRx daemon state, metrics, and events via WebSocket streaming. Currently, users must manually query daemon status via CLI or refresh the web UI repeatedly to see current state. This creates a disconnected experience where users cannot observe real-time behavior during testing or production use.

By implementing WebSocket event streaming (building on the existing WebSocket endpoint in keyrx_daemon/src/web/ws.rs), we can push state updates, latency metrics, and key events to the web UI in real-time. Users will see active modifiers, lock states, layer changes, and performance metrics update instantly as they use their keyboard.

## Requirements

### Requirement 1: WebSocket Event Streaming

**User Story:** As a user testing my configuration, I want to see daemon state updates in real-time as I press keys, so that I can verify my mappings work correctly.

**Acceptance Criteria:**
1. WHEN daemon processes a key event THEN it SHALL broadcast the event to all connected WebSocket clients within 50ms
2. WHEN a client connects THEN it SHALL receive current daemon state immediately (active modifiers, locks, layer)
3. WHEN state changes occur THEN clients SHALL receive incremental updates (only changed fields)
4. WHEN >100 events/second occur THEN the daemon SHALL batch updates at 50ms intervals to prevent overwhelming clients
5. WHEN a WebSocket connection drops THEN the client SHALL auto-reconnect within 3 seconds

### Requirement 2: Live Metrics Dashboard

**User Story:** As a user monitoring performance, I want to see latency statistics updated in real-time, so that I can identify performance degradation immediately.

**Acceptance Criteria:**
1. WHEN daemon calculates latency stats THEN it SHALL broadcast them every 1 second via WebSocket
2. WHEN the dashboard displays metrics THEN it SHALL show min/avg/max/p95/p99 latency in microseconds
3. WHEN latency exceeds 5ms THEN the metric SHALL be highlighted in red
4. WHEN metrics are stable (<1ms avg for 10 seconds) THEN a green indicator SHALL appear
5. WHEN the dashboard is opened THEN it SHALL display a 60-second rolling window of historical metrics

### Requirement 3: Event Timeline Visualization

**User Story:** As a user debugging unexpected behavior, I want to see a scrolling timeline of recent events (last 100 events), so that I can correlate key presses with state changes.

**Acceptance Criteria:**
1. WHEN an event is processed THEN it SHALL appear in the timeline within 100ms
2. WHEN the timeline reaches 100 events THEN the oldest event SHALL be removed (FIFO)
3. WHEN a user hovers over an event THEN a tooltip SHALL show full event details (timestamp, key code, input/output, state snapshot)
4. WHEN events occur rapidly (>50/sec) THEN the timeline SHALL virtualize rendering for performance
5. WHEN a user pauses the timeline THEN new events SHALL buffer without scrolling (resume shows buffered events)

### Requirement 4: State Indicator Panel

**User Story:** As a user, I want to see current daemon state at a glance (active modifiers, locks, layer), so that I understand what mode the daemon is in.

**Acceptance Criteria:**
1. WHEN modifiers are activated THEN they SHALL appear as lit badges (e.g., "MD_00" in blue)
2. WHEN locks are active THEN they SHALL appear as orange badges (e.g., "LK_00")
3. WHEN the active layer changes THEN the layer name SHALL update within 50ms with animation
4. WHEN no modifiers/locks are active THEN the panel SHALL show "No modifiers active" in gray
5. WHEN the user clicks a modifier/lock badge THEN a tooltip SHALL explain what it does (if documented)

### Requirement 5: Performance & Scalability

**User Story:** As a power user with high typing speed (>200 WPM), I want the dashboard to remain responsive even when processing many events per second.

**Acceptance Criteria:**
1. WHEN 100 events/second are processed THEN the dashboard SHALL maintain 60fps rendering
2. WHEN WebSocket messages are batched THEN the UI SHALL debounce updates to 50ms intervals
3. WHEN the dashboard runs for >1 hour THEN memory usage SHALL remain <100MB
4. WHEN the browser tab is inactive THEN WebSocket updates SHALL continue but UI rendering SHALL pause
5. WHEN the user returns to the tab THEN the dashboard SHALL resume with current state (no stale data)

## Non-Functional Requirements

- **Architecture**: React components with useWebSocket hook, Zustand for state management
- **Performance**: WebSocket latency <50ms, UI render <16ms (60fps), batch updates every 50ms
- **Reliability**: Auto-reconnect on disconnect, graceful degradation if WebSocket fails
- **Accessibility**: WCAG 2.1 AA (badges have aria-labels, timeline keyboard navigable)
- **Code Quality**: File sizes ≤250 lines (components), ≤300 lines (hooks), TypeScript strict mode

## Dependencies

- Existing WebSocket endpoint (keyrx_daemon/src/web/ws.rs)
- React 18+ with hooks
- New: react-use-websocket, zustand (state), recharts (metrics visualization)

## Sources

- [How to Build an Interactive Real‑Time Dashboard](https://medium.com/@gideont/how-to-build-an-interactive-real-time-dashboard-0a12eea9f210)
- [Real-Time Data Visualization in React using WebSockets](https://www.syncfusion.com/blogs/post/view-real-time-data-using-websocket)
- [Building Real-Time Dashboards with React and WebSockets](https://www.wildnetedge.com/blogs/building-real-time-dashboards-with-react-and-websockets)
