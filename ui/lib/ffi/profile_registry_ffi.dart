/// Profile Registry FFI bindings for revolutionary mapping.
///
/// Provides FFI access to profile registry operations:
/// - List all profile IDs
/// - Get profile by ID
/// - Save profiles
/// - Delete profiles
/// - Find compatible profiles by layout type
library;

import 'dart:convert';

import '../models/layout_type.dart';
import '../models/profile.dart';
import 'bindings.dart';

/// Result wrapper for profile registry operations.
class ProfileRegistryResult<T> {
  const ProfileRegistryResult({this.data, this.errorMessage});

  factory ProfileRegistryResult.success(T data) =>
      ProfileRegistryResult(data: data);

  factory ProfileRegistryResult.error(String message) =>
      ProfileRegistryResult(errorMessage: message);

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
  static ProfileRegistryResult<T> parse<T>(
    String raw,
    T Function(dynamic json)? decoder,
  ) {
    final trimmed = raw.trim();

    // Handle error responses
    if (trimmed.toLowerCase().startsWith('error:')) {
      return ProfileRegistryResult.error(
        trimmed.substring('error:'.length).trim(),
      );
    }

    // Handle success responses
    if (!trimmed.toLowerCase().startsWith('ok:')) {
      return ProfileRegistryResult.error('invalid response format: $trimmed');
    }

    final payload = trimmed.substring('ok:'.length).trim();

    // Empty payload means void success
    if (payload.isEmpty) {
      return ProfileRegistryResult.success(null as T);
    }

    // Parse JSON payload if decoder provided
    if (decoder == null) {
      return ProfileRegistryResult.success(payload as T);
    }

    try {
      final decoded = json.decode(payload);
      return ProfileRegistryResult.success(decoder(decoded));
    } catch (e) {
      return ProfileRegistryResult.error('JSON decode error: $e');
    }
  }
}

/// Mixin providing profile registry FFI methods.
mixin ProfileRegistryFFIMixin {
  KeyrxBindings? get bindings;
  Object? get loadFailure;

  /// List all profile IDs.
  ///
  /// Returns a list of profile ID strings or an error.
  ProfileRegistryResult<List<String>> listProfiles() {
    if (bindings == null) {
      return ProfileRegistryResult.error(
        'profileRegistryListProfiles not available',
      );
    }

    try {
      final raw = bindings!.profileRegistry.listProfiles();
      final result = _FfiResultParser.parse<List<dynamic>>(
        raw,
        (json) => json as List<dynamic>,
      );

      if (result.hasError) {
        return ProfileRegistryResult.error(result.errorMessage!);
      }

      // Convert JSON array to list of strings
      final profileIds = result.data!.map((item) => item as String).toList();

      return ProfileRegistryResult.success(profileIds);
    } catch (e) {
      return ProfileRegistryResult.error('listProfiles exception: $e');
    }
  }

  /// Get a profile by ID.
  ///
  /// [profileId] - Profile ID to retrieve
  ///
  /// Returns a Profile object or an error.
  ProfileRegistryResult<Profile> getProfile(String profileId) {
    if (bindings == null) {
      return ProfileRegistryResult.error(
        'profileRegistryGetProfile not available',
      );
    }

    try {
      final raw = bindings!.profileRegistry.getProfile(profileId);
      final result = _FfiResultParser.parse<Map<String, dynamic>>(
        raw,
        (json) => json as Map<String, dynamic>,
      );

      if (result.hasError) {
        return ProfileRegistryResult.error(result.errorMessage!);
      }

      // Parse JSON to Profile
      final profile = Profile.fromJson(result.data!);
      return ProfileRegistryResult.success(profile);
    } catch (e) {
      return ProfileRegistryResult.error('getProfile exception: $e');
    }
  }

  /// Save a profile.
  ///
  /// [profile] - Profile to save
  ///
  /// Returns success or error.
  ProfileRegistryResult<void> saveProfile(Profile profile) {
    if (bindings == null) {
      return ProfileRegistryResult.error(
        'profileRegistrySaveProfile not available',
      );
    }

    try {
      // Convert profile to JSON
      final profileJson = json.encode(profile.toJson());
      final raw = bindings!.profileRegistry.saveProfile(profileJson);
      final result = _FfiResultParser.parse<void>(raw, null);

      if (result.hasError) {
        return ProfileRegistryResult.error(result.errorMessage!);
      }

      return ProfileRegistryResult.success(null);
    } catch (e) {
      return ProfileRegistryResult.error('saveProfile exception: $e');
    }
  }

  /// Delete a profile by ID.
  ///
  /// [profileId] - Profile ID to delete
  ///
  /// Returns success or error.
  ProfileRegistryResult<void> deleteProfile(String profileId) {
    if (bindings == null) {
      return ProfileRegistryResult.error(
        'profileRegistryDeleteProfile not available',
      );
    }

    try {
      final raw = bindings!.profileRegistry.deleteProfile(profileId);
      final result = _FfiResultParser.parse<void>(raw, null);

      if (result.hasError) {
        return ProfileRegistryResult.error(result.errorMessage!);
      }

      return ProfileRegistryResult.success(null);
    } catch (e) {
      return ProfileRegistryResult.error('deleteProfile exception: $e');
    }
  }

  /// Find profiles compatible with a given layout type.
  ///
  /// [layoutType] - Layout type to search for
  ///
  /// Returns a list of compatible Profile objects or an error.
  ProfileRegistryResult<List<Profile>> findCompatibleProfiles(
    LayoutType layoutType,
  ) {
    if (bindings == null) {
      return ProfileRegistryResult.error(
        'profileRegistryFindCompatibleProfiles not available',
      );
    }

    try {
      final layoutTypeStr = layoutType.toJsonString();
      final raw = bindings!.profileRegistry.findCompatibleProfiles(
        layoutTypeStr,
      );
      final result = _FfiResultParser.parse<List<dynamic>>(
        raw,
        (json) => json as List<dynamic>,
      );

      if (result.hasError) {
        return ProfileRegistryResult.error(result.errorMessage!);
      }

      // Convert JSON array to list of Profile objects
      final profiles = result.data!
          .map((item) => Profile.fromJson(item as Map<String, dynamic>))
          .toList();

      return ProfileRegistryResult.success(profiles);
    } catch (e) {
      return ProfileRegistryResult.error(
        'findCompatibleProfiles exception: $e',
      );
    }
  }
}
