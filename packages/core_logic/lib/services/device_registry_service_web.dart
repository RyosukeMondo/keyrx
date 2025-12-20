/// Web implementation of DeviceRegistryService using HTTP API.
library;

import 'dart:async';

import '../models/bridge_models.dart';
import '../models/device_identity.dart';
import '../models/device_state.dart';
import '../web/api_client.dart';
import 'device_registry_service.dart';

/// Web-based implementation of DeviceRegistryService.
///
/// Uses HTTP API endpoints to communicate with backend server.
class DeviceRegistryServiceWeb implements DeviceRegistryService {
  DeviceRegistryServiceWeb({required WebApiClient client}) : _client = client {
    // Subscribe to device update events via WebSocket
    _subscription = _client
        .subscribe<void>(
          'device.updated',
          (_) {},
        )
        .listen((_) => _refreshDevices());
  }

  final WebApiClient _client;
  final _devicesController = StreamController<List<DeviceState>>.broadcast();
  StreamSubscription<void>? _subscription;

  @override
  Stream<List<DeviceState>> get devicesStream => _devicesController.stream;

  @override
  Future<List<DeviceState>> getDevices() async {
    final response = await _client.get('/api/devices');

    if (response.isError) {
      return [];
    }

    try {
      final json = response.data as Map<String, dynamic>;
      final data = json['devices'] as List<dynamic>;
      final devices = data
          .map((json) => DeviceState.fromJson(json as Map<String, dynamic>))
          .toList();

      // Update stream
      _devicesController.add(devices);

      return devices;
    } catch (e) {
      return [];
    }
  }

  @override
  Future<void> addVirtualDevice(DeviceIdentity identity) async {
    await _client.post(
      '/api/devices/virtual',
      body: identity.toJson(),
    );
    await _refreshDevices();
  }

  @override
  Future<void> removeVirtualDevice(String key) async {
    await _client.delete('/api/devices/virtual/$key');
    await _refreshDevices();
  }

  @override
  Future<DeviceRegistryResult<void>> toggleDevice(
    String deviceKey,
    bool enabled,
  ) async {
    final response = await _client.post(
      '/api/devices/$deviceKey/toggle',
      body: {'enabled': enabled},
    );

    if (response.isError) {
      return DeviceRegistryResult.error(response.error!);
    }

    await _refreshDevices();
    return DeviceRegistryResult(data: null);
  }

  @override
  Future<DeviceRegistryResult<void>> toggleRemap(
    String deviceKey,
    bool enabled,
  ) async {
    final response = await _client.post(
      '/api/devices/$deviceKey/toggle-remap',
      body: {'enabled': enabled},
    );

    if (response.isError) {
      return DeviceRegistryResult.error(response.error!);
    }

    await _refreshDevices();
    return DeviceRegistryResult(data: null);
  }

  @override
  Future<DeviceRegistryResult<void>> assignProfile(
    String deviceKey,
    String profileId,
  ) async {
    final response = await _client.post(
      '/api/devices/$deviceKey/assign-profile',
      body: {'profileId': profileId},
    );

    if (response.isError) {
      return DeviceRegistryResult.error(response.error!);
    }

    await _refreshDevices();
    return DeviceRegistryResult(data: null);
  }

  @override
  Future<DeviceRegistryResult<void>> setUserLabel(
    String deviceKey,
    String? label,
  ) async {
    final response = await _client.post(
      '/api/devices/$deviceKey/set-label',
      body: {'label': label},
    );

    if (response.isError) {
      return DeviceRegistryResult.error(response.error!);
    }

    await _refreshDevices();
    return DeviceRegistryResult(data: null);
  }

  @override
  Future<List<DeviceState>> refresh() async {
    await _client.post('/api/devices/refresh', body: {});
    return getDevices();
  }

  @override
  Future<void> migrateVirtualDevices(List<DeviceIdentity> identities) async {
    // Web platform doesn't need migration - devices managed by backend
  }

  Future<void> _refreshDevices() async {
    final devices = await getDevices();
    _devicesController.add(devices);
  }

  @override
  Future<void> dispose() async {
    await _subscription?.cancel();
    await _devicesController.close();
  }
}
