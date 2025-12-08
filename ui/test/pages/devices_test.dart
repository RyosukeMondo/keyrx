import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:keyrx_ui/models/device_identity.dart';
import 'package:keyrx_ui/models/device_state.dart';
import 'package:keyrx_ui/models/hardware_profile.dart';
import 'package:keyrx_ui/models/keymap.dart';
import 'package:keyrx_ui/models/runtime_config.dart';
import 'package:keyrx_ui/pages/devices.dart';
import 'package:keyrx_ui/services/config_result.dart';
import 'package:keyrx_ui/services/device_registry_service.dart';
import 'package:keyrx_ui/services/hardware_service.dart';
import 'package:keyrx_ui/services/keymap_service.dart';
import 'package:keyrx_ui/services/runtime_service.dart';

class FakeRuntimeService implements RuntimeService {
  FakeRuntimeService(this.config);

  RuntimeConfig config;

  @override
  Future<ConfigOperationResult<RuntimeConfig>> addSlot(
    DeviceInstanceId device,
    ProfileSlot slot,
  ) async => ConfigOperationResult.success(config);

  @override
  Future<ConfigOperationResult<RuntimeConfig>> getConfig() async =>
      ConfigOperationResult.success(config);

  @override
  Future<ConfigOperationResult<RuntimeConfig>> removeSlot(
    DeviceInstanceId device,
    String slotId,
  ) async => ConfigOperationResult.success(config);

  @override
  Future<ConfigOperationResult<RuntimeConfig>> reorderSlot(
    DeviceInstanceId device,
    String slotId,
    int priority,
  ) async => ConfigOperationResult.success(config);

  @override
  Future<ConfigOperationResult<RuntimeConfig>> setSlotActive(
    DeviceInstanceId device,
    String slotId,
    bool active,
  ) async => ConfigOperationResult.success(config);
}

class FakeHardwareService implements HardwareService {
  FakeHardwareService(this.profiles);

  final List<HardwareProfile> profiles;

  @override
  Future<ConfigOperationResult<void>> deleteProfile(String id) async =>
      ConfigOperationResult.success(null);

  @override
  Future<ConfigOperationResult<List<HardwareProfile>>> listProfiles() async =>
      ConfigOperationResult.success(profiles);

  @override
  Future<ConfigOperationResult<HardwareProfile>> saveProfile(
    HardwareProfile profile,
  ) async => ConfigOperationResult.success(profile);
}

class FakeKeymapService implements KeymapService {
  FakeKeymapService(this.keymaps);

  final List<Keymap> keymaps;

  @override
  Future<ConfigOperationResult<void>> deleteKeymap(String id) async =>
      ConfigOperationResult.success(null);

  @override
  Future<ConfigOperationResult<List<Keymap>>> listKeymaps() async =>
      ConfigOperationResult.success(keymaps);

  @override
  Future<ConfigOperationResult<Keymap>> saveKeymap(Keymap keymap) async =>
      ConfigOperationResult.success(keymap);
}

class FakeDeviceRegistryService implements DeviceRegistryService {
  FakeDeviceRegistryService(this.devices);

  final List<DeviceState> devices;

  @override
  Future<List<DeviceState>> getDevices() async => devices;

  @override
  Future<DeviceRegistryOperationResult> assignProfile(
    String deviceKey,
    String profileId,
  ) async => DeviceRegistryOperationResult.success();

  @override
  Future<DeviceRegistryOperationResult> setUserLabel(
    String deviceKey,
    String? label,
  ) async => DeviceRegistryOperationResult.success();

  @override
  Future<DeviceRegistryOperationResult> toggleRemap(
    String deviceKey,
    bool enabled,
  ) async => DeviceRegistryOperationResult.success();

  @override
  Future<List<DeviceState>> refresh() async => devices;

  @override
  Stream<List<DeviceState>> get devicesStream => Stream.value(devices);

  @override
  Future<void> addVirtualDevice(DeviceIdentity identity) async {}

  @override
  Future<void> removeVirtualDevice(String key) async {}

  @override
  Future<void> dispose() async {}
}

void main() {
  group('DevicesPage', () {
    testWidgets('shows empty state when runtime has no devices', (
      tester,
    ) async {
      final runtimeService = FakeRuntimeService(const RuntimeConfig());
      final hardwareService = FakeHardwareService(const []);
      final keymapService = FakeKeymapService(const []);
      final deviceRegistry = FakeDeviceRegistryService(const []);

      await tester.pumpWidget(
        MaterialApp(
          home: DevicesPage(
            runtimeService: runtimeService,
            hardwareService: hardwareService,
            keymapService: keymapService,
            deviceRegistryService: deviceRegistry,
          ),
        ),
      );

      await tester.pumpAndSettle();

      expect(
        find.text('No connected devices with runtime slots'),
        findsOneWidget,
      );
      expect(find.byIcon(Icons.devices_other_outlined), findsOneWidget);
    });

    testWidgets('renders device slots from runtime config', (tester) async {
      final deviceId = DeviceInstanceId(
        vendorId: 0x1234,
        productId: 0x5678,
        serial: 'ABC',
      );
      final runtime = RuntimeConfig(
        devices: [
          DeviceSlots(
            device: deviceId,
            slots: const [
              ProfileSlot(
                id: 'slot-1',
                hardwareProfileId: 'hw-1',
                keymapId: 'km-1',
                active: true,
                priority: 2,
              ),
              ProfileSlot(
                id: 'slot-2',
                hardwareProfileId: 'hw-1',
                keymapId: 'km-1',
                active: false,
                priority: 1,
              ),
            ],
          ),
        ],
      );

      final runtimeService = FakeRuntimeService(runtime);
      final hardwareService = FakeHardwareService([
        HardwareProfile(
          id: 'hw-1',
          vendorId: 0x1234,
          productId: 0x5678,
          name: 'Test Wiring',
          virtualLayoutId: 'layout-1',
        ),
      ]);
      final keymapService = FakeKeymapService([
        const Keymap(
          id: 'km-1',
          name: 'Test Keymap',
          virtualLayoutId: 'layout-1',
          layers: [],
        ),
      ]);
      final deviceRegistry = FakeDeviceRegistryService([
        const DeviceState(
          identity: DeviceIdentity(
            vendorId: 0x1234,
            productId: 0x5678,
            serialNumber: 'ABC',
            userLabel: 'My Board',
          ),
          remapEnabled: true,
          profileId: 'ignored',
          connectedAt: '2024-01-01T00:00:00Z',
          updatedAt: '2024-01-01T00:00:00Z',
        ),
      ]);

      await tester.pumpWidget(
        MaterialApp(
          home: DevicesPage(
            runtimeService: runtimeService,
            hardwareService: hardwareService,
            keymapService: keymapService,
            deviceRegistryService: deviceRegistry,
          ),
        ),
      );

      await tester.pumpAndSettle();

      expect(find.text('My Board'), findsOneWidget);
      expect(find.text('Slot 1'), findsOneWidget);
      expect(find.text('Slot 2'), findsOneWidget);
      expect(find.text('Priority 2'), findsOneWidget);
      expect(find.text('Priority 1'), findsOneWidget);
      expect(find.text('Test Wiring'), findsWidgets);
      expect(find.text('Test Keymap'), findsWidgets);
    });
  });
}
