import 'package:flutter_test/flutter_test.dart';
import 'package:keyrx_ui/ffi/bridge.dart';
import 'package:keyrx_ui/ffi/bridge_discovery.dart';
import 'package:keyrx_ui/ffi/bridge_engine.dart' as engine_bridge;
import 'package:keyrx_ui/models/validation.dart';
import 'package:keyrx_ui/services/device_service.dart';
import 'package:keyrx_ui/services/engine_service.dart';
import 'package:keyrx_ui/services/error_translator.dart';
import 'package:keyrx_ui/services/facade/facade_state.dart';
import 'package:keyrx_ui/services/facade/keyrx_facade.dart';
import 'package:keyrx_ui/services/facade/keyrx_facade_impl.dart';
import 'package:keyrx_ui/services/facade/result.dart';
import 'package:keyrx_ui/services/script_file_service.dart';
import 'package:keyrx_ui/services/service_registry.dart';
import 'package:keyrx_ui/services/test_service.dart';
import 'package:mocktail/mocktail.dart';
import 'package:rxdart/rxdart.dart';

// Mock classes
class MockServiceRegistry extends Mock implements ServiceRegistry {}

class MockEngineService extends Mock implements EngineService {}

class MockDeviceService extends Mock implements DeviceService {}

class MockScriptFileService extends Mock implements ScriptFileService {}

class MockTestService extends Mock implements TestService {}

class MockErrorTranslator extends Mock implements ErrorTranslator {}

class MockKeyrxBridge extends Mock implements KeyrxBridge {}

void main() {
  late MockServiceRegistry mockRegistry;
  late MockEngineService mockEngine;
  late MockDeviceService mockDevice;
  late MockScriptFileService mockScriptFile;
  late MockTestService mockTestService;
  late MockErrorTranslator mockTranslator;
  late MockKeyrxBridge mockBridge;
  late BehaviorSubject<EngineSnapshot> engineStateSubject;

  setUpAll(() {
    // Register fallback values for mocktail
    registerFallbackValue(EngineSnapshot(
      activeLayers: const [],
      heldKeys: const [],
      pendingDecisions: const [],
      timestamp: DateTime.now(),
    ));
    registerFallbackValue(const UserMessage(
      title: 'Error',
      body: 'An error occurred',
      category: MessageCategory.error,
    ));
    registerFallbackValue(const KeyboardDevice(
      path: '/dev/input/event0',
      name: 'Test Keyboard',
      vendorId: 0x1234,
      productId: 0x5678,
      hasProfile: false,
    ));
  });

  setUp(() {
    mockRegistry = MockServiceRegistry();
    mockEngine = MockEngineService();
    mockDevice = MockDeviceService();
    mockScriptFile = MockScriptFileService();
    mockTestService = MockTestService();
    mockTranslator = MockErrorTranslator();
    mockBridge = MockKeyrxBridge();

    // Setup engine state stream
    engineStateSubject = BehaviorSubject<EngineSnapshot>.seeded(
      EngineSnapshot(
        activeLayers: const [],
        heldKeys: const [],
        pendingDecisions: const [],
        timestamp: DateTime.now(),
      ),
    );

    // Wire up mocks
    when(() => mockRegistry.engineService).thenReturn(mockEngine);
    when(() => mockRegistry.deviceService).thenReturn(mockDevice);
    when(() => mockRegistry.scriptFileService).thenReturn(mockScriptFile);
    when(() => mockRegistry.testService).thenReturn(mockTestService);
    when(() => mockRegistry.errorTranslator).thenReturn(mockTranslator);
    when(() => mockRegistry.bridge).thenReturn(mockBridge);

    when(() => mockEngine.stateStream).thenAnswer((_) => engineStateSubject.stream);
    when(() => mockEngine.isInitialized).thenReturn(false);

    // Default translator behavior
    when(() => mockTranslator.translate(any())).thenReturn(
      const UserMessage(
        title: 'Error',
        body: 'An error occurred',
        category: MessageCategory.error,
      ),
    );
  });

  tearDown(() async {
    await engineStateSubject.close();
  });

  group('KeyrxFacadeImpl - Construction', () {
    test('creates with initial state', () {
      final facade = KeyrxFacadeImpl(mockRegistry);

      expect(facade.currentState.engine, EngineStatus.uninitialized);
      expect(facade.currentState.device, DeviceStatus.none);
      expect(facade.currentState.validation, ValidationStatus.none);
      expect(facade.currentState.discovery, DiscoveryStatus.idle);

      facade.dispose();
    });

    test('exposes services registry', () {
      final facade = KeyrxFacadeImpl(mockRegistry);

      expect(facade.services, mockRegistry);

      facade.dispose();
    });

    test('state stream emits initial state', () async {
      final facade = KeyrxFacadeImpl(mockRegistry);

      final firstState = await facade.stateStream.first;
      expect(firstState.engine, EngineStatus.uninitialized);

      await facade.dispose();
    });
  });

  group('KeyrxFacadeImpl - Engine Operations', () {
    test('startEngine: success flow initializes, loads, and starts engine', () async {
      final facade = KeyrxFacadeImpl(mockRegistry);
      const scriptPath = '/path/to/script.rhai';

      when(() => mockEngine.initialize()).thenAnswer((_) async => true);
      when(() => mockEngine.loadScript(any())).thenAnswer((_) async => true);

      final result = await facade.startEngine(scriptPath);

      expect(result.isOk, isTrue);
      expect(facade.currentState.engine, EngineStatus.running);
      expect(facade.currentState.scriptPath, scriptPath);

      verify(() => mockEngine.initialize()).called(1);
      verify(() => mockEngine.loadScript(scriptPath)).called(1);

      await facade.dispose();
    });

    test('startEngine: skips initialization if already initialized', () async {
      final facade = KeyrxFacadeImpl(mockRegistry);
      const scriptPath = '/path/to/script.rhai';

      when(() => mockEngine.isInitialized).thenReturn(true);
      when(() => mockEngine.loadScript(any())).thenAnswer((_) async => true);

      final result = await facade.startEngine(scriptPath);

      expect(result.isOk, isTrue);
      verifyNever(() => mockEngine.initialize());
      verify(() => mockEngine.loadScript(scriptPath)).called(1);

      await facade.dispose();
    });

    test('startEngine: returns error if initialization fails', () async {
      final facade = KeyrxFacadeImpl(mockRegistry);

      when(() => mockEngine.initialize()).thenAnswer((_) async => false);

      final result = await facade.startEngine('/path/to/script.rhai');

      expect(result.isErr, isTrue);
      expect(facade.currentState.engine, EngineStatus.error);

      final error = result.errOrNull!;
      expect(error.code, 'OPERATION_FAILED');
      expect(error.userMessage, contains('Failed to initialize'));

      await facade.dispose();
    });

    test('startEngine: returns error if script loading fails', () async {
      final facade = KeyrxFacadeImpl(mockRegistry);
      const scriptPath = '/path/to/script.rhai';

      when(() => mockEngine.initialize()).thenAnswer((_) async => true);
      when(() => mockEngine.loadScript(any())).thenAnswer((_) async => false);

      final result = await facade.startEngine(scriptPath);

      expect(result.isErr, isTrue);
      expect(facade.currentState.engine, EngineStatus.error);

      final error = result.errOrNull!;
      expect(error.code, 'OPERATION_FAILED');
      expect(error.userMessage, contains('Failed to load the script'));

      await facade.dispose();
    });

    test('startEngine: handles exceptions', () async {
      final facade = KeyrxFacadeImpl(mockRegistry);

      when(() => mockEngine.initialize()).thenThrow(Exception('Init failed'));

      final result = await facade.startEngine('/path/to/script.rhai');

      expect(result.isErr, isTrue);
      expect(facade.currentState.engine, EngineStatus.error);

      await facade.dispose();
    });

    test('stopEngine: succeeds when engine is running', () async {
      final facade = KeyrxFacadeImpl(mockRegistry);

      // Set engine to running state
      when(() => mockEngine.isInitialized).thenReturn(true);
      when(() => mockEngine.loadScript(any())).thenAnswer((_) async => true);
      await facade.startEngine('/path/to/script.rhai');

      final result = await facade.stopEngine();

      expect(result.isOk, isTrue);
      expect(facade.currentState.engine, EngineStatus.ready);

      await facade.dispose();
    });

    test('stopEngine: succeeds when engine is not running', () async {
      final facade = KeyrxFacadeImpl(mockRegistry);

      final result = await facade.stopEngine();

      expect(result.isOk, isTrue);

      await facade.dispose();
    });

    test('getEngineStatus: returns uninitialized when not initialized', () async {
      final facade = KeyrxFacadeImpl(mockRegistry);

      final result = await facade.getEngineStatus();

      expect(result.isOk, isTrue);
      expect(result.okOrNull, EngineStatus.uninitialized);

      await facade.dispose();
    });

    test('getEngineStatus: returns current engine status when initialized', () async {
      final facade = KeyrxFacadeImpl(mockRegistry);

      when(() => mockEngine.isInitialized).thenReturn(true);
      when(() => mockEngine.loadScript(any())).thenAnswer((_) async => true);
      await facade.startEngine('/path/to/script.rhai');

      final result = await facade.getEngineStatus();

      expect(result.isOk, isTrue);
      expect(result.okOrNull, EngineStatus.running);

      await facade.dispose();
    });
  });

  group('KeyrxFacadeImpl - Script Operations', () {
    test('validateScript: success with valid script', () async {
      final facade = KeyrxFacadeImpl(mockRegistry);
      const scriptPath = '/path/to/script.rhai';
      const scriptContent = 'fn test() {}';

      when(() => mockScriptFile.loadScript(scriptPath))
          .thenAnswer((_) async => scriptContent);
      when(() => mockBridge.validateScript(scriptContent)).thenReturn(
        ValidationResult(isValid: true, errors: const [], warnings: const []),
      );

      final result = await facade.validateScript(scriptPath);

      expect(result.isOk, isTrue);
      expect(result.okOrNull!.isValid, isTrue);
      expect(facade.currentState.validation, ValidationStatus.valid);
      expect(facade.currentState.validationErrorCount, 0);
      expect(facade.currentState.validationWarningCount, 0);

      await facade.dispose();
    });

    test('validateScript: success with warnings', () async {
      final facade = KeyrxFacadeImpl(mockRegistry);
      const scriptPath = '/path/to/script.rhai';

      when(() => mockScriptFile.loadScript(scriptPath))
          .thenAnswer((_) async => 'script');
      when(() => mockBridge.validateScript(any())).thenReturn(
        ValidationResult(
          isValid: true,
          errors: const [],
          warnings: const [
            ValidationWarning(
              code: 'W001',
              category: WarningCategory.safety,
              message: 'Unused variable',
              location: SourceLocation(line: 10, column: 5),
            ),
          ],
        ),
      );

      final result = await facade.validateScript(scriptPath);

      expect(result.isOk, isTrue);
      expect(result.okOrNull!.isValid, isTrue);
      expect(result.okOrNull!.warnings.length, 1);
      expect(facade.currentState.validation, ValidationStatus.validWithWarnings);
      expect(facade.currentState.validationWarningCount, 1);

      await facade.dispose();
    });

    test('validateScript: invalid script with errors', () async {
      final facade = KeyrxFacadeImpl(mockRegistry);
      const scriptPath = '/path/to/script.rhai';

      when(() => mockScriptFile.loadScript(scriptPath))
          .thenAnswer((_) async => 'invalid script');
      when(() => mockBridge.validateScript(any())).thenReturn(
        ValidationResult(
          isValid: false,
          errors: const [
            ValidationError(
              code: 'E001',
              message: 'Syntax error',
              location: SourceLocation(line: 5, column: 10),
              suggestions: [],
            ),
          ],
          warnings: const [],
        ),
      );

      final result = await facade.validateScript(scriptPath);

      expect(result.isOk, isTrue);
      expect(result.okOrNull!.isValid, isFalse);
      expect(result.okOrNull!.errors.length, 1);
      expect(facade.currentState.validation, ValidationStatus.invalid);
      expect(facade.currentState.validationErrorCount, 1);

      await facade.dispose();
    });

    test('validateScript: returns error if file not found', () async {
      final facade = KeyrxFacadeImpl(mockRegistry);

      when(() => mockScriptFile.loadScript(any())).thenAnswer((_) async => null);

      final result = await facade.validateScript('/nonexistent.rhai');

      expect(result.isErr, isTrue);
      expect(facade.currentState.validation, ValidationStatus.none);

      final error = result.errOrNull!;
      expect(error.code, 'OPERATION_FAILED');

      await facade.dispose();
    });

    test('loadScriptContent: success', () async {
      final facade = KeyrxFacadeImpl(mockRegistry);
      const content = 'fn main() { print("Hello"); }';

      when(() => mockScriptFile.loadScript(any())).thenAnswer((_) async => content);

      final result = await facade.loadScriptContent('/path/to/script.rhai');

      expect(result.isOk, isTrue);
      expect(result.okOrNull, content);

      await facade.dispose();
    });

    test('loadScriptContent: returns error if file not found', () async {
      final facade = KeyrxFacadeImpl(mockRegistry);

      when(() => mockScriptFile.loadScript(any())).thenAnswer((_) async => null);

      final result = await facade.loadScriptContent('/nonexistent.rhai');

      expect(result.isErr, isTrue);

      final error = result.errOrNull!;
      expect(error.code, 'OPERATION_FAILED');

      await facade.dispose();
    });

    test('saveScript: success', () async {
      final facade = KeyrxFacadeImpl(mockRegistry);

      when(() => mockScriptFile.saveScript(any(), any())).thenAnswer(
        (_) async => const ScriptFileResult(success: true),
      );

      final result = await facade.saveScript('/path/to/script.rhai', 'content');

      expect(result.isOk, isTrue);

      await facade.dispose();
    });

    test('saveScript: returns error on failure', () async {
      final facade = KeyrxFacadeImpl(mockRegistry);

      when(() => mockScriptFile.saveScript(any(), any())).thenAnswer(
        (_) async => const ScriptFileResult(
          success: false,
          errorMessage: 'Disk full',
        ),
      );

      final result = await facade.saveScript('/path/to/script.rhai', 'content');

      expect(result.isErr, isTrue);

      final error = result.errOrNull!;
      expect(error.userMessage, contains('Disk full'));

      await facade.dispose();
    });
  });

  group('KeyrxFacadeImpl - Device Operations', () {
    test('listDevices: success with devices', () async {
      final facade = KeyrxFacadeImpl(mockRegistry);
      final devices = [
        const KeyboardDevice(
          path: '/dev/input/event0',
          name: 'Keyboard 1',
          vendorId: 0x1234,
          productId: 0x5678,
          hasProfile: false,
        ),
        const KeyboardDevice(
          path: '/dev/input/event1',
          name: 'Keyboard 2',
          vendorId: 0xABCD,
          productId: 0xEF00,
          hasProfile: true,
        ),
      ];

      when(() => mockDevice.refresh()).thenAnswer((_) async => devices);

      final result = await facade.listDevices();

      expect(result.isOk, isTrue);
      expect(result.okOrNull, devices);
      expect(facade.currentState.device, DeviceStatus.available);

      await facade.dispose();
    });

    test('listDevices: success with no devices', () async {
      final facade = KeyrxFacadeImpl(mockRegistry);

      when(() => mockDevice.refresh()).thenAnswer((_) async => []);

      final result = await facade.listDevices();

      expect(result.isOk, isTrue);
      expect(result.okOrNull, isEmpty);
      expect(facade.currentState.device, DeviceStatus.none);

      await facade.dispose();
    });

    test('listDevices: handles exceptions', () async {
      final facade = KeyrxFacadeImpl(mockRegistry);

      when(() => mockDevice.refresh()).thenThrow(Exception('USB error'));

      final result = await facade.listDevices();

      expect(result.isErr, isTrue);
      expect(facade.currentState.device, DeviceStatus.error);

      await facade.dispose();
    });

    test('selectDevice: success', () async {
      final facade = KeyrxFacadeImpl(mockRegistry);
      const devicePath = '/dev/input/event0';

      when(() => mockDevice.selectDevice(devicePath)).thenAnswer(
        (_) async => DeviceSelectionResult.success(),
      );

      final result = await facade.selectDevice(devicePath);

      expect(result.isOk, isTrue);
      expect(facade.currentState.device, DeviceStatus.connected);
      expect(facade.currentState.selectedDevicePath, devicePath);

      await facade.dispose();
    });

    test('selectDevice: returns error on failure', () async {
      final facade = KeyrxFacadeImpl(mockRegistry);

      when(() => mockDevice.selectDevice(any())).thenAnswer(
        (_) async => DeviceSelectionResult.error('Device not found'),
      );

      final result = await facade.selectDevice('/dev/input/event99');

      expect(result.isErr, isTrue);
      expect(facade.currentState.device, DeviceStatus.error);

      await facade.dispose();
    });

    test('startDiscovery: success', () async {
      final facade = KeyrxFacadeImpl(mockRegistry);
      const device = KeyboardDevice(
        path: '/dev/input/event0',
        name: 'Test',
        vendorId: 0x1234,
        productId: 0x5678,
        hasProfile: false,
      );

      when(() => mockBridge.startDiscovery(any(), any(), any())).thenReturn(
        const DiscoveryStartResult(success: true, totalKeys: 61),
      );

      final result = await facade.startDiscovery(
        device: device,
        rows: 5,
        colsPerRow: [13, 13, 12, 12, 11],
      );

      expect(result.isOk, isTrue);
      expect(facade.currentState.discovery, DiscoveryStatus.active);
      expect(facade.currentState.discoveredDeviceCount, 61);

      await facade.dispose();
    });

    test('startDiscovery: returns error on failure', () async {
      final facade = KeyrxFacadeImpl(mockRegistry);
      const device = KeyboardDevice(
        path: '/dev/input/event0',
        name: 'Test',
        vendorId: 0x1234,
        productId: 0x5678,
        hasProfile: false,
      );

      when(() => mockBridge.startDiscovery(any(), any(), any())).thenReturn(
        DiscoveryStartResult.error('Device busy'),
      );

      final result = await facade.startDiscovery(
        device: device,
        rows: 5,
        colsPerRow: [13, 13, 12, 12, 11],
      );

      expect(result.isErr, isTrue);
      expect(facade.currentState.discovery, DiscoveryStatus.error);

      await facade.dispose();
    });

    test('cancelDiscovery: success when discovery active', () async {
      final facade = KeyrxFacadeImpl(mockRegistry);
      const device = KeyboardDevice(
        path: '/dev/input/event0',
        name: 'Test',
        vendorId: 0x1234,
        productId: 0x5678,
        hasProfile: false,
      );

      // Start discovery first
      when(() => mockBridge.startDiscovery(any(), any(), any())).thenReturn(
        const DiscoveryStartResult(success: true, totalKeys: 61),
      );
      await facade.startDiscovery(
        device: device,
        rows: 5,
        colsPerRow: [13, 13, 12, 12, 11],
      );

      when(() => mockBridge.cancelDiscovery()).thenReturn(0);

      final result = await facade.cancelDiscovery();

      expect(result.isOk, isTrue);
      expect(facade.currentState.discovery, DiscoveryStatus.cancelled);

      await facade.dispose();
    });

    test('cancelDiscovery: succeeds when no discovery active', () async {
      final facade = KeyrxFacadeImpl(mockRegistry);

      final result = await facade.cancelDiscovery();

      expect(result.isOk, isTrue);

      await facade.dispose();
    });
  });

  group('KeyrxFacadeImpl - Test Operations', () {
    test('discoverTests: success', () async {
      final facade = KeyrxFacadeImpl(mockRegistry);
      const testResult = TestDiscoveryServiceResult(
        tests: [
          TestCase(name: 'test_addition', file: '/path/to/script.rhai', line: 10),
          TestCase(name: 'test_subtraction', file: '/path/to/script.rhai', line: 20),
        ],
      );

      when(() => mockTestService.discoverTests(any()))
          .thenAnswer((_) async => testResult);

      final result = await facade.discoverTests('/path/to/script.rhai');

      expect(result.isOk, isTrue);
      expect(result.okOrNull!.tests.length, 2);

      await facade.dispose();
    });

    test('discoverTests: returns error on failure', () async {
      final facade = KeyrxFacadeImpl(mockRegistry);

      when(() => mockTestService.discoverTests(any())).thenAnswer(
        (_) async => TestDiscoveryServiceResult.error('Parse error'),
      );

      final result = await facade.discoverTests('/path/to/script.rhai');

      expect(result.isErr, isTrue);

      await facade.dispose();
    });

    test('runTests: success', () async {
      final facade = KeyrxFacadeImpl(mockRegistry);
      const testResult = TestRunServiceResult(
        total: 3,
        passed: 2,
        failed: 1,
        durationMs: 150.0,
        results: [
          TestCaseResult(name: 'test1', passed: true, durationMs: 50.0),
          TestCaseResult(name: 'test2', passed: true, durationMs: 50.0),
          TestCaseResult(
            name: 'test3',
            passed: false,
            durationMs: 50.0,
            error: 'Assertion failed',
          ),
        ],
      );

      when(() => mockTestService.runTests(any(), filter: any(named: 'filter')))
          .thenAnswer((_) async => testResult);

      final result = await facade.runTests('/path/to/script.rhai');

      expect(result.isOk, isTrue);
      expect(result.okOrNull!.total, 3);
      expect(result.okOrNull!.passed, 2);
      expect(result.okOrNull!.failed, 1);

      await facade.dispose();
    });

    test('runTests: with filter', () async {
      final facade = KeyrxFacadeImpl(mockRegistry);

      when(() => mockTestService.runTests(any(), filter: any(named: 'filter')))
          .thenAnswer(
        (_) async => const TestRunServiceResult(
          total: 1,
          passed: 1,
          failed: 0,
          durationMs: 50.0,
          results: [
            TestCaseResult(name: 'test_addition', passed: true, durationMs: 50.0),
          ],
        ),
      );

      final result = await facade.runTests(
        '/path/to/script.rhai',
        filter: 'addition',
      );

      expect(result.isOk, isTrue);
      expect(result.okOrNull!.total, 1);

      verify(() => mockTestService.runTests(
            '/path/to/script.rhai',
            filter: 'addition',
          )).called(1);

      await facade.dispose();
    });

    test('runTests: returns error on failure', () async {
      final facade = KeyrxFacadeImpl(mockRegistry);

      when(() => mockTestService.runTests(any(), filter: any(named: 'filter')))
          .thenAnswer(
        (_) async => TestRunServiceResult.error('Script compilation failed'),
      );

      final result = await facade.runTests('/path/to/script.rhai');

      expect(result.isErr, isTrue);

      await facade.dispose();
    });

    test('cancelTests: succeeds (no-op for now)', () async {
      final facade = KeyrxFacadeImpl(mockRegistry);

      final result = await facade.cancelTests();

      expect(result.isOk, isTrue);

      await facade.dispose();
    });
  });

  group('KeyrxFacadeImpl - State Stream', () {
    test('state stream aggregates engine state changes', () async {
      final facade = KeyrxFacadeImpl(mockRegistry);

      // Capture state emissions
      final states = <FacadeState>[];
      final subscription = facade.stateStream.listen(states.add);

      // Allow initial state to emit
      await Future<void>.delayed(const Duration(milliseconds: 200));

      // Emit engine state change
      engineStateSubject.add(
        EngineSnapshot(
          activeLayers: const ['base'],
          heldKeys: const [],
          pendingDecisions: const [],
          timestamp: DateTime.now(),
        ),
      );

      // Wait for debounce + processing (100ms debounce + buffer)
      await Future<void>.delayed(const Duration(milliseconds: 250));

      // Should have received initial state and update
      expect(states.length, greaterThanOrEqualTo(1));

      // Find a state with running engine
      final runningState = states.firstWhere(
        (s) => s.engine == EngineStatus.running,
        orElse: () => states.last,
      );
      expect(runningState.engine, EngineStatus.running);

      await subscription.cancel();
      await facade.dispose();
    });

    test('state stream debounces rapid changes', () async {
      final facade = KeyrxFacadeImpl(mockRegistry);

      final states = <FacadeState>[];
      final subscription = facade.stateStream.listen(states.add);

      // Allow initial state
      await Future<void>.delayed(const Duration(milliseconds: 150));
      final initialCount = states.length;

      // Emit multiple rapid changes
      for (var i = 0; i < 10; i++) {
        engineStateSubject.add(
          EngineSnapshot(
            activeLayers: ['layer$i'],
            heldKeys: const [],
            pendingDecisions: const [],
            timestamp: DateTime.now(),
          ),
        );
      }

      // Wait for debounce
      await Future<void>.delayed(const Duration(milliseconds: 150));

      // Should only emit once due to debounce
      expect(states.length, lessThan(initialCount + 10));

      await subscription.cancel();
      await facade.dispose();
    });

    test('state stream filters out duplicate states', () async {
      final facade = KeyrxFacadeImpl(mockRegistry);

      final states = <FacadeState>[];
      final subscription = facade.stateStream.listen(states.add);

      await Future<void>.delayed(const Duration(milliseconds: 150));
      final initialCount = states.length;

      // Emit same snapshot multiple times
      final snapshot = EngineSnapshot(
        activeLayers: const [],
        heldKeys: const [],
        pendingDecisions: const [],
        timestamp: DateTime.now(),
      );

      for (var i = 0; i < 5; i++) {
        engineStateSubject.add(snapshot);
        await Future<void>.delayed(const Duration(milliseconds: 50));
      }

      await Future<void>.delayed(const Duration(milliseconds: 150));

      // Should not emit duplicates
      expect(states.length, equals(initialCount));

      await subscription.cancel();
      await facade.dispose();
    });
  });

  group('KeyrxFacadeImpl - Lifecycle', () {
    test('dispose cancels subscriptions and closes state stream', () async {
      final facade = KeyrxFacadeImpl(mockRegistry);

      // Subscribe to state
      final subscription = facade.stateStream.listen((_) {});

      await facade.dispose();

      // Trying to use facade after disposal should throw
      expect(
        () => facade.startEngine('/path/to/script.rhai'),
        throwsStateError,
      );

      await subscription.cancel();
    });

    test('dispose can be called multiple times safely', () async {
      final facade = KeyrxFacadeImpl(mockRegistry);

      await facade.dispose();
      await facade.dispose(); // Should not throw

      expect(true, isTrue); // If we get here, test passed
    });

    test('operations throw StateError after disposal', () async {
      final facade = KeyrxFacadeImpl(mockRegistry);

      await facade.dispose();

      expect(() => facade.startEngine('/path'), throwsStateError);
      expect(() => facade.stopEngine(), throwsStateError);
      expect(() => facade.getEngineStatus(), throwsStateError);
      expect(() => facade.validateScript('/path'), throwsStateError);
      expect(() => facade.loadScriptContent('/path'), throwsStateError);
      expect(() => facade.saveScript('/path', 'content'), throwsStateError);
      expect(() => facade.listDevices(), throwsStateError);
      expect(() => facade.selectDevice('/dev/input/event0'), throwsStateError);
      expect(
        () => facade.startDiscovery(
          device: const KeyboardDevice(
            path: '/dev/input/event0',
            name: 'Test',
            vendorId: 0x1234,
            productId: 0x5678,
            hasProfile: false,
          ),
          rows: 5,
          colsPerRow: [13, 13, 12, 12, 11],
        ),
        throwsStateError,
      );
      expect(() => facade.cancelDiscovery(), throwsStateError);
      expect(() => facade.discoverTests('/path'), throwsStateError);
      expect(() => facade.runTests('/path'), throwsStateError);
      expect(() => facade.cancelTests(), throwsStateError);
    });
  });

  group('KeyrxFacadeImpl - Error Handling', () {
    test('translates exceptions to user-friendly errors', () async {
      final facade = KeyrxFacadeImpl(mockRegistry);

      when(() => mockEngine.initialize())
          .thenThrow(Exception('Low-level FFI error'));
      when(() => mockTranslator.translate(any())).thenReturn(
        const UserMessage(
          title: 'Engine Error',
          body: 'The engine could not be initialized',
          category: MessageCategory.error,
        ),
      );

      final result = await facade.startEngine('/path/to/script.rhai');

      expect(result.isErr, isTrue);
      final error = result.errOrNull!;
      expect(error.userMessage, contains('Engine Error'));

      await facade.dispose();
    });

    test('preserves original error in FacadeError', () async {
      final facade = KeyrxFacadeImpl(mockRegistry);
      final originalError = Exception('Original error');

      when(() => mockEngine.initialize()).thenThrow(originalError);

      final result = await facade.startEngine('/path/to/script.rhai');

      expect(result.isErr, isTrue);
      final error = result.errOrNull!;
      expect(error.originalError, originalError);

      await facade.dispose();
    });
  });

  group('KeyrxFacadeImpl - Factory', () {
    test('KeyrxFacade.real creates KeyrxFacadeImpl', () {
      final facade = KeyrxFacade.real(mockRegistry);

      expect(facade, isA<KeyrxFacadeImpl>());
      expect(facade.services, mockRegistry);

      facade.dispose();
    });
  });
}
