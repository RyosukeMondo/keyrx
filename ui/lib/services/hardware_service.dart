/// Service for managing Hardware Profiles via config FFI.
library;

import 'package:core_logic/ffi/bridge.dart';
import 'package:core_logic/ffi/generated_bindings.dart';
import 'package:core_logic/models/hardware_profile.dart';
import 'package:flutter/foundation.dart';

import 'config_result.dart';

class HardwareService with ChangeNotifier {
  HardwareService({required KeyrxBridge bridge}) : _bridge = bridge;

  final KeyrxBridge _bridge;

  /// List all hardware profiles.
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

  /// Persist or update a hardware profile.
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
      notifyListeners();
    }
    return result;
  }

  /// Delete a hardware profile by id.
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
      notifyListeners();
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
}
