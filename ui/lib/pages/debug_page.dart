/// Debug page for viewing performance metrics and diagnostics.
///
/// Provides access to:
/// - Real-time performance metrics with charts
/// - Metrics export functionality
/// - Threshold configuration
library;

import 'dart:convert';

import 'package:flutter/material.dart';
import 'package:flutter/services.dart';

import '../services/metrics_service.dart';
import '../widgets/metrics/metrics_dashboard.dart';
import '../widgets/settings/threshold_settings_dialog.dart';

/// Debug page for performance metrics and diagnostics.
class DebugPage extends StatefulWidget {
  const DebugPage({required this.metricsService, super.key});

  final UIMetricsService metricsService;

  @override
  State<DebugPage> createState() => _DebugPageState();
}

class _DebugPageState extends State<DebugPage>
    with SingleTickerProviderStateMixin {
  late TabController _tabController;
  MetricsThresholds? _currentThresholds;

  @override
  void initState() {
    super.initState();
    _tabController = TabController(length: 2, vsync: this);
    _loadThresholds();
  }

  Future<void> _loadThresholds() async {
    final thresholds = await widget.metricsService.getThresholds();
    if (mounted) {
      setState(() {
        _currentThresholds = thresholds ?? MetricsThresholds.defaults();
      });
    }
  }

  @override
  void dispose() {
    _tabController.dispose();
    super.dispose();
  }

  Future<void> _exportMetrics() async {
    final snapshot = widget.metricsService.cachedSnapshot;
    if (snapshot == null) {
      if (mounted) {
        ScaffoldMessenger.of(context).showSnackBar(
          const SnackBar(
            content: Text('No metrics data available to export'),
            backgroundColor: Colors.orange,
          ),
        );
      }
      return;
    }

    final jsonStr = const JsonEncoder.withIndent(
      '  ',
    ).convert(snapshot.toJson());

    await Clipboard.setData(ClipboardData(text: jsonStr));

    if (mounted) {
      ScaffoldMessenger.of(context).showSnackBar(
        const SnackBar(
          content: Text('Metrics exported to clipboard as JSON'),
          backgroundColor: Colors.green,
        ),
      );
    }
  }

  Future<void> _showThresholdSettings() async {
    if (_currentThresholds == null) {
      return;
    }

    await showDialog<void>(
      context: context,
      builder: (context) => ThresholdSettingsDialog(
        initialThresholds: _currentThresholds!,
        onSave: (newThresholds) async {
          final messenger = ScaffoldMessenger.of(context);
          await widget.metricsService.setThresholds(newThresholds);
          if (!mounted) return;
          setState(() {
            _currentThresholds = newThresholds;
          });
          messenger.showSnackBar(
            const SnackBar(
              content: Text('Thresholds updated successfully'),
              backgroundColor: Colors.green,
            ),
          );
        },
      ),
    );
  }

  @override
  Widget build(BuildContext context) => Scaffold(
    appBar: AppBar(
      title: const Text('Performance Metrics'),
      actions: [
        IconButton(
          icon: const Icon(Icons.settings),
          onPressed: _showThresholdSettings,
          tooltip: 'Configure thresholds',
        ),
        IconButton(
          icon: const Icon(Icons.download),
          onPressed: _exportMetrics,
          tooltip: 'Export metrics',
        ),
      ],
      bottom: TabBar(
        controller: _tabController,
        tabs: const [
          Tab(icon: Icon(Icons.dashboard), text: 'Dashboard'),
          Tab(icon: Icon(Icons.info_outline), text: 'Info'),
        ],
      ),
    ),
    body: TabBarView(
      controller: _tabController,
      children: [
        // Metrics dashboard tab
        MetricsDashboard(metricsService: widget.metricsService),
        // Info tab
        _buildInfoTab(),
      ],
    ),
  );

  Widget _buildInfoTab() {
    final theme = Theme.of(context);

    return SingleChildScrollView(
      padding: const EdgeInsets.all(16),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          _buildInfoCard(
            theme,
            title: 'About Metrics',
            icon: Icons.analytics,
            children: [
              _buildInfoRow(
                'Purpose',
                'Monitor KeyRx performance in real-time',
              ),
              _buildInfoRow(
                'Latency',
                'Tracks event processing time (P50, P95, P99 percentiles)',
              ),
              _buildInfoRow('Memory', 'Monitors current memory usage'),
              _buildInfoRow('Events', 'Counts processed events and errors'),
            ],
          ),
          const SizedBox(height: 16),
          _buildInfoCard(
            theme,
            title: 'Thresholds',
            icon: Icons.warning_amber,
            children: [
              if (_currentThresholds != null) ...[
                _buildInfoRow(
                  'Latency Warning',
                  '${_currentThresholds!.latencyWarningUs}μs',
                ),
                _buildInfoRow(
                  'Latency Error',
                  '${_currentThresholds!.latencyErrorUs}μs',
                ),
                _buildInfoRow(
                  'Memory Warning',
                  _formatMemory(_currentThresholds!.memoryWarningBytes),
                ),
                _buildInfoRow(
                  'Memory Error',
                  _formatMemory(_currentThresholds!.memoryErrorBytes),
                ),
              ] else
                const Padding(
                  padding: EdgeInsets.all(8),
                  child: Text('Loading thresholds...'),
                ),
            ],
          ),
          const SizedBox(height: 16),
          _buildInfoCard(
            theme,
            title: 'Export',
            icon: Icons.download,
            children: [
              _buildInfoRow('Format', 'JSON with all metric values'),
              _buildInfoRow('Destination', 'Clipboard (ready to paste)'),
              const SizedBox(height: 12),
              ElevatedButton.icon(
                onPressed: _exportMetrics,
                icon: const Icon(Icons.download),
                label: const Text('Export Current Metrics'),
              ),
            ],
          ),
        ],
      ),
    );
  }

  Widget _buildInfoCard(
    ThemeData theme, {
    required String title,
    required IconData icon,
    required List<Widget> children,
  }) => Card(
    child: Padding(
      padding: const EdgeInsets.all(16),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Row(
            children: [
              Icon(icon, color: theme.colorScheme.primary),
              const SizedBox(width: 12),
              Text(
                title,
                style: theme.textTheme.titleMedium?.copyWith(
                  fontWeight: FontWeight.bold,
                ),
              ),
            ],
          ),
          const SizedBox(height: 16),
          ...children,
        ],
      ),
    ),
  );

  Widget _buildInfoRow(String label, String value) => Padding(
    padding: const EdgeInsets.symmetric(vertical: 4),
    child: Row(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        SizedBox(
          width: 140,
          child: Text(
            label,
            style: const TextStyle(fontWeight: FontWeight.w600),
          ),
        ),
        Expanded(child: Text(value)),
      ],
    ),
  );

  String _formatMemory(int bytes) {
    if (bytes < 1024) {
      return '${bytes}B';
    } else if (bytes < 1024 * 1024) {
      return '${(bytes / 1024).toStringAsFixed(1)}KB';
    } else if (bytes < 1024 * 1024 * 1024) {
      return '${(bytes / (1024 * 1024)).toStringAsFixed(1)}MB';
    } else {
      return '${(bytes / (1024 * 1024 * 1024)).toStringAsFixed(2)}GB';
    }
  }
}
