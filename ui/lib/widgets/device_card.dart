// Device card widget showing device information and controls.
//
// Displays a Material Card containing device identity info,
// ProfileSelector, RemapToggle, and action buttons for managing
// device configuration.

import 'package:flutter/material.dart';
import '../models/device_state.dart';
import '../services/device_registry_service.dart';
import '../services/profile_registry_service.dart';
import 'profile_selector.dart';
import 'remap_toggle.dart';

/// Widget that displays a device's information and controls in a card layout.
///
/// Composes RemapToggle and ProfileSelector widgets along with device
/// information display (VID:PID:Serial, label) and action buttons for
/// editing label and managing profiles.
class DeviceCard extends StatelessWidget {
  /// The device state to display
  final DeviceState deviceState;

  /// Service for device registry operations
  final DeviceRegistryService deviceService;

  /// Service for profile registry operations
  final ProfileRegistryService profileService;

  /// Callback invoked when edit label is requested
  final VoidCallback? onEditLabel;

  /// Callback invoked when manage profiles is requested
  final VoidCallback? onManageProfiles;

  const DeviceCard({
    super.key,
    required this.deviceState,
    required this.deviceService,
    required this.profileService,
    this.onEditLabel,
    this.onManageProfiles,
  });

  /// Handle remap toggle changes
  Future<void> _handleRemapToggle(bool enabled) async {
    try {
      await deviceService.toggleRemap(deviceState.identity.toKey(), enabled);
    } catch (e) {
      // TODO: Show error snackbar
      debugPrint('Failed to toggle remap: $e');
    }
  }

  /// Handle profile selection changes
  Future<void> _handleProfileChange(String? profileId) async {
    try {
      if (profileId != null) {
        await deviceService.assignProfile(deviceState.identity.toKey(), profileId);
      }
    } catch (e) {
      // TODO: Show error snackbar
      debugPrint('Failed to assign profile: $e');
    }
  }

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);
    final identity = deviceState.identity;

    return Card(
      elevation: 2,
      margin: const EdgeInsets.symmetric(vertical: 8, horizontal: 16),
      child: Padding(
        padding: const EdgeInsets.all(16),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            // Header: Device name/label
            Row(
              children: [
                Icon(
                  Icons.keyboard,
                  color: theme.colorScheme.primary,
                  size: 24,
                ),
                const SizedBox(width: 12),
                Expanded(
                  child: Text(
                    identity.displayName,
                    style: theme.textTheme.titleLarge?.copyWith(
                      fontWeight: FontWeight.bold,
                    ),
                  ),
                ),
                // Status badge
                Container(
                  padding: const EdgeInsets.symmetric(
                    horizontal: 8,
                    vertical: 4,
                  ),
                  decoration: BoxDecoration(
                    color: deviceState.remapEnabled
                        ? theme.colorScheme.primaryContainer
                        : theme.colorScheme.surfaceContainerHighest,
                    borderRadius: BorderRadius.circular(12),
                  ),
                  child: Text(
                    deviceState.statusText,
                    style: theme.textTheme.labelSmall?.copyWith(
                      color: deviceState.remapEnabled
                          ? theme.colorScheme.onPrimaryContainer
                          : theme.colorScheme.onSurfaceVariant,
                      fontWeight: FontWeight.bold,
                    ),
                  ),
                ),
              ],
            ),
            const SizedBox(height: 12),

            // Device identity information
            Text(
              identity.toKey(),
              style: theme.textTheme.bodyMedium?.copyWith(
                fontFamily: 'monospace',
                color: theme.colorScheme.onSurfaceVariant,
              ),
            ),
            const SizedBox(height: 16),

            // Controls row
            Row(
              children: [
                // Remap toggle
                Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    Text(
                      'Remap',
                      style: theme.textTheme.labelMedium?.copyWith(
                        color: theme.colorScheme.onSurfaceVariant,
                      ),
                    ),
                    const SizedBox(height: 4),
                    RemapToggle(
                      enabled: deviceState.remapEnabled,
                      onChanged: _handleRemapToggle,
                    ),
                  ],
                ),
                const SizedBox(width: 32),

                // Profile selector
                Expanded(
                  child: Column(
                    crossAxisAlignment: CrossAxisAlignment.start,
                    children: [
                      Text(
                        'Profile',
                        style: theme.textTheme.labelMedium?.copyWith(
                          color: theme.colorScheme.onSurfaceVariant,
                        ),
                      ),
                      const SizedBox(height: 4),
                      ProfileSelector(
                        profileService: profileService,
                        selectedProfileId: deviceState.profileId,
                        onChanged: _handleProfileChange,
                        enabled: true,
                      ),
                    ],
                  ),
                ),
              ],
            ),
            const SizedBox(height: 16),

            // Action buttons
            Row(
              mainAxisAlignment: MainAxisAlignment.end,
              children: [
                TextButton.icon(
                  onPressed: onEditLabel,
                  icon: const Icon(Icons.edit, size: 16),
                  label: const Text('Edit Label'),
                ),
                const SizedBox(width: 8),
                TextButton.icon(
                  onPressed: onManageProfiles,
                  icon: const Icon(Icons.settings, size: 16),
                  label: const Text('Manage Profiles'),
                ),
              ],
            ),
          ],
        ),
      ),
    );
  }
}
