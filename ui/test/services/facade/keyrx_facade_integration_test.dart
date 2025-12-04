/// Integration tests for KeyrxFacade with real services.
///
/// These tests validate that the facade properly integrates with real service
/// implementations while using a mock FFI bridge. This tests the full stack
/// except for native code.
///
/// Note: These tests focus on integration between services and the facade.
/// Detailed edge cases are covered in unit tests.
library;

import 'package:flutter_test/flutter_test.dart';
import 'package:keyrx_ui/ffi/bridge.dart';
import 'package:keyrx_ui/ffi/bridge_discovery.dart';
import 'package:keyrx_ui/models/validation.dart';
import 'package:keyrx_ui/services/facade/facade_state.dart';
import 'package:keyrx_ui/services/facade/keyrx_facade.dart';
import 'package:keyrx_ui/services/facade/keyrx_facade_impl.dart';
import 'package:keyrx_ui/services/service_registry.dart';
import 'package:mocktail/mocktail.dart';

// Mock classes
class MockKeyrxBridge extends Mock implements KeyrxBridge {}

void main() {
  late MockKeyrxBridge mockBridge;
  late ServiceRegistry registry;
  late KeyrxFacade facade;

  /// Helper to setup engine initialization that tracks state
  void setupEngineInitialization({bool shouldSucceed = true}) {
    var initialized = false;
    when(() => mockBridge.isInitialized).thenAnswer((_) => initialized);
    when(() => mockBridge.initialize()).thenAnswer((_) {
      if (shouldSucceed) {
        initialized = true;
      }
      return shouldSucceed;
    });
  }

  setUpAll(() {
    // Register fallback values
    registerFallbackValue(const KeyboardDevice(
      path: '/dev/input/event0',
      name: 'Test Keyboard',
      vendorId: 0x1234,
      productId: 0x5678,
      hasProfile: false,
    ));
  });

  setUp(() {
    mockBridge = MockKeyrxBridge();

    // Setup default bridge behaviors
    when(() => mockBridge.loadFailure).thenReturn(null);
    setupEngineInitialization();
    when(() => mockBridge.loadScript(any())).thenReturn(true);
    when(() => mockBridge.validateScript(any())).thenReturn(
      ValidationResult(isValid: true, errors: const [], warnings: const []),
    );
    when(() => mockBridge.listDevices()).thenReturn(
      const DeviceListResult(devices: []),
    );
    when(() => mockBridge.selectDevice(any())).thenReturn(0);
    when(() => mockBridge.startDiscovery(any(), any(), any())).thenReturn(
      const DiscoveryStartResult(success: true, totalKeys: 61),
    );
    when(() => mockBridge.cancelDiscovery()).thenReturn(0);

    // Setup stream mocks - return null so services don't try to subscribe
    when(() => mockBridge.stateStream).thenReturn(null);

    // Create registry with real services and mock bridge
    registry = ServiceRegistry.real(bridge: mockBridge);
    facade = KeyrxFacadeImpl(registry);
  });

  tearDown(() async {
    await facade.dispose();
  });

  group('KeyrxFacade Integration - Engine Operations', () {
    test('complete engine lifecycle with real EngineService', () async {
      const scriptPath = '/path/to/script.rhai';

      // Start engine
      final startResult = await facade.startEngine(scriptPath);

      expect(startResult.isOk, isTrue);
      expect(facade.currentState.engine, EngineStatus.running);
      expect(facade.currentState.scriptPath, scriptPath);

      // Verify bridge was called through EngineService
      verify(() => mockBridge.initialize()).called(1);
      verify(() => mockBridge.loadScript(scriptPath)).called(1);

      // Stop engine
      final stopResult = await facade.stopEngine();

      expect(stopResult.isOk, isTrue);
      expect(facade.currentState.engine, EngineStatus.ready);
    });

    test('engine initialization failure propagates through service layer', () async {
      setupEngineInitialization(shouldSucceed: false);

      final result = await facade.startEngine('/path/to/script.rhai');

      expect(result.isErr, isTrue);
      expect(facade.currentState.engine, EngineStatus.error);

      final error = result.errOrNull!;
      expect(error.code, 'OPERATION_FAILED');
      expect(error.userMessage, contains('initialize'));
    });

    test('repeated engine starts use same initialization', () async {
      // First start
      await facade.startEngine('/path/script1.rhai');

      // Second start - should not re-initialize
      await facade.startEngine('/path/script2.rhai');

      // Init called only once, load called twice
      verify(() => mockBridge.initialize()).called(1);
      verify(() => mockBridge.loadScript(any())).called(2);
    });
  });

  group('KeyrxFacade Integration - Device Operations', () {
    test('list and select device with real DeviceService', () async {
      final devices = [
        const KeyboardDevice(
          path: '/dev/input/event0',
          name: 'Keyboard 1',
          vendorId: 0x1234,
          productId: 0x5678,
          hasProfile: false,
        ),
      ];

      when(() => mockBridge.listDevices()).thenReturn(
        DeviceListResult(devices: devices),
      );
      when(() => mockBridge.selectDevice('/dev/input/event0')).thenReturn(0);

      // List devices
      final listResult = await facade.listDevices();
      expect(listResult.isOk, isTrue);
      expect(listResult.okOrNull, devices);
      expect(facade.currentState.device, DeviceStatus.available);

      // Select device
      final selectResult = await facade.selectDevice('/dev/input/event0');
      expect(selectResult.isOk, isTrue);
      expect(facade.currentState.device, DeviceStatus.connected);
      expect(facade.currentState.selectedDevicePath, '/dev/input/event0');

      // Verify bridge calls through DeviceService
      verify(() => mockBridge.listDevices()).called(1);
      verify(() => mockBridge.selectDevice('/dev/input/event0')).called(1);
    });

    test('discovery workflow through facade', () async {
      const device = KeyboardDevice(
        path: '/dev/input/event0',
        name: 'Test',
        vendorId: 0x1234,
        productId: 0x5678,
        hasProfile: false,
      );

      // Start discovery
      final startResult = await facade.startDiscovery(
        device: device,
        rows: 5,
        colsPerRow: [13, 13, 12, 12, 11],
      );

      expect(startResult.isOk, isTrue);
      expect(facade.currentState.discovery, DiscoveryStatus.active);
      expect(facade.currentState.discoveredDeviceCount, 61);

      // Cancel discovery
      final cancelResult = await facade.cancelDiscovery();

      expect(cancelResult.isOk, isTrue);
      expect(facade.currentState.discovery, DiscoveryStatus.cancelled);

      verify(() => mockBridge.startDiscovery(any(), any(), any())).called(1);
      verify(() => mockBridge.cancelDiscovery()).called(1);
    });
  });

  group('KeyrxFacade Integration - Multi-Service Coordination', () {
    test('complete workflow: engine start → device select → discovery', () async {
      const scriptPath = '/path/to/script.rhai';
      const devicePath = '/dev/input/event0';

      // Setup mocks for full workflow
      when(() => mockBridge.selectDevice(devicePath)).thenReturn(0);

      // Step 1: Start engine
      final engineResult = await facade.startEngine(scriptPath);
      expect(engineResult.isOk, isTrue);

      // Step 2: Select device
      final deviceResult = await facade.selectDevice(devicePath);
      expect(deviceResult.isOk, isTrue);

      // Step 3: Start discovery
      final discoveryResult = await facade.startDiscovery(
        device: const KeyboardDevice(
          path: devicePath,
          name: 'Test',
          vendorId: 0x1234,
          productId: 0x5678,
          hasProfile: false,
        ),
        rows: 5,
        colsPerRow: [13, 13, 12, 12, 11],
      );
      expect(discoveryResult.isOk, isTrue);

      // Verify final state aggregates all operations
      expect(facade.currentState.engine, EngineStatus.running);
      expect(facade.currentState.device, DeviceStatus.connected);
      expect(facade.currentState.discovery, DiscoveryStatus.active);
      expect(facade.currentState.scriptPath, scriptPath);
      expect(facade.currentState.selectedDevicePath, devicePath);
    });

    test('partial workflow failure maintains consistent state', () async {
      const scriptPath = '/path/to/script.rhai';

      // Step 1: Start engine succeeds
      final engineResult = await facade.startEngine(scriptPath);
      expect(engineResult.isOk, isTrue);

      // Step 2: Device selection fails
      when(() => mockBridge.selectDevice(any())).thenReturn(-3); // not found

      final deviceResult = await facade.selectDevice('/dev/input/event99');
      expect(deviceResult.isErr, isTrue);

      // Verify state is consistent: engine running, device error
      expect(facade.currentState.engine, EngineStatus.running);
      expect(facade.currentState.device, DeviceStatus.error);
      expect(facade.currentState.scriptPath, scriptPath);
    });

    test('services coordinate through shared bridge instance', () async {
      // Verify all services in registry use the same bridge
      expect(registry.bridge, same(mockBridge));

      // Operations should use coordinated bridge calls
      await facade.startEngine('/path/to/script.rhai');
      await facade.listDevices();

      // Bridge methods called through different services
      verify(() => mockBridge.initialize()).called(1);
      verify(() => mockBridge.listDevices()).called(1);
    });
  });

  group('KeyrxFacade Integration - Error Propagation', () {
    test('bridge errors propagate through service layer to facade', () async {
      when(() => mockBridge.initialize()).thenThrow(
        Exception('FFI initialization failed'),
      );

      final result = await facade.startEngine('/path/to/script.rhai');

      expect(result.isErr, isTrue);
      expect(facade.currentState.engine, EngineStatus.error);

      final error = result.errOrNull!;
      expect(error.originalError, isA<Exception>());
    });

    test('service-level errors are translated to facade errors', () async {
      // Device service will handle bridge return code and translate
      when(() => mockBridge.selectDevice(any())).thenReturn(-3);

      final result = await facade.selectDevice('/dev/input/event0');

      expect(result.isErr, isTrue);
      expect(facade.currentState.device, DeviceStatus.error);

      final error = result.errOrNull!;
      expect(error.userMessage, contains('does not exist'));
    });
  });

  group('KeyrxFacade Integration - Lifecycle', () {
    test('facade disposal does not affect underlying registry', () async {
      // Use the facade
      await facade.startEngine('/path/to/script.rhai');

      // Dispose facade
      await facade.dispose();

      // Registry services should still be accessible
      expect(registry.engineService, isNotNull);
      expect(registry.deviceService, isNotNull);

      // Facade should reject further operations
      expect(
        () => facade.startEngine('/path/to/script.rhai'),
        throwsStateError,
      );
    });

    test('multiple facades can share same registry', () async {
      final facade2 = KeyrxFacadeImpl(registry);

      // Both facades work independently
      await facade.startEngine('/path/script1.rhai');
      await facade2.startEngine('/path/script2.rhai');

      // Each maintains own state
      expect(facade.currentState.scriptPath, '/path/script1.rhai');
      expect(facade2.currentState.scriptPath, '/path/script2.rhai');

      await facade2.dispose();
    });
  });
}
