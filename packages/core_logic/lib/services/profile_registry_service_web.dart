/// Web implementation of ProfileRegistryService using HTTP API.
library;

import '../models/layout_type.dart';
import '../models/profile.dart';
import '../web/api_client.dart';
import 'profile_registry_service.dart';

/// Web-based implementation of ProfileRegistryService.
///
/// Uses HTTP API endpoints to communicate with backend server.
class ProfileRegistryServiceWeb implements ProfileRegistryService {
  ProfileRegistryServiceWeb({required WebApiClient client}) : _client = client;

  final WebApiClient _client;

  @override
  Future<List<String>> listProfiles() async {
    final response = await _client.get('/api/profiles');

    if (response.isError) {
      return const [];
    }

    try {
      final json = response.data as Map<String, dynamic>;
      final data = json['profiles'] as List<dynamic>;
      return data.cast<String>();
    } catch (e) {
      return const [];
    }
  }

  @override
  Future<Profile?> getProfile(String profileId) async {
    final response = await _client.get('/api/profiles/$profileId');

    if (response.isError) {
      return null;
    }

    try {
      final json = response.data as Map<String, dynamic>;
      return Profile.fromJson(json);
    } catch (e) {
      return null;
    }
  }

  @override
  Future<ProfileRegistryOperationResult> saveProfile(Profile profile) async {
    final response = await _client.post(
      '/api/profiles',
      body: profile.toJson(),
    );

    if (response.isError) {
      return ProfileRegistryOperationResult.error(response.error!);
    }

    return ProfileRegistryOperationResult.success();
  }

  @override
  Future<ProfileRegistryOperationResult> deleteProfile(String profileId) async {
    final response = await _client.delete('/api/profiles/$profileId');

    if (response.isError) {
      return ProfileRegistryOperationResult.error(response.error!);
    }

    return ProfileRegistryOperationResult.success();
  }

  @override
  Future<List<Profile>> findCompatibleProfiles(LayoutType layoutType) async {
    final response = await _client.get(
      '/api/profiles/compatible/${layoutType.name}',
    );

    if (response.isError) {
      return const [];
    }

    try {
      final json = response.data as Map<String, dynamic>;
      final data = json['profiles'] as List<dynamic>;
      return data
          .map((json) => Profile.fromJson(json as Map<String, dynamic>))
          .toList();
    } catch (e) {
      return const [];
    }
  }

  @override
  Future<List<String>> refresh() async {
    await _client.post('/api/profiles/refresh', body: {});
    return listProfiles();
  }

  @override
  Future<void> dispose() async {
    // No resources to dispose in web implementation
  }
}
