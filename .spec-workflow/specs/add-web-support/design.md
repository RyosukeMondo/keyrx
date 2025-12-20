# Design: Add Web Support

**Spec Name**: add-web-support
**Created**: 2025-12-20
**Status**: Draft

---

## Architecture Overview

### Current State (Desktop Only)

```
┌─────────────────────────────────────┐
│         Flutter UI (Desktop)        │
│  └── ChangeNotifier Wrappers        │
└────────┬────────────────────────────┘
         │
┌────────▼────────────────────────────┐
│      core_logic (Pure Dart)         │
│  ├── Services (*_impl.dart)         │
│  └── Repositories                   │
└────────┬────────────────────────────┘
         │ FFI
┌────────▼────────────────────────────┐
│         Rust Core (Linux)           │
└─────────────────────────────────────┘
```

### Target State (Multi-Platform)

```
┌─────────────────────────────────────────────────────────────┐
│           Flutter UI (Shared - 0% Redundancy)               │
│        Compiles to: Windows | Linux | Web                   │
└────┬────────────────────────────┬───────────────────────────┘
     │                            │
┌────▼────────┐            ┌──────▼──────────────────┐
│  Desktop    │            │     Web Browser          │
│             │            │                          │
│ core_logic  │            │    core_logic            │
│  (FFI)      │            │     (HTTP)               │
└────┬────────┘            └──────┬───────────────────┘
     │                            │
┌────▼────────┐            ┌──────▼───────────────────┐
│ Rust Core   │            │   Backend API Server     │
│             │            │   ├── core_logic (FFI)   │
│             │            │   └── Rust Core          │
└─────────────┘            └──────────────────────────┘
```

---

## Design Decisions

### Decision 1: Conditional Exports Pattern

**Rationale**: Dart's conditional exports enable platform-specific implementations with zero redundancy.

**Pattern**:
```dart
// Same interface for both platforms
abstract class ServiceInterface { }

// Platform-specific export
export 'service_ffi.dart'
    if (dart.library.html) 'service_web.dart';
```

**Benefits**:
- ✅ Zero code duplication
- ✅ Compile-time platform selection
- ✅ Type-safe across platforms
- ✅ Same API for all consumers

---

### Decision 2: Backend Wraps core_logic

**Rationale**: Maintains SSOT by having backend use same core_logic as desktop.

**Alternative Rejected**: Backend reimplements logic
- ❌ Would violate SSOT
- ❌ Would create maintenance burden
- ❌ Would risk divergence

**Selected Approach**: Backend as thin wrapper
- ✅ core_logic is single source of truth
- ✅ Zero business logic in backend
- ✅ Same behavior guaranteed

---

### Decision 3: WebSocket for Real-Time

**Rationale**: Match FFI callback pattern with WebSocket events.

**Desktop (FFI)**:
```dart
bridge.registerEventCallback(EventType.deviceUpdated, _handleEvent);
```

**Web (WebSocket)**:
```dart
wsClient.subscribe('device.updated', _handleEvent);
```

**Benefits**:
- ✅ Same reactive pattern
- ✅ Low latency
- ✅ Server push
- ✅ No polling overhead

---

## Conditional Export Implementation

### Service Interface Pattern

```dart
// packages/core_logic/lib/services/device_registry_service.dart

/// Device registry service interface (platform-agnostic)
abstract class DeviceRegistryService {
  Stream<List<Device>> get devices;
  Future<void> setDeviceEnabled(String id, bool enabled);
  Future<void> assignProfile(String deviceId, String profileId);
  Future<void> setUserLabel(String deviceId, String label);
  void dispose();
}

/// Platform-specific export
/// Desktop: imports device_registry_service_ffi.dart
/// Web:     imports device_registry_service_web.dart
export 'device_registry_service_ffi.dart'
    if (dart.library.html) 'device_registry_service_web.dart';
```

### FFI Implementation (Desktop)

```dart
// packages/core_logic/lib/services/device_registry_service_ffi.dart

import 'dart:async';
import '../ffi/bridge.dart';
import '../models/device.dart';
import 'device_registry_service.dart';

class DeviceRegistryServiceImpl implements DeviceRegistryService {
  DeviceRegistryServiceImpl({required KeyrxBridge bridge}) : _bridge = bridge {
    _bridge.registerEventCallback(
      EventType.deviceUpdated,
      _handleDeviceUpdate,
    );
  }

  final KeyrxBridge _bridge;
  final _devicesController = StreamController<List<Device>>.broadcast();

  @override
  Stream<List<Device>> get devices => _devicesController.stream;

  @override
  Future<void> setDeviceEnabled(String id, bool enabled) async {
    final result = _bridge.setDeviceEnabled(id, enabled);
    if (result.hasError) {
      throw DeviceRegistryException(result.errorMessage!);
    }
  }

  void _handleDeviceUpdate(Uint8List payload) {
    final devices = _bridge.listDevices();
    _devicesController.add(devices.data ?? []);
  }

  @override
  void dispose() {
    _bridge.unregisterEventCallback(EventType.deviceUpdated);
    _devicesController.close();
  }
}
```

### Web Implementation

```dart
// packages/core_logic/lib/services/device_registry_service_web.dart

import 'dart:async';
import '../web/api_client.dart';
import '../models/device.dart';
import 'device_registry_service.dart';

class DeviceRegistryServiceImpl implements DeviceRegistryService {
  DeviceRegistryServiceImpl({required WebApiClient client}) : _client = client {
    _subscription = _client.subscribe<List<Device>>(
      'device.updated',
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

    // WebSocket will push update to stream
  }

  @override
  void dispose() {
    _subscription?.cancel();
    _devicesController.close();
  }
}
```

---

## Web Transport Layer

### WebApiClient Design

```dart
// packages/core_logic/lib/web/api_client.dart

import 'dart:convert';
import 'package:http/http.dart' as http;
import 'websocket_client.dart';

class WebApiClient {
  WebApiClient({
    required String baseUrl,
    required WebSocketClient wsClient,
  }) : _baseUrl = baseUrl,
       _wsClient = wsClient;

  final String _baseUrl;
  final WebSocketClient _wsClient;

  Future<ApiResponse> get(String path) async {
    final url = Uri.parse('$_baseUrl$path');
    final response = await http.get(url);
    return ApiResponse.fromHttpResponse(response);
  }

  Future<ApiResponse> post(String path, {
    required Map<String, dynamic> body,
  }) async {
    final url = Uri.parse('$_baseUrl$path');
    final response = await http.post(
      url,
      body: jsonEncode(body),
      headers: {'Content-Type': 'application/json'},
    );
    return ApiResponse.fromHttpResponse(response);
  }

  Stream<T> subscribe<T>(String event, T Function(dynamic) parser) {
    return _wsClient.subscribe<T>(event, parser);
  }

  void dispose() {
    _wsClient.dispose();
  }
}

class ApiResponse {
  const ApiResponse({
    required this.data,
    this.error,
  });

  factory ApiResponse.fromHttpResponse(http.Response response) {
    final json = jsonDecode(response.body) as Map<String, dynamic>;

    if (response.statusCode >= 200 && response.statusCode < 300) {
      return ApiResponse(data: json['data']);
    } else {
      return ApiResponse(data: null, error: json['error'] as String);
    }
  }

  final dynamic data;
  final String? error;

  bool get isError => error != null;
}
```

### WebSocketClient Design

```dart
// packages/core_logic/lib/web/websocket_client.dart

import 'dart:async';
import 'dart:convert';
import 'package:web_socket_channel/web_socket_channel.dart';

class WebSocketClient {
  WebSocketClient(String url) : _channel = WebSocketChannel.connect(Uri.parse(url)) {
    _channel.stream.listen(_handleMessage);
  }

  final WebSocketChannel _channel;
  final Map<String, StreamController<dynamic>> _eventStreams = {};

  Stream<T> subscribe<T>(String event, T Function(dynamic) parser) {
    if (!_eventStreams.containsKey(event)) {
      _eventStreams[event] = StreamController<dynamic>.broadcast();
    }

    return _eventStreams[event]!.stream.map((data) => parser(data)).cast<T>();
  }

  void _handleMessage(dynamic message) {
    final json = jsonDecode(message as String) as Map<String, dynamic>;
    final event = json['event'] as String;
    final data = json['data'];

    if (_eventStreams.containsKey(event)) {
      _eventStreams[event]!.add(data);
    }
  }

  void dispose() {
    _channel.sink.close();
    for (final controller in _eventStreams.values) {
      controller.close();
    }
  }
}
```

---

## Backend API Design

### Server Architecture

```dart
// apps/keyrx_server/bin/server.dart

import 'package:shelf/shelf.dart';
import 'package:shelf/shelf_io.dart' as io;
import '../lib/api/routes.dart';
import '../lib/api/middleware.dart';
import '../lib/api/websocket_handler.dart';

void main() async {
  // Initialize core_logic with FFI
  final bridge = KeyrxBridge();
  await bridge.init();

  // Create core_logic services (SSOT!)
  final services = ServiceContainer.create(bridge: bridge);

  // Setup API routes
  final routes = ApiRoutes(services: services);

  // Setup WebSocket handler
  final wsHandler = WebSocketHandler(services: services);

  // Build middleware pipeline
  final handler = Pipeline()
      .addMiddleware(logRequests())
      .addMiddleware(corsHeaders())
      .addMiddleware(handleErrors())
      .addHandler(
        Cascade()
          .add(wsHandler.handler)
          .add(routes.router)
          .handler,
      );

  // Start server
  final server = await io.serve(handler, 'localhost', 8080);
  print('Server running on http://${server.address.host}:${server.port}');
}
```

### API Routes

```dart
// apps/keyrx_server/lib/api/routes.dart

import 'package:shelf_router/shelf_router.dart';
import 'package:core_logic/core_logic.dart';

class ApiRoutes {
  ApiRoutes({required ServiceContainer services}) : _services = services;

  final ServiceContainer _services;

  Router get router {
    final router = Router();

    // Device Registry
    router.get('/api/devices', _listDevices);
    router.post('/api/devices/<id>/enable', _setDeviceEnabled);
    router.post('/api/devices/<id>/profile', _assignProfile);

    // Profile Registry
    router.get('/api/profiles', _listProfiles);
    router.get('/api/profiles/<id>', _getProfile);
    router.post('/api/profiles', _saveProfile);
    router.delete('/api/profiles/<id>', _deleteProfile);

    // Hardware Service
    router.get('/api/hardware', _getHardwareState);
    router.post('/api/hardware/sync', _syncHardware);

    return router;
  }

  Future<Response> _listDevices(Request request) async {
    final devices = await _services.deviceRegistry.devices.first;
    return _jsonResponse({'data': devices.map((d) => d.toJson()).toList()});
  }

  Future<Response> _setDeviceEnabled(Request request, String id) async {
    final body = await _parseBody(request);
    final enabled = body['enabled'] as bool;

    await _services.deviceRegistry.setDeviceEnabled(id, enabled);
    return _jsonResponse({'success': true});
  }

  Response _jsonResponse(Map<String, dynamic> data) {
    return Response.ok(
      jsonEncode(data),
      headers: {'Content-Type': 'application/json'},
    );
  }
}
```

---

## SSOT Violation Detection

### Build-Time Checker

```dart
// tools/ssot_checker/lib/ssot_checker.dart

import 'package:analyzer/dart/analysis/analysis_context.dart';
import 'package:analyzer/dart/analysis/results.dart';

class SSOTChecker {
  final List<SSOTViolation> violations = [];

  Future<void> check(String projectPath) async {
    await _checkFlutterImportsInCoreLogic(projectPath);
    await _checkBusinessLogicInUI(projectPath);
    await _checkInterfaceConsistency(projectPath);
    await _checkDuplicateImplementations(projectPath);
  }

  /// REQ-5.1: Detect Flutter imports in core_logic
  Future<void> _checkFlutterImportsInCoreLogic(String path) async {
    final coreLogicFiles = _findDartFiles('$path/packages/core_logic/lib');

    for (final file in coreLogicFiles) {
      final content = await File(file).readAsString();

      if (content.contains("import 'package:flutter/")) {
        violations.add(SSOTViolation(
          type: ViolationType.flutterInCoreLogic,
          file: file,
          message: 'Flutter import detected in core_logic',
          suggestion: 'Remove Flutter dependency, use pure Dart',
        ));
      }
    }
  }

  /// REQ-5.2: Detect business logic in UI layer
  Future<void> _checkBusinessLogicInUI(String path) async {
    final uiFiles = _findDartFiles('$path/ui/lib');

    for (final file in uiFiles) {
      // Skip service_registry and wrappers
      if (file.contains('service_registry') || file.endsWith('_ui.dart')) {
        continue;
      }

      final content = await File(file).readAsString();

      // Check for direct FFI usage
      if (content.contains('KeyrxBridge')) {
        violations.add(SSOTViolation(
          type: ViolationType.businessLogicInUI,
          file: file,
          message: 'Direct FFI usage detected in UI layer',
          suggestion: 'Use services from core_logic instead',
        ));
      }

      // Check for data transformations
      if (_containsBusinessLogic(content)) {
        violations.add(SSOTViolation(
          type: ViolationType.businessLogicInUI,
          file: file,
          message: 'Business logic detected in UI layer',
          suggestion: 'Move logic to core_logic service',
        ));
      }
    }
  }

  /// REQ-5.3: Verify interface consistency
  Future<void> _checkInterfaceConsistency(String path) async {
    final services = [
      'device_registry_service',
      'profile_registry_service',
      'hardware_service',
      // ... all services
    ];

    for (final service in services) {
      final ffiFile = '$path/packages/core_logic/lib/services/${service}_ffi.dart';
      final webFile = '$path/packages/core_logic/lib/services/${service}_web.dart';

      if (!await File(ffiFile).exists() || !await File(webFile).exists()) {
        continue;
      }

      final ffiMethods = await _extractMethods(ffiFile);
      final webMethods = await _extractMethods(webFile);

      if (!_methodsMatch(ffiMethods, webMethods)) {
        violations.add(SSOTViolation(
          type: ViolationType.interfaceInconsistency,
          file: webFile,
          message: 'FFI and Web implementations have different methods',
          suggestion: 'Ensure both implement same interface',
        ));
      }
    }
  }

  bool _containsBusinessLogic(String content) {
    // Heuristics for business logic
    return content.contains(RegExp(r'class \w+(?!Widget|Page|Screen|Provider)')) &&
           (content.contains('validate') ||
            content.contains('calculate') ||
            content.contains('transform'));
  }
}

enum ViolationType {
  flutterInCoreLogic,
  businessLogicInUI,
  interfaceInconsistency,
  duplicateImplementation,
}

class SSOTViolation {
  const SSOTViolation({
    required this.type,
    required this.file,
    required this.message,
    required this.suggestion,
  });

  final ViolationType type;
  final String file;
  final String message;
  final String suggestion;

  @override
  String toString() => '[$type] $file: $message\n  → $suggestion';
}
```

### Test-Time Verification

```dart
// packages/core_logic/test/ssot/platform_consistency_test.dart

import 'package:test/test.dart';
import 'package:core_logic/services/device_registry_service.dart';

void main() {
  group('SSOT Platform Consistency', () {
    test('FFI and Web implement same interface', () {
      // This test ensures conditional exports work correctly
      final service = DeviceRegistryServiceImpl(
        bridge: MockBridge(), // or client: MockClient()
      );

      // Verify interface methods exist
      expect(service.devices, isA<Stream<List<Device>>>());
      expect(service.setDeviceEnabled, isA<Function>());
      expect(service.assignProfile, isA<Function>());
      expect(service.dispose, isA<Function>());
    });

    test('Transport layer contains no business logic', () {
      // Mock transport to verify it only calls core_logic
      final mockBridge = MockBridge();
      final service = DeviceRegistryServiceImpl(bridge: mockBridge);

      service.setDeviceEnabled('device1', true);

      // Verify service only called transport, no logic
      verify(() => mockBridge.setDeviceEnabled('device1', true)).called(1);
      verifyNoMoreInteractions(mockBridge);
    });

    test('Real-time events consistent across platforms', () async {
      // Test FFI callbacks match WebSocket events
      final ffiService = DeviceRegistryServiceImpl(bridge: mockBridge);
      final webService = DeviceRegistryServiceImpl(client: mockClient);

      final ffiEvents = <List<Device>>[];
      final webEvents = <List<Device>>[];

      ffiService.devices.listen(ffiEvents.add);
      webService.devices.listen(webEvents.add);

      // Trigger update
      mockBridge.simulateEvent(EventType.deviceUpdated);
      mockClient.simulateEvent('device.updated');

      await Future.delayed(Duration(milliseconds: 100));

      // Both should receive same data
      expect(ffiEvents, equals(webEvents));
    });
  });
}
```

### Pre-Commit Hook Integration

```bash
#!/bin/bash
# .git/hooks/pre-commit

echo "Running SSOT violation check..."

dart run tools/ssot_checker/bin/check.dart

if [ $? -ne 0 ]; then
  echo "❌ SSOT violations detected. Commit aborted."
  echo "Run 'dart run tools/ssot_checker/bin/check.dart' for details."
  exit 1
fi

echo "✅ SSOT check passed"
```

---

## File Structure After Migration

```
packages/core_logic/lib/
├── services/
│   ├── device_registry_service.dart          ← Interface + export
│   ├── device_registry_service_ffi.dart      ← FFI impl
│   ├── device_registry_service_web.dart      ← Web impl (NEW)
│   ├── profile_registry_service.dart         ← Interface + export
│   ├── profile_registry_service_ffi.dart     ← FFI impl
│   ├── profile_registry_service_web.dart     ← Web impl (NEW)
│   └── ... (9 more services)
│
├── web/                                       ← NEW
│   ├── api_client.dart
│   ├── websocket_client.dart
│   └── models.dart
│
└── ffi/
    └── bridge.dart                            ← Desktop only

apps/keyrx_server/                             ← NEW
├── bin/
│   └── server.dart
├── lib/
│   ├── api/
│   │   ├── routes.dart
│   │   ├── websocket_handler.dart
│   │   └── middleware.dart
│   └── services/
│       └── service_container.dart
└── pubspec.yaml

tools/ssot_checker/                            ← NEW
├── bin/
│   └── check.dart
├── lib/
│   ├── ssot_checker.dart
│   └── analyzers/
│       ├── flutter_import_analyzer.dart
│       ├── business_logic_analyzer.dart
│       └── interface_consistency_analyzer.dart
└── pubspec.yaml

packages/core_logic/test/ssot/                 ← NEW
├── platform_consistency_test.dart
├── interface_parity_test.dart
└── transport_isolation_test.dart
```

---

## Migration Strategy

### Phase 1: Prepare core_logic
1. Rename `*_impl.dart` → `*_ffi.dart`
2. Extract interfaces to base files
3. Add conditional exports
4. Verify desktop still works

### Phase 2: Add Web Transport
5. Create WebApiClient
6. Create WebSocketClient
7. Implement `*_web.dart` for each service
8. Unit test web implementations

### Phase 3: Build Backend
9. Create keyrx_server project
10. Implement REST API routes
11. Add WebSocket handler
12. Integration test backend

### Phase 4: Add SSOT Checking
13. Create ssot_checker tool
14. Add build-time checks
15. Add test-time verification
16. Integrate into CI/CD

### Phase 5: Enable Flutter Web
17. Update ServiceRegistry
18. Configure web build
19. Test on browsers
20. Performance optimization

---

## Success Criteria

- ✅ Flutter builds to Windows, Linux, Web
- ✅ 100% UI code shared across platforms
- ✅ 100% business logic in core_logic
- ✅ SSOT checker reports 0 violations
- ✅ All platform consistency tests pass
- ✅ Web has real-time updates via WebSocket
- ✅ Performance within 20% of desktop
