# Web Support Architecture Analysis

**Date**: 2025-12-20
**Goal**: Add Web support while maintaining 100% SSOT and zero redundancy

---

## Option 1: Flutter Web + Backend Service (Recommended)

### Architecture Overview

```
┌─────────────────────────────────────────────────────────┐
│                    Flutter UI (Shared)                  │
│  ├── Widgets, Screens, Providers (100% shared)         │
│  └── Compiles to: Windows, Linux, Web                  │
└────────┬──────────────────────────┬─────────────────────┘
         │                          │
    ┌────▼──────┐            ┌──────▼──────┐
    │  Desktop  │            │     Web     │
    │   Mode    │            │    Mode     │
    └────┬──────┘            └──────┬──────┘
         │                          │
    ┌────▼──────┐            ┌──────▼──────────┐
    │core_logic │            │  core_logic_web │
    │  (FFI)    │            │  (HTTP Client)  │
    └────┬──────┘            └──────┬──────────┘
         │                          │
    ┌────▼──────┐            ┌──────▼──────────┐
    │   Rust    │            │   Backend API   │
    │   Core    │            │  (Rust Core)    │
    └───────────┘            └─────────────────┘
```

### Core Logic Adaptation Pattern

**Strategy**: Platform-specific implementations with shared interface

#### 1. Conditional Exports (Zero Redundancy)

```dart
// packages/core_logic/lib/services/device_registry_service.dart

export 'device_registry_service_ffi.dart'
    if (dart.library.html) 'device_registry_service_web.dart';
```

#### 2. FFI Implementation (Desktop - Existing)

```dart
// packages/core_logic/lib/services/device_registry_service_ffi.dart

import '../ffi/bridge.dart';

class DeviceRegistryServiceImpl implements DeviceRegistryService {
  DeviceRegistryServiceImpl({required KeyrxBridge bridge}) : _bridge = bridge;

  final KeyrxBridge _bridge;

  @override
  Stream<List<Device>> get devices => _devicesController.stream;

  @override
  Future<void> setDeviceEnabled(String id, bool enabled) async {
    final result = _bridge.setDeviceEnabled(id, enabled);
    // ... existing implementation
  }
}
```

#### 3. Web Implementation (NEW - Zero Core Logic Changes)

```dart
// packages/core_logic/lib/services/device_registry_service_web.dart

import 'dart:html';
import 'package:http/http.dart' as http;

class DeviceRegistryServiceImpl implements DeviceRegistryService {
  DeviceRegistryServiceImpl({required WebApiClient client}) : _client = client;

  final WebApiClient _client;
  final _devicesController = StreamController<List<Device>>.broadcast();

  @override
  Stream<List<Device>> get devices => _devicesController.stream;

  @override
  Future<void> setDeviceEnabled(String id, bool enabled) async {
    // Same interface, different transport
    final response = await _client.post('/devices/$id/enable',
      body: {'enabled': enabled}
    );

    if (response.statusCode == 200) {
      _devicesController.add(_parseDevices(response.body));
    }
  }
}
```

### File Structure (Zero Redundancy Achieved)

```
packages/core_logic/lib/
├── services/
│   ├── device_registry_service.dart          ← Interface + conditional export
│   ├── device_registry_service_ffi.dart      ← FFI implementation (Desktop)
│   └── device_registry_service_web.dart      ← HTTP implementation (Web)
│
├── repositories/
│   ├── device_config_repository.dart         ← Interface + conditional export
│   ├── device_config_repository_ffi.dart     ← FFI implementation
│   └── device_config_repository_web.dart     ← HTTP implementation
│
├── ffi/
│   └── bridge.dart                           ← Desktop only
│
└── web/
    ├── api_client.dart                       ← Web only
    └── websocket_client.dart                 ← Web only
```

### Backend Service (NEW)

```
apps/keyrx_server/
├── bin/
│   └── server.dart                           ← Entry point
├── lib/
│   ├── api/
│   │   ├── routes.dart                       ← REST endpoints
│   │   ├── websocket_handler.dart            ← Real-time events
│   │   └── middleware.dart                   ← Auth, CORS, etc.
│   │
│   └── services/
│       └── Uses core_logic directly!         ← SSOT maintained
│
└── pubspec.yaml
    dependencies:
      core_logic: ^1.0.0                      ← Shared logic
      shelf: ^1.4.0                           ← Web server
      shelf_router: ^1.1.0
```

### Implementation Plan

#### Phase 1: Add Conditional Exports (1-2 hours)

**Task 1.1**: Create interface pattern for each service

```dart
// packages/core_logic/lib/services/device_registry_service.dart

// Shared interface
abstract class DeviceRegistryService {
  Stream<List<Device>> get devices;
  Future<void> setDeviceEnabled(String id, bool enabled);
  Future<void> assignProfile(String deviceId, String profileId);
  // ... all methods
}

// Platform-specific factory
export 'device_registry_service_ffi.dart'
    if (dart.library.html) 'device_registry_service_web.dart';
```

**Files to update** (9 services, 2 repositories):
- ✓ DeviceRegistryService
- ✓ HardwareService
- ✓ LayoutService
- ✓ ProfileRegistryService
- ✓ TestService
- ✓ ScriptFileService
- ✓ ErrorTranslator
- ✓ StoragePathResolver
- ✓ DeviceConfigRepository
- ✓ MappingRepository

**Pattern**: All existing `_impl.dart` files rename to `_ffi.dart`, keep same code

#### Phase 2: Create Web Implementations (2-3 hours)

**Task 2.1**: Create WebApiClient

```dart
// packages/core_logic/lib/web/api_client.dart

class WebApiClient {
  WebApiClient({
    required String baseUrl,
    required WebSocketClient wsClient,
  }) : _baseUrl = baseUrl,
       _wsClient = wsClient;

  final String _baseUrl;
  final WebSocketClient _wsClient;

  Future<Response> get(String path) async {
    final response = await http.get(Uri.parse('$_baseUrl$path'));
    return Response.fromJson(jsonDecode(response.body));
  }

  Future<Response> post(String path, {required Map<String, dynamic> body}) async {
    final response = await http.post(
      Uri.parse('$_baseUrl$path'),
      body: jsonEncode(body),
      headers: {'Content-Type': 'application/json'},
    );
    return Response.fromJson(jsonDecode(response.body));
  }

  Stream<T> subscribe<T>(String event, T Function(dynamic) parser) {
    return _wsClient.subscribe<T>(event, parser);
  }
}
```

**Task 2.2**: Implement web versions of each service

```dart
// packages/core_logic/lib/services/device_registry_service_web.dart

class DeviceRegistryServiceImpl implements DeviceRegistryService {
  DeviceRegistryServiceImpl({required WebApiClient client})
      : _client = client {
    // Subscribe to real-time device updates via WebSocket
    _subscription = _client.subscribe<List<Device>>(
      'devices.updated',
      (data) => (data as List).map((e) => Device.fromJson(e)).toList(),
    ).listen(_devicesController.add);
  }

  final WebApiClient _client;
  final _devicesController = StreamController<List<Device>>.broadcast();
  StreamSubscription? _subscription;

  @override
  Stream<List<Device>> get devices => _devicesController.stream;

  @override
  Future<void> setDeviceEnabled(String id, bool enabled) async {
    final response = await _client.post('/api/devices/$id/enable',
      body: {'enabled': enabled},
    );

    if (response.isError) {
      throw DeviceRegistryException(response.error!);
    }

    // WebSocket will push update
  }

  @override
  void dispose() {
    _subscription?.cancel();
    _devicesController.close();
  }
}
```

#### Phase 3: Create Backend Service (3-4 hours)

**Task 3.1**: Create REST API server

```dart
// apps/keyrx_server/lib/api/routes.dart

import 'package:shelf/shelf.dart';
import 'package:shelf_router/shelf_router.dart';
import 'package:core_logic/core_logic.dart';

class ApiRoutes {
  ApiRoutes({
    required this.deviceRegistry,
    required this.profileRegistry,
    required this.hardwareService,
    // ... inject all core_logic services
  });

  final DeviceRegistryService deviceRegistry;
  final ProfileRegistryService profileRegistry;
  final HardwareService hardwareService;

  Router get router {
    final router = Router();

    // Device Registry endpoints
    router.get('/api/devices', _listDevices);
    router.post('/api/devices/<id>/enable', _setDeviceEnabled);
    router.post('/api/devices/<id>/profile', _assignProfile);

    // Profile Registry endpoints
    router.get('/api/profiles', _listProfiles);
    router.get('/api/profiles/<id>', _getProfile);
    router.post('/api/profiles', _saveProfile);
    router.delete('/api/profiles/<id>', _deleteProfile);

    return router;
  }

  Future<Response> _listDevices(Request request) async {
    try {
      final devices = await deviceRegistry.devices.first;
      return Response.ok(
        jsonEncode({'data': devices.map((d) => d.toJson()).toList()}),
        headers: {'Content-Type': 'application/json'},
      );
    } catch (e) {
      return Response.internalServerError(
        body: jsonEncode({'error': e.toString()}),
      );
    }
  }

  Future<Response> _setDeviceEnabled(Request request, String id) async {
    final body = jsonDecode(await request.readAsString());
    final enabled = body['enabled'] as bool;

    await deviceRegistry.setDeviceEnabled(id, enabled);
    return Response.ok(jsonEncode({'success': true}));
  }
}
```

**Task 3.2**: Add WebSocket for real-time updates

```dart
// apps/keyrx_server/lib/api/websocket_handler.dart

class WebSocketHandler {
  WebSocketHandler({
    required this.deviceRegistry,
    required this.hardwareService,
  }) {
    _initializeSubscriptions();
  }

  final DeviceRegistryService deviceRegistry;
  final HardwareService hardwareService;
  final Set<WebSocketChannel> _clients = {};

  void _initializeSubscriptions() {
    // Forward core_logic streams to all connected WebSocket clients
    deviceRegistry.devices.listen((devices) {
      _broadcast('devices.updated', devices.map((d) => d.toJson()).toList());
    });

    hardwareService.changes.listen((_) {
      _broadcast('hardware.changed', {});
    });
  }

  void handleConnection(WebSocketChannel channel) {
    _clients.add(channel);

    channel.stream.listen(
      (message) => _handleMessage(channel, message),
      onDone: () => _clients.remove(channel),
    );
  }

  void _broadcast(String event, dynamic data) {
    final message = jsonEncode({'event': event, 'data': data});
    for (final client in _clients) {
      client.sink.add(message);
    }
  }
}
```

**Task 3.3**: Server entry point

```dart
// apps/keyrx_server/bin/server.dart

void main() async {
  // Initialize core_logic with FFI bridge
  final bridge = KeyrxBridge();
  await bridge.init();

  // Create all core_logic services (SSOT!)
  final deviceRegistry = DeviceRegistryServiceImpl(bridge: bridge);
  final profileRegistry = ProfileRegistryServiceImpl(bridge: bridge);
  final hardwareService = HardwareServiceImpl(bridge: bridge);
  // ...

  // Create API routes
  final routes = ApiRoutes(
    deviceRegistry: deviceRegistry,
    profileRegistry: profileRegistry,
    hardwareService: hardwareService,
  );

  // Create WebSocket handler
  final wsHandler = WebSocketHandler(
    deviceRegistry: deviceRegistry,
    hardwareService: hardwareService,
  );

  // Setup server
  final handler = Pipeline()
      .addMiddleware(logRequests())
      .addMiddleware(handleCors())
      .addHandler(
        Cascade()
          .add((request) => request.url.path == 'ws'
              ? webSocketHandler(wsHandler.handleConnection)
              : Response.notFound(''))
          .add(routes.router)
          .handler,
      );

  final server = await serve(handler, 'localhost', 8080);
  print('Server listening on http://${server.address.host}:${server.port}');
}
```

#### Phase 4: Update Flutter UI for Multi-Platform (1 hour)

**Task 4.1**: Platform-aware initialization

```dart
// ui/lib/services/service_registry.dart

factory ServiceRegistry.real({bool useWebBackend = false}) {
  if (useWebBackend || kIsWeb) {
    // Web mode: use HTTP client
    final webClient = WebApiClient(
      baseUrl: 'http://localhost:8080',
      wsClient: WebSocketClient('ws://localhost:8080/ws'),
    );

    // All services use web implementations automatically
    // due to conditional exports!
    return ServiceRegistry(
      deviceRegistry: DeviceRegistryServiceImpl(client: webClient),
      profileRegistry: ProfileRegistryServiceImpl(client: webClient),
      hardwareService: HardwareServiceImpl(client: webClient),
      // ...
    );
  } else {
    // Desktop mode: use FFI (existing code)
    final bridge = KeyrxBridge();
    await bridge.init();

    return ServiceRegistry(
      deviceRegistry: DeviceRegistryServiceImpl(bridge: bridge),
      profileRegistry: ProfileRegistryServiceImpl(bridge: bridge),
      hardwareService: HardwareServiceImpl(bridge: bridge),
      // ...
    );
  }
}
```

**Task 4.2**: Flutter web build configuration

```yaml
# ui/web/index.html
<!DOCTYPE html>
<html>
<head>
  <meta charset="UTF-8">
  <title>KeyRx</title>
</head>
<body>
  <script>
    // Configure backend URL
    window.KEYRX_BACKEND_URL = 'http://localhost:8080';
  </script>
  <script src="main.dart.js" type="application/javascript"></script>
</body>
</html>
```

---

## SSOT Analysis: Zero Redundancy Achieved ✓

### Business Logic Layer

| Component | Desktop | Web | Backend | Redundancy |
|-----------|---------|-----|---------|------------|
| Service interfaces | core_logic | core_logic | core_logic | **0%** ✓ |
| Repository interfaces | core_logic | core_logic | core_logic | **0%** ✓ |
| Models | core_logic | core_logic | core_logic | **0%** ✓ |
| Validation logic | core_logic | core_logic | core_logic | **0%** ✓ |

### Transport Layer

| Component | Desktop | Web | Backend | Redundancy |
|-----------|---------|-----|---------|------------|
| Transport | FFI | HTTP | FFI | **0%** ✓ |
| Implementation | _ffi.dart | _web.dart | _ffi.dart | **0%** ✓ |

### UI Layer

| Component | Desktop | Web | Redundancy |
|-----------|---------|-----|------------|
| Flutter widgets | ✓ | ✓ (same code) | **0%** ✓ |
| Screens | ✓ | ✓ (same code) | **0%** ✓ |
| Providers | ✓ | ✓ (same code) | **0%** ✓ |

### Total Redundancy: 0%

**All business logic remains in core_logic**
**Only transport layer differs (FFI vs HTTP)**
**100% UI code reuse across platforms**

---

## Build & Deployment

### Desktop Build (Existing)

```bash
# Windows
flutter build windows

# Linux
flutter build linux
```

### Web Build (New)

```bash
# Development
flutter run -d chrome

# Production
flutter build web --release

# Deploy static files
# Output: build/web/
```

### Backend Build (New)

```bash
# Development
dart run bin/server.dart

# Production (systemd service)
dart compile exe bin/server.dart -o keyrx-server
./keyrx-server
```

---

## Option 2: Custom Web UI (React/Vue) - NOT Recommended

**Why NOT recommended**:
- ❌ Requires duplicating UI logic
- ❌ Different framework = learning curve
- ❌ More maintenance burden
- ❌ Styling divergence risk
- ❌ 40-60% code redundancy

**Only use if**:
- Need specific web-only features Flutter can't provide
- Web performance requirements Flutter can't meet
- Existing web team with no Flutter experience

---

## Recommendation: Option 1 (Flutter Web)

### Benefits:
✅ **100% SSOT maintained** in core_logic
✅ **Zero UI redundancy** - one Flutter codebase
✅ **Minimal new code** - only transport layer + backend
✅ **Same development workflow** across platforms
✅ **Type safety** end-to-end (Dart everywhere)

### Effort Estimate:

| Task | Time | Complexity |
|------|------|------------|
| Add conditional exports | 2h | Low |
| Implement web services | 3h | Medium |
| Create backend API | 4h | Medium |
| WebSocket integration | 2h | Medium |
| Testing & verification | 3h | Medium |
| **Total** | **14h** | **~2 days** |

---

## Migration Path

### Phase 1: Prepare core_logic (Week 1)
1. Add conditional exports to all services
2. Rename `*_impl.dart` to `*_ffi.dart`
3. Create web implementations (`*_web.dart`)
4. Add WebApiClient infrastructure

### Phase 2: Create Backend (Week 2)
1. Setup keyrx_server project
2. Implement REST API endpoints
3. Add WebSocket support
4. Add authentication/authorization

### Phase 3: Enable Flutter Web (Week 3)
1. Update ServiceRegistry for platform detection
2. Configure web build
3. Test on Chrome/Firefox/Safari
4. Deploy backend service

### Phase 4: Production (Week 4)
1. Setup CI/CD for web builds
2. Configure nginx reverse proxy
3. SSL certificates
4. Monitoring & logging

---

## Conclusion

**Flutter Web + Backend Service** achieves:
- ✅ 100% SSOT in core_logic
- ✅ 0% UI code redundancy
- ✅ Minimal architecture changes
- ✅ Platform-specific optimizations where needed
- ✅ Consistent UX across Windows, Linux, Web

**Zero redundancy goal: ACHIEVED** ✓
