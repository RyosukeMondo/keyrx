/// Web implementation of KeymapService.
library;

import '../models/keymap.dart';
import '../web/api_client.dart';
import 'config_result.dart';
import 'keymap_service.dart';

class KeymapServiceWeb implements KeymapService {
  KeymapServiceWeb({required WebApiClient client}) : _client = client;

  final WebApiClient _client;

  @override
  Future<ConfigOperationResult<List<Keymap>>> listKeymaps() async {
    final response = await _client.get('/api/keymaps');

    if (response.isError) {
      return ConfigOperationResult.error(response.error!);
    }

    try {
      final data = response.data as List<dynamic>;
      final keymaps = data
          .map((json) => Keymap.fromJson(json as Map<String, dynamic>))
          .toList();
      return ConfigOperationResult.success(keymaps);
    } catch (e) {
      return ConfigOperationResult.error('Failed to parse keymaps: $e');
    }
  }

  @override
  Future<ConfigOperationResult<Keymap>> saveKeymap(Keymap keymap) async {
    final response = await _client.post(
      '/api/keymaps',
      body: {'keymap': keymap.toJson()},
    );

    if (response.isError) {
      return ConfigOperationResult.error(response.error!);
    }

    try {
      final saved = Keymap.fromJson(response.data as Map<String, dynamic>);
      return ConfigOperationResult.success(saved);
    } catch (e) {
      return ConfigOperationResult.error('Failed to parse saved keymap: $e');
    }
  }

  @override
  Future<ConfigOperationResult<void>> deleteKeymap(String id) async {
    final response = await _client.delete('/api/keymaps/$id');

    if (response.isError) {
      return ConfigOperationResult.error(response.error!);
    }

    return ConfigOperationResult.success();
  }
}
