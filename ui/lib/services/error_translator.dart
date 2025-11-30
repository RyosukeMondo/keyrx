/// Contracts for translating engine errors into user-friendly messages.
library;

/// Severity buckets for user-facing messages.
enum MessageCategory { info, warning, error }

/// UI-ready message payload.
class UserMessage {
  const UserMessage({
    required this.title,
    required this.body,
    this.category = MessageCategory.error,
  });

  final String title;
  final String body;
  final MessageCategory category;
}

/// Translates internal errors into UI-friendly messages.
abstract class ErrorTranslator {
  /// Convert any error/exception into a message safe for display.
  UserMessage translate(Object error);
}
