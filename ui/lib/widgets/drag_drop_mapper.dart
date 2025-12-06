// Drag-and-drop style mapper widget composing LayoutGrid and SoftKeyboard.
//
// Implements a two-step mapping workflow:
// 1. Select a physical key from the layout
// 2. Select an output key from the palette
// 3. Auto-save the profile on mapping changes
//
// Provides an intuitive UX for creating and editing key mappings.

import 'package:flutter/material.dart';
import '../models/layout_type.dart';
import '../models/profile.dart';
import '../services/profile_registry_service.dart';
import 'layout_grid.dart';
import 'soft_keyboard.dart';

/// State of the mapping workflow
enum MappingState {
  /// Waiting for user to select a physical key
  selectingPhysical,

  /// Physical key selected, waiting for output key selection
  selectingOutput,
}

/// Widget that coordinates the mapping workflow between LayoutGrid and SoftKeyboard.
///
/// Implements a two-step selection process:
/// 1. User clicks a physical key in the layout
/// 2. User clicks an output key in the palette
/// 3. Mapping is created and profile is auto-saved
///
/// The widget manages state internally and auto-saves on each mapping change.
class DragDropMapper extends StatefulWidget {
  /// Layout information for the device
  final LayoutInfo layoutInfo;

  /// Current profile being edited
  final Profile profile;

  /// Profile registry service for auto-saving
  final ProfileRegistryService profileRegistryService;

  /// Callback when profile is updated (optional)
  final void Function(Profile updatedProfile)? onProfileUpdated;

  /// Callback when a save error occurs (optional)
  final void Function(String errorMessage)? onSaveError;

  const DragDropMapper({
    super.key,
    required this.layoutInfo,
    required this.profile,
    required this.profileRegistryService,
    this.onProfileUpdated,
    this.onSaveError,
  });

  @override
  State<DragDropMapper> createState() => _DragDropMapperState();
}

class _DragDropMapperState extends State<DragDropMapper> {
  /// Current workflow state
  MappingState _state = MappingState.selectingPhysical;

  /// Currently selected physical position
  PhysicalPosition? _selectedPhysicalPosition;

  /// Currently selected output key variant
  String? _selectedOutputKey;

  /// Current profile (mutable copy)
  late Profile _currentProfile;

  /// Whether a save operation is in progress
  bool _isSaving = false;

  @override
  void initState() {
    super.initState();
    _currentProfile = widget.profile;
  }

  @override
  void didUpdateWidget(DragDropMapper oldWidget) {
    super.didUpdateWidget(oldWidget);
    // Update profile if it changed externally
    if (widget.profile != oldWidget.profile) {
      _currentProfile = widget.profile;
    }
  }

  /// Handle physical key tap from LayoutGrid
  void _onPhysicalKeyTap(int row, int col) {
    setState(() {
      _selectedPhysicalPosition = PhysicalPosition(row: row, col: col);
      _state = MappingState.selectingOutput;
      _selectedOutputKey = null;

      // Check if this position already has a mapping
      final existingAction = _currentProfile.getAction(_selectedPhysicalPosition!);
      if (existingAction != null) {
        // Pre-select the current mapping in the soft keyboard
        existingAction.when(
          key: (key) {
            _selectedOutputKey = key;
          },
          chord: (_) {
            // For chords, don't pre-select (would need multi-select)
            _selectedOutputKey = null;
          },
          script: (_) {
            _selectedOutputKey = null;
          },
          block: () {
            _selectedOutputKey = null;
          },
          pass: () {
            _selectedOutputKey = null;
          },
        );
      }
    });
  }

  /// Handle output key selection from SoftKeyboard
  void _onOutputKeySelected(String keyVariant) async {
    if (_selectedPhysicalPosition == null) {
      // Should not happen, but handle gracefully
      return;
    }

    setState(() {
      _selectedOutputKey = keyVariant;
    });

    // Create the mapping and auto-save
    await _createMappingAndSave(keyVariant);
  }

  /// Create a mapping and auto-save the profile
  Future<void> _createMappingAndSave(String outputKeyVariant) async {
    if (_selectedPhysicalPosition == null) {
      return;
    }

    final positionKey = _selectedPhysicalPosition!.toKey();
    final newAction = KeyAction.key(key: outputKeyVariant);

    // Update profile with new mapping
    final updatedMappings = Map<String, KeyAction>.from(_currentProfile.mappings);
    updatedMappings[positionKey] = newAction;

    final updatedProfile = _currentProfile.copyWith(
      mappings: updatedMappings,
      updatedAt: DateTime.now().toUtc().toIso8601String(),
    );

    setState(() {
      _currentProfile = updatedProfile;
      _isSaving = true;
    });

    // Auto-save the profile
    final result = await widget.profileRegistryService.saveProfile(updatedProfile);

    setState(() {
      _isSaving = false;
    });

    if (result.success) {
      // Notify parent of update
      widget.onProfileUpdated?.call(updatedProfile);

      // Reset to selecting physical for next mapping
      setState(() {
        _selectedPhysicalPosition = null;
        _selectedOutputKey = null;
        _state = MappingState.selectingPhysical;
      });
    } else {
      // Handle save error
      widget.onSaveError?.call(result.errorMessage ?? 'Failed to save profile');

      // Revert the profile change
      setState(() {
        _currentProfile = widget.profile;
      });
    }
  }

  /// Remove a mapping from the profile
  Future<void> _removeMapping(PhysicalPosition position) async {
    final positionKey = position.toKey();

    // Update profile by removing the mapping
    final updatedMappings = Map<String, KeyAction>.from(_currentProfile.mappings);
    updatedMappings.remove(positionKey);

    final updatedProfile = _currentProfile.copyWith(
      mappings: updatedMappings,
      updatedAt: DateTime.now().toUtc().toIso8601String(),
    );

    setState(() {
      _currentProfile = updatedProfile;
      _isSaving = true;
    });

    // Auto-save the profile
    final result = await widget.profileRegistryService.saveProfile(updatedProfile);

    setState(() {
      _isSaving = false;
    });

    if (result.success) {
      // Notify parent of update
      widget.onProfileUpdated?.call(updatedProfile);
    } else {
      // Handle save error
      widget.onSaveError?.call(result.errorMessage ?? 'Failed to save profile');

      // Revert the profile change
      setState(() {
        _currentProfile = widget.profile;
      });
    }
  }

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);

    return Column(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        // Status bar showing current state
        _buildStatusBar(theme),

        const SizedBox(height: 16),

        // Split view: LayoutGrid on left, SoftKeyboard on right
        Expanded(
          child: Row(
            crossAxisAlignment: CrossAxisAlignment.stretch,
            children: [
              // Left side: Device layout
              Expanded(
                flex: 1,
                child: Card(
                  child: Padding(
                    padding: const EdgeInsets.all(16.0),
                    child: Column(
                      crossAxisAlignment: CrossAxisAlignment.stretch,
                      children: [
                        Row(
                          mainAxisAlignment: MainAxisAlignment.spaceBetween,
                          children: [
                            Text(
                              'Device Layout',
                              style: theme.textTheme.titleMedium,
                            ),
                            if (_isSaving)
                              Row(
                                children: [
                                  const SizedBox(
                                    width: 12,
                                    height: 12,
                                    child: CircularProgressIndicator(
                                      strokeWidth: 2,
                                    ),
                                  ),
                                  const SizedBox(width: 8),
                                  Text(
                                    'Saving...',
                                    style: theme.textTheme.bodySmall,
                                  ),
                                ],
                              ),
                          ],
                        ),
                        const SizedBox(height: 8),
                        Text(
                          'Click a key to remap',
                          style: theme.textTheme.bodySmall?.copyWith(
                            color: theme.colorScheme.onSurface.withOpacity(0.6),
                          ),
                        ),
                        const SizedBox(height: 16),
                        Expanded(
                          child: SingleChildScrollView(
                            child: LayoutGrid(
                              layoutInfo: widget.layoutInfo,
                              profile: _currentProfile,
                              onKeyTap: _onPhysicalKeyTap,
                              selectedPosition: _selectedPhysicalPosition,
                            ),
                          ),
                        ),
                      ],
                    ),
                  ),
                ),
              ),

              const SizedBox(width: 16),

              // Right side: Output key palette
              Expanded(
                flex: 1,
                child: Card(
                  child: Padding(
                    padding: const EdgeInsets.all(16.0),
                    child: Column(
                      crossAxisAlignment: CrossAxisAlignment.stretch,
                      children: [
                        Text(
                          'Output Keys',
                          style: theme.textTheme.titleMedium,
                        ),
                        const SizedBox(height: 8),
                        Text(
                          _state == MappingState.selectingPhysical
                              ? 'Select a physical key first'
                              : 'Select an output key to map',
                          style: theme.textTheme.bodySmall?.copyWith(
                            color: theme.colorScheme.onSurface.withOpacity(0.6),
                          ),
                        ),
                        const SizedBox(height: 16),
                        Expanded(
                          child: _state == MappingState.selectingPhysical
                              ? _buildDisabledPaletteOverlay(theme)
                              : SoftKeyboard(
                                  onKeySelected: _onOutputKeySelected,
                                  selectedKey: _selectedOutputKey,
                                ),
                        ),
                      ],
                    ),
                  ),
                ),
              ),
            ],
          ),
        ),

        // Action buttons
        if (_selectedPhysicalPosition != null &&
            _currentProfile.getAction(_selectedPhysicalPosition!) != null)
          Padding(
            padding: const EdgeInsets.only(top: 16.0),
            child: Row(
              mainAxisAlignment: MainAxisAlignment.center,
              children: [
                ElevatedButton.icon(
                  onPressed: _isSaving
                      ? null
                      : () => _removeMapping(_selectedPhysicalPosition!),
                  icon: const Icon(Icons.delete),
                  label: const Text('Remove Mapping'),
                  style: ElevatedButton.styleFrom(
                    backgroundColor: theme.colorScheme.errorContainer,
                    foregroundColor: theme.colorScheme.onErrorContainer,
                  ),
                ),
              ],
            ),
          ),
      ],
    );
  }

  /// Build the status bar showing current workflow state
  Widget _buildStatusBar(ThemeData theme) {
    final statusColor = _state == MappingState.selectingPhysical
        ? theme.colorScheme.primary
        : theme.colorScheme.secondary;

    final statusText = _state == MappingState.selectingPhysical
        ? 'Step 1: Select a physical key from the layout'
        : 'Step 2: Select an output key from the palette';

    final statusIcon =
        _state == MappingState.selectingPhysical ? Icons.keyboard : Icons.check;

    return Container(
      padding: const EdgeInsets.all(12.0),
      decoration: BoxDecoration(
        color: statusColor.withOpacity(0.1),
        borderRadius: BorderRadius.circular(8.0),
        border: Border.all(color: statusColor, width: 1),
      ),
      child: Row(
        children: [
          Icon(statusIcon, color: statusColor, size: 20),
          const SizedBox(width: 8),
          Expanded(
            child: Text(
              statusText,
              style: theme.textTheme.bodyMedium?.copyWith(
                color: statusColor,
                fontWeight: FontWeight.w500,
              ),
            ),
          ),
          if (_selectedPhysicalPosition != null) ...[
            const SizedBox(width: 8),
            Container(
              padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 4),
              decoration: BoxDecoration(
                color: statusColor,
                borderRadius: BorderRadius.circular(4),
              ),
              child: Text(
                'Selected: ${_selectedPhysicalPosition!.toKey()}',
                style: theme.textTheme.bodySmall?.copyWith(
                  color: theme.colorScheme.onPrimary,
                  fontWeight: FontWeight.w500,
                ),
              ),
            ),
          ],
        ],
      ),
    );
  }

  /// Build disabled palette overlay when no physical key is selected
  Widget _buildDisabledPaletteOverlay(ThemeData theme) {
    return LayoutBuilder(
      builder: (context, constraints) {
        return SingleChildScrollView(
          child: ConstrainedBox(
            constraints: BoxConstraints(minHeight: constraints.maxHeight),
            child: Center(
              child: Padding(
                padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 12),
                child: Column(
                  mainAxisSize: MainAxisSize.min,
                  children: [
                    Icon(
                      Icons.touch_app,
                      size: 64,
                      color: theme.disabledColor,
                    ),
                    const SizedBox(height: 16),
                    Text(
                      'Select a physical key first',
                      style: theme.textTheme.titleMedium?.copyWith(
                        color: theme.disabledColor,
                      ),
                    ),
                    const SizedBox(height: 8),
                    Text(
                      'Click any key in the device layout to begin mapping',
                      style: theme.textTheme.bodyMedium?.copyWith(
                        color: theme.disabledColor,
                      ),
                      textAlign: TextAlign.center,
                    ),
                  ],
                ),
              ),
            ),
          ),
        );
      },
    );
  }
}
