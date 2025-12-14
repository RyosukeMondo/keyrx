import 'package:flutter_test/flutter_test.dart';

import 'package:keyrx_ui/ffi/debug_tools.dart';

void main() {
  TestWidgetsFlutterBinding.ensureInitialized();

  const asyncThresholdMs = 40;
  const syncThresholdMs = 10;

  group('FFI latency benchmarks', () {
    test('traceAsync finishes under ${asyncThresholdMs}ms', () async {
      final stopwatch = Stopwatch()..start();

      final result = await FfiDebugTools.traceAsync<int>(
        operation: 'mock.async_call',
        includeTimeline: false,
        context: const {'path': '/tmp/mock'},
        call: () async {
          await Future<void>.delayed(const Duration(milliseconds: 12));
          return 42;
        },
      );

      stopwatch.stop();
      expect(result, 42);
      expect(stopwatch.elapsedMilliseconds, lessThan(asyncThresholdMs));
    });

    test('traceSync overhead stays under ${syncThresholdMs}ms', () {
      final stopwatch = Stopwatch()..start();

      final result = FfiDebugTools.traceSync<int>(
        operation: 'mock.sync_call',
        includeTimeline: false,
        context: const {'operation': 'noop'},
        call: () {
          var sum = 0;
          for (var i = 0; i < 1000; i++) {
            sum += i;
          }
          return sum;
        },
      );

      stopwatch.stop();
      expect(result, 499500);
      expect(stopwatch.elapsedMilliseconds, lessThan(syncThresholdMs));
    });
  });
}
