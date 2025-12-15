/// Slot actions for KeyFlow CRUD operations.
///
/// Provides a convenient API for adding, updating, deleting, and reordering
/// device slots, with automatic state synchronization.
library;

import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../models/config_ids.dart';
import '../models/hardware_profile.dart';
import '../models/runtime_config.dart';
import '../services/config_result.dart';
import 'keyflow_providers.dart';

/// Actions for managing device slots.
class SlotActions {
  SlotActions(this._ref);

  final Ref _ref;

  KeyFlowActions get _actions => _ref.read(keyFlowActionsProvider);

  /// Add a new slot to a device.
  ///
  /// If [priorityOverride] is null, the repository will auto-calculate the
  /// next priority based on existing slots.
  Future<ConfigOperationResult<RuntimeConfig>> addSlot({
    required DeviceInstanceId device,
    required HardwareProfileId hardwareProfileId,
    required KeymapId keymapId,
    bool active = false,
    int? priorityOverride,
  }) {
    final slot = ProfileSlot(
      id: _generateSlotId(),
      hardwareProfileId: hardwareProfileId,
      keymapId: keymapId,
      active: active,
      priority: priorityOverride ?? 0,
    );

    return _actions.addSlot(device, slot, priorityOverride: priorityOverride);
  }

  /// Update an existing slot.
  Future<ConfigOperationResult<RuntimeConfig>> updateSlot({
    required DeviceInstanceId device,
    required ProfileSlot slot,
  }) {
    return _actions.updateSlot(device, slot);
  }

  /// Delete a slot from a device.
  Future<ConfigOperationResult<RuntimeConfig>> deleteSlot({
    required DeviceInstanceId device,
    required String slotId,
  }) {
    return _actions.removeSlot(device, slotId);
  }

  /// Toggle a slot's active state.
  Future<ConfigOperationResult<RuntimeConfig>> toggleSlotActive({
    required DeviceInstanceId device,
    required ProfileSlot slot,
  }) {
    final updatedSlot = slot.copyWith(active: !slot.active);
    return _actions.updateSlot(device, updatedSlot);
  }

  /// Reorder slots by updating their priorities.
  ///
  /// Takes a list of slots in the desired order and reassigns priorities
  /// sequentially (0, 1, 2, ...).
  Future<List<ConfigOperationResult<RuntimeConfig>>> reorderSlots({
    required DeviceInstanceId device,
    required List<ProfileSlot> slotsInOrder,
  }) async {
    final results = <ConfigOperationResult<RuntimeConfig>>[];

    for (var i = 0; i < slotsInOrder.length; i++) {
      final slot = slotsInOrder[i];
      if (slot.priority != i) {
        final updatedSlot = slot.copyWith(priority: i);
        final result = await _actions.updateSlot(device, updatedSlot);
        results.add(result);

        if (result.hasError) {
          break;
        }
      }
    }

    return results;
  }

  String _generateSlotId() {
    return DateTime.now().millisecondsSinceEpoch.toString();
  }
}

/// Provider for slot actions.
final slotActionsProvider = Provider<SlotActions>((ref) {
  return SlotActions(ref);
});
