/// Device Registry service for revolutionary mapping.
///
/// Provides a high-level async API for device registry operations,
/// wrapping FFI calls with error handling and user-friendly messages.
library;

import 'dart:async';
import 'dart:convert';

import 'package:shared_preferences/shared_preferences.dart';

import '../ffi/bridge.dart';
import '../models/device_state.dart';
import '../models/device_identity.dart';

/// Result of a device registry operation.
class DeviceRegistryOperationResult {
  const DeviceRegistryOperationResult.success()
    : success = true,
      errorMessage = null;

  const DeviceRegistryOperationResult.error(this.errorMessage)
    : success = false;

  final bool success;
  final String? errorMessage;
}

/// Abstraction for device registry operations.
abstract class DeviceRegistryService {
  /// Get all registered devices (connected + virtual).
  Future<List<DeviceState>> getDevices();

  /// Register a local virtual device.
  Future<void> addVirtualDevice(DeviceIdentity identity);

  /// Unregister a local virtual device.
  Future<void> removeVirtualDevice(String key);

  /// Toggle remap enabled state for a device.
  Future<DeviceRegistryOperationResult> toggleRemap(
    String deviceKey,
    bool enabled,
  );

  /// Assign a profile to a device.
  Future<DeviceRegistryOperationResult> assignProfile(
    String deviceKey,
    String profileId,
  );

  /// Set user label for a device.
  Future<DeviceRegistryOperationResult> setUserLabel(
    String deviceKey,
    String? label,
  );

  /// Refresh the device list.
  Future<List<DeviceState>> refresh();

  /// Stream of device list updates.
  Stream<List<DeviceState>> get devicesStream;

  /// Dispose any held resources.
  Future<void> dispose();
}

/// Real DeviceRegistryService that wraps the KeyrxBridge.
class DeviceRegistryServiceImpl implements DeviceRegistryService {
  DeviceRegistryServiceImpl({required KeyrxBridge bridge}) : _bridge = bridge;

  final KeyrxBridge _bridge;
  List<DeviceState>? _cachedDevices;
  final _devicesController = StreamController<List<DeviceState>>.broadcast();

  static const _virtualDevicesKey = 'keyrx_virtual_devices';

  @override
  Stream<List<DeviceState>> get devicesStream => _devicesController.stream;

  @override
  Future<List<DeviceState>> getDevices() async {
    if (_cachedDevices != null) {
      return _cachedDevices!;
    }
    return refresh();
  }

  @override
  Future<void> addVirtualDevice(DeviceIdentity identity) async {
    final prefs = await SharedPreferences.getInstance();
    final List<String> current = prefs.getStringList(_virtualDevicesKey) ?? [];

    // Check if already exists, update if so
    final key = identity.toKey();
    final existingIndex = current.indexWhere((jsonStr) {
      try {
        final existing = DeviceIdentity.fromJson(jsonDecode(jsonStr));
        return existing.toKey() == key;
      } catch (_) {
        return false;
      }
    });

    final jsonStr = jsonEncode(identity.toJson());

    if (existingIndex >= 0) {
      current[existingIndex] = jsonStr;
    } else {
      current.add(jsonStr);
    }

    await prefs.setStringList(_virtualDevicesKey, current);
    await refresh(); // Refresh to include new device
  }

  @override
  Future<void> removeVirtualDevice(String key) async {
    final prefs = await SharedPreferences.getInstance();
    final List<String> current = prefs.getStringList(_virtualDevicesKey) ?? [];

    current.removeWhere((jsonStr) {
      try {
        final identity = DeviceIdentity.fromJson(jsonDecode(jsonStr));
        return identity.toKey() == key;
      } catch (_) {
        return false;
      }
    });

    await prefs.setStringList(_virtualDevicesKey, current);
    await refresh();
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
        _makeUserFriendly(
          result.errorMessage ?? 'Unknown error',
          'toggle remap',
        ),
      );
    }

    // Invalidate cache to force refresh
    _updateCachedDevice(
      deviceKey,
      (device) => device.copyWith(
        remapEnabled: enabled,
        updatedAt: DateTime.now().toUtc().toIso8601String(),
      ),
    );

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
        _makeUserFriendly(
          result.errorMessage ?? 'Unknown error',
          'assign profile',
        ),
      );
    }

    // Invalidate cache to force refresh
    _updateCachedDevice(
      deviceKey,
      (device) => device.copyWith(
        profileId: profileId,
        updatedAt: DateTime.now().toUtc().toIso8601String(),
      ),
    );

    return DeviceRegistryOperationResult.success();
  }

  @override
  Future<DeviceRegistryOperationResult> setUserLabel(
    String deviceKey,
    String? label,
  ) async {
    // Check if it's a virtual device first
    final prefs = await SharedPreferences.getInstance();
    final List<String> virtuals = prefs.getStringList(_virtualDevicesKey) ?? [];

    int virtualIndex = -1;
    DeviceIdentity? virtualIdentity;

    for (int i = 0; i < virtuals.length; i++) {
      try {
        final identity = DeviceIdentity.fromJson(jsonDecode(virtuals[i]));
        if (identity.toKey() == deviceKey) {
          virtualIndex = i;
          virtualIdentity = identity;
          break;
        }
      } catch (_) {}
    }

    if (virtualIndex >= 0 && virtualIdentity != null) {
      // Update local virtual device
      final updated = virtualIdentity.copyWith(userLabel: label);
      virtuals[virtualIndex] = jsonEncode(updated.toJson());
      await prefs.setStringList(_virtualDevicesKey, virtuals);
      await refresh();
      return DeviceRegistryOperationResult.success();
    }

    if (_bridge.loadFailure != null) {
      return DeviceRegistryOperationResult.error(
        'Engine unavailable: ${_bridge.loadFailure}',
      );
    }

    final result = _bridge.setUserLabel(deviceKey, label);

    if (result.hasError) {
      return DeviceRegistryOperationResult.error(
        _makeUserFriendly(result.errorMessage ?? 'Unknown error', 'set label'),
      );
    }

    // Invalidate cache to force refresh
    _updateCachedDevice(
      deviceKey,
      (device) => device.copyWith(
        identity: device.identity.copyWith(userLabel: label),
        updatedAt: DateTime.now().toUtc().toIso8601String(),
      ),
    );

    return DeviceRegistryOperationResult.success();
  }

  @override
  Future<List<DeviceState>> refresh() async {
    if (_bridge.loadFailure != null) {
      // Fallback to minimal behavior? Or just throw.
      // If engine is down, we might still want to show virtual devices?
      // For now, adhere to existing pattern but maybe we can be more robust.
      // Let's try to proceed.
      print(
        'DeviceRegistryService: Engine unavailable. Proceeding with virtual devices only.',
      );
    }

    List<DeviceState> ffiDevices = [];
    if (_bridge.loadFailure == null) {
      // print('DeviceRegistryService: refreshing devices via FFI...');
      final result = _bridge.listRegisteredDevices();

      if (result.hasError) {
        final message = _makeUserFriendly(
          result.errorMessage ?? 'Unknown error',
          'load devices',
        );
        print('DeviceRegistryService: FFI returned error: $message');
        // Don't throw immediately, we want to try loading virtuals
      } else {
        ffiDevices = result.data ?? [];
      }
    }

    // Load virtual devices
    final prefs = await SharedPreferences.getInstance();
    final List<String> virtualsJson =
        prefs.getStringList(_virtualDevicesKey) ?? [];
    final List<DeviceState> virtualDevices = [];

    final now = DateTime.now().toUtc().toIso8601String();

    for (final jsonStr in virtualsJson) {
      try {
        final identity = DeviceIdentity.fromJson(jsonDecode(jsonStr));

        // Should verify if this ID is already in FFI list
        final isConnected = ffiDevices.any(
          (d) => d.identity.toKey() == identity.toKey(),
        );

        if (!isConnected) {
          // Create a placeholder state for the virtual device
          virtualDevices.add(
            DeviceState(
              identity: identity,
              remapEnabled: false, // Default for offline/virtual
              connectedAt: '', // Empty indicates not connected
              updatedAt: now,
              profileId:
                  null, // We don't track virtual assignments here yet? Or could we?
            ),
          );
        }
      } catch (e) {
        print('Error parsing virtual device: $e');
      }
    }

    // Merge: FFI devices take precedence
    // Actually, if an FFI device matches a virtual device, we might want to carry over the 'userLabel' from virtual if FFI doesn't have one?
    // But implementation of setUserLabel handles both. Ideally FFI state is truth.

    final allDevices = [...ffiDevices, ...virtualDevices];

    // print('DeviceRegistryService: fetched ${allDevices.length} devices (${ffiDevices.length} connected, ${virtualDevices.length} virtual)');
    _cachedDevices = allDevices;
    _devicesController.add(_cachedDevices!);
    return _cachedDevices!;
  }

  @override
  Future<void> dispose() async {
    await _devicesController.close();
    _cachedDevices = null;
  }

  /// Convert technical error messages to user-friendly messages.
  String _makeUserFriendly(String technicalError, String operation) {
    // ... (existing implementation)
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

  void _updateCachedDevice(
    String deviceKey,
    DeviceState Function(DeviceState current) update,
  ) {
    if (_cachedDevices == null || _cachedDevices!.isEmpty) {
      return;
    }

    final index = _cachedDevices!.indexWhere(
      (device) => device.identity.toKey() == deviceKey,
    );

    if (index == -1) {
      return;
    }

    final updated = update(_cachedDevices![index]);
    final mutable = List<DeviceState>.from(_cachedDevices!);
    mutable[index] = updated;
    _cachedDevices = mutable;
    _devicesController.add(_cachedDevices!);
  }
}
