import 'package:flutter/material.dart';
import '../../models/flow/flow_graph.dart';
import 'accessible_flow_canvas.dart';

/// Enhanced device node widget with Material Design 3 styling
///
/// Status colors:
/// - Green: Connected with active slots
/// - Blue: Connected with no active slots
/// - Red: Error state
/// - Grey: Disconnected
class EnhancedDeviceNode extends StatelessWidget {
  const EnhancedDeviceNode({
    required this.node,
    super.key,
    this.activeSlotCount = 0,
    this.totalSlotCount = 0,
    this.hasError = false,
    this.deviceEnabled = true,
    this.onAddSlot,
    this.onTap,
  });
  final DeviceFlowNode node;
  final int activeSlotCount;
  final int totalSlotCount;
  final bool hasError;
  final bool deviceEnabled;
  final VoidCallback? onAddSlot;
  final VoidCallback? onTap;

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);
    final colorScheme = theme.colorScheme;
    final status = _getDeviceStatus();

    // Generate semantic label for screen readers
    final semanticLabel = node.toSemanticLabel(
      isConnected: node.isConnected,
      activeSlotCount: activeSlotCount,
      totalSlotCount: totalSlotCount,
      hasError: hasError,
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
            width: 260,
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
                _buildDeviceInfo(theme),
                const SizedBox(height: 12),
                _buildSlotInfo(theme, status),
                if (onAddSlot != null) ...[
                  const SizedBox(height: 12),
                  _buildAddSlotButton(theme),
                ],
                const SizedBox(height: 8),
                _buildConnectionHandles(status),
              ],
            ),
          ),
        ),
      ),
    );
  }

  Widget _buildHeader(ThemeData theme, _DeviceStatus status) => Row(
    children: [
      // Status icon with colored background
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
              node.label ?? 'Unknown Device',
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

  Widget _buildStatusChip(ThemeData theme, _DeviceStatus status) => Container(
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
        Container(
          width: 6,
          height: 6,
          decoration: BoxDecoration(
            color: status.iconColor,
            shape: BoxShape.circle,
          ),
        ),
        const SizedBox(width: 6),
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

  Widget _buildDeviceInfo(ThemeData theme) => Tooltip(
    message:
        'VID: ${node.deviceId.vendorId}, PID: ${node.deviceId.productId}\n'
        'Serial: ${node.deviceId.serial ?? "N/A"}',
    child: Container(
      padding: const EdgeInsets.all(8),
      decoration: BoxDecoration(
        color: theme.colorScheme.surfaceContainerHighest.withValues(alpha: 0.3),
        borderRadius: BorderRadius.circular(8),
      ),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Row(
            children: [
              Icon(
                Icons.fingerprint,
                size: 14,
                color: theme.colorScheme.onSurfaceVariant,
              ),
              const SizedBox(width: 6),
              Text(
                'VID:${node.deviceId.vendorId.toRadixString(16).toUpperCase().padLeft(4, '0')} '
                'PID:${node.deviceId.productId.toRadixString(16).toUpperCase().padLeft(4, '0')}',
                style: theme.textTheme.bodySmall?.copyWith(
                  fontFamily: 'monospace',
                  color: theme.colorScheme.onSurfaceVariant,
                ),
              ),
            ],
          ),
          if (node.deviceId.serial != null &&
              node.deviceId.serial!.isNotEmpty) ...[
            const SizedBox(height: 4),
            Row(
              children: [
                Icon(
                  Icons.tag,
                  size: 14,
                  color: theme.colorScheme.onSurfaceVariant,
                ),
                const SizedBox(width: 6),
                Expanded(
                  child: Text(
                    node.deviceId.serial!,
                    style: theme.textTheme.bodySmall?.copyWith(
                      fontFamily: 'monospace',
                      color: theme.colorScheme.onSurfaceVariant,
                    ),
                    overflow: TextOverflow.ellipsis,
                  ),
                ),
              ],
            ),
          ],
        ],
      ),
    ),
  );

  Widget _buildSlotInfo(ThemeData theme, _DeviceStatus status) {
    final inactiveSlotCount = totalSlotCount - activeSlotCount;

    return Row(
      children: [
        Icon(Icons.layers_outlined, size: 16, color: status.iconColor),
        const SizedBox(width: 6),
        Text(
          '$totalSlotCount slot${totalSlotCount != 1 ? 's' : ''}',
          style: theme.textTheme.bodyMedium?.copyWith(
            fontWeight: FontWeight.w500,
          ),
        ),
        if (totalSlotCount > 0) ...[
          const SizedBox(width: 8),
          Flexible(
            child: Container(
              padding: const EdgeInsets.symmetric(horizontal: 6, vertical: 2),
              decoration: BoxDecoration(
                color: status.iconBackgroundColor.withValues(alpha: 0.15),
                borderRadius: BorderRadius.circular(8),
              ),
              child: Text(
                '$activeSlotCount active, $inactiveSlotCount inactive',
                style: theme.textTheme.labelSmall?.copyWith(
                  color: status.iconColor,
                  fontWeight: FontWeight.w600,
                ),
                overflow: TextOverflow.ellipsis,
                maxLines: 1,
              ),
            ),
          ),
        ],
      ],
    );
  }

  Widget _buildAddSlotButton(ThemeData theme) => Semantics(
    button: true,
    label: 'Add slot to ${node.label ?? "device"}',
    child: SizedBox(
      width: double.infinity,
      // Ensure minimum touch target height
      height: 48,
      child: OutlinedButton.icon(
        onPressed: onAddSlot,
        icon: const Icon(Icons.add, size: 18),
        label: const Text('Add Slot'),
        style: OutlinedButton.styleFrom(
          padding: const EdgeInsets.symmetric(vertical: 12, horizontal: 16),
          side: BorderSide(
            color: theme.colorScheme.outline.withValues(alpha: 0.5),
          ),
        ),
      ),
    ),
  );

  Widget _buildConnectionHandles(_DeviceStatus status) {
    // Visual connection handles on the right edge
    return Align(
      alignment: Alignment.centerRight,
      child: Wrap(
        spacing: 4,
        runSpacing: 4,
        children: List.generate(
          totalSlotCount.clamp(0, 5),
          (index) => Container(
            width: 8,
            height: 8,
            decoration: BoxDecoration(
              color: index < activeSlotCount
                  ? status.iconColor
                  : status.iconColor.withValues(alpha: 0.3),
              shape: BoxShape.circle,
              border: Border.all(
                color: status.iconColor.withValues(alpha: 0.5),
              ),
            ),
          ),
        ),
      ),
    );
  }

  _DeviceStatus _getDeviceStatus() {
    if (hasError) {
      return _DeviceStatus.error;
    }
    if (!node.isConnected || !deviceEnabled) {
      return _DeviceStatus.disconnected;
    }
    if (activeSlotCount > 0) {
      return _DeviceStatus.connectedActive;
    }
    return _DeviceStatus.connectedInactive;
  }
}

enum _DeviceStatus {
  connectedActive,
  connectedInactive,
  error,
  disconnected;

  String get label {
    switch (this) {
      case _DeviceStatus.connectedActive:
        return 'Connected';
      case _DeviceStatus.connectedInactive:
        return 'Connected';
      case _DeviceStatus.error:
        return 'Error';
      case _DeviceStatus.disconnected:
        return 'Disconnected';
    }
  }

  IconData get icon {
    switch (this) {
      case _DeviceStatus.connectedActive:
        return Icons.keyboard;
      case _DeviceStatus.connectedInactive:
        return Icons.keyboard_outlined;
      case _DeviceStatus.error:
        return Icons.error_outline;
      case _DeviceStatus.disconnected:
        return Icons.keyboard_hide;
    }
  }

  Color get iconColor {
    switch (this) {
      case _DeviceStatus.connectedActive:
        return const Color(0xFF2ECC71); // Green
      case _DeviceStatus.connectedInactive:
        return const Color(0xFF3498DB); // Blue
      case _DeviceStatus.error:
        return const Color(0xFFE74C3C); // Red
      case _DeviceStatus.disconnected:
        return const Color(0xFF95A5A6); // Grey
    }
  }

  Color get iconBackgroundColor => iconColor;

  Color borderColor(ColorScheme colorScheme) {
    switch (this) {
      case _DeviceStatus.connectedActive:
        return const Color(0xFF2ECC71);
      case _DeviceStatus.connectedInactive:
        return const Color(0xFF3498DB);
      case _DeviceStatus.error:
        return const Color(0xFFE74C3C);
      case _DeviceStatus.disconnected:
        return colorScheme.outline.withValues(alpha: 0.3);
    }
  }
}
