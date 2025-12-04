/// Core FFI initialization and lifecycle management.
///
/// Provides initialization, version, and disposal functionality
/// for the KeyRx bridge.
library;

import 'dart:async';
import 'dart:ffi';
import 'dart:io';

import 'package:ffi/ffi.dart';

import 'bindings.dart';

/// Mixin providing core FFI initialization and lifecycle management.
mixin BridgeCoreMixin {
  KeyrxBindings? get bindings;
  bool get initialized;
  set initialized(bool value);
  Object? get loadFailure;

  /// Load the native library based on platform.
  static DynamicLibrary loadLibrary() {
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
    if (initialized) return true;
    if (bindings == null || loadFailure != null) {
      return false;
    }

    final result = bindings!.init();
    initialized = result == 0;
    return initialized;
  }

  /// Get the core library version.
  String get version {
    if (bindings == null) {
      return 'unavailable';
    }

    final ptr = bindings!.version();
    return ptr.cast<Utf8>().toDartString();
  }

  /// Check if the engine is initialized.
  bool get isInitialized => initialized;
}

/// Extension providing stream controller disposal.
extension BridgeCoreDispose on BridgeCoreMixin {
  /// Close stream controllers and reset state.
  Future<void> disposeControllers(
    StreamController<dynamic>? stateController,
  ) async {
    await stateController?.close();
  }
}
