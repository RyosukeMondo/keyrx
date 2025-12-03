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
