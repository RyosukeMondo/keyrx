import 'dart:async';
import 'dart:convert';
import 'dart:typed_data';

import 'package:flutter_test/flutter_test.dart';
import 'package:keyrx_ui/ffi/bridge.dart';
import 'package:keyrx_ui/models/hardware_profile.dart';
import 'package:keyrx_ui/models/runtime_config.dart';
import 'package:keyrx_ui/repositories/keyflow_repository.dart';

class _FakeRuntimeApi implements RuntimeApi {
  _FakeRuntimeApi({required RuntimeConfig initialConfig, this.loadFailure})
      : _config = initialConfig;

  RuntimeConfig _config;

  @override
  Object? loadFailure;

  int getConfigCalls = 0;
  int addSlotCalls = 0;
  int removeSlotCalls = 0;
  int? lastPriority;

  RuntimeConfig? addSlotResponse;
  RuntimeConfig? removeSlotResponse;

  void setConfig(RuntimeConfig config) {
    _config = config;
  }

  @override
  String addSlot(String deviceJson, String slotJson, int priority) {
    addSlotCalls++;
    lastPriority = priority;
    return _wrap(addSlotResponse ?? _config);
  }

  @override
  String getConfig() {
    getConfigCalls++;
    return _wrap(_config);
  }

  @override
  String removeSlot(String deviceJson, String slotId) {
    removeSlotCalls++;
    return _wrap(removeSlotResponse ?? _config);
  }

  String _wrap(RuntimeConfig config) => 'ok:${jsonEncode(config.toJson())}';
}

class _EventRegistry {
  final Map<EventType, void Function(Uint8List)> handlers = {};

  void register(EventType type, void Function(Uint8List) handler) {
    handlers[type] = handler;
  }

  void unregister(EventType type) {
    handlers.remove(type);
  }
}

RuntimeConfig _configWithSlots(
  List<ProfileSlot> slots, {
  DeviceInstanceId? device,
}) {
  final dev = device ??
      const DeviceInstanceId(
        vendorId: 1,
        productId: 2,
        serial: 'abc',
      );
  return RuntimeConfig(devices: [DeviceSlots(device: dev, slots: slots)]);
}

ProfileSlot _slot({
  String id = 'slot-1',
  int priority = 0,
  bool active = false,
}) =>
    ProfileSlot(
      id: id,
      hardwareProfileId: 'hp',
      keymapId: 'km',
      priority: priority,
      active: active,
    );

void main() {
  late DateTime now;
  late DeviceInstanceId device;

  setUp(() {
    now = DateTime(2025, 1, 1, 12);
    device = const DeviceInstanceId(vendorId: 1, productId: 2, serial: 'abc');
  });

  group('KeyFlowRepository caching', () {
    test('returns cached config within TTL without new bridge call', () async {
      final api = _FakeRuntimeApi(initialConfig: _configWithSlots(const []));
      final repo = KeyFlowRepository(
        runtimeApi: api,
        cacheTtl: const Duration(minutes: 5),
        clock: () => now,
      );

      final first = await repo.getRuntimeConfig();
      expect(first.data, isNotNull);
      expect(api.getConfigCalls, 1);

      now = now.add(const Duration(minutes: 4));
      final second = await repo.getRuntimeConfig();

      expect(second.data, same(first.data));
      expect(api.getConfigCalls, 1, reason: 'No extra bridge call within TTL');
      await repo.dispose();
    });

    test('refreshes after TTL expiry or when forceRefresh is true', () async {
      final api = _FakeRuntimeApi(initialConfig: _configWithSlots(const []));
      final repo = KeyFlowRepository(
        runtimeApi: api,
        cacheTtl: const Duration(minutes: 5),
        clock: () => now,
      );

      await repo.getRuntimeConfig();
      expect(api.getConfigCalls, 1);

      now = now.add(const Duration(minutes: 6));
      final refreshedConfig =
          _configWithSlots([_slot(id: 'fresh', priority: 0)]);
      api.setConfig(refreshedConfig);

      final refreshed = await repo.getRuntimeConfig();
      expect(api.getConfigCalls, 2);
      expect(refreshed.data, refreshedConfig);

      final forcedConfig =
          _configWithSlots([_slot(id: 'forced', priority: 1)]);
      api.setConfig(forcedConfig);
      final forced =
          await repo.getRuntimeConfig(forceRefresh: true);
      expect(api.getConfigCalls, 3);
      expect(forced.data, forcedConfig);
      await repo.dispose();
    });
  });

  group('KeyFlowRepository mutations', () {
    test('auto-assigns next priority when adding a slot', () async {
      final baseConfig = _configWithSlots(
        [
          _slot(id: 'a', priority: 0),
          _slot(id: 'b', priority: 1),
        ],
        device: device,
      );
      final api = _FakeRuntimeApi(initialConfig: baseConfig);
      final repo = KeyFlowRepository(
        runtimeApi: api,
        clock: () => now,
      );
      await repo.getRuntimeConfig(); // seed cache

      final newSlot = _slot(id: 'c');
      final updatedConfig = _configWithSlots(
        [
          ...baseConfig.devices.first.slots,
          newSlot.copyWith(priority: 2),
        ],
        device: device,
      );
      api.addSlotResponse = updatedConfig;

      final result = await repo.addSlot(device, newSlot);

      expect(api.lastPriority, 2);
      expect(result.data, updatedConfig);
      await repo.dispose();
    });

    test('emits updates to watchers on mutation and bridge events', () async {
      final registry = _EventRegistry();
      final initialConfig = _configWithSlots([_slot(id: 'a')], device: device);
      final api = _FakeRuntimeApi(initialConfig: initialConfig);
      final repo = KeyFlowRepository(
        runtimeApi: api,
        clock: () => now,
        registerEventCallback: registry.register,
        unregisterEventCallback: registry.unregister,
      );

      final seen = <RuntimeConfig>[];
      final sub = repo.watchRuntimeConfig().listen(seen.add);

      await repo.getRuntimeConfig(); // emit cached value

      final mutated = _configWithSlots(
        [
          _slot(id: 'a'),
          _slot(id: 'b', priority: 1),
        ],
        device: device,
      );
      api.addSlotResponse = mutated;
      await repo.addSlot(device, _slot(id: 'b'));

      expect(seen.last, mutated);

      final eventConfig = _configWithSlots(
        [_slot(id: 'c', priority: 0)],
        device: device,
      );
      api.setConfig(eventConfig);
      // Trigger bridge event refresh
      registry.handlers[EventType.deviceUpdated]?.call(Uint8List.fromList([]));
      await pumpEventQueue();

      expect(seen.last, eventConfig);
      await sub.cancel();
      await repo.dispose();
    });

    test('propagates load failures as errors', () async {
      final api = _FakeRuntimeApi(
        initialConfig: _configWithSlots(const []),
        loadFailure: 'missing library',
      );
      final repo = KeyFlowRepository(runtimeApi: api);

      final result = await repo.getRuntimeConfig(forceRefresh: true);

      expect(result.hasError, isTrue);
      expect(result.errorMessage, contains('missing library'));
      await repo.dispose();
    });
  });
}
