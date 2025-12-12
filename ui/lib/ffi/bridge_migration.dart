/// Migration FFI methods.
///
/// Provides access to migration functionality for upgrading profiles from V1 to V2.
library;

import 'dart:convert';
import 'dart:ffi';

import 'package:ffi/ffi.dart';
import 'package:flutter/foundation.dart';

import '../pages/migration_prompt_page.dart';
import 'bindings.dart';

/// Mixin providing migration FFI methods.
mixin BridgeMigrationMixin {
  KeyrxBindings? get bindings;
  Object? get loadFailure;

  /// Check if migration is needed.
  ///
  /// Checks if the old profiles directory exists and contains V1 profiles.
  ///
  /// [oldProfilesDir] - Path to directory containing old V1 profiles.
  Future<bool> checkMigrationNeeded(String oldProfilesDir) async {
    final checkFn = bindings?.migrationCheckNeeded;
    if (checkFn == null) {
      debugPrint('migrationCheckNeeded not available');
      return false;
    }

    final oldDirPtr = oldProfilesDir.toNativeUtf8();
    Pointer<Char>? ptr;

    try {
      // Execute on a separate thread/event loop implicitly via FFI if possible?
      // No, these are blocking calls. We rely on them being fast or wrapping in Future.
      // Since file I/O is involved, ideally this should be async in Rust, but here we block.
      // However, check is usually fast (just checking dir existence).
      
      ptr = checkFn(oldDirPtr.cast<Char>());
      
      if (ptr == nullptr) {
        return false;
      }

      final raw = ptr.cast<Utf8>().toDartString();
      final trimmed = raw.trim();

      if (trimmed.toLowerCase().startsWith('ok:')) {
        final payload = trimmed.substring(trimmed.indexOf(':') + 1).trim();
        return payload.toLowerCase() == 'true';
      } else if (trimmed.toLowerCase().startsWith('error:')) {
        debugPrint('Migration check error: $trimmed');
        return false;
      }

      return false;
    } catch (e) {
      debugPrint('Migration check exception: $e');
      return false;
    } finally {
      calloc.free(oldDirPtr);
      if (ptr != null && ptr != nullptr) {
        try {
          bindings?.freeString(ptr);
        } catch (_) {}
      }
    }
  }

  /// Run migration from V1 to V2.
  ///
  /// [oldProfilesDir] - Path to directory containing old V1 profiles.
  /// [newProfilesDir] - Path to directory where new V2 profiles should be stored.
  /// [createBackup] - Whether to create a backup of old profiles (default: true).
  Future<MigrationReport> runMigration(
    String oldProfilesDir,
    String newProfilesDir, {
    bool createBackup = true,
  }) async {
    final runFn = bindings?.migrationRun;
    if (runFn == null) {
      throw Exception('migrationRun FFI function not available');
    }

    final oldDirPtr = oldProfilesDir.toNativeUtf8();
    final newDirPtr = newProfilesDir.toNativeUtf8();
    Pointer<Char>? ptr;

    try {
      ptr = runFn(
        oldDirPtr.cast<Char>(),
        newDirPtr.cast<Char>(),
        createBackup ? 1 : 0,
      );

      if (ptr == nullptr) {
        throw Exception('Migration returned null pointer');
      }

      final raw = ptr.cast<Utf8>().toDartString();
      final trimmed = raw.trim();

      if (trimmed.toLowerCase().startsWith('error:')) {
        throw Exception(trimmed.substring(6).trim());
      } else if (trimmed.toLowerCase().startsWith('ok:')) {
        final jsonStr = trimmed.substring(3).trim();
        final jsonMap = json.decode(jsonStr) as Map<String, dynamic>;
        return MigrationReport.fromJson(jsonMap);
      } else {
        throw Exception('Unexpected response format: $trimmed');
      }
    } catch (e) {
      // Re-throw to be handled by the UI
      rethrow;
    } finally {
      calloc.free(oldDirPtr);
      calloc.free(newDirPtr);
      if (ptr != null && ptr != nullptr) {
        try {
          bindings?.freeString(ptr);
        } catch (_) {}
      }
    }
  }
}
