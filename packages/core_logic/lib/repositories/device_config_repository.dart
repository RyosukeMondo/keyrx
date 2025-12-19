import 'dart:convert';
import 'dart:io';
import 'package:path/path.dart' as p;
import '../models/device_config.dart';

/// Repository for persisting device configuration to the file system.
///
/// This repository handles loading and saving [DeviceConfig] to a JSON file,
/// following platform-specific standards for configuration storage.
///
/// ## Concurrent Access
/// This repository uses atomic write patterns (write to temp file, then rename)
/// to prevent file corruption. However, it does NOT implement file locking.
/// If multiple processes (e.g., UI + CLI) write simultaneously, the last write
/// wins. This is acceptable for the expected usage pattern where concurrent
/// writes are rare. Reads are always safe due to atomic writes.
class DeviceConfigRepository {
  /// Create a new [DeviceConfigRepository].
  ///
  /// If [configDir] is not provided, it falls back to platform-standard
  /// locations (e.g., ~/.config/keyrx on Linux).
  DeviceConfigRepository({String? configDir})
      : _configDir = configDir ?? _defaultConfigDir();

  final String _configDir;
  static const String _fileName = 'devices.json';

  /// Absolute path to the configuration file.
  String get filePath => p.join(_configDir, _fileName);

  /// Load the device configuration from disk.
  ///
  /// Returns a new empty [DeviceConfig] if the file does not exist or is corrupt.
  /// If the file is corrupt, a backup is created before returning empty.
  Future<DeviceConfig> load() async {
    final file = File(filePath);
    if (!await file.exists()) {
      return const DeviceConfig();
    }

    try {
      final content = await file.readAsString();
      if (content.isEmpty) {
        return const DeviceConfig();
      }
      final json = jsonDecode(content) as Map<String, dynamic>;
      return DeviceConfig.fromJson(json);
    } catch (e) {
      // Create backup on corruption
      try {
        final backupPath = '$filePath.${DateTime.now().millisecondsSinceEpoch}.bak';
        await file.copy(backupPath);
      } catch (_) {
        // Best effort backup
      }
      return const DeviceConfig();
    }
  }

  /// Save the device configuration to disk using an atomic write pattern.
  ///
  /// This ensures that the file is not left in a half-written state if
  /// a crash or power failure occurs during the write operation.
  Future<void> save(DeviceConfig config) async {
    final directory = Directory(_configDir);
    if (!await directory.exists()) {
      await directory.create(recursive: true);
    }

    final json = const JsonEncoder.withIndent('  ').convert(config.toJson());
    final tempFile = File('$filePath.tmp');
    final finalFile = File(filePath);

    // Atomic write: write to temp file then rename
    await tempFile.writeAsString(json);
    await tempFile.rename(finalFile.path);
  }

  /// Delete the configuration file (primarily for tests).
  Future<void> delete() async {
    final file = File(filePath);
    if (await file.exists()) {
      await file.delete();
    }
  }

  static String _defaultConfigDir() {
    // Replicate Rust's config_dir() logic for consistency
    final xdgConfigHome = Platform.environment['XDG_CONFIG_HOME'];
    if (xdgConfigHome != null && xdgConfigHome.isNotEmpty) {
      return p.join(xdgConfigHome, 'keyrx');
    }

    if (Platform.isLinux) {
      final home = Platform.environment['HOME'];
      if (home != null && home.isNotEmpty) {
        return p.join(home, '.config', 'keyrx');
      }
    } else if (Platform.isWindows) {
      final appData = Platform.environment['APPDATA'];
      if (appData != null && appData.isNotEmpty) {
        return p.join(appData, 'keyrx');
      }
    } else if (Platform.isMacOS) {
      final home = Platform.environment['HOME'];
      if (home != null && home.isNotEmpty) {
        return p.join(home, 'Library', 'Application Support', 'keyrx');
      }
    }

    // Last resort fallback
    return p.join('.', '.config', 'keyrx');
  }
}
