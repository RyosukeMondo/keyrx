/// Focus indicator wrapper for keyboard navigation.
library;

import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:keyrx_ui/widgets/flow/keyboard_navigable_canvas.dart';
import 'package:keyrx_ui/state/selection_state.dart';

/// Wraps a node widget with a focus indicator for keyboard navigation.
///
/// Shows a highly visible outline when the node is focused via keyboard
/// navigation. The outline meets WCAG visibility requirements with high
/// contrast and 2px width.
class FocusIndicator extends ConsumerWidget {
  const FocusIndicator({super.key, required this.nodeId, required this.child});

  final String nodeId;
  final Widget child;

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final focusTracker = ref.watch(focusTrackerProvider);
    final selection = ref.watch(selectionNotifierProvider);

    final isFocused = focusTracker.isNodeFocused(nodeId);
    final isSelected = selection is NodeSelection && selection.nodeId == nodeId;

    // Show indicator if focused or selected
    final showIndicator = isFocused || isSelected;

    if (!showIndicator) {
      return child;
    }

    // Use different colors for focus vs selection
    final indicatorColor = isFocused
        ? const Color(0xFF4A9EFF) // Bright blue for focus
        : const Color(0xFF64B5F6); // Lighter blue for selection

    return Container(
      decoration: BoxDecoration(
        borderRadius: BorderRadius.circular(14),
        border: Border.all(color: indicatorColor, width: 3),
        boxShadow: [
          BoxShadow(
            color: indicatorColor.withOpacity(0.3),
            blurRadius: 8,
            spreadRadius: 1,
          ),
        ],
      ),
      child: child,
    );
  }
}
