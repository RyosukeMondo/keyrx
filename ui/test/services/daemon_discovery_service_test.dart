// ignore_for_file: unawaited_futures

import 'dart:async';

import 'package:flutter_test/flutter_test.dart';
import 'package:keyrx_ui/models/daemon_protocol.dart';
import 'package:keyrx_ui/services/daemon_discovery_service.dart';
import 'package:keyrx_ui/services/daemon_service.dart';
import 'package:mockito/annotations.dart';
import 'package:mockito/mockito.dart';

import 'daemon_discovery_service_test.mocks.dart';

@GenerateMocks([DaemonService])
void main() {
  late MockDaemonService mockDaemonService;
  late DaemonDiscoveryService discoveryService;
  late StreamController<DaemonEvent> daemonEventController;

  setUp(() {
    mockDaemonService = MockDaemonService();
    daemonEventController = StreamController<DaemonEvent>.broadcast();
    when(
      mockDaemonService.events,
    ).thenAnswer((_) => daemonEventController.stream);
    discoveryService = DaemonDiscoveryService(mockDaemonService);
  });

  tearDown(() {
    daemonEventController.close();
    discoveryService.dispose();
  });

  test('startDiscovery calls the correct daemon method', () async {
    when(
      mockDaemonService.sendRequest<void>(
        DaemonMethods.discoveryStart,
        parseResult: anyNamed('parseResult'),
      ),
    ).thenAnswer((_) async {});

    await discoveryService.startDiscovery();

    verify(
      mockDaemonService.sendRequest<void>(
        DaemonMethods.discoveryStart,
        parseResult: anyNamed('parseResult'),
      ),
    ).called(1);
    expect(discoveryService.isDiscovering, isTrue);
  });

  test('stopDiscovery calls the correct daemon method', () async {
    // Start discovery first
    when(
      mockDaemonService.sendRequest<void>(
        DaemonMethods.discoveryStart,
        parseResult: anyNamed('parseResult'),
      ),
    ).thenAnswer((_) async {});
    await discoveryService.startDiscovery();

    // Then stop it
    when(
      mockDaemonService.sendRequest<void>(
        DaemonMethods.discoveryStop,
        parseResult: anyNamed('parseResult'),
      ),
    ).thenAnswer((_) async {});
    await discoveryService.stopDiscovery();

    verify(
      mockDaemonService.sendRequest<void>(
        DaemonMethods.discoveryStop,
        parseResult: anyNamed('parseResult'),
      ),
    ).called(1);
    expect(discoveryService.isDiscovering, isFalse);
  });

  test('discoveryEvents stream emits DiscoveryCandidate events', () async {
    const candidate = DiscoveryCandidate(
      deviceId: 'test_device',
      vendorId: 1234,
      productId: 5678,
      name: 'Test Device',
    );

    final discoveryEvents = <DiscoveryCandidate>[];
    discoveryService.discoveryEvents.listen(discoveryEvents.add);

    daemonEventController.add(const DaemonEvent.discovery(candidate));

    await Future<void>.delayed(Duration.zero);

    expect(discoveryEvents, hasLength(1));
    expect(discoveryEvents.first, equals(candidate));
  });

  test('non-discovery events are ignored', () async {
    const inputEvent = InputEvent(
      timestampUs: 0,
      devicePath: 'test_path',
      deviceName: 'test_device',
      eventType: 'key_press',
      keyCode: 0,
      keyName: 'a',
    );

    final discoveryEvents = <DiscoveryCandidate>[];
    discoveryService.discoveryEvents.listen(discoveryEvents.add);

    daemonEventController.add(const DaemonEvent.input(inputEvent));

    await Future<void>.delayed(Duration.zero);

    expect(discoveryEvents, isEmpty);
  });
}
