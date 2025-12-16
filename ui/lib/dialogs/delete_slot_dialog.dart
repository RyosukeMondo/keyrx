/// Confirmation dialog for deleting a slot.
library;

import 'package:flutter/material.dart';

/// Shows a confirmation dialog for deleting a slot.
///
/// Returns true if the user confirmed the deletion, false otherwise.
Future<bool?> showDeleteSlotConfirmation(
  BuildContext context, {
  required String slotDescription,
}) => showDialog<bool>(
  context: context,
  builder: (context) => AlertDialog(
    title: const Row(
      children: [
        Icon(Icons.warning_amber, color: Colors.orange),
        SizedBox(width: 8),
        Text('Delete Slot'),
      ],
    ),
    content: Column(
      mainAxisSize: MainAxisSize.min,
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        const Text('Are you sure you want to delete this slot?'),
        const SizedBox(height: 12),
        Container(
          padding: const EdgeInsets.all(8),
          decoration: BoxDecoration(
            color: Colors.grey.withOpacity(0.1),
            borderRadius: BorderRadius.circular(4),
            border: Border.all(color: Colors.grey.withOpacity(0.3)),
          ),
          child: Text(
            slotDescription,
            style: const TextStyle(
              fontFamily: 'monospace',
              fontSize: 12,
              fontWeight: FontWeight.w500,
            ),
          ),
        ),
        const SizedBox(height: 12),
        const Text(
          'This action cannot be undone.',
          style: TextStyle(
            fontSize: 12,
            color: Colors.grey,
            fontStyle: FontStyle.italic,
          ),
        ),
      ],
    ),
    actions: [
      TextButton(
        onPressed: () => Navigator.of(context).pop(false),
        child: const Text('Cancel'),
      ),
      FilledButton(
        onPressed: () => Navigator.of(context).pop(true),
        style: FilledButton.styleFrom(backgroundColor: Colors.red),
        child: const Text('Delete'),
      ),
    ],
  ),
);
