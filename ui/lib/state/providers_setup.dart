/// Riverpod providers for application-wide dependencies.
///
/// This file centralizes Riverpod configuration so the app can use
/// Riverpod for lifecycle management while legacy `provider` consumers
/// continue to work via the adapter in [createProviders].
library;

import 'package:flutter/foundation.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../services/api_docs_service.dart';
import '../services/facade/facade_state.dart';
import '../services/facade/keyrx_facade.dart';
import '../services/service_registry.dart';
import 'app_state.dart';

final serviceRegistryProvider = Provider<ServiceRegistry>((ref) {
  debugPrint('[DEBUG] serviceRegistryProvider: initializing');

  // ServiceRegistry.real() automatically detects web platform and creates appropriate services
  // It also handles bridge creation internally (including MOCK_FFI support)
  const backendUrl = kIsWeb ? 'http://192.168.11.24:18888' : null;
  debugPrint('[DEBUG] serviceRegistryProvider: backendUrl=$backendUrl');

  try {
    final registry = ServiceRegistry.real(
      backendUrl: backendUrl,
    );
    debugPrint('[DEBUG] serviceRegistryProvider: initialized successfully');
    ref.onDispose(registry.dispose);
    return registry;
  } on Exception catch (e, stackTrace) {
    debugPrint('[ERROR] serviceRegistryProvider: failed to initialize: $e');
    debugPrint('[ERROR] Stack trace: $stackTrace');
    rethrow;
  }
});

final keyrxFacadeProvider = Provider<KeyrxFacade>((ref) {
  debugPrint('[DEBUG] keyrxFacadeProvider: initializing');
  try {
    final registry = ref.watch(serviceRegistryProvider);
    debugPrint('[DEBUG] keyrxFacadeProvider: registry obtained');
    final facade = KeyrxFacade.real(registry);
    debugPrint('[DEBUG] keyrxFacadeProvider: initialized successfully');
    ref.onDispose(facade.dispose);
    return facade;
  } on Exception catch (e, stackTrace) {
    debugPrint('[ERROR] keyrxFacadeProvider: failed to initialize: $e');
    debugPrint('[ERROR] Stack trace: $stackTrace');
    rethrow;
  }
});

final apiDocsServiceProvider = Provider<ApiDocsService>((ref) {
  debugPrint('[DEBUG] apiDocsServiceProvider: initializing');
  final registry = ref.watch(serviceRegistryProvider);
  debugPrint('[DEBUG] apiDocsServiceProvider: initialized successfully');
  return registry.apiDocsService;
});

final appStateProvider = ChangeNotifierProvider<AppState>((ref) {
  debugPrint('[DEBUG] appStateProvider: initializing');
  try {
    final registry = ref.watch(serviceRegistryProvider);
    debugPrint('[DEBUG] appStateProvider: registry obtained');
    final state = AppState(
      engineService: registry.engineService,
      errorTranslator: registry.errorTranslator,
    );
    debugPrint('[DEBUG] appStateProvider: initialized successfully');
    ref.onDispose(state.dispose);
    return state;
  } on Exception catch (e, stackTrace) {
    debugPrint('[ERROR] appStateProvider: failed to initialize: $e');
    debugPrint('[ERROR] Stack trace: $stackTrace');
    rethrow;
  }
});

/// Example KeyFlow provider to standardize Riverpod patterns.
///
/// This exposes the aggregated facade state so widgets can watch
/// KeyRx runtime status without wiring individual streams.
final keyFlowProvider = StreamProvider<FacadeState>((ref) {
  final facade = ref.watch(keyrxFacadeProvider);
  return facade.stateStream;
});
