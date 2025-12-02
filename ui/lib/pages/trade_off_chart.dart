// Trade-off chart widget for timing visualization.
//
// Contains the fl_chart configuration and rendering logic for
// displaying the tap-hold timeout vs miss rate relationship.

import 'dart:math' as math;

import 'package:fl_chart/fl_chart.dart';
import 'package:flutter/material.dart';

import 'typing_simulator.dart';

/// A preset timing region with display properties.
class PresetRegion {
  const PresetRegion({
    required this.name,
    required this.minMs,
    required this.maxMs,
    required this.color,
    required this.description,
  });

  final String name;
  final int minMs;
  final int maxMs;
  final Color color;
  final String description;

  /// Default preset configurations.
  static const List<PresetRegion> defaults = [
    PresetRegion(
      name: 'Gaming',
      minMs: 100,
      maxMs: 150,
      color: Colors.red,
      description: 'Fast response, higher miss rate',
    ),
    PresetRegion(
      name: 'Typing',
      minMs: 175,
      maxMs: 250,
      color: Colors.green,
      description: 'Balanced for regular typing',
    ),
    PresetRegion(
      name: 'Relaxed',
      minMs: 300,
      maxMs: 500,
      color: Colors.blue,
      description: 'Slower, fewer accidental taps',
    ),
  ];

  /// Check if a timeout value is within this preset's range.
  bool containsTimeout(double timeout) {
    return timeout >= minMs && timeout <= maxMs;
  }

  /// Get the middle value of this preset's range.
  double get middleValue => (minMs + maxMs) / 2;
}

/// Configuration for the trade-off chart.
class TradeOffChartConfig {
  const TradeOffChartConfig({
    this.minTimeout = 100,
    this.maxTimeout = 1000,
    this.maxMissRate = 35,
  });

  final double minTimeout;
  final double maxTimeout;
  final double maxMissRate;
}

/// Widget that displays the trade-off chart between tap-hold timeout
/// and estimated miss rate.
class TradeOffChart extends StatelessWidget {
  const TradeOffChart({
    super.key,
    required this.statistics,
    required this.currentTimeout,
    this.config = const TradeOffChartConfig(),
    this.userTypingMean,
    this.userTypingStdDev,
    this.presets = PresetRegion.defaults,
  });

  final TypingStatistics statistics;
  final double currentTimeout;
  final TradeOffChartConfig config;
  final double? userTypingMean;
  final double? userTypingStdDev;
  final List<PresetRegion> presets;

  bool get hasUserProfile => userTypingMean != null;

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);
    final spots = _generateChartData();
    final currentMissRate = statistics.calculateMissRate(currentTimeout);

    return LineChart(
      LineChartData(
        minX: config.minTimeout,
        maxX: config.maxTimeout,
        minY: 0,
        maxY: config.maxMissRate,
        gridData: _buildGridData(theme),
        titlesData: _buildTitlesData(theme),
        borderData: FlBorderData(
          show: true,
          border: Border.all(color: theme.dividerColor),
        ),
        lineBarsData: [
          // Preset region backgrounds (for reference)
          ...presets.map(_buildPresetRegionBar),
          // Main curve
          _buildMainCurve(theme, spots),
        ],
        extraLinesData: _buildExtraLines(theme, currentMissRate),
        lineTouchData: _buildTouchData(theme),
      ),
    );
  }

  FlGridData _buildGridData(ThemeData theme) {
    return FlGridData(
      show: true,
      drawVerticalLine: true,
      horizontalInterval: 5,
      verticalInterval: 100,
      getDrawingHorizontalLine: (value) => FlLine(
        color: theme.dividerColor.withValues(alpha: 0.3),
        strokeWidth: 1,
      ),
      getDrawingVerticalLine: (value) => FlLine(
        color: theme.dividerColor.withValues(alpha: 0.3),
        strokeWidth: 1,
      ),
    );
  }

  FlTitlesData _buildTitlesData(ThemeData theme) {
    return FlTitlesData(
      leftTitles: AxisTitles(
        axisNameWidget: const Text('Miss Rate (%)'),
        sideTitles: SideTitles(
          showTitles: true,
          reservedSize: 40,
          interval: 5,
          getTitlesWidget: (value, meta) => Text(
            '${value.toInt()}%',
            style: theme.textTheme.bodySmall,
          ),
        ),
      ),
      bottomTitles: AxisTitles(
        axisNameWidget: const Text('Tap-Hold Timeout (ms)'),
        sideTitles: SideTitles(
          showTitles: true,
          reservedSize: 30,
          interval: 200,
          getTitlesWidget: (value, meta) => Text(
            '${value.toInt()}',
            style: theme.textTheme.bodySmall,
          ),
        ),
      ),
      topTitles: const AxisTitles(
        sideTitles: SideTitles(showTitles: false),
      ),
      rightTitles: const AxisTitles(
        sideTitles: SideTitles(showTitles: false),
      ),
    );
  }

  LineChartBarData _buildPresetRegionBar(PresetRegion preset) {
    // Create a subtle background bar for the preset region
    return LineChartBarData(
      spots: [
        FlSpot(preset.minMs.toDouble(), 0),
        FlSpot(preset.maxMs.toDouble(), 0),
      ],
      isCurved: false,
      color: Colors.transparent,
      barWidth: 0,
      dotData: const FlDotData(show: false),
      belowBarData: BarAreaData(show: false),
    );
  }

  LineChartBarData _buildMainCurve(ThemeData theme, List<FlSpot> spots) {
    return LineChartBarData(
      spots: spots,
      isCurved: true,
      curveSmoothness: 0.3,
      color: theme.colorScheme.primary,
      barWidth: 3,
      dotData: const FlDotData(show: false),
      belowBarData: BarAreaData(
        show: true,
        color: theme.colorScheme.primary.withValues(alpha: 0.1),
      ),
    );
  }

  ExtraLinesData _buildExtraLines(ThemeData theme, double currentMissRate) {
    final verticalLines = <VerticalLine>[];

    // User typing profile band (if available)
    if (hasUserProfile && userTypingMean != null) {
      final stdDev = userTypingStdDev ?? 50;
      verticalLines.addAll([
        VerticalLine(
          x: (userTypingMean! - stdDev).clamp(config.minTimeout, config.maxTimeout),
          color: Colors.purple.withValues(alpha: 0.5),
          strokeWidth: 1,
          dashArray: [3, 3],
        ),
        VerticalLine(
          x: userTypingMean!.clamp(config.minTimeout, config.maxTimeout),
          color: Colors.purple,
          strokeWidth: 2,
          label: VerticalLineLabel(
            show: true,
            alignment: Alignment.topLeft,
            style: const TextStyle(
              color: Colors.purple,
              fontWeight: FontWeight.bold,
              fontSize: 10,
            ),
            labelResolver: (_) => 'Your typing\n${userTypingMean!.toInt()}ms',
          ),
        ),
        VerticalLine(
          x: (userTypingMean! + stdDev).clamp(config.minTimeout, config.maxTimeout),
          color: Colors.purple.withValues(alpha: 0.5),
          strokeWidth: 1,
          dashArray: [3, 3],
        ),
      ]);
    }

    // Current threshold line
    verticalLines.add(
      VerticalLine(
        x: currentTimeout,
        color: theme.colorScheme.secondary,
        strokeWidth: 2,
        dashArray: [5, 5],
        label: VerticalLineLabel(
          show: true,
          alignment: Alignment.topRight,
          style: TextStyle(
            color: theme.colorScheme.secondary,
            fontWeight: FontWeight.bold,
            fontSize: 12,
          ),
          labelResolver: (_) =>
              '${currentTimeout.toInt()}ms\n${currentMissRate.toStringAsFixed(1)}%',
        ),
      ),
    );

    return ExtraLinesData(verticalLines: verticalLines);
  }

  LineTouchData _buildTouchData(ThemeData theme) {
    return LineTouchData(
      touchTooltipData: LineTouchTooltipData(
        getTooltipItems: (touchedSpots) {
          return touchedSpots.map((spot) {
            return LineTooltipItem(
              'Timeout: ${spot.x.toInt()}ms\nMiss Rate: ${spot.y.toStringAsFixed(1)}%',
              TextStyle(color: theme.colorScheme.onSurface),
            );
          }).toList();
        },
      ),
    );
  }

  List<FlSpot> _generateChartData() {
    final spots = <FlSpot>[];
    for (double timeout = config.minTimeout; timeout <= config.maxTimeout; timeout += 10) {
      final missRate = statistics.calculateMissRate(timeout);
      spots.add(FlSpot(timeout, missRate.clamp(0, config.maxMissRate)));
    }
    return spots;
  }
}
