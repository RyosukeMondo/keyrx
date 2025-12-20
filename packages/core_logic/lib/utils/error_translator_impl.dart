import 'dart:async';
import 'dart:io';

import '../ffi/debug_tools.dart';
import 'error_translator.dart';

/// Structured entry describing an error code, user-facing message, and action.
class ErrorRegistryEntry {
  const ErrorRegistryEntry({
    required this.code,
    required this.title,
    required this.message,
    required this.action,
    required this.docsSection,
    this.category = MessageCategory.error,
  });

  final String code;
  final String title;
  final String message;
  final String action;
  final String docsSection;
  final MessageCategory category;

  /// Clone entry with contextual message or action overrides.
  ErrorRegistryEntry withContext({
    String? messageOverride,
    String? actionOverride,
  }) => ErrorRegistryEntry(
    code: code,
    title: title,
    message: messageOverride ?? message,
    action: actionOverride ?? action,
    docsSection: docsSection,
    category: category,
  );

  /// Render a UI-friendly message string including next steps and docs link.
  String body({required String docsPath, String? details}) {
    final buffer = StringBuffer()
      ..write(message)
      ..write(' Next step: ')
      ..write(action);

    if (details != null && details.isNotEmpty && details != message) {
      buffer
        ..write(' Details: ')
        ..write(details);
    }

    buffer
      ..write(' See ')
      ..write(docsPath)
      ..write('#')
      ..write(docsSection)
      ..write(' for troubleshooting.');

    return buffer.toString();
  }

  /// Build a UserMessage for UI surfaces.
  UserMessage toUserMessage({required String docsPath, String? details}) =>
      UserMessage(
        title: '[$code] $title',
        body: body(docsPath: docsPath, details: details),
        category: category,
      );
}

/// Centralized error handler that maps common errors to actionable messages.
class ErrorHandler implements ErrorTranslator {
  const ErrorHandler({this.docsPath = 'docs/ERROR_HANDLING.md'});

  final String docsPath;

  @override
  UserMessage translate(Object error) {
    final entry = _resolve(error);
    return entry.toUserMessage(docsPath: docsPath, details: _details(error));
  }

  ErrorRegistryEntry _resolve(Object error) {
    if (error is FfiErrorDetails) {
      return _fromFfiDetails(error);
    }

    if (error is TimeoutException) {
      return _registry['E4001']!;
    }

    if (error is StateError) {
      return _registry['E2001']!.withContext(messageOverride: error.toString());
    }

    if (error is UnsupportedError) {
      return _registry['E2002']!;
    }

    if (error is ArgumentError ||
        error is FormatException ||
        error is RangeError) {
      return _registry['E3001']!.withContext(messageOverride: error.toString());
    }

    if (error is FileSystemException) {
      return _registry['E3002']!.withContext(messageOverride: error.message);
    }

    final raw = error.toString();
    if (raw.contains('FFI')) {
      return _registry['E1000']!.withContext(messageOverride: raw);
    }

    return _registry['E9000']!.withContext(messageOverride: raw);
  }

  ErrorRegistryEntry _fromFfiDetails(FfiErrorDetails details) {
    final entry = switch (details.code) {
      'FFI_LIBRARY_LOAD' => _registry['E1001']!,
      'FFI_ARGUMENT' || 'FFI_NULL_POINTER' => _registry['E1002']!,
      'FFI_STATE' => _registry['E2001']!,
      _ => _registry['E1000']!,
    };

    return entry.withContext(
      messageOverride: details.message,
      actionOverride: details.hint,
    );
  }

  String _details(Object error) {
    if (error is FfiErrorDetails) {
      final hint = error.hint != null ? ' Hint: ${error.hint}' : '';
      return '${error.code}: ${error.message}.$hint';
    }
    if (error is TimeoutException && error.duration != null) {
      return 'Operation exceeded ${error.duration!.inSeconds}s';
    }
    return error.toString();
  }
}

const Map<String, ErrorRegistryEntry> _registry = {
  'E1000': ErrorRegistryEntry(
    code: 'E1000',
    title: 'FFI Failure',
    message: 'Native integration reported an unexpected failure.',
    action: 'Enable FFI tracing and retry the operation.',
    docsSection: 'ffi-errors',
  ),
  'E1001': ErrorRegistryEntry(
    code: 'E1001',
    title: 'Native Library Missing',
    message: 'KeyRx native library failed to load.',
    action:
        'Rebuild bindings and ensure the shared library is on PATH/LD_LIBRARY_PATH.',
    docsSection: 'ffi-errors',
  ),
  'E1002': ErrorRegistryEntry(
    code: 'E1002',
    title: 'FFI Argument Issue',
    message: 'Invalid data was passed to the native layer.',
    action:
        'Validate pointers, buffer lengths, and nullability before calling FFI.',
    docsSection: 'ffi-errors',
  ),
  'E2001': ErrorRegistryEntry(
    code: 'E2001',
    title: 'Engine Not Ready',
    message: 'The engine or bridge is not initialized.',
    action:
        'Run engine initialization and wait for ready status before retrying.',
    docsSection: 'engine-state',
  ),
  'E2002': ErrorRegistryEntry(
    code: 'E2002',
    title: 'Unsupported Operation',
    message: 'The requested action is not supported in the current mode.',
    action:
        'Switch to a supported workflow or disable the incompatible setting.',
    docsSection: 'engine-state',
  ),
  'E3001': ErrorRegistryEntry(
    code: 'E3001',
    title: 'Validation Error',
    message: 'Input validation failed.',
    action: 'Check required fields, formats, and allowed ranges.',
    docsSection: 'validation-and-input',
  ),
  'E3002': ErrorRegistryEntry(
    code: 'E3002',
    title: 'File Access Issue',
    message: 'A file operation failed.',
    action:
        'Verify the path exists and that the app has permission to read/write.',
    docsSection: 'file-issues',
  ),
  'E4001': ErrorRegistryEntry(
    code: 'E4001',
    title: 'Timeout or Network Issue',
    message: 'The operation took too long or the network is unavailable.',
    action: 'Check connectivity or retry when the network is stable.',
    docsSection: 'timeouts-and-network',
    category: MessageCategory.warning,
  ),
  'E9000': ErrorRegistryEntry(
    code: 'E9000',
    title: 'Unexpected Error',
    message: 'An unexpected condition occurred.',
    action: 'Retry and attach logs if the issue repeats.',
    docsSection: 'escalation',
  ),
};

/// Backward-compatible wrapper around [ErrorHandler].
class ErrorTranslatorImpl extends ErrorHandler {
  const ErrorTranslatorImpl({super.docsPath = 'docs/ERROR_HANDLING.md'});
}
