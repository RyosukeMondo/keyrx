/// Service for parsing and classifying console output.
///
/// Extracts parsing logic from UI components for testability.
library;

/// Type of console entry based on content classification.
enum ConsoleEntryType {
  /// User input command.
  command,

  /// Error output (starts with "error:").
  error,

  /// Success output (starts with "ok:").
  ok,

  /// General output.
  output,
}

/// Parses and classifies Rhai console output.
///
/// Pure parsing logic with no UI dependencies, making it testable in isolation.
class ConsoleParser {
  const ConsoleParser();

  /// Classifies text into a console entry type.
  ///
  /// Returns [ConsoleEntryType.command] for user input,
  /// [ConsoleEntryType.error] for text starting with "error:",
  /// [ConsoleEntryType.ok] for text starting with "ok:",
  /// [ConsoleEntryType.output] otherwise.
  ConsoleEntryType classify(String text, {required bool isInput}) {
    if (isInput) {
      return ConsoleEntryType.command;
    }

    final lower = text.toLowerCase();
    if (lower.startsWith('error:')) {
      return ConsoleEntryType.error;
    }
    if (lower.startsWith('ok:')) {
      return ConsoleEntryType.ok;
    }
    return ConsoleEntryType.output;
  }

  /// Checks if text indicates the engine needs initialization.
  ///
  /// Returns true if the text is an error containing "not initialized".
  bool needsInitButton(String text, {required bool isError}) {
    if (!isError) return false;
    final lower = text.toLowerCase();
    return lower.contains('not initialized');
  }

  /// Strips "ok:" or "error:" prefix from text.
  ///
  /// Returns the text without the prefix, trimmed of leading whitespace.
  String stripPrefix(String text) {
    final lower = text.toLowerCase();
    if (lower.startsWith('ok:') || lower.startsWith('error:')) {
      final idx = text.indexOf(':');
      return idx > -1 ? text.substring(idx + 1).trimLeft() : text;
    }
    return text;
  }
}
