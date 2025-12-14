/// Providers for dependency injection.
///
/// Central place for configuring all Provider-based dependencies
/// used throughout the application. Riverpod now owns the lifecycle
/// of these objects; Provider is used to expose them to existing
/// widget code until migrations are complete.
library;

import 'package:provider/provider.dart';
import 'package:provider/single_child_widget.dart';

import '../services/api_docs_service.dart';
import '../services/facade/keyrx_facade.dart';
import '../services/service_registry.dart';
import 'app_state.dart';

/// Creates the list of providers for the application using existing
/// `provider` consumers while allowing Riverpod to manage the
/// underlying instances.
List<SingleChildWidget> createProviders({
  required ServiceRegistry registry,
  required KeyrxFacade facade,
  required ApiDocsService apiDocsService,
  required AppState appState,
}) {
  return [
    // Core service registry - foundation for all services
    Provider<ServiceRegistry>.value(value: registry),

    // Unified facade - simplified API over services
    Provider<KeyrxFacade>.value(value: facade),

    // API documentation service
    Provider<ApiDocsService>.value(value: apiDocsService),

    // Application state - ChangeNotifier for reactive UI updates
    ChangeNotifierProvider<AppState>.value(value: appState),
  ];
}
