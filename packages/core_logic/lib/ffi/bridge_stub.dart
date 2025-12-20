/// Stub implementation of KeyrxBridge for unsupported platforms (web).
///
/// This stub provides the same interface as the real KeyrxBridge but throws
/// UnsupportedError for all operations. It's used on web platform where FFI
/// is not available. Web applications should use ServiceRegistry.real() which
/// automatically creates HTTP-based service implementations instead of FFI.
library;

import '../models/bridge_models.dart';
import '../models/models.dart';

// Re-export all models and types so code using conditional imports can access them
export '../models/bridge_models.dart';
export '../models/models.dart';

// Re-export types from type-only files (no FFI dependencies)
export 'bridge_session_types.dart';
export 'bridge_testing_types.dart';
export 'event_types.dart';

/// Stub bridge that throws UnsupportedError for all operations.
/// This is used on web platform where FFI is not available.
class KeyrxBridge {
  KeyrxBridge._();

  static KeyrxBridge open() {
    throw UnsupportedError(
      'KeyrxBridge.open() is not supported on web platform. '
      'Use ServiceRegistry.real() which automatically creates web-compatible services.',
    );
  }

  dynamic get bindings => throw UnsupportedError('FFI not available on web');
  dynamic get runtimeBindings => throw UnsupportedError('FFI not available on web');
  String? get loadFailure => null;
  bool get isInitialized => false;
  String get version => 'stub-0.0.0';
  dynamic get stateStream => throw UnsupportedError('FFI not available on web');

  bool initialize() {
    throw UnsupportedError('FFI not available on web');
  }

  bool checkMigrationNeeded(String oldPath) {
    throw UnsupportedError('FFI not available on web');
  }

  Future<MigrationReport> runMigration(String oldPath, String newPath) {
    throw UnsupportedError('FFI not available on web');
  }

  // Device profile methods
  DeviceProfileResult getDeviceProfile(int vendorId, int productId) {
    throw UnsupportedError('FFI not available on web');
  }

  bool hasDeviceProfile(int vendorId, int productId) {
    throw UnsupportedError('FFI not available on web');
  }

  bool saveDeviceProfile(String profileJson) {
    throw UnsupportedError('FFI not available on web');
  }

  // Device discovery methods
  int selectDevice(String path) {
    throw UnsupportedError('FFI not available on web');
  }

  // Lifecycle methods
  Future<void> dispose() async {
    // No-op on web
  }

  // Testing methods
  dynamic discoverTests(String scriptPath) {
    throw UnsupportedError('FFI not available on web');
  }

  dynamic runTests(String scriptPath, {String? filter}) {
    throw UnsupportedError('FFI not available on web');
  }

  // Event callback methods
  bool registerEventCallback(dynamic eventType, dynamic handler) {
    throw UnsupportedError('FFI not available on web');
  }

  bool unregisterEventCallback(dynamic eventType) {
    throw UnsupportedError('FFI not available on web');
  }

  // Validation methods
  dynamic validateScript(String content) {
    throw UnsupportedError('FFI not available on web');
  }

  // Config methods
  dynamic getConfigRoot() {
    throw UnsupportedError('FFI not available on web');
  }

  // Profile methods
  dynamic getProfile(String profileId) {
    throw UnsupportedError('FFI not available on web');
  }

  dynamic saveProfile(dynamic profile) {
    throw UnsupportedError('FFI not available on web');
  }

  dynamic deleteProfile(String profileId) {
    throw UnsupportedError('FFI not available on web');
  }

  dynamic findCompatibleProfiles(dynamic layoutType) {
    throw UnsupportedError('FFI not available on web');
  }

  dynamic listProfiles() {
    throw UnsupportedError('FFI not available on web');
  }

  // Device registry methods
  dynamic setDeviceEnabled(String deviceKey, bool enabled) {
    throw UnsupportedError('FFI not available on web');
  }

  dynamic setRemapEnabled(String deviceKey, bool enabled) {
    throw UnsupportedError('FFI not available on web');
  }

  dynamic assignProfile(String deviceKey, String profileId) {
    throw UnsupportedError('FFI not available on web');
  }

  dynamic setUserLabel(String deviceKey, String? label) {
    throw UnsupportedError('FFI not available on web');
  }

  dynamic listRegisteredDevices() {
    throw UnsupportedError('FFI not available on web');
  }

  // Engine methods
  dynamic loadScript(String path) {
    throw UnsupportedError('FFI not available on web');
  }

  dynamic eval(String command) {
    throw UnsupportedError('FFI not available on web');
  }

  dynamic listKeys() {
    throw UnsupportedError('FFI not available on web');
  }

  bool startEngineLoop() {
    throw UnsupportedError('FFI not available on web');
  }

  bool stopEngineLoop() {
    throw UnsupportedError('FFI not available on web');
  }

  void requestFullStateResubscribe() {
    throw UnsupportedError('FFI not available on web');
  }

  // Stub needs to provide all methods from mixins but they all throw
  // For simplicity, we'll add methods as needed when compilation fails
  @override
  dynamic noSuchMethod(Invocation invocation) {
    throw UnsupportedError(
      'FFI method ${invocation.memberName} not available on web platform. '
      'Use ServiceRegistry.real() for web-compatible services.',
    );
  }
}
