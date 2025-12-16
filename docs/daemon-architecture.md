# KeyRx Daemon Architecture

## Overview

KeyRx follows the daemon-based architecture pattern used by professional keyboard remapping tools (Karabiner-Elements, kmonad, etc.):

- **keyrx daemon**: Background service running with elevated privileges
- **Flutter app**: Control panel, config editor, monitor, and viewer

## Architecture Diagram

```
┌──────────────────────────────────────────────────────┐
│  Flutter App (UI - User Mode)                        │
│  ┌─────────────┬─────────────┬──────────────────┐   │
│  │ Config Tab  │ Monitor Tab │ Devices Tab      │   │
│  │ Edit configs│ View events │ Manage profiles  │   │
│  └─────────────┴─────────────┴──────────────────┘   │
│                                                       │
│  ┌────────────────────────────────────────────────┐ │
│  │ Daemon Service (Dart)                          │ │
│  │ - Connect to IPC socket                        │ │
│  │ - Send control commands                        │ │
│  │ - Stream events                                │ │
│  └────────────────────────────────────────────────┘ │
└───────────────────────┬──────────────────────────────┘
                        │
                        │ IPC (JSON-RPC over Unix Socket)
                        │
┌───────────────────────▼──────────────────────────────┐
│  keyrx daemon (System Service - Root/Elevated)       │
│  ┌────────────────────────────────────────────────┐ │
│  │ IPC Server                                     │ │
│  │ - Unix socket: /var/run/keyrx/daemon.sock     │ │
│  │ - JSON-RPC 2.0 protocol                       │ │
│  │ - Event broadcasting                           │ │
│  └────────────────────────────────────────────────┘ │
│  ┌────────────────────────────────────────────────┐ │
│  │ Engine Manager                                 │ │
│  │ - Start/stop input capture                    │ │
│  │ - Load config/script                          │ │
│  │ - Device discovery (--all mode)               │ │
│  └────────────────────────────────────────────────┘ │
│  ┌────────────────────────────────────────────────┐ │
│  │ Multi-Device Input (Linux Driver)             │ │
│  │ - Capture from all keyboards                  │ │
│  │ - Event merging and ordering                  │ │
│  └────────────────────────────────────────────────┘ │
└──────────────────────────────────────────────────────┘
```

## IPC Protocol

### Transport Layer

- **Linux/macOS**: Unix Domain Socket
  - Path: `/var/run/keyrx/daemon.sock` (root) or `$XDG_RUNTIME_DIR/keyrx/daemon.sock` (user)
- **Windows**: Named Pipe
  - Name: `\\.\pipe\keyrx-daemon`

### Protocol: JSON-RPC 2.0

All communication uses JSON-RPC 2.0 format.

#### Message Types

1. **Request**: Client → Daemon (expects response)
   ```json
   {
     "jsonrpc": "2.0",
     "method": "engine.start",
     "params": { ... },
     "id": 1
   }
   ```

2. **Response**: Daemon → Client (reply to request)
   ```json
   {
     "jsonrpc": "2.0",
     "result": { ... },
     "id": 1
   }
   ```

3. **Notification**: Daemon → Client (no response expected)
   ```json
   {
     "jsonrpc": "2.0",
     "method": "event.input",
     "params": { ... }
   }
   ```

4. **Error**: Daemon → Client (error response)
   ```json
   {
     "jsonrpc": "2.0",
     "error": {
       "code": -32600,
       "message": "Invalid request"
     },
     "id": 1
   }
   ```

## API Methods

### Control Methods

#### `engine.start`

Start the input capture engine.

**Request:**
```json
{
  "jsonrpc": "2.0",
  "method": "engine.start",
  "params": {
    "config_path": "~/.config/keyrx/runtime.json",
    "script_path": "~/.config/keyrx/my_script.rhai",
    "use_all_devices": true
  },
  "id": 1
}
```

**Response:**
```json
{
  "jsonrpc": "2.0",
  "result": {
    "status": "started",
    "devices": [
      { "path": "/dev/input/event3", "name": "Logitech K380", "vendor_id": "0x046d", "product_id": "0xb342" },
      { "path": "/dev/input/event4", "name": "Apple Magic Keyboard", "vendor_id": "0x05ac", "product_id": "0x0267" }
    ]
  },
  "id": 1
}
```

#### `engine.stop`

Stop the input capture engine.

**Request:**
```json
{
  "jsonrpc": "2.0",
  "method": "engine.stop",
  "id": 2
}
```

**Response:**
```json
{
  "jsonrpc": "2.0",
  "result": {
    "status": "stopped"
  },
  "id": 2
}
```

#### `engine.reload`

Reload config and/or script without stopping the engine.

**Request:**
```json
{
  "jsonrpc": "2.0",
  "method": "engine.reload",
  "params": {
    "config_path": "~/.config/keyrx/runtime.json",
    "script_path": "~/.config/keyrx/updated_script.rhai"
  },
  "id": 3
}
```

**Response:**
```json
{
  "jsonrpc": "2.0",
  "result": {
    "status": "reloaded",
    "loaded_script": "~/.config/keyrx/updated_script.rhai"
  },
  "id": 3
}
```

#### `engine.status`

Get current engine status.

**Request:**
```json
{
  "jsonrpc": "2.0",
  "method": "engine.status",
  "id": 4
}
```

**Response:**
```json
{
  "jsonrpc": "2.0",
  "result": {
    "status": "running",
    "config_path": "~/.config/keyrx/runtime.json",
    "script_path": "~/.config/keyrx/my_script.rhai",
    "devices": [
      { "path": "/dev/input/event3", "name": "Logitech K380" }
    ],
    "uptime_seconds": 3600,
    "events_processed": 125430
  },
  "id": 4
}
```

#### `events.subscribe`

Subscribe to real-time event stream.

**Request:**
```json
{
  "jsonrpc": "2.0",
  "method": "events.subscribe",
  "params": {
    "event_types": ["input", "output", "state_change"]
  },
  "id": 5
}
```

**Response:**
```json
{
  "jsonrpc": "2.0",
  "result": {
    "subscribed": true
  },
  "id": 5
}
```

#### `events.unsubscribe`

Unsubscribe from event stream.

**Request:**
```json
{
  "jsonrpc": "2.0",
  "method": "events.unsubscribe",
  "id": 6
}
```

**Response:**
```json
{
  "jsonrpc": "2.0",
  "result": {
    "subscribed": false
  },
  "id": 6
}
```

### Event Notifications

#### `event.input`

Raw input event from device.

```json
{
  "jsonrpc": "2.0",
  "method": "event.input",
  "params": {
    "timestamp_us": 1234567890123456,
    "device_path": "/dev/input/event3",
    "device_name": "Logitech K380",
    "event_type": "key_press",
    "key_code": 30,
    "key_name": "A",
    "modifiers": ["shift"]
  }
}
```

#### `event.output`

Output action sent to virtual device.

```json
{
  "jsonrpc": "2.0",
  "method": "event.output",
  "params": {
    "timestamp_us": 1234567890123460,
    "action_type": "key_press",
    "key_code": 48,
    "key_name": "B"
  }
}
```

#### `event.state_change`

Engine state change notification.

```json
{
  "jsonrpc": "2.0",
  "method": "event.state_change",
  "params": {
    "type": "config_reloaded",
    "timestamp_us": 1234567890123456,
    "message": "Successfully reloaded runtime.json",
    "details": {
      "config_path": "~/.config/keyrx/runtime.json"
    }
  }
}
```

## Error Codes

Standard JSON-RPC 2.0 error codes:

- `-32700`: Parse error
- `-32600`: Invalid request
- `-32601`: Method not found
- `-32602`: Invalid params
- `-32603`: Internal error

Custom error codes:

- `-32000`: Engine not running
- `-32001`: Device not found
- `-32002`: Config load failed
- `-32003`: Script compilation failed
- `-32004`: Permission denied

## Daemon Lifecycle

### Startup

```bash
# Start daemon (requires root/elevated privileges)
sudo keyrx daemon --socket /var/run/keyrx/daemon.sock

# Or as systemd service (Linux)
systemctl start keyrx

# Or as launchd service (macOS)
launchctl load ~/Library/LaunchAgents/dev.keyrx.daemon.plist
```

### Shutdown

```bash
# Graceful shutdown via IPC
keyrx-cli shutdown

# Or send SIGTERM
sudo kill -TERM $(cat /var/run/keyrx/daemon.pid)

# Or via systemd
systemctl stop keyrx
```

## Flutter Integration

### DaemonService (Dart)

```dart
class DaemonService {
  Socket? _socket;
  int _requestId = 0;
  final Map<int, Completer<dynamic>> _pendingRequests = {};
  final StreamController<DaemonEvent> _eventController = StreamController.broadcast();

  Stream<DaemonEvent> get events => _eventController.stream;

  Future<void> connect(String socketPath) async {
    _socket = await Socket.connect(InternetAddress(socketPath, type: InternetAddressType.unix), 0);
    _socket!.listen(_handleMessage);
  }

  Future<dynamic> sendRequest(String method, [Map<String, dynamic>? params]) async {
    final id = ++_requestId;
    final request = {
      'jsonrpc': '2.0',
      'method': method,
      'params': params,
      'id': id,
    };

    final completer = Completer<dynamic>();
    _pendingRequests[id] = completer;

    _socket!.write(jsonEncode(request) + '\n');

    return completer.future;
  }

  Future<void> startEngine({String? configPath, String? scriptPath}) async {
    await sendRequest('engine.start', {
      'config_path': configPath,
      'script_path': scriptPath,
      'use_all_devices': true,
    });
  }

  Future<void> stopEngine() async {
    await sendRequest('engine.stop');
  }

  Future<EngineStatus> getStatus() async {
    final result = await sendRequest('engine.status');
    return EngineStatus.fromJson(result);
  }

  Future<void> subscribeEvents() async {
    await sendRequest('events.subscribe', {
      'event_types': ['input', 'output', 'state_change'],
    });
  }

  void _handleMessage(Uint8List data) {
    final message = jsonDecode(utf8.decode(data));

    if (message['id'] != null) {
      // Response to our request
      final id = message['id'] as int;
      final completer = _pendingRequests.remove(id);

      if (message['error'] != null) {
        completer?.completeError(DaemonException(message['error']));
      } else {
        completer?.complete(message['result']);
      }
    } else if (message['method'] != null) {
      // Notification (event)
      _eventController.add(DaemonEvent.fromJson(message));
    }
  }
}
```

### MonitorPage (Flutter)

```dart
class MonitorPage extends StatefulWidget {
  final DaemonService daemonService;

  @override
  State<MonitorPage> createState() => _MonitorPageState();
}

class _MonitorPageState extends State<MonitorPage> {
  final List<DaemonEvent> _events = [];
  StreamSubscription? _eventSubscription;

  @override
  void initState() {
    super.initState();
    _eventSubscription = widget.daemonService.events.listen((event) {
      setState(() {
        _events.insert(0, event); // Latest first
        if (_events.length > 1000) {
          _events.removeLast(); // Limit buffer
        }
      });
    });
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        title: const Text('Monitor'),
        actions: [
          IconButton(
            icon: const Icon(Icons.clear),
            onPressed: () => setState(() => _events.clear()),
          ),
        ],
      ),
      body: ListView.builder(
        itemCount: _events.length,
        itemBuilder: (context, index) {
          final event = _events[index];
          return _buildEventTile(event);
        },
      ),
    );
  }

  Widget _buildEventTile(DaemonEvent event) {
    // Display event details (timestamp, device, key, etc.)
    return ListTile(
      leading: _getEventIcon(event.type),
      title: Text(event.keyName ?? event.type),
      subtitle: Text('${event.deviceName} • ${_formatTimestamp(event.timestamp)}'),
    );
  }
}
```

## Security Considerations

1. **Socket Permissions**:
   - Daemon socket should be writable only by authorized users
   - Use file permissions: `chmod 660 /var/run/keyrx/daemon.sock`
   - Set group ownership to `keyrx` group

2. **Privilege Separation**:
   - Daemon runs as root for input capture
   - Flutter app runs as normal user
   - IPC provides controlled interface

3. **Command Validation**:
   - Validate all paths (no path traversal)
   - Whitelist allowed config/script directories
   - Rate limiting on reload commands

## Implementation Phases

### Phase 1: Core Daemon (Rust)
- [ ] IPC server (Unix socket)
- [ ] JSON-RPC protocol parser
- [ ] Engine lifecycle manager
- [ ] Event broadcasting

### Phase 2: Flutter Client
- [ ] DaemonService (IPC client)
- [ ] MonitorService (event stream handler)
- [ ] MonitorPage (read-only event viewer)

### Phase 3: Integration
- [ ] System service setup (systemd/launchd)
- [ ] Auto-start configuration
- [ ] Graceful shutdown handling

### Phase 4: Testing
- [ ] IPC protocol tests
- [ ] Multi-client support
- [ ] Error handling and recovery
- [ ] Performance under load
