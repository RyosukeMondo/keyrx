/// Device discovery FFI methods.
///
/// Provides device discovery and listing functionality for the KeyRx bridge.
library;

import 'dart:convert';
import 'dart:ffi';

import 'package:ffi/ffi.dart';

import 'bindings.dart';

/// Keyboard device information from the FFI layer.
class KeyboardDevice {
  const KeyboardDevice({
    required this.name,
    required this.vendorId,
    required this.productId,
    required this.path,
    required this.hasProfile,
  });

  final String name;
  final int vendorId;
  final int productId;
  final String path;
  final bool hasProfile;
}

/// Result of listing keyboard devices.
class DeviceListResult {
  const DeviceListResult({required this.devices, this.errorMessage});

  factory DeviceListResult.error(String message) =>
      DeviceListResult(devices: const [], errorMessage: message);

  factory DeviceListResult.parse(String raw) {
    final trimmed = raw.trim();
    if (trimmed.toLowerCase().startsWith('error:')) {
      return DeviceListResult.error(trimmed.substring('error:'.length).trim());
    }

    final payload = trimmed.toLowerCase().startsWith('ok:')
        ? trimmed.substring(trimmed.indexOf(':') + 1)
        : trimmed;

    try {
      final decoded = json.decode(payload);
      if (decoded is! List) {
        return DeviceListResult.error('invalid device list payload');
      }

      final devices = decoded
          .map((entry) {
            if (entry is! Map<String, dynamic>) {
              return null;
            }
            return KeyboardDevice(
              name: entry['name']?.toString() ?? 'Unknown',
              vendorId: (entry['vendorId'] as num?)?.toInt() ?? 0,
              productId: (entry['productId'] as num?)?.toInt() ?? 0,
              path: entry['path']?.toString() ?? '',
              hasProfile: entry['hasProfile'] as bool? ?? false,
            );
          })
          .whereType<KeyboardDevice>()
          .toList();

      return DeviceListResult(devices: devices);
    } catch (e) {
      return DeviceListResult.error('$e');
    }
  }

  final List<KeyboardDevice> devices;
  final String? errorMessage;

  bool get hasError => errorMessage != null;
}

/// Discovery start result.
class DiscoveryStartResult {
  const DiscoveryStartResult({
    required this.success,
    this.totalKeys,
    this.errorMessage,
  });

  factory DiscoveryStartResult.error(String message) =>
      DiscoveryStartResult(success: false, errorMessage: message);

  factory DiscoveryStartResult.parse(String raw) {
    final trimmed = raw.trim();
    if (trimmed.toLowerCase().startsWith('error:')) {
      return DiscoveryStartResult.error(
        trimmed.substring('error:'.length).trim(),
      );
    }

    final payload = trimmed.toLowerCase().startsWith('ok:')
        ? trimmed.substring(trimmed.indexOf(':') + 1)
        : trimmed;

    try {
      final decoded = json.decode(payload);
      if (decoded is! Map<String, dynamic>) {
        return DiscoveryStartResult.error('invalid discovery payload');
      }

      return DiscoveryStartResult(
        success: decoded['success'] as bool? ?? false,
        totalKeys: (decoded['totalKeys'] as num?)?.toInt(),
        errorMessage: decoded['error']?.toString(),
      );
    } catch (e) {
      return DiscoveryStartResult.error('$e');
    }
  }

  final bool success;
  final int? totalKeys;
  final String? errorMessage;

  bool get hasError => errorMessage != null || !success;
}

/// Mixin providing device discovery and listing FFI methods.
mixin BridgeDiscoveryMixin {
  KeyrxBindings? get bindings;
  Object? get loadFailure;

  /// Select a device by path for the engine to use.
  ///
  /// Returns 0 on success, negative on error:
  /// - -1: Null pointer
  /// - -2: Invalid UTF-8
  /// - -3: Device path does not exist
  /// - -4: Lock error
  int selectDevice(String path) {
    final selectFn = bindings?.selectDevice;
    if (selectFn == null) return -1;

    final pathPtr = path.toNativeUtf8();
    try {
      return selectFn(pathPtr.cast<Char>());
    } finally {
      calloc.free(pathPtr);
    }
  }

  // NOTE: Legacy discovery methods (startDiscovery, etc.) have been removed.
  // Use DeviceRegistryService for device management.
}
