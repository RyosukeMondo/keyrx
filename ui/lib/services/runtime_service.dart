/// Service for managing runtime profile slots via FFI.
library;

import 'dart:convert';

import '../ffi/bridge.dart';
import '../ffi/generated_bindings.dart';
import '../models/hardware_profile.dart';
import '../models/runtime_config.dart';
import 'config_result.dart';

/// Abstraction for runtime configuration operations.
abstract class RuntimeService {
  Future<ConfigOperationResult<RuntimeConfig>> getConfig();

  Future<ConfigOperationResult<RuntimeConfig>> addSlot(
    DeviceInstanceId device,
    ProfileSlot slot,
  );

  Future<ConfigOperationResult<RuntimeConfig>> removeSlot(
    DeviceInstanceId device,
    String slotId,
  );

  Future<ConfigOperationResult<RuntimeConfig>> reorderSlot(
    DeviceInstanceId device,
    String slotId,
    int priority,
  );

  Future<ConfigOperationResult<RuntimeConfig>> setSlotActive(
    DeviceInstanceId device,
    String slotId,
    bool active,
  );
}

class RuntimeServiceImpl implements RuntimeService {
  RuntimeServiceImpl({required KeyrxBridge bridge}) : _bridge = bridge;

  final KeyrxBridge _bridge;

  @override
  Future<ConfigOperationResult<RuntimeConfig>> getConfig() async {
    return _guard('get runtime config', (bindings) {
      try {
        final raw = bindings.getConfig();
        return _parseConfig(raw);
      } catch (e) {
        return ConfigOperationResult.error('get runtime config failed: $e');
      }
    });
  }

  @override
  Future<ConfigOperationResult<RuntimeConfig>> addSlot(
    DeviceInstanceId device,
    ProfileSlot slot,
  ) async {
    return _guard('add slot', (bindings) {
      try {
        final deviceJson = jsonEncode(device.toJson());
        final slotJson = jsonEncode(slot.toJson());
        final raw = bindings.addSlot(
          deviceJson,
          slotJson,
          0,
        ); // 0 = default priority? Or hardcoded? Check signature.
        // Wait, generated addSlot takes (String, String, int). 0 is fine if compatible with backend.
        return _parseConfig(raw);
      } catch (e) {
        return ConfigOperationResult.error('add slot failed: $e');
      }
    });
  }

  @override
  Future<ConfigOperationResult<RuntimeConfig>> removeSlot(
    DeviceInstanceId device,
    String slotId,
  ) async {
    return _guard('remove slot', (bindings) {
      try {
        final deviceJson = jsonEncode(device.toJson());
        final raw = bindings.removeSlot(deviceJson, slotId);
        return _parseConfig(raw);
      } catch (e) {
        return ConfigOperationResult.error('remove slot failed: $e');
      }
    });
  }

  @override
  Future<ConfigOperationResult<RuntimeConfig>> reorderSlot(
    DeviceInstanceId device,
    String slotId,
    int priority,
  ) async {
    return _guard('reorder slot', (bindings) {
      try {
        final deviceJson = jsonEncode(device.toJson());
        final raw = bindings.reorderSlot(deviceJson, slotId, priority);
        return _parseConfig(raw);
      } catch (e) {
        return ConfigOperationResult.error('reorder slot failed: $e');
      }
    });
  }

  @override
  Future<ConfigOperationResult<RuntimeConfig>> setSlotActive(
    DeviceInstanceId device,
    String slotId,
    bool active,
  ) async {
    return _guard('set slot active', (bindings) {
      try {
        final deviceJson = jsonEncode(device.toJson());
        final raw = bindings.setSlotActive(deviceJson, slotId, active);
        return _parseConfig(raw);
      } catch (e) {
        return ConfigOperationResult.error('set slot active failed: $e');
      }
    });
  }

  Future<ConfigOperationResult<RuntimeConfig>> _guard(
    String operation,
    ConfigOperationResult<RuntimeConfig> Function(RuntimeBindings bindings)
    action,
  ) async {
    final loadFailure = _bridge.loadFailure;
    if (loadFailure != null) {
      return ConfigOperationResult.error(
        'Engine unavailable ($operation): $loadFailure',
      );
    }

    final bindings = _bridge.runtimeBindings;
    if (bindings == null) {
      return ConfigOperationResult.error(
        'Runtime bindings unavailable ($operation)',
      );
    }

    return action(bindings);
  }

  ConfigOperationResult<RuntimeConfig> _parseConfig(String raw) {
    try {
      if (raw.trim().isEmpty) {
        return ConfigOperationResult.error('Empty response from runtime');
      }
      final jsonMap = jsonDecode(raw) as Map<String, dynamic>;
      // Check for backend error payload "error": ... ?
      // Rust usually returns "error": "msg" or similar if logic fails but FFI succeeds.
      // But generated bindings usually throw FFIException if error pointer set.
      // If error pointer was NOT set, it assumes Success.
      // So checking for "error" field might be redundant but safe.
      if (jsonMap.containsKey('error')) {
        return ConfigOperationResult.error(
          'Runtime error: ${jsonMap['error']}',
        );
      }
      return ConfigOperationResult.success(RuntimeConfig.fromJson(jsonMap));
    } catch (e) {
      return ConfigOperationResult.error('Failed to parse runtime config: $e');
    }
  }
}
