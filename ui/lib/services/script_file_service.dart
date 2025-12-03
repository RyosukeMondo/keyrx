/// Service for script file I/O operations.
///
/// Handles reading and writing Rhai script files, isolated from UI concerns.
library;

import 'dart:io';

/// Result of a script file operation.
class ScriptFileResult {
  const ScriptFileResult({
    required this.success,
    this.errorMessage,
  });

  /// Whether the operation succeeded.
  final bool success;

  /// Error message if operation failed.
  final String? errorMessage;
}

/// Handles file I/O for Rhai scripts.
///
/// Pure file operations with no UI dependencies, making it testable in isolation.
class ScriptFileService {
  const ScriptFileService();

  /// Saves script content to the specified path.
  ///
  /// Creates parent directories if they don't exist.
  /// Returns a [ScriptFileResult] indicating success or failure.
  Future<ScriptFileResult> saveScript(String path, String content) async {
    try {
      final file = File(path);
      await file.parent.create(recursive: true);
      await file.writeAsString(content);
      return const ScriptFileResult(success: true);
    } catch (e) {
      return ScriptFileResult(
        success: false,
        errorMessage: e.toString(),
      );
    }
  }

  /// Loads script content from the specified path.
  ///
  /// Returns the script content or null if the file doesn't exist or can't be read.
  Future<String?> loadScript(String path) async {
    try {
      final file = File(path);
      if (!await file.exists()) {
        return null;
      }
      return await file.readAsString();
    } catch (_) {
      return null;
    }
  }
}
