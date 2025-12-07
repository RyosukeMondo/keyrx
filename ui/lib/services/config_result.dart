/// Shared helpers for parsing config FFI responses.
library;

import 'dart:convert';

/// Standard result wrapper for config CRUD operations.
class ConfigOperationResult<T> {
  const ConfigOperationResult._({this.data, this.errorMessage});

  factory ConfigOperationResult.success([T? data]) =>
      ConfigOperationResult._(data: data);

  factory ConfigOperationResult.error(String message) =>
      ConfigOperationResult._(errorMessage: message);

  final T? data;
  final String? errorMessage;

  bool get hasError => errorMessage != null;
  bool get isSuccess => !hasError;
}

/// Parse an FFI response of shape `ok:<json>` or `error:<json>`.
ConfigOperationResult<T> parseConfigFfiResult<T>(
  String raw,
  T Function(dynamic json)? decoder,
) {
  final trimmed = raw.trim();
  final lower = trimmed.toLowerCase();

  if (lower.startsWith('error:')) {
    final payload = trimmed.substring('error:'.length).trim();
    if (payload.isEmpty) {
      return ConfigOperationResult.error('Unknown error');
    }

    try {
      final jsonMap = json.decode(payload) as Map<String, dynamic>;
      final code = jsonMap['code'] as String?;
      final message = jsonMap['message'] as String?;
      final details = jsonMap['details'];
      final detailStr = details == null ? '' : ' ($details)';
      final composed = [
        if (code != null && code.isNotEmpty) code,
        if (message != null && message.isNotEmpty) message,
      ].join(': ');
      if (composed.isEmpty) {
        return ConfigOperationResult.error('Error: $payload');
      }
      return ConfigOperationResult.error('$composed$detailStr');
    } catch (e) {
      return ConfigOperationResult.error('Error response parse failed: $e');
    }
  }

  if (!lower.startsWith('ok:')) {
    return ConfigOperationResult.error('Invalid response: $trimmed');
  }

  final payload = trimmed.substring('ok:'.length).trim();
  if (payload.isEmpty) {
    return ConfigOperationResult.success(null);
  }

  if (decoder == null) {
    return ConfigOperationResult.success(payload as T);
  }

  try {
    final decoded = json.decode(payload);
    final value = decoder(decoded);
    return ConfigOperationResult.success(value);
  } catch (e) {
    return ConfigOperationResult.error('JSON decode error: $e');
  }
}
