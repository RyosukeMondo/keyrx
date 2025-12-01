/// Engine-level operations exposed to the UI layer.
///
/// Keeps FFI concerns out of widgets by routing through this service.
library;

/// Abstraction for interacting with the engine core.
abstract class EngineService {
  /// Whether the engine has been initialized.
  bool get isInitialized;

  /// Core library version string.
  String get version;

  /// Initialize the engine and underlying bridge.
  Future<bool> initialize();

  /// Load a script path into the engine.
  Future<bool> loadScript(String path);

  /// Dispose any held resources.
  Future<void> dispose();
}
