/// Slot actions for KeyFlow CRUD operations.
///
/// Provides a convenient API for adding, updating, deleting, and reordering
/// device slots, with automatic state synchronization.
library;

import 'package:core_logic/models/config_ids.dart';
import 'package:core_logic/models/hardware_profile.dart';
import 'package:core_logic/models/runtime_config.dart';
import 'package:flutter/foundation.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../repositories/keyflow_repository.dart';
import '../services/config_result.dart';
import '../services/daemon_service.dart';
import 'analytics_providers.dart';
import 'daemon_providers.dart';
import 'keyflow_providers.dart';

/// Actions for managing device slots.
class SlotActions {
  SlotActions(this._ref);

  final Ref _ref;

  KeyFlowRepository get _repo => _ref.read(keyFlowRepositoryProvider);
  DaemonService get _daemon => _ref.read(daemonServiceProvider);

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

    // Track analytics and reload daemon
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
      await _reloadDaemon();
    }

    return result;
  }

  /// Update an existing slot.
  Future<ConfigOperationResult<RuntimeConfig>> updateSlot({
    required DeviceInstanceId device,
    required ProfileSlot slot,
  }) async {
    final result = await _repo.updateSlot(device, slot);
    if (result.isSuccess) {
      await _reloadDaemon();
    }
    return result;
  }

  /// Delete a slot from a device.
  Future<ConfigOperationResult<RuntimeConfig>> deleteSlot({
    required DeviceInstanceId device,
    required String slotId,
  }) async {
    final result = await _repo.removeSlot(device, slotId);

    // Track analytics and reload daemon
    if (result.isSuccess) {
      _ref
          .read(analyticsTrackerProvider.notifier)
          .trackSlotDeleted(deviceId: device.toString(), slotId: slotId);
      await _reloadDaemon();
    }

    return result;
  }

  /// Toggle a slot's active state.
  Future<ConfigOperationResult<RuntimeConfig>> toggleSlotActive({
    required DeviceInstanceId device,
    required ProfileSlot slot,
  }) async {
    final updatedSlot = slot.copyWith(active: !slot.active);
    final result = await _repo.updateSlot(device, updatedSlot);
    if (result.isSuccess) {
      await _reloadDaemon();
    }
    return result;
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
    var anySuccess = false;

    for (var i = 0; i < slotsInOrder.length; i++) {
      final slot = slotsInOrder[i];
      if (slot.priority != i) {
        final oldPriority = slot.priority;
        final updatedSlot = slot.copyWith(priority: i);
        final result = await _repo.updateSlot(device, updatedSlot);
        results.add(result);

        // Track analytics for each reorder
        if (result.isSuccess) {
          anySuccess = true;
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

    if (anySuccess) {
      await _reloadDaemon();
    }

    return results;
  }

  /// Reloads the daemon engine configuration.
  /// Handles daemon disconnection gracefully.
  Future<void> _reloadDaemon() async {
    try {
      if (_daemon.isConnected) {
        await _daemon.reloadConfig();
      }
    } on DaemonException catch (e) {
      // Non-fatal, UI has already been updated optimistically.
      // Log the error for debugging.
      debugPrint('Failed to reload daemon after slot change: $e');
    }
  }

  String _generateSlotId() => DateTime.now().millisecondsSinceEpoch.toString();
}

/// Provider for slot actions.
final slotActionsProvider = Provider<SlotActions>(SlotActions.new);
