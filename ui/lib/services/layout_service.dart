/// Service for managing Virtual Layouts via config FFI.
library;

import 'dart:convert';
import 'dart:ffi';

import 'package:flutter/foundation.dart';
import 'package:ffi/ffi.dart';

import '../ffi/bridge.dart';
import '../ffi/bindings.dart';
import '../models/virtual_layout.dart';
import 'config_result.dart';

class LayoutService with ChangeNotifier {
  LayoutService({required KeyrxBridge bridge}) : _bridge = bridge;

  final KeyrxBridge _bridge;

  /// List all virtual layouts stored in the config directory.
  Future<ConfigOperationResult<List<VirtualLayout>>> listLayouts() async {
    return _guard('list layouts', (bindings) {
      final errorPtr = calloc<Pointer<Utf8>>();
      Pointer<Char>? ptr;
      try {
        ptr = bindings.configListVirtualLayouts(errorPtr);

        if (errorPtr.value.address != 0) {
          final error = errorPtr.value.toDartString();
          bindings.freeString(errorPtr.value.cast<Char>());
          return ConfigOperationResult.error(error);
        }

        if (ptr == nullptr) {
          return ConfigOperationResult.error(
            'configListVirtualLayouts returned null',
          );
        }

        final raw = ptr?.cast<Utf8>().toDartString();
        return parseConfigFfiResult<List<VirtualLayout>>(raw!, (json) {
          final list = json as List<dynamic>;
          return list
              .map(
                (item) => VirtualLayout.fromJson(item as Map<String, dynamic>),
              )
              .toList();
        });
      } catch (e) {
        return ConfigOperationResult.error('list layouts failed: $e');
      } finally {
        if (ptr != null && ptr != nullptr) {
          bindings.freeString(ptr);
        }
        calloc.free(errorPtr);
      }
    });
  }

  /// Persist a virtual layout definition and return the saved value.
  Future<ConfigOperationResult<VirtualLayout>> saveLayout(
    VirtualLayout layout,
  ) async {
    final result = await _guard<VirtualLayout>('save layout', (bindings) {
      final errorPtr = calloc<Pointer<Utf8>>();
      Pointer<Utf8>? jsonPtr;
      Pointer<Char>? resultPtr;
      try {
        final jsonStr = json.encode(layout.toJson());
        jsonPtr = jsonStr.toNativeUtf8();
        resultPtr = bindings.configSaveVirtualLayout(
          jsonPtr.cast<Char>(),
          errorPtr,
        );

        if (errorPtr.value.address != 0) {
          final error = errorPtr.value.toDartString();
          bindings.freeString(errorPtr.value.cast<Char>());
          return ConfigOperationResult.error(error);
        }

        if (resultPtr == nullptr) {
          return ConfigOperationResult.error(
            'configSaveVirtualLayout returned null',
          );
        }

        final raw = resultPtr?.cast<Utf8>().toDartString();
        return parseConfigFfiResult<VirtualLayout>(
          raw!,
          (json) => VirtualLayout.fromJson(json as Map<String, dynamic>),
        );
      } catch (e) {
        return ConfigOperationResult.error('save layout failed: $e');
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

    if (!result.hasError) {
      notifyListeners();
    }
    return result;
  }

  /// Delete a virtual layout by id.
  Future<ConfigOperationResult<void>> deleteLayout(String id) async {
    final result = await _guard<void>('delete layout', (bindings) {
      final errorPtr = calloc<Pointer<Utf8>>();
      final idPtr = id.toNativeUtf8();
      Pointer<Char>? resultPtr;

      try {
        resultPtr = bindings.configDeleteVirtualLayout(
          idPtr.cast<Char>(),
          errorPtr,
        );

        if (errorPtr.value.address != 0) {
          final error = errorPtr.value.toDartString();
          bindings.freeString(errorPtr.value.cast<Char>());
          return ConfigOperationResult.error(error);
        }

        if (resultPtr == nullptr) {
          return ConfigOperationResult.error(
            'configDeleteVirtualLayout returned null',
          );
        }

        final raw = resultPtr?.cast<Utf8>().toDartString();
        return parseConfigFfiResult<void>(raw!, null);
      } catch (e) {
        return ConfigOperationResult.error('delete layout failed: $e');
      } finally {
        calloc.free(idPtr);
        if (resultPtr != null && resultPtr != nullptr) {
          bindings.freeString(resultPtr);
        }
        calloc.free(errorPtr);
      }
    });

    if (!result.hasError) {
      notifyListeners();
    }
    return result;
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
