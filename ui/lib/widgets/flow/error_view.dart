/// Error state widgets for KeyFlow with user-friendly error messages and retry functionality.
library;

import 'package:flutter/material.dart';

/// Error types for KeyFlow with specific user-friendly messages.
enum FlowErrorType {
  /// FFI connection failed
  ffiConnectionFailed,

  /// Runtime config not found
  runtimeConfigNotFound,

  /// Validation errors in flow configuration
  validationError,

  /// Generic unknown error
  unknown,
}

/// Extension to get user-friendly error messages.
extension FlowErrorTypeExt on FlowErrorType {
  String get message => switch (this) {
    FlowErrorType.ffiConnectionFailed =>
      'Unable to connect to the KeyRx service',
    FlowErrorType.runtimeConfigNotFound => 'Runtime configuration not found',
    FlowErrorType.validationError => 'Configuration validation failed',
    FlowErrorType.unknown => 'Something went wrong',
  };

  String get suggestion => switch (this) {
    FlowErrorType.ffiConnectionFailed =>
      'Make sure the KeyRx service is running and try again.',
    FlowErrorType.runtimeConfigNotFound =>
      'Please check your device connections and configuration.',
    FlowErrorType.validationError =>
      'Please check your flow configuration for errors.',
    FlowErrorType.unknown => 'Please try again or contact support.',
  };

  IconData get icon => switch (this) {
    FlowErrorType.ffiConnectionFailed => Icons.cloud_off,
    FlowErrorType.runtimeConfigNotFound => Icons.search_off,
    FlowErrorType.validationError => Icons.warning,
    FlowErrorType.unknown => Icons.error_outline,
  };
}

/// Polished error view widget with user-friendly error messages.
///
/// Displays different error types with appropriate icons, messages,
/// and actionable suggestions. Includes a retry button for recovery.
class FlowErrorView extends StatelessWidget {
  const FlowErrorView({
    required this.error,
    required this.onRetry,
    super.key,
    this.errorType,
  });

  /// The error object (for debugging)
  final Object? error;

  /// Callback to retry the failed operation
  final VoidCallback onRetry;

  /// Optional error type for specific handling
  final FlowErrorType? errorType;

  FlowErrorType _determineErrorType() {
    if (errorType != null) return errorType!;

    // Try to determine error type from error message
    final errorStr = error.toString().toLowerCase();
    if (errorStr.contains('ffi') || errorStr.contains('connection')) {
      return FlowErrorType.ffiConnectionFailed;
    }
    if (errorStr.contains('not found') || errorStr.contains('config')) {
      return FlowErrorType.runtimeConfigNotFound;
    }
    if (errorStr.contains('validation') || errorStr.contains('invalid')) {
      return FlowErrorType.validationError;
    }
    return FlowErrorType.unknown;
  }

  @override
  Widget build(BuildContext context) {
    final type = _determineErrorType();
    final theme = Theme.of(context);

    return Scaffold(
      backgroundColor: const Color(0xFF121212),
      body: Center(
        child: Semantics(
          label:
              '${type.message}. ${type.suggestion}. Press retry button to try again.',
          child: Container(
            constraints: const BoxConstraints(maxWidth: 480),
            padding: const EdgeInsets.all(32),
            child: Column(
              mainAxisSize: MainAxisSize.min,
              children: [
                // Error icon with background
                Container(
                  width: 80,
                  height: 80,
                  decoration: BoxDecoration(
                    color: theme.colorScheme.errorContainer,
                    shape: BoxShape.circle,
                  ),
                  child: Icon(
                    type.icon,
                    size: 42,
                    color: theme.colorScheme.error,
                  ),
                ),
                const SizedBox(height: 24),

                // Error message
                Text(
                  type.message,
                  style: theme.textTheme.headlineSmall?.copyWith(
                    color: Colors.white,
                    fontWeight: FontWeight.w600,
                  ),
                  textAlign: TextAlign.center,
                ),
                const SizedBox(height: 12),

                // Suggestion
                Text(
                  type.suggestion,
                  style: theme.textTheme.bodyMedium?.copyWith(
                    color: Colors.grey[400],
                  ),
                  textAlign: TextAlign.center,
                ),

                // Technical details (collapsed by default)
                const SizedBox(height: 24),
                _TechnicalDetailsExpansion(error: error),

                const SizedBox(height: 32),

                // Retry button
                FilledButton.icon(
                  onPressed: onRetry,
                  icon: const Icon(Icons.refresh),
                  label: const Text('Retry'),
                  style: FilledButton.styleFrom(
                    padding: const EdgeInsets.symmetric(
                      horizontal: 32,
                      vertical: 16,
                    ),
                  ),
                ),
              ],
            ),
          ),
        ),
      ),
    );
  }
}

/// Expandable section showing technical error details.
class _TechnicalDetailsExpansion extends StatefulWidget {
  const _TechnicalDetailsExpansion({required this.error});

  final Object? error;

  @override
  State<_TechnicalDetailsExpansion> createState() =>
      _TechnicalDetailsExpansionState();
}

class _TechnicalDetailsExpansionState
    extends State<_TechnicalDetailsExpansion> {
  bool _expanded = false;

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);

    return Column(
      mainAxisSize: MainAxisSize.min,
      children: [
        TextButton.icon(
          onPressed: () {
            setState(() {
              _expanded = !_expanded;
            });
          },
          icon: Icon(
            _expanded ? Icons.expand_less : Icons.expand_more,
            size: 18,
          ),
          label: Text(
            _expanded ? 'Hide technical details' : 'Show technical details',
            style: theme.textTheme.bodySmall?.copyWith(color: Colors.grey[500]),
          ),
          style: TextButton.styleFrom(
            padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 4),
          ),
        ),
        if (_expanded) ...[
          const SizedBox(height: 8),
          Container(
            width: double.infinity,
            padding: const EdgeInsets.all(16),
            decoration: BoxDecoration(
              color: Colors.black.withValues(alpha: 0.3),
              borderRadius: BorderRadius.circular(8),
              border: Border.all(color: Colors.grey[800]!),
            ),
            child: SelectableText(
              widget.error?.toString() ?? 'Unknown error',
              style: theme.textTheme.bodySmall?.copyWith(
                color: Colors.grey[400],
                fontFamily: 'monospace',
                fontSize: 11,
              ),
            ),
          ),
        ],
      ],
    );
  }
}

/// Inline error indicator for use within the flow canvas.
///
/// Used to show specific operation errors without taking over the entire screen.
class InlineErrorIndicator extends StatelessWidget {
  const InlineErrorIndicator({
    required this.message,
    super.key,
    this.onDismiss,
  });

  /// Error message to display
  final String message;

  /// Optional callback when user dismisses the error
  final VoidCallback? onDismiss;

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);

    return Semantics(
      label:
          'Error: $message. ${onDismiss != null ? 'Press dismiss button to close.' : ''}',
      child: Container(
        padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 12),
        decoration: BoxDecoration(
          color: theme.colorScheme.errorContainer,
          borderRadius: BorderRadius.circular(8),
          border: Border.all(color: theme.colorScheme.error),
        ),
        child: Row(
          mainAxisSize: MainAxisSize.min,
          children: [
            Icon(Icons.error_outline, size: 20, color: theme.colorScheme.error),
            const SizedBox(width: 12),
            Flexible(
              child: Text(
                message,
                style: theme.textTheme.bodySmall?.copyWith(
                  color: theme.colorScheme.onErrorContainer,
                  fontWeight: FontWeight.w500,
                ),
              ),
            ),
            if (onDismiss != null) ...[
              const SizedBox(width: 8),
              IconButton(
                icon: const Icon(Icons.close, size: 16),
                tooltip: 'Dismiss',
                onPressed: onDismiss,
                padding: EdgeInsets.zero,
                constraints: const BoxConstraints(minWidth: 24, minHeight: 24),
              ),
            ],
          ],
        ),
      ),
    );
  }
}
