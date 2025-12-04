/// Fake service implementations for Flutter tests.
///
/// Provides centralized test doubles that implement service interfaces
/// without FFI dependencies.
library;

import 'dart:async';

import 'package:keyrx_ui/ffi/bindings.dart';
import 'package:keyrx_ui/ffi/bridge.dart';
import 'package:keyrx_ui/models/validation.dart';
import 'package:keyrx_ui/services/api_docs_service.dart';
import 'package:keyrx_ui/services/audio_service.dart';
import 'package:keyrx_ui/services/device_service.dart';
import 'package:keyrx_ui/services/engine_service.dart';
import 'package:keyrx_ui/services/error_translator.dart';
import 'package:keyrx_ui/services/permission_service.dart';
import 'package:keyrx_ui/services/script_file_service.dart';
import 'package:keyrx_ui/services/test_service.dart';

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
  }) async =>
      const TestRunServiceResult(
        total: 0,
        passed: 0,
        failed: 0,
        durationMs: 0,
        results: [],
      );

  @override
  Future<void> dispose() async {}
}

/// Fake AudioService that returns success for all operations.
class FakeAudioService implements AudioService {
  @override
  AudioState get state => AudioState.idle;

  @override
  Stream<ClassificationResult> get classificationStream => const Stream.empty();

  @override
  Future<AudioOperationResult> start({required int bpm}) async =>
      const AudioOperationResult(success: true);

  @override
  Future<AudioOperationResult> stop() async =>
      const AudioOperationResult(success: true);

  @override
  Future<AudioOperationResult> setBpm(int bpm) async =>
      const AudioOperationResult(success: true);

  @override
  Future<void> dispose() async {}
}

/// Fake PermissionService that always grants permissions.
class FakePermissionService implements PermissionService {
  @override
  Future<PermissionResult> checkMicrophone() async =>
      const PermissionResult(state: PermissionState.granted);

  @override
  Future<PermissionResult> requestMicrophone() async =>
      const PermissionResult(state: PermissionState.granted);
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
}

/// Fake KeyrxBridge that provides minimal stub implementations.
///
/// This fake bridge does not load any native library and returns
/// safe default values for all operations.
class FakeBridge implements KeyrxBridge {
  final StreamController<BridgeClassification> _classificationController =
      StreamController<BridgeClassification>.broadcast();
  final StreamController<BridgeState> _stateController =
      StreamController<BridgeState>.broadcast();

  bool _initialized = true;

  // Mixin interface implementations - these are required by the mixins.
  @override
  KeyrxBindings? get bindings => null;

  @override
  StreamController<BridgeClassification>? get classificationController =>
      _classificationController;

  @override
  StreamController<BridgeState>? get stateController => _stateController;

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
  Stream<BridgeClassification>? get classificationStream =>
      _classificationController.stream;

  @override
  Stream<BridgeState>? get stateStream => _stateController.stream;

  @override
  Future<bool> startAudio({required int bpm}) async => true;

  @override
  Future<bool> stopAudio() async => true;

  @override
  Future<bool> setBpm(int bpm) async => true;

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
  DeviceListResult listDevices() => const DeviceListResult(devices: []);

  @override
  int selectDevice(String path) => 0;

  @override
  ScriptValidationResult checkScript(String path) =>
      const ScriptValidationResult(valid: true, errors: []);

  @override
  TestDiscoveryResult discoverTests(String path) =>
      const TestDiscoveryResult(tests: []);

  @override
  TestRunResult runTests(String path, {String? filter}) =>
      const TestRunResult(total: 0, passed: 0, failed: 0, durationMs: 0, results: []);

  @override
  SimulationResult simulate(
    List<KeyInput> keys, {
    String? scriptPath,
    bool comboMode = false,
  }) =>
      const SimulationResult(mappings: [], activeLayers: [], pending: []);

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
  DiscoveryStartResult startDiscovery(
    String deviceId,
    int rows,
    List<int> colsPerRow,
  ) =>
      DiscoveryStartResult.error('Not implemented in fake');

  @override
  int cancelDiscovery() => 0;

  @override
  RecordingStartResult startRecording(String path) =>
      RecordingStartResult.error('Not implemented in fake');

  @override
  RecordingStopResult stopRecording() =>
      RecordingStopResult.error('Not implemented in fake');

  /// Emit a classification event to listeners.
  void emitClassification(BridgeClassification classification) {
    _classificationController.add(classification);
  }

  /// Emit a state snapshot to listeners.
  void emitState(BridgeState state) {
    _stateController.add(state);
  }

  @override
  ValidationResult validateScript(String script, [ValidationOptions? options]) =>
      const ValidationResult(isValid: true, errors: [], warnings: []);

  @override
  List<String> suggestKeys(String partial) => const [];

  @override
  List<String> allKeyNames() => const [];

  @override
  bool get isEventCallbackRegistered => false;

  @override
  void registerEventCallback(void Function(String) callback) {}

  @override
  void unregisterEventCallback() {}

  @override
  Future<void> dispose() async {
    await _classificationController.close();
    await _stateController.close();
  }
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
}
