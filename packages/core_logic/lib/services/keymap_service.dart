/// Service for managing Keymaps via config FFI.
library;

import '../ffi/bridge.dart';
import '../ffi/generated_bindings.dart';
import '../models/keymap.dart';
import 'config_result.dart';

class KeymapService {
  KeymapService({required KeyrxBridge bridge}) : _bridge = bridge;

  final KeyrxBridge _bridge;

  /// List all keymaps.
  Future<ConfigOperationResult<List<Keymap>>> listKeymaps() async =>
      _guard('list keymaps', (bindings) {
        try {
          final keymaps = bindings.listKeymaps();
          return ConfigOperationResult.success(keymaps);
        } catch (e) {
          return ConfigOperationResult.error('list keymaps failed: $e');
        }
      });

  /// Persist or update a keymap definition.
  Future<ConfigOperationResult<Keymap>> saveKeymap(Keymap keymap) async =>
      _guard('save keymap', (bindings) {
        try {
          final saved = bindings.saveKeymap(keymap);
          return ConfigOperationResult.success(saved);
        } catch (e) {
          return ConfigOperationResult.error('save keymap failed: $e');
        }
      });

  /// Delete a keymap by id.
  Future<ConfigOperationResult<void>> deleteKeymap(String id) async =>
      _guard('delete keymap', (bindings) {
        try {
          bindings.deleteKeymap(id);
          return ConfigOperationResult.success();
        } catch (e) {
          return ConfigOperationResult.error('delete keymap failed: $e');
        }
      });

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
}
