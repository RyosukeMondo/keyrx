/// Tests for provider configuration.
library;

import 'package:core_logic/core_logic.dart' as core;
import 'package:flutter_test/flutter_test.dart';
import 'package:keyrx_ui/repositories/mapping_repository_ui.dart';
import 'package:keyrx_ui/services/api_docs_service.dart';
import 'package:keyrx_ui/services/facade/keyrx_facade.dart';
import 'package:keyrx_ui/services/service_registry.dart';
import 'package:keyrx_ui/state/app_state.dart';
import 'package:keyrx_ui/state/providers.dart';
import 'package:provider/provider.dart';

import '../helpers/fake_services.dart';

void main() {
  group('createProviders', () {
    test('creates real providers', () {
      final registry = ServiceRegistry(
        deviceService: FakeDeviceService(),
        deviceRegistryService: FakeDeviceRegistryService(),
        profileRegistryService: FakeProfileRegistryService(),
        scriptFileService: FakeScriptFileService(),
        runtimeService: FakeRuntimeService(),
        engineService: FakeEngineService(),
        testService: FakeTestService(),
        errorTranslator: FakeErrorTranslator(),
        mappingRepository: MappingRepositoryUI(core.MappingRepository()),
        bridge: FakeBridge(),
        apiDocsService: FakeApiDocsService(),
        storagePathResolver: FakeStoragePathResolver(),
        profileAutosaveService: FakeProfileAutosaveService(),
        layoutService: FakeLayoutService(),
        hardwareService: FakeHardwareService(),
        keymapService: FakeKeymapService(),
        deviceProfileService: FakeDeviceProfileService(),
        metricsService: FakeMetricsService(),
        benchmarkService: FakeBenchmarkService(),
        daemonManager: MockDaemonManager(),
      );
      final facade = KeyrxFacade.real(registry);
      final apiDocs = FakeApiDocsService();
      final appState = AppState(
        engineService: registry.engineService,
        errorTranslator: registry.errorTranslator,
      );

      final providers = createProviders(
        registry: registry,
        facade: facade,
        apiDocsService: apiDocs,
        appState: appState,
      );

      final serviceRegistryProvider =
          providers.firstWhere((p) => p is Provider<ServiceRegistry>)
              as Provider<ServiceRegistry>;

      expect(serviceRegistryProvider, isNotNull);
    });

    test('provides KeyrxFacade', () {
      final registry = ServiceRegistry(
        deviceService: FakeDeviceService(),
        deviceRegistryService: FakeDeviceRegistryService(),
        profileRegistryService: FakeProfileRegistryService(),
        scriptFileService: FakeScriptFileService(),
        runtimeService: FakeRuntimeService(),
        engineService: FakeEngineService(),
        testService: FakeTestService(),
        errorTranslator: FakeErrorTranslator(),
        mappingRepository: MappingRepositoryUI(core.MappingRepository()),
        bridge: FakeBridge(),
        apiDocsService: FakeApiDocsService(),
        storagePathResolver: FakeStoragePathResolver(),
        profileAutosaveService: FakeProfileAutosaveService(),
        layoutService: FakeLayoutService(),
        hardwareService: FakeHardwareService(),
        keymapService: FakeKeymapService(),
        deviceProfileService: FakeDeviceProfileService(),
        metricsService: FakeMetricsService(),
        benchmarkService: FakeBenchmarkService(),
        daemonManager: MockDaemonManager(),
      );
      final facade = KeyrxFacade.real(registry);
      final apiDocs = FakeApiDocsService();
      final appState = AppState(
        engineService: registry.engineService,
        errorTranslator: registry.errorTranslator,
      );

      final providers = createProviders(
        registry: registry,
        facade: facade,
        apiDocsService: apiDocs,
        appState: appState,
      );

      final facadeProvider =
          providers.firstWhere((p) => p is Provider<KeyrxFacade>)
              as Provider<KeyrxFacade>;

      expect(facadeProvider, isNotNull);
    });

    test('provides AppState', () {
      final registry = ServiceRegistry(
        deviceService: FakeDeviceService(),
        deviceRegistryService: FakeDeviceRegistryService(),
        profileRegistryService: FakeProfileRegistryService(),
        scriptFileService: FakeScriptFileService(),
        runtimeService: FakeRuntimeService(),
        engineService: FakeEngineService(),
        testService: FakeTestService(),
        errorTranslator: FakeErrorTranslator(),
        mappingRepository: MappingRepositoryUI(core.MappingRepository()),
        bridge: FakeBridge(),
        apiDocsService: FakeApiDocsService(),
        storagePathResolver: FakeStoragePathResolver(),
        profileAutosaveService: FakeProfileAutosaveService(),
        layoutService: FakeLayoutService(),
        hardwareService: FakeHardwareService(),
        keymapService: FakeKeymapService(),
        deviceProfileService: FakeDeviceProfileService(),
        metricsService: FakeMetricsService(),
        benchmarkService: FakeBenchmarkService(),
        daemonManager: MockDaemonManager(),
      );
      final facade = KeyrxFacade.real(registry);
      final apiDocs = FakeApiDocsService();
      final appState = AppState(
        engineService: registry.engineService,
        errorTranslator: registry.errorTranslator,
      );

      final providers = createProviders(
        registry: registry,
        facade: facade,
        apiDocsService: apiDocs,
        appState: appState,
      );

      final appStateProvider =
          providers.firstWhere((p) => p is ChangeNotifierProvider<AppState>)
              as ChangeNotifierProvider<AppState>;

      expect(appStateProvider, isNotNull);
    });

    test('providers are in correct dependency order', () {
      final registry = ServiceRegistry(
        deviceService: FakeDeviceService(),
        deviceRegistryService: FakeDeviceRegistryService(),
        profileRegistryService: FakeProfileRegistryService(),
        scriptFileService: FakeScriptFileService(),
        runtimeService: FakeRuntimeService(),
        engineService: FakeEngineService(),
        testService: FakeTestService(),
        errorTranslator: FakeErrorTranslator(),
        mappingRepository: MappingRepositoryUI(core.MappingRepository()),
        bridge: FakeBridge(),
        apiDocsService: FakeApiDocsService(),
        storagePathResolver: FakeStoragePathResolver(),
        profileAutosaveService: FakeProfileAutosaveService(),
        layoutService: FakeLayoutService(),
        hardwareService: FakeHardwareService(),
        keymapService: FakeKeymapService(),
        deviceProfileService: FakeDeviceProfileService(),
        metricsService: FakeMetricsService(),
        benchmarkService: FakeBenchmarkService(),
        daemonManager: MockDaemonManager(),
      );
      final facade = KeyrxFacade.real(registry);
      final apiDocs = FakeApiDocsService();
      final appState = AppState(
        engineService: registry.engineService,
        errorTranslator: registry.errorTranslator,
      );

      final providers = createProviders(
        registry: registry,
        facade: facade,
        apiDocsService: apiDocs,
        appState: appState,
      );

      // ServiceRegistry should come first
      expect(providers[0], isA<Provider<ServiceRegistry>>());

      // KeyrxFacade should come after ServiceRegistry (depends on it)
      expect(providers[1], isA<Provider<KeyrxFacade>>());

      // AppState should come after ServiceRegistry (depends on it)
      expect(providers[2], isA<Provider<ApiDocsService>>());
      expect(providers[3], isA<ChangeNotifierProvider<AppState>>());
    });
  });
}
