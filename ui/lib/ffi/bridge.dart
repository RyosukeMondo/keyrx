/// High-level bridge to KeyRx Core.
///
/// Provides a Dart-friendly API over the raw FFI bindings.
library;

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
}
