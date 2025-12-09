import 'package:flutter/material.dart';
import '../../models/virtual_layout.dart';
import '../../models/virtual_layout_type.dart';
import 'grid_generator.dart';

class LayoutSelector extends StatelessWidget {
  const LayoutSelector({
    super.key,
    required this.layouts,
    required this.onSelectLayout,
    required this.onDeleteLayout,
    required this.onCreateFreeform,
    required this.onCreateGrid,
  });

  final List<VirtualLayout> layouts;
  final ValueChanged<VirtualLayout> onSelectLayout;
  final ValueChanged<String> onDeleteLayout;
  final VoidCallback onCreateFreeform;
  final ValueChanged<List<VirtualKeyDef>> onCreateGrid;

  @override
  Widget build(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.all(24.0),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          Text(
            'Create New Layout',
            style: Theme.of(context).textTheme.headlineSmall,
          ),
          const SizedBox(height: 16),
          SizedBox(
            height: 240,
            child: Row(
              crossAxisAlignment: CrossAxisAlignment.stretch,
              children: [
                Expanded(
                  child: _BentoCard(
                    title: 'Freeform',
                    subtitle: 'Start blank',
                    icon: Icons.edit_note,
                    color: Theme.of(context).colorScheme.primaryContainer,
                    onColor: Theme.of(context).colorScheme.onPrimaryContainer,
                    onTap: onCreateFreeform,
                  ),
                ),
                const SizedBox(width: 16),
                Expanded(
                  flex: 2,
                  child: _GridGeneratorCard(onCreateGrid: onCreateGrid),
                ),
              ],
            ),
          ),
          const SizedBox(height: 32),
          Text(
            'Saved Layouts',
            style: Theme.of(context).textTheme.headlineSmall,
          ),
          const SizedBox(height: 16),
          if (layouts.isEmpty) const Text('No saved layouts found.'),
          Expanded(
            child: GridView.builder(
              gridDelegate: const SliverGridDelegateWithMaxCrossAxisExtent(
                maxCrossAxisExtent: 300,
                childAspectRatio: 1.5,
                crossAxisSpacing: 16,
                mainAxisSpacing: 16,
              ),
              itemCount: layouts.length,
              itemBuilder: (context, index) {
                final layout = layouts[index];
                return _BentoCard(
                  title: layout.name,
                  subtitle:
                      '${layout.layoutType.label} • ${layout.keys.length} keys',
                  icon: Icons.keyboard_alt_outlined,
                  onTap: () => onSelectLayout(layout),
                  onDelete: () => onDeleteLayout(layout.id),
                );
              },
            ),
          ),
        ],
      ),
    );
  }
}

class _BentoCard extends StatelessWidget {
  const _BentoCard({
    required this.title,
    required this.subtitle,
    required this.icon,
    required this.onTap,
    this.onDelete,
    this.color,
    this.onColor,
  });

  final String title;
  final String subtitle;
  final IconData icon;
  final VoidCallback onTap;
  final VoidCallback? onDelete;
  final Color? color;
  final Color? onColor;

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);
    return Card(
      elevation: 0,
      color: color ?? theme.colorScheme.surfaceContainerHighest,
      clipBehavior: Clip.antiAlias,
      child: InkWell(
        onTap: onTap,
        child: Stack(
          children: [
            Padding(
              padding: const EdgeInsets.all(20.0),
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                mainAxisAlignment: MainAxisAlignment.spaceBetween,
                children: [
                  Icon(
                    icon,
                    size: 32,
                    color: onColor ?? theme.colorScheme.onSurface,
                  ),
                  Column(
                    crossAxisAlignment: CrossAxisAlignment.start,
                    children: [
                      Text(
                        title,
                        style: theme.textTheme.titleMedium?.copyWith(
                          fontWeight: FontWeight.bold,
                          color: onColor ?? theme.colorScheme.onSurface,
                        ),
                      ),
                      const SizedBox(height: 4),
                      Text(
                        subtitle,
                        style: theme.textTheme.bodySmall?.copyWith(
                          color: (onColor ?? theme.colorScheme.onSurfaceVariant)
                              .withValues(alpha: 0.8),
                        ),
                      ),
                    ],
                  ),
                ],
              ),
            ),
            if (onDelete != null)
              Positioned(
                top: 4,
                right: 4,
                child: IconButton(
                  icon: const Icon(Icons.close, size: 18),
                  padding: EdgeInsets.zero,
                  constraints: const BoxConstraints(),
                  style: IconButton.styleFrom(
                    tapTargetSize: MaterialTapTargetSize.shrinkWrap,
                    backgroundColor: Colors.transparent,
                    foregroundColor: theme.colorScheme.onSurfaceVariant
                        .withValues(alpha: 0.7),
                  ),
                  onPressed: onDelete,
                ),
              ),
          ],
        ),
      ),
    );
  }
}

class _GridGeneratorCard extends StatefulWidget {
  const _GridGeneratorCard({required this.onCreateGrid});

  final ValueChanged<List<VirtualKeyDef>> onCreateGrid;

  @override
  State<_GridGeneratorCard> createState() => _GridGeneratorCardState();
}

class _GridGeneratorCardState extends State<_GridGeneratorCard> {
  // We delegate the logic to the inner GridGenerator widget,
  // but for the Bento card feel, we wrap it with a header.

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);
    return Card(
      elevation: 0,
      color: theme.colorScheme.surfaceContainerHighest,
      clipBehavior: Clip.antiAlias,
      child: Padding(
        padding: const EdgeInsets.all(20.0),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Row(
              children: [
                Icon(
                  Icons.grid_on,
                  size: 32,
                  color: theme.colorScheme.onSurface,
                ),
                const SizedBox(width: 12),
                Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    Text(
                      'Grid Generator',
                      style: theme.textTheme.titleMedium?.copyWith(
                        fontWeight: FontWeight.bold,
                      ),
                    ),
                    Text(
                      'Hover to pick size',
                      style: theme.textTheme.bodySmall?.copyWith(
                        color: theme.colorScheme.onSurfaceVariant,
                      ),
                    ),
                  ],
                ),
              ],
            ),
            const Spacer(),
            Expanded(
              flex: 4,
              child: GridGenerator(onGenerate: widget.onCreateGrid),
            ),
          ],
        ),
      ),
    );
  }
}

