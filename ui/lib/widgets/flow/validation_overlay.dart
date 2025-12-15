/// Validation overlay widget for displaying errors on nodes and edges.
library;

import 'package:flutter/material.dart';
import 'package:keyrx_ui/models/flow/flow_graph.dart';
import 'package:keyrx_ui/services/flow_validator.dart';

/// Badge widget displaying validation error count on a node.
///
/// Shows a red exclamation icon with error count for errors,
/// yellow warning icon for warnings, and blue info icon for info messages.
class ValidationBadge extends StatelessWidget {
  const ValidationBadge({super.key, required this.errors, this.onTap});

  /// List of validation errors to display
  final List<ValidationError> errors;

  /// Optional callback when badge is tapped
  final VoidCallback? onTap;

  @override
  Widget build(BuildContext context) {
    if (errors.isEmpty) return const SizedBox.shrink();

    final theme = Theme.of(context);

    // Count errors by severity
    final errorCount = errors
        .where((e) => e.severity == ValidationSeverity.error)
        .length;
    final warningCount = errors
        .where((e) => e.severity == ValidationSeverity.warning)
        .length;
    final infoCount = errors
        .where((e) => e.severity == ValidationSeverity.info)
        .length;

    // Determine primary severity (highest priority)
    final severity = errorCount > 0
        ? ValidationSeverity.error
        : warningCount > 0
        ? ValidationSeverity.warning
        : ValidationSeverity.info;

    final count = errorCount + warningCount + infoCount;

    return Semantics(
      label:
          '$count ${severity.name}${count > 1 ? 's' : ''} found. Tap to view details.',
      button: true,
      child: GestureDetector(
        onTap: onTap,
        child: Tooltip(
          message: _getTooltipMessage(errorCount, warningCount, infoCount),
          preferBelow: false,
          child: Container(
            padding: const EdgeInsets.symmetric(horizontal: 6, vertical: 4),
            decoration: BoxDecoration(
              color: _getSeverityColor(severity, theme),
              borderRadius: BorderRadius.circular(12),
              border: Border.all(
                color: _getSeverityBorderColor(severity, theme),
                width: 1.5,
              ),
            ),
            child: Row(
              mainAxisSize: MainAxisSize.min,
              children: [
                Icon(
                  _getSeverityIcon(severity),
                  size: 14,
                  color: _getSeverityIconColor(severity, theme),
                ),
                if (count > 1) ...[
                  const SizedBox(width: 4),
                  Text(
                    count.toString(),
                    style: theme.textTheme.labelSmall?.copyWith(
                      color: _getSeverityIconColor(severity, theme),
                      fontWeight: FontWeight.bold,
                      fontSize: 11,
                    ),
                  ),
                ],
              ],
            ),
          ),
        ),
      ),
    );
  }

  String _getTooltipMessage(int errors, int warnings, int info) {
    final parts = <String>[];
    if (errors > 0) parts.add('$errors error${errors > 1 ? 's' : ''}');
    if (warnings > 0) parts.add('$warnings warning${warnings > 1 ? 's' : ''}');
    if (info > 0) parts.add('$info info');
    return parts.join(', ');
  }

  Color _getSeverityColor(ValidationSeverity severity, ThemeData theme) {
    return switch (severity) {
      ValidationSeverity.error => theme.colorScheme.errorContainer,
      ValidationSeverity.warning => const Color(0xFFFFF3CD),
      ValidationSeverity.info => theme.colorScheme.primaryContainer,
    };
  }

  Color _getSeverityBorderColor(ValidationSeverity severity, ThemeData theme) {
    return switch (severity) {
      ValidationSeverity.error => theme.colorScheme.error,
      ValidationSeverity.warning => const Color(0xFFFFC107),
      ValidationSeverity.info => theme.colorScheme.primary,
    };
  }

  Color _getSeverityIconColor(ValidationSeverity severity, ThemeData theme) {
    return switch (severity) {
      ValidationSeverity.error => theme.colorScheme.error,
      ValidationSeverity.warning => const Color(0xFFFF8F00),
      ValidationSeverity.info => theme.colorScheme.primary,
    };
  }

  IconData _getSeverityIcon(ValidationSeverity severity) {
    return switch (severity) {
      ValidationSeverity.error => Icons.error,
      ValidationSeverity.warning => Icons.warning,
      ValidationSeverity.info => Icons.info,
    };
  }
}

/// Error details dialog showing all validation errors for a node or edge.
class ValidationErrorsDialog extends StatelessWidget {
  const ValidationErrorsDialog({
    super.key,
    required this.errors,
    required this.title,
  });

  /// List of errors to display
  final List<ValidationError> errors;

  /// Dialog title (e.g., "Node Validation Errors")
  final String title;

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);

    return AlertDialog(
      title: Row(
        children: [
          Icon(Icons.warning, color: theme.colorScheme.error),
          const SizedBox(width: 8),
          Text(title),
        ],
      ),
      content: SizedBox(
        width: double.maxFinite,
        child: ListView.separated(
          shrinkWrap: true,
          itemCount: errors.length,
          separatorBuilder: (_, __) => const Divider(height: 24),
          itemBuilder: (context, index) {
            final error = errors[index];
            return _ValidationErrorItem(error: error);
          },
        ),
      ),
      actions: [
        TextButton(
          onPressed: () => Navigator.of(context).pop(),
          child: const Text('Close'),
        ),
      ],
    );
  }
}

/// Individual validation error item in the errors dialog.
class _ValidationErrorItem extends StatelessWidget {
  const _ValidationErrorItem({required this.error});

  final ValidationError error;

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);

    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      mainAxisSize: MainAxisSize.min,
      children: [
        // Severity badge and message
        Row(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Icon(
              _getSeverityIcon(error.severity),
              size: 20,
              color: _getSeverityColor(error.severity, theme),
            ),
            const SizedBox(width: 8),
            Expanded(
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                mainAxisSize: MainAxisSize.min,
                children: [
                  Text(
                    error.message,
                    style: theme.textTheme.bodyMedium?.copyWith(
                      fontWeight: FontWeight.w600,
                      color: Colors.white,
                    ),
                  ),
                  const SizedBox(height: 4),
                  Text(
                    error.suggestion,
                    style: theme.textTheme.bodySmall?.copyWith(
                      color: Colors.grey[400],
                    ),
                  ),
                ],
              ),
            ),
          ],
        ),
        // Edge ID if applicable
        if (error.edgeId != null) ...[
          const SizedBox(height: 8),
          Container(
            padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 4),
            decoration: BoxDecoration(
              color: Colors.black.withValues(alpha: 0.3),
              borderRadius: BorderRadius.circular(4),
              border: Border.all(color: Colors.grey[800]!, width: 1),
            ),
            child: Text(
              'Slot ID: ${error.edgeId}',
              style: theme.textTheme.bodySmall?.copyWith(
                fontFamily: 'monospace',
                fontSize: 10,
                color: Colors.grey[500],
              ),
            ),
          ),
        ],
      ],
    );
  }

  IconData _getSeverityIcon(ValidationSeverity severity) {
    return switch (severity) {
      ValidationSeverity.error => Icons.error,
      ValidationSeverity.warning => Icons.warning,
      ValidationSeverity.info => Icons.info_outline,
    };
  }

  Color _getSeverityColor(ValidationSeverity severity, ThemeData theme) {
    return switch (severity) {
      ValidationSeverity.error => theme.colorScheme.error,
      ValidationSeverity.warning => const Color(0xFFFFC107),
      ValidationSeverity.info => theme.colorScheme.primary,
    };
  }
}

/// Edge validation indicator shown on invalid edges.
///
/// Displays a red highlight with error tooltip on edges with validation errors.
class EdgeValidationIndicator extends StatelessWidget {
  const EdgeValidationIndicator({
    super.key,
    required this.edge,
    required this.errors,
    required this.onTap,
  });

  /// The edge being validated
  final FlowEdge edge;

  /// Validation errors for this edge
  final List<ValidationError> errors;

  /// Callback when indicator is tapped
  final VoidCallback onTap;

  @override
  Widget build(BuildContext context) {
    if (errors.isEmpty) return const SizedBox.shrink();

    final theme = Theme.of(context);

    return Semantics(
      label:
          'Edge has ${errors.length} validation error${errors.length > 1 ? 's' : ''}',
      button: true,
      child: Tooltip(
        message: errors.first.message,
        preferBelow: false,
        child: GestureDetector(
          onTap: onTap,
          child: Container(
            padding: const EdgeInsets.all(4),
            decoration: BoxDecoration(
              color: theme.colorScheme.errorContainer,
              shape: BoxShape.circle,
              border: Border.all(color: theme.colorScheme.error, width: 1.5),
            ),
            child: Icon(Icons.error, size: 12, color: theme.colorScheme.error),
          ),
        ),
      ),
    );
  }
}
