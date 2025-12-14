/// Lightweight telemetry recorder for KeyFlow operations.
library;

import 'dart:math';

/// Snapshot of aggregated telemetry values.
class KeyFlowTelemetrySnapshot {
  const KeyFlowTelemetrySnapshot({
    required this.fetchLatencyP95Ms,
    required this.fetchSuccessCount,
    required this.fetchFailureCount,
    required this.renderLatencyP95Ms,
    required this.parseErrorCount,
    required this.parseErrorSources,
  });

  final int? fetchLatencyP95Ms;
  final int fetchSuccessCount;
  final int fetchFailureCount;
  final int? renderLatencyP95Ms;
  final int parseErrorCount;
  final List<String> parseErrorSources;

  Map<String, dynamic> toJson() {
    return {
      'fetchLatencyP95Ms': fetchLatencyP95Ms,
      'fetchSuccessCount': fetchSuccessCount,
      'fetchFailureCount': fetchFailureCount,
      'renderLatencyP95Ms': renderLatencyP95Ms,
      'parseErrorCount': parseErrorCount,
      'parseErrorSources': parseErrorSources,
    };
  }

  String cliSummary() {
    final latency = fetchLatencyP95Ms != null
        ? '${fetchLatencyP95Ms}ms'
        : 'n/a';
    final render = renderLatencyP95Ms != null
        ? '${renderLatencyP95Ms}ms'
        : 'n/a';
    final parseSources = parseErrorSources.isEmpty
        ? 'none'
        : parseErrorSources.join(',');
    return [
      'keyflow-telemetry',
      'fetch_p95=$latency',
      'success=$fetchSuccessCount',
      'failure=$fetchFailureCount',
      'render_p95=$render',
      'parse_errors=$parseErrorCount',
      'parse_sources=$parseSources',
    ].join(' | ');
  }
}

/// Records KeyFlow fetch and render timings without PII payloads.
class KeyFlowTelemetry {
  KeyFlowTelemetry({
    this.maxSamples = 48,
    this.largeLayoutKeyThreshold = 150,
    this.largeLayerThreshold = 5,
  }) : assert(maxSamples > 0);

  final int maxSamples;
  final int largeLayoutKeyThreshold;
  final int largeLayerThreshold;

  final List<int> _fetchLatenciesMs = [];
  final List<int> _renderLatenciesMs = [];
  final List<String> _parseErrorSources = [];
  int _fetchSuccess = 0;
  int _fetchFailure = 0;

  void recordFetch({required Duration duration, required bool success}) {
    _push(_fetchLatenciesMs, duration.inMilliseconds);
    if (success) {
      _fetchSuccess += 1;
    } else {
      _fetchFailure += 1;
    }
  }

  void recordParseError({required String source, int? payloadBytes}) {
    final payloadMarker = payloadBytes != null ? '(${payloadBytes}b)' : '';
    _parseErrorSources.add('$source$payloadMarker');
  }

  void recordRender({
    required Duration duration,
    required int keyCount,
    required int layerCount,
  }) {
    final isLarge =
        keyCount > largeLayoutKeyThreshold || layerCount > largeLayerThreshold;
    if (!isLarge) {
      return;
    }
    _push(_renderLatenciesMs, duration.inMilliseconds);
  }

  KeyFlowTelemetrySnapshot snapshot() {
    return KeyFlowTelemetrySnapshot(
      fetchLatencyP95Ms: _p95(_fetchLatenciesMs),
      fetchSuccessCount: _fetchSuccess,
      fetchFailureCount: _fetchFailure,
      renderLatencyP95Ms: _p95(_renderLatenciesMs),
      parseErrorCount: _parseErrorSources.length,
      parseErrorSources: List<String>.from(_parseErrorSources),
    );
  }

  String cliSummary() {
    return snapshot().cliSummary();
  }

  void reset() {
    _fetchLatenciesMs.clear();
    _renderLatenciesMs.clear();
    _parseErrorSources.clear();
    _fetchSuccess = 0;
    _fetchFailure = 0;
  }

  void _push(List<int> list, int value) {
    list.add(value);
    if (list.length > maxSamples) {
      list.removeAt(0);
    }
  }

  int? _p95(List<int> values) {
    if (values.isEmpty) {
      return null;
    }
    final sorted = List<int>.from(values)..sort();
    final index = (0.95 * (sorted.length - 1)).ceil();
    return sorted[min(index, sorted.length - 1)];
  }
}
