// Trade-off visualizer widget components.
//
// Contains reusable widgets and dialogs for the trade-off visualizer page.

import 'package:flutter/material.dart';

import 'trade_off_chart.dart';

/// Widget for displaying a metric tile with colored background.
class MetricTile extends StatelessWidget {
  const MetricTile({
    super.key,
    required this.label,
    required this.value,
    required this.color,
  });

  final String label;
  final String value;
  final Color color;

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);
    return Container(
      padding: const EdgeInsets.all(12),
      decoration: BoxDecoration(
        color: color.withValues(alpha: 0.1),
        borderRadius: BorderRadius.circular(8),
        border: Border.all(color: color.withValues(alpha: 0.3)),
      ),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Text(
            label,
            style: theme.textTheme.bodySmall?.copyWith(
              color: theme.colorScheme.onSurface.withValues(alpha: 0.7),
            ),
          ),
          const SizedBox(height: 4),
          Text(
            value,
            style: theme.textTheme.titleMedium?.copyWith(
              color: color,
              fontWeight: FontWeight.bold,
            ),
          ),
        ],
      ),
    );
  }
}

/// Widget for displaying a statistic tile with icon.
class StatTile extends StatelessWidget {
  const StatTile({
    super.key,
    required this.label,
    required this.value,
    required this.icon,
  });

  final String label;
  final String value;
  final IconData icon;

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);
    return Container(
      padding: const EdgeInsets.all(12),
      decoration: BoxDecoration(
        color: theme.colorScheme.surfaceContainerHighest,
        borderRadius: BorderRadius.circular(8),
      ),
      child: Row(
        children: [
          Icon(icon, size: 24, color: theme.colorScheme.primary),
          const SizedBox(width: 12),
          Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              Text(label, style: theme.textTheme.bodySmall),
              Text(
                value,
                style: theme.textTheme.titleSmall?.copyWith(
                  fontWeight: FontWeight.bold,
                ),
              ),
            ],
          ),
        ],
      ),
    );
  }
}

/// Widget for displaying a preset region row.
class PresetRow extends StatelessWidget {
  const PresetRow({
    super.key,
    required this.preset,
    required this.isSelected,
    required this.onTap,
  });

  final PresetRegion preset;
  final bool isSelected;
  final VoidCallback onTap;

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);

    return Padding(
      padding: const EdgeInsets.only(bottom: 8),
      child: InkWell(
        onTap: onTap,
        borderRadius: BorderRadius.circular(8),
        child: Container(
          padding: const EdgeInsets.all(12),
          decoration: BoxDecoration(
            color: isSelected
                ? preset.color.withValues(alpha: 0.15)
                : theme.colorScheme.surface,
            borderRadius: BorderRadius.circular(8),
            border: Border.all(
              color: isSelected ? preset.color : theme.dividerColor,
              width: isSelected ? 2 : 1,
            ),
          ),
          child: Row(
            children: [
              Container(
                width: 12,
                height: 12,
                decoration: BoxDecoration(
                  color: preset.color,
                  shape: BoxShape.circle,
                ),
              ),
              const SizedBox(width: 12),
              Expanded(
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    Text(
                      preset.name,
                      style: theme.textTheme.titleSmall?.copyWith(
                        fontWeight: isSelected ? FontWeight.bold : FontWeight.normal,
                      ),
                    ),
                    Text(
                      preset.description,
                      style: theme.textTheme.bodySmall?.copyWith(
                        color: theme.colorScheme.onSurface.withValues(alpha: 0.7),
                      ),
                    ),
                  ],
                ),
              ),
              Text(
                '${preset.minMs}-${preset.maxMs}ms',
                style: theme.textTheme.bodyMedium?.copyWith(
                  color: preset.color,
                  fontWeight: FontWeight.bold,
                ),
              ),
              if (isSelected) ...[
                const SizedBox(width: 8),
                Icon(Icons.check_circle, color: preset.color, size: 20),
              ],
            ],
          ),
        ),
      ),
    );
  }
}

/// Widget for displaying a recommendation banner.
class RecommendationBanner extends StatelessWidget {
  const RecommendationBanner({
    super.key,
    required this.recommendedTimeout,
    required this.userTypingMean,
    required this.onApply,
  });

  final double recommendedTimeout;
  final double userTypingMean;
  final VoidCallback onApply;

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);

    return Container(
      padding: const EdgeInsets.all(12),
      decoration: BoxDecoration(
        color: Colors.purple.withValues(alpha: 0.1),
        borderRadius: BorderRadius.circular(8),
        border: Border.all(color: Colors.purple.withValues(alpha: 0.3)),
      ),
      child: Row(
        children: [
          const Icon(Icons.recommend, color: Colors.purple),
          const SizedBox(width: 12),
          Expanded(
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Text(
                  'Recommended: ${recommendedTimeout.toInt()} ms',
                  style: theme.textTheme.titleSmall?.copyWith(
                    color: Colors.purple,
                    fontWeight: FontWeight.bold,
                  ),
                ),
                Text(
                  'Based on your typing profile (mean: ${userTypingMean.toInt()}ms)',
                  style: theme.textTheme.bodySmall,
                ),
              ],
            ),
          ),
          TextButton(
            onPressed: onApply,
            child: const Text('Apply'),
          ),
        ],
      ),
    );
  }
}

/// Dialog for editing typing model parameters.
class ModelEditDialog extends StatefulWidget {
  const ModelEditDialog({
    super.key,
    required this.initialMean,
    required this.initialStdDev,
    required this.onApply,
  });

  final double initialMean;
  final double initialStdDev;
  final void Function(double mean, double stdDev) onApply;

  @override
  State<ModelEditDialog> createState() => _ModelEditDialogState();
}

class _ModelEditDialogState extends State<ModelEditDialog> {
  late double _tempMean;
  late double _tempStdDev;

  @override
  void initState() {
    super.initState();
    _tempMean = widget.initialMean;
    _tempStdDev = widget.initialStdDev;
  }

  @override
  Widget build(BuildContext context) {
    return AlertDialog(
      title: const Text('Edit Typing Model'),
      content: Column(
        mainAxisSize: MainAxisSize.min,
        children: [
          Text('Mean Key Duration: ${_tempMean.toInt()} ms'),
          Slider(
            value: _tempMean,
            min: 100,
            max: 400,
            divisions: 30,
            onChanged: (v) => setState(() => _tempMean = v),
          ),
          const SizedBox(height: 16),
          Text('Standard Deviation: ${_tempStdDev.toInt()} ms'),
          Slider(
            value: _tempStdDev,
            min: 20,
            max: 100,
            divisions: 16,
            onChanged: (v) => setState(() => _tempStdDev = v),
          ),
        ],
      ),
      actions: [
        TextButton(
          onPressed: () => Navigator.pop(context),
          child: const Text('Cancel'),
        ),
        FilledButton(
          onPressed: () {
            widget.onApply(_tempMean, _tempStdDev);
            Navigator.pop(context);
          },
          child: const Text('Apply'),
        ),
      ],
    );
  }
}

/// Dialog for displaying help information.
class TradeOffHelpDialog extends StatelessWidget {
  const TradeOffHelpDialog({super.key});

  @override
  Widget build(BuildContext context) {
    return AlertDialog(
      title: const Text('Understanding Trade-offs'),
      content: const SingleChildScrollView(
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          mainAxisSize: MainAxisSize.min,
          children: [
            Text(
              'Tap-Hold Timeout',
              style: TextStyle(fontWeight: FontWeight.bold),
            ),
            SizedBox(height: 8),
            Text(
              'This is the time threshold that determines whether a key '
              'press is interpreted as a "tap" or a "hold".\n\n'
              '- Shorter timeout: Faster hold activation, but more '
              'accidental holds during fast typing\n'
              '- Longer timeout: Fewer accidental holds, but slower '
              'hold activation',
            ),
            SizedBox(height: 16),
            Text(
              'Miss Rate',
              style: TextStyle(fontWeight: FontWeight.bold),
            ),
            SizedBox(height: 8),
            Text(
              'The estimated percentage of key presses that will be '
              'incorrectly classified. This is calculated using a '
              'statistical model of your typing speed.',
            ),
            SizedBox(height: 16),
            Text(
              'Presets',
              style: TextStyle(fontWeight: FontWeight.bold),
            ),
            SizedBox(height: 8),
            Text(
              '- Gaming: 100-150ms - Optimized for fast reactions\n'
              '- Typing: 175-250ms - Balanced for daily use\n'
              '- Relaxed: 300-500ms - Slower, more forgiving',
            ),
          ],
        ),
      ),
      actions: [
        FilledButton(
          onPressed: () => Navigator.pop(context),
          child: const Text('Got it'),
        ),
      ],
    );
  }
}
