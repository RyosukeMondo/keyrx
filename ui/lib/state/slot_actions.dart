/// Slot actions for KeyFlow CRUD operations.
///
/// Provides a convenient API for adding, updating, deleting, and reordering
/// device slots, with automatic state synchronization.
library;

import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../models/config_ids.dart';
import '../models/hardware_profile.dart';
import '../models/runtime_config.dart';
import '../repositories/keyflow_repository.dart';
import '../services/config_result.dart';
import 'keyflow_providers.dart';
import 'analytics_providers.dart';

/// Actions for managing device slots.
class SlotActions {
  SlotActions(this._ref);

  final Ref _ref;

  KeyFlowRepository get _repo => _ref.read(keyFlowRepositoryProvider);

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
  }) async {
    final slot = ProfileSlot(
      id: _generateSlotId(),
      hardwareProfileId: hardwareProfileId,
      keymapId: keymapId,
      active: active,
      priority: priorityOverride ?? 0,
    );

    final result = await _repo.addSlot(
      device,
      slot,
      priorityOverride: priorityOverride,
    );

    // Track analytics
    if (result.isSuccess) {
      _ref
          .read(analyticsTrackerProvider.notifier)
          .trackSlotAdded(
            deviceId: device.toString(),
            profileId: hardwareProfileId,
            keymapId: keymapId,
            priority: slot.priority,
            autoAssigned: priorityOverride == null,
          );
    }

    return result;
  }

  /// Update an existing slot.
  Future<ConfigOperationResult<RuntimeConfig>> updateSlot({
    required DeviceInstanceId device,
    required ProfileSlot slot,
  }) {
    return _repo.updateSlot(device, slot);
  }

  /// Delete a slot from a device.
  Future<ConfigOperationResult<RuntimeConfig>> deleteSlot({
    required DeviceInstanceId device,
    required String slotId,
  }) async {
    final result = await _repo.removeSlot(device, slotId);

    // Track analytics
    if (result.isSuccess) {
      _ref
          .read(analyticsTrackerProvider.notifier)
          .trackSlotDeleted(deviceId: device.toString(), slotId: slotId);
    }

    return result;
  }

  /// Toggle a slot's active state.
  Future<ConfigOperationResult<RuntimeConfig>> toggleSlotActive({
    required DeviceInstanceId device,
    required ProfileSlot slot,
  }) {
    final updatedSlot = slot.copyWith(active: !slot.active);
    return _repo.updateSlot(device, updatedSlot);
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
        final oldPriority = slot.priority;
        final updatedSlot = slot.copyWith(priority: i);
        final result = await _repo.updateSlot(device, updatedSlot);
        results.add(result);

        // Track analytics for each reorder
        if (result.isSuccess) {
          _ref
              .read(analyticsTrackerProvider.notifier)
              .trackSlotReordered(
                deviceId: device.toString(),
                slotId: slot.id,
                oldPriority: oldPriority,
                newPriority: i,
              );
        }

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
