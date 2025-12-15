import 'package:flutter/material.dart';
import '../../models/flow/flow_graph.dart';
import 'accessible_flow_canvas.dart';

/// Enhanced keymap node widget with Material Design 3 styling
///
/// Status colors:
/// - Green: Valid keymap loaded
/// - Red: Compilation failed or invalid
class EnhancedKeymapNode extends StatelessWidget {
  final KeymapFlowNode node;
  final bool isValid;
  final bool compilationFailed;
  final String? errorMessage;
  final VoidCallback? onTap;

  const EnhancedKeymapNode({
    super.key,
    required this.node,
    this.isValid = true,
    this.compilationFailed = false,
    this.errorMessage,
    this.onTap,
  });

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);
    final colorScheme = theme.colorScheme;
    final status = _getKeymapStatus();

    // Generate semantic label for screen readers
    final semanticLabel = node.toSemanticLabel(
      isConnected: true,
      hasError: compilationFailed || !isValid,
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
            width: 200,
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
                _buildKeymapInfo(theme, status),
                const SizedBox(height: 12),
                _buildLayerPreview(theme, status),
                const SizedBox(height: 8),
                _buildConnectionHandle(status),
              ],
            ),
          ),
        ),
      ),
    );
  }

  Widget _buildHeader(ThemeData theme, _KeymapStatus status) {
    return Row(
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
                node.label ?? 'Unknown Keymap',
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
  }

  Widget _buildStatusChip(ThemeData theme, _KeymapStatus status) {
    return Container(
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
  }

  Widget _buildKeymapInfo(ThemeData theme, _KeymapStatus status) {
    final layerCount = node.layers ?? 0;
    final tooltipMessage =
        errorMessage ??
        'Keymap ID: ${node.id}\n'
            'Layers: $layerCount';

    return Tooltip(
      message: tooltipMessage,
      child: Container(
        padding: const EdgeInsets.all(8),
        decoration: BoxDecoration(
          color: theme.colorScheme.surfaceContainerHighest.withValues(
            alpha: 0.3,
          ),
          borderRadius: BorderRadius.circular(8),
        ),
        child: Row(
          children: [
            Icon(Icons.layers, size: 14, color: status.iconColor),
            const SizedBox(width: 6),
            Text(
              '$layerCount layer${layerCount != 1 ? 's' : ''}',
              style: theme.textTheme.bodySmall?.copyWith(
                color: theme.colorScheme.onSurfaceVariant,
                fontWeight: FontWeight.w500,
              ),
            ),
          ],
        ),
      ),
    );
  }

  Widget _buildLayerPreview(ThemeData theme, _KeymapStatus status) {
    final layerCount = (node.layers ?? 0).clamp(0, 4);

    return Container(
      height: 60,
      decoration: BoxDecoration(
        color: status.iconBackgroundColor.withValues(alpha: 0.05),
        borderRadius: BorderRadius.circular(8),
        border: Border.all(color: status.iconColor.withValues(alpha: 0.2)),
      ),
      padding: const EdgeInsets.all(8),
      child: Column(
        mainAxisAlignment: MainAxisAlignment.center,
        children: [
          // Visual stack of layers
          for (int i = 0; i < layerCount; i++)
            Container(
              height: 10,
              margin: EdgeInsets.only(
                bottom: i < layerCount - 1 ? 2 : 0,
                left: i * 4.0,
                right: (layerCount - i - 1) * 4.0,
              ),
              decoration: BoxDecoration(
                color: status.iconColor.withValues(alpha: 0.7 - (i * 0.15)),
                borderRadius: BorderRadius.circular(3),
                border: Border.all(
                  color: status.iconColor.withValues(alpha: 0.5),
                  width: 0.5,
                ),
              ),
            ),
        ],
      ),
    );
  }

  Widget _buildConnectionHandle(_KeymapStatus status) {
    // Input connection handle on the left
    return Align(
      alignment: Alignment.centerLeft,
      child: Row(
        mainAxisSize: MainAxisSize.min,
        children: List.generate(
          2,
          (index) => Container(
            width: 8,
            height: 8,
            margin: const EdgeInsets.only(right: 4),
            decoration: BoxDecoration(
              color: status.iconColor.withValues(alpha: 0.5),
              shape: BoxShape.circle,
              border: Border.all(color: status.iconColor, width: 1),
            ),
          ),
        ),
      ),
    );
  }

  _KeymapStatus _getKeymapStatus() {
    if (compilationFailed || !isValid) {
      return _KeymapStatus.invalid;
    }
    return _KeymapStatus.valid;
  }
}

enum _KeymapStatus {
  valid,
  invalid;

  String get label {
    switch (this) {
      case _KeymapStatus.valid:
        return 'Valid';
      case _KeymapStatus.invalid:
        return 'Failed';
    }
  }

  IconData get icon {
    switch (this) {
      case _KeymapStatus.valid:
        return Icons.code;
      case _KeymapStatus.invalid:
        return Icons.code_off;
    }
  }

  IconData get statusIcon {
    switch (this) {
      case _KeymapStatus.valid:
        return Icons.check_circle;
      case _KeymapStatus.invalid:
        return Icons.error;
    }
  }

  Color get iconColor {
    switch (this) {
      case _KeymapStatus.valid:
        return const Color(0xFF2ECC71); // Green
      case _KeymapStatus.invalid:
        return const Color(0xFFE74C3C); // Red
    }
  }

  Color get iconBackgroundColor {
    return iconColor;
  }

  Color borderColor(ColorScheme colorScheme) {
    switch (this) {
      case _KeymapStatus.valid:
        return const Color(0xFF2ECC71);
      case _KeymapStatus.invalid:
        return const Color(0xFFE74C3C);
    }
  }
}
