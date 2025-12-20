/// Web implementation of HardwareService.
library;

import 'dart:async';

import '../models/hardware_profile.dart';
import '../web/api_client.dart';
import 'config_result.dart';
import 'hardware_service.dart';

class HardwareServiceWeb implements HardwareService {
  HardwareServiceWeb({required WebApiClient client}) : _client = client {
    _subscription = _client
        .subscribe<void>(
          'hardware.changed',
          (_) {},
        )
        .listen((_) => _changesController.add(null));
  }

  final WebApiClient _client;
  final StreamController<void> _changesController =
      StreamController.broadcast();
  StreamSubscription<void>? _subscription;

  @override
  Stream<void> get changes => _changesController.stream;

  @override
  Future<ConfigOperationResult<List<HardwareProfile>>> listProfiles() async {
    final response = await _client.get('/api/hardware/profiles');

    if (response.isError) {
      return ConfigOperationResult.error(response.error!);
    }

    try {
      final data = response.data as List<dynamic>;
      final profiles = data
          .map((json) => HardwareProfile.fromJson(json as Map<String, dynamic>))
          .toList();
      return ConfigOperationResult.success(profiles);
    } catch (e) {
      return ConfigOperationResult.error('Failed to parse profiles: $e');
    }
  }

  @override
  Future<ConfigOperationResult<HardwareProfile>> saveProfile(
    HardwareProfile profile,
  ) async {
    final response = await _client.post(
      '/api/hardware/profiles',
      body: {'profile': profile.toJson()},
    );

    if (response.isError) {
      return ConfigOperationResult.error(response.error!);
    }

    try {
      final saved = HardwareProfile.fromJson(
        response.data as Map<String, dynamic>,
      );
      return ConfigOperationResult.success(saved);
    } catch (e) {
      return ConfigOperationResult.error('Failed to parse saved profile: $e');
    }
  }

  @override
  Future<ConfigOperationResult<void>> deleteProfile(String id) async {
    final response = await _client.delete('/api/hardware/profiles/$id');

    if (response.isError) {
      return ConfigOperationResult.error(response.error!);
    }

    return ConfigOperationResult.success();
  }

  @override
  void dispose() {
    _subscription?.cancel();
    _changesController.close();
  }
}
