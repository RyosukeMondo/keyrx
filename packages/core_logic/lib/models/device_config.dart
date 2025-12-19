import 'package:freezed_annotation/freezed_annotation.dart';
import 'device_identity.dart';

part 'device_config.freezed.dart';
part 'device_config.g.dart';

/// Persistent configuration for the device registry.
///
/// Stores a list of known devices, including virtual devices
/// and user-assigned labels for physical devices.
@freezed
class DeviceConfig with _$DeviceConfig {
  const factory DeviceConfig({
    /// List of known devices and their configurations.
    @Default([]) List<DeviceIdentity> devices,
  }) = _DeviceConfig;

  /// Create from JSON
  factory DeviceConfig.fromJson(Map<String, dynamic> json) =>
      _$DeviceConfigFromJson(json);
}
