import 'dart:ffi';
import 'dart:io';
import 'package:path/path.dart' as path;

/// Logic to load the native KeyRx library.
///
/// Handles differences between:
/// - Flutter App (Linux/Windows)
/// - Dart CLI (JIT/AOT)
/// - Development vs Release
class LibraryLoader {
  static const String _linuxLibName = 'libkeyrx_core.so';
  static const String _windowsLibName = 'keyrx_core.dll';

  /// Load the library.
  static DynamicLibrary load() {
    if (Platform.isLinux) {
      return _loadLinux();
    } else if (Platform.isWindows) {
      return _loadWindows();
    } else {
      throw UnsupportedError(
        'Platform not supported: ${Platform.operatingSystem}',
      );
    }
  }

  static DynamicLibrary _loadLinux() {
    // 1. Try standard dlopen (works for Flutter AOT/snap/installed)
    try {
      return DynamicLibrary.open(_linuxLibName);
    } catch (_) {
      // Continue to search
    }

    // 2. Search common development paths (relative to script)
    // This handles running `dart run` from various locations in the repo.
    final scriptDir = path.dirname(Platform.script.toFilePath());
    final searchPaths = [
       // If running from packages/core_logic/test/
      path.join(scriptDir, '../../../target/debug', _linuxLibName),
      path.join(scriptDir, '../../../target/release', _linuxLibName),
      // If running from root/
      path.join(scriptDir, 'target/debug', _linuxLibName),
      path.join(scriptDir, 'target/release', _linuxLibName),
       // If running from apps/cli_tool/bin/
      path.join(scriptDir, '../../../target/debug', _linuxLibName),
      path.join(scriptDir, '../../../target/release', _linuxLibName),
    ];

    for (final p in searchPaths) {
      if (File(p).existsSync()) {
        try {
          return DynamicLibrary.open(p);
        } catch (e) {
          print('Found library at $p but failed to load: $e');
        }
      }
    }

    // 3. Last resort: current directory
    try {
      return DynamicLibrary.open('./$_linuxLibName');
    } catch (_) {}

    throw Exception(
      'Could not load $_linuxLibName. Checked system path and: ${searchPaths.join(", ")}',
    );
  }

  static DynamicLibrary _loadWindows() {
    // Similar logic for Windows could be added here.
    // For now, rely on standard loading or specific dev paths.
     try {
      return DynamicLibrary.open(_windowsLibName);
    } catch (_) {
      // TODO: Add Windows dev paths if needed
      throw Exception('Could not load $_windowsLibName');
    }
  }
}
