/// Empty state widgets for KeyFlow with helpful onboarding messages.
library;

import 'package:flutter/material.dart';

/// Empty state displayed when no devices are connected.
///
/// Shows a friendly illustration, helpful message, and link to device
/// connection guide to help users get started with KeyFlow.
class FlowEmptyState extends StatelessWidget {
  const FlowEmptyState({super.key, this.onHelp});

  /// Optional callback when user taps the help link
  final VoidCallback? onHelp;

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);

    return Scaffold(
      backgroundColor: const Color(0xFF121212),
      body: Center(
        child: Semantics(
          label: 'No devices connected. Connect a device to get started.',
          child: Container(
            constraints: const BoxConstraints(maxWidth: 480),
            padding: const EdgeInsets.all(32),
            child: Column(
              mainAxisSize: MainAxisSize.min,
              children: [
                // Illustration: Device icon with plug icon
                Container(
                  width: 120,
                  height: 120,
                  decoration: BoxDecoration(
                    color: theme.colorScheme.surfaceContainerHighest,
                    shape: BoxShape.circle,
                  ),
                  child: Stack(
                    alignment: Alignment.center,
                    children: [
                      Icon(Icons.keyboard, size: 60, color: Colors.grey[600]),
                      Positioned(
                        bottom: 20,
                        right: 20,
                        child: Container(
                          padding: const EdgeInsets.all(4),
                          decoration: BoxDecoration(
                            color: theme.colorScheme.surface,
                            shape: BoxShape.circle,
                          ),
                          child: Icon(
                            Icons.power_off,
                            size: 24,
                            color: Colors.grey[500],
                          ),
                        ),
                      ),
                    ],
                  ),
                ),
                const SizedBox(height: 32),

                // Title
                Text(
                  'No devices connected',
                  style: theme.textTheme.headlineSmall?.copyWith(
                    color: Colors.white,
                    fontWeight: FontWeight.w600,
                  ),
                  textAlign: TextAlign.center,
                ),
                const SizedBox(height: 12),

                // Description
                Text(
                  'Connect a device to get started',
                  style: theme.textTheme.bodyLarge?.copyWith(
                    color: Colors.grey[400],
                  ),
                  textAlign: TextAlign.center,
                ),
                const SizedBox(height: 8),

                // Additional help text
                Text(
                  'KeyFlow will automatically detect connected devices '
                  'and display their slot configurations here.',
                  style: theme.textTheme.bodyMedium?.copyWith(
                    color: Colors.grey[500],
                  ),
                  textAlign: TextAlign.center,
                ),

                const SizedBox(height: 32),

                // Help link
                if (onHelp != null)
                  TextButton.icon(
                    onPressed: onHelp,
                    icon: const Icon(Icons.help_outline, size: 18),
                    label: const Text('How to connect a device'),
                    style: TextButton.styleFrom(
                      padding: const EdgeInsets.symmetric(
                        horizontal: 16,
                        vertical: 12,
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

/// Empty state displayed when devices are connected but none are active.
///
/// This is shown when all connected devices are disconnected or offline.
class FlowWaitingState extends StatelessWidget {
  const FlowWaitingState({super.key, this.deviceCount = 0});

  /// Number of devices that are disconnected
  final int deviceCount;

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);

    return Scaffold(
      backgroundColor: const Color(0xFF121212),
      body: Center(
        child: Semantics(
          label:
              'Waiting for device connection. $deviceCount device${deviceCount != 1 ? 's' : ''} detected but disconnected.',
          child: Container(
            constraints: const BoxConstraints(maxWidth: 480),
            padding: const EdgeInsets.all(32),
            child: Column(
              mainAxisSize: MainAxisSize.min,
              children: [
                // Animated waiting indicator
                SizedBox(
                  width: 80,
                  height: 80,
                  child: CircularProgressIndicator(
                    strokeWidth: 3,
                    color: theme.colorScheme.primary,
                  ),
                ),
                const SizedBox(height: 32),

                // Title
                Text(
                  'Waiting for connection',
                  style: theme.textTheme.headlineSmall?.copyWith(
                    color: Colors.white,
                    fontWeight: FontWeight.w600,
                  ),
                  textAlign: TextAlign.center,
                ),
                const SizedBox(height: 12),

                // Device count
                if (deviceCount > 0) ...[
                  Text(
                    '$deviceCount device${deviceCount != 1 ? 's' : ''} detected',
                    style: theme.textTheme.bodyLarge?.copyWith(
                      color: theme.colorScheme.primary,
                    ),
                    textAlign: TextAlign.center,
                  ),
                  const SizedBox(height: 8),
                ],

                // Description
                Text(
                  'Connect your device via USB to see its slot configuration',
                  style: theme.textTheme.bodyMedium?.copyWith(
                    color: Colors.grey[500],
                  ),
                  textAlign: TextAlign.center,
                ),
              ],
            ),
          ),
        ),
      ),
    );
  }
}

/// Empty state displayed when no slots are configured on a connected device.
///
/// Encourages users to add their first slot configuration.
class NoSlotsEmptyState extends StatelessWidget {
  const NoSlotsEmptyState({
    required this.deviceName,
    super.key,
    this.onAddSlot,
  });

  /// Name of the device with no slots
  final String deviceName;

  /// Optional callback to add a slot
  final VoidCallback? onAddSlot;

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);

    return Center(
      child: Semantics(
        label:
            'Device $deviceName has no slots configured. Add a slot to get started.',
        child: Container(
          constraints: const BoxConstraints(maxWidth: 400),
          padding: const EdgeInsets.all(24),
          decoration: BoxDecoration(
            color: Colors.black.withValues(alpha: 0.3),
            borderRadius: BorderRadius.circular(12),
            border: Border.all(color: Colors.grey[800]!),
          ),
          child: Column(
            mainAxisSize: MainAxisSize.min,
            children: [
              // Icon
              Icon(Icons.add_circle_outline, size: 48, color: Colors.grey[500]),
              const SizedBox(height: 16),

              // Title
              Text(
                'No slots configured',
                style: theme.textTheme.titleMedium?.copyWith(
                  color: Colors.white,
                  fontWeight: FontWeight.w600,
                ),
                textAlign: TextAlign.center,
              ),
              const SizedBox(height: 8),

              // Description
              Text(
                'Add a slot to assign a hardware profile and keymap to $deviceName',
                style: theme.textTheme.bodySmall?.copyWith(
                  color: Colors.grey[400],
                ),
                textAlign: TextAlign.center,
              ),

              if (onAddSlot != null) ...[
                const SizedBox(height: 16),
                FilledButton.icon(
                  onPressed: onAddSlot,
                  icon: const Icon(Icons.add, size: 18),
                  label: const Text('Add Slot'),
                ),
              ],
            ],
          ),
        ),
      ),
    );
  }
}
