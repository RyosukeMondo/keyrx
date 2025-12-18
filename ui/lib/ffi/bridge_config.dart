// Config domain FFI methods.
///
/// Provides configuration path resolution via FFI.
library;

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
    if (bindings == null) {
      return ConfigPathResult.error('getConfigRoot not available');
    }

    try {
      final raw = bindings!.config.getConfigRoot();
      return ConfigPathResult.success(raw);
    } catch (e) {
      return ConfigPathResult.error('$e');
    }
  }
}
