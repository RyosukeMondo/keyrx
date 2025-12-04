import 'dart:async';

import 'error_translator.dart';

/// Default implementation that maps internal errors to user-friendly messages.
class ErrorTranslatorImpl implements ErrorTranslator {
  const ErrorTranslatorImpl();

  @override
  UserMessage translate(Object error) {
    if (error is TimeoutException) {
      return const UserMessage(
        title: 'Operation timed out',
        body: 'The request took too long. Please try again.',
        category: MessageCategory.warning,
      );
    }

    if (error is StateError) {
      return const UserMessage(
        title: 'Engine not ready',
        body: 'The engine is not ready yet. Please try again.',
      );
    }

    return const UserMessage(
      title: 'Something went wrong',
      body: 'An unexpected error occurred. Please retry or restart the app.',
    );
  }
}
