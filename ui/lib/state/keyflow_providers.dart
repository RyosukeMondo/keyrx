/// Riverpod providers for KeyFlow state, graph data, and actions.
library;

import 'dart:async';

import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:riverpod_annotation/riverpod_annotation.dart';
import '../models/config_ids.dart';
import '../models/flow/flow_graph.dart';
import '../models/hardware_profile.dart';
import '../models/keymap.dart';
import '../models/runtime_config.dart';
import '../repositories/keyflow_repository.dart';
import '../services/auto_save_service.dart';
import '../services/config_result.dart';
import 'providers_setup.dart';
import 'save_state_provider.dart';

part 'keyflow_providers.g.dart';

@Riverpod(keepAlive: true)
KeyFlowRepository keyFlowRepository(KeyFlowRepositoryRef ref) {
  final bridge = ref.watch(bridgeProvider);
  final repository = KeyFlowRepository.forBridge(bridge);
  ref.onDispose(repository.dispose);
  return repository;
}

@Riverpod(keepAlive: true)
class RuntimeConfigNotifier extends _$RuntimeConfigNotifier {
  StreamSubscription<RuntimeConfig>? _subscription;

  @override
  Future<RuntimeConfig> build() async {
    final repository = ref.watch(keyFlowRepositoryProvider);

    _subscription = repository.watchRuntimeConfig().listen(
      (config) => state = AsyncValue.data(config),
      onError: (error, stackTrace) =>
          state = AsyncValue.error(error, stackTrace),
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
Future<RuntimeConfig> runtimeConfig(RuntimeConfigRef ref) {
  return ref.watch(runtimeConfigNotifierProvider.future);
}

@riverpod
Future<FlowGraph> flowGraph(FlowGraphRef ref) async {
  final runtime = await ref.watch(runtimeConfigProvider.future);
  return FlowGraph.fromRuntimeConfig(runtime);
}

class KeyFlowActions {
  KeyFlowActions(this._ref);

  final Ref _ref;

  /// Refresh runtime configuration and sync the provider state.
  Future<ConfigOperationResult<RuntimeConfig>> refreshRuntimeConfig() {
    return _repo.getRuntimeConfig(forceRefresh: true);
  }

  /// Add a slot with auto-save and optimistic updates.
  void addSlot(
    DeviceInstanceId device,
    ProfileSlot slot, {
    int? priorityOverride,
  }) {
    final operation = AddSlotOperation(
      device,
      slot,
      priorityOverride: priorityOverride,
    );
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
KeyFlowActions keyFlowActions(KeyFlowActionsRef ref) {
  return KeyFlowActions(ref);
}

/// Provider for available hardware profiles.
@riverpod
Future<List<HardwareProfile>> availableHardwareProfiles(
  AvailableHardwareProfilesRef ref,
) async {
  final registry = ref.watch(serviceRegistryProvider);
  final result = await registry.hardwareService.listProfiles();

  if (result.hasError) {
    throw StateError(result.errorMessage ?? 'Failed to load hardware profiles');
  }

  return result.data ?? [];
}

/// Provider for available keymaps.
@riverpod
Future<List<Keymap>> availableKeymaps(AvailableKeymapsRef ref) async {
  final registry = ref.watch(serviceRegistryProvider);
  final result = await registry.keymapService.listKeymaps();

  if (result.hasError) {
    throw StateError(result.errorMessage ?? 'Failed to load keymaps');
  }

  return result.data ?? [];
}
