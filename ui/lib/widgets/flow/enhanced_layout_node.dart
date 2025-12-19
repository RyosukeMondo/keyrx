import 'package:core_logic/models/flow/flow_graph.dart';
import 'package:flutter/material.dart';

import 'accessible_flow_canvas.dart';

/// Enhanced hardware profile/layout node widget with Material Design 3 styling
///
/// Status colors:
/// - Blue: Valid profile
/// - Red: Invalid profile (not found or error)
class EnhancedLayoutNode extends StatelessWidget {
  const EnhancedLayoutNode({
    required this.node,
    super.key,
    this.layoutType,
    this.isValid = true,
    this.errorMessage,
    this.onTap,
  });
  final HardwareProfileFlowNode node;
  final String? layoutType;
  final bool isValid;
  final String? errorMessage;
  final VoidCallback? onTap;

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);
    final colorScheme = theme.colorScheme;
    final status = _getLayoutStatus();

    // Generate semantic label for screen readers
    final semanticLabel = node.toSemanticLabel(
      isConnected: true,
      hasError: !isValid,
    );

    return Semantics(
      label: semanticLabel,
      button: onTap != null,
      enabled: true,
      child: Material(
        elevation: 2,
        borderRadius: BorderRadius.circular(12),
        shadowColor: Colors.black.withValues(alpha: 0.2),
        child: InkWell(
          onTap: onTap,
          borderRadius: BorderRadius.circular(12),
          child: Container(
            width: 220,
            // Ensure minimum touch target height (48px minimum)
            constraints: const BoxConstraints(minHeight: 48),
            padding: const EdgeInsets.all(16),
            decoration: BoxDecoration(
              color: colorScheme.surface,
              borderRadius: BorderRadius.circular(12),
              border: Border.all(
                color: status.borderColor(colorScheme),
                width: 2,
              ),
            ),
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              mainAxisSize: MainAxisSize.min,
              children: [
                _buildHeader(theme, status),
                const SizedBox(height: 12),
                _buildLayoutInfo(theme, status),
                const SizedBox(height: 12),
                _buildLayoutPreview(theme, status),
                const SizedBox(height: 8),
                _buildConnectionHandles(status),
              ],
            ),
          ),
        ),
      ),
    );
  }

  Widget _buildHeader(ThemeData theme, _LayoutStatus status) => Row(
    children: [
      // Icon with colored background
      Container(
        width: 40,
        height: 40,
        decoration: BoxDecoration(
          color: status.iconBackgroundColor.withValues(alpha: 0.15),
          borderRadius: BorderRadius.circular(8),
        ),
        child: Icon(status.icon, color: status.iconColor, size: 24),
      ),
      const SizedBox(width: 12),
      Expanded(
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Text(
              node.label ?? 'Unknown Profile',
              style: theme.textTheme.titleMedium?.copyWith(
                fontWeight: FontWeight.w600,
              ),
              overflow: TextOverflow.ellipsis,
              maxLines: 1,
            ),
            const SizedBox(height: 2),
            _buildStatusChip(theme, status),
          ],
        ),
      ),
    ],
  );

  Widget _buildStatusChip(ThemeData theme, _LayoutStatus status) => Container(
    padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 4),
    decoration: BoxDecoration(
      color: status.iconBackgroundColor.withValues(alpha: 0.15),
      borderRadius: BorderRadius.circular(12),
      border: Border.all(
        color: status.iconBackgroundColor.withValues(alpha: 0.3),
      ),
    ),
    child: Row(
      mainAxisSize: MainAxisSize.min,
      children: [
        Icon(status.statusIcon, size: 12, color: status.iconColor),
        const SizedBox(width: 4),
        Text(
          status.label,
          style: theme.textTheme.labelSmall?.copyWith(
            color: status.iconColor,
            fontWeight: FontWeight.w600,
          ),
        ),
      ],
    ),
  );

  Widget _buildLayoutInfo(ThemeData theme, _LayoutStatus status) => Tooltip(
    message:
        errorMessage ??
        'Layout Type: ${layoutType ?? 'Unknown'}\n'
            'Profile ID: ${node.id}',
    child: Container(
      padding: const EdgeInsets.all(8),
      decoration: BoxDecoration(
        color: theme.colorScheme.surfaceContainerHighest.withValues(alpha: 0.3),
        borderRadius: BorderRadius.circular(8),
      ),
      child: Row(
        children: [
          Icon(
            Icons.view_module_outlined,
            size: 14,
            color: theme.colorScheme.onSurfaceVariant,
          ),
          const SizedBox(width: 6),
          Expanded(
            child: Text(
              'Type: ${layoutType ?? 'Unknown'}',
              style: theme.textTheme.bodySmall?.copyWith(
                color: theme.colorScheme.onSurfaceVariant,
              ),
              overflow: TextOverflow.ellipsis,
            ),
          ),
        ],
      ),
    ),
  );

  Widget _buildLayoutPreview(ThemeData theme, _LayoutStatus status) {
    // Visual representation of keyboard layout
    return Container(
      height: 50,
      decoration: BoxDecoration(
        color: status.iconBackgroundColor.withValues(alpha: 0.05),
        borderRadius: BorderRadius.circular(8),
        border: Border.all(color: status.iconColor.withValues(alpha: 0.2)),
      ),
      padding: const EdgeInsets.all(8),
      child: Row(
        mainAxisAlignment: MainAxisAlignment.center,
        children: [
          // Simplified keyboard layout visualization
          Expanded(
            child: Row(
              children: List.generate(
                5,
                (i) => Expanded(
                  child: Container(
                    margin: const EdgeInsets.symmetric(horizontal: 2),
                    decoration: BoxDecoration(
                      color: status.iconColor.withValues(alpha: 0.3),
                      borderRadius: BorderRadius.circular(4),
                    ),
                  ),
                ),
              ),
            ),
          ),
        ],
      ),
    );
  }

  Widget _buildConnectionHandles(_LayoutStatus status) {
    // Connection handles on left and right edges
    return Row(
      mainAxisAlignment: MainAxisAlignment.spaceBetween,
      children: [
        // Input handle (left)
        Row(
          children: List.generate(
            2,
            (index) => Container(
              width: 8,
              height: 8,
              margin: const EdgeInsets.only(right: 4),
              decoration: BoxDecoration(
                color: status.iconColor.withValues(alpha: 0.5),
                shape: BoxShape.circle,
                border: Border.all(color: status.iconColor),
              ),
            ),
          ),
        ),
        // Output handle (right)
        Row(
          children: List.generate(
            2,
            (index) => Container(
              width: 8,
              height: 8,
              margin: const EdgeInsets.only(left: 4),
              decoration: BoxDecoration(
                color: status.iconColor.withValues(alpha: 0.5),
                shape: BoxShape.circle,
                border: Border.all(color: status.iconColor),
              ),
            ),
          ),
        ),
      ],
    );
  }

  _LayoutStatus _getLayoutStatus() =>
      isValid ? _LayoutStatus.valid : _LayoutStatus.invalid;
}

enum _LayoutStatus {
  valid,
  invalid;

  String get label {
    switch (this) {
      case _LayoutStatus.valid:
        return 'Valid';
      case _LayoutStatus.invalid:
        return 'Invalid';
    }
  }

  IconData get icon {
    switch (this) {
      case _LayoutStatus.valid:
        return Icons.grid_view_rounded;
      case _LayoutStatus.invalid:
        return Icons.grid_off_rounded;
    }
  }

  IconData get statusIcon {
    switch (this) {
      case _LayoutStatus.valid:
        return Icons.check_circle;
      case _LayoutStatus.invalid:
        return Icons.error;
    }
  }

  Color get iconColor {
    switch (this) {
      case _LayoutStatus.valid:
        return const Color(0xFF3498DB); // Blue
      case _LayoutStatus.invalid:
        return const Color(0xFFE74C3C); // Red
    }
  }

  Color get iconBackgroundColor => iconColor;

  Color borderColor(ColorScheme colorScheme) {
    switch (this) {
      case _LayoutStatus.valid:
        return const Color(0xFF3498DB);
      case _LayoutStatus.invalid:
        return const Color(0xFFE74C3C);
    }
  }
}
