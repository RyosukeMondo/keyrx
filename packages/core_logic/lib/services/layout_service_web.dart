/// Web implementation of LayoutService.
library;

import 'dart:async';

import '../models/virtual_layout.dart';
import '../web/api_client.dart';
import 'config_result.dart';
import 'layout_service.dart';

class LayoutServiceWeb implements LayoutService {
  LayoutServiceWeb({required WebApiClient client}) : _client = client {
    _subscription = _client
        .subscribe<void>(
          'layout.changed',
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
  Future<ConfigOperationResult<List<VirtualLayout>>> listLayouts() async {
    final response = await _client.get('/api/layouts');

    if (response.isError) {
      return ConfigOperationResult.error(response.error!);
    }

    try {
      final data = response.data as List<dynamic>;
      final layouts = data
          .map((json) => VirtualLayout.fromJson(json as Map<String, dynamic>))
          .toList();
      return ConfigOperationResult.success(layouts);
    } catch (e) {
      return ConfigOperationResult.error('Failed to parse layouts: $e');
    }
  }

  @override
  Future<ConfigOperationResult<VirtualLayout>> saveLayout(
    VirtualLayout layout,
  ) async {
    final response = await _client.post(
      '/api/layouts',
      body: {'layout': layout.toJson()},
    );

    if (response.isError) {
      return ConfigOperationResult.error(response.error!);
    }

    try {
      final saved = VirtualLayout.fromJson(
        response.data as Map<String, dynamic>,
      );
      return ConfigOperationResult.success(saved);
    } catch (e) {
      return ConfigOperationResult.error('Failed to parse saved layout: $e');
    }
  }

  @override
  Future<ConfigOperationResult<void>> deleteLayout(String id) async {
    final response = await _client.delete('/api/layouts/$id');

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
