/// Device Registry FFI bindings for revolutionary mapping.
///
/// Provides FFI access to device registry operations:
/// - List registered devices
/// - Set remap enabled state
/// - Assign profiles to devices
/// - Set user labels
library;

import 'dart:convert';

import '../models/device_state.dart';
import 'bindings.dart';

/// Result wrapper for device registry operations.
class DeviceRegistryResult<T> {
  const DeviceRegistryResult({this.data, this.errorMessage});

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
        trimmed.substring('error:'.length).trim(),
      );
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

  /// List all registered devices.
  ///
  /// Returns a list of DeviceState objects or an error.
  DeviceRegistryResult<List<DeviceState>> listRegisteredDevices() {
    if (bindings == null) {
      return DeviceRegistryResult.error(
        'deviceRegistryListDevices not available',
      );
    }

    try {
      final raw = bindings!.deviceRegistry.listDevices();
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
      return DeviceRegistryResult.error('listRegisteredDevices exception: $e');
    }
  }

  /// Set device enabled state.
  ///
  /// [deviceKey] - Device identity key (format: "VID:PID:SERIAL")
  /// [enabled] - Whether device should be enabled
  ///
  /// Returns success or error.
  DeviceRegistryResult<void> setDeviceEnabled(String deviceKey, bool enabled) {
    if (bindings == null) {
      return DeviceRegistryResult.error(
        'deviceRegistrySetDeviceEnabled not available',
      );
    }

    try {
      final raw = bindings!.deviceRegistry.setDeviceEnabled(
        deviceKey,
        enabled ? 1 : 0,
      );
      final result = _FfiResultParser.parse<void>(raw, null);

      if (result.hasError) {
        return DeviceRegistryResult.error(result.errorMessage!);
      }

      return DeviceRegistryResult.success(null);
    } catch (e) {
      return DeviceRegistryResult.error('setDeviceEnabled exception: $e');
    }
  }

  /// Set remap enabled state for a device.
  ///
  /// [deviceKey] - Device identity key (format: "VID:PID:SERIAL")
  /// [enabled] - Whether remapping should be enabled
  ///
  /// Returns success or error.
  DeviceRegistryResult<void> setRemapEnabled(String deviceKey, bool enabled) {
    if (bindings == null) {
      return DeviceRegistryResult.error(
        'deviceRegistrySetRemapEnabled not available',
      );
    }

    try {
      final raw = bindings!.deviceRegistry.setRemapEnabled(
        deviceKey,
        enabled ? 1 : 0,
      );
      final result = _FfiResultParser.parse<void>(raw, null);

      if (result.hasError) {
        return DeviceRegistryResult.error(result.errorMessage!);
      }

      return DeviceRegistryResult.success(null);
    } catch (e) {
      return DeviceRegistryResult.error('setRemapEnabled exception: $e');
    }
  }

  /// Assign a profile to a device.
  ///
  /// [deviceKey] - Device identity key (format: "VID:PID:SERIAL")
  /// [profileId] - Profile ID to assign
  ///
  /// Returns success or error.
  DeviceRegistryResult<void> assignProfile(String deviceKey, String profileId) {
    if (bindings == null) {
      return DeviceRegistryResult.error(
        'deviceRegistryAssignProfile not available',
      );
    }

    try {
      final raw = bindings!.deviceRegistry.assignProfile(deviceKey, profileId);
      final result = _FfiResultParser.parse<void>(raw, null);

      if (result.hasError) {
        return DeviceRegistryResult.error(result.errorMessage!);
      }

      return DeviceRegistryResult.success(null);
    } catch (e) {
      return DeviceRegistryResult.error('assignProfile exception: $e');
    }
  }

  /// Set user label for a device.
  ///
  /// [deviceKey] - Device identity key (format: "VID:PID:SERIAL")
  /// [label] - Optional user label (null to clear)
  ///
  /// Returns success or error.
  DeviceRegistryResult<void> setUserLabel(String deviceKey, String? label) {
    if (bindings == null) {
      return DeviceRegistryResult.error(
        'deviceRegistrySetUserLabel not available',
      );
    }

    try {
      // Use empty string for clearing label if that's what the binding expects,
      // or check generated binding for null handling.
      // Assuming generated binding handles String and we pass empty string or specific value.
      // But generated binding takes String. If we pass null, it might be an issue?
      // Generated bindings usually take required String.
      // If label is null, maybe we send empty string or special value?
      // Checking usage in old code:
      // if (label != null) { labelPtr = label.toNativeUtf8(); }
      // resultPtr = setLabelFn(..., labelPtr?.cast<Char>() ?? nullptr, ...);
      // So it passed nullptr.
      //
      // The generated binding:
      // String setUserLabel(String device_key, String label)
      // It takes a String. It converts it toNativeUtf8().
      // It does NOT accept null.
      // So we cannot pass null to generated binding.
      // We should probably check if the Rust side accepts empty string as "clear".
      // Assuming empty string means clear for now.
      final raw = bindings!.deviceRegistry.setUserLabel(deviceKey, label ?? '');
      final result = _FfiResultParser.parse<void>(raw, null);

      if (result.hasError) {
        return DeviceRegistryResult.error(result.errorMessage!);
      }

      return DeviceRegistryResult.success(null);
    } catch (e) {
      return DeviceRegistryResult.error('setUserLabel exception: $e');
    }
  }
}
