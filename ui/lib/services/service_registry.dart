import 'dart:async';

import 'package:core_logic/core_logic.dart';
import 'package:core_logic/ffi/bridge.dart'
    if (dart.library.html) 'package:core_logic/ffi/bridge_stub.dart';
import 'package:core_logic/models/daemon_protocol.dart';
import 'package:core_logic/repositories/mapping_repository.dart' as core;
import 'package:core_logic/services/device_registry_service.dart' as core_impl;
import 'package:core_logic/services/hardware_service_ffi.dart'
    if (dart.library.html) 'package:core_logic/services/hardware_service_ffi_stub.dart';
import 'package:core_logic/services/keymap_service_ffi.dart'
    if (dart.library.html) 'package:core_logic/services/keymap_service_ffi_stub.dart';
import 'package:core_logic/services/layout_service_ffi.dart'
    if (dart.library.html) 'package:core_logic/services/layout_service_ffi_stub.dart';
import 'package:core_logic/services/layout_service_web.dart';
import 'package:core_logic/services/hardware_service_web.dart';
import 'package:core_logic/services/keymap_service_web.dart';
import 'package:core_logic/services/device_registry_service_web.dart' as core_web;
import 'package:core_logic/services/runtime_service_web.dart';
import 'package:core_logic/services/script_file_service_web.dart';
import 'package:core_logic/web/api_client.dart';
import 'package:core_logic/web/websocket_client.dart';
import 'package:flutter/foundation.dart' show kIsWeb;

import '../repositories/mapping_repository_ui.dart';
import '../utils/performance_timeline.dart';
import 'api_docs_service.dart';
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
  /// Create a registry with explicit (often mocked) implementations.
  factory ServiceRegistry.withOverrides({
    required ErrorTranslator errorTranslator,
    required EngineService engineService,
    required MappingRepositoryUI mappingRepository,
    required DeviceService deviceService,
    required DeviceProfileService deviceProfileService,
    required ui.DeviceRegistryService deviceRegistryService,
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
    String? backendUrl,
    KeyrxBridge? bridge,
    ErrorTranslator? errorTranslator,
    MappingRepositoryUI? mappingRepository,
    DeviceService? deviceService,
    DeviceProfileService? deviceProfileService,
    ui.DeviceRegistryService? deviceRegistryService,
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
      context: {'source': 'real', 'platform': kIsWeb ? 'web' : 'desktop'},
    );

    if (kIsWeb) {
      // Web platform: Use HTTP/WebSocket transport
      return _createWebRegistry(
        backendUrl:
            backendUrl ??
            const String.fromEnvironment('BACKEND_URL', defaultValue: 'http://localhost:8080'),
        errorTranslator: errorTranslator,
        mappingRepository: mappingRepository,
        apiDocsService: apiDocsService,
        profileAutosaveService: profileAutosaveService,
        layoutService: layoutService,
        hardwareService: hardwareService,
        keymapService: keymapService,
        bootstrapSpan: bootstrapSpan,
      );
    } else {
      // Desktop platform: Use FFI bridge
      return _createDesktopRegistry(
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
        daemonManager: daemonManager,
        bootstrapSpan: bootstrapSpan,
      );
    }
  }

  /// Create service registry for web platform.
  static ServiceRegistry _createWebRegistry({
    required String backendUrl,
    required PerformanceSpan bootstrapSpan,
    ErrorTranslator? errorTranslator,
    MappingRepositoryUI? mappingRepository,
    ApiDocsService? apiDocsService,
    ProfileAutosaveService? profileAutosaveService,
    LayoutServiceUI? layoutService,
    HardwareServiceUI? hardwareService,
    KeymapService? keymapService,
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
    final coreDeviceRegistry = core_web.DeviceRegistryServiceWeb(client: apiClient);
    final coreRuntime = RuntimeServiceWeb(client: apiClient);
    final coreMapping = core.MappingRepository();

    // Create UI wrappers
    final layouts = layoutService ?? LayoutServiceUI(coreLayout);
    final hardware = hardwareService ?? HardwareServiceUI(coreHardware);
    final mapping = mappingRepository ?? MappingRepositoryUI(coreMapping);
    final keymaps = keymapService ?? coreKeymap;
    final deviceRegistryUi = ui.DeviceRegistryServiceImpl(coreService: coreDeviceRegistry);

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
      daemonManager: stubDaemon,
    );

    bootstrapSpan.finish(context: {'platform': 'web', 'backend_url': backendUrl});

    return registry;
  }

  /// Create service registry for desktop platform.
  static ServiceRegistry _createDesktopRegistry({
    required PerformanceSpan bootstrapSpan,
    KeyrxBridge? bridge,
    ErrorTranslator? errorTranslator,
    MappingRepositoryUI? mappingRepository,
    DeviceService? deviceService,
    DeviceProfileService? deviceProfileService,
    ui.DeviceRegistryService? deviceRegistryService,
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
    final bridgeSpan = PerformanceTimelineTracer.start(
      'ffi.bridge.open',
      context: {'source': 'service_registry'},
    );
    final effectiveBridge = bridge ?? KeyrxBridge.open();
    bridgeSpan.finish(context: {'load_failure': effectiveBridge.loadFailure?.toString()});

    final resolver = storagePathResolver ?? StoragePathResolver(bridge: effectiveBridge);
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
    final deviceProfile = deviceProfileService ?? DeviceProfileServiceImpl(bridge: effectiveBridge);

    // Shared DeviceRegistry from core_logic
    final deviceConfigRepository = DeviceConfigRepository();
    final sharedDeviceRegistry = core_impl.DeviceRegistryServiceImpl(
      bridge: effectiveBridge,
      repository: deviceConfigRepository,
    );

    // Migrate from legacy SharedPreferences if needed
    // Note: Fire-and-forget migration to avoid blocking UI startup.
    // Migration failures are handled gracefully (best-effort).
    unawaited(ui.DeviceRegistryServiceImpl.migrateFromSharedPreferences(sharedDeviceRegistry));

    final deviceRegistry =
        deviceRegistryService ?? ui.DeviceRegistryServiceImpl(coreService: sharedDeviceRegistry);

    final profileRegistry =
        profileRegistryService ?? ProfileRegistryServiceImpl(bridge: effectiveBridge);
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
    final runtime = runtimeService ?? RuntimeServiceImpl(bridge: effectiveBridge);
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
  final ui.DeviceRegistryService deviceRegistryService;
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
    MappingRepositoryUI? mappingRepository,
    DeviceService? deviceService,
    DeviceProfileService? deviceProfileService,
    ui.DeviceRegistryService? deviceRegistryService,
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
    profileRegistryService: profileRegistryService ?? this.profileRegistryService,
    scriptFileService: scriptFileService ?? this.scriptFileService,
    testService: testService ?? this.testService,
    bridge: bridge ?? this.bridge,
    apiDocsService: apiDocsService ?? this.apiDocsService,
    storagePathResolver: storagePathResolver ?? this.storagePathResolver,
    profileAutosaveService: profileAutosaveService ?? this.profileAutosaveService,
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
      const ConsoleEvalResult(success: false, output: 'Console not available on web platform');

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
  dynamic noSuchMethod(Invocation invocation) =>
      throw UnsupportedError('DeviceProfileService not available on web platform');
}

class _WebStubDeviceRegistryService implements ui.DeviceRegistryService {
  final _devicesController = StreamController<List<DeviceState>>.broadcast();

  @override
  Stream<List<DeviceState>> get devicesStream => _devicesController.stream;

  @override
  Future<List<DeviceState>> getDevices() async => [];

  @override
  Future<void> addVirtualDevice(DeviceIdentity identity) async {}

  @override
  Future<void> removeVirtualDevice(String key) async {}

  @override
  Future<ui.DeviceRegistryOperationResult> toggleDevice(
    String deviceKey, {
    required bool enabled,
  }) async => const ui.DeviceRegistryOperationResult.error(
    'Device management not available on web platform',
  );

  @override
  Future<ui.DeviceRegistryOperationResult> toggleRemap(
    String deviceKey, {
    required bool enabled,
  }) async =>
      const ui.DeviceRegistryOperationResult.error('Device remap not available on web platform');

  @override
  Future<ui.DeviceRegistryOperationResult> assignProfile(
    String deviceKey,
    String profileId,
  ) async => const ui.DeviceRegistryOperationResult.error(
    'Profile assignment not available on web platform',
  );

  @override
  Future<ui.DeviceRegistryOperationResult> setUserLabel(String deviceKey, String? label) async =>
      const ui.DeviceRegistryOperationResult.error('Label setting not available on web platform');

  @override
  Future<List<DeviceState>> refresh() async => [];

  @override
  Future<void> dispose() async {
    await _devicesController.close();
  }
}

class _WebStubProfileRegistryService implements ProfileRegistryService {
  @override
  Future<void> dispose() async {}

  @override
  dynamic noSuchMethod(Invocation invocation) =>
      throw UnsupportedError('ProfileRegistryService not available on web platform');
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
  String resolveProfilesPath() {
    // Return a sensible default path for web platform
    // The backend will handle the actual file system paths
    return '.keyrx';
  }

  @override
  Future<String> ensureProfilesDirectory() async {
    // On web, directory creation is handled by backend
    // Just return the path that will be sent to the backend
    return resolveProfilesPath();
  }
}

class _WebStubProfileAutosaveService implements ProfileAutosaveService {
  @override
  Future<void> dispose() async {}

  @override
  dynamic noSuchMethod(Invocation invocation) =>
      throw UnsupportedError('ProfileAutosaveService not available on web platform');
}

class _WebStubRuntimeService implements RuntimeService {
  @override
  dynamic noSuchMethod(Invocation invocation) =>
      throw UnsupportedError('RuntimeService not available on web platform');
}

class _WebStubDaemonService implements DaemonService {
  _WebStubDaemonService() {
    _eventController = StreamController<DaemonEvent>.broadcast();
    _notificationController = StreamController<JsonRpcNotification>.broadcast();
    _stateController = StreamController<DaemonConnectionState>.broadcast();
    _statusController = StreamController<EngineStatus>.broadcast();
  }

  late final StreamController<DaemonEvent> _eventController;
  late final StreamController<JsonRpcNotification> _notificationController;
  late final StreamController<DaemonConnectionState> _stateController;
  late final StreamController<EngineStatus> _statusController;

  @override
  Stream<DaemonEvent> get events => _eventController.stream;

  @override
  Stream<JsonRpcNotification> get notificationStream => _notificationController.stream;

  @override
  Stream<DaemonConnectionState> get connectionState => _stateController.stream;

  @override
  Stream<EngineStatus> get statusStream => _statusController.stream;

  @override
  EngineStatus get currentStatus => const EngineStatus(status: 'unavailable');

  @override
  DaemonConnectionState get state => DaemonConnectionState.disconnected;

  @override
  bool get isConnected => false;

  @override
  Future<void> connect() async {
    // No-op on web
  }

  @override
  Future<void> disconnect() async {
    // No-op on web
  }

  @override
  Future<void> subscribeEvents({
    List<String> eventTypes = const ['input', 'output', 'state_change'],
  }) async {
    // No-op on web - events come via WebSocket
  }

  @override
  Future<void> unsubscribeEvents() async {
    // No-op on web
  }

  @override
  Future<List<DaemonDeviceInfo>> startEngine({
    String? configPath,
    String? scriptPath,
    bool useAllDevices = true,
  }) async {
    // No-op on web - engine controlled via backend API
    return [];
  }

  @override
  Future<void> stopEngine() async {
    // No-op on web
  }

  @override
  Future<EngineStatus> getStatus() async {
    // Return unavailable status on web
    return const EngineStatus(status: 'unavailable');
  }

  @override
  Future<void> reloadConfig({String? configPath, String? scriptPath}) async {
    // No-op on web
  }

  @override
  Future<dynamic> sendRequest(String method, [Map<String, dynamic>? params]) async {
    // No-op on web - API calls go through WebApiClient
    return null;
  }

  @override
  Future<void> dispose() async {
    await _eventController.close();
    await _notificationController.close();
    await _stateController.close();
    await _statusController.close();
  }

  @override
  dynamic noSuchMethod(Invocation invocation) {
    // Gracefully handle any other methods
    return null;
  }
}

class _WebStubDaemonManager implements DaemonManager {
  @override
  dynamic noSuchMethod(Invocation invocation) =>
      throw UnsupportedError('DaemonManager not available on web platform');
}
