/// Web implementation of RuntimeService using HTTP API.
library;

import '../models/hardware_profile.dart';
import '../models/runtime_config.dart';
import '../web/api_client.dart';
import 'config_result.dart';
import 'runtime_service.dart';

/// Web-based implementation of RuntimeService.
///
/// Uses HTTP API endpoints to communicate with backend server.
class RuntimeServiceWeb implements RuntimeService {
  RuntimeServiceWeb({required WebApiClient client}) : _client = client;

  final WebApiClient _client;
  RuntimeConfig? _lastConfig;

  @override
  RuntimeConfig? get lastConfig => _lastConfig;

  @override
  Future<ConfigOperationResult<RuntimeConfig>> getConfig() async {
    final response = await _client.get('/api/runtime/config');

    if (response.isError) {
      return ConfigOperationResult.error(response.error!);
    }

    try {
      final json = response.data as Map<String, dynamic>;
      final config = RuntimeConfig.fromJson(
        json['config'] as Map<String, dynamic>,
      );
      _lastConfig = config;
      _lastConfig = config;
      return ConfigOperationResult.success(config);
    } catch (e) {
      return ConfigOperationResult.error('Failed to parse runtime config: $e');
    }
  }

  @override
  Future<ConfigOperationResult<RuntimeConfig>> addSlot(
    DeviceInstanceId device,
    ProfileSlot slot,
  ) async {
    final response = await _client.post(
      '/api/runtime/slots/add',
      body: {
        'device': device.toJson(),
        'slot': slot.toJson(),
        'priority': slot.priority,
      },
    );

    if (response.isError) {
      return ConfigOperationResult.error(response.error!);
    }

    try {
      final json = response.data as Map<String, dynamic>;
      final config = RuntimeConfig.fromJson(
        json['config'] as Map<String, dynamic>,
      );
      _lastConfig = config;
      _lastConfig = config;
      return ConfigOperationResult.success(config);
    } catch (e) {
      return ConfigOperationResult.error('Failed to parse runtime config: $e');
    }
  }

  @override
  Future<ConfigOperationResult<RuntimeConfig>> removeSlot(
    DeviceInstanceId device,
    String slotId,
  ) async {
    final response = await _client.post(
      '/api/runtime/slots/remove',
      body: {
        'device': device.toJson(),
        'slotId': slotId,
      },
    );

    if (response.isError) {
      return ConfigOperationResult.error(response.error!);
    }

    try {
      final json = response.data as Map<String, dynamic>;
      final config = RuntimeConfig.fromJson(
        json['config'] as Map<String, dynamic>,
      );
      return ConfigOperationResult.success(config);
    } catch (e) {
      return ConfigOperationResult.error('Failed to parse runtime config: $e');
    }
  }

  @override
  Future<ConfigOperationResult<RuntimeConfig>> reorderSlot(
    DeviceInstanceId device,
    String slotId,
    int priority,
  ) async {
    final response = await _client.post(
      '/api/runtime/slots/reorder',
      body: {
        'device': device.toJson(),
        'slotId': slotId,
        'priority': priority,
      },
    );

    if (response.isError) {
      return ConfigOperationResult.error(response.error!);
    }

    try {
      final json = response.data as Map<String, dynamic>;
      final config = RuntimeConfig.fromJson(
        json['config'] as Map<String, dynamic>,
      );
      return ConfigOperationResult.success(config);
    } catch (e) {
      return ConfigOperationResult.error('Failed to parse runtime config: $e');
    }
  }

  @override
  Future<ConfigOperationResult<RuntimeConfig>> setSlotActive(
    DeviceInstanceId device,
    String slotId,
    bool active,
  ) async {
    final response = await _client.post(
      '/api/runtime/slots/set-active',
      body: {
        'device': device.toJson(),
        'slotId': slotId,
        'active': active,
      },
    );

    if (response.isError) {
      return ConfigOperationResult.error(response.error!);
    }

    try {
      final json = response.data as Map<String, dynamic>;
      final config = RuntimeConfig.fromJson(
        json['config'] as Map<String, dynamic>,
      );
      return ConfigOperationResult.success(config);
    } catch (e) {
      return ConfigOperationResult.error('Failed to parse runtime config: $e');
    }
  }
}
