/// Fake service implementations for Flutter tests.
///
/// Provides centralized test doubles that implement service interfaces
/// without FFI dependencies.
library;

import 'dart:async';
import 'dart:typed_data';

import 'package:keyrx_ui/ffi/bindings.dart';
import 'package:keyrx_ui/ffi/bridge.dart';
import 'package:keyrx_ui/ffi/device_registry_ffi.dart';
import 'package:keyrx_ui/ffi/profile_registry_ffi.dart';
import 'package:keyrx_ui/ffi/bridge_config.dart';
import 'package:keyrx_ui/models/device_state.dart';
import 'package:keyrx_ui/models/device_identity.dart';
import 'package:keyrx_ui/models/layout_type.dart';
import 'package:keyrx_ui/models/profile.dart';

import 'package:keyrx_ui/services/api_docs_service.dart';
import 'package:keyrx_ui/services/device_profile_service.dart';
import 'package:keyrx_ui/services/device_registry_service.dart';
import 'package:keyrx_ui/services/device_service.dart';
import 'package:keyrx_ui/services/engine_service.dart';
import 'package:keyrx_ui/services/error_translator.dart';
import 'package:keyrx_ui/services/profile_registry_service.dart';
import 'package:keyrx_ui/services/script_file_service.dart';

import 'package:keyrx_ui/services/config_result.dart';
import 'package:keyrx_ui/services/runtime_service.dart';
import 'package:keyrx_ui/models/hardware_profile.dart';

import 'package:keyrx_ui/models/runtime_config.dart';
import 'package:keyrx_ui/services/test_service.dart';
import 'package:keyrx_ui/pages/migration_prompt_page.dart';

/// Fake DeviceService that returns empty lists and success results.
class FakeDeviceService implements DeviceService {
  List<KeyboardDevice> devices = const [];

  @override
  Future<List<KeyboardDevice>> listDevices() async => devices;

  @override
  Future<DeviceSelectionResult> selectDevice(String path) async =>
      DeviceSelectionResult.success();

  @override
  Future<bool> hasProfile(String deviceId) async => false;

  @override
  Future<List<KeyboardDevice>> refresh() async => devices;

  @override
  Future<void> dispose() async {}
}

/// Fake TestService that returns empty test results.
class FakeTestService implements TestService {
  @override
  Future<TestDiscoveryServiceResult> discoverTests(String scriptPath) async =>
      const TestDiscoveryServiceResult(tests: []);

  @override
  Future<TestRunServiceResult> runTests(
    String scriptPath, {
    String? filter,
  }) async => const TestRunServiceResult(
    total: 0,
    passed: 0,
    failed: 0,
    durationMs: 0,
    results: [],
  );

  @override
  Future<void> dispose() async {}
}

/// Fake ErrorTranslator that returns generic error messages.
class FakeErrorTranslator implements ErrorTranslator {
  @override
  UserMessage translate(Object error) =>
      const UserMessage(title: 'Error', body: 'An error occurred');
}

/// Controllable fake EngineService for tests that need to emit state.
class FakeEngineService implements EngineService {
  final StreamController<EngineSnapshot> _stateController =
      StreamController<EngineSnapshot>.broadcast();

  @override
  bool get isInitialized => true;

  @override
  String get version => 'test';

  @override
  Future<bool> initialize() async => true;

  @override
  Future<bool> loadScript(String path) async => true;

  @override
  Future<ConsoleEvalResult> eval(String command) async =>
      const ConsoleEvalResult(success: true, output: 'ok');

  @override
  Stream<EngineSnapshot> get stateStream => _stateController.stream;

  @override
  Future<KeyRegistryResult> fetchKeyRegistry() async =>
      const KeyRegistryResult(entries: []);

  /// Emit a state snapshot to listeners.
  void emit(EngineSnapshot snapshot) {
    _stateController.add(snapshot);
  }

  @override
  Future<void> dispose() async {
    await _stateController.close();
  }

  @override
  Future<void> start() async {
    // No-op for fake
  }

  @override
  Future<void> stop() async {
    // No-op for fake
  }
}

/// Fake KeyrxBridge that provides minimal stub implementations.
///
/// This fake bridge does not load any native library and returns
/// safe default values for all operations.
class FakeBridge implements KeyrxBridge {
  final StreamController<BridgeStateUpdate> _stateController =
      StreamController<BridgeStateUpdate>.broadcast();

  bool _initialized = true;
  final Map<EventType, void Function(Uint8List)> _eventCallbacks = {};

  @override
  KeyrxBindings? get bindings => null;

  @override
  bool startEngineLoop() => true;

  @override
  bool stopEngineLoop() => true;

  @override
  StreamController<BridgeStateUpdate>? get stateController => _stateController;

  @override
  bool get initialized => _initialized;

  @override
  set initialized(bool value) => _initialized = value;

  @override
  bool get isInitialized => _initialized;

  @override
  Object? get loadFailure => null;

  @override
  String get version => 'fake-1.0.0';

  @override
  bool initialize() => true;

  @override
  Stream<BridgeStateUpdate>? get stateStream => _stateController.stream;

  /// Emit a state snapshot to listeners.
  void emitState(BridgeStateUpdate state) {
    _stateController.add(state);
  }

  @override
  bool loadScript(String path) => true;

  @override
  Future<String?> eval(String command) async => 'ok:';

  @override
  KeyRegistryResult listKeys() => const KeyRegistryResult(entries: []);

  @override
  bool isBypassActive() => false;

  @override
  void setBypass(bool active) {}

  @override
  int selectDevice(String path) => 0;

  @override
  ScriptValidationResult checkScript(String path) =>
      const ScriptValidationResult(valid: true, errors: []);

  @override
  TestDiscoveryResult discoverTests(String path) =>
      const TestDiscoveryResult(tests: []);

  @override
  TestRunResult runTests(String path, {String? filter}) => const TestRunResult(
    total: 0,
    passed: 0,
    failed: 0,
    durationMs: 0,
    results: [],
  );

  @override
  SimulationResult simulate(
    List<KeyInput> keys, {
    String? scriptPath,
    bool comboMode = false,
  }) => const SimulationResult(mappings: [], activeLayers: [], pending: []);

  @override
  SessionListResult listSessions(String dirPath) =>
      const SessionListResult(sessions: []);

  @override
  SessionAnalysisResult analyzeSession(String path) =>
      SessionAnalysisResult.error('Not implemented in fake');

  @override
  ReplayResult replaySession(String path, {bool verify = false}) =>
      ReplayResult.error('Not implemented in fake');

  @override
  BenchmarkResult runBenchmark(int iterations, {String? scriptPath}) =>
      BenchmarkResult.error('Not implemented in fake');

  @override
  DoctorResult runDoctor() => DoctorResult.error('Not implemented in fake');

  @override
  void requestFullStateResubscribe() {}

  @override
  DeviceProfileResult getDeviceProfile(int vendorId, int productId) =>
      DeviceProfileResult.error('Not implemented in fake');

  @override
  bool hasDeviceProfile(int vendorId, int productId) => false;

  @override
  bool saveDeviceProfile(String profileJson) => false;

  @override
  DeviceRegistryResult<List<DeviceState>> listRegisteredDevices() =>
      DeviceRegistryResult.error('Not implemented in fake');

  @override
  DeviceRegistryResult<void> assignProfile(
    String deviceKey,
    String profileId,
  ) => DeviceRegistryResult.error('Not implemented in fake');

  @override
  DeviceRegistryResult<void> setRemapEnabled(String deviceKey, bool enabled) =>
      DeviceRegistryResult.error('Not implemented in fake');

  @override
  DeviceRegistryResult<void> setUserLabel(String deviceKey, String? label) =>
      DeviceRegistryResult.error('Not implemented in fake');

  @override
  ProfileRegistryResult<List<String>> listProfiles() =>
      ProfileRegistryResult.error('Not implemented in fake');

  @override
  ProfileRegistryResult<Profile> getProfile(String profileId) =>
      ProfileRegistryResult.error('Not implemented in fake');

  @override
  ProfileRegistryResult<void> saveProfile(Profile profile) =>
      ProfileRegistryResult.error('Not implemented in fake');

  @override
  ProfileRegistryResult<void> deleteProfile(String profileId) =>
      ProfileRegistryResult.error('Not implemented in fake');

  @override
  ProfileRegistryResult<List<Profile>> findCompatibleProfiles(
    LayoutType layoutType,
  ) => ProfileRegistryResult.error('Not implemented in fake');

  @override
  RecordingStartResult startRecording(String path) =>
      RecordingStartResult.error('Not implemented in fake');

  @override
  RecordingStopResult stopRecording() =>
      RecordingStopResult.error('Not implemented in fake');

  @override
  ValidationResult validateScript(
    String script, [
    ValidationOptions? options,
  ]) => const ValidationResult(isValid: true, errors: [], warnings: []);

  @override
  List<String> suggestKeys(String partial) => const [];

  @override
  List<String> allKeyNames() => const [];

  @override
  bool isEventCallbackRegistered(EventType eventType) =>
      _eventCallbacks.containsKey(eventType);

  @override
  bool registerEventCallback(
    EventType eventType,
    void Function(Uint8List jsonPayload) handler,
  ) {
    _eventCallbacks[eventType] = handler;
    return true;
  }

  @override
  bool unregisterEventCallback(EventType eventType) =>
      _eventCallbacks.remove(eventType) != null;

  @override
  Future<void> dispose() async {
    await _stateController.close();
  }

  @override
  void setConfigRoot(String path) {
    // No-op for fake
  }

  @override
  void shutdown() {
    // No-op for fake
  }

  @override
  Future<bool> checkMigrationNeeded(String oldProfilesDir) async => false;

  @override
  Future<MigrationReport> runMigration(
    String oldProfilesDir,
    String newProfilesDir, {
    bool createBackup = true,
  }) async => MigrationReport(
    totalCount: 0,
    migratedCount: 0,
    failedCount: 0,
    failures: [],
  );

  @override
  ConfigPathResult getConfigRoot() => ConfigPathResult.success('.');
}

/// Fake ScriptFileService that simulates file operations in memory.
class FakeScriptFileService implements ScriptFileService {
  final Map<String, String> _scripts = {};

  @override
  Future<ScriptFileResult> saveScript(String path, String content) async {
    _scripts[path] = content;
    return const ScriptFileResult(success: true);
  }

  @override
  Future<String?> loadScript(String path) async {
    return _scripts[path];
  }
}

/// Fake ApiDocsService with empty documentation.
class FakeApiDocsService extends ApiDocsService {
  @override
  bool get isLoaded => false;

  @override
  dynamic noSuchMethod(Invocation invocation) => super.noSuchMethod(invocation);
}

/// Fake DeviceProfileService that returns empty results.
class FakeDeviceProfileService implements DeviceProfileService {
  @override
  Future<void> dispose() async {}

  @override
  dynamic noSuchMethod(Invocation invocation) => super.noSuchMethod(invocation);
}

/// Fake DeviceRegistryService that returns controlled device lists.
class FakeDeviceRegistryService implements DeviceRegistryService {
  final _controller = StreamController<List<DeviceState>>.broadcast();
  List<DeviceState> _devices = [];

  void emit(List<DeviceState> devices) {
    _devices = devices;
    _controller.add(devices);
  }

  @override
  Future<List<DeviceState>> getDevices() async => _devices;

  @override
  Future<void> addVirtualDevice(DeviceIdentity identity) async {}

  @override
  Future<void> removeVirtualDevice(String key) async {}

  @override
  Future<DeviceRegistryOperationResult> toggleRemap(
    String deviceKey,
    bool enabled,
  ) async => DeviceRegistryOperationResult.success();

  @override
  Future<DeviceRegistryOperationResult> assignProfile(
    String deviceKey,
    String profileId,
  ) async => DeviceRegistryOperationResult.success();

  @override
  Future<DeviceRegistryOperationResult> setUserLabel(
    String deviceKey,
    String? label,
  ) async => DeviceRegistryOperationResult.success();

  @override
  Future<List<DeviceState>> refresh() async {
    _controller.add(_devices);
    return _devices;
  }

  @override
  Stream<List<DeviceState>> get devicesStream => _controller.stream;

  @override
  Future<void> dispose() async {
    await _controller.close();
  }
}

/// Fake ProfileRegistryService that returns empty profile lists.
class FakeProfileRegistryService implements ProfileRegistryService {
  @override
  Future<List<String>> listProfiles() async => const [];

  @override
  Future<Profile?> getProfile(String profileId) async => null;

  @override
  Future<ProfileRegistryOperationResult> saveProfile(Profile profile) async =>
      ProfileRegistryOperationResult.success();

  @override
  Future<ProfileRegistryOperationResult> deleteProfile(
    String profileId,
  ) async => ProfileRegistryOperationResult.success();

  @override
  Future<List<Profile>> findCompatibleProfiles(LayoutType layoutType) async =>
      const [];

  @override
  Future<List<String>> refresh() async => const [];

  @override
  Future<void> dispose() async {}
}

/// Fake RuntimeService that returns empty config.
class FakeRuntimeService implements RuntimeService {
  @override
  Future<ConfigOperationResult<RuntimeConfig>> getConfig() async =>
      ConfigOperationResult.success(RuntimeConfig(devices: []));

  @override
  Future<ConfigOperationResult<RuntimeConfig>> addSlot(
    DeviceInstanceId device,
    ProfileSlot slot,
  ) async => ConfigOperationResult.success(RuntimeConfig(devices: []));

  @override
  Future<ConfigOperationResult<RuntimeConfig>> removeSlot(
    DeviceInstanceId device,
    String slotId,
  ) async => ConfigOperationResult.success(RuntimeConfig(devices: []));

  @override
  Future<ConfigOperationResult<RuntimeConfig>> reorderSlot(
    DeviceInstanceId device,
    String slotId,
    int priority,
  ) async => ConfigOperationResult.success(RuntimeConfig(devices: []));

  @override
  Future<ConfigOperationResult<RuntimeConfig>> setSlotActive(
    DeviceInstanceId device,
    String slotId,
    bool active,
  ) async => ConfigOperationResult.success(RuntimeConfig(devices: []));

  @override
  dynamic noSuchMethod(Invocation invocation) => super.noSuchMethod(invocation);
}
