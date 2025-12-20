/// Stub implementation of KeyrxBridge for unsupported platforms (web).
library;

import '../models/models.dart';
import '../services/config_result.dart';

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
  String? get loadFailure => null;

  bool initialize() {
    throw UnsupportedError('FFI not available on web');
  }

  bool checkMigrationNeeded(String oldPath) {
    throw UnsupportedError('FFI not available on web');
  }

  Future<MigrationReport> runMigration(String oldPath, String newPath) {
    throw UnsupportedError('FFI not available on web');
  }

  DeviceProfile? getDeviceProfile(String profileId) {
    throw UnsupportedError('FFI not available on web');
  }

  List<DeviceProfile> listDeviceProfiles() {
    throw UnsupportedError('FFI not available on web');
  }

  ConfigResult<DeviceProfile> createDeviceProfile(DeviceProfile profile) {
    throw UnsupportedError('FFI not available on web');
  }

  ConfigResult<DeviceProfile> updateDeviceProfile(DeviceProfile profile) {
    throw UnsupportedError('FFI not available on web');
  }

  ConfigResult<void> deleteDeviceProfile(String profileId) {
    throw UnsupportedError('FFI not available on web');
  }
}
