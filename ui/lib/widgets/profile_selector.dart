// Profile selector dropdown widget for device-to-profile assignment.
//
// Uses FutureBuilder to asynchronously load profiles from the registry
// and displays them in a dropdown, showing the current selection.

import 'package:flutter/material.dart';
import '../services/profile_registry_service.dart';

/// Widget that provides a dropdown selector for profiles.
///
/// Loads available profiles asynchronously from the ProfileRegistryService
/// and displays them in a Material DropdownButton. Handles loading state,
/// empty state, and errors gracefully.
class ProfileSelector extends StatelessWidget {
  /// The profile registry service to load profiles from.
  final ProfileRegistryService profileService;

  /// Currently selected profile ID, or null if no profile assigned.
  final String? selectedProfileId;

  /// Callback invoked when a profile is selected from the dropdown.
  /// Called with null when "No Profile" is selected.
  final ValueChanged<String?>? onChanged;

  /// Whether the dropdown should be enabled or disabled.
  final bool enabled;

  const ProfileSelector({
    super.key,
    required this.profileService,
    this.selectedProfileId,
    this.onChanged,
    this.enabled = true,
  });

  @override
  Widget build(BuildContext context) {
    return FutureBuilder<List<String>>(
      future: profileService.listProfiles(),
      builder: (context, snapshot) {
        // Loading state
        if (snapshot.connectionState == ConnectionState.waiting) {
          return const Row(
            mainAxisSize: MainAxisSize.min,
            children: [
              SizedBox(
                width: 16,
                height: 16,
                child: CircularProgressIndicator(strokeWidth: 2),
              ),
              SizedBox(width: 8),
              Text('Loading profiles...'),
            ],
          );
        }

        // Error state
        if (snapshot.hasError) {
          return Row(
            mainAxisSize: MainAxisSize.min,
            children: [
              Icon(
                Icons.error_outline,
                color: Theme.of(context).colorScheme.error,
                size: 16,
              ),
              const SizedBox(width: 8),
              Text(
                'Failed to load profiles',
                style: TextStyle(color: Theme.of(context).colorScheme.error),
              ),
            ],
          );
        }

        final profileIds = snapshot.data ?? [];

        // Empty state
        if (profileIds.isEmpty) {
          return const Row(
            mainAxisSize: MainAxisSize.min,
            children: [
              Icon(Icons.info_outline, size: 16),
              SizedBox(width: 8),
              Text('No profiles available'),
            ],
          );
        }

        // Build dropdown items
        final items = <DropdownMenuItem<String?>>[
          // "No Profile" option
          const DropdownMenuItem<String?>(
            value: null,
            child: Text('No Profile'),
          ),
          // Profile items
          ...profileIds.map((id) {
            return DropdownMenuItem<String?>(value: id, child: Text(id));
          }),
        ];

        return DropdownButton<String?>(
          value: selectedProfileId,
          items: items,
          onChanged: enabled ? onChanged : null,
          isDense: true,
          underline: Container(
            height: 1,
            color: Theme.of(context).colorScheme.primary.withValues(alpha: 0.5),
          ),
          icon: Icon(
            Icons.arrow_drop_down,
            color: enabled
                ? Theme.of(context).colorScheme.primary
                : Theme.of(context).disabledColor,
          ),
        );
      },
    );
  }
}

