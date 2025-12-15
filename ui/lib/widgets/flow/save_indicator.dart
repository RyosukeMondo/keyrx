/// Save state indicator widget showing saving/saved/error states.
library;

import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../../services/auto_save_service.dart';
import '../../state/save_state_provider.dart';

/// Displays the current auto-save state with appropriate icon and text.
///
/// Shows:
/// - Idle: Hidden
/// - Saving: Spinner + "Saving..."
/// - Saved: Checkmark + "Saved"
/// - Error: Error icon + "Failed to save" with retry button
class SaveIndicator extends ConsumerWidget {
  const SaveIndicator({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final saveState = ref.watch(saveStateNotifierProvider);

    if (saveState.isIdle) {
      return const SizedBox.shrink();
    }

    return Semantics(
      label: _getSemanticLabel(saveState),
      child: Container(
        padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 8),
        decoration: BoxDecoration(
          color: _getBackgroundColor(context, saveState),
          borderRadius: BorderRadius.circular(8),
          boxShadow: [
            BoxShadow(
              color: Colors.black.withValues(alpha: 0.1),
              blurRadius: 4,
              offset: const Offset(0, 2),
            ),
          ],
        ),
        child: Row(
          mainAxisSize: MainAxisSize.min,
          children: [
            _buildIcon(saveState),
            const SizedBox(width: 8),
            Text(
              _getStatusText(saveState),
              style: Theme.of(context).textTheme.bodySmall?.copyWith(
                color: _getTextColor(context, saveState),
                fontWeight: FontWeight.w500,
              ),
            ),
            if (saveState.isError) ...[
              const SizedBox(width: 8),
              IconButton(
                icon: const Icon(Icons.refresh, size: 16),
                tooltip: 'Retry save',
                onPressed: () {
                  ref.read(saveStateNotifierProvider.notifier).retry();
                },
                padding: EdgeInsets.zero,
                constraints: const BoxConstraints(minWidth: 24, minHeight: 24),
              ),
            ],
          ],
        ),
      ),
    );
  }

  Widget _buildIcon(SaveState state) {
    return switch (state.status) {
      SaveStatus.saving => const SizedBox(
        width: 16,
        height: 16,
        child: CircularProgressIndicator(
          strokeWidth: 2,
          valueColor: AlwaysStoppedAnimation<Color>(Colors.blue),
        ),
      ),
      SaveStatus.saved => const Icon(
        Icons.check_circle,
        size: 16,
        color: Colors.green,
      ),
      SaveStatus.error => const Icon(Icons.error, size: 16, color: Colors.red),
      SaveStatus.idle => const SizedBox.shrink(),
    };
  }

  String _getStatusText(SaveState state) {
    return switch (state.status) {
      SaveStatus.saving => 'Saving...',
      SaveStatus.saved => 'Saved',
      SaveStatus.error => 'Failed to save',
      SaveStatus.idle => '',
    };
  }

  Color _getBackgroundColor(BuildContext context, SaveState state) {
    final colorScheme = Theme.of(context).colorScheme;
    return switch (state.status) {
      SaveStatus.saving => colorScheme.primaryContainer,
      SaveStatus.saved => colorScheme.tertiaryContainer,
      SaveStatus.error => colorScheme.errorContainer,
      SaveStatus.idle => Colors.transparent,
    };
  }

  Color _getTextColor(BuildContext context, SaveState state) {
    final colorScheme = Theme.of(context).colorScheme;
    return switch (state.status) {
      SaveStatus.saving => colorScheme.onPrimaryContainer,
      SaveStatus.saved => colorScheme.onTertiaryContainer,
      SaveStatus.error => colorScheme.onErrorContainer,
      SaveStatus.idle => colorScheme.onSurface,
    };
  }

  String _getSemanticLabel(SaveState state) {
    if (state.isError && state.errorMessage != null) {
      return 'Save failed: ${state.errorMessage}. Press retry to try again.';
    }
    return _getStatusText(state);
  }
}

/// Inline save indicator for use within the flow editor page.
///
/// Positioned in the app bar or toolbar area.
class InlineSaveIndicator extends ConsumerWidget {
  const InlineSaveIndicator({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final saveState = ref.watch(saveStateNotifierProvider);

    if (saveState.isIdle) {
      return const SizedBox.shrink();
    }

    return Padding(
      padding: const EdgeInsets.symmetric(horizontal: 8),
      child: Tooltip(
        message: saveState.isError && saveState.errorMessage != null
            ? saveState.errorMessage!
            : _getStatusText(saveState),
        child: Row(
          mainAxisSize: MainAxisSize.min,
          children: [
            _buildIcon(saveState),
            const SizedBox(width: 4),
            Text(
              _getStatusText(saveState),
              style: Theme.of(context).textTheme.bodySmall,
            ),
          ],
        ),
      ),
    );
  }

  Widget _buildIcon(SaveState state) {
    return switch (state.status) {
      SaveStatus.saving => const SizedBox(
        width: 14,
        height: 14,
        child: CircularProgressIndicator(
          strokeWidth: 2,
          valueColor: AlwaysStoppedAnimation<Color>(Colors.blue),
        ),
      ),
      SaveStatus.saved => const Icon(
        Icons.check_circle,
        size: 14,
        color: Colors.green,
      ),
      SaveStatus.error => const Icon(Icons.error, size: 14, color: Colors.red),
      SaveStatus.idle => const SizedBox.shrink(),
    };
  }

  String _getStatusText(SaveState state) {
    return switch (state.status) {
      SaveStatus.saving => 'Saving',
      SaveStatus.saved => 'Saved',
      SaveStatus.error => 'Error',
      SaveStatus.idle => '',
    };
  }
}
