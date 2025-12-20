/// Platform consistency tests to verify SSOT compliance.
///
/// This test suite ensures:
/// 1. FFI and Web implementations conform to the same interfaces
/// 2. Transport layers contain no business logic
/// 3. Real-time event parity across platforms
/// 4. Interface method signatures match exactly
library;

import 'dart:async';

import 'package:core_logic/ffi/bridge.dart';
import 'package:core_logic/models/hardware_profile.dart';
import 'package:core_logic/models/keymap.dart';
import 'package:core_logic/models/virtual_layout.dart';
import 'package:core_logic/services/config_result.dart';
import 'package:core_logic/services/hardware_service.dart';
import 'package:core_logic/services/hardware_service_web.dart';
import 'package:core_logic/services/keymap_service.dart';
import 'package:core_logic/services/keymap_service_web.dart';
import 'package:core_logic/services/layout_service.dart';
import 'package:core_logic/services/layout_service_web.dart';
import 'package:core_logic/web/api_client.dart';
import 'package:core_logic/web/websocket_client.dart';
import 'package:mocktail/mocktail.dart';
import 'package:test/test.dart';

// Mocks
class MockKeyrxBridge extends Mock implements KeyrxBridge {}

class MockWebApiClient extends Mock implements WebApiClient {}

class MockWebSocketClient extends Mock implements WebSocketClient {}

void main() {
  group('Interface Consistency', () {
    late MockWebApiClient mockApiClient;

    setUp(() {
      mockApiClient = MockWebApiClient();
      // Mock subscribe to return an empty broadcast stream
      when(
        () => mockApiClient.subscribe<void>(any(), any()),
      ).thenAnswer((_) => Stream<void>.empty());
    });

    test('HardwareService - FFI and Web implement same interface', () {
      final mockBridge = MockKeyrxBridge();

      final ffiService = HardwareServiceFFI(bridge: mockBridge);
      final webService = HardwareServiceWeb(client: mockApiClient);

      // Both must be HardwareService
      expect(ffiService, isA<HardwareService>());
      expect(webService, isA<HardwareService>());

      ffiService.dispose();
      webService.dispose();
    });

    test('KeymapService - FFI and Web implement same interface', () {
      final mockBridge = MockKeyrxBridge();

      final ffiService = KeymapServiceFFI(bridge: mockBridge);
      final webService = KeymapServiceWeb(client: mockApiClient);

      // Both must be KeymapService
      expect(ffiService, isA<KeymapService>());
      expect(webService, isA<KeymapService>());
    });

    test('LayoutService - FFI and Web implement same interface', () {
      final mockBridge = MockKeyrxBridge();

      final ffiService = LayoutServiceFFI(bridge: mockBridge);
      final webService = LayoutServiceWeb(client: mockApiClient);

      // Both must be LayoutService
      expect(ffiService, isA<LayoutService>());
      expect(webService, isA<LayoutService>());

      ffiService.dispose();
      webService.dispose();
    });
  });

  group('Transport Layer Isolation', () {
    late MockKeyrxBridge mockBridge;
    late MockWebApiClient mockApiClient;

    setUp(() {
      mockBridge = MockKeyrxBridge();
      mockApiClient = MockWebApiClient();

      // Set up default mocks to prevent null errors
      when(() => mockBridge.loadFailure).thenReturn(null);
      when(() => mockBridge.configBindings).thenReturn(null);

      // Mock subscribe to return an empty broadcast stream
      when(
        () => mockApiClient.subscribe<void>(any(), any()),
      ).thenAnswer((_) => Stream<void>.empty());
    });

    test('HardwareService Web - only makes HTTP calls, no business logic', () async {
      final service = HardwareServiceWeb(client: mockApiClient);

      // Mock API response
      when(
        () => mockApiClient.get('/api/hardware/profiles'),
      ).thenAnswer((_) async => const ApiResponse(data: []));

      // Call service method
      await service.listProfiles();

      // Verify it only calls the transport layer (HTTP GET)
      verify(() => mockApiClient.get('/api/hardware/profiles')).called(1);

      // Also verify WebSocket subscription was set up during initialization
      verify(() => mockApiClient.subscribe<void>('hardware.changed', any())).called(1);

      service.dispose();
    });

    test('HardwareService Web - save profile makes POST call', () async {
      final service = HardwareServiceWeb(client: mockApiClient);

      final profile = HardwareProfile(
        id: 'test-id',
        virtualLayoutId: 'test-layout-id',
        vendorId: 0x1234,
        productId: 0x5678,
        name: 'Test Profile',
      );

      // Mock API response
      when(
        () => mockApiClient.post('/api/hardware/profiles', body: any(named: 'body')),
      ).thenAnswer((_) async => ApiResponse(data: profile.toJson()));

      // Call service method
      await service.saveProfile(profile);

      // Verify POST was called with correct endpoint
      verify(
        () => mockApiClient.post('/api/hardware/profiles', body: {'profile': profile.toJson()}),
      ).called(1);

      service.dispose();
    });

    test('HardwareService Web - delete profile makes DELETE call', () async {
      final service = HardwareServiceWeb(client: mockApiClient);

      // Mock API response
      when(
        () => mockApiClient.delete('/api/hardware/profiles/test-id'),
      ).thenAnswer((_) async => const ApiResponse(data: null));

      // Call service method
      await service.deleteProfile('test-id');

      // Verify DELETE was called
      verify(() => mockApiClient.delete('/api/hardware/profiles/test-id')).called(1);

      service.dispose();
    });

    test('KeymapService Web - only makes HTTP calls, no business logic', () async {
      final service = KeymapServiceWeb(client: mockApiClient);

      // Mock API response
      when(
        () => mockApiClient.get('/api/keymaps'),
      ).thenAnswer((_) async => const ApiResponse(data: []));

      // Call service method
      await service.listKeymaps();

      // Verify it only calls the transport layer
      verify(() => mockApiClient.get('/api/keymaps')).called(1);
    });

    test('LayoutService Web - only makes HTTP calls, no business logic', () async {
      final service = LayoutServiceWeb(client: mockApiClient);

      // Mock API response
      when(
        () => mockApiClient.get('/api/layouts'),
      ).thenAnswer((_) async => const ApiResponse(data: []));

      // Call service method
      await service.listLayouts();

      // Verify it only calls the transport layer
      verify(() => mockApiClient.get('/api/layouts')).called(1);

      service.dispose();
    });
  });

  group('Real-time Event Parity', () {
    late MockWebApiClient mockApiClient;
    late StreamController<void> wsStreamController;

    setUp(() {
      mockApiClient = MockWebApiClient();
      wsStreamController = StreamController<void>.broadcast();

      // Mock WebSocket subscription
      when(
        () => mockApiClient.subscribe<void>(any(), any()),
      ).thenAnswer((_) => wsStreamController.stream);
    });

    tearDown(() {
      wsStreamController.close();
    });

    test('HardwareService Web - subscribes to hardware.changed events', () async {
      final service = HardwareServiceWeb(client: mockApiClient);

      // Verify subscription was created
      verify(() => mockApiClient.subscribe<void>('hardware.changed', any())).called(1);

      final events = <void>[];
      service.changes.listen(events.add);

      // Simulate WebSocket event
      wsStreamController.add(null);
      await Future<void>.delayed(const Duration(milliseconds: 10));

      // Verify event was forwarded to changes stream
      expect(events, hasLength(1));

      service.dispose();
    });

    test('LayoutService Web - subscribes to layout.changed events', () async {
      final service = LayoutServiceWeb(client: mockApiClient);

      // Verify subscription was created
      verify(() => mockApiClient.subscribe<void>('layout.changed', any())).called(1);

      final events = <void>[];
      service.changes.listen(events.add);

      // Simulate WebSocket event
      wsStreamController.add(null);
      await Future<void>.delayed(const Duration(milliseconds: 10));

      // Verify event was forwarded to changes stream
      expect(events, hasLength(1));

      service.dispose();
    });

    test('Real-time events trigger same stream behavior across platforms', () async {
      // This test verifies that both FFI and Web services expose
      // changes through the same Stream<void> interface

      final mockBridge = MockKeyrxBridge();
      when(() => mockBridge.loadFailure).thenReturn(null);

      final ffiService = HardwareServiceFFI(bridge: mockBridge);
      final webService = HardwareServiceWeb(client: mockApiClient);

      // Both expose changes stream
      expect(ffiService.changes, isA<Stream<void>>());
      expect(webService.changes, isA<Stream<void>>());

      // Both are broadcast streams (multiple listeners allowed)
      expect(ffiService.changes.isBroadcast, isTrue);
      expect(webService.changes.isBroadcast, isTrue);

      ffiService.dispose();
      webService.dispose();
    });
  });

  group('Interface Method Verification', () {
    late MockWebApiClient mockApiClient;

    setUp(() {
      mockApiClient = MockWebApiClient();
      // Mock subscribe to return an empty broadcast stream
      when(
        () => mockApiClient.subscribe<void>(any(), any()),
      ).thenAnswer((_) => Stream<void>.empty());
    });

    test('HardwareService - FFI and Web have identical method signatures', () {
      final mockBridge = MockKeyrxBridge();

      final ffiService = HardwareServiceFFI(bridge: mockBridge);
      final webService = HardwareServiceWeb(client: mockApiClient);

      // Verify both have the same public API
      // This is enforced by implementing the same abstract class

      // changes stream
      expect(ffiService.changes, isA<Stream<void>>());
      expect(webService.changes, isA<Stream<void>>());

      // listProfiles method
      expect(
        ffiService.listProfiles,
        isA<Future<ConfigOperationResult<List<HardwareProfile>>> Function()>(),
      );
      expect(
        webService.listProfiles,
        isA<Future<ConfigOperationResult<List<HardwareProfile>>> Function()>(),
      );

      // saveProfile method
      expect(
        ffiService.saveProfile,
        isA<Future<ConfigOperationResult<HardwareProfile>> Function(HardwareProfile)>(),
      );
      expect(
        webService.saveProfile,
        isA<Future<ConfigOperationResult<HardwareProfile>> Function(HardwareProfile)>(),
      );

      // deleteProfile method
      expect(ffiService.deleteProfile, isA<Future<ConfigOperationResult<void>> Function(String)>());
      expect(webService.deleteProfile, isA<Future<ConfigOperationResult<void>> Function(String)>());

      // dispose method
      expect(ffiService.dispose, isA<void Function()>());
      expect(webService.dispose, isA<void Function()>());

      ffiService.dispose();
      webService.dispose();
    });

    test('KeymapService - FFI and Web have identical method signatures', () {
      final mockBridge = MockKeyrxBridge();

      final ffiService = KeymapServiceFFI(bridge: mockBridge);
      final webService = KeymapServiceWeb(client: mockApiClient);

      // listKeymaps method
      expect(ffiService.listKeymaps, isA<Future<ConfigOperationResult<List<Keymap>>> Function()>());
      expect(webService.listKeymaps, isA<Future<ConfigOperationResult<List<Keymap>>> Function()>());

      // saveKeymap method
      expect(ffiService.saveKeymap, isA<Future<ConfigOperationResult<Keymap>> Function(Keymap)>());
      expect(webService.saveKeymap, isA<Future<ConfigOperationResult<Keymap>> Function(Keymap)>());

      // deleteKeymap method
      expect(ffiService.deleteKeymap, isA<Future<ConfigOperationResult<void>> Function(String)>());
      expect(webService.deleteKeymap, isA<Future<ConfigOperationResult<void>> Function(String)>());
    });

    test('LayoutService - FFI and Web have identical method signatures', () {
      final mockBridge = MockKeyrxBridge();

      final ffiService = LayoutServiceFFI(bridge: mockBridge);
      final webService = LayoutServiceWeb(client: mockApiClient);

      // changes stream
      expect(ffiService.changes, isA<Stream<void>>());
      expect(webService.changes, isA<Stream<void>>());

      // listLayouts method
      expect(
        ffiService.listLayouts,
        isA<Future<ConfigOperationResult<List<VirtualLayout>>> Function()>(),
      );
      expect(
        webService.listLayouts,
        isA<Future<ConfigOperationResult<List<VirtualLayout>>> Function()>(),
      );

      // saveLayout method
      expect(
        ffiService.saveLayout,
        isA<Future<ConfigOperationResult<VirtualLayout>> Function(VirtualLayout)>(),
      );
      expect(
        webService.saveLayout,
        isA<Future<ConfigOperationResult<VirtualLayout>> Function(VirtualLayout)>(),
      );

      // deleteLayout method
      expect(ffiService.deleteLayout, isA<Future<ConfigOperationResult<void>> Function(String)>());
      expect(webService.deleteLayout, isA<Future<ConfigOperationResult<void>> Function(String)>());

      // dispose method
      expect(ffiService.dispose, isA<void Function()>());
      expect(webService.dispose, isA<void Function()>());

      ffiService.dispose();
      webService.dispose();
    });
  });

  group('Error Handling Parity', () {
    late MockWebApiClient mockApiClient;

    setUp(() {
      mockApiClient = MockWebApiClient();

      // Mock subscribe to return an empty broadcast stream
      when(
        () => mockApiClient.subscribe<void>(any(), any()),
      ).thenAnswer((_) => Stream<void>.empty());
    });

    test('Web services return ConfigOperationResult on error', () async {
      final service = HardwareServiceWeb(client: mockApiClient);

      // Mock API error
      when(
        () => mockApiClient.get('/api/hardware/profiles'),
      ).thenAnswer((_) async => const ApiResponse(data: null, error: 'Network error'));

      final result = await service.listProfiles();

      // Verify error is wrapped in ConfigOperationResult
      expect(result.hasError, isTrue);
      expect(result.errorMessage, contains('Network error'));
      expect(result.data, isNull);

      service.dispose();
    });

    test('Web services handle JSON parse errors', () async {
      final service = KeymapServiceWeb(client: mockApiClient);

      // Mock API response with invalid data
      when(
        () => mockApiClient.get('/api/keymaps'),
      ).thenAnswer((_) async => const ApiResponse(data: 'invalid'));

      final result = await service.listKeymaps();

      // Verify parse error is caught and wrapped
      expect(result.hasError, isTrue);
      expect(result.errorMessage, contains('Failed to parse'));
    });

    test('Both platforms return same ConfigOperationResult type', () async {
      final mockBridge = MockKeyrxBridge();
      when(() => mockBridge.loadFailure).thenReturn('Test error');

      final ffiService = HardwareServiceFFI(bridge: mockBridge);
      final webService = HardwareServiceWeb(client: mockApiClient);

      when(
        () => mockApiClient.get('/api/hardware/profiles'),
      ).thenAnswer((_) async => const ApiResponse(data: null, error: 'Test error'));

      final ffiResult = await ffiService.listProfiles();
      final webResult = await webService.listProfiles();

      // Both return same result type with errors
      expect(ffiResult, isA<ConfigOperationResult<List<HardwareProfile>>>());
      expect(webResult, isA<ConfigOperationResult<List<HardwareProfile>>>());
      expect(ffiResult.hasError, isTrue);
      expect(webResult.hasError, isTrue);

      ffiService.dispose();
      webService.dispose();
    });
  });
}
