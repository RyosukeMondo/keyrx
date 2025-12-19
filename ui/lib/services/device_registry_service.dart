/// Device Registry service wrapper for UI compatibility.
///
/// Wraps the shared [core.DeviceRegistryService] from packages/core_logic
/// to provide UI-specific features like migration and legacy result types.
library;

import 'dart:async';
import 'dart:convert';

import 'package:core_logic/core_logic.dart' as core;
import 'package:core_logic/models/device_identity.dart';
import 'package:core_logic/models/device_state.dart';
import 'package:shared_preferences/shared_preferences.dart';

/// Result of a device registry operation (UI compatibility).
class DeviceRegistryOperationResult {
  const DeviceRegistryOperationResult({
    required this.success,
    this.errorMessage,
  });

  const DeviceRegistryOperationResult.success()
    : success = true,
      errorMessage = null;

  const DeviceRegistryOperationResult.error(this.errorMessage)
    : success = false;

  /// Create from a core logic result.
  factory DeviceRegistryOperationResult.fromCore(
    core.DeviceRegistryResult<void> result,
  ) => DeviceRegistryOperationResult(
    success: result.isSuccess,
    errorMessage: result.errorMessage,
  );

  final bool success;
  final String? errorMessage;
}

/// Abstraction for device registry operations (UI-facing).
abstract class DeviceRegistryService {
  /// Get all registered devices (connected + virtual).
  Future<List<DeviceState>> getDevices();

  /// Register a local virtual device.
  Future<void> addVirtualDevice(DeviceIdentity identity);

  /// Unregister a local virtual device.
  Future<void> removeVirtualDevice(String key);

  /// Toggle device enabled state.
  Future<DeviceRegistryOperationResult> toggleDevice(
    String deviceKey, {
    required bool enabled,
  });

  /// Toggle remap enabled state for a device.
  Future<DeviceRegistryOperationResult> toggleRemap(
    String deviceKey, {
    required bool enabled,
  });

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

/// Implementation of [DeviceRegistryService] that wraps the core service.
class DeviceRegistryServiceImpl implements DeviceRegistryService {
  DeviceRegistryServiceImpl({required core.DeviceRegistryService coreService})
    : _core = coreService;

  final core.DeviceRegistryService _core;

  @override
  Stream<List<DeviceState>> get devicesStream => _core.devicesStream;

  @override
  Future<List<DeviceState>> getDevices() => _core.getDevices();

  @override
  Future<void> addVirtualDevice(DeviceIdentity identity) =>
      _core.addVirtualDevice(identity);

  @override
  Future<void> removeVirtualDevice(String key) =>
      _core.removeVirtualDevice(key);

  @override
  Future<DeviceRegistryOperationResult> toggleDevice(
    String deviceKey, {
    required bool enabled,
  }) async {
    final result = await _core.toggleDevice(deviceKey, enabled);
    return DeviceRegistryOperationResult.fromCore(result);
  }

  @override
  Future<DeviceRegistryOperationResult> toggleRemap(
    String deviceKey, {
    required bool enabled,
  }) async {
    final result = await _core.toggleRemap(deviceKey, enabled);
    return DeviceRegistryOperationResult.fromCore(result);
  }

  @override
  Future<DeviceRegistryOperationResult> assignProfile(
    String deviceKey,
    String profileId,
  ) async {
    final result = await _core.assignProfile(deviceKey, profileId);
    return DeviceRegistryOperationResult.fromCore(result);
  }

  @override
  Future<DeviceRegistryOperationResult> setUserLabel(
    String deviceKey,
    String? label,
  ) async {
    final result = await _core.setUserLabel(deviceKey, label);
    return DeviceRegistryOperationResult.fromCore(result);
  }

  @override
  Future<List<DeviceState>> refresh() => _core.refresh();

  @override
  Future<void> dispose() => _core.dispose();

  /// Migrate virtual devices from SharedPreferences to the new JSON config.
  static Future<void> migrateFromSharedPreferences(
    core.DeviceRegistryService coreService,
  ) async {
    try {
      final prefs = await SharedPreferences.getInstance();
      const virtualDevicesKey = 'keyrx_virtual_devices';
      final virtualsJson = prefs.getStringList(virtualDevicesKey) ?? [];

      if (virtualsJson.isNotEmpty) {
        final identities = <DeviceIdentity>[];
        for (final jsonStr in virtualsJson) {
          try {
            final jsonMap = jsonDecode(jsonStr) as Map<String, dynamic>;
            identities.add(DeviceIdentity.fromJson(jsonMap));
          } on Exception catch (_) {
            // Skip invalid entries
          }
        }

        if (identities.isNotEmpty) {
          await coreService.migrateVirtualDevices(identities);
        }

        // Clear after migration to avoid repeated work
        await prefs.remove(virtualDevicesKey);
      }
    } on Exception catch (_) {
      // Best effort migration
    }
  }
}
