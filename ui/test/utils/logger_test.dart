import 'dart:convert';

import 'package:flutter_test/flutter_test.dart';
import 'package:core_logic/utils/logger.dart';
import 'package:logging/logging.dart';

void main() {
  setUp(() async {
    await LoggingConfigurator.resetForTesting();
  });

  test('emits structured JSON with context and error fields', () {
    final lines = <String>[];
    final logger = getLogger('test_service', level: Level.ALL, sink: lines.add);

    logger.info(
      'device_selected',
      message: 'Device selection succeeded',
      context: {
        'path': '/dev/input/event3',
        'attempts': 1,
        'flags': ['usb', 'wired'],
      },
    );

    logger.error(
      'device_selection_error',
      message: 'Device selection failed',
      context: {'path': '/dev/input/event3'},
      error: Exception('bad device'),
      stackTrace: StackTrace.fromString('stack here'),
    );

    expect(lines.length, 2);

    final infoPayload = jsonDecode(lines.first) as Map<String, dynamic>;
    expect(infoPayload['level'], 'INFO');
    expect(infoPayload['service'], 'test_service');
    expect(infoPayload['event'], 'device_selected');
    expect(infoPayload['context'], {
      'path': '/dev/input/event3',
      'attempts': 1,
      'flags': ['usb', 'wired'],
    });

    final errorPayload = jsonDecode(lines[1]) as Map<String, dynamic>;
    expect(errorPayload['level'], 'SEVERE');
    expect(errorPayload['error'], contains('bad device'));
    expect(errorPayload['stack'], contains('stack here'));
  });
}
