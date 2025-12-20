import 'dart:async';

import 'package:core_logic/core_logic.dart';
import 'package:core_logic/ffi/mock_bridge.dart'
    if (dart.library.html) 'package:core_logic/ffi/mock_bridge_stub.dart';
import 'package:core_logic/repositories/mapping_repository.dart' as core;
import 'package:core_logic/services/benchmark_service.dart' as core_benchmark;
import 'package:core_logic/services/device_registry_service.dart' as core_impl;
import 'package:core_logic/services/device_registry_service_ffi.dart'
    as core_ffi;
import 'package:core_logic/services/device_registry_service_web.dart'
    as core_web;
import 'package:core_logic/services/hardware_service_web.dart';
import 'package:core_logic/services/keymap_service_web.dart';
import 'package:core_logic/services/layout_service_web.dart';
import 'package:core_logic/services/metrics_service.dart' as core_metrics;
import 'package:core_logic/services/runtime_service_web.dart';
import 'package:core_logic/services/script_file_service_web.dart';
import 'package:core_logic/web/api_client.dart';
import 'package:core_logic/web/websocket_client.dart';
import 'package:flutter/foundation.dart' show kIsWeb;

import '../repositories/mapping_repository_ui.dart';
import '../utils/performance_timeline.dart';
import 'api_docs_service.dart';
import 'benchmark_service.dart' as ui_benchmark;
import 'daemon_manager.dart';
import 'daemon_service.dart';
import 'device_profile_service.dart';
import 'device_registry_service.dart' as ui;
import 'device_service.dart';
import 'engine_service.dart';
import 'engine_service_impl.dart';
import 'hardware_service_ui.dart';
import 'layout_service_ui.dart';
import 'profile_autosave_service.dart';

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
    required this.metricsService,
    required this.benchmarkService,
    required this.daemonManager,
  });

  /// Create a registry with explicit (often mocked) implementations.
  factory ServiceRegistry.withOverrides({
    required ErrorTranslator errorTranslator,
    required EngineService engineService,
    required MappingRepositoryUI mappingRepository,
    required DeviceService deviceService,
    required DeviceProfileService deviceProfileService,
    required ui.UIDeviceRegistryAdapter deviceRegistryService,
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
    required core_metrics.MetricsService metricsService,
    required ui_benchmark.BenchmarkService benchmarkService,
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
      metricsService: metricsService,
      benchmarkService: benchmarkService,
      daemonManager: daemonManager,
    );
  }

  /// Build a registry with real implementations, allowing overrides for tests.
  factory ServiceRegistry.real({
    String? backendUrl,
    KeyrxBridge? bridge,
    ErrorTranslator? errorTranslator,
    MappingRepositoryUI? mappingRepository,
    DeviceService? deviceService,
    DeviceProfileService? deviceProfileService,
    ui.UIDeviceRegistryAdapter? deviceRegistryService,
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
    core_metrics.MetricsService? metricsService,
    ui_benchmark.BenchmarkService? benchmarkService,
    DaemonManager? daemonManager,
  }) {
    final bootstrapSpan = PerformanceTimelineTracer.start(
      'services.bootstrap',
      context: {'source': 'real', 'platform': kIsWeb ? 'web' : 'desktop'},
    );

    if (kIsWeb) {
      // Web platform: Use HTTP/WebSocket transport
      return ServiceRegistry._createWebRegistry(
        backendUrl:
            backendUrl ??
            const String.fromEnvironment(
              'BACKEND_URL',
              defaultValue: 'http://localhost:8080',
            ),
        errorTranslator: errorTranslator,
        mappingRepository: mappingRepository,
        apiDocsService: apiDocsService,
        layoutService: layoutService,
        hardwareService: hardwareService,
        keymapService: keymapService,
        metricsService: metricsService,
        benchmarkService: benchmarkService,
        bootstrapSpan: bootstrapSpan,
      );
    } else {
      // Desktop platform: Use FFI bridge
      return ServiceRegistry._createDesktopRegistry(
        bridge: bridge,
        errorTranslator: errorTranslator,
        mappingRepository: mappingRepository,
        deviceService: deviceService,
        deviceProfileService: deviceProfileService,
        deviceRegistryService: deviceRegistryService,
        profileRegistryService: profileRegistryService,
        scriptFileService: scriptFileService,
        testService: testService,
        apiDocsService: apiDocsService,
        storagePathResolver: storagePathResolver,
        profileAutosaveService: profileAutosaveService,
        layoutService: layoutService,
        hardwareService: hardwareService,
        keymapService: keymapService,
        runtimeService: runtimeService,
        metricsService: metricsService,
        benchmarkService: benchmarkService,
        daemonManager: daemonManager,
        bootstrapSpan: bootstrapSpan,
      );
    }
  }

  /// Create service registry for web platform.
  factory ServiceRegistry._createWebRegistry({
    required String backendUrl,
    required PerformanceSpan bootstrapSpan,
    ErrorTranslator? errorTranslator,
    MappingRepositoryUI? mappingRepository,
    ApiDocsService? apiDocsService,
    LayoutServiceUI? layoutService,
    HardwareServiceUI? hardwareService,
    KeymapService? keymapService,
    core_metrics.MetricsService? metricsService,
    ui_benchmark.BenchmarkService? benchmarkService,
  }) {
    final translator = errorTranslator ?? const ErrorTranslatorImpl();

    // Create web transport layer
    final wsUrl = backendUrl.replaceFirst('http', 'ws');
    final wsClient = WebSocketClient('$wsUrl/ws');
    final apiClient = WebApiClient(baseUrl: backendUrl, wsClient: wsClient);

    // Create web-based services
    final coreLayout = LayoutServiceWeb(client: apiClient);
    final coreHardware = HardwareServiceWeb(client: apiClient);
    final coreKeymap = KeymapServiceWeb(client: apiClient);
    final coreDeviceRegistry = core_web.DeviceRegistryServiceWeb(
      client: apiClient,
    );
    final coreRuntime = RuntimeServiceWeb(client: apiClient);
    final coreMapping = core.MappingRepository();

    // Create UI wrappers
    final layouts = layoutService ?? LayoutServiceUI(coreLayout);
    final hardware = hardwareService ?? HardwareServiceUI(coreHardware);
    final mapping = mappingRepository ?? MappingRepositoryUI(coreMapping);
    final keymaps = keymapService ?? coreKeymap;
    final deviceRegistryUi = ui.UIDeviceRegistryAdapterImpl(
      coreService: coreDeviceRegistry,
    );

    final docs = apiDocsService ?? ApiDocsServiceImpl();

    // Stub implementations for services not available on web
    final stubBridge = _WebStubBridge();
    final stubEngine = _WebStubEngineService();
    final stubDevice = _WebStubDeviceService();
    final stubDeviceProfile = _WebStubDeviceProfileService();
    final stubProfileRegistry = _WebStubProfileRegistryService();
    final coreScriptFile = ScriptFileServiceWeb(client: apiClient);
    final stubTest = _WebStubTestService();
    final stubResolver = _WebStubStoragePathResolver();
    final stubAutosave = _WebStubProfileAutosaveService();
    final stubMetrics = metricsService ?? _WebStubMetricsService();
    final stubBenchmark = benchmarkService ?? _WebStubBenchmarkService();
    final stubDaemon = _WebStubDaemonManager();

    final registry = ServiceRegistry(
      errorTranslator: translator,
      engineService: stubEngine,
      mappingRepository: mapping,
      deviceService: stubDevice,
      deviceProfileService: stubDeviceProfile,
      deviceRegistryService: deviceRegistryUi,
      profileRegistryService: stubProfileRegistry,
      scriptFileService: coreScriptFile,
      testService: stubTest,
      bridge: stubBridge,
      apiDocsService: docs,
      storagePathResolver: stubResolver,
      profileAutosaveService: stubAutosave,
      layoutService: layouts,
      hardwareService: hardware,
      keymapService: keymaps,
      runtimeService: coreRuntime,
      metricsService: stubMetrics,
      benchmarkService: stubBenchmark,
      daemonManager: stubDaemon,
    );

    bootstrapSpan.finish(
      context: {'platform': 'web', 'backend_url': backendUrl},
    );

    return registry;
  }

  /// Create service registry for desktop platform.
  factory ServiceRegistry._createDesktopRegistry({
    required PerformanceSpan bootstrapSpan,
    KeyrxBridge? bridge,
    ErrorTranslator? errorTranslator,
    MappingRepositoryUI? mappingRepository,
    DeviceService? deviceService,
    DeviceProfileService? deviceProfileService,
    ui.UIDeviceRegistryAdapter? deviceRegistryService,
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
    core_metrics.MetricsService? metricsService,
    ui_benchmark.BenchmarkService? benchmarkService,
    DaemonManager? daemonManager,
  }) {
    final bridgeSpan = PerformanceTimelineTracer.start(
      'ffi.bridge.open',
      context: {'source': 'service_registry'},
    );

    // Create bridge if not provided, respecting MOCK_FFI environment variable
    final KeyrxBridge effectiveBridge;
    if (bridge != null) {
      effectiveBridge = bridge;
    } else {
      const useMockBridge = bool.fromEnvironment('MOCK_FFI');
      if (useMockBridge) {
        effectiveBridge = MockKeyrxBridge();
      } else {
        effectiveBridge = KeyrxBridge.open();
      }
    }

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
    final sharedDeviceRegistry = core_ffi.DeviceRegistryServiceFFI(
      bridge: effectiveBridge,
      repository: deviceConfigRepository,
    );

    // Migrate from legacy SharedPreferences if needed
    // Note: Fire-and-forget migration to avoid blocking UI startup.
    // Migration failures are handled gracefully (best-effort).
    unawaited(
      ui.UIDeviceRegistryAdapterImpl.migrateFromSharedPreferences(
        sharedDeviceRegistry,
      ),
    );

    final deviceRegistry =
        deviceRegistryService ??
        ui.UIDeviceRegistryAdapterImpl(coreService: sharedDeviceRegistry);

    final profileRegistry =
        profileRegistryService ??
        ProfileRegistryServiceFFI(bridge: effectiveBridge);
    final scriptFile = scriptFileService ?? const ScriptFileService();
    final tests = testService ?? TestServiceImpl(bridge: effectiveBridge);

    // Create core services
    final coreLayout = LayoutServiceFFI(bridge: effectiveBridge);
    final coreHardware = HardwareServiceFFI(bridge: effectiveBridge);
    final coreMapping = core.MappingRepository();

    // Create UI wrappers
    final layouts = layoutService ?? LayoutServiceUI(coreLayout);
    final hardware = hardwareService ?? HardwareServiceUI(coreHardware);
    final mapping = mappingRepository ?? MappingRepositoryUI(coreMapping);

    final keymaps = keymapService ?? KeymapServiceFFI(bridge: effectiveBridge);
    final runtime =
        runtimeService ?? RuntimeServiceImpl(bridge: effectiveBridge);
    final coreMetrics =
        metricsService ??
        core_metrics.MetricsServiceImpl(bridge: effectiveBridge);

    // Create core benchmark service and UI wrapper
    final coreBenchmark = core_benchmark.BenchmarkServiceImpl(
      bridge: effectiveBridge,
    );
    final benchmark =
        benchmarkService ??
        ui_benchmark.BenchmarkServiceImpl(coreService: coreBenchmark);

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
      mappingRepository: mapping,
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
      metricsService: coreMetrics,
      benchmarkService: benchmark,
      daemonManager: daemon,
    );

    bootstrapSpan.finish(
      context: {
        'platform': 'desktop',
        'bridge_initialized': initialized,
        'load_failure': effectiveBridge.loadFailure?.toString(),
      },
    );

    return registry;
  }

  final ErrorTranslator errorTranslator;
  final EngineService engineService;
  final MappingRepositoryUI mappingRepository;
  final DeviceService deviceService;
  final DeviceProfileService deviceProfileService;
  final ui.UIDeviceRegistryAdapter deviceRegistryService;
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
  final core_metrics.MetricsService metricsService;
  final ui_benchmark.BenchmarkService benchmarkService;
  final DaemonManager daemonManager;

  /// Convenience for producing a registry with selective overrides.
  ServiceRegistry copyWith({
    ErrorTranslator? errorTranslator,
    EngineService? engineService,
    MappingRepositoryUI? mappingRepository,
    DeviceService? deviceService,
    DeviceProfileService? deviceProfileService,
    ui.UIDeviceRegistryAdapter? deviceRegistryService,
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
    core_metrics.MetricsService? metricsService,
    ui_benchmark.BenchmarkService? benchmarkService,
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
    metricsService: metricsService ?? this.metricsService,
    benchmarkService: benchmarkService ?? this.benchmarkService,
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
    mappingRepository.dispose();
    await bridge.dispose();
  }
}

// Stub implementations for web platform (desktop-only services)

class _WebStubBridge implements KeyrxBridge {
  @override
  Null get bindings => null;

  @override
  String? get loadFailure => null;

  @override
  bool initialize() => true;

  @override
  Future<void> dispose() async {}

  @override
  dynamic noSuchMethod(Invocation invocation) =>
      throw UnsupportedError('KeyrxBridge not available on web platform');
}

class _WebStubEngineService implements EngineService {
  _WebStubEngineService() {
    // Create a broadcast stream that stays open but never emits
    _stateController = StreamController<EngineSnapshot>.broadcast();
  }

  late final StreamController<EngineSnapshot> _stateController;

  @override
  bool get isInitialized => false;

  @override
  String get version => 'web-stub-0.0.0';

  @override
  Future<bool> initialize() async => false;

  @override
  Future<bool> loadScript(String path) async => false;

  @override
  Future<ConsoleEvalResult> eval(String command) async =>
      const ConsoleEvalResult(
        success: false,
        output: 'Console not available on web platform',
      );

  @override
  Stream<EngineSnapshot> get stateStream => _stateController.stream;

  @override
  Future<KeyRegistryResult> fetchKeyRegistry() async =>
      KeyRegistryResult.fallback('Key registry not available on web');

  @override
  Future<void> start() async {}

  @override
  Future<void> stop() async {}

  @override
  Future<void> dispose() async {
    await _stateController.close();
  }
}

class _WebStubDeviceService implements DeviceService {
  @override
  Future<void> dispose() async {}

  @override
  dynamic noSuchMethod(Invocation invocation) =>
      throw UnsupportedError('DeviceService not available on web platform');
}

class _WebStubDeviceProfileService implements DeviceProfileService {
  @override
  Future<void> dispose() async {}

  @override
  dynamic noSuchMethod(Invocation invocation) => throw UnsupportedError(
    'DeviceProfileService not available on web platform',
  );
}

class _WebStubProfileRegistryService implements ProfileRegistryService {
  @override
  Future<void> dispose() async {}

  @override
  dynamic noSuchMethod(Invocation invocation) => throw UnsupportedError(
    'ProfileRegistryService not available on web platform',
  );
}

class _WebStubTestService implements TestService {
  @override
  Future<void> dispose() async {}

  @override
  dynamic noSuchMethod(Invocation invocation) =>
      throw UnsupportedError('TestService not available on web platform');
}

class _WebStubStoragePathResolver implements StoragePathResolver {
  @override
  // Return a sensible default path for web platform
  // The backend will handle the actual file system paths
  String resolveProfilesPath() => '.keyrx';

  @override
  // On web, directory creation is handled by backend
  // Just return the path that will be sent to the backend
  Future<String> ensureProfilesDirectory() async => resolveProfilesPath();
}

class _WebStubProfileAutosaveService implements ProfileAutosaveService {
  @override
  Future<void> dispose() async {}

  @override
  dynamic noSuchMethod(Invocation invocation) => throw UnsupportedError(
    'ProfileAutosaveService not available on web platform',
  );
}

class _WebStubMetricsService implements core_metrics.MetricsService {
  @override
  Future<core_metrics.MetricsSnapshot?> getSnapshot() async => null;

  @override
  Future<void> startUpdates() async {}

  @override
  Future<void> stopUpdates() async {}

  @override
  Future<void> setThresholds(core_metrics.MetricsThresholds thresholds) async {}

  @override
  Future<core_metrics.MetricsThresholds?> getThresholds() async => null;
}

class _WebStubBenchmarkService implements ui_benchmark.BenchmarkService {
  @override
  Future<ui_benchmark.BenchmarkServiceResult> runBenchmark(
    int iterations, {
    String? scriptPath,
  }) async =>
      ui_benchmark.BenchmarkServiceResult.error(
        'Benchmark not available on web platform',
      );

  @override
  Future<void> dispose() async {}
}

class _WebStubDaemonManager implements DaemonManager {
  @override
  dynamic noSuchMethod(Invocation invocation) =>
      throw UnsupportedError('DaemonManager not available on web platform');
}
