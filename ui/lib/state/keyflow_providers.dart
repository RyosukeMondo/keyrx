/// Riverpod providers for KeyFlow state, graph data, and actions.
library;

import 'dart:async';
import 'dart:convert';

import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:riverpod_annotation/riverpod_annotation.dart';
import 'package:shared_preferences/shared_preferences.dart';

import '../config/storage_keys.dart';
import '../models/config_ids.dart';
import '../models/flow/flow_graph.dart';
import '../models/hardware_profile.dart';
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

@Riverpod(keepAlive: true)
class NodePositions extends _$NodePositions {
  @override
  Future<Map<String, FlowPosition>> build() async {
    final prefs = await SharedPreferences.getInstance();
    final raw = prefs.getString(StorageKeys.keyFlowNodePositions);
    if (raw == null || raw.isEmpty) {
      return {};
    }

    try {
      final decoded = jsonDecode(raw) as Map<String, dynamic>;
      return decoded.map(
        (key, value) => MapEntry(
          key,
          FlowPosition.fromJson(Map<String, dynamic>.from(value as Map)),
        ),
      );
    } catch (_) {
      await prefs.remove(StorageKeys.keyFlowNodePositions);
      return {};
    }
  }

  /// Persist a single node position to storage.
  Future<void> setPosition(String nodeId, FlowPosition position) async {
    final updated = Map<String, FlowPosition>.from(state.value ?? {})
      ..[nodeId] = position;
    state = AsyncValue.data(updated);
    await _persist(updated);
  }

  /// Persist multiple node positions to storage.
  Future<void> setPositions(Map<String, FlowPosition> positions) async {
    final updated = Map<String, FlowPosition>.from(state.value ?? {})
      ..addAll(positions);
    state = AsyncValue.data(updated);
    await _persist(updated);
  }

  /// Remove a stored node position.
  Future<void> removePosition(String nodeId) async {
    final updated = Map<String, FlowPosition>.from(state.value ?? {})
      ..remove(nodeId);
    state = AsyncValue.data(updated);
    await _persist(updated);
  }

  /// Clear all persisted node positions.
  Future<void> clear() async {
    state = const AsyncValue.data({});
    final prefs = await SharedPreferences.getInstance();
    await prefs.remove(StorageKeys.keyFlowNodePositions);
  }

  Future<void> _persist(Map<String, FlowPosition> positions) async {
    final prefs = await SharedPreferences.getInstance();
    final serialized = jsonEncode(
      positions.map((key, value) => MapEntry(key, value.toJson())),
    );
    await prefs.setString(StorageKeys.keyFlowNodePositions, serialized);
  }
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
    final notifier = _ref.read(runtimeConfigNotifierProvider.notifier);
    if (result.data != null) {
      notifier.state = AsyncValue.data(result.data!);
    } else if (result.hasError) {
      notifier.state = AsyncValue.error(
        StateError(result.errorMessage ?? 'Runtime config unavailable'),
        StackTrace.current,
      );
    }
  }
}

@riverpod
KeyFlowActions keyFlowActions(KeyFlowActionsRef ref) {
  return KeyFlowActions(ref);
}
