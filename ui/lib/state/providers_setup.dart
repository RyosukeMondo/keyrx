/// Riverpod providers for application-wide dependencies.
///
/// This file centralizes Riverpod configuration so the app can use
/// Riverpod for lifecycle management while legacy `provider` consumers
/// continue to work via the adapter in [createProviders].
library;

import 'package:core_logic/ffi/bridge.dart'
    if (dart.library.html) 'package:core_logic/ffi/bridge_stub.dart';
import 'package:core_logic/ffi/mock_bridge.dart'
    if (dart.library.html) 'package:core_logic/ffi/mock_bridge_stub.dart';
import 'package:flutter/foundation.dart' show kIsWeb;
import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../services/api_docs_service.dart';
import '../services/facade/facade_state.dart';
import '../services/facade/keyrx_facade.dart';
import '../services/service_registry.dart';
import 'app_state.dart';

final bridgeProvider = Provider<KeyrxBridge?>((ref) {
  print('[DEBUG] bridgeProvider: initializing (kIsWeb=$kIsWeb)');
  // On web, we don't use a bridge - ServiceRegistry.real() handles web automatically
  if (kIsWeb) {
    print('[DEBUG] bridgeProvider: returning null for web platform');
    return null;
  }

  const useMockBridge = bool.fromEnvironment('MOCK_FFI');

  final KeyrxBridge bridge;
  if (useMockBridge) {
    bridge = MockKeyrxBridge();
  } else {
    bridge = KeyrxBridge.open();
  }

  ref.onDispose(() async {
    if (bridge is MockKeyrxBridge) {
      await bridge.dispose();
    }
  });

  print('[DEBUG] bridgeProvider: initialized successfully');
  return bridge;
});

final serviceRegistryProvider = Provider<ServiceRegistry>((ref) {
  print('[DEBUG] serviceRegistryProvider: initializing');
  final bridge = ref.watch(bridgeProvider);
  print('[DEBUG] serviceRegistryProvider: bridge obtained (isNull=${bridge == null})');

  // ServiceRegistry.real() automatically detects web platform and creates appropriate services
  const backendUrl = kIsWeb ? 'http://192.168.11.24:18888' : null;
  print('[DEBUG] serviceRegistryProvider: backendUrl=$backendUrl');

  try {
    final registry = ServiceRegistry.real(bridge: bridge, backendUrl: backendUrl);
    print('[DEBUG] serviceRegistryProvider: initialized successfully');
    ref.onDispose(registry.dispose);
    return registry;
  } catch (e, stackTrace) {
    print('[ERROR] serviceRegistryProvider: failed to initialize: $e');
    print('[ERROR] Stack trace: $stackTrace');
    rethrow;
  }
});

final keyrxFacadeProvider = Provider<KeyrxFacade>((ref) {
  print('[DEBUG] keyrxFacadeProvider: initializing');
  try {
    final registry = ref.watch(serviceRegistryProvider);
    print('[DEBUG] keyrxFacadeProvider: registry obtained');
    final facade = KeyrxFacade.real(registry);
    print('[DEBUG] keyrxFacadeProvider: initialized successfully');
    ref.onDispose(facade.dispose);
    return facade;
  } catch (e, stackTrace) {
    print('[ERROR] keyrxFacadeProvider: failed to initialize: $e');
    print('[ERROR] Stack trace: $stackTrace');
    rethrow;
  }
});

final apiDocsServiceProvider = Provider<ApiDocsService>((ref) {
  print('[DEBUG] apiDocsServiceProvider: initializing');
  final registry = ref.watch(serviceRegistryProvider);
  print('[DEBUG] apiDocsServiceProvider: initialized successfully');
  return registry.apiDocsService;
});

final appStateProvider = ChangeNotifierProvider<AppState>((ref) {
  print('[DEBUG] appStateProvider: initializing');
  try {
    final registry = ref.watch(serviceRegistryProvider);
    print('[DEBUG] appStateProvider: registry obtained');
    final state = AppState(
      engineService: registry.engineService,
      errorTranslator: registry.errorTranslator,
    );
    print('[DEBUG] appStateProvider: initialized successfully');
    ref.onDispose(state.dispose);
    return state;
  } catch (e, stackTrace) {
    print('[ERROR] appStateProvider: failed to initialize: $e');
    print('[ERROR] Stack trace: $stackTrace');
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
