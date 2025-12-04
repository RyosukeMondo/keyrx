/// Widgets for displaying script validation status banners.
library;

import 'package:flutter/material.dart';

import '../../ffi/bridge.dart';
import '../../models/validation.dart' as validation_models;

/// Widget for displaying script validation status banner (simple version).
class ValidationBanner extends StatelessWidget {
  const ValidationBanner({
    super.key,
    required this.isValidating,
    required this.validationResult,
    required this.onShowErrors,
  });

  final bool isValidating;
  final ScriptValidationResult? validationResult;
  final void Function(List<ScriptValidationError> errors) onShowErrors;

  @override
  Widget build(BuildContext context) {
    if (isValidating) {
      return Container(
        color: Colors.blue.withValues(alpha: 0.1),
        padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 8),
        child: const Row(
          children: [
            SizedBox(
              width: 16,
              height: 16,
              child: CircularProgressIndicator(strokeWidth: 2),
            ),
            SizedBox(width: 8),
            Text('Validating script...'),
          ],
        ),
      );
    }

    final result = validationResult;
    if (result == null || result.valid) {
      return const SizedBox.shrink();
    }

    final errors = result.errors;
    final errorMessage = result.errorMessage;

    return Material(
      color: Colors.red.withValues(alpha: 0.15),
      child: InkWell(
        onTap: errors.isNotEmpty ? () => onShowErrors(errors) : null,
        child: Padding(
          padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 8),
          child: Row(
            children: [
              const Icon(Icons.error_outline, color: Colors.red, size: 20),
              const SizedBox(width: 8),
              Expanded(
                child: Text(
                  errorMessage ??
                      (errors.isNotEmpty
                          ? 'Script has ${errors.length} error${errors.length > 1 ? 's' : ''}: ${errors.first.message}'
                          : 'Script validation failed'),
                  style: const TextStyle(color: Colors.red),
                  overflow: TextOverflow.ellipsis,
                ),
              ),
              if (errors.isNotEmpty) ...[
                const SizedBox(width: 8),
                const Icon(Icons.chevron_right, color: Colors.red),
              ],
            ],
          ),
        ),
      ),
    );
  }
}

/// Shows a dialog with validation errors.
void showValidationErrorsDialog(BuildContext context, List<ScriptValidationError> errors) {
  showDialog(
    context: context,
    builder: (ctx) => AlertDialog(
      title: const Text('Script Validation Errors'),
      content: SizedBox(
        width: double.maxFinite,
        child: ListView.separated(
          shrinkWrap: true,
          itemCount: errors.length,
          separatorBuilder: (_, __) => const Divider(),
          itemBuilder: (_, index) {
            final e = errors[index];
            return ListTile(
              leading: const Icon(Icons.error, color: Colors.red),
              title: Text(e.message),
              subtitle: e.line != null
                  ? Text('Line ${e.line}${e.column != null ? ', Column ${e.column}' : ''}')
                  : null,
              dense: true,
            );
          },
        ),
      ),
      actions: [TextButton(onPressed: () => Navigator.pop(ctx), child: const Text('Close'))],
    ),
  );
}

/// Rich validation banner showing errors and warnings with suggestions.
class ValidationBannerRich extends StatelessWidget {
  const ValidationBannerRich({
    super.key,
    required this.isValidating,
    required this.validationResult,
  });

  final bool isValidating;
  final validation_models.ValidationResult? validationResult;

  @override
  Widget build(BuildContext context) {
    if (isValidating) {
      return Container(
        color: Colors.blue.withValues(alpha: 0.1),
        padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 8),
        child: const Row(
          children: [
            SizedBox(
              width: 16,
              height: 16,
              child: CircularProgressIndicator(strokeWidth: 2),
            ),
            SizedBox(width: 8),
            Text('Validating script...'),
          ],
        ),
      );
    }

    final result = validationResult;
    if (result == null) {
      return const SizedBox.shrink();
    }

    // Valid with no issues
    if (result.isValid && !result.hasErrors && !result.hasWarnings) {
      return const SizedBox.shrink();
    }

    final hasErrors = result.hasErrors;
    final hasWarnings = result.hasWarnings;

    return Material(
      color: hasErrors
          ? Colors.red.withValues(alpha: 0.12)
          : Colors.orange.withValues(alpha: 0.12),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          // Summary header
          InkWell(
            onTap: () => _showValidationDetailsDialog(context, result),
            child: Padding(
              padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 8),
              child: Row(
                children: [
                  Icon(
                    hasErrors ? Icons.error_outline : Icons.warning_amber,
                    color: hasErrors ? Colors.red : Colors.orange,
                    size: 20,
                  ),
                  const SizedBox(width: 8),
                  Expanded(
                    child: Text(
                      _buildSummary(result),
                      style: TextStyle(
                        color: hasErrors ? Colors.red : Colors.orange.shade800,
                      ),
                    ),
                  ),
                  const Icon(Icons.chevron_right, size: 20),
                ],
              ),
            ),
          ),
          // Inline preview of first error/warning
          if (hasErrors && result.errors.isNotEmpty)
            _buildInlineIssue(context, result.errors.first, isError: true),
          if (!hasErrors && hasWarnings && result.warnings.isNotEmpty)
            _buildInlineWarning(context, result.warnings.first),
        ],
      ),
    );
  }

  String _buildSummary(validation_models.ValidationResult result) {
    final parts = <String>[];
    if (result.errors.isNotEmpty) {
      parts.add('${result.errors.length} error${result.errors.length > 1 ? 's' : ''}');
    }
    if (result.warnings.isNotEmpty) {
      parts.add('${result.warnings.length} warning${result.warnings.length > 1 ? 's' : ''}');
    }
    return parts.isEmpty ? 'Validation issues' : parts.join(', ');
  }

  Widget _buildInlineIssue(
    BuildContext context,
    validation_models.ValidationError error, {
    required bool isError,
  }) {
    return Padding(
      padding: const EdgeInsets.only(left: 44, right: 16, bottom: 8),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Text(
            error.message,
            style: TextStyle(
              fontSize: 13,
              color: isError ? Colors.red.shade700 : Colors.orange.shade800,
            ),
            maxLines: 2,
            overflow: TextOverflow.ellipsis,
          ),
          if (error.location != null)
            Padding(
              padding: const EdgeInsets.only(top: 2),
              child: Text(
                'Line ${error.location!.line}${error.location!.column != null ? ':${error.location!.column}' : ''}',
                style: TextStyle(
                  fontSize: 11,
                  color: Colors.grey.shade600,
                ),
              ),
            ),
          if (error.hasSuggestions)
            Padding(
              padding: const EdgeInsets.only(top: 4),
              child: Wrap(
                spacing: 4,
                runSpacing: 4,
                children: [
                  Text(
                    'Did you mean:',
                    style: TextStyle(
                      fontSize: 12,
                      color: Colors.grey.shade600,
                    ),
                  ),
                  ...error.suggestions.take(3).map(
                        (s) => _SuggestionChip(suggestion: s),
                      ),
                ],
              ),
            ),
        ],
      ),
    );
  }

  Widget _buildInlineWarning(
    BuildContext context,
    validation_models.ValidationWarning warning,
  ) {
    return Padding(
      padding: const EdgeInsets.only(left: 44, right: 16, bottom: 8),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Row(
            children: [
              Icon(
                _iconForCategory(warning.category),
                size: 14,
                color: Colors.orange.shade700,
              ),
              const SizedBox(width: 4),
              Expanded(
                child: Text(
                  warning.message,
                  style: TextStyle(
                    fontSize: 13,
                    color: Colors.orange.shade800,
                  ),
                  maxLines: 2,
                  overflow: TextOverflow.ellipsis,
                ),
              ),
            ],
          ),
          if (warning.location != null)
            Padding(
              padding: const EdgeInsets.only(top: 2, left: 18),
              child: Text(
                'Line ${warning.location!.line}${warning.location!.column != null ? ':${warning.location!.column}' : ''}',
                style: TextStyle(
                  fontSize: 11,
                  color: Colors.grey.shade600,
                ),
              ),
            ),
        ],
      ),
    );
  }

  IconData _iconForCategory(validation_models.WarningCategory category) {
    return switch (category) {
      validation_models.WarningCategory.conflict => Icons.compare_arrows,
      validation_models.WarningCategory.safety => Icons.warning_amber,
      validation_models.WarningCategory.performance => Icons.speed,
    };
  }

  void _showValidationDetailsDialog(
    BuildContext context,
    validation_models.ValidationResult result,
  ) {
    showDialog(
      context: context,
      builder: (ctx) => _ValidationDetailsDialog(result: result),
    );
  }
}

/// Small chip widget for displaying key suggestions.
class _SuggestionChip extends StatelessWidget {
  const _SuggestionChip({required this.suggestion});

  final String suggestion;

  @override
  Widget build(BuildContext context) {
    return Container(
      padding: const EdgeInsets.symmetric(horizontal: 6, vertical: 2),
      decoration: BoxDecoration(
        color: Colors.blue.withValues(alpha: 0.15),
        borderRadius: BorderRadius.circular(4),
        border: Border.all(color: Colors.blue.withValues(alpha: 0.3)),
      ),
      child: Text(
        suggestion,
        style: const TextStyle(
          fontSize: 12,
          color: Colors.blue,
          fontFamily: 'monospace',
        ),
      ),
    );
  }
}

/// Dialog showing full validation details with errors and warnings.
class _ValidationDetailsDialog extends StatelessWidget {
  const _ValidationDetailsDialog({required this.result});

  final validation_models.ValidationResult result;

  @override
  Widget build(BuildContext context) {
    return AlertDialog(
      title: const Text('Validation Details'),
      content: SizedBox(
        width: double.maxFinite,
        child: DefaultTabController(
          length: 2,
          child: Column(
            mainAxisSize: MainAxisSize.min,
            children: [
              TabBar(
                tabs: [
                  Tab(
                    child: Row(
                      mainAxisSize: MainAxisSize.min,
                      children: [
                        const Icon(Icons.error_outline, size: 16),
                        const SizedBox(width: 4),
                        Text('Errors (${result.errors.length})'),
                      ],
                    ),
                  ),
                  Tab(
                    child: Row(
                      mainAxisSize: MainAxisSize.min,
                      children: [
                        const Icon(Icons.warning_amber, size: 16),
                        const SizedBox(width: 4),
                        Text('Warnings (${result.warnings.length})'),
                      ],
                    ),
                  ),
                ],
              ),
              const SizedBox(height: 8),
              ConstrainedBox(
                constraints: const BoxConstraints(maxHeight: 300),
                child: TabBarView(
                  children: [
                    _buildErrorsList(),
                    _buildWarningsList(),
                  ],
                ),
              ),
            ],
          ),
        ),
      ),
      actions: [
        TextButton(
          onPressed: () => Navigator.pop(context),
          child: const Text('Close'),
        ),
      ],
    );
  }

  Widget _buildErrorsList() {
    if (result.errors.isEmpty) {
      return const Center(
        child: Text('No errors', style: TextStyle(color: Colors.grey)),
      );
    }

    return ListView.separated(
      shrinkWrap: true,
      itemCount: result.errors.length,
      separatorBuilder: (_, __) => const Divider(height: 1),
      itemBuilder: (context, index) {
        final error = result.errors[index];
        return _ValidationErrorTile(error: error);
      },
    );
  }

  Widget _buildWarningsList() {
    if (result.warnings.isEmpty) {
      return const Center(
        child: Text('No warnings', style: TextStyle(color: Colors.grey)),
      );
    }

    return ListView.separated(
      shrinkWrap: true,
      itemCount: result.warnings.length,
      separatorBuilder: (_, __) => const Divider(height: 1),
      itemBuilder: (context, index) {
        final warning = result.warnings[index];
        return _ValidationWarningTile(warning: warning);
      },
    );
  }
}

/// List tile for displaying a validation error with suggestions.
class _ValidationErrorTile extends StatelessWidget {
  const _ValidationErrorTile({required this.error});

  final validation_models.ValidationError error;

  @override
  Widget build(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.symmetric(vertical: 8, horizontal: 4),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Row(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              const Icon(Icons.error, color: Colors.red, size: 18),
              const SizedBox(width: 8),
              Expanded(
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    Text(
                      error.message,
                      style: const TextStyle(fontWeight: FontWeight.w500),
                    ),
                    if (error.location != null)
                      Padding(
                        padding: const EdgeInsets.only(top: 2),
                        child: Text(
                          'Line ${error.location!.line}${error.location!.column != null ? ', Column ${error.location!.column}' : ''}',
                          style: TextStyle(
                            fontSize: 12,
                            color: Colors.grey.shade600,
                          ),
                        ),
                      ),
                    if (error.location?.context != null)
                      Padding(
                        padding: const EdgeInsets.only(top: 4),
                        child: Container(
                          padding: const EdgeInsets.all(4),
                          decoration: BoxDecoration(
                            color: Colors.grey.shade100,
                            borderRadius: BorderRadius.circular(4),
                          ),
                          child: Text(
                            error.location!.context!,
                            style: const TextStyle(
                              fontFamily: 'monospace',
                              fontSize: 12,
                            ),
                          ),
                        ),
                      ),
                  ],
                ),
              ),
            ],
          ),
          if (error.hasSuggestions)
            Padding(
              padding: const EdgeInsets.only(top: 8, left: 26),
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  Text(
                    'Did you mean:',
                    style: TextStyle(
                      fontSize: 12,
                      color: Colors.grey.shade600,
                    ),
                  ),
                  const SizedBox(height: 4),
                  Wrap(
                    spacing: 6,
                    runSpacing: 4,
                    children: error.suggestions
                        .map((s) => _SuggestionChip(suggestion: s))
                        .toList(),
                  ),
                ],
              ),
            ),
        ],
      ),
    );
  }
}

/// List tile for displaying a validation warning.
class _ValidationWarningTile extends StatelessWidget {
  const _ValidationWarningTile({required this.warning});

  final validation_models.ValidationWarning warning;

  @override
  Widget build(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.symmetric(vertical: 8, horizontal: 4),
      child: Row(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Icon(
            _iconForCategory(warning.category),
            color: Colors.orange,
            size: 18,
          ),
          const SizedBox(width: 8),
          Expanded(
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Row(
                  children: [
                    _CategoryBadge(category: warning.category),
                    const SizedBox(width: 8),
                    Text(
                      warning.code,
                      style: TextStyle(
                        fontSize: 11,
                        color: Colors.grey.shade600,
                        fontFamily: 'monospace',
                      ),
                    ),
                  ],
                ),
                const SizedBox(height: 2),
                Text(warning.message),
                if (warning.location != null)
                  Padding(
                    padding: const EdgeInsets.only(top: 2),
                    child: Text(
                      'Line ${warning.location!.line}${warning.location!.column != null ? ', Column ${warning.location!.column}' : ''}',
                      style: TextStyle(
                        fontSize: 12,
                        color: Colors.grey.shade600,
                      ),
                    ),
                  ),
              ],
            ),
          ),
        ],
      ),
    );
  }

  IconData _iconForCategory(validation_models.WarningCategory category) {
    return switch (category) {
      validation_models.WarningCategory.conflict => Icons.compare_arrows,
      validation_models.WarningCategory.safety => Icons.warning_amber,
      validation_models.WarningCategory.performance => Icons.speed,
    };
  }
}

/// Small badge showing warning category.
class _CategoryBadge extends StatelessWidget {
  const _CategoryBadge({required this.category});

  final validation_models.WarningCategory category;

  @override
  Widget build(BuildContext context) {
    final (label, color) = switch (category) {
      validation_models.WarningCategory.conflict => ('Conflict', Colors.purple),
      validation_models.WarningCategory.safety => ('Safety', Colors.orange),
      validation_models.WarningCategory.performance => ('Perf', Colors.blue),
    };

    return Container(
      padding: const EdgeInsets.symmetric(horizontal: 6, vertical: 1),
      decoration: BoxDecoration(
        color: color.withValues(alpha: 0.15),
        borderRadius: BorderRadius.circular(4),
      ),
      child: Text(
        label,
        style: TextStyle(
          fontSize: 10,
          fontWeight: FontWeight.w600,
          color: color,
        ),
      ),
    );
  }
}
