import 'dart:developer';

import 'package:flutter/widgets.dart';

import 'logger.dart';

/// Lightweight helper for emitting Timeline spans + structured logs.
class PerformanceTimelineTracer {
  PerformanceTimelineTracer._();

  static final StructuredLogger _logger = getLogger('performance');

  /// Start a span that writes to the Flutter DevTools timeline and logs.
  static PerformanceSpan start(
    String name, {
    Map<String, Object?> context = const {},
  }) {
    final task = TimelineTask()..start(name, arguments: context);
    _logger.info('perf.span.start', context: {'name': name, ...context});
    return PerformanceSpan._(
      name: name,
      task: task,
      logger: _logger,
      context: context,
    );
  }

  /// Trace an async operation and return its result.
  static Future<T> traceAsync<T>(
    String name,
    Future<T> Function() run, {
    Map<String, Object?> context = const {},
  }) async {
    final span = start(name, context: context);
    try {
      return await run();
    } finally {
      span.finish();
    }
  }

  /// Emit an instantaneous marker for the timeline.
  static void instant(String name, {Map<String, Object?> context = const {}}) {
    Timeline.instantSync(name, arguments: context);
    _logger.debug('perf.instant', context: {'name': name, ...context});
  }
}

/// Handle to a running performance span.
class PerformanceSpan {
  PerformanceSpan._({
    required this.name,
    required TimelineTask task,
    required StructuredLogger logger,
    required Map<String, Object?> context,
  }) : _task = task,
       _logger = logger,
       _baseContext = context,
       _stopwatch = Stopwatch()..start();

  final String name;
  final TimelineTask _task;
  final StructuredLogger _logger;
  final Map<String, Object?> _baseContext;
  final Stopwatch _stopwatch;
  bool _finished = false;

  /// Complete the span and log duration.
  Duration finish({Map<String, Object?> context = const {}}) {
    if (_finished) {
      return _stopwatch.elapsed;
    }

    _finished = true;
    _stopwatch.stop();
    _task.finish();
    final duration = _stopwatch.elapsed;
    _logger.info(
      'perf.span.finish',
      context: {
        'name': name,
        'duration_ms': duration.inMilliseconds,
        ..._baseContext,
        ...context,
      },
    );
    return duration;
  }

  /// Finish the span on the next frame (useful for startup instrumentation).
  void completeOnFirstFrame({Map<String, Object?> context = const {}}) {
    WidgetsBinding.instance.addPostFrameCallback((_) {
      finish(context: context);
    });
  }
}
