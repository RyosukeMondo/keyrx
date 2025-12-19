import 'package:core_logic/models/flow/flow_data.dart';
import 'package:flutter/material.dart';

class DeviceNodeWidget extends StatelessWidget {
  const DeviceNodeWidget({required this.data, super.key});
  final DeviceNodeData data;

  @override
  Widget build(BuildContext context) {
    final isConnected = data.isConnected;
    final bgColor = isConnected
        ? const Color(0xFF1E1E1E)
        : const Color(0xFF1A1A1A);

    return Container(
      width: 240,
      padding: const EdgeInsets.all(12),
      decoration: BoxDecoration(
        color: bgColor,
        borderRadius: BorderRadius.circular(8),
        border: Border.all(
          color: isConnected
              ? const Color(0xFF333333)
              : const Color(0xFF444444),
          style: isConnected
              ? BorderStyle.solid
              : BorderStyle
                    .none, // Dashed border implementation requires CustomPainter, simplifying for now
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
          // Header
          Row(
            children: [
              Container(
                width: 32,
                height: 32,
                decoration: BoxDecoration(
                  color: isConnected
                      ? Colors.white.withValues(alpha: 0.1)
                      : Colors.white.withValues(alpha: 0.05),
                  borderRadius: BorderRadius.circular(4),
                ),
                alignment: Alignment.center,
                child: Text(
                  '⌨️',
                  style: TextStyle(
                    fontSize: 18,
                    color: isConnected
                        ? null
                        : Colors.grey, // Grayscale effect mock
                  ),
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
                      '${data.vid}:${data.pid}',
                      style: const TextStyle(fontSize: 11, color: Colors.grey),
                    ),
                  ],
                ),
              ),
            ],
          ),
          const SizedBox(height: 12),

          // Active Profiles
          if (data.wirings.isNotEmpty) ...[
            const Text(
              'ACTIVE PROFILES (PRIORITY ORDER)',
              style: TextStyle(
                fontSize: 9,
                fontWeight: FontWeight.bold,
                color: Colors.grey,
              ),
            ),
            const SizedBox(height: 4),
            ...data.wirings.asMap().entries.map((entry) {
              final index = entry.key;
              final wiring = entry.value;
              final isActive = wiring.isActive;

              return Container(
                margin: const EdgeInsets.only(bottom: 4),
                padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 6),
                decoration: BoxDecoration(
                  color: isActive
                      ? const Color(0x221AB394)
                      : const Color(0xFF333333),
                  borderRadius: BorderRadius.circular(4),
                  border: Border.all(
                    color: isActive
                        ? const Color(0x441AB394)
                        : const Color(0xFF444444),
                  ),
                ),
                child: Row(
                  mainAxisAlignment: MainAxisAlignment.spaceBetween,
                  children: [
                    Row(
                      children: [
                        Text(
                          '#${index + 1}',
                          style: TextStyle(
                            fontSize: 9,
                            color: isActive
                                ? const Color(0xFF1AB394)
                                : Colors.grey,
                          ),
                        ),
                        const SizedBox(width: 6),
                        Text(
                          wiring.name,
                          style: TextStyle(
                            fontSize: 11,
                            color: isActive
                                ? const Color(0xFF1AB394)
                                : Colors.grey,
                          ),
                        ),
                      ],
                    ),
                    // Port Handle would be handled by fl_nodes wrapping this widget
                    // Just visual placeholder here if needed, but fl_nodes handles logical ports
                    Container(
                      width: 8,
                      height: 8,
                      decoration: BoxDecoration(
                        color: isActive ? const Color(0xFF1AB394) : Colors.grey,
                        shape: BoxShape.circle,
                      ),
                    ),
                  ],
                ),
              );
            }),
          ],

          const SizedBox(height: 8),

          // Footer
          Row(
            mainAxisAlignment: MainAxisAlignment.spaceBetween,
            children: [
              const Text(
                'VIRTUAL DEVICE',
                style: TextStyle(fontSize: 10, color: Colors.grey),
              ),
              Container(
                padding: const EdgeInsets.symmetric(horizontal: 6, vertical: 2),
                decoration: BoxDecoration(
                  borderRadius: BorderRadius.circular(10),
                  color: isConnected
                      ? const Color(0x332ECC71)
                      : const Color(0x337F8C8D),
                ),
                child: Text(
                  isConnected ? '● Connected' : '○ Offline',
                  style: TextStyle(
                    fontSize: 9,
                    color: isConnected
                        ? const Color(0xFF2ECC71)
                        : const Color(0xFFBDC3C7),
                  ),
                ),
              ),
            ],
          ),
        ],
      ),
    );
  }
}
