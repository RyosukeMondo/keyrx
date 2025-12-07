import '../ffi/bridge.dart';
import '../repositories/mapping_repository.dart';
import 'api_docs_service.dart';
import 'device_profile_service.dart';
import 'device_registry_service.dart';
import 'device_service.dart';
import 'engine_service.dart';
import 'engine_service_impl.dart';
import 'error_translator.dart';
import 'error_translator_impl.dart';
import 'hardware_service.dart';
import 'keymap_service.dart';
import 'layout_service.dart';
import 'profile_autosave_service.dart';
import 'profile_registry_service.dart';
import 'script_file_service.dart';
import 'storage_path_resolver.dart';
import 'test_service.dart';

/// Simple service registry to construct and pass dependencies without globals.
class ServiceRegistry {
  ServiceRegistry({
    required this.errorTranslator,
    required this.engineService,
    required this.mappingRepository,
    required this.deviceService,
    required this.deviceProfileService,
    required this.deviceRegistryService,
    required this.profileRegistryService,
    required this.scriptFileService,
    required this.testService,
    required this.bridge,
    required this.apiDocsService,
    required this.storagePathResolver,
    required this.profileAutosaveService,
    required this.layoutService,
    required this.hardwareService,
    required this.keymapService,
  });

  /// Build a registry with real implementations, allowing overrides for tests.
  factory ServiceRegistry.real({
    KeyrxBridge? bridge,
    ErrorTranslator? errorTranslator,
    MappingRepository? mappingRepository,
    DeviceService? deviceService,
    DeviceProfileService? deviceProfileService,
    DeviceRegistryService? deviceRegistryService,
    ProfileRegistryService? profileRegistryService,
    ScriptFileService? scriptFileService,
    TestService? testService,
    ApiDocsService? apiDocsService,
    StoragePathResolver? storagePathResolver,
    ProfileAutosaveService? profileAutosaveService,
    LayoutService? layoutService,
    HardwareService? hardwareService,
    KeymapService? keymapService,
  }) {
    final translator = errorTranslator ?? const ErrorTranslatorImpl();
    final effectiveBridge = bridge ?? KeyrxBridge.open();
    final engine = EngineServiceImpl(bridge: effectiveBridge);
    final device = deviceService ?? DeviceServiceImpl(bridge: effectiveBridge);
    final deviceProfile =
        deviceProfileService ??
        DeviceProfileServiceImpl(bridge: effectiveBridge);
    final deviceRegistry =
        deviceRegistryService ??
        DeviceRegistryServiceImpl(bridge: effectiveBridge);
    final profileRegistry =
        profileRegistryService ??
        ProfileRegistryServiceImpl(bridge: effectiveBridge);
    final scriptFile = scriptFileService ?? const ScriptFileService();
    final tests = testService ?? TestServiceImpl(bridge: effectiveBridge);
    final layouts = layoutService ?? LayoutService(bridge: effectiveBridge);
    final hardware =
        hardwareService ?? HardwareService(bridge: effectiveBridge);
    final keymaps = keymapService ?? KeymapService(bridge: effectiveBridge);

    final docs = apiDocsService ?? ApiDocsServiceImpl();
    final resolver = storagePathResolver ?? const StoragePathResolver();
    final autosave =
        profileAutosaveService ??
        ProfileAutosaveService(
          profileRegistryService: profileRegistry,
          storagePathResolver: resolver,
        );

    return ServiceRegistry(
      errorTranslator: translator,
      engineService: engine,
      mappingRepository: mappingRepository ?? MappingRepository(),
      deviceService: device,
      deviceProfileService: deviceProfile,
      deviceRegistryService: deviceRegistry,
      profileRegistryService: profileRegistry,
      scriptFileService: scriptFile,
      testService: tests,
      bridge: effectiveBridge,
      apiDocsService: docs,
      storagePathResolver: resolver,
      profileAutosaveService: autosave,
      layoutService: layouts,
      hardwareService: hardware,
      keymapService: keymaps,
    );
  }

  /// Create a registry with explicit (often mocked) implementations.
  factory ServiceRegistry.withOverrides({
    required ErrorTranslator errorTranslator,
    required EngineService engineService,
    required MappingRepository mappingRepository,
    required DeviceService deviceService,
    required DeviceProfileService deviceProfileService,
    required DeviceRegistryService deviceRegistryService,
    required ProfileRegistryService profileRegistryService,
    required ScriptFileService scriptFileService,
    required TestService testService,
    required KeyrxBridge bridge,
    required ApiDocsService apiDocsService,
    required StoragePathResolver storagePathResolver,
    ProfileAutosaveService? profileAutosaveService,
    required LayoutService layoutService,
    required HardwareService hardwareService,
    required KeymapService keymapService,
  }) {
    final autosave =
        profileAutosaveService ??
        ProfileAutosaveService(
          profileRegistryService: profileRegistryService,
          storagePathResolver: storagePathResolver,
        );

    return ServiceRegistry(
      errorTranslator: errorTranslator,
      engineService: engineService,
      mappingRepository: mappingRepository,
      deviceService: deviceService,
      deviceProfileService: deviceProfileService,
      deviceRegistryService: deviceRegistryService,
      profileRegistryService: profileRegistryService,
      scriptFileService: scriptFileService,
      testService: testService,
      bridge: bridge,
      apiDocsService: apiDocsService,
      storagePathResolver: storagePathResolver,
      profileAutosaveService: autosave,
      layoutService: layoutService,
      hardwareService: hardwareService,
      keymapService: keymapService,
    );
  }

  final ErrorTranslator errorTranslator;
  final EngineService engineService;
  final MappingRepository mappingRepository;
  final DeviceService deviceService;
  final DeviceProfileService deviceProfileService;
  final DeviceRegistryService deviceRegistryService;
  final ProfileRegistryService profileRegistryService;
  final ScriptFileService scriptFileService;
  final TestService testService;
  final KeyrxBridge bridge;
  final ApiDocsService apiDocsService;
  final StoragePathResolver storagePathResolver;
  final ProfileAutosaveService profileAutosaveService;
  final LayoutService layoutService;
  final HardwareService hardwareService;
  final KeymapService keymapService;

  /// Convenience for producing a registry with selective overrides.
  ServiceRegistry copyWith({
    ErrorTranslator? errorTranslator,
    EngineService? engineService,
    MappingRepository? mappingRepository,
    DeviceService? deviceService,
    DeviceProfileService? deviceProfileService,
    DeviceRegistryService? deviceRegistryService,
    ProfileRegistryService? profileRegistryService,
    ScriptFileService? scriptFileService,
    TestService? testService,
    KeyrxBridge? bridge,
    ApiDocsService? apiDocsService,
    StoragePathResolver? storagePathResolver,
    ProfileAutosaveService? profileAutosaveService,
    LayoutService? layoutService,
    HardwareService? hardwareService,
    KeymapService? keymapService,
  }) {
    return ServiceRegistry(
      errorTranslator: errorTranslator ?? this.errorTranslator,
      engineService: engineService ?? this.engineService,
      mappingRepository: mappingRepository ?? this.mappingRepository,
      deviceService: deviceService ?? this.deviceService,
      deviceProfileService: deviceProfileService ?? this.deviceProfileService,
      deviceRegistryService:
          deviceRegistryService ?? this.deviceRegistryService,
      profileRegistryService:
          profileRegistryService ?? this.profileRegistryService,
      scriptFileService: scriptFileService ?? this.scriptFileService,
      testService: testService ?? this.testService,
      bridge: bridge ?? this.bridge,
      apiDocsService: apiDocsService ?? this.apiDocsService,
      storagePathResolver: storagePathResolver ?? this.storagePathResolver,
      profileAutosaveService:
          profileAutosaveService ?? this.profileAutosaveService,
      layoutService: layoutService ?? this.layoutService,
      hardwareService: hardwareService ?? this.hardwareService,
      keymapService: keymapService ?? this.keymapService,
    );
  }

  /// Dispose any owned resources. Extend as more disposable services appear.
  Future<void> dispose() async {
    await engineService.dispose();
    await deviceService.dispose();
    await deviceProfileService.dispose();
    await deviceRegistryService.dispose();
    await profileRegistryService.dispose();
    await testService.dispose();
    await profileAutosaveService.dispose();
    await bridge.dispose();
  }
}
