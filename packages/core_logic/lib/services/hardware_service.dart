/// Service for managing Hardware Profiles.
library;

import 'dart:async';

import '../models/hardware_profile.dart';
import 'config_result.dart';

export 'hardware_service_ffi.dart'
    if (dart.library.html) 'hardware_service_web.dart';

/// Service for managing hardware profiles.
abstract class HardwareService {
  /// Stream of changes to hardware profiles.
  Stream<void> get changes;

  /// List all hardware profiles.
  Future<ConfigOperationResult<List<HardwareProfile>>> listProfiles();

  /// Persist or update a hardware profile.
  Future<ConfigOperationResult<HardwareProfile>> saveProfile(
    HardwareProfile profile,
  );

  /// Delete a hardware profile by id.
  Future<ConfigOperationResult<void>> deleteProfile(String id);

  /// Dispose resources.
  void dispose();
}
