import 'dart:async';
import 'dart:typed_data';
import '../ffi/bridge.dart';
import '../ffi/device_registry_ffi.dart';
import '../models/device_config.dart';
import '../models/device_identity.dart';
import '../models/device_state.dart';
import '../repositories/device_config_repository.dart';

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
  Future<DeviceRegistryResult<void>> toggleDevice(
    String deviceKey,
    bool enabled,
  );

  /// Toggle remap enabled state for a device.
  Future<DeviceRegistryResult<void>> toggleRemap(
    String deviceKey,
    bool enabled,
  );

  /// Assign a profile to a device.
  Future<DeviceRegistryResult<void>> assignProfile(
    String deviceKey,
    String profileId,
  );

  /// Set user label for a device.
  Future<DeviceRegistryResult<void>> setUserLabel(
    String deviceKey,
    String? label,
  );

  /// Refresh the device list.
  Future<List<DeviceState>> refresh();

  /// Stream of device list updates.
  Stream<List<DeviceState>> get devicesStream;

  /// Migrate virtual devices from old storage (e.g., SharedPreferences).
  Future<void> migrateVirtualDevices(List<DeviceIdentity> identities);

  /// Dispose any held resources.
  Future<void> dispose();
}

/// Implementation of [DeviceRegistryService] that uses [KeyrxBridge] and [DeviceConfigRepository].
class DeviceRegistryServiceImpl implements DeviceRegistryService {
  /// Create a new [DeviceRegistryServiceImpl].
  DeviceRegistryServiceImpl({
    required KeyrxBridge bridge,
    required DeviceConfigRepository repository,
  }) : _bridge = bridge,
       _repository = repository {
    _bridge.registerEventCallback(
      EventType.deviceConnected,
      _handleDeviceEvent,
    );
    _bridge.registerEventCallback(
      EventType.deviceDisconnected,
      _handleDeviceEvent,
    );
    _bridge.registerEventCallback(EventType.deviceUpdated, _handleDeviceEvent);
  }

  final KeyrxBridge _bridge;
  final DeviceConfigRepository _repository;
  List<DeviceState>? _cachedDevices;
  final _devicesController = StreamController<List<DeviceState>>.broadcast();

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
    final config = await _repository.load();
    final key = identity.toKey();

    final updatedDevices = List<DeviceIdentity>.from(config.devices);
    final index = updatedDevices.indexWhere((d) => d.toKey() == key);

    if (index >= 0) {
      updatedDevices[index] = identity;
    } else {
      updatedDevices.add(identity);
    }

    await _repository.save(config.copyWith(devices: updatedDevices));
    await refresh();
  }

  @override
  Future<void> removeVirtualDevice(String key) async {
    final config = await _repository.load();
    final updatedDevices =
        config.devices.where((d) => d.toKey() != key).toList();

    await _repository.save(config.copyWith(devices: updatedDevices));
    await refresh();
  }

  @override
  Future<DeviceRegistryResult<void>> toggleDevice(
    String deviceKey,
    bool enabled,
  ) async {
    if (_bridge.loadFailure != null) {
      return DeviceRegistryResult.error(
        'Engine unavailable: ${_bridge.loadFailure}',
      );
    }

    final result = _bridge.setDeviceEnabled(deviceKey, enabled);
    if (result.hasError) {
      return DeviceRegistryResult.error(
        _makeUserFriendly(result.errorMessage!, 'toggle device'),
      );
    }

    await refresh();
    return DeviceRegistryResult.success(null);
  }

  @override
  Future<DeviceRegistryResult<void>> toggleRemap(
    String deviceKey,
    bool enabled,
  ) async {
    if (_bridge.loadFailure != null) {
      return DeviceRegistryResult.error(
        'Engine unavailable: ${_bridge.loadFailure}',
      );
    }

    final result = _bridge.setRemapEnabled(deviceKey, enabled);
    if (result.hasError) {
      return DeviceRegistryResult.error(
        _makeUserFriendly(result.errorMessage!, 'toggle remap'),
      );
    }

    await refresh();
    return DeviceRegistryResult.success(null);
  }

  @override
  Future<DeviceRegistryResult<void>> assignProfile(
    String deviceKey,
    String profileId,
  ) async {
    if (_bridge.loadFailure != null) {
      return DeviceRegistryResult.error(
        'Engine unavailable: ${_bridge.loadFailure}',
      );
    }

    final result = _bridge.assignProfile(deviceKey, profileId);
    if (result.hasError) {
      return DeviceRegistryResult.error(
        _makeUserFriendly(result.errorMessage!, 'assign profile'),
      );
    }

    await refresh();
    return DeviceRegistryResult.success(null);
  }

  @override
  Future<DeviceRegistryResult<void>> setUserLabel(
    String deviceKey,
    String? label,
  ) async {
    // 1. Update config repository (SSOT for labels and virtual devices)
    final config = await _repository.load();
    final updatedDevices = List<DeviceIdentity>.from(config.devices);

    final index = updatedDevices.indexWhere((d) => d.toKey() == deviceKey);
    if (index >= 0) {
      updatedDevices[index] = updatedDevices[index].copyWith(userLabel: label);
    } else {
      // If not in config, we add it now so the label is persisted even for real devices
      final devices = await getDevices();
      final found = devices.where((d) => d.identity.toKey() == deviceKey);
      if (found.isNotEmpty) {
        updatedDevices.add(found.first.identity.copyWith(userLabel: label));
      }
    }

    await _repository.save(config.copyWith(devices: updatedDevices));

    // 2. Update FFI if available
    if (_bridge.loadFailure == null) {
      final ffiResult = _bridge.setUserLabel(deviceKey, label);
      if (ffiResult.hasError) {
        return DeviceRegistryResult.error(
          _makeUserFriendly(ffiResult.errorMessage!, 'set label'),
        );
      }
    }

    await refresh();
    return DeviceRegistryResult.success(null);
  }

  @override
  Future<List<DeviceState>> refresh() async {
    // 1. Load from FFI
    var ffiDevices = <DeviceState>[];
    if (_bridge.loadFailure == null) {
      final result = _bridge.listRegisteredDevices();
      if (result.isSuccess) {
        ffiDevices = result.data!;
      }
    }

    // 2. Load from Repository
    final config = await _repository.load();
    final knownDevices = config.devices;

    // 3. Merge
    final allDevicesMap = <String, DeviceState>{};
    final now = DateTime.now().toUtc().toIso8601String();

    // Add all known devices from config initially
    for (final identity in knownDevices) {
      allDevicesMap[identity.toKey()] = DeviceState(
        identity: identity,
        deviceEnabled: false,
        remapEnabled: false,
        connectedAt: '',
        updatedAt: now,
      );
    }

    // Overwrite with real devices from FFI, while preserving local labels
    for (final ffiDevice in ffiDevices) {
      final key = ffiDevice.identity.toKey();
      final known = allDevicesMap[key];

      var identity = ffiDevice.identity;
      // If we have a local label in config that FFI doesn't have, prefer local
      if (known != null &&
          (identity.userLabel == null || identity.userLabel!.isEmpty)) {
        identity = identity.copyWith(userLabel: known.identity.userLabel);
      }

      allDevicesMap[key] = ffiDevice.copyWith(identity: identity);
    }

    final allDevices = allDevicesMap.values.toList();
    // Sort by connected status then by key
    allDevices.sort((a, b) {
      final aConnected = a.connectedAt.isNotEmpty;
      final bConnected = b.connectedAt.isNotEmpty;
      if (aConnected && !bConnected) return -1;
      if (!aConnected && bConnected) return 1;
      return a.identity.toKey().compareTo(b.identity.toKey());
    });

    _cachedDevices = allDevices;
    _devicesController.add(_cachedDevices!);
    return _cachedDevices!;
  }

  @override
  Future<void> migrateVirtualDevices(List<DeviceIdentity> identities) async {
    if (identities.isEmpty) return;
    
    final config = await _repository.load();
    final existingKeys = config.devices.map((d) => d.toKey()).toSet();

    final toAdd =
        identities.where((id) => !existingKeys.contains(id.toKey())).toList();
    if (toAdd.isNotEmpty) {
      final updated = [...config.devices, ...toAdd];
      await _repository.save(config.copyWith(devices: updated));
      await refresh();
    }
  }

  @override
  Future<void> dispose() async {
    _bridge.unregisterEventCallback(EventType.deviceConnected);
    _bridge.unregisterEventCallback(EventType.deviceDisconnected);
    _bridge.unregisterEventCallback(EventType.deviceUpdated);
    await _devicesController.close();
  }

  void _handleDeviceEvent(Uint8List payload) {
    refresh();
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
