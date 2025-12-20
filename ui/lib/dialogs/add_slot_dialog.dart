/// Dialog for adding a new slot to a device.
library;

import 'package:core_logic/models/config_ids.dart';
import 'package:core_logic/models/hardware_profile.dart';
import 'package:core_logic/models/keymap.dart';
import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../state/keyflow_providers.dart';
import '../state/slot_actions.dart';

/// Dialog for adding a new device slot.
///
/// Allows the user to select a hardware profile and keymap to create a new
/// slot assignment for a specific device.
class AddSlotDialog extends ConsumerStatefulWidget {
  const AddSlotDialog({required this.device, required this.existingSlots, super.key});

  final DeviceInstanceId device;
  final List<ProfileSlotSelection> existingSlots;

  @override
  ConsumerState<AddSlotDialog> createState() => _AddSlotDialogState();
}

class _AddSlotDialogState extends ConsumerState<AddSlotDialog> {
  HardwareProfileId? _selectedProfileId;
  KeymapId? _selectedKeymapId;
  bool _isActive = false;
  bool _isSaving = false;
  String? _errorMessage;

  @override
  Widget build(BuildContext context) {
    final hardwareProfilesAsync = ref.watch(availableHardwareProfilesProvider);
    final keymapsAsync = ref.watch(availableKeymapsProvider);

    return AlertDialog(
      title: const Text('Add Slot'),
      content: SizedBox(
        width: 400,
        child: Column(
          mainAxisSize: MainAxisSize.min,
          crossAxisAlignment: CrossAxisAlignment.stretch,
          children: [
            if (_errorMessage != null) ...[
              Container(
                padding: const EdgeInsets.all(8),
                decoration: BoxDecoration(
                  color: Colors.red.withValues(alpha: 0.1),
                  borderRadius: BorderRadius.circular(4),
                  border: Border.all(color: Colors.red.withValues(alpha: 0.3)),
                ),
                child: Row(
                  children: [
                    const Icon(Icons.error_outline, color: Colors.red, size: 20),
                    const SizedBox(width: 8),
                    Expanded(
                      child: Text(
                        _errorMessage!,
                        style: const TextStyle(color: Colors.red, fontSize: 13),
                      ),
                    ),
                  ],
                ),
              ),
              const SizedBox(height: 16),
            ],
            const Text(
              'Hardware Profile',
              style: TextStyle(fontWeight: FontWeight.w600, fontSize: 13),
            ),
            const SizedBox(height: 8),
            hardwareProfilesAsync.when(
              data: _buildProfileDropdown,
              loading: () => const LinearProgressIndicator(),
              error: (error, _) => Text(
                'Failed to load profiles: $error',
                style: const TextStyle(color: Colors.red, fontSize: 12),
              ),
            ),
            const SizedBox(height: 16),
            const Text('Keymap', style: TextStyle(fontWeight: FontWeight.w600, fontSize: 13)),
            const SizedBox(height: 8),
            keymapsAsync.when(
              data: _buildKeymapDropdown,
              loading: () => const LinearProgressIndicator(),
              error: (error, _) => Text(
                'Failed to load keymaps: $error',
                style: const TextStyle(color: Colors.red, fontSize: 12),
              ),
            ),
            const SizedBox(height: 16),
            CheckboxListTile(
              title: const Text('Active', style: TextStyle(fontSize: 13)),
              subtitle: const Text(
                'Mark this slot as currently active',
                style: TextStyle(fontSize: 11),
              ),
              value: _isActive,
              onChanged: (value) {
                setState(() {
                  _isActive = value ?? false;
                });
              },
              controlAffinity: ListTileControlAffinity.leading,
              contentPadding: EdgeInsets.zero,
            ),
          ],
        ),
      ),
      actions: [
        TextButton(
          onPressed: _isSaving ? null : () => Navigator.of(context).pop(),
          child: const Text('Cancel'),
        ),
        FilledButton(
          onPressed: _isSaving ? null : _handleAdd,
          child: _isSaving
              ? const SizedBox(
                  width: 16,
                  height: 16,
                  child: CircularProgressIndicator(strokeWidth: 2),
                )
              : const Text('Add'),
        ),
      ],
    );
  }

  Widget _buildProfileDropdown(List<HardwareProfile> profiles) {
    if (profiles.isEmpty) {
      return const Text(
        'No hardware profiles available',
        style: TextStyle(color: Colors.grey, fontSize: 12),
      );
    }

    return DropdownButtonFormField<HardwareProfileId>(
      initialValue: _selectedProfileId,
      decoration: const InputDecoration(
        border: OutlineInputBorder(),
        contentPadding: EdgeInsets.symmetric(horizontal: 12, vertical: 8),
        isDense: true,
      ),
      hint: const Text('Select a hardware profile', style: TextStyle(fontSize: 13)),
      items: profiles
          .map(
            (profile) => DropdownMenuItem(
              value: profile.id,
              child: Text(profile.name ?? profile.id, style: const TextStyle(fontSize: 13)),
            ),
          )
          .toList(),
      onChanged: (value) {
        setState(() {
          _selectedProfileId = value;
          _errorMessage = null;
        });
      },
    );
  }

  Widget _buildKeymapDropdown(List<Keymap> keymaps) {
    if (keymaps.isEmpty) {
      return const Text('No keymaps available', style: TextStyle(color: Colors.grey, fontSize: 12));
    }

    return DropdownButtonFormField<KeymapId>(
      initialValue: _selectedKeymapId,
      decoration: const InputDecoration(
        border: OutlineInputBorder(),
        contentPadding: EdgeInsets.symmetric(horizontal: 12, vertical: 8),
        isDense: true,
      ),
      hint: const Text('Select a keymap', style: TextStyle(fontSize: 13)),
      items: keymaps
          .map(
            (keymap) => DropdownMenuItem(
              value: keymap.id,
              child: Text(keymap.name, style: const TextStyle(fontSize: 13)),
            ),
          )
          .toList(),
      onChanged: (value) {
        setState(() {
          _selectedKeymapId = value;
          _errorMessage = null;
        });
      },
    );
  }

  Future<void> _handleAdd() async {
    if (!_validate()) return;

    setState(() {
      _isSaving = true;
      _errorMessage = null;
    });

    try {
      final slotActions = ref.read(slotActionsProvider);
      final result = await slotActions.addSlot(
        device: widget.device,
        hardwareProfileId: _selectedProfileId!,
        keymapId: _selectedKeymapId!,
        active: _isActive,
      );

      if (!mounted) return;

      if (result.hasError) {
        setState(() {
          _errorMessage = result.errorMessage ?? 'Failed to add slot';
          _isSaving = false;
        });
      } else {
        Navigator.of(context).pop(true);
      }
    } on Exception {
      if (!mounted) return;
      setState(() {
        _errorMessage = 'Error adding slot: $e';
        _isSaving = false;
      });
    }
  }

  bool _validate() {
    if (_selectedProfileId == null) {
      setState(() {
        _errorMessage = 'Please select a hardware profile';
      });
      return false;
    }

    if (_selectedKeymapId == null) {
      setState(() {
        _errorMessage = 'Please select a keymap';
      });
      return false;
    }

    final duplicate = widget.existingSlots.any(
      (slot) => slot.hardwareProfileId == _selectedProfileId && slot.keymapId == _selectedKeymapId,
    );

    if (duplicate) {
      setState(() {
        _errorMessage = 'This profile + keymap combination already exists for this device';
      });
      return false;
    }

    return true;
  }
}

/// Simple structure to pass existing slot information for validation.
class ProfileSlotSelection {
  ProfileSlotSelection({required this.hardwareProfileId, required this.keymapId});

  final HardwareProfileId hardwareProfileId;
  final KeymapId keymapId;
}

/// Shows the Add Slot dialog and returns true if a slot was added.
Future<bool?> showAddSlotDialog(
  BuildContext context, {
  required DeviceInstanceId device,
  required List<ProfileSlotSelection> existingSlots,
}) => showDialog<bool>(
  context: context,
  builder: (context) => AddSlotDialog(device: device, existingSlots: existingSlots),
);
