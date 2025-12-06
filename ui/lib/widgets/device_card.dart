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
class DeviceCard extends StatefulWidget {
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

  /// Callback invoked when the device state changes optimistically.
  final ValueChanged<DeviceState>? onDeviceUpdated;

  const DeviceCard({
    super.key,
    required this.deviceState,
    required this.deviceService,
    required this.profileService,
    this.onEditLabel,
    this.onManageProfiles,
    this.onDeviceUpdated,
  });

  @override
  State<DeviceCard> createState() => _DeviceCardState();
}

class _DeviceCardState extends State<DeviceCard> {
  late DeviceState _currentState;
  late bool _remapEnabled;
  String? _selectedProfileId;
  bool _remapInFlight = false;
  bool _profileInFlight = false;

  @override
  void initState() {
    super.initState();
    _currentState = widget.deviceState;
    _remapEnabled = widget.deviceState.remapEnabled;
    _selectedProfileId = widget.deviceState.profileId;
  }

  @override
  void didUpdateWidget(covariant DeviceCard oldWidget) {
    super.didUpdateWidget(oldWidget);
    if (oldWidget.deviceState != widget.deviceState) {
      _currentState = widget.deviceState;
      _remapEnabled = widget.deviceState.remapEnabled;
      _selectedProfileId = widget.deviceState.profileId;
    }
  }

  bool get _actionsDisabled => _remapInFlight || _profileInFlight;
  String get _statusText {
    if (!_remapEnabled) {
      return 'Passthrough';
    }
    if (_selectedProfileId != null) {
      return 'Active';
    }
    return 'No Profile';
  }

  /// Handle remap toggle changes with optimistic UI and rollback on failure.
  Future<void> _handleRemapToggle(bool enabled) async {
    final previous = _remapEnabled;
    setState(() {
      _remapEnabled = enabled;
      _remapInFlight = true;
    });

    final result = await widget.deviceService
        .toggleRemap(_currentState.identity.toKey(), enabled);

    if (!mounted) return;

    if (!result.success) {
      setState(() {
        _remapEnabled = previous;
        _remapInFlight = false;
      });
      _showSnack(
        result.errorMessage ??
            'Failed to ${enabled ? 'enable' : 'disable'} remap.',
        isError: true,
      );
      return;
    }

    final updatedState = _currentState.copyWith(remapEnabled: enabled);
    setState(() {
      _currentState = updatedState;
      _remapInFlight = false;
    });
    widget.onDeviceUpdated?.call(updatedState);
    _showSnack(enabled ? 'Remap enabled' : 'Remap disabled');
  }

  /// Handle profile selection changes with optimistic UI and rollback on failure.
  Future<void> _handleProfileChange(String? profileId) async {
    if (profileId == null) {
      _showSnack('Select a profile to assign', isError: true);
      return;
    }

    final previous = _selectedProfileId;
    setState(() {
      _selectedProfileId = profileId;
      _profileInFlight = true;
    });

    final result = await widget.deviceService
        .assignProfile(_currentState.identity.toKey(), profileId);

    if (!mounted) return;

    if (!result.success) {
      setState(() {
        _selectedProfileId = previous;
        _profileInFlight = false;
      });
      _showSnack(
        result.errorMessage ?? 'Failed to assign profile.',
        isError: true,
      );
      return;
    }

    final updatedState = _currentState.copyWith(profileId: profileId);
    setState(() {
      _currentState = updatedState;
      _profileInFlight = false;
    });
    widget.onDeviceUpdated?.call(updatedState);
    _showSnack('Profile assigned');
  }

  void _showSnack(String message, {bool isError = false}) {
    ScaffoldMessenger.of(context).showSnackBar(
      SnackBar(
        content: Text(message),
        backgroundColor:
            isError ? Theme.of(context).colorScheme.error : null,
      ),
    );
  }

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);
    final identity = _currentState.identity;

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
                    color: _remapEnabled
                        ? theme.colorScheme.primaryContainer
                        : theme.colorScheme.surfaceContainerHighest,
                    borderRadius: BorderRadius.circular(12),
                  ),
                  child: Text(
                    _statusText,
                    style: theme.textTheme.labelSmall?.copyWith(
                      color: _remapEnabled
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
                      enabled: _remapEnabled,
                      onChanged: _actionsDisabled ? null : _handleRemapToggle,
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
                        profileService: widget.profileService,
                        selectedProfileId: _selectedProfileId,
                        onChanged: _actionsDisabled ? null : _handleProfileChange,
                        enabled: !_actionsDisabled,
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
                  onPressed: widget.onEditLabel,
                  icon: const Icon(Icons.edit, size: 16),
                  label: const Text('Edit Label'),
                ),
                const SizedBox(width: 8),
                TextButton.icon(
                  onPressed: widget.onManageProfiles,
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
