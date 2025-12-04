/// Providers for dependency injection.
///
/// Central place for configuring all Provider-based dependencies
/// used throughout the application.

import 'package:provider/provider.dart';
import 'package:provider/single_child_widget.dart';

import '../services/api_docs_service.dart';
import '../services/service_registry.dart';
import '../services/facade/keyrx_facade.dart';
import 'app_state.dart';

/// Creates the list of providers for the application.
///
/// This is the single source of truth for all DI configuration.
/// Providers are configured with proper disposal to prevent leaks.
///
/// Usage in main():
/// ```dart
/// runApp(
///   MultiProvider(
///     providers: createProviders(),
///     child: const KeyrxApp(),
///   ),
/// );
/// ```
List<SingleChildWidget> createProviders() {
  return [
    // Core service registry - foundation for all services
    Provider<ServiceRegistry>(
      create: (_) => ServiceRegistry.real(),
      dispose: (_, registry) => registry.dispose(),
    ),

    // Unified facade - simplified API over services
    Provider<KeyrxFacade>(
      create: (context) {
        final registry = context.read<ServiceRegistry>();
        return KeyrxFacade.real(registry);
      },
      dispose: (_, facade) => facade.dispose(),
    ),

    // API documentation service - ChangeNotifier for reactive updates
    ChangeNotifierProvider<ApiDocsService>(
      create: (context) {
        final registry = context.read<ServiceRegistry>();
        return registry.apiDocsService;
      },
    ),

    // Application state - ChangeNotifier for reactive UI updates
    ChangeNotifierProvider<AppState>(
      create: (context) {
        final registry = context.read<ServiceRegistry>();
        return AppState(
          engineService: registry.engineService,
          errorTranslator: registry.errorTranslator,
        );
      },
    ),
  ];
}
