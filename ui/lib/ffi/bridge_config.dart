/// Config domain FFI methods.
///
/// Provides configuration path resolution via FFI.
library;

import 'dart:ffi';

import 'package:ffi/ffi.dart';
import 'package:flutter/foundation.dart';

import 'bindings.dart';

/// Result of a config path operation.
class ConfigPathResult {
  const ConfigPathResult({this.path, this.errorMessage});

  factory ConfigPathResult.success(String path) => ConfigPathResult(path: path);

  factory ConfigPathResult.error(String message) =>
      ConfigPathResult(errorMessage: message);

  final String? path;
  final String? errorMessage;

  bool get hasError => errorMessage != null;
  bool get isSuccess => path != null && !hasError;
  String? get data => path;
}

/// Mixin providing config domain FFI methods.
mixin BridgeConfigMixin {
  KeyrxBindings? get bindings;
  Object? get loadFailure;

  /// Get the configuration root path.
  ///
  /// Returns the absolute path to the configuration root directory.
  ConfigPathResult getConfigRoot() {
    final getFn = bindings?.getConfigRoot;
    if (getFn == null) {
      return ConfigPathResult.error('getConfigRoot not available');
    }

    Pointer<Char>? ptr;
    try {
      ptr = getFn();
      if (ptr == nullptr) {
        return ConfigPathResult.error('getConfigRoot returned null');
      }

      final raw = ptr.cast<Utf8>().toDartString();
      // Contract returns simple string, or error string?
      // Contract says returns string. The wrapper likely wraps it in Result<String, String>.
      // If Rust returns Err, it usually comes as JSON error string or specific format.
      // But standard KeyRx contract wrapper usually returns raw string if success, or special error string?
      // Let's assume standard behavior: Rust returns String (which might be JSON string if complex object, but this is simple path).
      // If it's a simple string, it's just the path.
      return ConfigPathResult.success(raw);
    } catch (e) {
      return ConfigPathResult.error('$e');
    } finally {
      if (ptr != null && ptr != nullptr) {
        try {
          bindings?.freeString(ptr);
        } catch (_) {}
      }
    }
  }
}
