/// Device state data model for revolutionary mapping.
///
/// Represents the runtime state of a connected device,
/// matching the Rust DeviceState struct.
library;

import 'package:freezed_annotation/freezed_annotation.dart';
import 'device_identity.dart';

part 'device_state.freezed.dart';
part 'device_state.g.dart';

/// Runtime state of a connected device.
///
/// Tracks the device's identity, remap status, assigned profile,
/// and connection timestamps.
@freezed
class DeviceState with _$DeviceState {
  const factory DeviceState({
    /// Device identity
    required DeviceIdentity identity,

    /// Whether remapping is enabled for this device
    @JsonKey(name: 'remap_enabled') required bool remapEnabled,

    /// Assigned profile ID (if any)
    @JsonKey(name: 'profile_id') String? profileId,

    /// Connection timestamp (ISO 8601)
    @JsonKey(name: 'connected_at') required String connectedAt,

    /// Last update timestamp (ISO 8601)
    @JsonKey(name: 'updated_at') required String updatedAt,
  }) = _DeviceState;

  const DeviceState._();

  /// Create from JSON
  factory DeviceState.fromJson(Map<String, dynamic> json) =>
      _$DeviceStateFromJson(json);

  /// Check if this device has an assigned profile
  bool get hasProfile => profileId != null;

  /// Get a user-friendly status string
  String get statusText {
    if (!remapEnabled) {
      return 'Passthrough';
    }
    if (hasProfile) {
      return 'Active';
    }
    return 'No Profile';
  }
}
