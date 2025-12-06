/// Embedded metrics dashboard page.
///
/// Provides a real-time view of KeyRx performance metrics with selectable
/// history windows for quick comparisons without leaving the app.
library;

import 'package:flutter/material.dart';
import 'package:provider/provider.dart';

import '../services/metrics_service.dart';
import '../services/service_registry.dart';
import '../widgets/metrics/metrics_dashboard.dart';

/// Page that hosts the embedded metrics dashboard UI.
class MetricsDashboardPage extends StatefulWidget {
  const MetricsDashboardPage({super.key});

  @override
  State<MetricsDashboardPage> createState() => _MetricsDashboardPageState();
}

class _MetricsDashboardPageState extends State<MetricsDashboardPage> {
  late final MetricsService _metricsService;
  Duration _selectedRange = const Duration(minutes: 1);

  static const List<Duration> _rangeOptions = <Duration>[
    Duration(minutes: 1),
    Duration(minutes: 5),
    Duration(minutes: 15),
  ];

  int get _historyPoints => _selectedRange.inSeconds
      .clamp(30, 1800)
      .toInt(); // 1 sample/sec retention

  @override
  void initState() {
    super.initState();
    final registry = context.read<ServiceRegistry>();
    _metricsService = MetricsServiceImpl(bridge: registry.bridge);
  }

  @override
  void dispose() {
    _metricsService.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);

    return Scaffold(
      appBar: AppBar(
        title: const Text('Metrics Dashboard'),
        actions: [
          IconButton(
            icon: const Icon(Icons.info_outline),
            onPressed: _showHelp,
            tooltip: 'What am I seeing?',
          ),
        ],
      ),
      body: Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          Padding(
            padding: const EdgeInsets.fromLTRB(16, 16, 16, 8),
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.stretch,
              children: [
                Text(
                  'Live engine metrics',
                  style: theme.textTheme.titleLarge?.copyWith(
                    fontWeight: FontWeight.bold,
                  ),
                ),
                const SizedBox(height: 6),
                Text(
                  'Track latency percentiles, throughput, memory, and alert thresholds without external Grafana.',
                  style: theme.textTheme.bodyMedium,
                ),
                const SizedBox(height: 12),
                _buildRangeSelector(theme),
              ],
            ),
          ),
          const Divider(height: 1),
          Expanded(
            child: Padding(
              padding: const EdgeInsets.symmetric(horizontal: 8),
              child: MetricsDashboard(
                metricsService: _metricsService,
                maxHistoryPoints: _historyPoints,
              ),
            ),
          ),
        ],
      ),
    );
  }

  Widget _buildRangeSelector(ThemeData theme) {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Text(
          'Time range',
          style: theme.textTheme.titleSmall?.copyWith(
            letterSpacing: 0.2,
            fontWeight: FontWeight.w600,
          ),
        ),
        const SizedBox(height: 8),
        SegmentedButton<Duration>(
          segments: _rangeOptions
              .map(
                (range) => ButtonSegment<Duration>(
                  value: range,
                  label: Text(_formatRange(range)),
                  icon: const Icon(Icons.timeline),
                ),
              )
              .toList(),
          selected: {_selectedRange},
          onSelectionChanged: (selection) {
            setState(() {
              _selectedRange = selection.first;
            });
          },
        ),
        const SizedBox(height: 6),
        Text(
          'Adjusts how many samples are retained in the live charts for quick comparisons.',
          style: theme.textTheme.bodySmall?.copyWith(
            color: theme.colorScheme.onSurfaceVariant,
          ),
        ),
      ],
    );
  }

  void _showHelp() {
    final theme = Theme.of(context);
    showDialog<void>(
      context: context,
      builder: (context) {
        return AlertDialog(
          title: const Text('Embedded dashboard'),
          content: Column(
            mainAxisSize: MainAxisSize.min,
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              Text(
                'This view streams metrics directly from the KeyRx engine via FFI. Use the time range to keep 1, 5, or 15 minutes of samples in memory.',
                style: theme.textTheme.bodyMedium,
              ),
              const SizedBox(height: 12),
              Text(
                'Charts update once per second and surface threshold violations inline with the latency/memory panels.',
                style: theme.textTheme.bodyMedium,
              ),
            ],
          ),
          actions: [
            TextButton(
              onPressed: () => Navigator.of(context).pop(),
              child: const Text('Close'),
            ),
          ],
        );
      },
    );
  }

  String _formatRange(Duration range) {
    if (range.inMinutes < 1) {
      return '${range.inSeconds}s';
    }
    if (range.inMinutes < 60) {
      return '${range.inMinutes}m';
    }
    final hours = range.inHours;
    final minutes = range.inMinutes % 60;
    return minutes == 0 ? '${hours}h' : '${hours}h ${minutes}m';
  }
}
