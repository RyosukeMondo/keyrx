/// Enhanced device node widget with integrated slot management.
///
/// Provides add, delete, and reorder capabilities directly on the device node.
library;

import 'package:core_logic/models/hardware_profile.dart';
import 'package:core_logic/models/runtime_config.dart';
import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../../dialogs/add_slot_dialog.dart';
import 'slot_priority_adjuster.dart';

/// Enhanced device node with slot management capabilities.
class EnhancedDeviceNodeWithSlotManagement extends ConsumerStatefulWidget {
  const EnhancedDeviceNodeWithSlotManagement({
    required this.device,
    required this.slots,
    required this.deviceName,
    required this.isConnected,
    this.onSlotAdded,
    this.onSlotDeleted,
    this.onSlotsReordered,
    super.key,
  });

  final DeviceInstanceId device;
  final List<ProfileSlot> slots;
  final String deviceName;
  final bool isConnected;
  final VoidCallback? onSlotAdded;
  final VoidCallback? onSlotDeleted;
  final VoidCallback? onSlotsReordered;

  @override
  ConsumerState<EnhancedDeviceNodeWithSlotManagement> createState() =>
      _EnhancedDeviceNodeWithSlotManagementState();
}

class _EnhancedDeviceNodeWithSlotManagementState
    extends ConsumerState<EnhancedDeviceNodeWithSlotManagement> {
  @override
  Widget build(BuildContext context) {
    final bgColor = widget.isConnected ? const Color(0xFF1E1E1E) : const Color(0xFF1A1A1A);

    return Container(
      width: 280,
      padding: const EdgeInsets.all(12),
      decoration: BoxDecoration(
        color: bgColor,
        borderRadius: BorderRadius.circular(8),
        border: Border.all(
          color: widget.isConnected ? const Color(0xFF333333) : const Color(0xFF444444),
        ),
        boxShadow: [
          BoxShadow(
            color: Colors.black.withValues(alpha: 0.3),
            blurRadius: 6,
            offset: const Offset(0, 4),
          ),
        ],
      ),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        mainAxisSize: MainAxisSize.min,
        children: [
          _buildHeader(),
          const SizedBox(height: 12),
          if (widget.slots.isNotEmpty) ...[
            _buildSlotsHeader(),
            const SizedBox(height: 4),
            SlotPriorityAdjuster(
              device: widget.device,
              slots: widget.slots,
              onReordered: widget.onSlotsReordered,
            ),
            const SizedBox(height: 8),
          ],
          _buildAddSlotButton(),
          const SizedBox(height: 8),
          _buildFooter(),
        ],
      ),
    );
  }

  Widget _buildHeader() => Row(
    children: [
      Container(
        width: 32,
        height: 32,
        decoration: BoxDecoration(
          color: widget.isConnected
              ? Colors.white.withValues(alpha: 0.1)
              : Colors.white.withValues(alpha: 0.05),
          borderRadius: BorderRadius.circular(4),
        ),
        alignment: Alignment.center,
        child: Text(
          '⌨️',
          style: TextStyle(fontSize: 18, color: widget.isConnected ? null : Colors.grey),
        ),
      ),
      const SizedBox(width: 10),
      Expanded(
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Text(
              widget.deviceName,
              style: const TextStyle(
                fontWeight: FontWeight.w600,
                fontSize: 14,
                color: Colors.white,
              ),
            ),
            Text(
              '${widget.device.vendorId}:${widget.device.productId}',
              style: const TextStyle(fontSize: 11, color: Colors.grey),
            ),
          ],
        ),
      ),
    ],
  );

  Widget _buildSlotsHeader() => Row(
    mainAxisAlignment: MainAxisAlignment.spaceBetween,
    children: [
      const Text(
        'SLOTS (PRIORITY ORDER)',
        style: TextStyle(fontSize: 9, fontWeight: FontWeight.bold, color: Colors.grey),
      ),
      Text(
        '${widget.slots.length} slot${widget.slots.length != 1 ? 's' : ''}',
        style: const TextStyle(fontSize: 9, color: Colors.grey),
      ),
    ],
  );

  Widget _buildAddSlotButton() => SizedBox(
    width: double.infinity,
    child: OutlinedButton.icon(
      onPressed: _handleAddSlot,
      icon: const Icon(Icons.add, size: 16),
      label: const Text('Add Slot', style: TextStyle(fontSize: 12)),
      style: OutlinedButton.styleFrom(
        padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 8),
        side: const BorderSide(color: Color(0xFF444444)),
      ),
    ),
  );

  Widget _buildFooter() => Row(
    mainAxisAlignment: MainAxisAlignment.spaceBetween,
    children: [
      const Text('VIRTUAL DEVICE', style: TextStyle(fontSize: 10, color: Colors.grey)),
      Container(
        padding: const EdgeInsets.symmetric(horizontal: 6, vertical: 2),
        decoration: BoxDecoration(
          borderRadius: BorderRadius.circular(10),
          color: widget.isConnected ? const Color(0x332ECC71) : const Color(0x337F8C8D),
        ),
        child: Text(
          widget.isConnected ? '● Connected' : '○ Offline',
          style: TextStyle(
            fontSize: 9,
            color: widget.isConnected ? const Color(0xFF2ECC71) : const Color(0xFFBDC3C7),
          ),
        ),
      ),
    ],
  );

  Future<void> _handleAddSlot() async {
    final existingSlots = widget.slots
        .map(
          (s) => ProfileSlotSelection(hardwareProfileId: s.hardwareProfileId, keymapId: s.keymapId),
        )
        .toList();

    final added = await showAddSlotDialog(
      context,
      device: widget.device,
      existingSlots: existingSlots,
    );

    if (added == true && mounted) {
      widget.onSlotAdded?.call();
      ScaffoldMessenger.of(context).showSnackBar(
        const SnackBar(
          content: Text('Slot added successfully'),
          backgroundColor: Colors.green,
          duration: Duration(seconds: 2),
        ),
      );
    }
  }
}
