import 'dart:async';

import 'package:core_logic/ffi/bridge.dart';
import 'package:core_logic/repositories/device_config_repository.dart';
import 'package:core_logic/services/device_registry_service.dart' as core_impl;
import 'package:core_logic/services/hardware_service.dart' as core;
import 'package:core_logic/services/keymap_service.dart';
import 'package:core_logic/services/layout_service.dart' as core;
import 'package:core_logic/services/runtime_service.dart';
import '../repositories/mapping_repository.dart';
import '../utils/performance_timeline.dart';
import 'api_docs_service.dart';
import 'daemon_manager.dart';
import 'daemon_service.dart';
import 'device_profile_service.dart';
import 'device_registry_service.dart';
import 'device_service.dart';
import 'engine_service.dart';
import 'engine_service_impl.dart';
import 'error_translator.dart';
import 'error_translator_impl.dart';
import 'hardware_service_ui.dart';
import 'layout_service_ui.dart';
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
    required this.runtimeService,
    required this.daemonManager,
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
    LayoutServiceUI? layoutService,
    HardwareServiceUI? hardwareService,
    KeymapService? keymapService,
    RuntimeService? runtimeService,
    DaemonManager? daemonManager,
  }) {
    final bootstrapSpan = PerformanceTimelineTracer.start(
      'services.bootstrap',
      context: {'source': 'real'},
    );

    final bridgeSpan = PerformanceTimelineTracer.start(
      'ffi.bridge.open',
      context: {'source': 'service_registry'},
    );
    final effectiveBridge = bridge ?? KeyrxBridge.open();
    bridgeSpan.finish(
      context: {'load_failure': effectiveBridge.loadFailure?.toString()},
    );

    final resolver =
        storagePathResolver ?? StoragePathResolver(bridge: effectiveBridge);
    final translator = errorTranslator ?? const ErrorTranslatorImpl();

    // Initialize the bridge (and revolutionary runtime)
    final initSpan = PerformanceTimelineTracer.start(
      'ffi.bridge.initialize',
      context: {'has_bindings': effectiveBridge.bindings != null},
    );
    final initialized = effectiveBridge.initialize();
    initSpan.finish(context: {'initialized': initialized});

    final engine = EngineServiceImpl(bridge: effectiveBridge);
    final device = deviceService ?? DeviceServiceImpl(bridge: effectiveBridge);
    final deviceProfile =
        deviceProfileService ??
        DeviceProfileServiceImpl(bridge: effectiveBridge);

    // Shared DeviceRegistry from core_logic
    final deviceConfigRepository = DeviceConfigRepository();
    final sharedDeviceRegistry = core_impl.DeviceRegistryServiceImpl(
      bridge: effectiveBridge,
      repository: deviceConfigRepository,
    );

    // Migrate from legacy SharedPreferences if needed
    // Note: Fire-and-forget migration to avoid blocking UI startup.
    // Migration failures are handled gracefully (best-effort).
    unawaited(
      DeviceRegistryServiceImpl.migrateFromSharedPreferences(
        sharedDeviceRegistry,
      ),
    );

    final deviceRegistry =
        deviceRegistryService ??
        DeviceRegistryServiceImpl(coreService: sharedDeviceRegistry);

    final profileRegistry =
        profileRegistryService ??
        ProfileRegistryServiceImpl(bridge: effectiveBridge);
    final scriptFile = scriptFileService ?? const ScriptFileService();
    final tests = testService ?? TestServiceImpl(bridge: effectiveBridge);

    // Create core services
    final coreLayout = core.LayoutService(bridge: effectiveBridge);
    final coreHardware = core.HardwareService(bridge: effectiveBridge);

    // Create UI wrappers
    final layouts = layoutService ?? LayoutServiceUI(coreLayout);
    final hardware = hardwareService ?? HardwareServiceUI(coreHardware);

    final keymaps = keymapService ?? KeymapService(bridge: effectiveBridge);
    final runtime =
        runtimeService ?? RuntimeServiceImpl(bridge: effectiveBridge);
    final daemon = daemonManager ?? DaemonManager(DaemonService());

    final docs = apiDocsService ?? ApiDocsServiceImpl();
    final autosave =
        profileAutosaveService ??
        ProfileAutosaveService(
          profileRegistryService: profileRegistry,
          storagePathResolver: resolver,
        );

    final registry = ServiceRegistry(
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
      runtimeService: runtime,
      daemonManager: daemon,
    );

    bootstrapSpan.finish(
      context: {
        'bridge_initialized': initialized,
        'load_failure': effectiveBridge.loadFailure?.toString(),
      },
    );

    return registry;
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
    required LayoutServiceUI layoutService,
    required HardwareServiceUI hardwareService,
    required KeymapService keymapService,
    required RuntimeService runtimeService,
    required DaemonManager daemonManager,
    ProfileAutosaveService? profileAutosaveService,
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
      runtimeService: runtimeService,
      daemonManager: daemonManager,
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
  final LayoutServiceUI layoutService;
  final HardwareServiceUI hardwareService;
  final KeymapService keymapService;
  final RuntimeService runtimeService;
  final DaemonManager daemonManager;

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
    LayoutServiceUI? layoutService,
    HardwareServiceUI? hardwareService,
    KeymapService? keymapService,
    RuntimeService? runtimeService,
    DaemonManager? daemonManager,
  }) => ServiceRegistry(
    errorTranslator: errorTranslator ?? this.errorTranslator,
    engineService: engineService ?? this.engineService,
    mappingRepository: mappingRepository ?? this.mappingRepository,
    deviceService: deviceService ?? this.deviceService,
    deviceProfileService: deviceProfileService ?? this.deviceProfileService,
    deviceRegistryService: deviceRegistryService ?? this.deviceRegistryService,
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
    runtimeService: runtimeService ?? this.runtimeService,
    daemonManager: daemonManager ?? this.daemonManager,
  );

  /// Dispose any owned resources. Extend as more disposable services appear.
  Future<void> dispose() async {
    await engineService.dispose();
    await deviceService.dispose();
    await deviceProfileService.dispose();
    await deviceRegistryService.dispose();
    await profileRegistryService.dispose();
    await testService.dispose();
    await profileAutosaveService.dispose();
    layoutService.dispose();
    hardwareService.dispose();
    await bridge.dispose();
  }
}
