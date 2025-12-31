# Requirements Document

## Introduction

The WebSocket Event Streaming feature enables real-time monitoring of keyboard events through the web UI. Currently, users must poll the API or refresh the page to see updated metrics, event logs, and daemon state. This feature will push events from the daemon to connected web clients instantly, providing a live view of keyboard activity, remapping operations, and system state changes.

This is critical for debugging configurations, monitoring performance, and understanding how the keyboard remapping system processes input in real-time.

## Alignment with Product Vision

This feature aligns with the KeyRx vision of providing a comprehensive, user-friendly keyboard remapping solution by:
- **Transparency**: Users can see exactly how their configuration processes each keystroke
- **Developer Experience**: Real-time debugging makes it easier to test and refine configurations
- **Modern UX**: Live updates are expected in 2025 web applications, matching user expectations
- **Performance Monitoring**: Instant visibility into latency and throughput metrics

## Requirements

### Requirement 1: Real-time Event Broadcasting

**User Story:** As a KeyRx user debugging my configuration, I want to see keyboard events as they happen in the web UI, so that I can verify my tap-hold timings and layer switches work correctly.

#### Acceptance Criteria

1. WHEN a physical key is pressed or released THEN the daemon SHALL broadcast the event to all connected WebSocket clients within 10ms
2. WHEN an event is broadcast THEN the system SHALL include timestamp, key code, event type, device ID, active layer, and processing latency
3. WHEN the daemon processes 1000 events/second THEN event broadcasting SHALL NOT increase processing latency by more than 50μs per event

### Requirement 2: State Change Notifications

**User Story:** As a web UI user, I want to see when modifiers, locks, or layers change state, so that I understand the current keyboard configuration without polling.

#### Acceptance Criteria

1. WHEN a modifier is activated or deactivated THEN the daemon SHALL broadcast a state change event
2. WHEN a lock is activated or deactivated THEN the daemon SHALL broadcast a state change event
3. WHEN the active layer changes THEN the daemon SHALL broadcast a state change event with the new layer ID
4. WHEN a state change occurs THEN the broadcast SHALL include the full current state (active modifiers, locks, and layers)

### Requirement 3: Latency Metrics Streaming

**User Story:** As a performance-conscious user, I want to see latency metrics update in real-time, so that I can identify performance bottlenecks as they occur.

#### Acceptance Criteria

1. WHEN latency statistics are updated (every 1 second) THEN the daemon SHALL broadcast the latest min/avg/max/p95/p99 values
2. WHEN latency spikes above 5ms THEN the system SHALL include a warning flag in the broadcast
3. WHEN a client connects mid-stream THEN it SHALL receive the current latency statistics within 100ms

### Requirement 4: Client Subscription Management

**User Story:** As a web UI developer, I want to subscribe only to specific event types (e.g., state changes but not every key press), so that I don't overwhelm the UI with unnecessary data.

#### Acceptance Criteria

1. WHEN a WebSocket client connects THEN it SHALL be able to send a subscription message specifying desired event types
2. WHEN a client subscribes to "events" THEN it SHALL receive key press/release events only
3. WHEN a client subscribes to "state" THEN it SHALL receive state change notifications only
4. WHEN a client subscribes to "latency" THEN it SHALL receive latency metric updates only
5. WHEN a client subscribes to multiple types THEN it SHALL receive all specified event types
6. WHEN a client unsubscribes from a type THEN it SHALL stop receiving those events

### Requirement 5: WebSocket Connection Management

**User Story:** As a web UI user, I want the connection to automatically reconnect if it drops, so that I don't lose monitoring data due to network glitches.

#### Acceptance Criteria

1. WHEN a WebSocket client connects THEN the server SHALL send a welcome message with protocol version and timestamp
2. WHEN a client connection is idle for 30 seconds THEN the server SHALL send a heartbeat ping
3. WHEN a client fails to respond to 3 consecutive heartbeats THEN the server SHALL close the connection
4. WHEN the server capacity reaches 100 concurrent connections THEN new connections SHALL be rejected with error code 503

### Requirement 6: Error Handling and Resilience

**User Story:** As a system administrator, I want the event broadcasting to be resilient to slow or disconnected clients, so that one bad client doesn't affect daemon performance.

#### Acceptance Criteria

1. WHEN a WebSocket client's send buffer is full THEN the system SHALL drop that client's events (not block the daemon)
2. WHEN a broadcast fails to send to a client THEN the system SHALL log the error and continue broadcasting to other clients
3. WHEN the daemon restarts THEN existing WebSocket connections SHALL be cleanly terminated with close code 1012 (service restart)
4. WHEN event broadcast queue exceeds 10,000 messages THEN the system SHALL drop oldest messages (FIFO) to prevent memory exhaustion

## Non-Functional Requirements

### Code Architecture and Modularity

- **Single Responsibility Principle**:
  - Event capture logic remains in daemon's event loop
  - Broadcasting logic is isolated in a separate module
  - WebSocket handling is isolated in web server
- **Modular Design**:
  - Use tokio broadcast channel for event distribution
  - WebSocket handlers are stateless (subscription state in Arc<Mutex>)
  - Event serialization is separate from broadcast logic
- **Dependency Management**:
  - Web server depends on daemon only via broadcast channel
  - No direct coupling between event loop and WebSocket code
- **Clear Interfaces**:
  - Event types defined in shared module
  - Subscription API clearly documented
  - Message format versioned for future compatibility

### Performance

- **Latency**: Event broadcast SHALL add <50μs overhead to daemon processing
- **Throughput**: System SHALL support broadcasting 10,000 events/second to 50 concurrent clients
- **Memory**: Event broadcast queue SHALL use <10MB memory under normal load
- **CPU**: WebSocket handling SHALL use <5% CPU with 50 connected clients

### Security

- **Origin Validation**: WebSocket SHALL only accept connections from `localhost` or configured allowed origins
- **Rate Limiting**: Clients SHALL be limited to 100 messages/second to prevent abuse
- **Buffer Limits**: Per-client send buffer capped at 1MB to prevent memory exhaustion
- **Authentication**: WebSocket SHALL reuse HTTP server's security context (same port, no additional authentication needed for localhost)

### Reliability

- **No Event Loss (daemon)**: Daemon SHALL never block on slow WebSocket clients
- **Graceful Degradation**: If broadcast channel is full, system SHALL drop oldest events, not crash
- **Connection Recovery**: Web UI SHALL automatically reconnect with exponential backoff (1s, 2s, 4s, 8s, max 30s)
- **Zero Downtime**: Adding event broadcasting SHALL not require daemon restart (hot-reload compatible)

### Usability

- **Discovery**: WebSocket endpoint SHALL be documented in API health check response
- **Debugging**: All broadcast messages SHALL be valid JSON for easy inspection
- **Error Messages**: Connection errors SHALL include human-readable explanations
- **Compatibility**: Message format SHALL use standard JSON-RPC 2.0 style notifications
