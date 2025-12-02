// Trade-off visualizer for tap-hold timing configuration.
//
// Shows an interactive chart displaying the relationship between
// tap-hold timeout and estimated miss rate, helping users find
// optimal timing settings for their typing style.

import 'dart:math' as math;

import 'package:fl_chart/fl_chart.dart';
import 'package:flutter/material.dart';
import 'package:provider/provider.dart';

import '../services/engine_service.dart';
import '../services/service_registry.dart';
import 'trade_off_widgets.dart';

/// Trade-off visualizer page for timing configuration.
class TradeOffVisualizerPage extends StatefulWidget {
  const TradeOffVisualizerPage({super.key});

  @override
  State<TradeOffVisualizerPage> createState() => _TradeOffVisualizerPageState();
}

class _TradeOffVisualizerPageState extends State<TradeOffVisualizerPage> {
  // Timeout range: 100ms to 1000ms
  static const double _minTimeout = 100;
  static const double _maxTimeout = 1000;

  // Default typing statistics (can be overridden by user simulation)
  double _typingMean = 180.0; // Average inter-key delay in ms
  double _typingStdDev = 50.0; // Standard deviation

  // Current slider value
  double _currentTimeout = 200.0;

  // User typing profile from simulation
  bool _hasUserProfile = false;
  double? _userTypingMean;
  double? _userTypingStdDev;
  double? _recommendedTimeout;

  // Sample text for typing simulation
  static const String _sampleText =
      'The quick brown fox jumps over the lazy dog. '
      'Pack my box with five dozen liquor jugs.';

  // Preset regions
  static const List<_PresetRegion> _presets = [
    _PresetRegion(
      name: 'Gaming',
      minMs: 100,
      maxMs: 150,
      color: Colors.red,
      description: 'Fast response, higher miss rate',
    ),
    _PresetRegion(
      name: 'Typing',
      minMs: 175,
      maxMs: 250,
      color: Colors.green,
      description: 'Balanced for regular typing',
    ),
    _PresetRegion(
      name: 'Relaxed',
      minMs: 300,
      maxMs: 500,
      color: Colors.blue,
      description: 'Slower, fewer accidental taps',
    ),
  ];

  EngineService? _engine;

  @override
  void initState() {
    super.initState();
    final registry = Provider.of<ServiceRegistry>(context, listen: false);
    _engine = registry.engineService;
    _loadCurrentTiming();
  }


  void _loadCurrentTiming() {
    // Try to load current timing from engine state
    final stateStream = _engine?.stateStream;
    if (stateStream != null) {
      stateStream.first.then((snapshot) {
        final timing = snapshot.timing;
        if (timing?.tapTimeoutMs != null && mounted) {
          setState(() {
            _currentTimeout = timing!.tapTimeoutMs!.toDouble();
          });
        }
      }).catchError((_) {});
    }
  }

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);

    return Scaffold(
      appBar: AppBar(
        title: const Text('Timing Trade-offs'),
        actions: [
          IconButton(
            icon: const Icon(Icons.help_outline),
            onPressed: _showHelpDialog,
            tooltip: 'Help',
          ),
        ],
      ),
      body: SingleChildScrollView(
        padding: const EdgeInsets.all(16),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            _buildExplanationCard(theme),
            const SizedBox(height: 16),
            _buildChartCard(theme),
            const SizedBox(height: 16),
            _buildSliderCard(theme),
            const SizedBox(height: 16),
            _buildPresetsCard(theme),
            const SizedBox(height: 16),
            _buildStatisticsCard(theme),
          ],
        ),
      ),
    );
  }

  Widget _buildExplanationCard(ThemeData theme) {
    return Card(
      child: Padding(
        padding: const EdgeInsets.all(16),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Row(
              children: [
                Icon(Icons.info_outline, color: theme.colorScheme.primary),
                const SizedBox(width: 8),
                Text(
                  'Understanding Tap-Hold Timing',
                  style: theme.textTheme.titleMedium,
                ),
              ],
            ),
            const SizedBox(height: 12),
            Text(
              'The tap-hold timeout determines how long you must hold a key '
              'before it triggers the "hold" action instead of "tap". '
              'A shorter timeout means faster hold activation but higher '
              'chance of accidental holds during fast typing.',
              style: theme.textTheme.bodyMedium,
            ),
          ],
        ),
      ),
    );
  }

  Widget _buildChartCard(ThemeData theme) {
    return Card(
      child: Padding(
        padding: const EdgeInsets.all(16),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Text(
              'Miss Rate vs. Timeout',
              style: theme.textTheme.titleMedium,
            ),
            const SizedBox(height: 8),
            SizedBox(
              height: 300,
              child: _buildChart(theme),
            ),
          ],
        ),
      ),
    );
  }

  Widget _buildChart(ThemeData theme) {
    final spots = _generateChartData();
    final currentMissRate = _calculateMissRate(_currentTimeout);

    return LineChart(
      LineChartData(
        minX: _minTimeout,
        maxX: _maxTimeout,
        minY: 0,
        maxY: 35,
        gridData: FlGridData(
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
        ),
        titlesData: FlTitlesData(
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
        ),
        borderData: FlBorderData(
          show: true,
          border: Border.all(color: theme.dividerColor),
        ),
        lineBarsData: [
          // Preset region backgrounds
          ..._presets.map((preset) => _buildPresetRegionBar(preset)),
          // Main curve
          LineChartBarData(
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
          ),
        ],
        extraLinesData: ExtraLinesData(
          verticalLines: [
            // User typing profile band (if available)
            if (_hasUserProfile && _userTypingMean != null) ...[
              VerticalLine(
                x: (_userTypingMean! - (_userTypingStdDev ?? 50)).clamp(_minTimeout, _maxTimeout),
                color: Colors.purple.withValues(alpha: 0.5),
                strokeWidth: 1,
                dashArray: [3, 3],
              ),
              VerticalLine(
                x: _userTypingMean!.clamp(_minTimeout, _maxTimeout),
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
                  labelResolver: (_) => 'Your typing\n${_userTypingMean!.toInt()}ms',
                ),
              ),
              VerticalLine(
                x: (_userTypingMean! + (_userTypingStdDev ?? 50)).clamp(_minTimeout, _maxTimeout),
                color: Colors.purple.withValues(alpha: 0.5),
                strokeWidth: 1,
                dashArray: [3, 3],
              ),
            ],
            // Current threshold line
            VerticalLine(
              x: _currentTimeout,
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
                    '${_currentTimeout.toInt()}ms\n${currentMissRate.toStringAsFixed(1)}%',
              ),
            ),
          ],
        ),
        lineTouchData: LineTouchData(
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
        ),
      ),
    );
  }

  LineChartBarData _buildPresetRegionBar(_PresetRegion preset) {
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

  List<FlSpot> _generateChartData() {
    final spots = <FlSpot>[];
    for (double timeout = _minTimeout; timeout <= _maxTimeout; timeout += 10) {
      final missRate = _calculateMissRate(timeout);
      spots.add(FlSpot(timeout, missRate.clamp(0, 35)));
    }
    return spots;
  }

  /// Calculate estimated miss rate using cumulative distribution function.
  ///
  /// The miss rate represents the probability that a key press intended as
  /// a "tap" will exceed the threshold and be interpreted as a "hold".
  /// This uses a normal distribution model of inter-key delays.
  double _calculateMissRate(double threshold) {
    // P(miss) = P(key_held_time > threshold)
    // For typing, key held time follows approximately normal distribution
    // Miss rate = 1 - normalCdf(threshold, mean, stddev)
    // But we want P(intended_tap > threshold), so we model key press duration
    //
    // Using complementary error function for normal CDF:
    // normalCdf(x) = 0.5 * (1 + erf((x - mean) / (stddev * sqrt(2))))

    final z = (threshold - _typingMean) / (_typingStdDev * math.sqrt(2));
    final cdf = 0.5 * (1 + _erf(z));

    // Miss rate is the probability of being below threshold
    // (i.e., releasing too quickly when trying to hold)
    // For tap-hold, we want P(tap_duration > threshold) which causes false hold
    // This is 1 - CDF
    final missRate = (1 - cdf) * 100;

    return missRate;
  }

  /// Approximate error function using Horner's method.
  double _erf(double x) {
    // Approximation constants
    const a1 = 0.254829592;
    const a2 = -0.284496736;
    const a3 = 1.421413741;
    const a4 = -1.453152027;
    const a5 = 1.061405429;
    const p = 0.3275911;

    final sign = x < 0 ? -1 : 1;
    final absX = x.abs();

    final t = 1.0 / (1.0 + p * absX);
    final y =
        1.0 - (((((a5 * t + a4) * t) + a3) * t + a2) * t + a1) * t * math.exp(-absX * absX);

    return sign * y;
  }

  Widget _buildSliderCard(ThemeData theme) {
    final currentMissRate = _calculateMissRate(_currentTimeout);
    final currentPreset = _getPresetForTimeout(_currentTimeout);

    return Card(
      child: Padding(
        padding: const EdgeInsets.all(16),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Row(
              mainAxisAlignment: MainAxisAlignment.spaceBetween,
              children: [
                Text(
                  'Adjust Timeout',
                  style: theme.textTheme.titleMedium,
                ),
                Container(
                  padding:
                      const EdgeInsets.symmetric(horizontal: 12, vertical: 6),
                  decoration: BoxDecoration(
                    color: theme.colorScheme.primaryContainer,
                    borderRadius: BorderRadius.circular(8),
                  ),
                  child: Text(
                    '${_currentTimeout.toInt()} ms',
                    style: theme.textTheme.titleLarge?.copyWith(
                      fontWeight: FontWeight.bold,
                      color: theme.colorScheme.onPrimaryContainer,
                    ),
                  ),
                ),
              ],
            ),
            const SizedBox(height: 16),
            SliderTheme(
              data: SliderTheme.of(context).copyWith(
                activeTrackColor: currentPreset?.color ??
                    theme.colorScheme.primary,
                thumbColor: currentPreset?.color ?? theme.colorScheme.primary,
              ),
              child: Slider(
                value: _currentTimeout,
                min: _minTimeout,
                max: _maxTimeout,
                divisions: 90,
                label: '${_currentTimeout.toInt()} ms',
                onChanged: (value) {
                  setState(() {
                    _currentTimeout = value;
                  });
                },
              ),
            ),
            const SizedBox(height: 8),
            Row(
              mainAxisAlignment: MainAxisAlignment.spaceBetween,
              children: [
                Text(
                  'Faster (${_minTimeout.toInt()}ms)',
                  style: theme.textTheme.bodySmall,
                ),
                Text(
                  'Slower (${_maxTimeout.toInt()}ms)',
                  style: theme.textTheme.bodySmall,
                ),
              ],
            ),
            const SizedBox(height: 16),
            Row(
              children: [
                Expanded(
                  child: _buildMetricTile(
                    theme,
                    'Estimated Miss Rate',
                    '${currentMissRate.toStringAsFixed(1)}%',
                    currentMissRate < 5
                        ? Colors.green
                        : currentMissRate < 15
                            ? Colors.orange
                            : Colors.red,
                  ),
                ),
                const SizedBox(width: 16),
                Expanded(
                  child: _buildMetricTile(
                    theme,
                    'Category',
                    currentPreset?.name ?? 'Custom',
                    currentPreset?.color ?? theme.colorScheme.secondary,
                  ),
                ),
              ],
            ),
          ],
        ),
      ),
    );
  }

  Widget _buildMetricTile(
    ThemeData theme,
    String label,
    String value,
    Color color,
  ) {
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

  _PresetRegion? _getPresetForTimeout(double timeout) {
    for (final preset in _presets) {
      if (timeout >= preset.minMs && timeout <= preset.maxMs) {
        return preset;
      }
    }
    return null;
  }

  Widget _buildPresetsCard(ThemeData theme) {
    return Card(
      child: Padding(
        padding: const EdgeInsets.all(16),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Text(
              'Preset Configurations',
              style: theme.textTheme.titleMedium,
            ),
            const SizedBox(height: 12),
            ..._presets.map((preset) => _buildPresetRow(theme, preset)),
          ],
        ),
      ),
    );
  }

  Widget _buildPresetRow(ThemeData theme, _PresetRegion preset) {
    final isSelected = _currentTimeout >= preset.minMs &&
        _currentTimeout <= preset.maxMs;

    return Padding(
      padding: const EdgeInsets.only(bottom: 8),
      child: InkWell(
        onTap: () {
          setState(() {
            // Set to middle of preset range
            _currentTimeout = (preset.minMs + preset.maxMs) / 2;
          });
        },
        borderRadius: BorderRadius.circular(8),
        child: Container(
          padding: const EdgeInsets.all(12),
          decoration: BoxDecoration(
            color: isSelected
                ? preset.color.withValues(alpha: 0.15)
                : theme.colorScheme.surface,
            borderRadius: BorderRadius.circular(8),
            border: Border.all(
              color: isSelected
                  ? preset.color
                  : theme.dividerColor,
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
                        fontWeight:
                            isSelected ? FontWeight.bold : FontWeight.normal,
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

  Widget _buildStatisticsCard(ThemeData theme) {
    return Card(
      child: Padding(
        padding: const EdgeInsets.all(16),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Row(
              mainAxisAlignment: MainAxisAlignment.spaceBetween,
              children: [
                Text(
                  'Typing Model Parameters',
                  style: theme.textTheme.titleMedium,
                ),
                Row(
                  mainAxisSize: MainAxisSize.min,
                  children: [
                    IconButton(
                      icon: const Icon(Icons.edit, size: 20),
                      onPressed: _showModelEditDialog,
                      tooltip: 'Edit parameters',
                    ),
                  ],
                ),
              ],
            ),
            const SizedBox(height: 12),
            Row(
              children: [
                Expanded(
                  child: _buildStatTile(
                    theme,
                    'Mean Key Duration',
                    '${_typingMean.toInt()} ms',
                    Icons.timer,
                  ),
                ),
                const SizedBox(width: 16),
                Expanded(
                  child: _buildStatTile(
                    theme,
                    'Std Deviation',
                    '${_typingStdDev.toInt()} ms',
                    Icons.show_chart,
                  ),
                ),
              ],
            ),
            const SizedBox(height: 16),
            // Simulate button
            SizedBox(
              width: double.infinity,
              child: FilledButton.icon(
                onPressed: _startSimulation,
                icon: const Icon(Icons.speed),
                label: const Text('Simulate My Typing Speed'),
              ),
            ),
            if (_hasUserProfile && _recommendedTimeout != null) ...[
              const SizedBox(height: 12),
              _buildRecommendationBanner(theme),
            ],
            const SizedBox(height: 12),
            Text(
              _hasUserProfile
                  ? 'Profile based on your measured typing speed. '
                    'The purple band on the chart shows your typing range.'
                  : 'Measure your actual typing speed to get personalized '
                    'recommendations and see your profile on the chart.',
              style: theme.textTheme.bodySmall?.copyWith(
                color: theme.colorScheme.onSurface.withValues(alpha: 0.6),
              ),
            ),
          ],
        ),
      ),
    );
  }

  Widget _buildRecommendationBanner(ThemeData theme) {
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
                  'Recommended: ${_recommendedTimeout!.toInt()} ms',
                  style: theme.textTheme.titleSmall?.copyWith(
                    color: Colors.purple,
                    fontWeight: FontWeight.bold,
                  ),
                ),
                Text(
                  'Based on your typing profile (mean: ${_userTypingMean!.toInt()}ms)',
                  style: theme.textTheme.bodySmall,
                ),
              ],
            ),
          ),
          TextButton(
            onPressed: () {
              setState(() {
                _currentTimeout = _recommendedTimeout!;
              });
            },
            child: const Text('Apply'),
          ),
        ],
      ),
    );
  }

  Widget _buildStatTile(
    ThemeData theme,
    String label,
    String value,
    IconData icon,
  ) {
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
              Text(
                label,
                style: theme.textTheme.bodySmall,
              ),
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

  void _showModelEditDialog() {
    double tempMean = _typingMean;
    double tempStdDev = _typingStdDev;

    showDialog(
      context: context,
      builder: (context) => AlertDialog(
        title: const Text('Edit Typing Model'),
        content: StatefulBuilder(
          builder: (context, setDialogState) => Column(
            mainAxisSize: MainAxisSize.min,
            children: [
              Text('Mean Key Duration: ${tempMean.toInt()} ms'),
              Slider(
                value: tempMean,
                min: 100,
                max: 400,
                divisions: 30,
                onChanged: (v) => setDialogState(() => tempMean = v),
              ),
              const SizedBox(height: 16),
              Text('Standard Deviation: ${tempStdDev.toInt()} ms'),
              Slider(
                value: tempStdDev,
                min: 20,
                max: 100,
                divisions: 16,
                onChanged: (v) => setDialogState(() => tempStdDev = v),
              ),
            ],
          ),
        ),
        actions: [
          TextButton(
            onPressed: () => Navigator.pop(context),
            child: const Text('Cancel'),
          ),
          FilledButton(
            onPressed: () {
              setState(() {
                _typingMean = tempMean;
                _typingStdDev = tempStdDev;
              });
              Navigator.pop(context);
            },
            child: const Text('Apply'),
          ),
        ],
      ),
    );
  }

  void _startSimulation() {
    showDialog(
      context: context,
      barrierDismissible: false,
      builder: (context) => TypingSimulationDialog(
        sampleText: _sampleText,
        onComplete: (mean, stdDev) {
          setState(() {
            _userTypingMean = mean;
            _userTypingStdDev = stdDev;
            _typingMean = mean;
            _typingStdDev = stdDev;
            _hasUserProfile = true;
            // Recommend threshold at mean + 1 stddev for ~84% accuracy
            _recommendedTimeout = (mean + stdDev).clamp(_minTimeout, _maxTimeout);
          });
        },
        onCancel: () {},
      ),
    );
  }

  void _showHelpDialog() {
    showDialog(
      context: context,
      builder: (context) => AlertDialog(
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
      ),
    );
  }
}

/// A preset timing region with display properties.
class _PresetRegion {
  const _PresetRegion({
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
}

