/// Result type for explicit error handling without exceptions.
///
/// Inspired by Rust's Result type, this provides a type-safe way to handle
/// success and error cases. Use pattern matching or the fold method to handle
/// both cases explicitly.
library;

import 'package:freezed_annotation/freezed_annotation.dart';
import '../error_translator.dart';

part 'result.freezed.dart';

/// A sealed Result type representing either success (Ok) or failure (Err).
///
/// Example:
/// ```dart
/// Result<int> divide(int a, int b) {
///   if (b == 0) {
///     return Result.err(FacadeError.validation('Division by zero'));
///   }
///   return Result.ok(a ~/ b);
/// }
///
/// final result = divide(10, 2);
/// result.when(
///   ok: (value) => print('Result: $value'),
///   err: (error) => print('Error: ${error.message}'),
/// );
/// ```
@freezed
sealed class Result<T> with _$Result<T> {
  const Result._();

  /// Success variant containing the result value.
  const factory Result.ok(T value) = Ok<T>;

  /// Error variant containing the error details.
  const factory Result.err(FacadeError error) = Err<T>;

  /// Check if this is a success result.
  bool get isOk => this is Ok<T>;

  /// Check if this is an error result.
  bool get isErr => this is Err<T>;

  /// Get the value if Ok, otherwise return null.
  T? get okOrNull => when(
        ok: (value) => value,
        err: (_) => null,
      );

  /// Get the error if Err, otherwise return null.
  FacadeError? get errOrNull => when(
        ok: (_) => null,
        err: (error) => error,
      );

  /// Transform the Ok value using the provided function.
  ///
  /// If this is an Err, returns the error unchanged.
  Result<R> mapValue<R>(R Function(T) f) {
    return when(
      ok: (value) => Result.ok(f(value)),
      err: (error) => Result.err(error),
    );
  }

  /// Transform the Ok value using a function that returns a Result.
  ///
  /// If this is an Err, returns the error unchanged.
  Result<R> andThen<R>(Result<R> Function(T) f) {
    return when(
      ok: (value) => f(value),
      err: (error) => Result.err(error),
    );
  }

  /// Transform both Ok and Err cases to a single value.
  R fold<R>(R Function(T) onOk, R Function(FacadeError) onErr) {
    return when(ok: onOk, err: onErr);
  }

  /// Unwrap the Ok value or throw an exception if Err.
  ///
  /// Use with caution - prefer using pattern matching or fold instead.
  T unwrap() {
    return when(
      ok: (value) => value,
      err: (error) => throw StateError('Called unwrap on Err: ${error.message}'),
    );
  }

  /// Unwrap the Ok value or return a default value if Err.
  T unwrapOr(T defaultValue) {
    return when(
      ok: (value) => value,
      err: (_) => defaultValue,
    );
  }

  /// Unwrap the Ok value or compute it from the error.
  T unwrapOrElse(T Function(FacadeError) f) {
    return when(
      ok: (value) => value,
      err: (error) => f(error),
    );
  }
}

/// Error type for facade operations.
///
/// Contains error code, technical message, and user-friendly message.
@freezed
class FacadeError with _$FacadeError {
  const FacadeError._();

  /// Create a facade error with all fields.
  const factory FacadeError({
    required String code,
    required String message,
    required String userMessage,
    Object? originalError,
  }) = _FacadeError;

  /// Create an error from a generic exception or error object.
  factory FacadeError.from(Object error, ErrorTranslator translator) {
    final userMsg = translator.translate(error);
    return FacadeError(
      code: error.runtimeType.toString(),
      message: error.toString(),
      userMessage: '${userMsg.title}: ${userMsg.body}',
      originalError: error,
    );
  }

  /// Validation error factory.
  factory FacadeError.validation(String message, {String? userMessage}) {
    return FacadeError(
      code: 'VALIDATION_ERROR',
      message: message,
      userMessage: userMessage ?? 'Invalid input: $message',
    );
  }

  /// Service unavailable error factory.
  factory FacadeError.serviceUnavailable(
    String service, {
    String? userMessage,
  }) {
    return FacadeError(
      code: 'SERVICE_UNAVAILABLE',
      message: 'Service "$service" is not available',
      userMessage: userMessage ?? 'Service temporarily unavailable. Please try again.',
    );
  }

  /// Operation failed error factory.
  factory FacadeError.operationFailed(
    String operation,
    String reason, {
    String? userMessage,
  }) {
    return FacadeError(
      code: 'OPERATION_FAILED',
      message: 'Operation "$operation" failed: $reason',
      userMessage: userMessage ?? 'Operation failed: $reason',
    );
  }

  /// Invalid state error factory.
  factory FacadeError.invalidState(String currentState, String requiredState) {
    return FacadeError(
      code: 'INVALID_STATE',
      message: 'Invalid state: currently $currentState, requires $requiredState',
      userMessage: 'Cannot perform this action right now. Please try again.',
    );
  }

  /// Timeout error factory.
  factory FacadeError.timeout(String operation, {Duration? duration}) {
    final msg = duration != null
        ? 'Operation "$operation" timed out after ${duration.inSeconds}s'
        : 'Operation "$operation" timed out';
    return FacadeError(
      code: 'TIMEOUT',
      message: msg,
      userMessage: 'Operation took too long. Please try again.',
    );
  }

  /// Permission denied error factory.
  factory FacadeError.permissionDenied(String permission, {String? userMessage}) {
    return FacadeError(
      code: 'PERMISSION_DENIED',
      message: 'Permission denied: $permission',
      userMessage: userMessage ?? 'Permission required: $permission. Please grant access in settings.',
    );
  }

  /// Resource not found error factory.
  factory FacadeError.notFound(String resource, {String? userMessage}) {
    return FacadeError(
      code: 'NOT_FOUND',
      message: 'Resource not found: $resource',
      userMessage: userMessage ?? 'Requested item not found.',
    );
  }

  /// File I/O error factory.
  factory FacadeError.fileError(String path, String reason, {String? userMessage}) {
    return FacadeError(
      code: 'FILE_ERROR',
      message: 'File error at "$path": $reason',
      userMessage: userMessage ?? 'File operation failed: $reason',
    );
  }

  /// Network error factory.
  factory FacadeError.networkError(String reason, {String? userMessage}) {
    return FacadeError(
      code: 'NETWORK_ERROR',
      message: 'Network error: $reason',
      userMessage: userMessage ?? 'Network error. Please check your connection.',
    );
  }

  /// Unknown error factory.
  factory FacadeError.unknown(String message) {
    return FacadeError(
      code: 'UNKNOWN_ERROR',
      message: message,
      userMessage: 'An unexpected error occurred. Please try again.',
    );
  }

  /// Convert to UserMessage for display.
  UserMessage toUserMessage() {
    final category = _categoryFromCode(code);
    return UserMessage(
      title: _titleFromCode(code),
      body: userMessage,
      category: category,
    );
  }

  MessageCategory _categoryFromCode(String code) {
    if (code.contains('WARNING') || code == 'TIMEOUT') {
      return MessageCategory.warning;
    }
    if (code.contains('INFO') || code == 'NOT_FOUND') {
      return MessageCategory.info;
    }
    return MessageCategory.error;
  }

  String _titleFromCode(String code) {
    return switch (code) {
      'VALIDATION_ERROR' => 'Validation Error',
      'SERVICE_UNAVAILABLE' => 'Service Unavailable',
      'OPERATION_FAILED' => 'Operation Failed',
      'INVALID_STATE' => 'Invalid State',
      'TIMEOUT' => 'Timeout',
      'PERMISSION_DENIED' => 'Permission Denied',
      'NOT_FOUND' => 'Not Found',
      'FILE_ERROR' => 'File Error',
      'NETWORK_ERROR' => 'Network Error',
      _ => 'Error',
    };
  }
}
