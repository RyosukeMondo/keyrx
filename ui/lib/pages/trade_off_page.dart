// Trade-off visualizer page for timing configuration.
//
// Main page that shows the relationship between tap-hold timeout
// and estimated miss rate, helping users find optimal timing settings.

import 'package:flutter/material.dart';
import 'package:provider/provider.dart';

import '../services/engine_service.dart';
import '../services/service_registry.dart';
import 'trade_off_chart.dart';
import 'trade_off_widgets.dart';
import 'typing_simulator.dart';

/// Trade-off visualizer page for timing configuration.
class TradeOffVisualizerPage extends StatefulWidget {
  const TradeOffVisualizerPage({super.key});

  @override
  State<TradeOffVisualizerPage> createState() => _TradeOffVisualizerPageState();
}

class _TradeOffVisualizerPageState extends State<TradeOffVisualizerPage> {
  static const double _minTimeout = 100;
  static const double _maxTimeout = 1000;

  double _typingMean = 180.0;
  double _typingStdDev = 50.0;
  double _currentTimeout = 200.0;

  bool _hasUserProfile = false;
  double? _userTypingMean;
  double? _userTypingStdDev;
  double? _recommendedTimeout;

  static const String _sampleText =
      'The quick brown fox jumps over the lazy dog. '
      'Pack my box with five dozen liquor jugs.';

  EngineService? _engine;

  TypingStatistics get _statistics => TypingStatistics(
        mean: _typingMean,
        stdDev: _typingStdDev,
      );

  @override
  void initState() {
    super.initState();
    final registry = Provider.of<ServiceRegistry>(context, listen: false);
    _engine = registry.engineService;
    _loadCurrentTiming();
  }

  void _loadCurrentTiming() {
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
            onPressed: () => showDialog(
              context: context,
              builder: (_) => const TradeOffHelpDialog(),
            ),
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
                Text('Understanding Tap-Hold Timing',
                    style: theme.textTheme.titleMedium),
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
            Text('Miss Rate vs. Timeout', style: theme.textTheme.titleMedium),
            const SizedBox(height: 8),
            SizedBox(
              height: 300,
              child: TradeOffChart(
                statistics: _statistics,
                currentTimeout: _currentTimeout,
                userTypingMean: _userTypingMean,
                userTypingStdDev: _userTypingStdDev,
              ),
            ),
          ],
        ),
      ),
    );
  }

  Widget _buildSliderCard(ThemeData theme) {
    final currentMissRate = _statistics.calculateMissRate(_currentTimeout);
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
                Text('Adjust Timeout', style: theme.textTheme.titleMedium),
                Container(
                  padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 6),
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
                activeTrackColor: currentPreset?.color ?? theme.colorScheme.primary,
                thumbColor: currentPreset?.color ?? theme.colorScheme.primary,
              ),
              child: Slider(
                value: _currentTimeout,
                min: _minTimeout,
                max: _maxTimeout,
                divisions: 90,
                label: '${_currentTimeout.toInt()} ms',
                onChanged: (v) => setState(() => _currentTimeout = v),
              ),
            ),
            const SizedBox(height: 8),
            Row(
              mainAxisAlignment: MainAxisAlignment.spaceBetween,
              children: [
                Text('Faster (${_minTimeout.toInt()}ms)',
                    style: theme.textTheme.bodySmall),
                Text('Slower (${_maxTimeout.toInt()}ms)',
                    style: theme.textTheme.bodySmall),
              ],
            ),
            const SizedBox(height: 16),
            Row(
              children: [
                Expanded(
                  child: MetricTile(
                    label: 'Estimated Miss Rate',
                    value: '${currentMissRate.toStringAsFixed(1)}%',
                    color: currentMissRate < 5
                        ? Colors.green
                        : currentMissRate < 15 ? Colors.orange : Colors.red,
                  ),
                ),
                const SizedBox(width: 16),
                Expanded(
                  child: MetricTile(
                    label: 'Category',
                    value: currentPreset?.name ?? 'Custom',
                    color: currentPreset?.color ?? theme.colorScheme.secondary,
                  ),
                ),
              ],
            ),
          ],
        ),
      ),
    );
  }

  PresetRegion? _getPresetForTimeout(double timeout) {
    for (final preset in PresetRegion.defaults) {
      if (preset.containsTimeout(timeout)) return preset;
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
            Text('Preset Configurations', style: theme.textTheme.titleMedium),
            const SizedBox(height: 12),
            ...PresetRegion.defaults.map((preset) => PresetRow(
                  preset: preset,
                  isSelected: preset.containsTimeout(_currentTimeout),
                  onTap: () => setState(() => _currentTimeout = preset.middleValue),
                )),
          ],
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
                Text('Typing Model Parameters', style: theme.textTheme.titleMedium),
                IconButton(
                  icon: const Icon(Icons.edit, size: 20),
                  onPressed: _showModelEditDialog,
                  tooltip: 'Edit parameters',
                ),
              ],
            ),
            const SizedBox(height: 12),
            Row(
              children: [
                Expanded(
                  child: StatTile(
                    label: 'Mean Key Duration',
                    value: '${_typingMean.toInt()} ms',
                    icon: Icons.timer,
                  ),
                ),
                const SizedBox(width: 16),
                Expanded(
                  child: StatTile(
                    label: 'Std Deviation',
                    value: '${_typingStdDev.toInt()} ms',
                    icon: Icons.show_chart,
                  ),
                ),
              ],
            ),
            const SizedBox(height: 16),
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
              RecommendationBanner(
                recommendedTimeout: _recommendedTimeout!,
                userTypingMean: _userTypingMean!,
                onApply: () => setState(() => _currentTimeout = _recommendedTimeout!),
              ),
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

  void _showModelEditDialog() {
    showDialog(
      context: context,
      builder: (_) => ModelEditDialog(
        initialMean: _typingMean,
        initialStdDev: _typingStdDev,
        onApply: (mean, stdDev) {
          setState(() {
            _typingMean = mean;
            _typingStdDev = stdDev;
          });
        },
      ),
    );
  }

  void _startSimulation() {
    showDialog(
      context: context,
      barrierDismissible: false,
      builder: (_) => TypingSimulationDialog(
        sampleText: _sampleText,
        onComplete: (mean, stdDev) {
          setState(() {
            _userTypingMean = mean;
            _userTypingStdDev = stdDev;
            _typingMean = mean;
            _typingStdDev = stdDev;
            _hasUserProfile = true;
            _recommendedTimeout = (mean + stdDev).clamp(_minTimeout, _maxTimeout);
          });
        },
        onCancel: () {},
      ),
    );
  }
}
