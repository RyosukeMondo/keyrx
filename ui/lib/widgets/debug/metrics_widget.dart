/// MetricsWidget for displaying real-time performance metrics.
///
/// Provides a dashboard-style view of key metrics:
/// - Event latency percentiles (P50, P95, P99)
/// - Events processed count
/// - Error count
/// - Memory usage
library;

import 'dart:async';

import 'package:flutter/material.dart';

import '../../services/observability_service.dart';

/// Widget for viewing real-time performance metrics.
class MetricsWidget extends StatefulWidget {
  const MetricsWidget({required this.observabilityService, super.key});

  final ObservabilityService observabilityService;

  @override
  State<MetricsWidget> createState() => _MetricsWidgetState();
}

class _MetricsWidgetState extends State<MetricsWidget> {
  MetricsSnapshot? _currentSnapshot;
  MetricsSnapshot? _previousSnapshot;
  StreamSubscription<MetricsSnapshot>? _metricsSubscription;
  bool _isRunning = false;

  @override
  void initState() {
    super.initState();
    _initializeMetrics();
  }

  Future<void> _initializeMetrics() async {
    await widget.observabilityService.initialize();
    await _startMetrics();
  }

  Future<void> _startMetrics() async {
    if (_isRunning) {
      return;
    }

    await widget.observabilityService.startMetricsUpdates();

    // Subscribe to metrics stream
    _metricsSubscription = widget.observabilityService.metricsStream.listen((
      snapshot,
    ) {
      setState(() {
        _previousSnapshot = _currentSnapshot;
        _currentSnapshot = snapshot;
      });
    });

    setState(() {
      _isRunning = true;
    });
  }

  Future<void> _stopMetrics() async {
    if (!_isRunning) {
      return;
    }

    await _metricsSubscription?.cancel();
    await widget.observabilityService.stopMetricsUpdates();

    setState(() {
      _isRunning = false;
    });
  }

  Future<void> _refreshMetrics() async {
    final snapshot = await widget.observabilityService.getMetrics();
    if (snapshot != null) {
      setState(() {
        _previousSnapshot = _currentSnapshot;
        _currentSnapshot = snapshot;
      });
    }
  }

  String _formatLatency(int microseconds) {
    if (microseconds < 1000) {
      return '$microsecondsμs';
    } else if (microseconds < 1000000) {
      return '${(microseconds / 1000).toStringAsFixed(2)}ms';
    } else {
      return '${(microseconds / 1000000).toStringAsFixed(2)}s';
    }
  }

  String _formatMemory(int bytes) {
    if (bytes < 1024) {
      return '${bytes}B';
    } else if (bytes < 1024 * 1024) {
      return '${(bytes / 1024).toStringAsFixed(2)}KB';
    } else if (bytes < 1024 * 1024 * 1024) {
      return '${(bytes / (1024 * 1024)).toStringAsFixed(2)}MB';
    } else {
      return '${(bytes / (1024 * 1024 * 1024)).toStringAsFixed(2)}GB';
    }
  }

  String _formatCount(int count) {
    if (count < 1000) {
      return count.toString();
    } else if (count < 1000000) {
      return '${(count / 1000).toStringAsFixed(1)}K';
    } else {
      return '${(count / 1000000).toStringAsFixed(1)}M';
    }
  }

  int? _getDelta(int current, int? previous) {
    if (previous == null) {
      return null;
    }
    return current - previous;
  }

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);

    return Column(
      children: [
        // Control bar
        Card(
          margin: const EdgeInsets.all(8),
          child: Padding(
            padding: const EdgeInsets.all(8),
            child: Row(
              children: [
                Text('Metrics Dashboard', style: theme.textTheme.titleMedium),
                const Spacer(),
                // Refresh button
                Tooltip(
                  message: 'Refresh now',
                  child: IconButton(
                    icon: const Icon(Icons.refresh),
                    onPressed: _isRunning ? _refreshMetrics : null,
                  ),
                ),
                // Start/Stop toggle
                Tooltip(
                  message: _isRunning ? 'Stop updates' : 'Start updates',
                  child: IconButton(
                    icon: Icon(_isRunning ? Icons.pause : Icons.play_arrow),
                    onPressed: _isRunning ? _stopMetrics : _startMetrics,
                    color: _isRunning
                        ? theme.colorScheme.primary
                        : theme.iconTheme.color,
                  ),
                ),
              ],
            ),
          ),
        ),
        // Metrics content
        Expanded(
          child: _currentSnapshot == null
              ? Center(
                  child: Column(
                    mainAxisAlignment: MainAxisAlignment.center,
                    children: [
                      Icon(
                        Icons.analytics_outlined,
                        size: 64,
                        color: theme.colorScheme.onSurface.withValues(alpha: 0.3),
                      ),
                      const SizedBox(height: 16),
                      Text(
                        _isRunning
                            ? 'Waiting for metrics...'
                            : 'Click play to start metrics',
                        style: theme.textTheme.bodyLarge?.copyWith(
                          color: theme.colorScheme.onSurface.withValues(alpha: 0.5),
                        ),
                      ),
                    ],
                  ),
                )
              : SingleChildScrollView(
                  padding: const EdgeInsets.all(8),
                  child: Column(
                    children: [
                      // Timestamp
                      _buildInfoCard(
                        context,
                        title: 'Last Updated',
                        value: _formatTimestamp(_currentSnapshot!.dateTime),
                        icon: Icons.access_time,
                      ),
                      const SizedBox(height: 8),
                      // Latency metrics
                      _buildSectionHeader(context, 'Event Latency'),
                      Row(
                        children: [
                          Expanded(
                            child: _buildMetricCard(
                              context,
                              label: 'P50',
                              value: _formatLatency(
                                _currentSnapshot!.eventLatencyP50,
                              ),
                              icon: Icons.speed,
                              color: Colors.blue,
                            ),
                          ),
                          const SizedBox(width: 8),
                          Expanded(
                            child: _buildMetricCard(
                              context,
                              label: 'P95',
                              value: _formatLatency(
                                _currentSnapshot!.eventLatencyP95,
                              ),
                              icon: Icons.speed,
                              color: Colors.orange,
                            ),
                          ),
                          const SizedBox(width: 8),
                          Expanded(
                            child: _buildMetricCard(
                              context,
                              label: 'P99',
                              value: _formatLatency(
                                _currentSnapshot!.eventLatencyP99,
                              ),
                              icon: Icons.speed,
                              color: Colors.red,
                            ),
                          ),
                        ],
                      ),
                      const SizedBox(height: 16),
                      // Throughput metrics
                      _buildSectionHeader(context, 'Throughput'),
                      Row(
                        children: [
                          Expanded(
                            child: _buildMetricCard(
                              context,
                              label: 'Events Processed',
                              value: _formatCount(
                                _currentSnapshot!.eventsProcessed,
                              ),
                              delta: _getDelta(
                                _currentSnapshot!.eventsProcessed,
                                _previousSnapshot?.eventsProcessed,
                              ),
                              icon: Icons.event_available,
                              color: Colors.green,
                            ),
                          ),
                          const SizedBox(width: 8),
                          Expanded(
                            child: _buildMetricCard(
                              context,
                              label: 'Errors',
                              value: _formatCount(
                                _currentSnapshot!.errorsCount,
                              ),
                              delta: _getDelta(
                                _currentSnapshot!.errorsCount,
                                _previousSnapshot?.errorsCount,
                              ),
                              icon: Icons.error_outline,
                              color: Colors.red,
                              highlightDelta: true,
                            ),
                          ),
                        ],
                      ),
                      const SizedBox(height: 16),
                      // Resource metrics
                      _buildSectionHeader(context, 'Resources'),
                      _buildMetricCard(
                        context,
                        label: 'Memory Used',
                        value: _formatMemory(_currentSnapshot!.memoryUsed),
                        delta: _getDelta(
                          _currentSnapshot!.memoryUsed,
                          _previousSnapshot?.memoryUsed,
                        ),
                        icon: Icons.memory,
                        color: Colors.purple,
                      ),
                    ],
                  ),
                ),
        ),
      ],
    );
  }

  Widget _buildSectionHeader(BuildContext context, String title) {
    final theme = Theme.of(context);
    return Padding(
      padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 4),
      child: Align(
        alignment: Alignment.centerLeft,
        child: Text(
          title,
          style: theme.textTheme.titleSmall?.copyWith(
            fontWeight: FontWeight.bold,
            color: theme.colorScheme.primary,
          ),
        ),
      ),
    );
  }

  Widget _buildInfoCard(
    BuildContext context, {
    required String title,
    required String value,
    required IconData icon,
  }) {
    final theme = Theme.of(context);
    return Card(
      child: Padding(
        padding: const EdgeInsets.all(12),
        child: Row(
          children: [
            Icon(icon, size: 20, color: theme.colorScheme.primary),
            const SizedBox(width: 12),
            Text(
              title,
              style: theme.textTheme.bodyMedium?.copyWith(
                color: theme.colorScheme.onSurface.withValues(alpha: 0.7),
              ),
            ),
            const SizedBox(width: 8),
            Text(
              value,
              style: theme.textTheme.bodyLarge?.copyWith(
                fontFamily: 'monospace',
                fontWeight: FontWeight.bold,
              ),
            ),
          ],
        ),
      ),
    );
  }

  Widget _buildMetricCard(
    BuildContext context, {
    required String label,
    required String value,
    required IconData icon,
    required Color color,
    int? delta,
    bool highlightDelta = false,
  }) {
    final theme = Theme.of(context);
    final hasDelta = delta != null;

    Color? deltaColor;
    IconData? deltaIcon;
    if (hasDelta) {
      if (highlightDelta) {
        // For errors, increases are bad (red), decreases are good (green)
        if (delta > 0) {
          deltaColor = Colors.red;
          deltaIcon = Icons.arrow_upward;
        } else if (delta < 0) {
          deltaColor = Colors.green;
          deltaIcon = Icons.arrow_downward;
        }
      } else {
        // For normal metrics, just show direction
        if (delta > 0) {
          deltaColor = Colors.blue;
          deltaIcon = Icons.arrow_upward;
        } else if (delta < 0) {
          deltaColor = Colors.grey;
          deltaIcon = Icons.arrow_downward;
        }
      }
    }

    return Card(
      child: Padding(
        padding: const EdgeInsets.all(12),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Row(
              children: [
                Icon(icon, size: 16, color: color),
                const SizedBox(width: 8),
                Expanded(
                  child: Text(
                    label,
                    style: theme.textTheme.bodySmall?.copyWith(
                      color: theme.colorScheme.onSurface.withValues(alpha: 0.7),
                    ),
                    overflow: TextOverflow.ellipsis,
                  ),
                ),
              ],
            ),
            const SizedBox(height: 8),
            Row(
              children: [
                Expanded(
                  child: Text(
                    value,
                    style: theme.textTheme.titleLarge?.copyWith(
                      fontFamily: 'monospace',
                      fontWeight: FontWeight.bold,
                      color: color,
                    ),
                  ),
                ),
                if (hasDelta && delta != 0 && deltaIcon != null)
                  Row(
                    mainAxisSize: MainAxisSize.min,
                    children: [
                      Icon(deltaIcon, size: 14, color: deltaColor),
                      const SizedBox(width: 2),
                      Text(
                        _formatCount(delta.abs()),
                        style: theme.textTheme.bodySmall?.copyWith(
                          fontFamily: 'monospace',
                          color: deltaColor,
                          fontWeight: FontWeight.bold,
                        ),
                      ),
                    ],
                  ),
              ],
            ),
          ],
        ),
      ),
    );
  }

  String _formatTimestamp(DateTime dt) {
    return '${dt.hour.toString().padLeft(2, '0')}:'
        '${dt.minute.toString().padLeft(2, '0')}:'
        '${dt.second.toString().padLeft(2, '0')}';
  }

  @override
  void dispose() {
    _metricsSubscription?.cancel();
    widget.observabilityService.stopMetricsUpdates();
    super.dispose();
  }
}

