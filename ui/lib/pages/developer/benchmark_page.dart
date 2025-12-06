/// Benchmark page for latency performance testing.
///
/// Provides iteration configuration, run button with progress,
/// and results display with latency metrics.
library;

import 'package:flutter/material.dart';

import '../../services/benchmark_service.dart';

/// Benchmark page for running latency performance tests.
class BenchmarkPage extends StatefulWidget {
  const BenchmarkPage({super.key, required this.benchmarkService});

  final BenchmarkService benchmarkService;

  @override
  State<BenchmarkPage> createState() => _BenchmarkPageState();
}

class _BenchmarkPageState extends State<BenchmarkPage> {
  static const _minIterations = 1000;
  static const _maxIterations = 100000;
  static const _warningThresholdNs = 1000000; // 1ms

  double _iterations = 10000;
  BenchmarkData? _result;
  bool _isRunning = false;
  String? _error;

  Future<void> _runBenchmark() async {
    setState(() {
      _isRunning = true;
      _error = null;
      _result = null;
    });

    final result = await widget.benchmarkService.runBenchmark(_iterations.toInt());

    setState(() {
      _isRunning = false;
      if (result.hasError) {
        _error = result.errorMessage;
      } else {
        _result = result.data;
      }
    });
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(title: const Text('Latency Benchmark')),
      body: SingleChildScrollView(
        padding: const EdgeInsets.all(16),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.stretch,
          children: [
            _buildConfigCard(),
            const SizedBox(height: 24),
            if (_error != null) _buildErrorBanner(),
            if (_result != null) ...[
              if (_result!.hasWarning) _buildWarningBanner(),
              const SizedBox(height: 16),
              _buildResultsGrid(),
            ],
            if (_result == null && !_isRunning) _buildPlaceholder(),
          ],
        ),
      ),
    );
  }

  Widget _buildConfigCard() {
    return Card(
      child: Padding(
        padding: const EdgeInsets.all(16),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Text('Configuration', style: Theme.of(context).textTheme.titleMedium),
            const SizedBox(height: 16),
            Row(
              children: [
                const Icon(Icons.repeat, size: 20),
                const SizedBox(width: 8),
                Text('Iterations: ${_formatNumber(_iterations.toInt())}'),
              ],
            ),
            const SizedBox(height: 8),
            Slider(
              value: _iterations,
              min: _minIterations.toDouble(),
              max: _maxIterations.toDouble(),
              divisions: 99,
              label: _formatNumber(_iterations.toInt()),
              onChanged: _isRunning ? null : (value) => setState(() => _iterations = value),
            ),
            const SizedBox(height: 8),
            Row(
              mainAxisAlignment: MainAxisAlignment.spaceBetween,
              children: [
                Text(_formatNumber(_minIterations), style: Theme.of(context).textTheme.bodySmall),
                Text(_formatNumber(_maxIterations), style: Theme.of(context).textTheme.bodySmall),
              ],
            ),
            const SizedBox(height: 16),
            SizedBox(
              width: double.infinity,
              child: FilledButton.icon(
                onPressed: _isRunning ? null : _runBenchmark,
                icon: _isRunning
                    ? const SizedBox(
                        width: 16,
                        height: 16,
                        child: CircularProgressIndicator(strokeWidth: 2, color: Colors.white),
                      )
                    : const Icon(Icons.speed),
                label: Text(_isRunning ? 'Running...' : 'Run Benchmark'),
              ),
            ),
          ],
        ),
      ),
    );
  }

  Widget _buildErrorBanner() {
    return Card(
      color: Theme.of(context).colorScheme.errorContainer,
      child: Padding(
        padding: const EdgeInsets.all(12),
        child: Row(
          children: [
            Icon(Icons.error, color: Theme.of(context).colorScheme.error),
            const SizedBox(width: 12),
            Expanded(
              child: Text(
                _error!,
                style: TextStyle(color: Theme.of(context).colorScheme.onErrorContainer),
              ),
            ),
            IconButton(
              icon: const Icon(Icons.close),
              onPressed: () => setState(() => _error = null),
            ),
          ],
        ),
      ),
    );
  }

  Widget _buildWarningBanner() {
    final warning = _result?.warning ?? 'Latency exceeds 1ms threshold';

    return Card(
      color: Colors.orange.shade100,
      child: Padding(
        padding: const EdgeInsets.all(12),
        child: Row(
          children: [
            const Icon(Icons.warning, color: Colors.orange),
            const SizedBox(width: 12),
            Expanded(
              child: Text(warning, style: const TextStyle(color: Colors.black87)),
            ),
          ],
        ),
      ),
    );
  }

  Widget _buildResultsGrid() {
    final result = _result!;

    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Text('Results', style: Theme.of(context).textTheme.titleMedium),
        const SizedBox(height: 16),
        Row(
          children: [
            Expanded(child: _buildMetricCard('Min', result.minUs, Colors.green)),
            const SizedBox(width: 12),
            Expanded(child: _buildMetricCard('Mean', result.meanUs, Colors.blue)),
          ],
        ),
        const SizedBox(height: 12),
        Row(
          children: [
            Expanded(child: _buildMetricCard('P99', result.p99Us, _getP99Color(result.p99Us))),
            const SizedBox(width: 12),
            Expanded(child: _buildMetricCard('Max', result.maxUs, _getMaxColor(result.maxUs))),
          ],
        ),
        const SizedBox(height: 16),
        _buildSummaryCard(result),
      ],
    );
  }

  Widget _buildMetricCard(String label, double valueUs, Color color) {
    final valueNs = valueUs * 1000;
    final isHighLatency = valueNs >= _warningThresholdNs;

    return Card(
      child: Padding(
        padding: const EdgeInsets.all(16),
        child: Column(
          children: [
            Text(label, style: Theme.of(context).textTheme.bodySmall),
            const SizedBox(height: 8),
            Text(
              _formatLatency(valueUs),
              style: Theme.of(context).textTheme.headlineSmall?.copyWith(
                    color: color,
                    fontWeight: FontWeight.bold,
                  ),
            ),
            if (isHighLatency)
              Padding(
                padding: const EdgeInsets.only(top: 4),
                child: Icon(Icons.warning, size: 16, color: Colors.orange[700]),
              ),
          ],
        ),
      ),
    );
  }

  Widget _buildSummaryCard(BenchmarkData result) {
    return Card(
      child: Padding(
        padding: const EdgeInsets.all(16),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Text('Summary', style: Theme.of(context).textTheme.titleSmall),
            const SizedBox(height: 12),
            _buildSummaryRow('Iterations', _formatNumber(result.iterations)),
            _buildSummaryRow('Range', '${_formatLatency(result.minUs)} - ${_formatLatency(result.maxUs)}'),
            _buildSummaryRow('Jitter', _formatLatency(result.maxUs - result.minUs)),
          ],
        ),
      ),
    );
  }

  Widget _buildSummaryRow(String label, String value) {
    return Padding(
      padding: const EdgeInsets.symmetric(vertical: 4),
      child: Row(
        mainAxisAlignment: MainAxisAlignment.spaceBetween,
        children: [
          Text(label, style: Theme.of(context).textTheme.bodySmall),
          Text(value, style: const TextStyle(fontWeight: FontWeight.w500)),
        ],
      ),
    );
  }

  Widget _buildPlaceholder() {
    return Center(
      child: Column(
        mainAxisAlignment: MainAxisAlignment.center,
        children: [
          const SizedBox(height: 48),
          Icon(Icons.speed_outlined, size: 64, color: Colors.grey[400]),
          const SizedBox(height: 16),
          Text(
            'Run a benchmark to see results',
            style: TextStyle(color: Colors.grey[500]),
          ),
        ],
      ),
    );
  }

  Color _getP99Color(double us) {
    final ns = us * 1000;
    if (ns < 500000) return Colors.green;
    if (ns < _warningThresholdNs) return Colors.orange;
    return Colors.red;
  }

  Color _getMaxColor(double us) {
    final ns = us * 1000;
    if (ns < _warningThresholdNs) return Colors.blue;
    if (ns < 2000000) return Colors.orange;
    return Colors.red;
  }

  String _formatLatency(double us) {
    if (us < 1) return '${(us * 1000).toStringAsFixed(0)}ns';
    if (us < 1000) return '${us.toStringAsFixed(1)}µs';
    return '${(us / 1000).toStringAsFixed(2)}ms';
  }

  String _formatNumber(int n) {
    if (n >= 1000000) return '${(n / 1000000).toStringAsFixed(1)}M';
    if (n >= 1000) return '${(n / 1000).toStringAsFixed(0)}K';
    return '$n';
  }
}
