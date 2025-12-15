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
import '../services/config_result.dart';
import 'providers_setup.dart';

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
    return _run(() => _repo.getRuntimeConfig(forceRefresh: true));
  }

  /// Add a slot and update runtime state.
  Future<ConfigOperationResult<RuntimeConfig>> addSlot(
    DeviceInstanceId device,
    ProfileSlot slot, {
    int? priorityOverride,
  }) {
    return _run(
      () => _repo.addSlot(device, slot, priorityOverride: priorityOverride),
    );
  }

  /// Update an existing slot and update runtime state.
  Future<ConfigOperationResult<RuntimeConfig>> updateSlot(
    DeviceInstanceId device,
    ProfileSlot slot,
  ) {
    return _run(() => _repo.updateSlot(device, slot));
  }

  /// Remove a slot and update runtime state.
  Future<ConfigOperationResult<RuntimeConfig>> removeSlot(
    DeviceInstanceId device,
    String slotId,
  ) {
    return _run(() => _repo.removeSlot(device, slotId));
  }

  KeyFlowRepository get _repo => _ref.read(keyFlowRepositoryProvider);

  Future<ConfigOperationResult<RuntimeConfig>> _run(
    Future<ConfigOperationResult<RuntimeConfig>> Function() op,
  ) async {
    final result = await op();
    _syncRuntimeState(result);
    return result;
  }

  void _syncRuntimeState(ConfigOperationResult<RuntimeConfig> result) {
    if (result.data != null || result.hasError) {
      final notifier = _ref.read(runtimeConfigNotifierProvider.notifier);
      notifier.refresh(force: true);
    }
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
