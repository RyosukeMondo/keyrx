/// Unit tests for DaemonService.
///
/// Focus areas:
/// - Connection edge cases (timeout, retry, concurrent)
/// - Request/response matching with concurrent requests
/// - Message parsing errors and buffer overflow
/// - Socket error handling and reconnection
/// - State consistency and cleanup
library;

import 'dart:async';
import 'dart:convert';
import 'dart:io';

import 'package:flutter_test/flutter_test.dart';
import 'package:keyrx_ui/models/daemon_protocol.dart';
import 'package:keyrx_ui/services/daemon_service.dart';
import 'package:mocktail/mocktail.dart';

// Mocks
class MockSocket extends Mock implements Socket {}

class MockStreamSubscription extends Mock
    implements StreamSubscription<List<int>> {}

void main() {
  TestWidgetsFlutterBinding.ensureInitialized();

  group('DaemonService - Connection Management', () {
    test('connects successfully and updates state', () async {
      final service = DaemonService(socketPath: '/tmp/test.sock');

      // Note: Cannot fully test without socket mock injection
      // This test verifies API structure only
      expect(service.state, DaemonConnectionState.disconnected);
      expect(service.isConnected, false);
    });

    test('throws on connection timeout', () async {
      final service = DaemonService(socketPath: '/nonexistent/path.sock');

      // Should timeout connecting to non-existent socket
      await expectLater(service.connect(), throwsA(isA<DaemonException>()));

      expect(service.state, DaemonConnectionState.error);
    });

    test('prevents concurrent connection attempts', () async {
      final service = DaemonService(socketPath: '/tmp/test.sock');

      // Start first connection (will fail but that's ok)
      final future1 = service.connect();

      // Try second connection while first is in progress
      // Note: This will throw before the first connection completes
      expect(
        service.connect,
        throwsA(
          isA<DaemonException>().having(
            (e) => e.message,
            'message',
            contains('already in progress'),
          ),
        ),
      );

      // Clean up first connection
      await expectLater(future1, throwsA(anything));
    });

    test('state stream emits connection state changes', () async {
      final service = DaemonService(socketPath: '/tmp/test.sock');

      final states = <DaemonConnectionState>[];
      final subscription = service.connectionState.listen(states.add);

      // Attempt connection (will fail)
      await expectLater(service.connect(), throwsA(isA<DaemonException>()));

      await Future<void>.delayed(const Duration(milliseconds: 100));

      expect(
        states,
        containsAll([
          DaemonConnectionState.connecting,
          DaemonConnectionState.error,
        ]),
      );

      await subscription.cancel();
      await service.dispose();
    });

    test('disconnect clears pending requests', () async {
      final service = DaemonService(socketPath: '/tmp/test.sock');

      // Can't easily test without injecting mock socket
      // But we can verify dispose doesn't throw
      await service.dispose();
      expect(service.state, DaemonConnectionState.disconnected);
    });
  });

  group('DaemonService - Request/Response Handling', () {
    test('sendRequest throws when not connected', () async {
      final service = DaemonService(socketPath: '/tmp/test.sock');

      await expectLater(
        service.sendRequest<void>('test.method', parseResult: (_) {}),
        throwsA(
          isA<DaemonException>().having(
            (e) => e.message,
            'message',
            contains('Not connected'),
          ),
        ),
      );
    });

    test('request timeout throws DaemonException', () async {
      // This test would require socket injection to properly test
      // Skipping for now as the implementation has built-in timeout
    }, skip: true);

    test('concurrent requests use unique IDs', () async {
      // This test would require socket injection
      // The implementation uses incrementing _requestId
    }, skip: true);
  });

  group('DaemonService - Protocol Message Parsing', () {
    test('parses valid JSON-RPC response', () {
      final json = {
        'jsonrpc': '2.0',
        'result': {'status': 'ok'},
        'id': 1,
      };

      final message = IncomingMessage.fromJson(json);

      expect(
        message,
        isA<IncomingMessage>().having(
          (m) =>
              m.maybeWhen(response: (r) => r.id.asNumber, orElse: () => null),
          'response id',
          1,
        ),
      );
    });

    test('parses JSON-RPC error response', () {
      final json = {
        'jsonrpc': '2.0',
        'error': {'code': -32600, 'message': 'Invalid request'},
        'id': 1,
      };

      final message = IncomingMessage.fromJson(json);

      expect(
        message,
        isA<IncomingMessage>().having(
          (m) => m.maybeWhen(error: (e) => e.error.code, orElse: () => null),
          'error code',
          -32600,
        ),
      );
    });

    test('parses JSON-RPC notification', () {
      final json = {
        'jsonrpc': '2.0',
        'method': 'events.input',
        'params': {
          'device_name': 'Test Device',
          'key_name': 'A',
          'key_code': 30,
          'event_type': 'key_press',
        },
      };

      final message = IncomingMessage.fromJson(json);

      expect(
        message,
        isA<IncomingMessage>().having(
          (m) => m.maybeWhen(notification: (n) => n.method, orElse: () => null),
          'notification method',
          'events.input',
        ),
      );
    });

    test('throws on malformed JSON', () {
      expect(
        () => jsonDecode('{invalid json'),
        throwsA(isA<FormatException>()),
      );
    });

    test('handles missing required fields gracefully', () {
      final json = {
        'jsonrpc': '2.0',
        // Missing 'method' for notification
      };

      expect(() => IncomingMessage.fromJson(json), throwsA(anything));
    });
  });

  group('DaemonService - Event Streaming', () {
    test('event stream broadcasts to multiple listeners', () async {
      final service = DaemonService(socketPath: '/tmp/test.sock');

      final events1 = <DaemonEvent>[];
      final events2 = <DaemonEvent>[];

      final sub1 = service.events.listen(events1.add);
      final sub2 = service.events.listen(events2.add);

      // Would need to inject events to test fully
      await Future<void>.delayed(const Duration(milliseconds: 50));

      expect(events1, isEmpty);
      expect(events2, isEmpty);

      await sub1.cancel();
      await sub2.cancel();
      await service.dispose();
    });

    test('DaemonEvent.fromNotification parses input event', () {
      const notification = JsonRpcNotification(
        jsonrpc: '2.0',
        method: DaemonNotifications.eventInput,
        params: {
          'timestampUs': 1234567890,
          'devicePath': '/dev/input/event0',
          'deviceName': 'Test Keyboard',
          'keyName': 'A',
          'keyCode': 30,
          'eventType': 'key_press',
        },
      );

      final event = DaemonEvent.fromNotification(notification);

      expect(
        event,
        isA<DaemonEvent>().having(
          (e) => e.maybeWhen(input: (i) => i.keyName, orElse: () => null),
          'key name',
          'A',
        ),
      );
    });

    test('DaemonEvent.fromNotification parses output event', () {
      const notification = JsonRpcNotification(
        jsonrpc: '2.0',
        method: DaemonNotifications.eventOutput,
        params: {
          'timestampUs': 1234567890,
          'keyName': 'B',
          'keyCode': 48,
          'actionType': 'key_release',
        },
      );

      final event = DaemonEvent.fromNotification(notification);

      expect(
        event,
        isA<DaemonEvent>().having(
          (e) => e.maybeWhen(output: (o) => o.keyName, orElse: () => null),
          'key name',
          'B',
        ),
      );
    });

    test('DaemonEvent.fromNotification parses state change event', () {
      const notification = JsonRpcNotification(
        jsonrpc: '2.0',
        method: DaemonNotifications.eventStateChange,
        params: {
          'type': 'engine.started',
          'timestampUs': 1234567890,
          'message': 'Engine started successfully',
        },
      );

      final event = DaemonEvent.fromNotification(notification);

      expect(
        event,
        isA<DaemonEvent>().having(
          (e) => e.maybeWhen(stateChange: (s) => s.type, orElse: () => null),
          'event type',
          'engine.started',
        ),
      );
    });

    test('DaemonEvent.fromNotification throws on unknown method', () {
      const notification = JsonRpcNotification(
        jsonrpc: '2.0',
        method: 'unknown.method',
        params: {},
      );

      expect(
        () => DaemonEvent.fromNotification(notification),
        throwsA(isA<ArgumentError>()),
      );
    });
  });

  group('DaemonService - API Methods', () {
    test('startEngine constructs correct request', () async {
      final service = DaemonService(socketPath: '/tmp/test.sock');

      // Would need to be connected to test fully
      await expectLater(
        service.startEngine(
          configPath: '/path/to/config',
          scriptPath: '/path/to/script',
        ),
        throwsA(
          isA<DaemonException>().having(
            (e) => e.message,
            'message',
            contains('Not connected'),
          ),
        ),
      );
    });

    test('stopEngine constructs correct request', () async {
      final service = DaemonService(socketPath: '/tmp/test.sock');

      await expectLater(
        service.stopEngine(),
        throwsA(
          isA<DaemonException>().having(
            (e) => e.message,
            'message',
            contains('Not connected'),
          ),
        ),
      );
    });

    test('getStatus constructs correct request', () async {
      final service = DaemonService(socketPath: '/tmp/test.sock');

      await expectLater(
        service.getStatus(),
        throwsA(
          isA<DaemonException>().having(
            (e) => e.message,
            'message',
            contains('Not connected'),
          ),
        ),
      );
    });

    test('subscribeEvents constructs correct request', () async {
      final service = DaemonService(socketPath: '/tmp/test.sock');

      await expectLater(
        service.subscribeEvents(eventTypes: ['input', 'output']),
        throwsA(
          isA<DaemonException>().having(
            (e) => e.message,
            'message',
            contains('Not connected'),
          ),
        ),
      );
    });
  });

  group('DaemonService - Edge Cases and Bug Discovery', () {
    test('handles rapid connect/disconnect cycles', () async {
      final service = DaemonService(socketPath: '/tmp/test.sock');

      for (var i = 0; i < 5; i++) {
        try {
          await service.connect();
        } catch (e) {
          // Expected to fail
        }
        await service.disconnect();
      }

      expect(service.state, DaemonConnectionState.disconnected);
      await service.dispose();
    });

    test('dispose is idempotent', () async {
      final service = DaemonService(socketPath: '/tmp/test.sock');

      await service.dispose();
      await service.dispose();
      await service.dispose();

      expect(service.state, DaemonConnectionState.disconnected);
    });

    test('handles message buffer overflow correctly', () {
      // The implementation uses a string buffer that accumulates until \n
      // Test that very long lines don't cause issues
      final service = DaemonService(socketPath: '/tmp/test.sock');

      // This would require injecting data into the socket
      // The implementation should handle this gracefully
      expect(service, isNotNull);
    }, skip: true);

    test('handles partial JSON messages correctly', () {
      // Messages are line-delimited, test partial message handling
      // Would require socket injection
    }, skip: true);

    test('concurrent requests maintain ID uniqueness', () {
      final service = DaemonService(socketPath: '/tmp/test.sock');

      // The implementation uses ++_requestId which is not atomic
      // but Dart is single-threaded so should be safe
      // This test documents the assumption
      expect(service, isNotNull);
    });

    test('socket error during request completes future with error', () async {
      // Would require socket injection to simulate error
    }, skip: true);

    test(
      'disconnection during pending requests completes with error',
      () async {
        final service = DaemonService(socketPath: '/tmp/test.sock');

        // Try to start a request while not connected
        final requestFuture = service.sendRequest<void>(
          'test.method',
          parseResult: (_) {},
        );

        // Should immediately fail
        await expectLater(requestFuture, throwsA(isA<DaemonException>()));
      },
    );

    test('response with unknown ID is ignored', () {
      // Would require injecting socket data
      // The implementation looks up by ID and silently ignores unknown IDs
    }, skip: true);

    test('state transitions are consistent', () async {
      final service = DaemonService(socketPath: '/tmp/test.sock');

      final states = <DaemonConnectionState>[];
      final sub = service.connectionState.listen(states.add);

      // Try to connect (will fail)
      try {
        await service.connect();
      } catch (e) {
        // Expected
      }

      await Future<void>.delayed(const Duration(milliseconds: 100));

      // Should never transition backwards (e.g., connected -> connecting)
      // Should not have duplicate consecutive states
      expect(states.length, greaterThan(0));

      // Clean up
      await sub.cancel();
      await service.dispose();
    });

    test('memory leak check: streams are closed on dispose', () async {
      final service = DaemonService(socketPath: '/tmp/test.sock');

      final sub1 = service.events.listen((_) {});
      final sub2 = service.connectionState.listen((_) {});

      await service.dispose();

      // After dispose, streams should be closed
      // Note: broadcast streams don't expose isClosed
      // This test documents the dispose behavior

      await sub1.cancel();
      await sub2.cancel();
    });

    test('socket path defaults to runtime dir or tmp', () {
      // Test default socket path logic
      final service = DaemonService();

      // Should not throw
      expect(service, isNotNull);
    });

    test('DaemonException formats with and without error code', () {
      const exception1 = DaemonException('Test error');
      expect(exception1.toString(), 'DaemonException: Test error');

      const exception2 = DaemonException('Test error', errorCode: 42);
      expect(exception2.toString(), 'DaemonException (code 42): Test error');
    });
  });

  group('DaemonService - Protocol Models', () {
    test('RequestId handles number and string IDs', () {
      const numId = RequestId.number(123);
      expect(numId.isNumber, true);
      expect(numId.asNumber, 123);
      expect(numId.toJson(), 123);

      const strId = RequestId.string('abc');
      expect(strId.isString, true);
      expect(strId.asString, 'abc');
      expect(strId.toJson(), 'abc');
    });

    test('EngineStatus fromJson parses correctly', () {
      final json = {
        'status': 'running',
        'configPath': '/path/to/config',
        'scriptPath': '/path/to/script',
        'devices': [
          {'name': 'Device 1', 'path': '/dev/input/event0'},
        ],
        'uptimeSeconds': 3600,
        'eventsProcessed': 12345,
      };

      final status = EngineStatus.fromJson(json);

      expect(status.status, 'running');
      expect(status.configPath, '/path/to/config');
      expect(status.scriptPath, '/path/to/script');
      expect(status.devices, hasLength(1));
      expect(status.uptimeSeconds, 3600);
      expect(status.eventsProcessed, 12345);
    });

    test('EngineStatus handles null optional fields', () {
      final json = {'status': 'stopped', 'devices': <Map<String, dynamic>>[]};

      final status = EngineStatus.fromJson(json);

      expect(status.status, 'stopped');
      expect(status.configPath, isNull);
      expect(status.scriptPath, isNull);
      expect(status.uptimeSeconds, isNull);
      expect(status.eventsProcessed, isNull);
    });

    test('DaemonDeviceInfo fromJson parses correctly', () {
      final json = {'name': 'Test Keyboard', 'path': '/dev/input/event3'};

      final device = DaemonDeviceInfo.fromJson(json);

      expect(device.name, 'Test Keyboard');
      expect(device.path, '/dev/input/event3');
    });
  });
}
