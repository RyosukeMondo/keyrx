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
  const DeviceListResult({
    required this.devices,
    this.errorMessage,
  });

  factory DeviceListResult.error(String message) => DeviceListResult(
        devices: const [],
        errorMessage: message,
      );

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

      final devices = decoded.map((entry) {
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
      }).whereType<KeyboardDevice>().toList();

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

  factory DiscoveryStartResult.error(String message) => DiscoveryStartResult(
        success: false,
        errorMessage: message,
      );

  factory DiscoveryStartResult.parse(String raw) {
    final trimmed = raw.trim();
    if (trimmed.toLowerCase().startsWith('error:')) {
      return DiscoveryStartResult.error(
          trimmed.substring('error:'.length).trim());
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

  /// List available keyboard devices.
  ///
  /// Returns a list of [KeyboardDevice] or an error result.
  DeviceListResult listDevices() {
    final listFn = bindings?.listDevices;
    if (listFn == null) {
      return DeviceListResult.error('listDevices not available');
    }

    Pointer<Char>? ptr;
    try {
      ptr = listFn();
      if (ptr == nullptr) {
        return DeviceListResult.error('listDevices returned null');
      }

      final raw = ptr!.cast<Utf8>().toDartString();
      return DeviceListResult.parse(raw);
    } catch (e) {
      return DeviceListResult.error('$e');
    } finally {
      if (ptr != null && ptr != nullptr) {
        try {
          bindings?.freeString(ptr!);
        } catch (_) {}
      }
    }
  }

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

  /// Start a discovery session for a device.
  ///
  /// [deviceId] - Device identifier as "vendorId:productId" (hex format)
  /// [rows] - Number of rows in the keyboard layout
  /// [colsPerRow] - Number of columns for each row
  DiscoveryStartResult startDiscovery(
    String deviceId,
    int rows,
    List<int> colsPerRow,
  ) {
    final discoveryFn = bindings?.startDiscovery;
    if (discoveryFn == null) {
      return DiscoveryStartResult.error('startDiscovery not available');
    }

    final devicePtr = deviceId.toNativeUtf8();
    final colsJson = json.encode(colsPerRow);
    final colsPtr = colsJson.toNativeUtf8();
    Pointer<Char>? ptr;
    try {
      ptr = discoveryFn(devicePtr.cast<Char>(), rows, colsPtr.cast<Char>());
      if (ptr == nullptr) {
        return DiscoveryStartResult.error('startDiscovery returned null');
      }

      final raw = ptr!.cast<Utf8>().toDartString();
      return DiscoveryStartResult.parse(raw);
    } catch (e) {
      return DiscoveryStartResult.error('$e');
    } finally {
      calloc.free(devicePtr);
      calloc.free(colsPtr);
      if (ptr != null && ptr != nullptr) {
        try {
          bindings?.freeString(ptr!);
        } catch (_) {}
      }
    }
  }

  /// Cancel the active discovery session.
  ///
  /// Returns 0 on success, -1 if no discovery is active.
  int cancelDiscovery() {
    final cancelFn = bindings?.cancelDiscovery;
    if (cancelFn == null) return -1;

    try {
      return cancelFn();
    } catch (e) {
      return -1;
    }
  }

  /// Process a discovery event.
  ///
  /// Returns:
  /// - 0: Success
  /// - 1: Discovery complete
  /// - -1: No active session
  /// - -2: Cancelled
  int processDiscoveryEvent(int scanCode, bool pressed, int timestampUs) {
    final processFn = bindings?.processDiscoveryEvent;
    if (processFn == null) return -1;

    try {
      Pointer<Char>? resultPtr = processFn(scanCode, pressed, timestampUs);
      if (resultPtr == nullptr) return -1;

      final resultStr = resultPtr!.cast<Utf8>().toDartString();
      try {
        bindings?.freeString(resultPtr!);
      } catch (_) {}

      if (resultStr.startsWith('ok:')) {
        final payload = resultStr.substring(3).trim();
        return int.tryParse(payload) ?? 0;
      }
      return -1;
    } catch (e) {
      return -1;
    }
  }
}
