import 'dart:io';
import 'package:core_logic/ffi/bridge.dart';
import 'package:core_logic/ffi/mock_bridge.dart';
import 'package:core_logic/models/device_identity.dart';
import 'package:core_logic/repositories/device_config_repository.dart';
import 'package:core_logic/services/device_registry_service.dart';
import 'package:test/test.dart';

void main() {
  late Directory tempDir;
  late DeviceConfigRepository repository;
  late MockKeyrxBridge bridge;
  late DeviceRegistryService service;

  setUp(() {
    tempDir = Directory.systemTemp.createTempSync('device_registry_service_test');
    repository = DeviceConfigRepository(configDir: tempDir.path);
    bridge = MockKeyrxBridge();
    service = DeviceRegistryServiceImpl(bridge: bridge, repository: repository);
  });

  tearDown(() async {
    await service.dispose();
    if (tempDir.existsSync()) {
      tempDir.deleteSync(recursive: true);
    }
  });

  group('DeviceRegistryService', () {
    test('refresh() merges FFI and repository devices', () async {
      // 1. Add a virtual device to repository
      const virtualIdentity = DeviceIdentity(
        vendorId: 0x9999,
        productId: 0x9999,
        serialNumber: 'VIRTUAL-001',
        userLabel: 'My Virtual KB',
      );
      await service.addVirtualDevice(virtualIdentity);

      // 2. Refresh
      final devices = await service.refresh();

      // Should have 2 mock devices from bridge + 1 virtual
      expect(devices, hasLength(3));
      
      final virtual = devices.firstWhere((d) => d.identity.serialNumber == 'VIRTUAL-001');
      expect(virtual.identity.userLabel, 'My Virtual KB');
      expect(virtual.connectedAt, isEmpty); // Virtual is not connected
    });

    test('setUserLabel persists to repository and FFI', () async {
      final initialDevices = await service.getDevices();
      print('Initial devices: ${initialDevices.map((d) => d.identity.toKey()).toList()}');
      final target = initialDevices.first;
      final key = target.identity.toKey();

      final result = await service.setUserLabel(key, 'New Label');
      if (result.hasError) print('Error: ${result.errorMessage}');
      expect(result.isSuccess, isTrue, reason: result.errorMessage);

      // Verify repository has it
      final config = await repository.load();
      final saved = config.devices.firstWhere((d) => d.toKey() == key);
      expect(saved.userLabel, 'New Label');

      // Verify merged list has it
      final refreshed = await service.getDevices();
      final updated = refreshed.firstWhere((d) => d.identity.toKey() == key);
      expect(updated.identity.userLabel, 'New Label');
    });

    test('toggleDevice, toggleRemap, assignProfile call bridge', () async {
      final initialDevices = await service.getDevices();
      final target = initialDevices.first;
      final key = target.identity.toKey();

      final r1 = await service.toggleDevice(key, true);
      expect(r1.isSuccess, isTrue, reason: r1.errorMessage);
      final r2 = await service.toggleRemap(key, true);
      expect(r2.isSuccess, isTrue, reason: r2.errorMessage);
      final r3 = await service.assignProfile(key, 'new-profile');
      expect(r3.isSuccess, isTrue, reason: r3.errorMessage);

      var refreshed = await service.getDevices();
      var updated = refreshed.firstWhere((d) => d.identity.toKey() == key);
      expect(updated.deviceEnabled, isTrue);
      expect(updated.remapEnabled, isTrue);
      expect(updated.profileId, 'new-profile');

      // Now disable device
      await service.toggleDevice(key, false);
      refreshed = await service.getDevices();
      updated = refreshed.firstWhere((d) => d.identity.toKey() == key);
      expect(updated.deviceEnabled, isFalse);
      expect(updated.remapEnabled, isFalse);
    });

    test('migrateVirtualDevices works', () async {
      const legacyDevices = [
        DeviceIdentity(vendorId: 0x1, productId: 0x1, serialNumber: 'LEGACY-1'),
      ];

      await service.migrateVirtualDevices(legacyDevices);

      final config = await repository.load();
      expect(config.devices.any((d) => d.serialNumber == 'LEGACY-1'), isTrue);
    });

    test('bridge events trigger refresh', () async {
      // Listen to stream
      final states = <List<Object>>[];
      service.devicesStream.listen((d) => states.add(d));

      // Simulate bridge event
      bridge.simulateEvent(EventType.deviceConnected, {});

      // Wait a bit for async refresh
      await Future<void>.delayed(const Duration(milliseconds: 50));

      expect(states, isNotEmpty);
    });
  });
}
