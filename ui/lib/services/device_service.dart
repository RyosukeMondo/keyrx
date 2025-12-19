/// Device management service for listing and selecting keyboard devices.
///
/// Keeps FFI concerns out of widgets by routing through this service.
library;

import 'package:core_logic/ffi/bridge.dart';
import 'package:core_logic/utils/logger.dart';

/// Result of a device selection operation.
class DeviceSelectionResult {
  const DeviceSelectionResult({required this.success, this.errorMessage});

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
  final StructuredLogger _log = getLogger('device_service');

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
      final message = 'Engine unavailable: ${_bridge.loadFailure}';
      _log.error(
        'device_select_failed',
        message: message,
        context: {'path': path},
      );
      return DeviceSelectionResult.error(message);
    }

    final result = _bridge.selectDevice(path);

    return switch (result) {
      0 => _onSelectSuccess(path),
      -1 => _onSelectFailure(path, 'Null pointer'),
      -2 => _onSelectFailure(path, 'Invalid UTF-8 path'),
      -3 => _onSelectFailure(path, 'Device path does not exist'),
      -4 => _onSelectFailure(path, 'Device lock error'),
      _ => _onSelectFailure(path, 'Unknown error: $result'),
    };
  }

  @override
  Future<bool> hasProfile(String deviceId) async {
    final devices = await listDevices();
    final device = devices.where((d) {
      final id =
          '${d.vendorId.toRadixString(16)}:${d.productId.toRadixString(16)}';
      return id == deviceId || d.path == deviceId;
    }).firstOrNull;

    return device?.hasProfile ?? false;
  }

  @override
  Future<List<KeyboardDevice>> refresh() async {
    if (_bridge.loadFailure != null) {
      _log.warn(
        'device_refresh_skipped',
        message: 'Bridge failed to load; cannot refresh devices',
        context: {'load_failure': _bridge.loadFailure.toString()},
      );
      return const [];
    }

    // Use the new DeviceRegistry API which uses the Revolutionary Runtime
    final result = _bridge.listRegisteredDevices();
    if (result.hasError) {
      _log.error(
        'device_refresh_failed',
        message: 'Failed to list registered devices',
        context: {'error': result.errorMessage ?? 'unknown'},
      );
      _cachedDevices = const [];
      return const [];
    }

    if (result.data == null) {
      _cachedDevices = const [];
      return const [];
    }

    // Map DeviceState to KeyboardDevice
    final devices = result.data!
        .map(
          (state) => KeyboardDevice(
            name:
                state.identity.userLabel ??
                state.identity.serialNumber, // Fallback name
            vendorId: state.identity.vendorId,
            productId: state.identity.productId,
            path: state.identity.toKey(), // Use unique key as path
            hasProfile: state.profileId != null,
          ),
        )
        .toList();

    _cachedDevices = devices;
    _log.info('device_refresh_completed', context: {'count': devices.length});
    return devices;
  }

  @override
  Future<void> dispose() async {
    _cachedDevices = null;
  }

  DeviceSelectionResult _onSelectSuccess(String path) {
    _log.info('device_selected', context: {'path': path});
    return DeviceSelectionResult.success();
  }

  DeviceSelectionResult _onSelectFailure(String path, String errorMessage) {
    _log.warn(
      'device_selection_error',
      message: errorMessage,
      context: {'path': path},
    );
    return DeviceSelectionResult.error(errorMessage);
  }
}
