/// Device Registry FFI bindings for revolutionary mapping.
///
/// Provides FFI access to device registry operations:
/// - List registered devices
/// - Set remap enabled state
/// - Assign profiles to devices
/// - Set user labels
library;

import 'dart:convert';
import 'dart:ffi';

import 'package:ffi/ffi.dart';

import '../models/device_state.dart';
import 'bindings.dart';

/// Result wrapper for device registry operations.
class DeviceRegistryResult<T> {
  const DeviceRegistryResult({
    this.data,
    this.errorMessage,
  });

  factory DeviceRegistryResult.success(T data) =>
      DeviceRegistryResult(data: data);

  factory DeviceRegistryResult.error(String message) =>
      DeviceRegistryResult(errorMessage: message);

  final T? data;
  final String? errorMessage;

  bool get hasError => errorMessage != null;
  bool get isSuccess => data != null && !hasError;
}

/// Parser for C string results from FFI.
class _FfiResultParser {
  /// Parse a raw C string result into structured data.
  ///
  /// Expected formats:
  /// - Success: "ok:" or "ok:\<json\>"
  /// - Error: "error:\<message\>"
  static DeviceRegistryResult<T> parse<T>(
    String raw,
    T Function(dynamic json)? decoder,
  ) {
    final trimmed = raw.trim();

    // Handle error responses
    if (trimmed.toLowerCase().startsWith('error:')) {
      return DeviceRegistryResult.error(
          trimmed.substring('error:'.length).trim());
    }

    // Handle success responses
    if (!trimmed.toLowerCase().startsWith('ok:')) {
      return DeviceRegistryResult.error('invalid response format: $trimmed');
    }

    final payload = trimmed.substring('ok:'.length).trim();

    // Empty payload means void success
    if (payload.isEmpty) {
      return DeviceRegistryResult.success(null as T);
    }

    // Parse JSON payload if decoder provided
    if (decoder == null) {
      return DeviceRegistryResult.success(payload as T);
    }

    try {
      final decoded = json.decode(payload);
      return DeviceRegistryResult.success(decoder(decoded));
    } catch (e) {
      return DeviceRegistryResult.error('JSON decode error: $e');
    }
  }
}

/// Mixin providing device registry FFI methods.
mixin DeviceRegistryFFIMixin {
  KeyrxBindings? get bindings;
  Object? get loadFailure;

  /// Free a C string pointer.
  void _freeString(Pointer<Char> ptr) {
    if (ptr != nullptr) {
      try {
        bindings?.freeString(ptr);
      } catch (_) {
        // Ignore errors during cleanup
      }
    }
  }

  /// List all registered devices.
  ///
  /// Returns a list of DeviceState objects or an error.
  DeviceRegistryResult<List<DeviceState>> listDevices() {
    final listFn = bindings?.deviceRegistryListDevices;
    if (listFn == null) {
      return DeviceRegistryResult.error('deviceRegistryListDevices not available');
    }

    Pointer<Char>? ptr;
    try {
      ptr = listFn();
      if (ptr == nullptr) {
        return DeviceRegistryResult.error('listDevices returned null');
      }

      final raw = ptr.cast<Utf8>().toDartString();
      final result = _FfiResultParser.parse<List<dynamic>>(
        raw,
        (json) => json as List<dynamic>,
      );

      if (result.hasError) {
        return DeviceRegistryResult.error(result.errorMessage!);
      }

      // Convert JSON array to list of DeviceState
      final devices = result.data!
          .map((item) => DeviceState.fromJson(item as Map<String, dynamic>))
          .toList();

      return DeviceRegistryResult.success(devices);
    } catch (e) {
      return DeviceRegistryResult.error('listDevices exception: $e');
    } finally {
      if (ptr != null && ptr != nullptr) {
        _freeString(ptr);
      }
    }
  }

  /// Set remap enabled state for a device.
  ///
  /// [deviceKey] - Device identity key (format: "VID:PID:SERIAL")
  /// [enabled] - Whether remapping should be enabled
  ///
  /// Returns success or error.
  DeviceRegistryResult<void> setRemapEnabled(
    String deviceKey,
    bool enabled,
  ) {
    final setRemapFn = bindings?.deviceRegistrySetRemapEnabled;
    if (setRemapFn == null) {
      return DeviceRegistryResult.error('deviceRegistrySetRemapEnabled not available');
    }

    final keyPtr = deviceKey.toNativeUtf8();
    Pointer<Char>? resultPtr;

    try {
      resultPtr = setRemapFn(keyPtr.cast<Char>(), enabled ? 1 : 0);
      if (resultPtr == nullptr) {
        return DeviceRegistryResult.error('setRemapEnabled returned null');
      }

      final raw = resultPtr.cast<Utf8>().toDartString();
      final result = _FfiResultParser.parse<void>(raw, null);

      if (result.hasError) {
        return DeviceRegistryResult.error(result.errorMessage!);
      }

      return DeviceRegistryResult.success(null);
    } catch (e) {
      return DeviceRegistryResult.error('setRemapEnabled exception: $e');
    } finally {
      calloc.free(keyPtr);
      if (resultPtr != null && resultPtr != nullptr) {
        _freeString(resultPtr);
      }
    }
  }

  /// Assign a profile to a device.
  ///
  /// [deviceKey] - Device identity key (format: "VID:PID:SERIAL")
  /// [profileId] - Profile ID to assign
  ///
  /// Returns success or error.
  DeviceRegistryResult<void> assignProfile(
    String deviceKey,
    String profileId,
  ) {
    final assignFn = bindings?.deviceRegistryAssignProfile;
    if (assignFn == null) {
      return DeviceRegistryResult.error('deviceRegistryAssignProfile not available');
    }

    final keyPtr = deviceKey.toNativeUtf8();
    final profilePtr = profileId.toNativeUtf8();
    Pointer<Char>? resultPtr;

    try {
      resultPtr = assignFn(keyPtr.cast<Char>(), profilePtr.cast<Char>());
      if (resultPtr == nullptr) {
        return DeviceRegistryResult.error('assignProfile returned null');
      }

      final raw = resultPtr.cast<Utf8>().toDartString();
      final result = _FfiResultParser.parse<void>(raw, null);

      if (result.hasError) {
        return DeviceRegistryResult.error(result.errorMessage!);
      }

      return DeviceRegistryResult.success(null);
    } catch (e) {
      return DeviceRegistryResult.error('assignProfile exception: $e');
    } finally {
      calloc.free(keyPtr);
      calloc.free(profilePtr);
      if (resultPtr != null && resultPtr != nullptr) {
        _freeString(resultPtr);
      }
    }
  }

  /// Set user label for a device.
  ///
  /// [deviceKey] - Device identity key (format: "VID:PID:SERIAL")
  /// [label] - Optional user label (null to clear)
  ///
  /// Returns success or error.
  DeviceRegistryResult<void> setUserLabel(
    String deviceKey,
    String? label,
  ) {
    final setLabelFn = bindings?.deviceRegistrySetUserLabel;
    if (setLabelFn == null) {
      return DeviceRegistryResult.error('deviceRegistrySetUserLabel not available');
    }

    final keyPtr = deviceKey.toNativeUtf8();
    Pointer<Utf8>? labelPtr;
    Pointer<Char>? resultPtr;

    try {
      // Convert label to native (null if not provided)
      if (label != null) {
        labelPtr = label.toNativeUtf8();
      }

      resultPtr = setLabelFn(
        keyPtr.cast<Char>(),
        labelPtr?.cast<Char>() ?? nullptr,
      );
      if (resultPtr == nullptr) {
        return DeviceRegistryResult.error('setUserLabel returned null');
      }

      final raw = resultPtr.cast<Utf8>().toDartString();
      final result = _FfiResultParser.parse<void>(raw, null);

      if (result.hasError) {
        return DeviceRegistryResult.error(result.errorMessage!);
      }

      return DeviceRegistryResult.success(null);
    } catch (e) {
      return DeviceRegistryResult.error('setUserLabel exception: $e');
    } finally {
      calloc.free(keyPtr);
      if (labelPtr != null) {
        calloc.free(labelPtr);
      }
      if (resultPtr != null && resultPtr != nullptr) {
        _freeString(resultPtr);
      }
    }
  }
}
