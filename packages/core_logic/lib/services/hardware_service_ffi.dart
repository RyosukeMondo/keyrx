/// FFI implementation of HardwareService.
library;

import 'dart:async';

import '../ffi/bridge.dart';
import '../ffi/generated_bindings.dart';
import '../models/hardware_profile.dart';
import 'config_result.dart';
import 'hardware_service.dart';

class HardwareServiceFFI implements HardwareService {
  HardwareServiceFFI({required KeyrxBridge bridge}) : _bridge = bridge;

  final KeyrxBridge _bridge;
  final StreamController<void> _changesController =
      StreamController.broadcast();

  @override
  Stream<void> get changes => _changesController.stream;

  @override
  Future<ConfigOperationResult<List<HardwareProfile>>> listProfiles() async =>
      _guard('list hardware profiles', (bindings) {
        try {
          final profiles = bindings.listHardwareProfiles();
          return ConfigOperationResult.success(profiles);
        } catch (e) {
          return ConfigOperationResult.error(
            'list hardware profiles failed: $e',
          );
        }
      });

  @override
  Future<ConfigOperationResult<HardwareProfile>> saveProfile(
    HardwareProfile profile,
  ) async {
    final result = await _guard<HardwareProfile>('save hardware profile', (
      bindings,
    ) {
      try {
        final saved = bindings.saveHardwareProfile(profile);
        return ConfigOperationResult.success(saved);
      } catch (e) {
        return ConfigOperationResult.error('save hardware profile failed: $e');
      }
    });

    if (!result.hasError) {
      _changesController.add(null);
    }
    return result;
  }

  @override
  Future<ConfigOperationResult<void>> deleteProfile(String id) async {
    final result = await _guard<void>('delete hardware profile', (bindings) {
      try {
        bindings.deleteHardwareProfile(id);
        return ConfigOperationResult.success();
      } catch (e) {
        return ConfigOperationResult.error(
          'delete hardware profile failed: $e',
        );
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
