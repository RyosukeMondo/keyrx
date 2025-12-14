/// Riverpod providers for application-wide dependencies.
///
/// This file centralizes Riverpod configuration so the app can use
/// Riverpod for lifecycle management while legacy `provider` consumers
/// continue to work via the adapter in [createProviders].
library;

import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../ffi/bridge.dart';
import '../services/api_docs_service.dart';
import '../services/facade/facade_state.dart';
import '../services/facade/keyrx_facade.dart';
import '../services/service_registry.dart';
import 'app_state.dart';

final bridgeProvider = Provider<KeyrxBridge>((ref) {
  final bridge = KeyrxBridge.open();
  ref.onDispose(bridge.dispose);
  return bridge;
});

final serviceRegistryProvider = Provider<ServiceRegistry>((ref) {
  final bridge = ref.watch(bridgeProvider);
  final registry = ServiceRegistry.real(bridge: bridge);
  ref.onDispose(registry.dispose);
  return registry;
});

final keyrxFacadeProvider = Provider<KeyrxFacade>((ref) {
  final registry = ref.watch(serviceRegistryProvider);
  final facade = KeyrxFacade.real(registry);
  ref.onDispose(facade.dispose);
  return facade;
});

final apiDocsServiceProvider = Provider<ApiDocsService>((ref) {
  final registry = ref.watch(serviceRegistryProvider);
  return registry.apiDocsService;
});

final appStateProvider = ChangeNotifierProvider<AppState>((ref) {
  final registry = ref.watch(serviceRegistryProvider);
  final state = AppState(
    engineService: registry.engineService,
    errorTranslator: registry.errorTranslator,
  );
  ref.onDispose(state.dispose);
  return state;
});

/// Example KeyFlow provider to standardize Riverpod patterns.
///
/// This exposes the aggregated facade state so widgets can watch
/// KeyRx runtime status without wiring individual streams.
final keyFlowProvider = StreamProvider<FacadeState>((ref) {
  final facade = ref.watch(keyrxFacadeProvider);
  return facade.stateStream;
});
