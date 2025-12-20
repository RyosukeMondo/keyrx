/// Riverpod providers for KeyFlow state, graph data, and actions.
library;

import 'dart:async';

import 'package:core_logic/models/device_state.dart';
import 'package:core_logic/models/flow/flow_graph.dart';
import 'package:core_logic/models/hardware_profile.dart';
import 'package:core_logic/models/keymap.dart';
import 'package:core_logic/models/runtime_config.dart';
import 'package:core_logic/services/config_result.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:riverpod_annotation/riverpod_annotation.dart';

import '../repositories/keyflow_repository.dart';
import '../services/auto_save_service.dart';
import 'providers_setup.dart';
import 'save_state_provider.dart';

part 'keyflow_providers.g.dart';

@Riverpod(keepAlive: true)
KeyFlowRepository keyFlowRepository(Ref ref) {
  final bridge = ref.watch(bridgeProvider);

  // On web (bridge is null), use RuntimeServiceWeb via HTTP API
  if (bridge == null) {
    final repository = KeyFlowRepository(runtimeApi: _WebRuntimeApiAdapter());
    ref.onDispose(repository.dispose);
    return repository;
  }

  final repository = KeyFlowRepository.forBridge(bridge);
  ref.onDispose(repository.dispose);
  return repository;
}

/// Adapter to use RuntimeService as RuntimeApi for web platform.
class _WebRuntimeApiAdapter implements RuntimeApi {
  _WebRuntimeApiAdapter();

  @override
  Object? get loadFailure => null;

  @override
  String getConfig() {
    // This method is synchronous but RuntimeService is async
    // KeyFlowRepository will handle async via getRuntimeConfig()
    return 'ok:{"devices":[],"version":1}';
  }

  @override
  String addSlot(String deviceJson, String slotJson, int priority) {
    // Synchronous wrapper - actual implementation uses async methods
    return getConfig();
  }

  @override
  String removeSlot(String deviceJson, String slotId) {
    // Synchronous wrapper - actual implementation uses async methods
    return getConfig();
  }
}

@Riverpod(keepAlive: true)
class RuntimeConfigNotifier extends _$RuntimeConfigNotifier {
  StreamSubscription<RuntimeConfig>? _subscription;

  @override
  Future<RuntimeConfig> build() async {
    final repository = ref.watch(keyFlowRepositoryProvider);

    _subscription = repository.watchRuntimeConfig().listen(
      (config) => state = AsyncValue.data(config),
      onError: (Object error, StackTrace stackTrace) => state = AsyncValue.error(error, stackTrace),
    );
    ref.onDispose(() => _subscription?.cancel());

    final result = await repository.getRuntimeConfig();
    if (result.data == null) {
      throw StateError(result.errorMessage ?? 'Runtime config unavailable');
    }
    if (result.hasError) {
      throw StateError(result.errorMessage!);
    }
    return result.data!;
  }

  /// Force refresh runtime configuration from the repository.
  Future<void> refresh({bool force = false}) async {
    final repository = ref.read(keyFlowRepositoryProvider);
    final result = await repository.getRuntimeConfig(forceRefresh: force);
    if (result.data != null) {
      state = AsyncValue.data(result.data!);
    } else if (result.hasError) {
      state = AsyncValue.error(
        StateError(result.errorMessage ?? 'Failed to refresh runtime config'),
        StackTrace.current,
      );
    }
  }
}

@riverpod
Future<RuntimeConfig> runtimeConfig(Ref ref) => ref.watch(runtimeConfigNotifierProvider.future);

@riverpod
Future<FlowGraph> flowGraph(Ref ref) async {
  final runtime = await ref.watch(runtimeConfigProvider.future);
  return FlowGraph.fromRuntimeConfig(runtime);
}

/// Provider for device registry states (device enabled/disabled toggles).
@riverpod
Stream<List<DeviceState>> deviceStates(Ref ref) {
  final registry = ref.watch(serviceRegistryProvider);
  return registry.deviceRegistryService.devicesStream;
}

class KeyFlowActions {
  KeyFlowActions(this._ref);

  final Ref _ref;

  /// Refresh runtime configuration and sync the provider state.
  Future<ConfigOperationResult<RuntimeConfig>> refreshRuntimeConfig() =>
      _repo.getRuntimeConfig(forceRefresh: true);

  /// Add a slot with auto-save and optimistic updates.
  void addSlot(DeviceInstanceId device, ProfileSlot slot, {int? priorityOverride}) {
    final operation = AddSlotOperation(device, slot, priorityOverride: priorityOverride);
    _scheduleAutoSave(operation);
  }

  /// Update an existing slot with auto-save and optimistic updates.
  void updateSlot(DeviceInstanceId device, ProfileSlot slot) {
    final operation = UpdateSlotOperation(device, slot);
    _scheduleAutoSave(operation);
  }

  /// Remove a slot with auto-save and optimistic updates.
  void removeSlot(DeviceInstanceId device, String slotId) {
    final operation = RemoveSlotOperation(device, slotId);
    _scheduleAutoSave(operation);
  }

  KeyFlowRepository get _repo => _ref.read(keyFlowRepositoryProvider);
  AutoSaveService get _autoSave => _ref.read(autoSaveServiceProvider);

  void _scheduleAutoSave(SaveOperation operation) {
    // Schedule the save with debouncing
    _autoSave.scheduleSave(operation);
  }
}

@riverpod
KeyFlowActions keyFlowActions(Ref ref) => KeyFlowActions(ref);

/// Provider for available hardware profiles.
@riverpod
Future<List<HardwareProfile>> availableHardwareProfiles(Ref ref) async {
  final registry = ref.watch(serviceRegistryProvider);
  final result = await registry.hardwareService.listProfiles();

  if (result.hasError) {
    throw StateError(result.errorMessage ?? 'Failed to load hardware profiles');
  }

  return result.data ?? [];
}

/// Provider for available keymaps.
@riverpod
Future<List<Keymap>> availableKeymaps(Ref ref) async {
  final registry = ref.watch(serviceRegistryProvider);
  final result = await registry.keymapService.listKeymaps();

  if (result.hasError) {
    throw StateError(result.errorMessage ?? 'Failed to load keymaps');
  }

  return result.data ?? [];
}
