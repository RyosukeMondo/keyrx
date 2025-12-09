import 'dart:async';
import 'dart:math';

import 'package:flutter/material.dart';

/// Interactive calibration page for measuring latency and generating tuned
/// timing profiles. This is UI-only scaffolding that simulates calibration
/// runs until the engine exposes real FFI hooks.
class CalibrationPage extends StatefulWidget {
  const CalibrationPage({super.key});

  @override
  State<CalibrationPage> createState() => _CalibrationPageState();
}

class _CalibrationPageState extends State<CalibrationPage> {
  final List<_CalibrationDevice> _devices = const [
    _CalibrationDevice(
      name: 'Input Club Kira',
      vendorId: '0x1C11',
      productId: '0xB04D',
      deviceClass: 'Mechanical',
      profile: 'Kira Default',
      timing: TimingConfig(
        debounceMs: 6,
        repeatDelayMs: 210,
        repeatRateMs: 26,
        scanIntervalUs: 780,
      ),
    ),
    _CalibrationDevice(
      name: 'ThinkPad T-series',
      vendorId: '0x17EF',
      productId: '0x606E',
      deviceClass: 'Laptop',
      profile: 'Quiet Office',
      timing: TimingConfig(
        debounceMs: 8,
        repeatDelayMs: 240,
        repeatRateMs: 32,
        scanIntervalUs: 1200,
      ),
    ),
  ];

  final List<double> _baselineLatencyMs = const [
    13.4,
    12.8,
    12.6,
    13.2,
    12.5,
    12.3,
    12.9,
    12.4,
    12.1,
    12.7,
    12.5,
    12.3,
  ];

  _CalibrationDevice? _selectedDevice;
  bool _isCalibrating = false;
  bool _applyPending = false;
  double _progress = 0;
  int _requestedSamples = 14;
  String _status = 'Idle';
  Timer? _timer;
  List<double> _latencySamples = [];
  CalibrationResult? _result;

  @override
  void initState() {
    super.initState();
    _selectedDevice = _devices.first;
  }

  @override
  void dispose() {
    _timer?.cancel();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final spacing = Theme.of(context).textTheme;
    final device = _selectedDevice!;
    final comparison = _result != null
        ? compareTimings(device.timing, _result!.proposedTiming)
        : null;

    return Scaffold(
      appBar: AppBar(
        title: const Text('Calibration'),
        actions: [
          TextButton.icon(
            onPressed: _isCalibrating ? null : _startCalibration,
            icon: const Icon(Icons.play_arrow),
            label: const Text('Start'),
          ),
          const SizedBox(width: 8),
        ],
      ),
      body: SingleChildScrollView(
        padding: const EdgeInsets.all(16),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Wrap(
              spacing: 12,
              runSpacing: 12,
              children: [_buildDeviceCard(device), _buildProfileCard(device)],
            ),
            const SizedBox(height: 12),
            _buildCalibrationCard(spacing),
            const SizedBox(height: 12),
            if (_result != null) ...[
              _buildComparisonCard(comparison!),
              const SizedBox(height: 12),
              _buildLatencyTrace(),
            ],
          ],
        ),
      ),
    );
  }

  Widget _buildDeviceCard(_CalibrationDevice device) {
    return SizedBox(
      width: 360,
      child: Card(
        child: Padding(
          padding: const EdgeInsets.all(16),
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              Text('Device', style: Theme.of(context).textTheme.titleMedium),
              const SizedBox(height: 12),
              DropdownButton<_CalibrationDevice>(
                value: _selectedDevice,
                isExpanded: true,
                onChanged: _isCalibrating
                    ? null
                    : (value) {
                        if (value == null) return;
                        setState(() {
                          _selectedDevice = value;
                          _result = null;
                          _applyPending = false;
                        });
                      },
                items: _devices
                    .map(
                      (d) => DropdownMenuItem(
                        value: d,
                        child: Text(
                          '${d.name} • ${d.deviceClass}',
                          overflow: TextOverflow.ellipsis,
                        ),
                      ),
                    )
                    .toList(),
              ),
              const SizedBox(height: 12),
              Wrap(
                spacing: 8,
                runSpacing: 8,
                children: [
                  Chip(
                    avatar: const Icon(Icons.developer_board, size: 16),
                    label: Text('VID ${device.vendorId}'),
                  ),
                  Chip(
                    avatar: const Icon(Icons.memory, size: 16),
                    label: Text('PID ${device.productId}'),
                  ),
                  Chip(
                    avatar: const Icon(Icons.category, size: 16),
                    label: Text(device.deviceClass),
                  ),
                ],
              ),
            ],
          ),
        ),
      ),
    );
  }

  Widget _buildProfileCard(_CalibrationDevice device) {
    return SizedBox(
      width: 360,
      child: Card(
        child: Padding(
          padding: const EdgeInsets.all(16),
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              Row(
                children: [
                  Text(
                    'Current Profile',
                    style: Theme.of(context).textTheme.titleMedium,
                  ),
                  const Spacer(),
                  FilledButton.tonalIcon(
                    onPressed: _result == null || _applyPending
                        ? null
                        : _applyRecommendation,
                    icon: const Icon(Icons.published_with_changes),
                    label: Text(_applyPending ? 'Applied' : 'Apply tuned'),
                  ),
                ],
              ),
              const SizedBox(height: 12),
              Text(
                device.profile,
                style: Theme.of(context).textTheme.bodyLarge,
              ),
              const SizedBox(height: 8),
              Text(
                'Baseline timing tuned for ${device.deviceClass.toLowerCase()} switches.',
                style: TextStyle(color: Colors.grey[400]),
              ),
              const SizedBox(height: 12),
              _TimingGrid(
                before: device.timing,
                after: _result?.proposedTiming,
              ),
            ],
          ),
        ),
      ),
    );
  }

  Widget _buildCalibrationCard(TextTheme spacing) {
    return Card(
      child: Padding(
        padding: const EdgeInsets.all(16),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Row(
              children: [
                Text('Interactive Calibration', style: spacing.titleMedium),
                const Spacer(),
                Chip(
                  avatar: Icon(
                    _isCalibrating ? Icons.graphic_eq : Icons.pause_circle,
                    size: 16,
                  ),
                  label: Text(_status),
                ),
              ],
            ),
            const SizedBox(height: 12),
            Row(
              children: [
                Expanded(
                  child: Column(
                    crossAxisAlignment: CrossAxisAlignment.start,
                    children: [
                      Text('Sample batches: $_requestedSamples'),
                      Slider(
                        value: _requestedSamples.toDouble(),
                        min: 10,
                        max: 28,
                        divisions: 9,
                        onChanged: _isCalibrating
                            ? null
                            : (value) => setState(
                                () => _requestedSamples = value.round(),
                              ),
                      ),
                    ],
                  ),
                ),
                const SizedBox(width: 12),
                FilledButton.icon(
                  onPressed: _isCalibrating ? null : _startCalibration,
                  icon: const Icon(Icons.speed),
                  label: const Text('Run calibration'),
                ),
              ],
            ),
            const SizedBox(height: 12),
            LinearProgressIndicator(
              value: _isCalibrating ? _progress.clamp(0, 1) : 0,
              minHeight: 8,
              backgroundColor: Theme.of(
                context,
              ).colorScheme.surfaceContainerHighest,
            ),
            const SizedBox(height: 16),
            if (_result != null) _buildResultSummary(_result!, spacing),
          ],
        ),
      ),
    );
  }

  Widget _buildResultSummary(CalibrationResult result, TextTheme spacing) {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Text('Results', style: spacing.titleMedium),
        const SizedBox(height: 8),
        Wrap(
          spacing: 12,
          runSpacing: 12,
          children: [
            _MetricTile(
              label: 'Avg latency',
              value: '${result.averageLatencyMs.toStringAsFixed(2)} ms',
              icon: Icons.bolt,
            ),
            _MetricTile(
              label: 'Jitter (p95)',
              value: '${result.jitterMs.toStringAsFixed(2)} ms',
              icon: Icons.stacked_line_chart,
            ),
            _MetricTile(
              label: 'Confidence',
              value: '${(result.confidence * 100).round()}%',
              icon: Icons.shield_moon,
            ),
          ],
        ),
      ],
    );
  }

  Widget _buildComparisonCard(CalibrationComparison comparison) {
    return Card(
      child: Padding(
        padding: const EdgeInsets.all(16),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Text(
              'Before / After',
              style: Theme.of(context).textTheme.titleMedium,
            ),
            const SizedBox(height: 12),
            Wrap(
              spacing: 12,
              runSpacing: 12,
              children: [
                _DeltaTile(
                  label: 'Debounce',
                  before: '${comparison.before.debounceMs} ms',
                  after: '${comparison.after.debounceMs} ms',
                  delta: comparison.debounceDelta,
                ),
                _DeltaTile(
                  label: 'Repeat delay',
                  before: '${comparison.before.repeatDelayMs} ms',
                  after: '${comparison.after.repeatDelayMs} ms',
                  delta: comparison.repeatDelayDelta,
                ),
                _DeltaTile(
                  label: 'Repeat rate',
                  before: '${comparison.before.repeatRateMs} ms',
                  after: '${comparison.after.repeatRateMs} ms',
                  delta: comparison.repeatRateDelta,
                ),
                _DeltaTile(
                  label: 'Scan interval',
                  before: '${comparison.before.scanIntervalUs} µs',
                  after: '${comparison.after.scanIntervalUs} µs',
                  delta: comparison.scanIntervalDelta,
                  suffix: 'µs',
                ),
              ],
            ),
          ],
        ),
      ),
    );
  }

  Widget _buildLatencyTrace() {
    final samples = _result?.samples ?? _latencySamples;
    if (samples.isEmpty) {
      return const SizedBox.shrink();
    }

    final maxValue = samples.reduce(max);
    return Card(
      child: Padding(
        padding: const EdgeInsets.all(16),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Text(
              'Latency samples',
              style: Theme.of(context).textTheme.titleMedium,
            ),
            const SizedBox(height: 12),
            SizedBox(
              height: 160,
              child: Row(
                crossAxisAlignment: CrossAxisAlignment.end,
                children: samples
                    .map(
                      (value) => Expanded(
                        child: Padding(
                          padding: const EdgeInsets.symmetric(horizontal: 3),
                          child: AnimatedContainer(
                            duration: const Duration(milliseconds: 200),
                            height: (value / maxValue) * 140 + 8,
                            decoration: BoxDecoration(
                              borderRadius: BorderRadius.circular(6),
                              gradient: LinearGradient(
                                begin: Alignment.bottomCenter,
                                end: Alignment.topCenter,
                                colors: [
                                  Theme.of(context).colorScheme.primary,
                                  Theme.of(
                                    context,
                                  ).colorScheme.primaryContainer,
                                ],
                              ),
                            ),
                          ),
                        ),
                      ),
                    )
                    .toList(),
              ),
            ),
            const SizedBox(height: 8),
            Text(
              'Shorter bars mean lower latency. Bars update on every calibration run.',
              style: TextStyle(color: Colors.grey[400]),
            ),
          ],
        ),
      ),
    );
  }

  void _startCalibration() {
    _timer?.cancel();
    final random = Random(7);
    setState(() {
      _isCalibrating = true;
      _applyPending = false;
      _progress = 0;
      _status = 'Collecting samples…';
      _latencySamples = [];
      _result = null;
    });

    final totalTicks = _requestedSamples;
    int tick = 0;

    _timer = Timer.periodic(const Duration(milliseconds: 220), (timer) {
      tick += 1;
      final sample = max<double>(
        8.6,
        _baselineLatencyMs[tick % _baselineLatencyMs.length] -
            2.1 +
            (random.nextDouble() * 0.8 - 0.35),
      );
      _latencySamples.add(double.parse(sample.toStringAsFixed(2)));

      setState(() {
        _progress = tick / totalTicks;
        _status = tick >= totalTicks
            ? 'Finalizing results…'
            : 'Collected ${_latencySamples.length}/$totalTicks samples';
      });

      if (tick >= totalTicks) {
        timer.cancel();
        _finalizeResults();
      }
    });
  }

  void _finalizeResults() {
    final avg =
        _latencySamples.reduce((a, b) => a + b) / _latencySamples.length;
    final jitter = _computeJitter(_latencySamples);
    final proposed = _deriveTiming(avg, jitter);

    setState(() {
      _isCalibrating = false;
      _status = 'Complete';
      _result = CalibrationResult(
        averageLatencyMs: avg,
        jitterMs: jitter,
        confidence: _confidenceFromJitter(jitter),
        proposedTiming: proposed,
        samples: _latencySamples,
      );
    });
  }

  void _applyRecommendation() {
    if (_result == null) return;
    setState(() {
      _applyPending = true;
    });
    ScaffoldMessenger.of(context).showSnackBar(
      SnackBar(
        content: Text(
          'Applied tuned timings to ${_selectedDevice!.name} (${_result!.proposedTiming.debounceMs}ms debounce, ${_result!.proposedTiming.scanIntervalUs}µs scan)',
        ),
      ),
    );
  }

  double _computeJitter(List<double> samples) {
    if (samples.isEmpty) return 0;
    final average = samples.reduce((a, b) => a + b) / samples.length;
    final squared =
        samples.map((s) => pow(s - average, 2)).reduce((a, b) => a + b) /
        samples.length;
    return sqrt(squared);
  }

  double _confidenceFromJitter(double jitter) {
    if (jitter <= 0.3) return 0.96;
    if (jitter <= 0.6) return 0.88;
    return 0.76;
  }

  TimingConfig _deriveTiming(double averageLatency, double jitter) {
    final device = _selectedDevice!;
    final jitterPenalty = (jitter * 10).round();
    final scanInterval = max(
      560,
      device.timing.scanIntervalUs - jitterPenalty * 2,
    );
    final repeatDelay = max(180, device.timing.repeatDelayMs - 10);
    final repeatRate = max(18, device.timing.repeatRateMs - 4);

    return TimingConfig(
      debounceMs: max(4, device.timing.debounceMs - 1),
      repeatDelayMs: repeatDelay,
      repeatRateMs: repeatRate,
      scanIntervalUs: scanInterval,
      notes:
          'Derived from ${_latencySamples.length} samples @ ${averageLatency.toStringAsFixed(2)}ms avg',
    );
  }

  CalibrationComparison compareTimings(
    TimingConfig before,
    TimingConfig after,
  ) {
    return CalibrationComparison(
      before: before,
      after: after,
      debounceDelta: after.debounceMs - before.debounceMs,
      repeatDelayDelta: after.repeatDelayMs - before.repeatDelayMs,
      repeatRateDelta: after.repeatRateMs - before.repeatRateMs,
      scanIntervalDelta: after.scanIntervalUs - before.scanIntervalUs,
    );
  }
}

class _TimingGrid extends StatelessWidget {
  const _TimingGrid({required this.before, this.after});

  final TimingConfig before;
  final TimingConfig? after;

  @override
  Widget build(BuildContext context) {
    return Table(
      columnWidths: const {
        0: IntrinsicColumnWidth(),
        1: FlexColumnWidth(),
        2: IntrinsicColumnWidth(),
      },
      children: [
        _row(
          context,
          label: 'Debounce',
          before: '${before.debounceMs} ms',
          after: after != null ? '${after!.debounceMs} ms' : '--',
        ),
        _row(
          context,
          label: 'Repeat delay',
          before: '${before.repeatDelayMs} ms',
          after: after != null ? '${after!.repeatDelayMs} ms' : '--',
        ),
        _row(
          context,
          label: 'Repeat rate',
          before: '${before.repeatRateMs} ms',
          after: after != null ? '${after!.repeatRateMs} ms' : '--',
        ),
        _row(
          context,
          label: 'Scan interval',
          before: '${before.scanIntervalUs} µs',
          after: after != null ? '${after!.scanIntervalUs} µs' : '--',
        ),
      ],
    );
  }

  TableRow _row(
    BuildContext context, {
    required String label,
    required String before,
    required String after,
  }) {
    return TableRow(
      children: [
        Padding(
          padding: const EdgeInsets.symmetric(vertical: 6),
          child: Text(label),
        ),
        Padding(
          padding: const EdgeInsets.symmetric(vertical: 6),
          child: Text(before, style: TextStyle(color: Colors.grey[400])),
        ),
        Padding(
          padding: const EdgeInsets.symmetric(vertical: 6),
          child: Text(after),
        ),
      ],
    );
  }
}

class _MetricTile extends StatelessWidget {
  const _MetricTile({
    required this.label,
    required this.value,
    required this.icon,
  });

  final String label;
  final String value;
  final IconData icon;

  @override
  Widget build(BuildContext context) {
    return Container(
      padding: const EdgeInsets.all(12),
      width: 180,
      decoration: BoxDecoration(
        borderRadius: BorderRadius.circular(12),
        color: Theme.of(context).colorScheme.surfaceContainerHighest,
      ),
      child: Row(
        children: [
          Icon(icon, size: 20),
          const SizedBox(width: 12),
          Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              Text(label, style: TextStyle(color: Colors.grey[400])),
              Text(value, style: Theme.of(context).textTheme.titleMedium),
            ],
          ),
        ],
      ),
    );
  }
}

class _DeltaTile extends StatelessWidget {
  const _DeltaTile({
    required this.label,
    required this.before,
    required this.after,
    required this.delta,
    this.suffix,
  });

  final String label;
  final String before;
  final String after;
  final num delta;
  final String? suffix;

  @override
  Widget build(BuildContext context) {
    final improved = delta < 0;
    final color = improved
        ? Colors.greenAccent.shade200
        : Colors.deepOrangeAccent;

    return Container(
      padding: const EdgeInsets.all(12),
      width: 200,
      decoration: BoxDecoration(
        borderRadius: BorderRadius.circular(12),
        color: Theme.of(context).colorScheme.surfaceContainerHighest,
      ),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Text(label, style: TextStyle(color: Colors.grey[400])),
          const SizedBox(height: 4),
          Row(
            children: [
              Text(before),
              const Padding(
                padding: EdgeInsets.symmetric(horizontal: 6),
                child: Icon(Icons.arrow_forward, size: 16),
              ),
              Text(after),
            ],
          ),
          const SizedBox(height: 8),
          Chip(
            backgroundColor: color.withValues(alpha: 0.12),
            label: Text(
              '${delta.toStringAsFixed(1)}${suffix ?? ' ms'}',
              style: TextStyle(color: color),
            ),
          ),
        ],
      ),
    );
  }
}

class _CalibrationDevice {
  const _CalibrationDevice({
    required this.name,
    required this.vendorId,
    required this.productId,
    required this.deviceClass,
    required this.profile,
    required this.timing,
  });

  final String name;
  final String vendorId;
  final String productId;
  final String deviceClass;
  final String profile;
  final TimingConfig timing;
}

/// Simple timing configuration used for before/after comparisons.
class TimingConfig {
  const TimingConfig({
    required this.debounceMs,
    required this.repeatDelayMs,
    required this.repeatRateMs,
    required this.scanIntervalUs,
    this.notes,
  });

  final int debounceMs;
  final int repeatDelayMs;
  final int repeatRateMs;
  final int scanIntervalUs;
  final String? notes;
}

/// Result of a simulated calibration run.
class CalibrationResult {
  const CalibrationResult({
    required this.averageLatencyMs,
    required this.jitterMs,
    required this.confidence,
    required this.proposedTiming,
    required this.samples,
  });

  final double averageLatencyMs;
  final double jitterMs;
  final double confidence;
  final TimingConfig proposedTiming;
  final List<double> samples;
}

class CalibrationComparison {
  CalibrationComparison({
    required this.before,
    required this.after,
    required this.debounceDelta,
    required this.repeatDelayDelta,
    required this.repeatRateDelta,
    required this.scanIntervalDelta,
  });

  final TimingConfig before;
  final TimingConfig after;
  final num debounceDelta;
  final num repeatDelayDelta;
  final num repeatRateDelta;
  final num scanIntervalDelta;
}

