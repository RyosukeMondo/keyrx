/// Device profile management service.
///
/// Provides access to device profiles containing row-column mappings
/// and physical key layouts.
library;

import '../ffi/bridge.dart';

/// Result of a device profile lookup operation.
class DeviceProfileLookupResult {
  const DeviceProfileLookupResult({
    this.profile,
    this.errorMessage,
  });

  factory DeviceProfileLookupResult.success(DeviceProfile profile) =>
      DeviceProfileLookupResult(profile: profile);

  factory DeviceProfileLookupResult.error(String message) =>
      DeviceProfileLookupResult(errorMessage: message);

  final DeviceProfile? profile;
  final String? errorMessage;

  bool get hasError => errorMessage != null;
  bool get isSuccess => profile != null && !hasError;
}

/// Abstraction for device profile operations.
abstract class DeviceProfileService {
  /// Get device profile for a specific device.
  ///
  /// Returns the complete device profile including keymap and layout
  /// configuration if a profile exists for this device.
  Future<DeviceProfileLookupResult> getProfile(int vendorId, int productId);

  /// Check if a device profile exists.
  ///
  /// Returns true if a profile has been created for this device through
  /// the discovery process.
  Future<bool> hasProfile(int vendorId, int productId);

  /// Dispose any held resources.
  Future<void> dispose();
}

/// Real DeviceProfileService that wraps the KeyrxBridge.
class DeviceProfileServiceImpl implements DeviceProfileService {
  DeviceProfileServiceImpl({required KeyrxBridge bridge}) : _bridge = bridge;

  final KeyrxBridge _bridge;
  final Map<String, DeviceProfile> _cache = {};

  @override
  Future<DeviceProfileLookupResult> getProfile(
      int vendorId, int productId) async {
    if (_bridge.loadFailure != null) {
      return DeviceProfileLookupResult.error(
        'Engine unavailable: ${_bridge.loadFailure}',
      );
    }

    // Check cache first
    final cacheKey = _getCacheKey(vendorId, productId);
    if (_cache.containsKey(cacheKey)) {
      return DeviceProfileLookupResult.success(_cache[cacheKey]!);
    }

    final result = _bridge.getDeviceProfile(vendorId, productId);

    if (result.hasError) {
      return DeviceProfileLookupResult.error(
        result.errorMessage ?? 'Unknown error',
      );
    }

    if (result.profile != null) {
      // Cache the profile
      _cache[cacheKey] = result.profile!;
      return DeviceProfileLookupResult.success(result.profile!);
    }

    return DeviceProfileLookupResult.error('No profile found');
  }

  @override
  Future<bool> hasProfile(int vendorId, int productId) async {
    if (_bridge.loadFailure != null) {
      return false;
    }

    // Check cache first
    final cacheKey = _getCacheKey(vendorId, productId);
    if (_cache.containsKey(cacheKey)) {
      return true;
    }

    return _bridge.hasDeviceProfile(vendorId, productId);
  }

  @override
  Future<void> dispose() async {
    _cache.clear();
  }

  String _getCacheKey(int vendorId, int productId) {
    return '${vendorId.toRadixString(16).padLeft(4, '0')}:${productId.toRadixString(16).padLeft(4, '0')}';
  }

  /// Clear the profile cache.
  ///
  /// Useful when a new profile is discovered and the cache needs to be invalidated.
  void clearCache() {
    _cache.clear();
  }

  /// Clear cache for a specific device.
  void clearCacheForDevice(int vendorId, int productId) {
    final cacheKey = _getCacheKey(vendorId, productId);
    _cache.remove(cacheKey);
  }
}
