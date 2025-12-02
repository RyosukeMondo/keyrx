// Typing speed simulation for trade-off visualizer.
//
// Contains the typing speed measurement dialog and statistics calculation.

import 'dart:async';
import 'dart:math' as math;

import 'package:flutter/material.dart';
import 'package:flutter/services.dart';

/// Dialog for typing speed simulation.
class TypingSimulationDialog extends StatefulWidget {
  const TypingSimulationDialog({
    super.key,
    required this.sampleText,
    required this.onComplete,
    required this.onCancel,
  });

  final String sampleText;
  final void Function(double mean, double stdDev) onComplete;
  final VoidCallback onCancel;

  @override
  State<TypingSimulationDialog> createState() => _TypingSimulationDialogState();
}

class _TypingSimulationDialogState extends State<TypingSimulationDialog> {
  final FocusNode _focusNode = FocusNode();
  final List<int> _keyPressTimestamps = [];
  String _typedText = '';
  Timer? _timer;
  int _secondsRemaining = 30;
  bool _isComplete = false;

  @override
  void initState() {
    super.initState();
    _startTimer();
    WidgetsBinding.instance.addPostFrameCallback((_) {
      _focusNode.requestFocus();
    });
  }

  @override
  void dispose() {
    _timer?.cancel();
    _focusNode.dispose();
    super.dispose();
  }

  void _startTimer() {
    _timer = Timer.periodic(const Duration(seconds: 1), (timer) {
      setState(() {
        _secondsRemaining--;
      });
      if (_secondsRemaining <= 0) {
        _finishSimulation();
      }
    });
  }

  void _finishSimulation() {
    _timer?.cancel();
    if (_keyPressTimestamps.length < 10) {
      // Not enough data
      Navigator.pop(context);
      ScaffoldMessenger.of(context).showSnackBar(
        const SnackBar(
          content: Text('Not enough keystrokes recorded. Please try again.'),
        ),
      );
      widget.onCancel();
      return;
    }

    // Calculate inter-key delays
    final delays = <double>[];
    for (int i = 1; i < _keyPressTimestamps.length; i++) {
      final delay =
          (_keyPressTimestamps[i] - _keyPressTimestamps[i - 1]).toDouble();
      // Filter out unreasonable delays (> 2 seconds = likely pause)
      if (delay > 0 && delay < 2000) {
        delays.add(delay);
      }
    }

    if (delays.length < 5) {
      Navigator.pop(context);
      ScaffoldMessenger.of(context).showSnackBar(
        const SnackBar(
          content:
              Text('Not enough valid keystrokes. Please type more continuously.'),
        ),
      );
      widget.onCancel();
      return;
    }

    // Calculate mean and standard deviation
    final mean = delays.reduce((a, b) => a + b) / delays.length;
    final variance = delays
            .map((d) => (d - mean) * (d - mean))
            .reduce((a, b) => a + b) /
        delays.length;
    final stdDev = math.sqrt(variance);

    setState(() {
      _isComplete = true;
    });

    Navigator.pop(context);
    widget.onComplete(mean, stdDev);
  }

  void _handleKeyPress(KeyEvent event) {
    if (event is KeyDownEvent && !_isComplete) {
      final now = DateTime.now().millisecondsSinceEpoch;
      setState(() {
        _keyPressTimestamps.add(now);
        // Track typed characters for progress display
        if (event.character != null && event.character!.isNotEmpty) {
          _typedText += event.character!;
        }
      });
    }
  }

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);
    final progress = _typedText.length / widget.sampleText.length;

    return AlertDialog(
      title: Row(
        children: [
          const Icon(Icons.speed, color: Colors.purple),
          const SizedBox(width: 8),
          const Expanded(child: Text('Typing Speed Simulation')),
          Container(
            padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 4),
            decoration: BoxDecoration(
              color: _secondsRemaining <= 10 ? Colors.red : Colors.grey,
              borderRadius: BorderRadius.circular(4),
            ),
            child: Text(
              '${_secondsRemaining}s',
              style: const TextStyle(
                color: Colors.white,
                fontWeight: FontWeight.bold,
                fontSize: 14,
              ),
            ),
          ),
        ],
      ),
      content: SizedBox(
        width: 400,
        child: Column(
          mainAxisSize: MainAxisSize.min,
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Text(
              'Type the following text as naturally as possible:',
              style: theme.textTheme.bodyMedium,
            ),
            const SizedBox(height: 12),
            Container(
              padding: const EdgeInsets.all(12),
              decoration: BoxDecoration(
                color: theme.colorScheme.surfaceContainerHighest,
                borderRadius: BorderRadius.circular(8),
              ),
              child: Text(
                widget.sampleText,
                style: theme.textTheme.bodyMedium?.copyWith(
                  fontFamily: 'monospace',
                ),
              ),
            ),
            const SizedBox(height: 16),
            LinearProgressIndicator(
              value: progress.clamp(0.0, 1.0),
              backgroundColor: theme.colorScheme.surfaceContainerHighest,
              valueColor: const AlwaysStoppedAnimation<Color>(Colors.purple),
            ),
            const SizedBox(height: 8),
            Row(
              mainAxisAlignment: MainAxisAlignment.spaceBetween,
              children: [
                Text(
                  '${_keyPressTimestamps.length} keystrokes',
                  style: theme.textTheme.bodySmall,
                ),
                Text(
                  '${(progress * 100).toInt()}% complete',
                  style: theme.textTheme.bodySmall,
                ),
              ],
            ),
            const SizedBox(height: 16),
            KeyboardListener(
              focusNode: _focusNode,
              onKeyEvent: _handleKeyPress,
              child: Container(
                padding: const EdgeInsets.all(12),
                decoration: BoxDecoration(
                  border: Border.all(color: Colors.purple),
                  borderRadius: BorderRadius.circular(8),
                ),
                child: Column(
                  children: [
                    const Icon(Icons.keyboard, size: 32, color: Colors.purple),
                    const SizedBox(height: 8),
                    Text(
                      'Click here and start typing',
                      style: theme.textTheme.bodySmall?.copyWith(
                        color: Colors.purple,
                      ),
                    ),
                  ],
                ),
              ),
            ),
          ],
        ),
      ),
      actions: [
        TextButton(
          onPressed: () {
            _timer?.cancel();
            Navigator.pop(context);
            widget.onCancel();
          },
          child: const Text('Cancel'),
        ),
        FilledButton(
          onPressed:
              _keyPressTimestamps.length >= 10 ? _finishSimulation : null,
          child: const Text('Finish Early'),
        ),
      ],
    );
  }
}

/// Statistics calculator for typing speed analysis.
class TypingStatistics {
  TypingStatistics({
    required this.mean,
    required this.stdDev,
  });

  final double mean;
  final double stdDev;

  /// Calculate recommended timeout based on typing statistics.
  /// Recommends threshold at mean + 1 stddev for ~84% accuracy.
  double calculateRecommendedTimeout({
    double minTimeout = 100,
    double maxTimeout = 1000,
  }) {
    return (mean + stdDev).clamp(minTimeout, maxTimeout);
  }

  /// Calculate estimated miss rate using cumulative distribution function.
  ///
  /// The miss rate represents the probability that a key press intended as
  /// a "tap" will exceed the threshold and be interpreted as a "hold".
  /// This uses a normal distribution model of inter-key delays.
  double calculateMissRate(double threshold) {
    // P(miss) = P(key_held_time > threshold)
    // For typing, key held time follows approximately normal distribution
    // Miss rate = 1 - normalCdf(threshold, mean, stddev)
    // Using complementary error function for normal CDF:
    // normalCdf(x) = 0.5 * (1 + erf((x - mean) / (stddev * sqrt(2))))

    final z = (threshold - mean) / (stdDev * math.sqrt(2));
    final cdf = 0.5 * (1 + _erf(z));

    // Miss rate is the probability of being below threshold
    // (i.e., releasing too quickly when trying to hold)
    // For tap-hold, we want P(tap_duration > threshold) which causes false hold
    // This is 1 - CDF
    final missRate = (1 - cdf) * 100;

    return missRate;
  }

  /// Approximate error function using Horner's method.
  static double _erf(double x) {
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
}
