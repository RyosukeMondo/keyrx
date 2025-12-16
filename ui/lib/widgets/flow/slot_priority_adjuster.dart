/// Widget for adjusting slot priority through drag-and-drop reordering.
library;

import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../../models/hardware_profile.dart';
import '../../models/runtime_config.dart';
import '../../state/slot_actions.dart';

/// Widget that allows reordering slots by dragging priority badges.
///
/// Displays a reorderable list of slots with visual priority indicators.
/// When the user drags to reorder, priorities are automatically updated.
class SlotPriorityAdjuster extends ConsumerStatefulWidget {
  const SlotPriorityAdjuster({
    required this.device,
    required this.slots,
    this.onReordered,
    super.key,
  });

  final DeviceInstanceId device;
  final List<ProfileSlot> slots;
  final VoidCallback? onReordered;

  @override
  ConsumerState<SlotPriorityAdjuster> createState() =>
      _SlotPriorityAdjusterState();
}

class _SlotPriorityAdjusterState extends ConsumerState<SlotPriorityAdjuster> {
  late List<ProfileSlot> _orderedSlots;
  bool _isReordering = false;

  @override
  void initState() {
    super.initState();
    _orderedSlots = List.from(widget.slots)
      ..sort((a, b) => a.priority.compareTo(b.priority));
  }

  @override
  void didUpdateWidget(SlotPriorityAdjuster oldWidget) {
    super.didUpdateWidget(oldWidget);
    if (widget.slots != oldWidget.slots) {
      _orderedSlots = List.from(widget.slots)
        ..sort((a, b) => a.priority.compareTo(b.priority));
    }
  }

  @override
  Widget build(BuildContext context) {
    if (_orderedSlots.isEmpty) {
      return const SizedBox.shrink();
    }

    return ReorderableListView.builder(
      shrinkWrap: true,
      physics: const NeverScrollableScrollPhysics(),
      itemCount: _orderedSlots.length,
      onReorder: _handleReorder,
      buildDefaultDragHandles: false,
      itemBuilder: (context, index) {
        final slot = _orderedSlots[index];
        final displayPriority = index + 1;

        return _SlotPriorityItem(
          key: ValueKey(slot.id),
          slot: slot,
          displayPriority: displayPriority,
          index: index,
          isReordering: _isReordering,
        );
      },
    );
  }

  Future<void> _handleReorder(int oldIndex, int newIndex) async {
    setState(() {
      _isReordering = true;
    });

    if (newIndex > oldIndex) {
      newIndex -= 1;
    }

    final reorderedSlots = List<ProfileSlot>.from(_orderedSlots);
    final movedSlot = reorderedSlots.removeAt(oldIndex);
    reorderedSlots.insert(newIndex, movedSlot);

    setState(() {
      _orderedSlots = reorderedSlots;
    });

    try {
      final slotActions = ref.read(slotActionsProvider);
      final results = await slotActions.reorderSlots(
        device: widget.device,
        slotsInOrder: reorderedSlots,
      );

      final hasError = results.any((r) => r.hasError);
      if (hasError && mounted) {
        ScaffoldMessenger.of(context).showSnackBar(
          const SnackBar(
            content: Text('Failed to reorder slots'),
            backgroundColor: Colors.red,
          ),
        );

        setState(() {
          _orderedSlots = List.from(widget.slots)
            ..sort((a, b) => a.priority.compareTo(b.priority));
        });
      } else {
        widget.onReordered?.call();
      }
    } catch (e) {
      if (mounted) {
        ScaffoldMessenger.of(context).showSnackBar(
          SnackBar(
            content: Text('Error reordering slots: $e'),
            backgroundColor: Colors.red,
          ),
        );

        setState(() {
          _orderedSlots = List.from(widget.slots)
            ..sort((a, b) => a.priority.compareTo(b.priority));
        });
      }
    } finally {
      if (mounted) {
        setState(() {
          _isReordering = false;
        });
      }
    }
  }
}

class _SlotPriorityItem extends StatelessWidget {
  const _SlotPriorityItem({
    required this.slot,
    required this.displayPriority,
    required this.index,
    required this.isReordering,
    super.key,
  });

  final ProfileSlot slot;
  final int displayPriority;
  final int index;
  final bool isReordering;

  @override
  Widget build(BuildContext context) => Container(
    margin: const EdgeInsets.only(bottom: 4),
    padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 6),
    decoration: BoxDecoration(
      color: slot.active ? const Color(0x221AB394) : const Color(0xFF333333),
      borderRadius: BorderRadius.circular(4),
      border: Border.all(
        color: slot.active ? const Color(0x441AB394) : const Color(0xFF444444),
      ),
    ),
    child: Row(
      children: [
        ReorderableDragStartListener(
          index: index,
          enabled: !isReordering,
          child: MouseRegion(
            cursor: SystemMouseCursors.grab,
            child: Container(
              padding: const EdgeInsets.all(4),
              child: Icon(
                Icons.drag_indicator,
                size: 16,
                color: isReordering ? Colors.grey : Colors.white70,
              ),
            ),
          ),
        ),
        const SizedBox(width: 4),
        Container(
          width: 24,
          height: 24,
          decoration: BoxDecoration(
            color: slot.active ? const Color(0xFF1AB394) : Colors.grey,
            shape: BoxShape.circle,
          ),
          alignment: Alignment.center,
          child: Text(
            '#$displayPriority',
            style: const TextStyle(
              fontSize: 9,
              fontWeight: FontWeight.bold,
              color: Colors.white,
            ),
          ),
        ),
        const SizedBox(width: 8),
        Expanded(
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            mainAxisSize: MainAxisSize.min,
            children: [
              Text(
                slot.hardwareProfileId,
                style: TextStyle(
                  fontSize: 11,
                  fontWeight: FontWeight.w500,
                  color: slot.active ? const Color(0xFF1AB394) : Colors.white70,
                ),
                overflow: TextOverflow.ellipsis,
              ),
              Text(
                slot.keymapId,
                style: TextStyle(
                  fontSize: 10,
                  color: slot.active
                      ? const Color(0xFF1AB394).withOpacity(0.7)
                      : Colors.grey,
                ),
                overflow: TextOverflow.ellipsis,
              ),
            ],
          ),
        ),
        if (slot.active)
          Container(
            padding: const EdgeInsets.symmetric(horizontal: 6, vertical: 2),
            decoration: BoxDecoration(
              color: const Color(0xFF1AB394).withOpacity(0.2),
              borderRadius: BorderRadius.circular(10),
            ),
            child: const Text(
              'ACTIVE',
              style: TextStyle(
                fontSize: 8,
                fontWeight: FontWeight.bold,
                color: Color(0xFF1AB394),
              ),
            ),
          ),
      ],
    ),
  );
}
