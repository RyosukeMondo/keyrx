/// High-level bridge to KeyRx Core.
///
/// Provides a Dart-friendly API over the raw FFI bindings.
library;

import 'dart:async';
import 'dart:convert';
import 'dart:ffi';
import 'dart:io';

import 'package:ffi/ffi.dart';

import 'bindings.dart';

/// Bridge to the KeyRx Core library.
class KeyrxBridge {
  static KeyrxBridge? _currentInstance;
  KeyrxBindings? _bindings;
  bool _initialized = false;
  Object? _loadFailure;
  StreamController<BridgeClassification>? _classificationController;
  StreamController<BridgeState>? _stateController;

  KeyrxBridge._({KeyrxBindings? bindings, Object? loadFailure})
    : _bindings = bindings,
      _loadFailure = loadFailure {
    if (_bindings != null) {
      _setupClassificationStream();
    }
    _currentInstance = this;
  }

  /// Create a new bridge instance.
  ///
  /// Gracefully handles missing native libraries so the UI can surface errors
  /// instead of crashing at startup.
  factory KeyrxBridge.open() {
    try {
      final lib = _loadLibrary();
      return KeyrxBridge._(bindings: KeyrxBindings(lib));
    } catch (e) {
      return KeyrxBridge._(loadFailure: e);
    }
  }

  /// Load the native library based on platform.
  static DynamicLibrary _loadLibrary() {
    if (Platform.isLinux) {
      return DynamicLibrary.open('libkeyrx_core.so');
    } else if (Platform.isWindows) {
      return DynamicLibrary.open('keyrx_core.dll');
    } else {
      throw UnsupportedError(
          'Platform not supported: ${Platform.operatingSystem}');
    }
  }

  /// Initialize the KeyRx engine.
  bool initialize() {
    if (_initialized) return true;
    if (_bindings == null || _loadFailure != null) {
      return false;
    }

    final result = _bindings!.init();
    _initialized = result == 0;
    return _initialized;
  }

  /// Get the core library version.
  String get version {
    if (_bindings == null) {
      return 'unavailable';
    }

    final ptr = _bindings!.version();
    return ptr.cast<Utf8>().toDartString();
  }

  /// Subscribe to classification events if the native layer exposes them.
  Stream<BridgeClassification>? get classificationStream =>
      _classificationController?.stream;

  /// Subscribe to engine state snapshots if exposed by the native layer.
  Stream<BridgeState>? get stateStream => _stateController?.stream;

  /// Start audio capture/processing via the native bridge.
  ///
  /// Returns `true` when the bridge reports success.
  Future<bool> startAudio({required int bpm}) async {
    final start = _bindings?.startAudio;
    if (start == null) {
      return false;
    }
    return start(bpm) == 0;
  }

  /// Stop audio capture/processing via the native bridge.
  Future<bool> stopAudio() async {
    final stop = _bindings?.stopAudio;
    if (stop == null) {
      return false;
    }
    return stop() == 0;
  }

  /// Update BPM on the native engine.
  Future<bool> setBpm(int bpm) async {
    final setter = _bindings?.setBpm;
    if (setter == null) {
      return false;
    }
    return setter(bpm) == 0;
  }

  /// Load a Rhai script file.
  bool loadScript(String path) {
    if (_bindings == null) return false;
    final pathPtr = path.toNativeUtf8();
    try {
      final result = _bindings!.loadScript(pathPtr);
      return result == 0;
    } finally {
      calloc.free(pathPtr);
    }
  }

  /// Evaluate a console command if the native binding is available.
  ///
  /// Returns stdout/stderr text. Caller interprets success.
  Future<String?> eval(String command) async {
    final evalFn = _bindings?.eval;
    if (evalFn == null) return 'error: eval not available';

    final cmdPtr = command.toNativeUtf8();
    Pointer<Char>? responsePtr;
    try {
      responsePtr = evalFn(cmdPtr);
      if (responsePtr == null || responsePtr == nullptr) {
        return 'error: eval returned null';
      }

      final raw = responsePtr.cast<Utf8>().toDartString();
      return _normalizeEval(raw);
    } catch (e) {
      return 'error: $e';
    } finally {
      calloc.free(cmdPtr);
      if (responsePtr != null) {
        try {
          _bindings?.freeString(responsePtr);
        } catch (_) {}
      }
    }
  }

  /// List canonical key names from the core definition table.
  KeyRegistryResult listKeys() {
    final listFn = _bindings?.listKeys;
    if (listFn == null) {
      return KeyRegistryResult.fallback('error: listKeys not available');
    }

    Pointer<Char>? ptr;
    try {
      ptr = listFn();
      if (ptr == null || ptr == nullptr) {
        return KeyRegistryResult.fallback('error: listKeys returned null');
      }

      final jsonStr = ptr.cast<Utf8>().toDartString();
      return KeyRegistryResult.parse(jsonStr);
    } catch (e) {
      return KeyRegistryResult.fallback('error: $e');
    } finally {
      if (ptr != null) {
        try {
          _bindings?.freeString(ptr);
        } catch (_) {}
      }
    }
  }

  /// Check if the engine is initialized.
  bool get isInitialized => _initialized;

  /// If the native library failed to load, this captures the underlying error.
  Object? get loadFailure => _loadFailure;

  /// Check if emergency bypass mode is currently active.
  ///
  /// When bypass mode is active, all key remapping is disabled.
  bool isBypassActive() {
    final fn = _bindings?.isBypassActive;
    if (fn == null) return false;
    return fn();
  }

  /// Set the emergency bypass mode state.
  ///
  /// [active] - If true, enable bypass mode (disable remapping).
  ///            If false, disable bypass mode (re-enable remapping).
  void setBypass(bool active) {
    final fn = _bindings?.setBypass;
    if (fn == null) return;
    fn(active);
  }

  /// List available keyboard devices.
  ///
  /// Returns a list of [KeyboardDevice] or an error result.
  DeviceListResult listDevices() {
    final listFn = _bindings?.listDevices;
    if (listFn == null) {
      return DeviceListResult.error('listDevices not available');
    }

    Pointer<Char>? ptr;
    try {
      ptr = listFn();
      if (ptr == nullptr) {
        return DeviceListResult.error('listDevices returned null');
      }

      final raw = ptr.cast<Utf8>().toDartString();
      return DeviceListResult.parse(raw);
    } catch (e) {
      return DeviceListResult.error('$e');
    } finally {
      if (ptr != null && ptr != nullptr) {
        try {
          _bindings?.freeString(ptr);
        } catch (_) {}
      }
    }
  }

  /// Select a device by path for the engine to use.
  ///
  /// Returns 0 on success, negative on error:
  /// - -1: Null pointer
  /// - -2: Invalid UTF-8
  /// - -3: Device path does not exist
  /// - -4: Lock error
  int selectDevice(String path) {
    final selectFn = _bindings?.selectDevice;
    if (selectFn == null) return -1;

    final pathPtr = path.toNativeUtf8();
    try {
      return selectFn(pathPtr);
    } finally {
      calloc.free(pathPtr);
    }
  }

  /// Validate a Rhai script without executing it.
  ///
  /// Returns validation result with any syntax errors.
  ScriptValidationResult checkScript(String path) {
    final checkFn = _bindings?.checkScript;
    if (checkFn == null) {
      return ScriptValidationResult.error('checkScript not available');
    }

    final pathPtr = path.toNativeUtf8();
    Pointer<Char>? ptr;
    try {
      ptr = checkFn(pathPtr);
      if (ptr == nullptr) {
        return ScriptValidationResult.error('checkScript returned null');
      }

      final raw = ptr.cast<Utf8>().toDartString();
      return ScriptValidationResult.parse(raw);
    } catch (e) {
      return ScriptValidationResult.error('$e');
    } finally {
      calloc.free(pathPtr);
      if (ptr != null && ptr != nullptr) {
        try {
          _bindings?.freeString(ptr);
        } catch (_) {}
      }
    }
  }

  /// Discover test functions in a Rhai script.
  ///
  /// Returns list of discovered test functions.
  TestDiscoveryResult discoverTests(String path) {
    final discoverFn = _bindings?.discoverTests;
    if (discoverFn == null) {
      return TestDiscoveryResult.error('discoverTests not available');
    }

    final pathPtr = path.toNativeUtf8();
    Pointer<Char>? ptr;
    try {
      ptr = discoverFn(pathPtr);
      if (ptr == nullptr) {
        return TestDiscoveryResult.error('discoverTests returned null');
      }

      final raw = ptr.cast<Utf8>().toDartString();
      return TestDiscoveryResult.parse(raw);
    } catch (e) {
      return TestDiscoveryResult.error('$e');
    } finally {
      calloc.free(pathPtr);
      if (ptr != null && ptr != nullptr) {
        try {
          _bindings?.freeString(ptr);
        } catch (_) {}
      }
    }
  }

  /// Run tests in a Rhai script with optional filter.
  ///
  /// [path] - Path to the script file.
  /// [filter] - Optional pattern to filter test names (null for all tests).
  TestRunResult runTests(String path, {String? filter}) {
    final runFn = _bindings?.runTests;
    if (runFn == null) {
      return TestRunResult.error('runTests not available');
    }

    final pathPtr = path.toNativeUtf8();
    final filterPtr = filter?.toNativeUtf8() ?? nullptr;
    Pointer<Char>? ptr;
    try {
      ptr = runFn(pathPtr, filterPtr);
      if (ptr == nullptr) {
        return TestRunResult.error('runTests returned null');
      }

      final raw = ptr.cast<Utf8>().toDartString();
      return TestRunResult.parse(raw);
    } catch (e) {
      return TestRunResult.error('$e');
    } finally {
      calloc.free(pathPtr);
      if (filterPtr != nullptr) {
        calloc.free(filterPtr);
      }
      if (ptr != null && ptr != nullptr) {
        try {
          _bindings?.freeString(ptr);
        } catch (_) {}
      }
    }
  }

  /// Simulate key sequences through the engine.
  ///
  /// [keys] - List of key inputs to simulate.
  /// [scriptPath] - Optional path to Rhai script (null uses active script).
  /// [comboMode] - If true, keys are pressed simultaneously; otherwise sequentially.
  SimulationResult simulate(
    List<KeyInput> keys, {
    String? scriptPath,
    bool comboMode = false,
  }) {
    final simFn = _bindings?.simulate;
    if (simFn == null) {
      return SimulationResult.error('simulate not available');
    }

    final keysJson = json.encode(keys.map((k) => k.toJson()).toList());
    final keysPtr = keysJson.toNativeUtf8();
    final scriptPtr = scriptPath?.toNativeUtf8() ?? nullptr;
    Pointer<Char>? ptr;
    try {
      ptr = simFn(keysPtr, scriptPtr, comboMode);
      if (ptr == nullptr) {
        return SimulationResult.error('simulate returned null');
      }

      final raw = ptr.cast<Utf8>().toDartString();
      return SimulationResult.parse(raw);
    } catch (e) {
      return SimulationResult.error('$e');
    } finally {
      calloc.free(keysPtr);
      if (scriptPtr != nullptr) {
        calloc.free(scriptPtr);
      }
      if (ptr != null && ptr != nullptr) {
        try {
          _bindings?.freeString(ptr);
        } catch (_) {}
      }
    }
  }

  /// List session files in a directory.
  SessionListResult listSessions(String dirPath) {
    final listFn = _bindings?.listSessions;
    if (listFn == null) {
      return SessionListResult.error('listSessions not available');
    }

    final dirPtr = dirPath.toNativeUtf8();
    Pointer<Char>? ptr;
    try {
      ptr = listFn(dirPtr);
      if (ptr == nullptr) {
        return SessionListResult.error('listSessions returned null');
      }

      final raw = ptr.cast<Utf8>().toDartString();
      return SessionListResult.parse(raw);
    } catch (e) {
      return SessionListResult.error('$e');
    } finally {
      calloc.free(dirPtr);
      if (ptr != null && ptr != nullptr) {
        try {
          _bindings?.freeString(ptr);
        } catch (_) {}
      }
    }
  }

  /// Analyze a session file.
  SessionAnalysisResult analyzeSession(String path) {
    final analyzeFn = _bindings?.analyzeSession;
    if (analyzeFn == null) {
      return SessionAnalysisResult.error('analyzeSession not available');
    }

    final pathPtr = path.toNativeUtf8();
    Pointer<Char>? ptr;
    try {
      ptr = analyzeFn(pathPtr);
      if (ptr == nullptr) {
        return SessionAnalysisResult.error('analyzeSession returned null');
      }

      final raw = ptr.cast<Utf8>().toDartString();
      return SessionAnalysisResult.parse(raw);
    } catch (e) {
      return SessionAnalysisResult.error('$e');
    } finally {
      calloc.free(pathPtr);
      if (ptr != null && ptr != nullptr) {
        try {
          _bindings?.freeString(ptr);
        } catch (_) {}
      }
    }
  }

  /// Replay a session file with optional verification.
  ReplayResult replaySession(String path, {bool verify = false}) {
    final replayFn = _bindings?.replaySession;
    if (replayFn == null) {
      return ReplayResult.error('replaySession not available');
    }

    final pathPtr = path.toNativeUtf8();
    Pointer<Char>? ptr;
    try {
      ptr = replayFn(pathPtr, verify);
      if (ptr == nullptr) {
        return ReplayResult.error('replaySession returned null');
      }

      final raw = ptr.cast<Utf8>().toDartString();
      return ReplayResult.parse(raw);
    } catch (e) {
      return ReplayResult.error('$e');
    } finally {
      calloc.free(pathPtr);
      if (ptr != null && ptr != nullptr) {
        try {
          _bindings?.freeString(ptr);
        } catch (_) {}
      }
    }
  }

  /// Run benchmark with specified iterations.
  BenchmarkResult runBenchmark(int iterations, {String? scriptPath}) {
    final benchFn = _bindings?.runBenchmark;
    if (benchFn == null) {
      return BenchmarkResult.error('runBenchmark not available');
    }

    final scriptPtr = scriptPath?.toNativeUtf8() ?? nullptr;
    Pointer<Char>? ptr;
    try {
      ptr = benchFn(iterations, scriptPtr);
      if (ptr == nullptr) {
        return BenchmarkResult.error('runBenchmark returned null');
      }

      final raw = ptr.cast<Utf8>().toDartString();
      return BenchmarkResult.parse(raw);
    } catch (e) {
      return BenchmarkResult.error('$e');
    } finally {
      if (scriptPtr != nullptr) {
        calloc.free(scriptPtr);
      }
      if (ptr != null && ptr != nullptr) {
        try {
          _bindings?.freeString(ptr);
        } catch (_) {}
      }
    }
  }

  /// Run system diagnostics.
  DoctorResult runDoctor() {
    final doctorFn = _bindings?.runDoctor;
    if (doctorFn == null) {
      return DoctorResult.error('runDoctor not available');
    }

    Pointer<Char>? ptr;
    try {
      ptr = doctorFn();
      if (ptr == nullptr) {
        return DoctorResult.error('runDoctor returned null');
      }

      final raw = ptr.cast<Utf8>().toDartString();
      return DoctorResult.parse(raw);
    } catch (e) {
      return DoctorResult.error('$e');
    } finally {
      if (ptr != null && ptr != nullptr) {
        try {
          _bindings?.freeString(ptr);
        } catch (_) {}
      }
    }
  }

  /// Start a discovery session for a device.
  ///
  /// [deviceId] - Device identifier as "vendorId:productId" (hex format)
  /// [rows] - Number of rows in the keyboard layout
  /// [colsPerRow] - Number of columns for each row
  DiscoveryStartResult startDiscovery(
    String deviceId,
    int rows,
    List<int> colsPerRow,
  ) {
    final discoveryFn = _bindings?.startDiscovery;
    if (discoveryFn == null) {
      return DiscoveryStartResult.error('startDiscovery not available');
    }

    final devicePtr = deviceId.toNativeUtf8();
    final colsJson = json.encode(colsPerRow);
    final colsPtr = colsJson.toNativeUtf8();
    Pointer<Char>? ptr;
    try {
      ptr = discoveryFn(devicePtr, rows, colsPtr);
      if (ptr == nullptr) {
        return DiscoveryStartResult.error('startDiscovery returned null');
      }

      final raw = ptr.cast<Utf8>().toDartString();
      return DiscoveryStartResult.parse(raw);
    } catch (e) {
      return DiscoveryStartResult.error('$e');
    } finally {
      calloc.free(devicePtr);
      calloc.free(colsPtr);
      if (ptr != null && ptr != nullptr) {
        try {
          _bindings?.freeString(ptr);
        } catch (_) {}
      }
    }
  }

  /// Start recording key events to a file.
  RecordingStartResult startRecording(String path) {
    final startFn = _bindings?.startRecording;
    if (startFn == null) {
      return RecordingStartResult.error('startRecording not available');
    }

    final pathPtr = path.toNativeUtf8();
    Pointer<Char>? ptr;
    try {
      ptr = startFn(pathPtr);
      if (ptr == nullptr) {
        return RecordingStartResult.error('startRecording returned null');
      }

      final raw = ptr.cast<Utf8>().toDartString();
      return RecordingStartResult.parse(raw);
    } catch (e) {
      return RecordingStartResult.error('$e');
    } finally {
      calloc.free(pathPtr);
      if (ptr != null && ptr != nullptr) {
        try {
          _bindings?.freeString(ptr);
        } catch (_) {}
      }
    }
  }

  /// Stop recording and save the session.
  RecordingStopResult stopRecording() {
    final stopFn = _bindings?.stopRecording;
    if (stopFn == null) {
      return RecordingStopResult.error('stopRecording not available');
    }

    Pointer<Char>? ptr;
    try {
      ptr = stopFn();
      if (ptr == nullptr) {
        return RecordingStopResult.error('stopRecording returned null');
      }

      final raw = ptr.cast<Utf8>().toDartString();
      return RecordingStopResult.parse(raw);
    } catch (e) {
      return RecordingStopResult.error('$e');
    } finally {
      if (ptr != null && ptr != nullptr) {
        try {
          _bindings?.freeString(ptr);
        } catch (_) {}
      }
    }
  }

  /// Close any native resources and stop dispatching callbacks.
  Future<void> dispose() async {
    _currentInstance = null;
    await _classificationController?.close();
    _classificationController = null;
    await _stateController?.close();
    _stateController = null;
  }

  void _setupClassificationStream() {
    if (_bindings?.onClassification == null) {
      return;
    }

    _classificationController ??=
        StreamController<BridgeClassification>.broadcast();

    _bindings!.onClassification!(
      Pointer.fromFunction<KeyrxClassificationCallbackNative>(
        _handleClassification,
      ),
    );

    if (_bindings?.onState != null) {
      _stateController ??= StreamController<BridgeState>.broadcast();
      _bindings!.onState!(
        Pointer.fromFunction<KeyrxStateCallbackNative>(
          _handleState,
        ),
      );
    }
  }

  static void _handleClassification(Pointer<Uint8> ptr, int length) {
    final instance = _currentInstance;
    final controller = instance?._classificationController;
    if (instance == null || controller == null || controller.isClosed) {
      return;
    }

    try {
      final bytes = ptr.asTypedList(length);
      final payload = json.decode(utf8.decode(bytes));
      if (payload is! Map<String, dynamic>) return;

      final label = payload['label'] as String? ?? 'unknown';
      final confidence = (payload['confidence'] as num?)?.toDouble() ?? 0.0;
      final timestampMs =
          (payload['timestamp'] as num?)?.toInt() ?? DateTime.now().millisecondsSinceEpoch;

      controller.add(
        BridgeClassification(
          label: label,
          confidence: confidence,
          timestamp: DateTime.fromMillisecondsSinceEpoch(timestampMs),
        ),
      );
    } catch (_) {
      // Swallow malformed payloads to avoid crashing listeners.
    }
  }

  static void _handleState(Pointer<Uint8> ptr, int length) {
    final instance = _currentInstance;
    final controller = instance?._stateController;
    if (instance == null || controller == null || controller.isClosed) {
      return;
    }

    try {
      final bytes = ptr.asTypedList(length);
      final payload = json.decode(utf8.decode(bytes));
      if (payload is! Map<String, dynamic>) return;

      final layers = (payload['layers'] as List<dynamic>?)
              ?.map((e) => e.toString())
              .toList() ??
          const <String>[];
      final modifiers = (payload['modifiers'] as List<dynamic>?)
              ?.map((e) => e.toString())
              .toList() ??
          const <String>[];
      final held = (payload['held'] as List<dynamic>?)
              ?.map((e) => e.toString())
              .toList() ??
          const <String>[];
      final pending = (payload['pending'] as List<dynamic>?)
              ?.map((e) => e.toString())
              .toList() ??
          const <String>[];
      final lastEvent = payload['event'] as String?;
      final latencyUs = (payload['latency_us'] as num?)?.toInt();
      final timing = _parseTiming(payload['timing']);

      controller.add(
        BridgeState(
          layers: layers,
          modifiers: modifiers,
          heldKeys: held,
          pendingDecisions: pending,
          lastEvent: lastEvent,
          latencyUs: latencyUs,
          timing: timing,
          timestamp: DateTime.now(),
        ),
      );
    } catch (_) {
      // Swallow malformed payloads to avoid crashing listeners.
    }
  }

  static String _normalizeEval(String raw) {
    final trimmed = raw.trim();
    final lower = trimmed.toLowerCase();
    if (lower.startsWith('ok:') || lower.startsWith('error:')) {
      return trimmed;
    }
    return 'ok:$trimmed';
  }

  static BridgeTiming? _parseTiming(dynamic raw) {
    if (raw is! Map<String, dynamic>) return null;
    try {
      return BridgeTiming(
        tapTimeoutMs: (raw['tap_timeout_ms'] as num?)?.toInt(),
        comboTimeoutMs: (raw['combo_timeout_ms'] as num?)?.toInt(),
        holdDelayMs: (raw['hold_delay_ms'] as num?)?.toInt(),
        eagerTap: raw['eager_tap'] as bool?,
        permissiveHold: raw['permissive_hold'] as bool?,
        retroTap: raw['retro_tap'] as bool?,
      );
    } catch (_) {
      return null;
    }
  }
}

/// Simple payload representing a classification event emitted by the bridge.
class BridgeClassification {
  const BridgeClassification({
    required this.label,
    required this.confidence,
    required this.timestamp,
  });

  final String label;
  final double confidence;
  final DateTime timestamp;
}

/// State snapshot payload from the bridge.
class BridgeState {
  const BridgeState({
    required this.layers,
    required this.modifiers,
    required this.heldKeys,
    required this.pendingDecisions,
    required this.timestamp,
    this.lastEvent,
    this.latencyUs,
    this.timing,
  });

  final List<String> layers;
  final List<String> modifiers;
  final List<String> heldKeys;
  final List<String> pendingDecisions;
  final DateTime timestamp;
  final String? lastEvent;
  final int? latencyUs;
  final BridgeTiming? timing;
}

/// Timing configuration snapshot from the engine.
class BridgeTiming {
  const BridgeTiming({
    this.tapTimeoutMs,
    this.comboTimeoutMs,
    this.holdDelayMs,
    this.eagerTap,
    this.permissiveHold,
    this.retroTap,
  });

  final int? tapTimeoutMs;
  final int? comboTimeoutMs;
  final int? holdDelayMs;
  final bool? eagerTap;
  final bool? permissiveHold;
  final bool? retroTap;
}

/// Canonical key definition entry returned by the FFI registry.
class KeyRegistryEntry {
  const KeyRegistryEntry({
    required this.name,
    this.aliases = const [],
    this.evdev,
    this.vk,
  });

  final String name;
  final List<String> aliases;
  final int? evdev;
  final int? vk;
}

/// Result of requesting the key registry (with fallback on errors).
class KeyRegistryResult {
  const KeyRegistryResult({
    required this.entries,
    this.error,
    this.usedFallback = false,
  });

  factory KeyRegistryResult.fallback(String error) => KeyRegistryResult(
        entries: const [],
        error: error,
        usedFallback: true,
      );

  factory KeyRegistryResult.parse(String raw) {
    final trimmed = raw.trim();
    if (trimmed.toLowerCase().startsWith('error:')) {
      return KeyRegistryResult.fallback(trimmed);
    }

    final payload = trimmed.toLowerCase().startsWith('ok:')
        ? trimmed.substring(trimmed.indexOf(':') + 1)
        : trimmed;

    try {
      final decoded = json.decode(payload);
      if (decoded is! List) {
        return KeyRegistryResult.fallback('error: invalid registry payload');
      }

      final entries = decoded.map((entry) {
        if (entry is! Map) {
          return const KeyRegistryEntry(name: '');
        }
        final name = entry['name']?.toString() ?? '';
        final aliases = (entry['aliases'] as List<dynamic>?)
                ?.map((e) => e.toString())
                .toList() ??
            const <String>[];
        final evdev = (entry['evdev'] as num?)?.toInt();
        final vk = (entry['vk'] as num?)?.toInt();
        return KeyRegistryEntry(
          name: name,
          aliases: aliases,
          evdev: evdev,
          vk: vk,
        );
      }).where((entry) => entry.name.isNotEmpty).toList();

      if (entries.isEmpty) {
        return KeyRegistryResult.fallback('error: empty registry payload');
      }

      return KeyRegistryResult(entries: entries);
    } catch (e) {
      return KeyRegistryResult.fallback('error: $e');
    }
  }

  final List<KeyRegistryEntry> entries;
  final String? error;
  final bool usedFallback;

  List<String> get names => entries.map((e) => e.name).toList();
}

/// Keyboard device information from the FFI layer.
class KeyboardDevice {
  const KeyboardDevice({
    required this.name,
    required this.vendorId,
    required this.productId,
    required this.path,
    required this.hasProfile,
  });

  final String name;
  final int vendorId;
  final int productId;
  final String path;
  final bool hasProfile;
}

/// Result of listing keyboard devices.
class DeviceListResult {
  const DeviceListResult({
    required this.devices,
    this.errorMessage,
  });

  factory DeviceListResult.error(String message) => DeviceListResult(
        devices: const [],
        errorMessage: message,
      );

  factory DeviceListResult.parse(String raw) {
    final trimmed = raw.trim();
    if (trimmed.toLowerCase().startsWith('error:')) {
      return DeviceListResult.error(trimmed.substring('error:'.length).trim());
    }

    final payload = trimmed.toLowerCase().startsWith('ok:')
        ? trimmed.substring(trimmed.indexOf(':') + 1)
        : trimmed;

    try {
      final decoded = json.decode(payload);
      if (decoded is! List) {
        return DeviceListResult.error('invalid device list payload');
      }

      final devices = decoded.map((entry) {
        if (entry is! Map<String, dynamic>) {
          return null;
        }
        return KeyboardDevice(
          name: entry['name']?.toString() ?? 'Unknown',
          vendorId: (entry['vendorId'] as num?)?.toInt() ?? 0,
          productId: (entry['productId'] as num?)?.toInt() ?? 0,
          path: entry['path']?.toString() ?? '',
          hasProfile: entry['hasProfile'] as bool? ?? false,
        );
      }).whereType<KeyboardDevice>().toList();

      return DeviceListResult(devices: devices);
    } catch (e) {
      return DeviceListResult.error('$e');
    }
  }

  final List<KeyboardDevice> devices;
  final String? errorMessage;

  bool get hasError => errorMessage != null;
}

/// Script validation error detail.
class ScriptValidationError {
  const ScriptValidationError({
    this.line,
    this.column,
    required this.message,
  });

  final int? line;
  final int? column;
  final String message;
}

/// Script validation result.
class ScriptValidationResult {
  const ScriptValidationResult({
    required this.valid,
    required this.errors,
    this.errorMessage,
  });

  factory ScriptValidationResult.error(String message) => ScriptValidationResult(
        valid: false,
        errors: const [],
        errorMessage: message,
      );

  factory ScriptValidationResult.parse(String raw) {
    final trimmed = raw.trim();
    if (trimmed.toLowerCase().startsWith('error:')) {
      return ScriptValidationResult.error(
          trimmed.substring('error:'.length).trim());
    }

    final payload = trimmed.toLowerCase().startsWith('ok:')
        ? trimmed.substring(trimmed.indexOf(':') + 1)
        : trimmed;

    try {
      final decoded = json.decode(payload);
      if (decoded is! Map<String, dynamic>) {
        return ScriptValidationResult.error('invalid validation payload');
      }

      final valid = decoded['valid'] as bool? ?? false;
      final errorsList = decoded['errors'] as List<dynamic>? ?? [];
      final errors = errorsList.map((e) {
        if (e is! Map<String, dynamic>) {
          return const ScriptValidationError(message: 'unknown error');
        }
        return ScriptValidationError(
          line: (e['line'] as num?)?.toInt(),
          column: (e['column'] as num?)?.toInt(),
          message: e['message']?.toString() ?? 'unknown error',
        );
      }).toList();

      return ScriptValidationResult(valid: valid, errors: errors);
    } catch (e) {
      return ScriptValidationResult.error('$e');
    }
  }

  final bool valid;
  final List<ScriptValidationError> errors;
  final String? errorMessage;

  bool get hasError => errorMessage != null;
}

/// Discovered test function.
class DiscoveredTest {
  const DiscoveredTest({
    required this.name,
    required this.file,
    this.line,
  });

  final String name;
  final String file;
  final int? line;
}

/// Test discovery result.
class TestDiscoveryResult {
  const TestDiscoveryResult({
    required this.tests,
    this.errorMessage,
  });

  factory TestDiscoveryResult.error(String message) => TestDiscoveryResult(
        tests: const [],
        errorMessage: message,
      );

  factory TestDiscoveryResult.parse(String raw) {
    final trimmed = raw.trim();
    if (trimmed.toLowerCase().startsWith('error:')) {
      return TestDiscoveryResult.error(
          trimmed.substring('error:'.length).trim());
    }

    final payload = trimmed.toLowerCase().startsWith('ok:')
        ? trimmed.substring(trimmed.indexOf(':') + 1)
        : trimmed;

    try {
      final decoded = json.decode(payload);
      if (decoded is! List) {
        return TestDiscoveryResult.error('invalid test list payload');
      }

      final tests = decoded.map((e) {
        if (e is! Map<String, dynamic>) return null;
        return DiscoveredTest(
          name: e['name']?.toString() ?? '',
          file: e['file']?.toString() ?? '',
          line: (e['line'] as num?)?.toInt(),
        );
      }).whereType<DiscoveredTest>().toList();

      return TestDiscoveryResult(tests: tests);
    } catch (e) {
      return TestDiscoveryResult.error('$e');
    }
  }

  final List<DiscoveredTest> tests;
  final String? errorMessage;

  bool get hasError => errorMessage != null;
}

/// Individual test result.
class TestResult {
  const TestResult({
    required this.name,
    required this.passed,
    this.error,
    required this.durationMs,
  });

  final String name;
  final bool passed;
  final String? error;
  final double durationMs;
}

/// Test run result.
class TestRunResult {
  const TestRunResult({
    required this.total,
    required this.passed,
    required this.failed,
    required this.durationMs,
    required this.results,
    this.errorMessage,
  });

  factory TestRunResult.error(String message) => TestRunResult(
        total: 0,
        passed: 0,
        failed: 0,
        durationMs: 0,
        results: const [],
        errorMessage: message,
      );

  factory TestRunResult.parse(String raw) {
    final trimmed = raw.trim();
    if (trimmed.toLowerCase().startsWith('error:')) {
      return TestRunResult.error(trimmed.substring('error:'.length).trim());
    }

    final payload = trimmed.toLowerCase().startsWith('ok:')
        ? trimmed.substring(trimmed.indexOf(':') + 1)
        : trimmed;

    try {
      final decoded = json.decode(payload);
      if (decoded is! Map<String, dynamic>) {
        return TestRunResult.error('invalid test run payload');
      }

      final resultsList = decoded['results'] as List<dynamic>? ?? [];
      final results = resultsList.map((e) {
        if (e is! Map<String, dynamic>) return null;
        return TestResult(
          name: e['name']?.toString() ?? '',
          passed: e['passed'] as bool? ?? false,
          error: e['error']?.toString(),
          durationMs: (e['durationMs'] as num?)?.toDouble() ?? 0,
        );
      }).whereType<TestResult>().toList();

      return TestRunResult(
        total: (decoded['total'] as num?)?.toInt() ?? 0,
        passed: (decoded['passed'] as num?)?.toInt() ?? 0,
        failed: (decoded['failed'] as num?)?.toInt() ?? 0,
        durationMs: (decoded['durationMs'] as num?)?.toDouble() ?? 0,
        results: results,
      );
    } catch (e) {
      return TestRunResult.error('$e');
    }
  }

  final int total;
  final int passed;
  final int failed;
  final double durationMs;
  final List<TestResult> results;
  final String? errorMessage;

  bool get hasError => errorMessage != null;
}

/// Key mapping from simulation.
class SimulationMapping {
  const SimulationMapping({
    required this.input,
    required this.output,
    required this.decision,
  });

  final String input;
  final String output;
  final String decision;
}

/// Simulation result.
class SimulationResult {
  const SimulationResult({
    required this.mappings,
    required this.activeLayers,
    required this.pending,
    this.errorMessage,
  });

  factory SimulationResult.error(String message) => SimulationResult(
        mappings: const [],
        activeLayers: const [],
        pending: const [],
        errorMessage: message,
      );

  factory SimulationResult.parse(String raw) {
    final trimmed = raw.trim();
    if (trimmed.toLowerCase().startsWith('error:')) {
      return SimulationResult.error(trimmed.substring('error:'.length).trim());
    }

    final payload = trimmed.toLowerCase().startsWith('ok:')
        ? trimmed.substring(trimmed.indexOf(':') + 1)
        : trimmed;

    try {
      final decoded = json.decode(payload);
      if (decoded is! Map<String, dynamic>) {
        return SimulationResult.error('invalid simulation payload');
      }

      final mappingsList = decoded['mappings'] as List<dynamic>? ?? [];
      final mappings = mappingsList.map((e) {
        if (e is! Map<String, dynamic>) return null;
        return SimulationMapping(
          input: e['input']?.toString() ?? '',
          output: e['output']?.toString() ?? '',
          decision: e['decision']?.toString() ?? '',
        );
      }).whereType<SimulationMapping>().toList();

      final layers = (decoded['activeLayers'] as List<dynamic>?)
              ?.map((e) => e.toString())
              .toList() ??
          const <String>[];
      final pending = (decoded['pending'] as List<dynamic>?)
              ?.map((e) => e.toString())
              .toList() ??
          const <String>[];

      return SimulationResult(
        mappings: mappings,
        activeLayers: layers,
        pending: pending,
      );
    } catch (e) {
      return SimulationResult.error('$e');
    }
  }

  final List<SimulationMapping> mappings;
  final List<String> activeLayers;
  final List<String> pending;
  final String? errorMessage;

  bool get hasError => errorMessage != null;
}

/// Key input for simulation.
class KeyInput {
  const KeyInput({
    required this.code,
    this.holdMs,
  });

  final String code;
  final int? holdMs;

  Map<String, dynamic> toJson() => {
        'code': code,
        if (holdMs != null) 'holdMs': holdMs,
      };
}

/// Session info returned by list sessions.
class SessionInfo {
  const SessionInfo({
    required this.path,
    required this.name,
    required this.created,
    required this.eventCount,
    required this.durationMs,
  });

  final String path;
  final String name;
  final String created;
  final int eventCount;
  final double durationMs;
}

/// Session list result.
class SessionListResult {
  const SessionListResult({
    required this.sessions,
    this.errorMessage,
  });

  factory SessionListResult.error(String message) => SessionListResult(
        sessions: const [],
        errorMessage: message,
      );

  factory SessionListResult.parse(String raw) {
    final trimmed = raw.trim();
    if (trimmed.toLowerCase().startsWith('error:')) {
      return SessionListResult.error(trimmed.substring('error:'.length).trim());
    }

    final payload = trimmed.toLowerCase().startsWith('ok:')
        ? trimmed.substring(trimmed.indexOf(':') + 1)
        : trimmed;

    try {
      final decoded = json.decode(payload);
      if (decoded is! List) {
        return SessionListResult.error('invalid session list payload');
      }

      final sessions = decoded.map((e) {
        if (e is! Map<String, dynamic>) return null;
        return SessionInfo(
          path: e['path']?.toString() ?? '',
          name: e['name']?.toString() ?? '',
          created: e['created']?.toString() ?? '',
          eventCount: (e['eventCount'] as num?)?.toInt() ?? 0,
          durationMs: (e['durationMs'] as num?)?.toDouble() ?? 0,
        );
      }).whereType<SessionInfo>().toList();

      return SessionListResult(sessions: sessions);
    } catch (e) {
      return SessionListResult.error('$e');
    }
  }

  final List<SessionInfo> sessions;
  final String? errorMessage;

  bool get hasError => errorMessage != null;
}

/// Decision breakdown from session analysis.
class DecisionBreakdown {
  const DecisionBreakdown({
    required this.passThrough,
    required this.remap,
    required this.block,
    required this.tap,
    required this.hold,
    required this.combo,
    required this.layer,
    required this.modifier,
  });

  final int passThrough;
  final int remap;
  final int block;
  final int tap;
  final int hold;
  final int combo;
  final int layer;
  final int modifier;
}

/// Session analysis result.
class SessionAnalysis {
  const SessionAnalysis({
    required this.sessionPath,
    required this.eventCount,
    required this.durationMs,
    required this.avgLatencyUs,
    required this.minLatencyUs,
    required this.maxLatencyUs,
    required this.decisionBreakdown,
  });

  final String sessionPath;
  final int eventCount;
  final double durationMs;
  final int avgLatencyUs;
  final int minLatencyUs;
  final int maxLatencyUs;
  final DecisionBreakdown decisionBreakdown;
}

/// Session analysis result wrapper.
class SessionAnalysisResult {
  const SessionAnalysisResult({
    this.analysis,
    this.errorMessage,
  });

  factory SessionAnalysisResult.error(String message) => SessionAnalysisResult(
        errorMessage: message,
      );

  factory SessionAnalysisResult.parse(String raw) {
    final trimmed = raw.trim();
    if (trimmed.toLowerCase().startsWith('error:')) {
      return SessionAnalysisResult.error(
          trimmed.substring('error:'.length).trim());
    }

    final payload = trimmed.toLowerCase().startsWith('ok:')
        ? trimmed.substring(trimmed.indexOf(':') + 1)
        : trimmed;

    try {
      final decoded = json.decode(payload);
      if (decoded is! Map<String, dynamic>) {
        return SessionAnalysisResult.error('invalid analysis payload');
      }

      final breakdown = decoded['decisionBreakdown'] as Map<String, dynamic>?;
      final analysis = SessionAnalysis(
        sessionPath: decoded['sessionPath']?.toString() ?? '',
        eventCount: (decoded['eventCount'] as num?)?.toInt() ?? 0,
        durationMs: (decoded['durationMs'] as num?)?.toDouble() ?? 0,
        avgLatencyUs: (decoded['avgLatencyUs'] as num?)?.toInt() ?? 0,
        minLatencyUs: (decoded['minLatencyUs'] as num?)?.toInt() ?? 0,
        maxLatencyUs: (decoded['maxLatencyUs'] as num?)?.toInt() ?? 0,
        decisionBreakdown: DecisionBreakdown(
          passThrough: (breakdown?['passThrough'] as num?)?.toInt() ?? 0,
          remap: (breakdown?['remap'] as num?)?.toInt() ?? 0,
          block: (breakdown?['block'] as num?)?.toInt() ?? 0,
          tap: (breakdown?['tap'] as num?)?.toInt() ?? 0,
          hold: (breakdown?['hold'] as num?)?.toInt() ?? 0,
          combo: (breakdown?['combo'] as num?)?.toInt() ?? 0,
          layer: (breakdown?['layer'] as num?)?.toInt() ?? 0,
          modifier: (breakdown?['modifier'] as num?)?.toInt() ?? 0,
        ),
      );

      return SessionAnalysisResult(analysis: analysis);
    } catch (e) {
      return SessionAnalysisResult.error('$e');
    }
  }

  final SessionAnalysis? analysis;
  final String? errorMessage;

  bool get hasError => errorMessage != null;
}

/// Mismatch detail from replay verification.
class ReplayMismatch {
  const ReplayMismatch({
    required this.seq,
    required this.recorded,
    required this.actual,
  });

  final int seq;
  final String recorded;
  final String actual;
}

/// Session replay result.
class ReplayResult {
  const ReplayResult({
    required this.totalEvents,
    required this.matched,
    required this.mismatched,
    required this.success,
    required this.mismatches,
    this.errorMessage,
  });

  factory ReplayResult.error(String message) => ReplayResult(
        totalEvents: 0,
        matched: 0,
        mismatched: 0,
        success: false,
        mismatches: const [],
        errorMessage: message,
      );

  factory ReplayResult.parse(String raw) {
    final trimmed = raw.trim();
    if (trimmed.toLowerCase().startsWith('error:')) {
      return ReplayResult.error(trimmed.substring('error:'.length).trim());
    }

    final payload = trimmed.toLowerCase().startsWith('ok:')
        ? trimmed.substring(trimmed.indexOf(':') + 1)
        : trimmed;

    try {
      final decoded = json.decode(payload);
      if (decoded is! Map<String, dynamic>) {
        return ReplayResult.error('invalid replay payload');
      }

      final mismatchList = decoded['mismatches'] as List<dynamic>? ?? [];
      final mismatches = mismatchList.map((e) {
        if (e is! Map<String, dynamic>) return null;
        return ReplayMismatch(
          seq: (e['seq'] as num?)?.toInt() ?? 0,
          recorded: e['recorded']?.toString() ?? '',
          actual: e['actual']?.toString() ?? '',
        );
      }).whereType<ReplayMismatch>().toList();

      return ReplayResult(
        totalEvents: (decoded['totalEvents'] as num?)?.toInt() ?? 0,
        matched: (decoded['matched'] as num?)?.toInt() ?? 0,
        mismatched: (decoded['mismatched'] as num?)?.toInt() ?? 0,
        success: decoded['success'] as bool? ?? false,
        mismatches: mismatches,
      );
    } catch (e) {
      return ReplayResult.error('$e');
    }
  }

  final int totalEvents;
  final int matched;
  final int mismatched;
  final bool success;
  final List<ReplayMismatch> mismatches;
  final String? errorMessage;

  bool get hasError => errorMessage != null;
}

/// Benchmark result.
class BenchmarkResult {
  const BenchmarkResult({
    required this.minNs,
    required this.maxNs,
    required this.meanNs,
    required this.p99Ns,
    required this.iterations,
    required this.hasWarning,
    this.warning,
    this.errorMessage,
  });

  factory BenchmarkResult.error(String message) => BenchmarkResult(
        minNs: 0,
        maxNs: 0,
        meanNs: 0,
        p99Ns: 0,
        iterations: 0,
        hasWarning: false,
        errorMessage: message,
      );

  factory BenchmarkResult.parse(String raw) {
    final trimmed = raw.trim();
    if (trimmed.toLowerCase().startsWith('error:')) {
      return BenchmarkResult.error(trimmed.substring('error:'.length).trim());
    }

    final payload = trimmed.toLowerCase().startsWith('ok:')
        ? trimmed.substring(trimmed.indexOf(':') + 1)
        : trimmed;

    try {
      final decoded = json.decode(payload);
      if (decoded is! Map<String, dynamic>) {
        return BenchmarkResult.error('invalid benchmark payload');
      }

      return BenchmarkResult(
        minNs: (decoded['minNs'] as num?)?.toInt() ?? 0,
        maxNs: (decoded['maxNs'] as num?)?.toInt() ?? 0,
        meanNs: (decoded['meanNs'] as num?)?.toInt() ?? 0,
        p99Ns: (decoded['p99Ns'] as num?)?.toInt() ?? 0,
        iterations: (decoded['iterations'] as num?)?.toInt() ?? 0,
        hasWarning: decoded['hasWarning'] as bool? ?? false,
        warning: decoded['warning']?.toString(),
      );
    } catch (e) {
      return BenchmarkResult.error('$e');
    }
  }

  final int minNs;
  final int maxNs;
  final int meanNs;
  final int p99Ns;
  final int iterations;
  final bool hasWarning;
  final String? warning;
  final String? errorMessage;

  bool get hasError => errorMessage != null;
}

/// Diagnostic check result.
class DiagnosticCheck {
  const DiagnosticCheck({
    required this.name,
    required this.status,
    this.details,
    this.remediation,
  });

  final String name;
  final String status;
  final String? details;
  final String? remediation;
}

/// Doctor result.
class DoctorResult {
  const DoctorResult({
    required this.checks,
    required this.passed,
    required this.failed,
    required this.warned,
    this.errorMessage,
  });

  factory DoctorResult.error(String message) => DoctorResult(
        checks: const [],
        passed: 0,
        failed: 0,
        warned: 0,
        errorMessage: message,
      );

  factory DoctorResult.parse(String raw) {
    final trimmed = raw.trim();
    if (trimmed.toLowerCase().startsWith('error:')) {
      return DoctorResult.error(trimmed.substring('error:'.length).trim());
    }

    final payload = trimmed.toLowerCase().startsWith('ok:')
        ? trimmed.substring(trimmed.indexOf(':') + 1)
        : trimmed;

    try {
      final decoded = json.decode(payload);
      if (decoded is! Map<String, dynamic>) {
        return DoctorResult.error('invalid doctor payload');
      }

      final checksList = decoded['checks'] as List<dynamic>? ?? [];
      final checks = checksList.map((e) {
        if (e is! Map<String, dynamic>) return null;
        return DiagnosticCheck(
          name: e['name']?.toString() ?? '',
          status: e['status']?.toString() ?? '',
          details: e['details']?.toString(),
          remediation: e['remediation']?.toString(),
        );
      }).whereType<DiagnosticCheck>().toList();

      return DoctorResult(
        checks: checks,
        passed: (decoded['passed'] as num?)?.toInt() ?? 0,
        failed: (decoded['failed'] as num?)?.toInt() ?? 0,
        warned: (decoded['warned'] as num?)?.toInt() ?? 0,
      );
    } catch (e) {
      return DoctorResult.error('$e');
    }
  }

  final List<DiagnosticCheck> checks;
  final int passed;
  final int failed;
  final int warned;
  final String? errorMessage;

  bool get hasError => errorMessage != null;
}

/// Discovery start result.
class DiscoveryStartResult {
  const DiscoveryStartResult({
    required this.success,
    this.totalKeys,
    this.errorMessage,
  });

  factory DiscoveryStartResult.error(String message) => DiscoveryStartResult(
        success: false,
        errorMessage: message,
      );

  factory DiscoveryStartResult.parse(String raw) {
    final trimmed = raw.trim();
    if (trimmed.toLowerCase().startsWith('error:')) {
      return DiscoveryStartResult.error(
          trimmed.substring('error:'.length).trim());
    }

    final payload = trimmed.toLowerCase().startsWith('ok:')
        ? trimmed.substring(trimmed.indexOf(':') + 1)
        : trimmed;

    try {
      final decoded = json.decode(payload);
      if (decoded is! Map<String, dynamic>) {
        return DiscoveryStartResult.error('invalid discovery payload');
      }

      return DiscoveryStartResult(
        success: decoded['success'] as bool? ?? false,
        totalKeys: (decoded['totalKeys'] as num?)?.toInt(),
        errorMessage: decoded['error']?.toString(),
      );
    } catch (e) {
      return DiscoveryStartResult.error('$e');
    }
  }

  final bool success;
  final int? totalKeys;
  final String? errorMessage;

  bool get hasError => errorMessage != null || !success;
}

/// Recording start result.
class RecordingStartResult {
  const RecordingStartResult({
    required this.success,
    this.outputPath,
    this.errorMessage,
  });

  factory RecordingStartResult.error(String message) => RecordingStartResult(
        success: false,
        errorMessage: message,
      );

  factory RecordingStartResult.parse(String raw) {
    final trimmed = raw.trim();
    if (trimmed.toLowerCase().startsWith('error:')) {
      return RecordingStartResult.error(
          trimmed.substring('error:'.length).trim());
    }

    final payload = trimmed.toLowerCase().startsWith('ok:')
        ? trimmed.substring(trimmed.indexOf(':') + 1)
        : trimmed;

    try {
      final decoded = json.decode(payload);
      if (decoded is! Map<String, dynamic>) {
        return RecordingStartResult.error('invalid recording payload');
      }

      return RecordingStartResult(
        success: decoded['success'] as bool? ?? false,
        outputPath: decoded['outputPath']?.toString(),
        errorMessage: decoded['error']?.toString(),
      );
    } catch (e) {
      return RecordingStartResult.error('$e');
    }
  }

  final bool success;
  final String? outputPath;
  final String? errorMessage;

  bool get hasError => errorMessage != null || !success;
}

/// Recording stop result.
class RecordingStopResult {
  const RecordingStopResult({
    required this.success,
    this.path,
    required this.eventCount,
    required this.durationMs,
    this.errorMessage,
  });

  factory RecordingStopResult.error(String message) => RecordingStopResult(
        success: false,
        eventCount: 0,
        durationMs: 0,
        errorMessage: message,
      );

  factory RecordingStopResult.parse(String raw) {
    final trimmed = raw.trim();
    if (trimmed.toLowerCase().startsWith('error:')) {
      return RecordingStopResult.error(
          trimmed.substring('error:'.length).trim());
    }

    final payload = trimmed.toLowerCase().startsWith('ok:')
        ? trimmed.substring(trimmed.indexOf(':') + 1)
        : trimmed;

    try {
      final decoded = json.decode(payload);
      if (decoded is! Map<String, dynamic>) {
        return RecordingStopResult.error('invalid recording stop payload');
      }

      return RecordingStopResult(
        success: decoded['success'] as bool? ?? false,
        path: decoded['path']?.toString(),
        eventCount: (decoded['eventCount'] as num?)?.toInt() ?? 0,
        durationMs: (decoded['durationMs'] as num?)?.toDouble() ?? 0,
        errorMessage: decoded['error']?.toString(),
      );
    } catch (e) {
      return RecordingStopResult.error('$e');
    }
  }

  final bool success;
  final String? path;
  final int eventCount;
  final double durationMs;
  final String? errorMessage;

  bool get hasError => errorMessage != null || !success;
}
