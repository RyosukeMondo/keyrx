/// Service for managing Keymaps via config FFI.
library;

import 'dart:convert';
import 'dart:ffi';

import 'package:ffi/ffi.dart';

import '../ffi/bridge.dart';
import '../ffi/bindings.dart';
import '../models/keymap.dart';
import 'config_result.dart';

class KeymapService {
  KeymapService({required KeyrxBridge bridge}) : _bridge = bridge;

  final KeyrxBridge _bridge;

  /// List all keymaps.
  Future<ConfigOperationResult<List<Keymap>>> listKeymaps() async {
    return _guard('list keymaps', (bindings) {
      final errorPtr = calloc<Pointer<Utf8>>();
      Pointer<Char>? ptr;
      try {
        ptr = bindings.configListKeymaps(errorPtr);

        if (errorPtr.value.address != 0) {
          final error = errorPtr.value.toDartString();
          bindings.freeString(errorPtr.value.cast<Char>());
          return ConfigOperationResult.error(error);
        }

        if (ptr == nullptr) {
          return ConfigOperationResult.error('configListKeymaps returned null');
        }

        final raw = ptr?.cast<Utf8>().toDartString();
        return parseConfigFfiResult<List<Keymap>>(raw!, (json) {
          final list = json as List<dynamic>;
          return list
              .map((item) => Keymap.fromJson(item as Map<String, dynamic>))
              .toList();
        });
      } catch (e) {
        return ConfigOperationResult.error('list keymaps failed: $e');
      } finally {
        if (ptr != null && ptr != nullptr) {
          bindings.freeString(ptr);
        }
        calloc.free(errorPtr);
      }
    });
  }

  /// Persist or update a keymap definition.
  Future<ConfigOperationResult<Keymap>> saveKeymap(Keymap keymap) async {
    return _guard('save keymap', (bindings) {
      final errorPtr = calloc<Pointer<Utf8>>();
      Pointer<Utf8>? jsonPtr;
      Pointer<Char>? resultPtr;
      try {
        final jsonStr = json.encode(keymap.toJson());
        jsonPtr = jsonStr.toNativeUtf8();
        resultPtr = bindings.configSaveKeymap(jsonPtr.cast<Char>(), errorPtr);

        if (errorPtr.value.address != 0) {
          final error = errorPtr.value.toDartString();
          bindings.freeString(errorPtr.value.cast<Char>());
          return ConfigOperationResult.error(error);
        }

        if (resultPtr == nullptr) {
          return ConfigOperationResult.error('configSaveKeymap returned null');
        }

        final raw = resultPtr?.cast<Utf8>().toDartString();
        return parseConfigFfiResult<Keymap>(
          raw!,
          (json) => Keymap.fromJson(json as Map<String, dynamic>),
        );
      } catch (e) {
        return ConfigOperationResult.error('save keymap failed: $e');
      } finally {
        if (jsonPtr != null) {
          calloc.free(jsonPtr);
        }
        if (resultPtr != null && resultPtr != nullptr) {
          bindings.freeString(resultPtr);
        }
        calloc.free(errorPtr);
      }
    });
  }

  /// Delete a keymap by id.
  Future<ConfigOperationResult<void>> deleteKeymap(String id) async {
    return _guard('delete keymap', (bindings) {
      final errorPtr = calloc<Pointer<Utf8>>();
      final idPtr = id.toNativeUtf8();
      Pointer<Char>? resultPtr;

      try {
        resultPtr = bindings.configDeleteKeymap(idPtr.cast<Char>(), errorPtr);

        if (errorPtr.value.address != 0) {
          final error = errorPtr.value.toDartString();
          bindings.freeString(errorPtr.value.cast<Char>());
          return ConfigOperationResult.error(error);
        }

        if (resultPtr == nullptr) {
          return ConfigOperationResult.error(
            'configDeleteKeymap returned null',
          );
        }

        final raw = resultPtr?.cast<Utf8>().toDartString();
        return parseConfigFfiResult<void>(raw!, null);
      } catch (e) {
        return ConfigOperationResult.error('delete keymap failed: $e');
      } finally {
        calloc.free(idPtr);
        if (resultPtr != null && resultPtr != nullptr) {
          bindings.freeString(resultPtr);
        }
        calloc.free(errorPtr);
      }
    });
  }

  Future<ConfigOperationResult<T>> _guard<T>(
    String operation,
    ConfigOperationResult<T> Function(KeyrxBindings bindings) action,
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
}
