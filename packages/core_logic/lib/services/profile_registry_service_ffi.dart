/// FFI implementation of ProfileRegistryService.
library;

import 'dart:typed_data';

import '../ffi/bridge.dart';
import '../models/layout_type.dart';
import '../models/profile.dart';
import 'profile_registry_service.dart';

/// Real ProfileRegistryService that wraps the KeyrxBridge.
class ProfileRegistryServiceFFI implements ProfileRegistryService {
  ProfileRegistryServiceFFI({required KeyrxBridge bridge}) : _bridge = bridge {
    _bridge.registerEventCallback(
      EventType.profileUpdated,
      _handleProfileEvent,
    );
  }

  final KeyrxBridge _bridge;
  List<String>? _cachedProfileIds;
  final Map<String, Profile> _profileCache = {};

  void _handleProfileEvent(Uint8List payload) {
    refresh();
  }

  @override
  Future<List<String>> listProfiles() async {
    if (_cachedProfileIds != null) {
      return _cachedProfileIds!;
    }
    return refresh();
  }

  @override
  Future<Profile?> getProfile(String profileId) async {
    // Check cache first
    if (_profileCache.containsKey(profileId)) {
      return _profileCache[profileId];
    }

    if (_bridge.loadFailure != null) {
      return null;
    }

    final result = _bridge.getProfile(profileId);

    if (result.hasError) {
      return null;
    }

    // Cache the profile
    final profile = result.data;
    if (profile != null) {
      _profileCache[profileId] = profile;
    }

    return profile;
  }

  @override
  Future<ProfileRegistryOperationResult> saveProfile(Profile profile) async {
    if (_bridge.loadFailure != null) {
      return ProfileRegistryOperationResult.error(
        'Engine unavailable: ${_bridge.loadFailure}',
      );
    }

    final result = _bridge.saveProfile(profile);

    if (result.hasError) {
      return ProfileRegistryOperationResult.error(
        _makeUserFriendly(result.errorMessage!, 'save profile'),
      );
    }

    // Invalidate caches to force refresh
    _cachedProfileIds = null;
    _profileCache[profile.id] = profile;

    return ProfileRegistryOperationResult.success();
  }

  @override
  Future<ProfileRegistryOperationResult> deleteProfile(String profileId) async {
    if (_bridge.loadFailure != null) {
      return ProfileRegistryOperationResult.error(
        'Engine unavailable: ${_bridge.loadFailure}',
      );
    }

    final result = _bridge.deleteProfile(profileId);

    if (result.hasError) {
      return ProfileRegistryOperationResult.error(
        _makeUserFriendly(result.errorMessage!, 'delete profile'),
      );
    }

    // Invalidate caches
    _cachedProfileIds = null;
    _profileCache.remove(profileId);

    return ProfileRegistryOperationResult.success();
  }

  @override
  Future<List<Profile>> findCompatibleProfiles(LayoutType layoutType) async {
    if (_bridge.loadFailure != null) {
      return const [];
    }

    final result = _bridge.findCompatibleProfiles(layoutType);

    if (result.hasError) {
      return const [];
    }

    // Update cache with found profiles
    final profiles = result.data ?? const [];
    for (final profile in profiles) {
      _profileCache[profile.id] = profile;
    }

    return profiles;
  }

  @override
  Future<List<String>> refresh() async {
    if (_bridge.loadFailure != null) {
      return const [];
    }

    final result = _bridge.listProfiles();

    if (result.hasError) {
      _cachedProfileIds = const [];
      return const [];
    }

    _cachedProfileIds = result.data ?? const [];
    return _cachedProfileIds!;
  }

  @override
  Future<void> dispose() async {
    _bridge.unregisterEventCallback(EventType.profileUpdated);
    _cachedProfileIds = null;
    _profileCache.clear();
  }

  /// Convert technical error messages to user-friendly messages.
  String _makeUserFriendly(String technicalError, String operation) {
    // Remove technical prefixes
    final cleaned = technicalError
        .replaceFirst('error:', '')
        .replaceFirst(RegExp(r'^\w+Exception:'), '')
        .trim();

    // Handle common error patterns
    if (cleaned.toLowerCase().contains('profile not found')) {
      return 'Profile not found. It may have been deleted.';
    }

    if (cleaned.toLowerCase().contains('invalid profile')) {
      return 'Invalid profile data: $cleaned';
    }

    if (cleaned.toLowerCase().contains('validation')) {
      return 'Profile validation failed. Please check the profile mappings.';
    }

    if (cleaned.toLowerCase().contains('json')) {
      return 'Failed to $operation due to a data format error. Please try again.';
    }

    if (cleaned.toLowerCase().contains('null')) {
      return 'Failed to $operation. The operation returned no response.';
    }

    if (cleaned.toLowerCase().contains('i/o') ||
        cleaned.toLowerCase().contains('io error')) {
      return 'Failed to $operation due to a file system error. Please check permissions.';
    }

    if (cleaned.toLowerCase().contains('duplicate')) {
      return 'A profile with this ID already exists.';
    }

    // If we can't map it, return a generic but helpful message
    return 'Failed to $operation: $cleaned';
  }
}
