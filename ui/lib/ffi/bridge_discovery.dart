/// Device discovery FFI methods.
///
/// Provides device discovery functionality for the KeyRx bridge.
library;

import 'dart:convert';
import 'dart:ffi';

import 'package:ffi/ffi.dart';

import 'bindings.dart';

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

/// Mixin providing device discovery FFI methods.
mixin BridgeDiscoveryMixin {
  KeyrxBindings? get bindings;
  Object? get loadFailure;

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
      ptr = discoveryFn(devicePtr, rows, colsPtr);
      if (ptr == nullptr) {
        return DiscoveryStartResult.error('startDiscovery returned null');
      }

      final raw = ptr.cast<Utf8>().toDartString();
      return DiscoveryStartResult.parse(raw);
    } catch (e) {
      return DiscoveryStartResult.error('$e');
    } finally {
      calloc.free(devicePtr);
      calloc.free(colsPtr);
      if (ptr != null && ptr != nullptr) {
        try {
          bindings?.freeString(ptr);
        } catch (_) {}
      }
    }
  }
}
