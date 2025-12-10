// Debugger meters and pending decision widgets.
//
// Contains latency meter and pending tap-hold/combo visualization components.

import 'dart:async';
import 'dart:math' as math;

import 'package:flutter/material.dart';

import '../config/config.dart';
import '../services/engine_service.dart';

/// Constants for latency thresholds.
///
/// Uses centralized values from [ThresholdConstants] and [TimingConfig].
class LatencyThresholds {
  static const int warningUs = ThresholdConstants.latencyWarningUs;
  static const int cautionUs = ThresholdConstants.latencyCautionUs;
  static const Duration animationDuration = Duration(
    milliseconds: TimingConfig.animationDurationMs,
  );
}

/// Builds the latency card with animated meter.
class LatencyMeterCard extends StatelessWidget {
  const LatencyMeterCard({
    super.key,
    required this.latencyUs,
    required this.previousLatency,
    required this.animationDuration,
  });

  final int? latencyUs;
  final int? previousLatency;
  final Duration animationDuration;

  @override
  Widget build(BuildContext context) {
    if (latencyUs == null) {
      return Card(
        child: Padding(
          padding: const EdgeInsets.all(12),
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              Row(
                children: [
                  const Icon(Icons.speed, color: Colors.grey),
                  const SizedBox(width: 8),
                  Text(
                    'Latency',
                    style: Theme.of(context).textTheme.titleMedium,
                  ),
                ],
              ),
              const SizedBox(height: 8),
              Text(
                'Waiting for samples...',
                style: TextStyle(
                  color: Theme.of(context).textTheme.bodySmall?.color,
                  fontStyle: FontStyle.italic,
                ),
              ),
            ],
          ),
        ),
      );
    }

    final color = latencyColor(latencyUs);
    final label = latencyUs! >= LatencyThresholds.warningUs
        ? 'High'
        : latencyUs! >= LatencyThresholds.cautionUs
        ? 'Caution'
        : 'Healthy';

    // Calculate meter progress (0-1, capped at warning threshold * 2)
    final maxLatency = LatencyThresholds.warningUs * 2;
    final progress = (latencyUs! / maxLatency).clamp(0.0, 1.0);

    // Detect latency change for animation
    final latencyChanged =
        previousLatency != null && latencyUs != previousLatency;
    final latencyIncreased =
        previousLatency != null && latencyUs! > previousLatency!;

    return Card(
      child: Padding(
        padding: const EdgeInsets.all(12),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Row(
              children: [
                AnimatedContainer(
                  duration: animationDuration,
                  child: Icon(Icons.speed, color: color),
                ),
                const SizedBox(width: 8),
                Expanded(
                  child: Text(
                    'Latency',
                    style: Theme.of(context).textTheme.titleMedium,
                    overflow: TextOverflow.ellipsis,
                  ),
                ),
                const SizedBox(width: 8),
                AnimatedContainer(
                  duration: animationDuration,
                  padding: const EdgeInsets.symmetric(
                    horizontal: 10,
                    vertical: 4,
                  ),
                  decoration: BoxDecoration(
                    color: color.withValues(alpha: 0.15),
                    borderRadius: BorderRadius.circular(12),
                    border: Border.all(
                      color: color.withValues(alpha: 0.3),
                      width: 1,
                    ),
                  ),
                  child: Row(
                    mainAxisSize: MainAxisSize.min,
                    children: [
                      if (latencyChanged)
                        AnimatedOpacity(
                          opacity: 1.0,
                          duration: animationDuration,
                          child: Icon(
                            latencyIncreased
                                ? Icons.arrow_upward
                                : Icons.arrow_downward,
                            size: 14,
                            color: latencyIncreased ? Colors.red : Colors.green,
                          ),
                        ),
                      Text(
                        label,
                        style: TextStyle(
                          color: color,
                          fontWeight: FontWeight.bold,
                          fontSize: 12,
                        ),
                      ),
                    ],
                  ),
                ),
              ],
            ),
            const SizedBox(height: 12),
            // Animated latency meter
            ClipRRect(
              borderRadius: BorderRadius.circular(4),
              child: AnimatedContainer(
                duration: animationDuration,
                height: 8,
                child: LinearProgressIndicator(
                  value: progress,
                  backgroundColor: Colors.grey.withValues(alpha: 0.2),
                  valueColor: AlwaysStoppedAnimation<Color>(color),
                ),
              ),
            ),
            const SizedBox(height: 8),
            Row(
              children: [
                Expanded(
                  child: AnimatedDefaultTextStyle(
                    duration: animationDuration,
                    style: TextStyle(
                      fontSize: 24,
                      fontWeight: FontWeight.bold,
                      color: color,
                    ),
                    child: Text('$latencyUs'),
                  ),
                ),
                Text(
                  'µs per event',
                  style: Theme.of(context).textTheme.bodySmall,
                ),
              ],
            ),
            // Threshold markers
            const SizedBox(height: 4),
            Wrap(
              alignment: WrapAlignment.spaceBetween,
              spacing: 8,
              runSpacing: 4,
              children: [
                _buildThresholdMarker('0', Colors.green),
                _buildThresholdMarker(
                  '${LatencyThresholds.cautionUs ~/ 1000}k',
                  Colors.orange,
                ),
                _buildThresholdMarker(
                  '${LatencyThresholds.warningUs ~/ 1000}k',
                  Colors.red,
                ),
              ],
            ),
          ],
        ),
      ),
    );
  }

  Widget _buildThresholdMarker(String label, Color color) {
    return Row(
      mainAxisSize: MainAxisSize.min,
      children: [
        Container(
          width: 8,
          height: 8,
          decoration: BoxDecoration(
            color: color.withValues(alpha: 0.3),
            shape: BoxShape.circle,
            border: Border.all(color: color, width: 1),
          ),
        ),
        const SizedBox(width: 4),
        Text(label, style: TextStyle(fontSize: 10, color: color)),
      ],
    );
  }

  /// Returns the color based on latency value.
  static Color latencyColor(int? latencyUs) {
    if (latencyUs == null) return Colors.grey;
    if (latencyUs >= LatencyThresholds.warningUs) return Colors.redAccent;
    if (latencyUs >= LatencyThresholds.cautionUs) return Colors.orangeAccent;
    return Colors.green;
  }
}

/// Pending tap-hold decision with countdown timer.
class PendingTapHoldWidget extends StatefulWidget {
  const PendingTapHoldWidget({
    super.key,
    required this.decision,
    this.timing,
    required this.pulse,
  });

  final String decision;
  final EngineTiming? timing;
  final Animation<double> pulse;

  @override
  State<PendingTapHoldWidget> createState() => _PendingTapHoldWidgetState();
}

class _PendingTapHoldWidgetState extends State<PendingTapHoldWidget> {
  Timer? _timer;
  double _progress = 1.0;

  @override
  void initState() {
    super.initState();
    final ms = widget.timing?.tapTimeoutMs ?? 200;
    final ticks = ms ~/ 16;
    var tick = 0;
    _timer = Timer.periodic(const Duration(milliseconds: 16), (t) {
      tick++;
      if (tick >= ticks) {
        t.cancel();
        if (mounted) setState(() => _progress = 0);
      } else if (mounted) {
        setState(() => _progress = 1 - tick / ticks);
      }
    });
  }

  @override
  void dispose() {
    _timer?.cancel();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final ms = widget.timing?.tapTimeoutMs ?? 200;
    return Padding(
      padding: const EdgeInsets.symmetric(vertical: 4),
      child: Row(
        children: [
          SizedBox(
            width: 32,
            height: 32,
            child: Stack(
              alignment: Alignment.center,
              children: [
                CircularProgressIndicator(
                  value: _progress,
                  strokeWidth: 3,
                  backgroundColor: Colors.grey.withValues(alpha: 0.2),
                  valueColor: AlwaysStoppedAnimation(
                    _progress > 0.3 ? Colors.orange : Colors.red,
                  ),
                ),
                Text(
                  '${(_progress * ms).round()}',
                  style: const TextStyle(
                    fontSize: 9,
                    fontWeight: FontWeight.bold,
                  ),
                ),
              ],
            ),
          ),
          const SizedBox(width: 8),
          Expanded(
            child: Container(
              padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 4),
              decoration: BoxDecoration(
                color: Colors.orange.withValues(alpha: 0.1),
                borderRadius: BorderRadius.circular(6),
                border: Border.all(color: Colors.orange.withValues(alpha: 0.4)),
              ),
              child: Row(
                children: [
                  const Icon(Icons.touch_app, size: 14, color: Colors.orange),
                  const SizedBox(width: 4),
                  Expanded(
                    child: Text(
                      widget.decision,
                      style: const TextStyle(fontSize: 12),
                      overflow: TextOverflow.ellipsis,
                    ),
                  ),
                ],
              ),
            ),
          ),
        ],
      ),
    );
  }
}

/// Pending combo with pulsing key highlights.
class PendingComboWidget extends StatelessWidget {
  const PendingComboWidget({
    super.key,
    required this.decision,
    required this.pulse,
  });

  final String decision;
  final Animation<double> pulse;

  @override
  Widget build(BuildContext context) {
    final keys = _extractKeys(decision);
    return Padding(
      padding: const EdgeInsets.symmetric(vertical: 4),
      child: Row(
        children: [
          const Icon(Icons.keyboard, size: 20, color: Colors.blue),
          const SizedBox(width: 8),
          Expanded(
            child: Wrap(
              spacing: 4,
              runSpacing: 4,
              children: [for (final key in keys) _buildKeyChip(key)],
            ),
          ),
        ],
      ),
    );
  }

  Widget _buildKeyChip(String key) {
    final w = 1.0 + (pulse.value - 1.0) * 3.0;
    return AnimatedContainer(
      duration: const Duration(milliseconds: 150),
      padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 4),
      decoration: BoxDecoration(
        color: Colors.blue.withValues(alpha: 0.15),
        borderRadius: BorderRadius.circular(6),
        border: Border.all(color: Colors.blue, width: math.max(1.0, w)),
        boxShadow: [
          BoxShadow(
            color: Colors.blue.withValues(alpha: (pulse.value - 1) * 3),
            blurRadius: (pulse.value - 1) * 40,
            spreadRadius: (pulse.value - 1) * 10,
          ),
        ],
      ),
      child: Text(
        key,
        style: const TextStyle(
          fontSize: 12,
          fontWeight: FontWeight.bold,
          color: Colors.blue,
        ),
      ),
    );
  }

  List<String> _extractKeys(String s) {
    // Try bracket notation: [A, B, C]
    final m = RegExp(r'\[([^\]]+)\]').firstMatch(s);
    if (m != null) {
      return m
          .group(1)!
          .split(RegExp(r'[,\s]+'))
          .where((x) => x.isNotEmpty)
          .toList();
    }
    // Try plus notation: A+B or "combo A+B"
    if (s.contains('+')) {
      final keys = <String>[];
      for (final part in s.split('+')) {
        // Get the last word (key name) from each part
        final words = part.trim().split(RegExp(r'\s+'));
        if (words.isNotEmpty) keys.add(words.last);
      }
      return keys.where((k) => k.isNotEmpty).toList();
    }
    return [s];
  }
}
