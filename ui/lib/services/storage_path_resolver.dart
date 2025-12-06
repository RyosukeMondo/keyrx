/// Resolves storage paths for user-specific KeyRx data.
///
/// Provides a cross-platform way to locate the profiles directory under the
/// user's home and ensures it exists before use.
library;

import 'dart:io';

import 'package:path/path.dart' as p;

/// Resolves and prepares storage paths for profiles.
class StoragePathResolver {
  /// Create a resolver using the provided environment (useful for tests) or
  /// the current process environment by default.
  const StoragePathResolver({Map<String, String>? environment})
    : _environment = environment;

  final Map<String, String>? _environment;

  Map<String, String> get _env => _environment ?? Platform.environment;

  /// Returns the resolved profiles directory path (e.g. `~/.keyrx` or
  /// `%USERPROFILE%/.keyrx`), without creating it.
  String resolveProfilesPath() {
    final home = _resolveHomeDirectory();
    return p.join(home, '.keyrx');
  }

  /// Ensures the profiles directory exists and returns its absolute path.
  Future<String> ensureProfilesDirectory() async {
    final path = resolveProfilesPath();
    final directory = Directory(path);
    await directory.create(recursive: true);
    return directory.path;
  }

  String _resolveHomeDirectory() {
    final env = _env;
    final home = Platform.isWindows
        ? env['USERPROFILE'] ?? env['HOME']
        : env['HOME'] ?? env['USERPROFILE'];

    if (home == null || home.isEmpty) {
      throw StateError('Unable to determine user home directory.');
    }

    return home;
  }
}
