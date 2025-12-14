import '../utils/error_handler.dart';

/// Backward-compatible wrapper around [ErrorHandler].
class ErrorTranslatorImpl extends ErrorHandler {
  const ErrorTranslatorImpl({super.docsPath = 'docs/ERROR_HANDLING.md'});
}
