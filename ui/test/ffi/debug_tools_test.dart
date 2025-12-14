import 'dart:convert';

import 'package:flutter_test/flutter_test.dart';
import 'package:keyrx_ui/ffi/debug_tools.dart';
import 'package:keyrx_ui/utils/logger.dart';
import 'package:logging/logging.dart';

void main() {
  late List<String> lines;

  setUp(() async {
    lines = [];
    await LoggingConfigurator.resetForTesting();
    LoggingConfigurator.useTestSink(lines.add);
    LoggingConfigurator.configure(level: Level.ALL);
  });

  test('traceSync logs start and success with duration', () {
    final result = FfiDebugTools.traceSync<int>(
      operation: 'core.test_call',
      call: () => 7,
      context: {'sample': true},
      includeTimeline: false,
    );

    expect(result, 7);
    expect(lines.length, 2);

    final start = jsonDecode(lines.first) as Map<String, dynamic>;
    final success = jsonDecode(lines.last) as Map<String, dynamic>;

    expect(start['event'], 'ffi.call.start');
    expect(
      (start['context'] as Map<String, dynamic>)['operation'],
      'core.test_call',
    );
    expect(success['event'], 'ffi.call.success');
    expect((success['context'] as Map)['duration_ms'], isA<num>());
  });

  test('traceSync logs formatted errors', () {
    expect(
      () => FfiDebugTools.traceSync<void>(
        operation: 'core.bad_call',
        call: () => throw ArgumentError('bad ptr'),
        includeTimeline: false,
      ),
      throwsArgumentError,
    );

    final errorPayload = jsonDecode(lines.last) as Map<String, dynamic>;
    expect(errorPayload['event'], 'ffi.call.error');
    final context = errorPayload['context'] as Map<String, dynamic>;
    expect(context['error_code'], 'FFI_ARGUMENT');
    expect(context['operation'], 'core.bad_call');
  });

  test('formatError provides fallback context', () {
    final details = FfiDebugTools.formatError(Exception('random failure'));
    expect(details.code, 'FFI_UNKNOWN');
    expect(details.toContext()['error_message'], contains('random failure'));
  });
}
