/// Tests for provider configuration.
library;

import 'package:flutter_test/flutter_test.dart';
import 'package:provider/provider.dart';

import 'package:keyrx_ui/services/service_registry.dart';
import 'package:keyrx_ui/services/facade/keyrx_facade.dart';
import 'package:keyrx_ui/state/app_state.dart';
import 'package:keyrx_ui/state/providers.dart';

void main() {
  group('createProviders', () {
    test('provides ServiceRegistry', () {
      final providers = createProviders();

      final serviceRegistryProvider =
          providers.firstWhere((p) => p is Provider<ServiceRegistry>)
              as Provider<ServiceRegistry>;

      expect(serviceRegistryProvider, isNotNull);
    });

    test('provides KeyrxFacade', () {
      final providers = createProviders();

      final facadeProvider =
          providers.firstWhere((p) => p is Provider<KeyrxFacade>)
              as Provider<KeyrxFacade>;

      expect(facadeProvider, isNotNull);
    });

    test('provides AppState', () {
      final providers = createProviders();

      final appStateProvider =
          providers.firstWhere((p) => p is ChangeNotifierProvider<AppState>)
              as ChangeNotifierProvider<AppState>;

      expect(appStateProvider, isNotNull);
    });

    test('providers are in correct dependency order', () {
      final providers = createProviders();

      // ServiceRegistry should come first
      expect(providers[0], isA<Provider<ServiceRegistry>>());

      // KeyrxFacade should come after ServiceRegistry (depends on it)
      expect(providers[1], isA<Provider<KeyrxFacade>>());

      // AppState should come after ServiceRegistry (depends on it)
      expect(providers[2], isA<ChangeNotifierProvider<AppState>>());
    });
  });
}
