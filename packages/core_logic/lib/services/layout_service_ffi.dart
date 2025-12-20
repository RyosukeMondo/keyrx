/// FFI implementation of LayoutService.
library;

import 'dart:async';

import '../ffi/bridge.dart';
import '../ffi/generated_bindings.dart';
import '../models/virtual_layout.dart';
import 'config_result.dart';
import 'layout_service.dart';

class LayoutServiceFFI implements LayoutService {
  LayoutServiceFFI({required KeyrxBridge bridge}) : _bridge = bridge;

  final KeyrxBridge _bridge;
  final StreamController<void> _changesController = StreamController.broadcast();

  @override
  Stream<void> get changes => _changesController.stream;

  @override
  Future<ConfigOperationResult<List<VirtualLayout>>> listLayouts() async =>
      _guard('list layouts', (bindings) {
        try {
          final layouts = bindings.listVirtualLayouts();
          return ConfigOperationResult.success(layouts);
        } catch (e) {
          return ConfigOperationResult.error('list layouts failed: $e');
        }
      });

  @override
  Future<ConfigOperationResult<VirtualLayout>> saveLayout(
    VirtualLayout layout,
  ) async {
    final result = await _guard<VirtualLayout>('save layout', (bindings) {
      try {
        final saved = bindings.saveVirtualLayout(layout);
        return ConfigOperationResult.success(saved);
      } catch (e) {
        return ConfigOperationResult.error('save layout failed: $e');
      }
    });

    if (!result.hasError) {
      _changesController.add(null);
    }
    return result;
  }

  @override
  Future<ConfigOperationResult<void>> deleteLayout(String id) async {
    final result = await _guard<void>('delete layout', (bindings) {
      try {
        bindings.deleteVirtualLayout(id);
        return ConfigOperationResult.success();
      } catch (e) {
        return ConfigOperationResult.error('delete layout failed: $e');
      }
    });

    if (!result.hasError) {
      _changesController.add(null);
    }
    return result;
  }

  Future<ConfigOperationResult<T>> _guard<T>(
    String operation,
    ConfigOperationResult<T> Function(ConfigBindings bindings) action,
  ) async {
    final loadFailure = _bridge.loadFailure;
    if (loadFailure != null) {
      return ConfigOperationResult.error(
        'Engine unavailable ($operation): $loadFailure',
      );
    }

    final bindings = _bridge.configBindings;
    if (bindings == null) {
      return ConfigOperationResult.error(
        'Config bindings unavailable ($operation)',
      );
    }

    return action(bindings);
  }

  @override
  void dispose() {
    _changesController.close();
  }
}
