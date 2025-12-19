import 'package:core_logic/models/flow/flow_data.dart';
import 'package:flutter/material.dart';

class KeymapNodeWidget extends StatelessWidget {
  const KeymapNodeWidget({required this.data, super.key});
  final KeymapNodeData data;

  @override
  Widget build(BuildContext context) => Container(
    width: 180,
    padding: const EdgeInsets.all(12),
    decoration: BoxDecoration(
      color: const Color(0xFF1E1E1E),
      borderRadius: BorderRadius.circular(8),
      border: Border.all(
        color: data.isActive
            ? const Color(0xFF9B59B6)
            : const Color(0xFF444444),
        width: data.isActive ? 2 : 1,
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
        Row(
          children: [
            Container(
              width: 32,
              height: 32,
              decoration: BoxDecoration(
                color: const Color(0xFF9B59B6).withValues(alpha: 0.2),
                borderRadius: BorderRadius.circular(4),
              ),
              alignment: Alignment.center,
              child: const Text(
                '⚡',
                style: TextStyle(fontSize: 18, color: Color(0xFF9B59B6)),
              ),
            ),
            const SizedBox(width: 10),
            Expanded(
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  Text(
                    data.name,
                    style: const TextStyle(
                      fontWeight: FontWeight.w600,
                      fontSize: 14,
                      color: Colors.white,
                    ),
                  ),
                  Text(
                    '${data.layers} Layers',
                    style: const TextStyle(fontSize: 11, color: Colors.grey),
                  ),
                ],
              ),
            ),
          ],
        ),
        const SizedBox(height: 8),
        Container(
          padding: const EdgeInsets.symmetric(horizontal: 6, vertical: 2),
          decoration: BoxDecoration(
            color: data.isActive
                ? const Color(0x339B59B6)
                : const Color(0x337F8C8D),
            borderRadius: BorderRadius.circular(10),
          ),
          child: Text(
            data.isActive ? '● Active' : '○ Pending',
            style: TextStyle(
              fontSize: 10,
              color: data.isActive
                  ? const Color(0xFF9B59B6)
                  : const Color(0xFFBDC3C7),
            ),
          ),
        ),
      ],
    ),
  );
}
