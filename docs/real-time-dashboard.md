# Real-Time Daemon Dashboard

## Introduction

The **Real-Time Daemon Dashboard** provides live monitoring and visualization of the KeyRX daemon's internal state, key event processing, and performance metrics. It enables users to observe keyboard remapping in action, debug configurations, and monitor system performance.

### Key Features

- **Live State Monitoring**: See active modifiers, locks, and layers in real-time
- **Latency Metrics**: Track processing latency with historical charts
- **Event Timeline**: View the last 100 key events with detailed information
- **Auto-Reconnect**: WebSocket connection automatically recovers from disconnects
- **Zero Configuration**: Works out-of-the-box with default daemon settings

## Getting Started

### Prerequisites

1. KeyRX daemon running with web server enabled (default: http://localhost:9867)
2. Modern web browser with WebSocket support

### Opening the Dashboard

1. Launch the KeyRX daemon with web server enabled:
   ```bash
   scripts/launch.sh
   # Or with custom port:
   scripts/launch.sh --port 9867
   ```

2. Open your browser to the dashboard:
   ```
   http://localhost:9867/dashboard
   ```

3. The dashboard automatically connects via WebSocket and begins streaming events

### Interface Overview

```
┌─────────────────────────────────────────────────────────────────┐
│ KeyRX Dashboard                        ● Connected to daemon    │
├─────────────────────────────────────────────────────────────────┤
│ Daemon State                                                     │
│ ┌─────────────┬─────────────┬─────────────────────────────────┐│
│ │ Modifiers   │ Locks       │ Layer                           ││
│ │ Ctrl Shift  │ CapsLock    │ base                            ││
│ └─────────────┴─────────────┴─────────────────────────────────┘│
├─────────────────────────────────────────────────────────────────┤
│ Latency Metrics (60s window)                                   │
│ ┌───────────────────────────────────────────────────────────┐  │
│ │     5ms ┤                                     ╭─╮          │  │
│ │         │                                  ╭──╯ ╰─╮        │  │
│ │     2ms ┤          ╭────╮            ╭────╯      ╰─╮      │  │
│ │         │    ╭─────╯    ╰────────────╯             ╰──╮   │  │
│ │     0ms └────┴─────────────────────────────────────────┴── │  │
│ │          0s                                          60s    │  │
│ └───────────────────────────────────────────────────────────┘  │
│ Min: 0.8ms  Avg: 1.2ms  Max: 4.5ms  P95: 2.1ms  P99: 3.8ms   │
├─────────────────────────────────────────────────────────────────┤
│ Event Timeline                        [⏸ Pause] [Clear]        │
│ ┌───────────────────────────────────────────────────────────┐  │
│ │ 12:34:56.789 │ KEY_A → KEY_B       │ 1.2ms  │ Press      │  │
│ │ 12:34:56.890 │ KEY_A → KEY_B       │ 1.1ms  │ Release    │  │
│ │ 12:34:57.012 │ KEY_SHIFT (modifier)│ 0.9ms  │ Press      │  │
│ │ 12:34:57.234 │ KEY_C → KEY_D       │ 1.5ms  │ Press      │  │
│ │ ...                                                        │  │
│ └───────────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────────┘
```

**Three Main Panels:**

1. **Daemon State Panel**: Shows currently active modifiers, locks, and layer
2. **Latency Metrics Panel**: Line chart tracking processing latency over 60 seconds
3. **Event Timeline Panel**: Scrollable list of the last 100 key events

## Using the Dashboard

### Monitoring Daemon State

The **Daemon State Panel** displays the current state snapshot:

- **Modifiers**: Active modifier keys (e.g., Ctrl, Shift, Alt)
  - Displayed as blue badges
  - Updates instantly when modifiers activate/deactivate
  - Empty when no modifiers active

- **Locks**: Active lock keys (e.g., CapsLock, NumLock)
  - Displayed as orange badges
  - Persist until toggled off
  - Empty when no locks active

- **Layer**: Current active layer name
  - Displayed as green badge
  - Shows the layer handling current key events
  - Defaults to "base" layer

**Example States:**

```
No modifiers/locks active:
┌─────────────┬─────────────┬─────────────┐
│ Modifiers   │ Locks       │ Layer       │
│ (none)      │ (none)      │ base        │
└─────────────┴─────────────┴─────────────┘

Multiple modifiers active:
┌─────────────┬─────────────┬─────────────┐
│ Modifiers   │ Locks       │ Layer       │
│ Ctrl Shift  │ CapsLock    │ gaming      │
└─────────────┴─────────────┴─────────────┘
```

### Reading Latency Metrics

The **Latency Metrics Panel** tracks processing performance:

- **Line Chart**: Shows latency over 60-second rolling window
  - X-axis: Time (0-60 seconds ago)
  - Y-axis: Latency in milliseconds
  - Data points updated in real-time
  - Red line indicates high latency (>5ms)

- **Statistics** (below chart):
  - **Min**: Minimum latency observed in window
  - **Avg**: Average latency across all events
  - **Max**: Maximum latency spike
  - **P95**: 95th percentile (95% of events faster than this)
  - **P99**: 99th percentile (99% of events faster than this)

**Interpreting Metrics:**

| Latency Range | Status | Action |
|--------------|--------|--------|
| < 1ms | Excellent | Normal operation |
| 1-2ms | Good | Normal operation |
| 2-5ms | Acceptable | Monitor for trends |
| > 5ms | High | Check system load, review config complexity |

**Red Line Warning:**
- Chart line turns red when average latency exceeds 5ms
- Indicates potential performance issue
- Check for:
  - Complex configuration with many layers
  - High CPU usage from other processes
  - Hardware limitations

### Viewing Event Timeline

The **Event Timeline Panel** displays recent key events:

- **Auto-scroll**: Newest events appear at top
- **Virtualized**: Handles 100+ events smoothly
- **Pause/Resume**: Freeze timeline to inspect events
- **Clear**: Remove all events from view

**Timeline Columns:**

1. **Timestamp**: Event time (HH:MM:SS.mmm format)
2. **Key Mapping**: Input → Output transformation
3. **Latency**: Processing time in milliseconds
4. **Event Type**: Press or Release

**Example Events:**

```
12:34:56.789 │ KEY_A → KEY_B       │ 1.2ms  │ Press
  ↑              ↑        ↑           ↑        ↑
  Time        Input    Output     Latency   Type
```

**Special Event Types:**

- **Modifier Events**: Show "(modifier)" instead of output
  ```
  12:34:57.012 │ KEY_SHIFT (modifier) │ 0.9ms │ Press
  ```

- **Unmapped Keys**: Input and output are identical
  ```
  12:34:57.123 │ KEY_ESC → KEY_ESC    │ 0.8ms │ Press
  ```

- **Layer Switches**: May trigger state change (check state panel)

**Using Pause:**

1. Click **[⏸ Pause]** to freeze timeline
2. Inspect events without new entries appearing
3. Events continue buffering in background
4. Click **[▶ Resume]** to show buffered events

**Using Clear:**

1. Click **[Clear]** to remove all events
2. Timeline starts fresh
3. Useful for focused debugging sessions

### Connection States

The dashboard shows connection status in the top banner:

- **Connecting...** (yellow): Establishing WebSocket connection
- **Connected** (green): Receiving live events from daemon
- **Disconnected** (red): Connection lost, auto-reconnect in progress

**Auto-Reconnect Behavior:**

- Automatically attempts reconnection every 3 seconds
- Infinite retry attempts
- No manual intervention required
- Events may be lost during disconnection

**Troubleshooting Connection Issues:**

1. **"Disconnected" persists:**
   - Check daemon is running: `ps aux | grep keyrx_daemon`
   - Verify web server enabled in daemon config
   - Check firewall/port blocking (default: 9867)

2. **Frequent disconnects:**
   - Check network stability
   - Increase daemon WebSocket buffer size
   - Review daemon logs for errors

## WebSocket Protocol

### Connection Details

- **Endpoint**: `ws://localhost:9867/ws`
- **Protocol**: WebSocket (RFC 6455)
- **Format**: JSON messages
- **Direction**: Server → Client (broadcast)

### Message Types

The daemon broadcasts three message types:

#### 1. State Update

Sent when daemon state changes (modifiers, locks, layer):

```json
{
  "type": "state",
  "payload": {
    "modifiers": ["MD_00", "MD_01"],
    "locks": ["LK_00"],
    "layer": "base"
  }
}
```

**Fields:**
- `modifiers`: Array of active modifier IDs
- `locks`: Array of active lock key IDs
- `layer`: Current active layer name

**Frequency:** Only on state changes (not every key event)

#### 2. Key Event

Sent for each key press/release:

```json
{
  "type": "event",
  "payload": {
    "timestamp": 1703941234567890,
    "keyCode": "KEY_A",
    "eventType": "press",
    "input": "KEY_A",
    "output": "KEY_B",
    "latency": 1234
  }
}
```

**Fields:**
- `timestamp`: Microseconds since UNIX epoch
- `keyCode`: Kernel key code constant (e.g., "KEY_A")
- `eventType`: Either "press" or "release"
- `input`: Key code before remapping
- `output`: Key code after remapping
- `latency`: Processing time in microseconds

**Frequency:** High-frequency (100+ events/second possible)

**Batching:** Events may be batched when frequency exceeds 100/sec (50ms intervals)

#### 3. Latency Statistics

Sent periodically with aggregate latency metrics:

```json
{
  "type": "latency",
  "payload": {
    "min": 800,
    "avg": 1200,
    "max": 4500,
    "p95": 2100,
    "p99": 3800,
    "timestamp": 1703941234567890
  }
}
```

**Fields:**
- `min`: Minimum latency (microseconds)
- `avg`: Average latency (microseconds)
- `max`: Maximum latency (microseconds)
- `p95`: 95th percentile latency (microseconds)
- `p99`: 99th percentile latency (microseconds)
- `timestamp`: Snapshot time (microseconds since UNIX epoch)

**Frequency:** Every 5 seconds (configurable)

### Example Message Flow

```
Client                          Daemon
  |                               |
  |-- WebSocket Connect --------->|
  |<-- Connection Accepted -------|
  |                               |
  |<-- {"type":"state",...} ------|  (Initial state)
  |<-- {"type":"latency",...} ----|  (Initial metrics)
  |                               |
[User presses KEY_A]
  |                               |
  |<-- {"type":"event",...} ------|  (KEY_A press)
  |<-- {"type":"state",...} ------|  (If modifier changed)
  |                               |
[User releases KEY_A]
  |                               |
  |<-- {"type":"event",...} ------|  (KEY_A release)
  |                               |
[5 seconds elapse]
  |                               |
  |<-- {"type":"latency",...} ----|  (Updated metrics)
  |                               |
```

### Building Custom Clients

**JavaScript/TypeScript:**

```typescript
const ws = new WebSocket('ws://localhost:9867/ws');

ws.onmessage = (event) => {
  const message = JSON.parse(event.data);

  switch (message.type) {
    case 'state':
      console.log('State:', message.payload);
      break;
    case 'event':
      console.log('Event:', message.payload);
      break;
    case 'latency':
      console.log('Latency:', message.payload);
      break;
  }
};

ws.onopen = () => console.log('Connected');
ws.onerror = (error) => console.error('Error:', error);
ws.onclose = () => console.log('Disconnected');
```

**Python:**

```python
import asyncio
import websockets
import json

async def monitor_daemon():
    uri = "ws://localhost:9867/ws"
    async with websockets.connect(uri) as websocket:
        while True:
            message = await websocket.recv()
            data = json.loads(message)

            if data['type'] == 'state':
                print(f"State: {data['payload']}")
            elif data['type'] == 'event':
                event = data['payload']
                print(f"Event: {event['input']} -> {event['output']} ({event['latency']}μs)")
            elif data['type'] == 'latency':
                metrics = data['payload']
                print(f"Latency avg: {metrics['avg']}μs")

asyncio.run(monitor_daemon())
```

**Rust:**

```rust
use tokio_tungstenite::{connect_async, tungstenite::Message};
use futures_util::StreamExt;
use serde_json::Value;

#[tokio::main]
async fn main() {
    let (ws_stream, _) = connect_async("ws://localhost:9867/ws")
        .await
        .expect("Failed to connect");

    let (_, mut read) = ws_stream.split();

    while let Some(msg) = read.next().await {
        if let Ok(Message::Text(text)) = msg {
            let data: Value = serde_json::from_str(&text).unwrap();
            match data["type"].as_str() {
                Some("state") => println!("State: {:?}", data["payload"]),
                Some("event") => println!("Event: {:?}", data["payload"]),
                Some("latency") => println!("Latency: {:?}", data["payload"]),
                _ => {}
            }
        }
    }
}
```

## Troubleshooting

### Dashboard Not Loading

**Symptom:** Browser shows "Cannot connect" or blank page

**Solutions:**
1. Verify daemon is running:
   ```bash
   ps aux | grep keyrx_daemon
   ```

2. Check web server is enabled:
   ```bash
   curl http://localhost:9867/health
   # Should return: {"status":"ok"}
   ```

3. Try alternate port if 9867 is blocked:
   ```bash
   scripts/launch.sh --port 8080
   # Then open: http://localhost:8080/dashboard
   ```

### WebSocket Won't Connect

**Symptom:** Dashboard shows "Disconnected" indefinitely

**Solutions:**
1. Check browser console for WebSocket errors (F12 → Console)

2. Verify WebSocket endpoint is accessible:
   ```bash
   websocat ws://localhost:9867/ws
   # Should connect and receive messages
   ```

3. Check firewall settings:
   ```bash
   sudo ufw status
   sudo ufw allow 9867/tcp
   ```

4. Review daemon logs:
   ```bash
   tail -f scripts/logs/launch_*.log
   # Look for WebSocket connection errors
   ```

### No Events Appearing

**Symptom:** Dashboard connected but timeline is empty

**Solutions:**
1. Verify daemon is receiving key events:
   ```bash
   # Check daemon logs
   tail -f scripts/logs/launch_*.log
   ```

2. Test with known working keys:
   - Press common keys (A, B, C)
   - Check if events appear in daemon logs

3. Verify configuration is loaded:
   - Check state panel shows correct layer
   - Verify configuration file is valid

### High Latency Warnings

**Symptom:** Metrics chart shows red lines (>5ms)

**Solutions:**
1. Check system CPU usage:
   ```bash
   top
   # Look for high CPU processes
   ```

2. Simplify configuration:
   - Reduce number of layers
   - Minimize complex tap/hold logic
   - Remove unused modifiers

3. Profile daemon performance:
   ```bash
   scripts/launch.sh --debug
   # Check logs for slow operations
   ```

4. Hardware considerations:
   - Older CPUs may struggle with complex configs
   - USB keyboard latency adds to total
   - Virtual machines have additional overhead

## Advanced Usage

### Custom WebSocket URL

Override default WebSocket URL:

```typescript
import { useDaemonWebSocket } from '../hooks/useDaemonWebSocket';

function CustomDashboard() {
  const { isConnected } = useDaemonWebSocket({
    url: 'ws://192.168.1.100:9867/ws',
  });

  // ... rest of component
}
```

### Disable Auto-Reconnect

For custom connection handling:

```typescript
const { isConnected } = useDaemonWebSocket({
  shouldReconnect: false,
});
```

### Export Timeline Data

Capture and export events for analysis:

```typescript
import { useDashboardStore } from '../store/dashboardStore';

function ExportButton() {
  const events = useDashboardStore((state) => state.events);

  const exportEvents = () => {
    const json = JSON.stringify(events, null, 2);
    const blob = new Blob([json], { type: 'application/json' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = 'keyrx-events.json';
    a.click();
  };

  return <button onClick={exportEvents}>Export Events</button>;
}
```

### Custom Event Filtering

Filter timeline to specific keys or latency thresholds:

```typescript
import { useDashboardStore } from '../store/dashboardStore';

function FilteredTimeline() {
  const events = useDashboardStore((state) => state.events);

  // Show only high-latency events (>2ms)
  const highLatencyEvents = events.filter(
    (event) => event.latency > 2000
  );

  // Show only specific key
  const keyAEvents = events.filter(
    (event) => event.input === 'KEY_A'
  );

  return (/* render filtered events */);
}
```

## Performance Considerations

### Browser Performance

- Dashboard uses virtualized rendering for event timeline
- Supports 100+ events without lag
- Chrome/Edge recommended for best performance

### Network Bandwidth

Typical bandwidth usage:

- **Idle**: < 1 KB/s (only state changes)
- **Light typing**: 2-5 KB/s (10-20 keys/sec)
- **Heavy typing**: 10-20 KB/s (100+ keys/sec)
- **Gaming**: 20-50 KB/s (rapid key events)

**Batching:** Events automatically batched when >100 events/sec to reduce bandwidth

### Memory Usage

- Dashboard maintains last 100 events in memory (~50 KB)
- Metrics chart maintains 60 seconds of data points (~10 KB)
- Total memory footprint: < 100 KB
- Old events automatically evicted (FIFO queue)

## Security

### Local-Only by Default

- WebSocket server binds to localhost (127.0.0.1) by default
- Not accessible from network
- Safe for single-user desktop environments

### Remote Access (Advanced)

To enable remote dashboard access:

1. **Configure daemon to bind to network interface:**
   ```bash
   scripts/launch.sh --bind 0.0.0.0
   ```

2. **Use firewall to restrict access:**
   ```bash
   # Allow only specific IP
   sudo ufw allow from 192.168.1.0/24 to any port 9867
   ```

3. **Consider using reverse proxy with authentication:**
   ```nginx
   location /ws {
     auth_basic "KeyRX Dashboard";
     auth_basic_user_file /etc/nginx/.htpasswd;
     proxy_pass http://localhost:9867/ws;
     proxy_http_version 1.1;
     proxy_set_header Upgrade $http_upgrade;
     proxy_set_header Connection "upgrade";
   }
   ```

**WARNING:** Exposing daemon WebSocket to untrusted networks is NOT recommended. The protocol has no authentication and reveals all key events.

## Related Documentation

- **Configuration Guide**: Learn how to create keyboard remapping configs
- **Visual Config Builder**: Build configs with drag-and-drop interface
- **Macro Recorder**: Record and replay keyboard macros
- **Development Guide**: Contribute to KeyRX development

## Appendix

### Keyboard Event Codes

Common key codes you'll see in the dashboard:

| Code | Description |
|------|-------------|
| KEY_A - KEY_Z | Letter keys |
| KEY_0 - KEY_9 | Number row keys |
| KEY_LEFTSHIFT, KEY_RIGHTSHIFT | Shift keys |
| KEY_LEFTCTRL, KEY_RIGHTCTRL | Control keys |
| KEY_LEFTALT, KEY_RIGHTALT | Alt keys |
| KEY_LEFTMETA, KEY_RIGHTMETA | Super/Windows keys |
| KEY_SPACE | Spacebar |
| KEY_ENTER | Enter key |
| KEY_ESC | Escape key |
| KEY_TAB | Tab key |
| KEY_BACKSPACE | Backspace key |
| KEY_CAPSLOCK | Caps Lock |
| KEY_F1 - KEY_F12 | Function keys |

Full list: https://github.com/torvalds/linux/blob/master/include/uapi/linux/input-event-codes.h

### Latency Units

All latency values in WebSocket messages use **microseconds (μs)**:

- 1 millisecond (ms) = 1,000 microseconds (μs)
- 1 second (s) = 1,000,000 microseconds (μs)

**Example conversions:**

| Microseconds | Milliseconds | Description |
|-------------|--------------|-------------|
| 800 μs | 0.8 ms | Excellent |
| 1,200 μs | 1.2 ms | Very good |
| 2,500 μs | 2.5 ms | Good |
| 5,000 μs | 5.0 ms | Acceptable threshold |
| 10,000 μs | 10.0 ms | Noticeable delay |

Dashboard automatically converts to milliseconds for display.
