/// Profile Registry service for revolutionary mapping.
///
/// Provides a high-level async API for profile registry operations,
/// wrapping FFI calls with error handling and user-friendly messages.
library;

import '../models/layout_type.dart';
import '../models/profile.dart';

export 'profile_registry_service_ffi.dart'
    if (dart.library.html) 'profile_registry_service_web.dart';

/// Result of a profile registry operation.
class ProfileRegistryOperationResult {
  const ProfileRegistryOperationResult({
    required this.success,
    this.errorMessage,
  });

  factory ProfileRegistryOperationResult.success() =>
      const ProfileRegistryOperationResult(success: true);

  factory ProfileRegistryOperationResult.error(String message) =>
      ProfileRegistryOperationResult(success: false, errorMessage: message);

  final bool success;
  final String? errorMessage;
}

/// Abstraction for profile registry operations.
abstract class ProfileRegistryService {
  /// Get all profile IDs.
  ///
  /// Returns a list of profile ID strings representing all saved profiles.
  Future<List<String>> listProfiles();

  /// Get a profile by ID.
  ///
  /// [profileId] - Profile ID to retrieve
  ///
  /// Returns the profile or null if not found or an error occurred.
  Future<Profile?> getProfile(String profileId);

  /// Save a profile.
  ///
  /// [profile] - Profile to save
  ///
  /// Returns a result indicating success or failure with error details.
  Future<ProfileRegistryOperationResult> saveProfile(Profile profile);

  /// Delete a profile by ID.
  ///
  /// [profileId] - Profile ID to delete
  ///
  /// Returns a result indicating success or failure with error details.
  Future<ProfileRegistryOperationResult> deleteProfile(String profileId);

  /// Find profiles compatible with a given layout type.
  ///
  /// [layoutType] - Layout type to search for
  ///
  /// Returns a list of compatible profiles.
  Future<List<Profile>> findCompatibleProfiles(LayoutType layoutType);

  /// Refresh the profile list.
  ///
  /// Forces a refresh of the profile list from the registry.
  Future<List<String>> refresh();

  /// Dispose any held resources.
  Future<void> dispose();
}
