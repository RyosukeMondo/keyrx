/// Device management service for listing and selecting keyboard devices.
///
/// Keeps FFI concerns out of widgets by routing through this service.
library;

import '../ffi/bridge.dart';

/// Result of a device selection operation.
class DeviceSelectionResult {
  const DeviceSelectionResult({
    required this.success,
    this.errorMessage,
  });

  factory DeviceSelectionResult.success() =>
      const DeviceSelectionResult(success: true);

  factory DeviceSelectionResult.error(String message) =>
      DeviceSelectionResult(success: false, errorMessage: message);

  final bool success;
  final String? errorMessage;
}

/// Abstraction for device management operations.
abstract class DeviceService {
  /// List available keyboard devices.
  Future<List<KeyboardDevice>> listDevices();

  /// Select a device by its path for the engine to use.
  ///
  /// Returns a result indicating success or failure with error details.
  Future<DeviceSelectionResult> selectDevice(String path);

  /// Check if a device has an associated profile.
  Future<bool> hasProfile(String deviceId);

  /// Refresh the device list (re-scan for devices).
  Future<List<KeyboardDevice>> refresh();

  /// Dispose any held resources.
  Future<void> dispose();
}

/// Real DeviceService that wraps the KeyrxBridge.
class DeviceServiceImpl implements DeviceService {
  DeviceServiceImpl({required KeyrxBridge bridge}) : _bridge = bridge;

  final KeyrxBridge _bridge;
  List<KeyboardDevice>? _cachedDevices;

  @override
  Future<List<KeyboardDevice>> listDevices() async {
    if (_cachedDevices != null) {
      return _cachedDevices!;
    }
    return refresh();
  }

  @override
  Future<DeviceSelectionResult> selectDevice(String path) async {
    if (_bridge.loadFailure != null) {
      return DeviceSelectionResult.error(
        'Engine unavailable: ${_bridge.loadFailure}',
      );
    }

    final result = _bridge.selectDevice(path);

    return switch (result) {
      0 => DeviceSelectionResult.success(),
      -1 => DeviceSelectionResult.error('Null pointer'),
      -2 => DeviceSelectionResult.error('Invalid UTF-8 path'),
      -3 => DeviceSelectionResult.error('Device path does not exist'),
      -4 => DeviceSelectionResult.error('Device lock error'),
      _ => DeviceSelectionResult.error('Unknown error: $result'),
    };
  }

  @override
  Future<bool> hasProfile(String deviceId) async {
    final devices = await listDevices();
    final device = devices.where((d) {
      final id = '${d.vendorId.toRadixString(16)}:${d.productId.toRadixString(16)}';
      return id == deviceId || d.path == deviceId;
    }).firstOrNull;

    return device?.hasProfile ?? false;
  }

  @override
  Future<List<KeyboardDevice>> refresh() async {
    if (_bridge.loadFailure != null) {
      return const [];
    }

    final result = _bridge.listDevices();
    if (result.hasError) {
      _cachedDevices = const [];
      return const [];
    }

    _cachedDevices = result.devices;
    return result.devices;
  }

  @override
  Future<void> dispose() async {
    _cachedDevices = null;
  }
}
