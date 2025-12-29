# Design Document

## Architecture

```
Daemon Event Loop → WebSocket Broadcast → React Dashboard
     ↓                       ↓                    ↓
  Events/State         JSON Messages      useWebSocket hook
                                                  ↓
                                            Zustand Store
                                                  ↓
                                          Dashboard Components
```

## Components

### 1. WebSocket Event Broadcaster (Rust - keyrx_daemon/src/web/ws.rs)
- Add broadcast channel: `tokio::sync::broadcast`
- Publish events from daemon main loop
- Batch updates every 50ms when >100 events/sec

### 2. useDaemonWebSocket Hook (keyrx_ui/src/hooks/useDaemonWebSocket.ts)
- Wrap react-use-websocket
- Auto-reconnect on disconnect
- Parse incoming JSON messages
- Update Zustand store

### 3. Dashboard Store (keyrx_ui/src/store/dashboardStore.ts)
- Zustand store for daemon state
- `currentState: { modifiers, locks, layer }`
- `events: Event[]` (last 100)
- `metrics: LatencyStats`

### 4. DashboardPage Component (keyrx_ui/src/pages/DashboardPage.tsx)

**UI Layout:**
```
+---------------------------------------------------------------+
| Real-Time Dashboard                          [●  Connected]   |
+---------------------------------------------------------------+
| Current State                                                 |
| Active Modifiers: [MD_00] [MD_01]        Layer: base          |
| Active Locks: [LK_00]                                         |
+---------------------------------------------------------------+
| Latency Metrics (Live)                                        |
| Min: 0.8ms  Avg: 1.2ms  Max: 3.5ms  P95: 2.1ms  P99: 2.8ms  |
| [============================] (last 60 sec chart)            |
+---------------------------------------------------------------+
| Event Timeline (Last 100)                      [Pause]        |
| 14:23:45.123 A↓ → A↓ (base)                  1.2ms           |
| 14:23:45.234 A↑ → A↑ (base)                  0.9ms           |
| 14:23:45.456 KEY_LEFTSHIFT↓ → MD_00+         1.5ms           |
| ...                                                            |
+---------------------------------------------------------------+
```

### 5. StateIndicatorPanel (keyrx_ui/src/components/StateIndicatorPanel.tsx)
- Display active modifiers/locks/layer
- Badges with color coding (blue=modifier, orange=lock, green=layer)

### 6. MetricsChart (keyrx_ui/src/components/MetricsChart.tsx)
- Line chart (recharts) showing latency over time
- 60-second rolling window
- Red highlight when >5ms

### 7. EventTimeline (keyrx_ui/src/components/EventTimeline.tsx)
- Virtualized list (react-window) for performance
- FIFO (100 events max)
- Pause/resume functionality

## Data Models

```typescript
interface DaemonState {
  modifiers: string[];  // ["MD_00", "MD_01"]
  locks: string[];      // ["LK_00"]
  layer: string;        // "base"
}

interface KeyEvent {
  timestamp: number;    // Unix timestamp in μs
  keyCode: string;      // "KEY_A"
  eventType: 'press' | 'release';
  input: string;        // Input key
  output: string;       // Output key (after mapping)
  latency: number;      // Processing latency in μs
}

interface LatencyStats {
  min: number;
  avg: number;
  max: number;
  p95: number;
  p99: number;
  timestamp: number;
}

type WebSocketMessage = 
  | { type: 'state', payload: DaemonState }
  | { type: 'event', payload: KeyEvent }
  | { type: 'latency', payload: LatencyStats };
```

## WebSocket Message Protocol

**State Update:**
```json
{
  "type": "state",
  "payload": {
    "modifiers": ["MD_00"],
    "locks": [],
    "layer": "base"
  }
}
```

**Key Event:**
```json
{
  "type": "event",
  "payload": {
    "timestamp": 1735654345123456,
    "keyCode": "KEY_A",
    "eventType": "press",
    "input": "A",
    "output": "B",
    "latency": 1200
  }
}
```

**Latency Metrics:**
```json
{
  "type": "latency",
  "payload": {
    "min": 800,
    "avg": 1200,
    "max": 3500,
    "p95": 2100,
    "p99": 2800,
    "timestamp": 1735654345000000
  }
}
```

## Dependencies

- `react-use-websocket@^4.8.1`
- `zustand@^4.5.0`
- `recharts@^2.10.0`
- `react-window@^1.8.10`

## Sources

- [Real-Time Data Visualization in React using WebSockets](https://www.syncfusion.com/blogs/post/view-real-time-data-using-websocket)
- [Building Real-Time Dashboards with React and WebSockets](https://www.wildnetedge.com/blogs/building-real-time-dashboards-with-react-and-websockets)
