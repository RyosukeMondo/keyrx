/// Migration FFI methods.
///
/// Provides access to migration functionality for upgrading profiles from V1 to V2.
library;

import 'dart:convert';

import 'package:meta/meta.dart';

import '../models/migration_report.dart';
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
    if (bindings == null) {
      print('migrationCheckNeeded not available');
      return false;
    }

    try {
      final raw = bindings!.migration.checkNeeded(oldProfilesDir);
      final trimmed = raw.trim();

      if (trimmed.toLowerCase().startsWith('ok:')) {
        final payload = trimmed.substring(trimmed.indexOf(':') + 1).trim();
        return payload.toLowerCase() == 'true';
      } else if (trimmed.toLowerCase().startsWith('error:')) {
        print('Migration check error: $trimmed');
        return false;
      }

      return false;
    } catch (e) {
      print('Migration check exception: $e');
      return false;
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
    if (bindings == null) {
      throw Exception('migrationRun FFI function not available');
    }

    try {
      final raw = bindings!.migration.run(
        oldProfilesDir,
        newProfilesDir,
        createBackup ? 1 : 0,
      );
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
      rethrow;
    }
  }
}
