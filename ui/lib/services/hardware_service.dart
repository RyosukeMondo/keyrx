/// Service for managing Hardware Profiles via config FFI.
library;

import 'dart:convert';
import 'dart:ffi';

import 'package:flutter/foundation.dart';
import 'package:ffi/ffi.dart';

import '../ffi/bridge.dart';
import '../ffi/bindings.dart';
import '../models/hardware_profile.dart';
import 'config_result.dart';

class HardwareService with ChangeNotifier {
  HardwareService({required KeyrxBridge bridge}) : _bridge = bridge;

  final KeyrxBridge _bridge;

  /// List all hardware profiles.
  Future<ConfigOperationResult<List<HardwareProfile>>> listProfiles() async {
    return _guard('list hardware profiles', (bindings) {
      Pointer<Char>? ptr;
      try {
        ptr = bindings.configListHardwareProfiles();
        if (ptr == nullptr) {
          return ConfigOperationResult.error(
            'configListHardwareProfiles returned null',
          );
        }

        final raw = ptr.cast<Utf8>().toDartString();
        return parseConfigFfiResult<List<HardwareProfile>>(raw, (json) {
          final list = json as List<dynamic>;
          return list
              .map(
                (item) =>
                    HardwareProfile.fromJson(item as Map<String, dynamic>),
              )
              .toList();
        });
      } catch (e) {
        return ConfigOperationResult.error('list hardware profiles failed: $e');
      } finally {
        if (ptr != null && ptr != nullptr) {
          bindings.freeString(ptr);
        }
      }
    });
  }

  /// Persist or update a hardware profile.
  Future<ConfigOperationResult<HardwareProfile>> saveProfile(
    HardwareProfile profile,
  ) async {
    final result = await _guard<HardwareProfile>('save hardware profile', (
      bindings,
    ) {
      Pointer<Utf8>? jsonPtr;
      Pointer<Char>? resultPtr;
      try {
        final jsonStr = json.encode(profile.toJson());
        jsonPtr = jsonStr.toNativeUtf8();
        resultPtr = bindings.configSaveHardwareProfile(jsonPtr.cast<Char>());
        if (resultPtr == nullptr) {
          return ConfigOperationResult.error(
            'configSaveHardwareProfile returned null',
          );
        }

        final raw = resultPtr.cast<Utf8>().toDartString();
        return parseConfigFfiResult<HardwareProfile>(
          raw,
          (json) => HardwareProfile.fromJson(json as Map<String, dynamic>),
        );
      } catch (e) {
        return ConfigOperationResult.error('save hardware profile failed: $e');
      } finally {
        if (jsonPtr != null) {
          calloc.free(jsonPtr);
        }
        if (resultPtr != null && resultPtr != nullptr) {
          bindings.freeString(resultPtr);
        }
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
      final idPtr = id.toNativeUtf8();
      Pointer<Char>? resultPtr;

      try {
        resultPtr = bindings.configDeleteHardwareProfile(idPtr.cast<Char>());
        if (resultPtr == nullptr) {
          return ConfigOperationResult.error(
            'configDeleteHardwareProfile returned null',
          );
        }

        final raw = resultPtr.cast<Utf8>().toDartString();
        return parseConfigFfiResult<void>(raw, null);
      } catch (e) {
        return ConfigOperationResult.error(
          'delete hardware profile failed: $e',
        );
      } finally {
        calloc.free(idPtr);
        if (resultPtr != null && resultPtr != nullptr) {
          bindings.freeString(resultPtr);
        }
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
