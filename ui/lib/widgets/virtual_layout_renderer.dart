import 'package:flutter/material.dart';
import '../models/virtual_layout.dart';
import 'virtual_layout_common.dart';

class VirtualLayoutRenderer extends StatelessWidget {
  const VirtualLayoutRenderer({
    super.key,
    required this.layout,
    required this.selectedKeyId,
    required this.mappedKeyIds,
    required this.onKeyTap,
  });

  final VirtualLayout layout;
  final String? selectedKeyId;
  final Set<String> mappedKeyIds;
  final ValueChanged<String> onKeyTap;

  @override
  Widget build(BuildContext context) {
    // Determine canvas size based on keys to minimize empty space, or use fixed large size?
    // Using a fixed large size is easier but might result in scrolling.
    // Let's compute bounds.
    double maxX = 0;
    double maxY = 0;
    for (final key in layout.keys) {
      final x = (key.position?.x ?? 0) + (key.size?.width ?? 50);
      final y = (key.position?.y ?? 0) + (key.size?.height ?? 50);
      if (x > maxX) maxX = x;
      if (y > maxY) maxY = y;
    }

    final width = maxX + 100; // Padding
    final height = maxY + 100;

    return Container(
      color: Theme.of(context).colorScheme.surfaceContainer,
      child: InteractiveViewer(
        boundaryMargin: const EdgeInsets.all(100),
        minScale: 0.1,
        maxScale: 2.0,
        constrained: false, // Allow unbounded canvas
        child: SizedBox(
          width: width < 800 ? 800 : width, // Min size
          height: height < 400 ? 400 : height,
          child: Stack(
            children: [
              // Optional Grid for context
              Positioned.fill(
                child: CustomPaint(
                  painter: VirtualLayoutGridPainter(
                    step: 60,
                    color: Theme.of(context).dividerColor.withValues(alpha: 0.05),
                  ),
                ),
              ),
              ...layout.keys.map((key) => _buildKey(context, key)),
            ],
          ),
        ),
      ),
    );
  }

  Widget _buildKey(BuildContext context, VirtualKeyDef key) {
    final isSelected = selectedKeyId == key.id;
    final isMapped = mappedKeyIds.contains(key.id);

    final color = isSelected
        ? Theme.of(context).colorScheme.primaryContainer
        : isMapped
        ? Theme.of(context).colorScheme.secondaryContainer.withValues(alpha: 0.5)
        : Theme.of(context).colorScheme.surface;

    final borderColor = isSelected
        ? Theme.of(context).colorScheme.primary
        : isMapped
        ? Theme.of(context).colorScheme.secondary
        : Theme.of(context).dividerColor;

    return Positioned(
      left: key.position?.x ?? 0,
      top: key.position?.y ?? 0,
      width: key.size?.width ?? 50,
      height: key.size?.height ?? 50,
      child: GestureDetector(
        onTap: () => onKeyTap(key.id),
        child: Container(
          margin: const EdgeInsets.all(2), // Gutter
          decoration: BoxDecoration(
            color: color,
            borderRadius: BorderRadius.circular(4),
            border: Border.all(color: borderColor, width: isSelected ? 2 : 1),
          ),
          alignment: Alignment.center,
          child: Column(
            mainAxisAlignment: MainAxisAlignment.center,
            children: [
              Text(
                key.label.isEmpty && key.id.isNotEmpty ? key.id : key.label,
                style: TextStyle(
                  fontSize: 12,
                  fontWeight: isSelected ? FontWeight.bold : FontWeight.normal,
                  color: isSelected
                      ? Theme.of(context).colorScheme.onPrimaryContainer
                      : null,
                ),
                overflow: TextOverflow.ellipsis,
                textAlign: TextAlign.center,
              ),
              // Show ID if label is different, optional
              /*
              if (key.label != key.id)
                Text(
                  key.id,
                   style: TextStyle(
                    fontSize: 8,
                    color: Theme.of(context).textTheme.bodySmall?.color?.withValues(alpha: 0.5),
                   ),
                ),
             */
            ],
          ),
        ),
      ),
    );
  }
}

