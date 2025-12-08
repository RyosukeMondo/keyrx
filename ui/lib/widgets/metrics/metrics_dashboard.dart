/// MetricsDashboard widget for visualizing real-time performance metrics.
///
/// Provides a comprehensive dashboard with:
/// - Latency percentile line charts (P50, P95, P99)
/// - Memory usage area chart
/// - Real-time metric updates
/// - Historical data visualization
library;

import 'dart:async';
import 'dart:collection';

import 'package:fl_chart/fl_chart.dart';
import 'package:flutter/material.dart';

import '../../services/metrics_service.dart';

/// Default number of data points to keep in history.
const int _kDefaultHistoryPoints = 60;

/// Dashboard widget for displaying performance metrics with charts.
class MetricsDashboard extends StatefulWidget {
  const MetricsDashboard({
    required this.metricsService,
    this.maxHistoryPoints = _kDefaultHistoryPoints,
    super.key,
  }) : assert(maxHistoryPoints > 0);

  final MetricsService metricsService;
  final int maxHistoryPoints;

  @override
  State<MetricsDashboard> createState() => _MetricsDashboardState();
}

class _MetricsDashboardState extends State<MetricsDashboard> {
  final Queue<_MetricsPoint> _history = Queue<_MetricsPoint>();
  StreamSubscription<MetricsSnapshot>? _metricsSubscription;
  StreamSubscription<ThresholdViolation>? _violationSubscription;
  MetricsSnapshot? _currentSnapshot;
  bool _isRunning = false;
  final List<ThresholdViolation> _recentViolations = [];

  @override
  void initState() {
    super.initState();
    _startMetrics();
  }

  @override
  void didUpdateWidget(covariant MetricsDashboard oldWidget) {
    super.didUpdateWidget(oldWidget);
    if (oldWidget.maxHistoryPoints != widget.maxHistoryPoints) {
      _trimHistory();
    }
  }

  Future<void> _startMetrics() async {
    if (_isRunning) {
      return;
    }

    await widget.metricsService.startUpdates();

    // Subscribe to metrics stream
    _metricsSubscription = widget.metricsService.metricsStream.listen((
      snapshot,
    ) {
      setState(() {
        _currentSnapshot = snapshot;
        _addToHistory(snapshot);
      });
    });

    // Subscribe to violation stream
    _violationSubscription = widget.metricsService.violationStream.listen((
      violation,
    ) {
      setState(() {
        _recentViolations.add(violation);
        // Keep only last 10 violations
        if (_recentViolations.length > 10) {
          _recentViolations.removeAt(0);
        }
      });
    });

    setState(() {
      _isRunning = true;
    });

    // Get initial snapshot
    final snapshot = await widget.metricsService.getSnapshot();
    if (snapshot != null) {
      setState(() {
        _currentSnapshot = snapshot;
        _addToHistory(snapshot);
      });
    }
  }

  Future<void> _stopMetrics() async {
    if (!_isRunning) {
      return;
    }

    await _metricsSubscription?.cancel();
    await _violationSubscription?.cancel();
    await widget.metricsService.stopUpdates();

    setState(() {
      _isRunning = false;
    });
  }

  void _addToHistory(MetricsSnapshot snapshot) {
    _history.add(
      _MetricsPoint(
        timestamp: snapshot.dateTime,
        latencyP50: snapshot.eventLatencyP50,
        latencyP95: snapshot.eventLatencyP95,
        latencyP99: snapshot.eventLatencyP99,
        memoryUsed: snapshot.memoryUsed,
      ),
    );

    // Keep only last N points
    while (_history.length > widget.maxHistoryPoints) {
      _history.removeFirst();
    }
  }

  void _clearHistory() {
    setState(() {
      _history.clear();
      _recentViolations.clear();
    });
  }

  void _trimHistory() {
    while (_history.length > widget.maxHistoryPoints) {
      _history.removeFirst();
    }
  }

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);

    return Column(
      children: [
        // Control bar
        _buildControlBar(theme),
        // Violations banner
        if (_recentViolations.isNotEmpty) _buildViolationsBanner(theme),
        // Charts
        Expanded(
          child: _currentSnapshot == null
              ? _buildEmptyState(theme)
              : SingleChildScrollView(
                  padding: const EdgeInsets.all(16),
                  child: Column(
                    crossAxisAlignment: CrossAxisAlignment.stretch,
                    children: [
                      // Current metrics summary
                      _buildMetricsSummary(theme),
                      const SizedBox(height: 24),
                      // Latency chart
                      _buildLatencyChart(theme),
                      const SizedBox(height: 24),
                      // Memory chart
                      _buildMemoryChart(theme),
                    ],
                  ),
                ),
        ),
      ],
    );
  }

  Widget _buildControlBar(ThemeData theme) {
    return Card(
      margin: const EdgeInsets.all(8),
      child: Padding(
        padding: const EdgeInsets.all(12),
        child: Row(
          children: [
            Icon(Icons.analytics, color: theme.colorScheme.primary),
            const SizedBox(width: 12),
            Text(
              'Performance Metrics',
              style: theme.textTheme.titleMedium?.copyWith(
                fontWeight: FontWeight.bold,
              ),
            ),
            const Spacer(),
            // Clear history button
            Tooltip(
              message: 'Clear history',
              child: IconButton(
                icon: const Icon(Icons.clear_all),
                onPressed: _isRunning ? _clearHistory : null,
              ),
            ),
            // Start/Stop toggle
            Tooltip(
              message: _isRunning ? 'Stop monitoring' : 'Start monitoring',
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
    );
  }

  Widget _buildViolationsBanner(ThemeData theme) {
    final latest = _recentViolations.last;
    final color =
        latest.type == ViolationType.latencyError ||
            latest.type == ViolationType.memoryError
        ? theme.colorScheme.error
        : theme.colorScheme.tertiary;

    return Container(
      margin: const EdgeInsets.symmetric(horizontal: 8),
      padding: const EdgeInsets.all(12),
      decoration: BoxDecoration(
        color: color.withOpacity(0.1),
        border: Border.all(color: color),
        borderRadius: BorderRadius.circular(8),
      ),
      child: Row(
        children: [
          Icon(Icons.warning_amber_rounded, color: color, size: 20),
          const SizedBox(width: 12),
          Expanded(
            child: Text(
              latest.message,
              style: theme.textTheme.bodyMedium?.copyWith(color: color),
            ),
          ),
          Text(
            '${_recentViolations.length}',
            style: theme.textTheme.labelSmall?.copyWith(
              color: color,
              fontWeight: FontWeight.bold,
            ),
          ),
        ],
      ),
    );
  }

  Widget _buildEmptyState(ThemeData theme) {
    return Center(
      child: Column(
        mainAxisAlignment: MainAxisAlignment.center,
        children: [
          Icon(
            Icons.analytics_outlined,
            size: 64,
            color: theme.colorScheme.onSurface.withOpacity(0.3),
          ),
          const SizedBox(height: 16),
          Text(
            _isRunning
                ? 'Waiting for metrics data...'
                : 'Click play to start monitoring',
            style: theme.textTheme.bodyLarge?.copyWith(
              color: theme.colorScheme.onSurface.withOpacity(0.5),
            ),
          ),
        ],
      ),
    );
  }

  Widget _buildMetricsSummary(ThemeData theme) {
    final snapshot = _currentSnapshot!;

    return Row(
      children: [
        Expanded(
          child: _buildSummaryCard(
            theme,
            label: 'Events',
            value: _formatCount(snapshot.eventsProcessed),
            icon: Icons.event_available,
            color: Colors.blue,
          ),
        ),
        const SizedBox(width: 12),
        Expanded(
          child: _buildSummaryCard(
            theme,
            label: 'Errors',
            value: _formatCount(snapshot.errorsCount),
            icon: Icons.error_outline,
            color: Colors.red,
          ),
        ),
      ],
    );
  }

  Widget _buildSummaryCard(
    ThemeData theme, {
    required String label,
    required String value,
    required IconData icon,
    required Color color,
  }) {
    return Card(
      child: Padding(
        padding: const EdgeInsets.all(16),
        child: Column(
          children: [
            Icon(icon, color: color, size: 32),
            const SizedBox(height: 8),
            Text(
              value,
              style: theme.textTheme.headlineMedium?.copyWith(
                fontFamily: 'monospace',
                fontWeight: FontWeight.bold,
                color: color,
              ),
            ),
            const SizedBox(height: 4),
            Text(
              label,
              style: theme.textTheme.bodySmall?.copyWith(
                color: theme.colorScheme.onSurface.withOpacity(0.7),
              ),
            ),
          ],
        ),
      ),
    );
  }

  Widget _buildLatencyChart(ThemeData theme) {
    if (_history.isEmpty) {
      return const SizedBox.shrink();
    }

    final points = _history.toList();
    final maxY = points
        .map(
          (p) => [
            p.latencyP50,
            p.latencyP95,
            p.latencyP99,
          ].reduce((a, b) => a > b ? a : b),
        )
        .reduce((a, b) => a > b ? a : b)
        .toDouble();

    return Card(
      child: Padding(
        padding: const EdgeInsets.all(16),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Text(
              'Event Latency',
              style: theme.textTheme.titleMedium?.copyWith(
                fontWeight: FontWeight.bold,
              ),
            ),
            const SizedBox(height: 16),
            SizedBox(
              height: 250,
              child: LineChart(
                LineChartData(
                  gridData: FlGridData(
                    show: true,
                    drawVerticalLine: false,
                    horizontalInterval: maxY / 5,
                    getDrawingHorizontalLine: (value) {
                      return FlLine(
                        color: theme.colorScheme.outline.withOpacity(0.2),
                        strokeWidth: 1,
                      );
                    },
                  ),
                  titlesData: FlTitlesData(
                    leftTitles: AxisTitles(
                      sideTitles: SideTitles(
                        showTitles: true,
                        reservedSize: 60,
                        getTitlesWidget: (value, meta) {
                          return Text(
                            _formatLatency(value.toInt()),
                            style: theme.textTheme.bodySmall,
                          );
                        },
                      ),
                    ),
                    rightTitles: const AxisTitles(
                      sideTitles: SideTitles(showTitles: false),
                    ),
                    topTitles: const AxisTitles(
                      sideTitles: SideTitles(showTitles: false),
                    ),
                    bottomTitles: AxisTitles(
                      sideTitles: SideTitles(
                        showTitles: true,
                        reservedSize: 30,
                        interval: (points.length / 5).ceilToDouble(),
                        getTitlesWidget: (value, meta) {
                          final idx = value.toInt();
                          if (idx < 0 || idx >= points.length) {
                            return const SizedBox.shrink();
                          }
                          final point = points[idx];
                          return Text(
                            '${point.timestamp.minute}:${point.timestamp.second.toString().padLeft(2, '0')}',
                            style: theme.textTheme.bodySmall,
                          );
                        },
                      ),
                    ),
                  ),
                  borderData: FlBorderData(show: false),
                  minY: 0,
                  maxY: maxY * 1.1,
                  lineBarsData: [
                    // P50 line
                    _buildLineData(
                      points,
                      (p) => p.latencyP50.toDouble(),
                      Colors.blue,
                    ),
                    // P95 line
                    _buildLineData(
                      points,
                      (p) => p.latencyP95.toDouble(),
                      Colors.orange,
                    ),
                    // P99 line
                    _buildLineData(
                      points,
                      (p) => p.latencyP99.toDouble(),
                      Colors.red,
                    ),
                  ],
                  lineTouchData: LineTouchData(
                    touchTooltipData: LineTouchTooltipData(
                      getTooltipItems: (touchedSpots) {
                        return touchedSpots.map((spot) {
                          final labels = ['P50', 'P95', 'P99'];
                          return LineTooltipItem(
                            '${labels[spot.barIndex]}: ${_formatLatency(spot.y.toInt())}',
                            TextStyle(
                              color: spot.bar.color,
                              fontWeight: FontWeight.bold,
                            ),
                          );
                        }).toList();
                      },
                    ),
                  ),
                ),
              ),
            ),
            const SizedBox(height: 12),
            // Legend
            Row(
              mainAxisAlignment: MainAxisAlignment.center,
              children: [
                _buildLegendItem(theme, 'P50', Colors.blue),
                const SizedBox(width: 16),
                _buildLegendItem(theme, 'P95', Colors.orange),
                const SizedBox(width: 16),
                _buildLegendItem(theme, 'P99', Colors.red),
              ],
            ),
          ],
        ),
      ),
    );
  }

  Widget _buildMemoryChart(ThemeData theme) {
    if (_history.isEmpty) {
      return const SizedBox.shrink();
    }

    final points = _history.toList();
    final maxY = points
        .map((p) => p.memoryUsed)
        .reduce((a, b) => a > b ? a : b)
        .toDouble();

    return Card(
      child: Padding(
        padding: const EdgeInsets.all(16),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Text(
              'Memory Usage',
              style: theme.textTheme.titleMedium?.copyWith(
                fontWeight: FontWeight.bold,
              ),
            ),
            const SizedBox(height: 16),
            SizedBox(
              height: 200,
              child: LineChart(
                LineChartData(
                  gridData: FlGridData(
                    show: true,
                    drawVerticalLine: false,
                    horizontalInterval: maxY / 4,
                    getDrawingHorizontalLine: (value) {
                      return FlLine(
                        color: theme.colorScheme.outline.withOpacity(0.2),
                        strokeWidth: 1,
                      );
                    },
                  ),
                  titlesData: FlTitlesData(
                    leftTitles: AxisTitles(
                      sideTitles: SideTitles(
                        showTitles: true,
                        reservedSize: 60,
                        getTitlesWidget: (value, meta) {
                          return Text(
                            _formatMemory(value.toInt()),
                            style: theme.textTheme.bodySmall,
                          );
                        },
                      ),
                    ),
                    rightTitles: const AxisTitles(
                      sideTitles: SideTitles(showTitles: false),
                    ),
                    topTitles: const AxisTitles(
                      sideTitles: SideTitles(showTitles: false),
                    ),
                    bottomTitles: AxisTitles(
                      sideTitles: SideTitles(
                        showTitles: true,
                        reservedSize: 30,
                        interval: (points.length / 5).ceilToDouble(),
                        getTitlesWidget: (value, meta) {
                          final idx = value.toInt();
                          if (idx < 0 || idx >= points.length) {
                            return const SizedBox.shrink();
                          }
                          final point = points[idx];
                          return Text(
                            '${point.timestamp.minute}:${point.timestamp.second.toString().padLeft(2, '0')}',
                            style: theme.textTheme.bodySmall,
                          );
                        },
                      ),
                    ),
                  ),
                  borderData: FlBorderData(show: false),
                  minY: 0,
                  maxY: maxY * 1.1,
                  lineBarsData: [
                    LineChartBarData(
                      spots: points.asMap().entries.map((entry) {
                        return FlSpot(
                          entry.key.toDouble(),
                          entry.value.memoryUsed.toDouble(),
                        );
                      }).toList(),
                      isCurved: true,
                      color: Colors.purple,
                      barWidth: 2,
                      isStrokeCapRound: true,
                      dotData: const FlDotData(show: false),
                      belowBarData: BarAreaData(
                        show: true,
                        color: Colors.purple.withOpacity(0.3),
                      ),
                    ),
                  ],
                  lineTouchData: LineTouchData(
                    touchTooltipData: LineTouchTooltipData(
                      getTooltipItems: (touchedSpots) {
                        return touchedSpots.map((spot) {
                          return LineTooltipItem(
                            _formatMemory(spot.y.toInt()),
                            const TextStyle(
                              color: Colors.purple,
                              fontWeight: FontWeight.bold,
                            ),
                          );
                        }).toList();
                      },
                    ),
                  ),
                ),
              ),
            ),
          ],
        ),
      ),
    );
  }

  LineChartBarData _buildLineData(
    List<_MetricsPoint> points,
    double Function(_MetricsPoint) getValue,
    Color color,
  ) {
    return LineChartBarData(
      spots: points.asMap().entries.map((entry) {
        return FlSpot(entry.key.toDouble(), getValue(entry.value));
      }).toList(),
      isCurved: true,
      color: color,
      barWidth: 2,
      isStrokeCapRound: true,
      dotData: const FlDotData(show: false),
    );
  }

  Widget _buildLegendItem(ThemeData theme, String label, Color color) {
    return Row(
      mainAxisSize: MainAxisSize.min,
      children: [
        Container(
          width: 16,
          height: 3,
          decoration: BoxDecoration(
            color: color,
            borderRadius: BorderRadius.circular(2),
          ),
        ),
        const SizedBox(width: 6),
        Text(label, style: theme.textTheme.bodySmall),
      ],
    );
  }

  String _formatLatency(int microseconds) {
    if (microseconds < 1000) {
      return '$microsecondsμs';
    } else if (microseconds < 1000000) {
      return '${(microseconds / 1000).toStringAsFixed(1)}ms';
    } else {
      return '${(microseconds / 1000000).toStringAsFixed(2)}s';
    }
  }

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

  String _formatCount(int count) {
    if (count < 1000) {
      return count.toString();
    } else if (count < 1000000) {
      return '${(count / 1000).toStringAsFixed(1)}K';
    } else {
      return '${(count / 1000000).toStringAsFixed(1)}M';
    }
  }

  @override
  void dispose() {
    _metricsSubscription?.cancel();
    _violationSubscription?.cancel();
    widget.metricsService.stopUpdates();
    super.dispose();
  }
}

/// Internal data point for historical tracking.
class _MetricsPoint {
  _MetricsPoint({
    required this.timestamp,
    required this.latencyP50,
    required this.latencyP95,
    required this.latencyP99,
    required this.memoryUsed,
  });

  final DateTime timestamp;
  final int latencyP50;
  final int latencyP95;
  final int latencyP99;
  final int memoryUsed;
}
