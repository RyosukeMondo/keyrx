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
  static KeyrxBridge? _instance;
  late final KeyrxBindings _bindings;
  bool _initialized = false;

  KeyrxBridge._() {
    final lib = _loadLibrary();
    _bindings = KeyrxBindings(lib);
    _setupClassificationStream();
  }

  /// Get the singleton instance.
  static KeyrxBridge get instance {
    _instance ??= KeyrxBridge._();
    return _instance!;
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

    final result = _bindings.init();
    _initialized = result == 0;
    return _initialized;
  }

  /// Get the core library version.
  String get version {
    final ptr = _bindings.version();
    return ptr.cast<Utf8>().toDartString();
  }

  /// Subscribe to classification events if the native layer exposes them.
  Stream<BridgeClassification>? get classificationStream =>
      _classificationController?.stream;

  /// Start audio capture/processing via the native bridge.
  ///
  /// Returns `true` when the bridge reports success.
  Future<bool> startAudio({required int bpm}) async {
    final start = _bindings.startAudio;
    if (start == null) {
      return false;
    }
    return start(bpm) == 0;
  }

  /// Stop audio capture/processing via the native bridge.
  Future<bool> stopAudio() async {
    final stop = _bindings.stopAudio;
    if (stop == null) {
      return false;
    }
    return stop() == 0;
  }

  /// Update BPM on the native engine.
  Future<bool> setBpm(int bpm) async {
    final setter = _bindings.setBpm;
    if (setter == null) {
      return false;
    }
    return setter(bpm) == 0;
  }

  /// Load a Rhai script file.
  bool loadScript(String path) {
    final pathPtr = path.toNativeUtf8();
    try {
      final result = _bindings.loadScript(pathPtr);
      return result == 0;
    } finally {
      calloc.free(pathPtr);
    }
  }

  /// Check if the engine is initialized.
  bool get isInitialized => _initialized;

  StreamController<BridgeClassification>? _classificationController;

  void _setupClassificationStream() {
    if (_bindings.onClassification == null) {
      return;
    }

    _classificationController ??=
        StreamController<BridgeClassification>.broadcast();

    _bindings.onClassification!(
      Pointer.fromFunction<KeyrxClassificationCallbackNative>(
        _handleClassification,
      ),
    );
  }

  static void _handleClassification(Pointer<Uint8> ptr, int length) {
    final instance = _instance;
    final controller = instance?._classificationController;
    if (instance == null || controller == null) {
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
