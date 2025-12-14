import 'dart:async';

import 'package:flutter_test/flutter_test.dart';
import 'package:keyrx_ui/ffi/bridge_engine.dart';
import 'package:keyrx_ui/models/device_identity.dart';
import 'package:keyrx_ui/models/device_state.dart';
import 'package:keyrx_ui/models/hardware_profile.dart';
import 'package:keyrx_ui/models/keymap.dart';
import 'package:keyrx_ui/models/runtime_config.dart';
import 'package:keyrx_ui/models/virtual_layout.dart';
import 'package:keyrx_ui/models/virtual_layout_type.dart';
import 'package:keyrx_ui/services/config_result.dart';
import 'package:keyrx_ui/services/device_registry_service.dart';
import 'package:keyrx_ui/services/facade/keyflow_repository.dart';
import 'package:keyrx_ui/services/hardware_service.dart';
import 'package:keyrx_ui/services/keymap_service.dart';
import 'package:keyrx_ui/services/layout_service.dart';
import 'package:keyrx_ui/services/runtime_service.dart';
import 'package:mocktail/mocktail.dart';

class _MockDeviceRegistryService extends Mock
    implements DeviceRegistryService {}

class _MockHardwareService extends Mock implements HardwareService {}

class _MockLayoutService extends Mock implements LayoutService {}

class _MockKeymapService extends Mock implements KeymapService {}

class _MockRuntimeService extends Mock implements RuntimeService {}

void main() {
  late _MockDeviceRegistryService deviceRegistry;
  late _MockHardwareService hardwareService;
  late _MockLayoutService layoutService;
  late _MockKeymapService keymapService;
  late _MockRuntimeService runtimeService;
  late StreamController<BridgeStateUpdate> runtimeController;
  late KeyFlowRepository repository;

  setUpAll(() {
    registerFallbackValue(
      const BridgeDeltaChange(BridgeDeltaChangeType.allKeysReleased),
    );
    registerFallbackValue(
      const BridgeStateDelta(fromVersion: 0, toVersion: 0, changes: []),
    );
    registerFallbackValue(
      const BridgeStateUpdate(
        delta: BridgeStateDelta(fromVersion: 0, toVersion: 0, changes: []),
      ),
    );
  });

  setUp(() {
    deviceRegistry = _MockDeviceRegistryService();
    hardwareService = _MockHardwareService();
    layoutService = _MockLayoutService();
    keymapService = _MockKeymapService();
    runtimeService = _MockRuntimeService();
    runtimeController = StreamController<BridgeStateUpdate>.broadcast();

    repository = KeyFlowRepository(
      deviceRegistryService: deviceRegistry,
      hardwareService: hardwareService,
      layoutService: layoutService,
      keymapService: keymapService,
      runtimeService: runtimeService,
      runtimeUpdates: runtimeController.stream,
    );
  });

  tearDown(() async {
    await repository.dispose();
    await runtimeController.close();
  });

  KeyFlowSnapshot _stubHappyPath({
    int duplicateWiring = 0,
    bool throwHardware = false,
  }) {
    final devices = [
      DeviceState(
        identity: const DeviceIdentity(
          vendorId: 1,
          productId: 2,
          serialNumber: 'abc',
        ),
        remapEnabled: true,
        profileId: 'profile-1',
        connectedAt: '2024-01-01T00:00:00Z',
        updatedAt: '2024-01-01T00:00:00Z',
      ),
    ];

    when(() => deviceRegistry.getDevices()).thenAnswer((_) async => devices);
    when(() => deviceRegistry.refresh()).thenAnswer((_) async => devices);

    final wiring = <int, String>{0: 'K1', if (duplicateWiring > 0) 1: 'K1'};
    final profiles = [
      HardwareProfile(
        id: 'profile-1',
        virtualLayoutId: 'layout-1',
        wiring: wiring,
      ),
    ];

    if (throwHardware) {
      when(() => hardwareService.listProfiles()).thenThrow(Exception('failed'));
    } else {
      when(
        () => hardwareService.listProfiles(),
      ).thenAnswer((_) async => ConfigOperationResult.success(profiles));
    }

    final layout = VirtualLayout(
      id: 'layout-1',
      name: 'Test',
      layoutType: VirtualLayoutType.matrix,
      keys: const [
        VirtualKeyDef(id: 'K1', label: 'A'),
        VirtualKeyDef(id: 'K2', label: 'B'),
        VirtualKeyDef(id: 'K3', label: 'C'),
      ],
    );

    when(
      () => layoutService.listLayouts(),
    ).thenAnswer((_) async => ConfigOperationResult.success([layout]));

    final keymap = Keymap(
      id: 'keymap-1',
      virtualLayoutId: 'layout-1',
      layers: const [],
    );

    when(
      () => keymapService.listKeymaps(),
    ).thenAnswer((_) async => ConfigOperationResult.success([keymap]));

    final runtimeConfig = RuntimeConfig(
      devices: [
        DeviceSlots(
          device: const DeviceInstanceId(
            vendorId: 1,
            productId: 2,
            serial: 'abc',
          ),
          slots: [
            ProfileSlot(
              id: 'slot-1',
              hardwareProfileId: 'profile-1',
              keymapId: 'keymap-1',
            ),
          ],
        ),
      ],
    );

    when(
      () => runtimeService.getConfig(),
    ).thenAnswer((_) async => ConfigOperationResult.success(runtimeConfig));

    return KeyFlowSnapshot(
      devices: [
        KeyFlowDevice(
          id: devices.first.identity.toKey(),
          displayName: devices.first.identity.displayName,
          remapEnabled: true,
          profileId: 'profile-1',
          layoutId: 'layout-1',
          wiring: wiring,
          gaps: duplicateWiring > 0 ? 2 : 3,
          conflicts: duplicateWiring > 0 ? duplicateWiring : 0,
          lastUpdated: devices.first.updatedAt,
        ),
      ],
      profiles: {for (final profile in profiles) profile.id: profile},
      layouts: {'layout-1': layout},
      keymaps: {'keymap-1': keymap},
      generatedAt: DateTime.now(),
      runtime: RuntimeOverlay.initial(),
      runtimeConfig: runtimeConfig,
    );
  }

  group('KeyFlowRepository', () {
    test('loadAll aggregates data and derives gaps/conflicts', () async {
      final expected = _stubHappyPath(duplicateWiring: 1);

      final snapshot = await repository.loadAll();

      expect(snapshot.devices, hasLength(1));
      expect(snapshot.devices.first.gaps, expected.devices.first.gaps);
      expect(snapshot.devices.first.conflicts, 1);
      expect(snapshot.profiles.length, expected.profiles.length);
      expect(snapshot.layouts.length, expected.layouts.length);
      expect(snapshot.keymaps.length, expected.keymaps.length);
      expect(snapshot.runtimeConfig?.devices.single.slots.single.id, 'slot-1');
      expect(snapshot.errors, isEmpty);
    });

    test(
      'refreshDevices falls back to cached list on refresh failure',
      () async {
        _stubHappyPath();
        final initial = await repository.loadAll();
        when(
          () => deviceRegistry.refresh(),
        ).thenThrow(Exception('refresh failed'));

        final snapshot = await repository.refreshDevices();

        expect(snapshot.devices, hasLength(1));
        expect(snapshot.devices.first.id, initial.devices.first.id);
        expect(snapshot.errors.map((e) => e.source), contains('devices'));
      },
    );

    test(
      'collects errors when a config source fails but still returns snapshot',
      () async {
        _stubHappyPath(throwHardware: true);

        final snapshot = await repository.loadAll();

        expect(snapshot.errors.map((e) => e.source), contains('hardware'));
        expect(snapshot.devices, isNotEmpty);
      },
    );

    test('runtimeStream emits overlays derived from bridge updates', () async {
      final update = BridgeStateUpdate(
        delta: const BridgeStateDelta(
          fromVersion: 0,
          toVersion: 1,
          changes: [
            BridgeDeltaChange(BridgeDeltaChangeType.layerActivated, payload: 1),
            BridgeDeltaChange(BridgeDeltaChangeType.keyPressed, payload: 'K1'),
            BridgeDeltaChange(BridgeDeltaChangeType.pendingAdded, payload: 42),
          ],
        ),
        fullSnapshot: null,
        event: 'delta',
        latencyUs: 123,
      );

      runtimeController.add(update);
      final overlay = await repository.runtimeStream().first;

      expect(overlay.activeLayers, contains(1));
      expect(overlay.heldKeys, contains('K1'));
      expect(overlay.pendingCount, 1);
      expect(overlay.lastEvent, 'delta');
      expect(overlay.latencyUs, 123);
      expect(overlay.version, 1);
    });
  });
}
