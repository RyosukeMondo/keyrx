/// Device Registry service for revolutionary mapping.
///
/// Provides a high-level async API for device registry operations,
/// wrapping FFI calls with error handling and user-friendly messages.
library;

import '../ffi/bridge.dart';
import '../models/device_state.dart';

/// Result of a device registry operation.
class DeviceRegistryOperationResult {
  const DeviceRegistryOperationResult({
    required this.success,
    this.errorMessage,
  });

  factory DeviceRegistryOperationResult.success() =>
      const DeviceRegistryOperationResult(success: true);

  factory DeviceRegistryOperationResult.error(String message) =>
      DeviceRegistryOperationResult(success: false, errorMessage: message);

  final bool success;
  final String? errorMessage;
}

/// Abstraction for device registry operations.
abstract class DeviceRegistryService {
  /// Get all registered devices.
  ///
  /// Returns a list of DeviceState objects representing all connected
  /// devices that have been registered with the engine.
  Future<List<DeviceState>> getDevices();

  /// Toggle remap enabled state for a device.
  ///
  /// [deviceKey] - Device identity key (format: "VID:PID:SERIAL")
  /// [enabled] - Whether remapping should be enabled
  ///
  /// Returns a result indicating success or failure with error details.
  Future<DeviceRegistryOperationResult> toggleRemap(
    String deviceKey,
    bool enabled,
  );

  /// Assign a profile to a device.
  ///
  /// [deviceKey] - Device identity key (format: "VID:PID:SERIAL")
  /// [profileId] - Profile ID to assign
  ///
  /// Returns a result indicating success or failure with error details.
  Future<DeviceRegistryOperationResult> assignProfile(
    String deviceKey,
    String profileId,
  );

  /// Set user label for a device.
  ///
  /// [deviceKey] - Device identity key (format: "VID:PID:SERIAL")
  /// [label] - Optional user label (null to clear)
  ///
  /// Returns a result indicating success or failure with error details.
  Future<DeviceRegistryOperationResult> setUserLabel(
    String deviceKey,
    String? label,
  );

  /// Refresh the device list.
  ///
  /// Forces a refresh of the device list from the engine.
  Future<List<DeviceState>> refresh();

  /// Dispose any held resources.
  Future<void> dispose();
}

/// Real DeviceRegistryService that wraps the KeyrxBridge.
class DeviceRegistryServiceImpl implements DeviceRegistryService {
  DeviceRegistryServiceImpl({required KeyrxBridge bridge}) : _bridge = bridge;

  final KeyrxBridge _bridge;
  List<DeviceState>? _cachedDevices;

  @override
  Future<List<DeviceState>> getDevices() async {
    if (_cachedDevices != null) {
      return _cachedDevices!;
    }
    return refresh();
  }

  @override
  Future<DeviceRegistryOperationResult> toggleRemap(
    String deviceKey,
    bool enabled,
  ) async {
    if (_bridge.loadFailure != null) {
      return DeviceRegistryOperationResult.error(
        'Engine unavailable: ${_bridge.loadFailure}',
      );
    }

    final result = _bridge.setRemapEnabled(deviceKey, enabled);

    if (result.hasError) {
      return DeviceRegistryOperationResult.error(
        _makeUserFriendly(result.errorMessage!, 'toggle remap'),
      );
    }

    // Invalidate cache to force refresh
    _cachedDevices = null;

    return DeviceRegistryOperationResult.success();
  }

  @override
  Future<DeviceRegistryOperationResult> assignProfile(
    String deviceKey,
    String profileId,
  ) async {
    if (_bridge.loadFailure != null) {
      return DeviceRegistryOperationResult.error(
        'Engine unavailable: ${_bridge.loadFailure}',
      );
    }

    final result = _bridge.assignProfile(deviceKey, profileId);

    if (result.hasError) {
      return DeviceRegistryOperationResult.error(
        _makeUserFriendly(result.errorMessage!, 'assign profile'),
      );
    }

    // Invalidate cache to force refresh
    _cachedDevices = null;

    return DeviceRegistryOperationResult.success();
  }

  @override
  Future<DeviceRegistryOperationResult> setUserLabel(
    String deviceKey,
    String? label,
  ) async {
    if (_bridge.loadFailure != null) {
      return DeviceRegistryOperationResult.error(
        'Engine unavailable: ${_bridge.loadFailure}',
      );
    }

    final result = _bridge.setUserLabel(deviceKey, label);

    if (result.hasError) {
      return DeviceRegistryOperationResult.error(
        _makeUserFriendly(result.errorMessage!, 'set label'),
      );
    }

    // Invalidate cache to force refresh
    _cachedDevices = null;

    return DeviceRegistryOperationResult.success();
  }

  @override
  Future<List<DeviceState>> refresh() async {
    if (_bridge.loadFailure != null) {
      return const [];
    }

    final result = _bridge.listDevices();

    if (result.hasError) {
      _cachedDevices = const [];
      return const [];
    }

    _cachedDevices = result.data ?? const [];
    return _cachedDevices!;
  }

  @override
  Future<void> dispose() async {
    _cachedDevices = null;
  }

  /// Convert technical error messages to user-friendly messages.
  String _makeUserFriendly(String technicalError, String operation) {
    // Remove technical prefixes
    final cleaned = technicalError
        .replaceFirst('error:', '')
        .replaceFirst(RegExp(r'^\w+Exception:'), '')
        .trim();

    // Handle common error patterns
    if (cleaned.toLowerCase().contains('device not found')) {
      return 'Device not found. It may have been disconnected.';
    }

    if (cleaned.toLowerCase().contains('profile not found')) {
      return 'Profile not found. Please select a valid profile.';
    }

    if (cleaned.toLowerCase().contains('invalid device key')) {
      return 'Invalid device identifier. Please try refreshing the device list.';
    }

    if (cleaned.toLowerCase().contains('json')) {
      return 'Failed to $operation due to a data format error. Please try again.';
    }

    if (cleaned.toLowerCase().contains('null')) {
      return 'Failed to $operation. The operation returned no response.';
    }

    // If we can't map it, return a generic but helpful message
    return 'Failed to $operation: $cleaned';
  }
}
