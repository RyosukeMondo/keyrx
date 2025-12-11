/// Service for managing runtime profile slots via FFI.
library;

import 'dart:convert';
import 'dart:ffi';

import 'package:ffi/ffi.dart';

import '../ffi/bindings.dart';
import '../ffi/bridge.dart';
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
      Pointer<Char>? ptr;
      try {
        ptr = bindings.runtimeGetConfig();
        if (ptr == nullptr) {
          return ConfigOperationResult.error('runtimeGetConfig returned null');
        }
        final raw = ptr.cast<Utf8>().toDartString();
        return _parseRuntime(raw);
      } catch (e) {
        return ConfigOperationResult.error('get runtime config failed: $e');
      } finally {
        if (ptr != null && ptr != nullptr) {
          bindings.freeString(ptr);
        }
      }
    });
  }

  @override
  Future<ConfigOperationResult<RuntimeConfig>> addSlot(
    DeviceInstanceId device,
    ProfileSlot slot,
  ) async {
    return _guard('add slot', (bindings) {
      Pointer<Utf8>? devicePtr;
      Pointer<Utf8>? slotPtr;
      Pointer<Char>? resultPtr;

      try {
        devicePtr = json.encode(device.toJson()).toNativeUtf8();
        slotPtr = json.encode(slot.toJson()).toNativeUtf8();
        resultPtr = bindings.runtimeAddSlot(
          devicePtr.cast<Char>(),
          slotPtr.cast<Char>(),
          0,
        );
        if (resultPtr == nullptr) {
          return ConfigOperationResult.error('runtimeAddSlot returned null');
        }
        final raw = resultPtr.cast<Utf8>().toDartString();
        return _parseRuntime(raw);
      } catch (e) {
        return ConfigOperationResult.error('add slot failed: $e');
      } finally {
        if (devicePtr != null) {
          calloc.free(devicePtr);
        }
        if (slotPtr != null) {
          calloc.free(slotPtr);
        }
        if (resultPtr != null && resultPtr != nullptr) {
          bindings.freeString(resultPtr);
        }
      }
    });
  }

  @override
  Future<ConfigOperationResult<RuntimeConfig>> removeSlot(
    DeviceInstanceId device,
    String slotId,
  ) async {
    return _guard('remove slot', (bindings) {
      Pointer<Utf8>? devicePtr;
      Pointer<Utf8>? slotPtr;
      Pointer<Char>? resultPtr;

      try {
        devicePtr = json.encode(device.toJson()).toNativeUtf8();
        slotPtr = slotId.toNativeUtf8();
        resultPtr = bindings.runtimeRemoveSlot(
          devicePtr.cast<Char>(),
          slotPtr.cast<Char>(),
        );
        if (resultPtr == nullptr) {
          return ConfigOperationResult.error('runtimeRemoveSlot returned null');
        }
        final raw = resultPtr.cast<Utf8>().toDartString();
        return _parseRuntime(raw);
      } catch (e) {
        return ConfigOperationResult.error('remove slot failed: $e');
      } finally {
        if (devicePtr != null) {
          calloc.free(devicePtr);
        }
        if (slotPtr != null) {
          calloc.free(slotPtr);
        }
        if (resultPtr != null && resultPtr != nullptr) {
          bindings.freeString(resultPtr);
        }
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
      Pointer<Utf8>? devicePtr;
      Pointer<Utf8>? slotPtr;
      Pointer<Char>? resultPtr;

      try {
        devicePtr = json.encode(device.toJson()).toNativeUtf8();
        slotPtr = slotId.toNativeUtf8();
        resultPtr = bindings.runtimeReorderSlot(
          devicePtr.cast<Char>(),
          slotPtr.cast<Char>(),
          priority,
        );
        if (resultPtr == nullptr) {
          return ConfigOperationResult.error(
            'runtimeReorderSlot returned null',
          );
        }
        final raw = resultPtr.cast<Utf8>().toDartString();
        return _parseRuntime(raw);
      } catch (e) {
        return ConfigOperationResult.error('reorder slot failed: $e');
      } finally {
        if (devicePtr != null) {
          calloc.free(devicePtr);
        }
        if (slotPtr != null) {
          calloc.free(slotPtr);
        }
        if (resultPtr != null && resultPtr != nullptr) {
          bindings.freeString(resultPtr);
        }
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
      Pointer<Utf8>? devicePtr;
      Pointer<Utf8>? slotPtr;
      Pointer<Char>? resultPtr;

      try {
        devicePtr = json.encode(device.toJson()).toNativeUtf8();
        slotPtr = slotId.toNativeUtf8();
        resultPtr = bindings.runtimeSetSlotActive(
          devicePtr.cast<Char>(),
          slotPtr.cast<Char>(),
          active,
        );
        if (resultPtr == nullptr) {
          return ConfigOperationResult.error(
            'runtimeSetSlotActive returned null',
          );
        }
        final raw = resultPtr.cast<Utf8>().toDartString();
        return _parseRuntime(raw);
      } catch (e) {
        return ConfigOperationResult.error('set slot active failed: $e');
      } finally {
        if (devicePtr != null) {
          calloc.free(devicePtr);
        }
        if (slotPtr != null) {
          calloc.free(slotPtr);
        }
        if (resultPtr != null && resultPtr != nullptr) {
          bindings.freeString(resultPtr);
        }
      }
    });
  }

  Future<ConfigOperationResult<RuntimeConfig>> _guard(
    String operation,
    ConfigOperationResult<RuntimeConfig> Function(KeyrxBindings bindings)
    action,
  ) async {
    final loadFailure = _bridge.loadFailure;
    if (loadFailure != null) {
      return ConfigOperationResult.error(
        'Engine unavailable ($operation): $loadFailure',
      );
    }

    final bindings = _bridge.bindings;
    if (bindings == null) {
      return ConfigOperationResult.error(
        'FFI bindings unavailable ($operation)',
      );
    }

    return action(bindings);
  }

  ConfigOperationResult<RuntimeConfig> _parseRuntime(String raw) {
    return parseConfigFfiResult<RuntimeConfig>(
      raw,
      (json) => RuntimeConfig.fromJson(json as Map<String, dynamic>),
    );
  }
}
