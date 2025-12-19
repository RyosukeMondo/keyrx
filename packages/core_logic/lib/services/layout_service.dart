/// Service for managing Virtual Layouts via config FFI.
library;

import 'dart:async';

import '../ffi/bridge.dart';
import '../ffi/generated_bindings.dart';
import '../models/virtual_layout.dart';
import 'config_result.dart';

class LayoutService {
  LayoutService({required KeyrxBridge bridge}) : _bridge = bridge;

  final KeyrxBridge _bridge;
  final StreamController<void> _changesController = StreamController.broadcast();

  /// Stream of changes to virtual layouts.
  Stream<void> get changes => _changesController.stream;

  /// List all virtual layouts stored in the config directory.
  Future<ConfigOperationResult<List<VirtualLayout>>> listLayouts() async =>
      _guard('list layouts', (bindings) {
        try {
          final layouts = bindings.listVirtualLayouts();
          return ConfigOperationResult.success(layouts);
        } catch (e) {
          return ConfigOperationResult.error('list layouts failed: $e');
        }
      });

  /// Persist a virtual layout definition and return the saved value.
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

  /// Delete a virtual layout by id.
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

  /// Dispose resources.
  void dispose() {
    _changesController.close();
  }
}
