/// Riverpod provider for auto-save state management.
library;

import 'dart:async';

import 'package:riverpod_annotation/riverpod_annotation.dart';

import '../services/auto_save_service.dart';
import 'keyflow_providers.dart';

part 'save_state_provider.g.dart';

/// Provider for the auto-save service instance.
@Riverpod(keepAlive: true)
AutoSaveService autoSaveService(AutoSaveServiceRef ref) {
  final repository = ref.watch(keyFlowRepositoryProvider);
  final service = AutoSaveService(repository: repository);
  ref.onDispose(service.dispose);
  return service;
}

/// Provider tracking the current save state.
@riverpod
class SaveStateNotifier extends _$SaveStateNotifier {
  StreamSubscription<SaveState>? _subscription;

  @override
  SaveState build() {
    final service = ref.watch(autoSaveServiceProvider);

    // Subscribe to state changes from the service
    _subscription = service.stateStream.listen((newState) => state = newState);

    ref.onDispose(() => _subscription?.cancel());

    return service.currentState;
  }

  /// Manually retry the last failed save operation.
  Future<void> retry() async {
    final service = ref.read(autoSaveServiceProvider);
    await service.retry();
  }
}
