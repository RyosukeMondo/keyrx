import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:keyrx_ui/ffi/bridge.dart';
import 'package:keyrx_ui/models/action_binding.dart';
import 'package:keyrx_ui/models/device_identity.dart';
import 'package:keyrx_ui/models/device_state.dart';
import 'package:keyrx_ui/models/hardware_profile.dart';
import 'package:keyrx_ui/models/keymap.dart';
import 'package:keyrx_ui/models/runtime_config.dart';
import 'package:keyrx_ui/models/virtual_layout.dart';
import 'package:keyrx_ui/models/virtual_layout_type.dart';
import 'package:keyrx_ui/pages/keyflow_page.dart';
import 'package:keyrx_ui/repositories/mapping_repository.dart';
import 'package:keyrx_ui/services/api_docs_service.dart';
import 'package:keyrx_ui/services/config_result.dart';
import 'package:keyrx_ui/services/device_profile_service.dart';
import 'package:keyrx_ui/services/device_registry_service.dart';
import 'package:keyrx_ui/services/device_service.dart';
import 'package:keyrx_ui/services/engine_service.dart';
import 'package:keyrx_ui/services/error_translator.dart';
import 'package:keyrx_ui/services/hardware_service.dart';
import 'package:keyrx_ui/services/keymap_service.dart';
import 'package:keyrx_ui/services/layout_service.dart';
import 'package:keyrx_ui/services/profile_autosave_service.dart';
import 'package:keyrx_ui/services/profile_registry_service.dart';
import 'package:keyrx_ui/services/runtime_service.dart';
import 'package:keyrx_ui/services/script_file_service.dart';
import 'package:keyrx_ui/services/service_registry.dart';
import 'package:keyrx_ui/services/storage_path_resolver.dart';
import 'package:keyrx_ui/services/test_service.dart';
import 'package:provider/provider.dart';

import 'helpers/fake_services.dart';

class _StubDeviceRegistryService implements DeviceRegistryService {
  _StubDeviceRegistryService(this.devices);

  final StreamController<List<DeviceState>> _controller =
      StreamController<List<DeviceState>>.broadcast();
  List<DeviceState> devices;

  @override
  Future<List<DeviceState>> getDevices() async => devices;

  @override
  Future<List<DeviceState>> refresh() async {
    _controller.add(devices);
    return devices;
  }

  @override
  Stream<List<DeviceState>> get devicesStream => _controller.stream;

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
  Future<void> addVirtualDevice(DeviceIdentity identity) async {}

  @override
  Future<void> removeVirtualDevice(String key) async {}

  @override
  Future<void> dispose() async {
    await _controller.close();
  }
}

class _StubHardwareService with ChangeNotifier implements HardwareService {
  _StubHardwareService(this.profiles, {this.fail = false});

  final List<HardwareProfile> profiles;
  final bool fail;

  @override
  Future<ConfigOperationResult<List<HardwareProfile>>> listProfiles() async {
    if (fail) {
      return ConfigOperationResult.error('profiles failed');
    }
    return ConfigOperationResult.success(profiles);
  }

  @override
  Future<ConfigOperationResult<HardwareProfile>> saveProfile(
    HardwareProfile profile,
  ) async => ConfigOperationResult.success(profile);

  @override
  Future<ConfigOperationResult<void>> deleteProfile(String id) async =>
      ConfigOperationResult.success(null);
}

class _StubLayoutService with ChangeNotifier implements LayoutService {
  _StubLayoutService(this.layouts);

  final List<VirtualLayout> layouts;

  @override
  Future<ConfigOperationResult<void>> deleteLayout(String id) async =>
      ConfigOperationResult.success(null);

  @override
  Future<ConfigOperationResult<List<VirtualLayout>>> listLayouts() async =>
      ConfigOperationResult.success(layouts);

  @override
  Future<ConfigOperationResult<VirtualLayout>> saveLayout(
    VirtualLayout layout,
  ) async => ConfigOperationResult.success(layout);
}

class _StubKeymapService implements KeymapService {
  _StubKeymapService(this.keymaps);

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

class _StubRuntimeService implements RuntimeService {
  _StubRuntimeService(this.config);

  RuntimeConfig? config;

  @override
  RuntimeConfig? get lastConfig => config;

  @override
  Future<ConfigOperationResult<RuntimeConfig>> addSlot(
    DeviceInstanceId device,
    ProfileSlot slot,
  ) async => ConfigOperationResult.success(config ?? const RuntimeConfig());

  @override
  Future<ConfigOperationResult<RuntimeConfig>> getConfig() async =>
      ConfigOperationResult.success(config ?? const RuntimeConfig());

  @override
  Future<ConfigOperationResult<RuntimeConfig>> removeSlot(
    DeviceInstanceId device,
    String slotId,
  ) async => ConfigOperationResult.success(config ?? const RuntimeConfig());

  @override
  Future<ConfigOperationResult<RuntimeConfig>> reorderSlot(
    DeviceInstanceId device,
    String slotId,
    int priority,
  ) async => ConfigOperationResult.success(config ?? const RuntimeConfig());

  @override
  Future<ConfigOperationResult<RuntimeConfig>> setSlotActive(
    DeviceInstanceId device,
    String slotId,
    bool active,
  ) async => ConfigOperationResult.success(config ?? const RuntimeConfig());
}

class _FakeErrorTranslator implements ErrorTranslator {
  @override
  UserMessage translate(Object error) =>
      const UserMessage(title: 'error', body: 'fail');
}

Widget _buildSubject(ServiceRegistry registry) {
  return MultiProvider(
    providers: [Provider<ServiceRegistry>.value(value: registry)],
    child: const MaterialApp(home: KeyFlowPage()),
  );
}

void main() {
  const deviceIdentity = DeviceIdentity(
    vendorId: 1,
    productId: 2,
    serialNumber: 'abc',
  );
  const deviceState = DeviceState(
    identity: deviceIdentity,
    remapEnabled: true,
    profileId: 'profile-1',
    connectedAt: '2024-01-01T00:00:00Z',
    updatedAt: '2024-01-01T00:00:00Z',
  );
  final profile = HardwareProfile(
    id: 'profile-1',
    virtualLayoutId: 'layout-1',
    wiring: const {0: 'K1', 1: 'K2'},
  );
  const layout = VirtualLayout(
    id: 'layout-1',
    name: 'Test Layout',
    layoutType: VirtualLayoutType.matrix,
    keys: [
      VirtualKeyDef(id: 'K1', label: 'A'),
      VirtualKeyDef(id: 'K2', label: 'B'),
    ],
  );
  final keymap = Keymap(
    id: 'keymap-1',
    virtualLayoutId: 'layout-1',
    layers: const [
      KeymapLayer(
        name: 'Base',
        bindings: {'K1': ActionBinding.standardKey(value: 'A')},
      ),
    ],
  );
  final runtimeConfig = RuntimeConfig(
    devices: const [
      DeviceSlots(
        device: DeviceInstanceId(vendorId: 1, productId: 2, serial: 'abc'),
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

  ServiceRegistry _registryWith({
    List<DeviceState> devices = const [],
    bool failHardware = false,
    bool includeLayout = false,
    RuntimeConfig? runtime,
    KeyrxBridge? bridge,
  }) {
    final fakeBridge = bridge ?? FakeBridge();
    final deviceRegistry = _StubDeviceRegistryService(devices);
    final hardwareService = _StubHardwareService(
      [profile],
      fail: failHardware,
    );
    final layoutService = _StubLayoutService(
      includeLayout && devices.isNotEmpty ? [layout] : const [],
    );
    final keymapService = _StubKeymapService(
      includeLayout && devices.isNotEmpty ? [keymap] : const [],
    );
    final runtimeService = _StubRuntimeService(runtime ?? runtimeConfig);

    return ServiceRegistry.withOverrides(
      errorTranslator: _FakeErrorTranslator(),
      engineService: FakeEngineService(),
      mappingRepository: MappingRepository(),
      deviceService: FakeDeviceService(),
      deviceProfileService: FakeDeviceProfileService(),
      deviceRegistryService: deviceRegistry,
      profileRegistryService: FakeProfileRegistryService(),
      scriptFileService: FakeScriptFileService(),
      testService: FakeTestService(),
      bridge: fakeBridge,
      apiDocsService: FakeApiDocsService(),
      storagePathResolver: StoragePathResolver(bridge: fakeBridge),
      layoutService: layoutService,
      hardwareService: hardwareService,
      keymapService: keymapService,
      runtimeService: runtimeService,
      profileAutosaveService: ProfileAutosaveService(
        profileRegistryService: FakeProfileRegistryService(),
        storagePathResolver: StoragePathResolver(bridge: fakeBridge),
      ),
    );
  }

  testWidgets('renders panes and device data', (tester) async {
    final registry = _registryWith(
      devices: const [deviceState],
      includeLayout: false,
    );
    addTearDown(registry.dispose);

    await tester.pumpWidget(_buildSubject(registry));
    await tester.pump();
    await tester.pump(const Duration(milliseconds: 50));

    await tester.scrollUntilVisible(
      find.text('Details'),
      500,
      scrollable: find.byType(Scrollable).first,
    );
    expect(find.text('Devices'), findsOneWidget);
    expect(find.text(deviceIdentity.displayName), findsWidgets);
    expect(find.text('Flow graph'), findsOneWidget);
    expect(find.text('Details'), findsOneWidget);
    expect(find.textContaining('Runtime'), findsWidgets);
    expect(
      find.text('No layout found for the selected device.'),
      findsOneWidget,
    );
  });

  testWidgets('shows empty and stale runtime states', (tester) async {
    final registry = _registryWith(devices: const []);
    addTearDown(registry.dispose);

    await tester.pumpWidget(_buildSubject(registry));
    await tester.pump();
    await tester.pump(const Duration(milliseconds: 50));

    expect(
      find.text('No devices detected. Connect a device or load mock data.'),
      findsOneWidget,
    );
    expect(find.textContaining('Runtime stale'), findsOneWidget);
    expect(find.textContaining('Runtime stream idle'), findsOneWidget);
  });

  testWidgets('surfaces repository errors in banner', (tester) async {
    final registry = _registryWith(
      devices: const [deviceState],
      failHardware: true,
    );
    addTearDown(registry.dispose);

    await tester.pumpWidget(_buildSubject(registry));
    await tester.pump();
    await tester.pump(const Duration(milliseconds: 50));

    expect(find.text('Issues detected'), findsOneWidget);
    expect(find.textContaining('profiles failed'), findsOneWidget);
  });

  testWidgets('preserves device selection after refresh', (tester) async {
    final secondDevice = deviceState.copyWith(
      identity: const DeviceIdentity(
        vendorId: 3,
        productId: 4,
        serialNumber: 'def',
        userLabel: 'Beta Board',
      ),
    );
    final registry = _registryWith(devices: [deviceState, secondDevice]);
    addTearDown(registry.dispose);

    await tester.pumpWidget(_buildSubject(registry));
    await tester.pump();
    await tester.pump(const Duration(milliseconds: 50));

    await tester.tap(find.text('Beta Board'));
    await tester.pump();
    await tester.pump(const Duration(milliseconds: 50));
    final betaTile = tester.widget<ListTile>(find.widgetWithText(
      ListTile,
      'Beta Board',
    ));
    expect(betaTile.selected, isTrue);

    await tester.tap(find.byTooltip('Refresh devices'));
    await tester.pump(const Duration(milliseconds: 250));
    await tester.pump();
    await tester.pump(const Duration(milliseconds: 50));

    final refreshedTile = tester.widget<ListTile>(find.widgetWithText(
      ListTile,
      'Beta Board',
    ));
    expect(refreshedTile.selected, isTrue);
  });
}
