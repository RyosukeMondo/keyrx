/// Service for managing the device registry.
library;

import 'dart:async';

import '../models/bridge_models.dart';
import '../models/device_identity.dart';
import '../models/device_state.dart';

export 'device_registry_service_ffi.dart'
    if (dart.library.html) 'device_registry_service_web.dart';

/// Service for managing the device registry.
///
/// Unifies real devices (from FFI) and virtual devices (from configuration).
/// This service is shared between the UI and CLI tools.
abstract class DeviceRegistryService {
  /// Get all registered devices (connected + virtual).
  Future<List<DeviceState>> getDevices();

  /// Register a local virtual device.
  Future<void> addVirtualDevice(DeviceIdentity identity);

  /// Unregister a local virtual device.
  Future<void> removeVirtualDevice(String key);

  /// Toggle device enabled state.
  Future<DeviceRegistryResult<void>> toggleDevice(String deviceKey, bool enabled);

  /// Toggle remap enabled state for a device.
  Future<DeviceRegistryResult<void>> toggleRemap(String deviceKey, bool enabled);

  /// Assign a profile to a device.
  Future<DeviceRegistryResult<void>> assignProfile(String deviceKey, String profileId);

  /// Set user label for a device.
  Future<DeviceRegistryResult<void>> setUserLabel(String deviceKey, String? label);

  /// Refresh the device list.
  Future<List<DeviceState>> refresh();

  /// Stream of device list updates.
  Stream<List<DeviceState>> get devicesStream;

  /// Migrate virtual devices from old storage (e.g., SharedPreferences).
  Future<void> migrateVirtualDevices(List<DeviceIdentity> identities);

  /// Dispose any held resources.
  Future<void> dispose();
}
