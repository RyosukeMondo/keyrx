/// Tests for verifying Rust <-> Dart Daemon Protocol serialization compatibility.
library;

import 'package:core_logic/models/daemon_protocol.dart';
import 'package:flutter_test/flutter_test.dart';

void main() {
  group('Daemon Protocol Serialization Compatibility', () {
    test('EngineStatus parses snake_case JSON from Rust', () {
      // JSON exactly as it appears in Rust log output / protocol definition
      final json = {
        'status': 'running',
        'config_path': '/path/to/config.toml',
        'script_path': '/path/to/script.rhai',
        'devices': [
          {
            'name': 'Test Device',
            'path': '/dev/input/event0',
            'vendor_id': '0x1234',
            'product_id': '0x5678',
          },
        ],
        'uptime_seconds': 120,
        'events_processed': 1000,
      };

      final status = EngineStatus.fromJson(json);

      expect(status.status, 'running');
      expect(status.configPath, '/path/to/config.toml');
      expect(status.scriptPath, '/path/to/script.rhai');
      expect(status.uptimeSeconds, 120);
      expect(status.eventsProcessed, 1000);

      expect(status.devices, hasLength(1));
      expect(status.devices.first.name, 'Test Device');
      expect(status.devices.first.vendorId, '0x1234');
      expect(status.devices.first.productId, '0x5678');
    });

    test('StateChangeEvent parses snake_case JSON from Rust', () {
      // From log: "timestamp_us": ..., "type": "engine_started", ...
      final json = {
        'type': 'engine_started',
        'timestamp_us': 1234567890,
        'message': 'Engine started',
        'details': {'some_detail': 'value'},
      };

      final event = StateChangeEvent.fromJson(json);

      expect(event.type, 'engine_started');
      expect(event.timestampUs, 1234567890);
      expect(event.message, 'Engine started');
      expect(event.details, isNotNull);
    });

    test('InputEvent parses snake_case JSON from Rust', () {
      final json = {
        'timestamp_us': 987654321,
        'device_path': '/dev/input/event3',
        'device_name': 'Mechanical Keyboard',
        'event_type': 'key_press',
        'key_code': 30,
        'key_name': 'A',
        'modifiers': ['SHIFT'],
      };

      final event = InputEvent.fromJson(json);

      expect(event.timestampUs, 987654321);
      expect(event.devicePath, '/dev/input/event3');
      expect(event.deviceName, 'Mechanical Keyboard');
      expect(event.eventType, 'key_press');
      expect(event.keyCode, 30);
      expect(event.keyName, 'A');
      expect(event.modifiers, contains('SHIFT'));
    });

    test('OutputEvent parses snake_case JSON from Rust', () {
      final json = {
        'timestamp_us': 1122334455,
        'action_type': 'tap',
        'key_code': 40,
        'key_name': 'ENTER',
      };

      final event = OutputEvent.fromJson(json);

      expect(event.timestampUs, 1122334455);
      expect(event.actionType, 'tap');
      expect(event.keyCode, 40);
      expect(event.keyName, 'ENTER');
    });

    test(
      'DaemonEvent.fromNotification handles state change notification from Rust',
      () {
        // Rust sends: method: "event.state_change", params: { ... }
        final params = {
          'type': 'script_reloaded',
          'timestamp_us': 55555,
          'message': 'Script reloaded',
          'details': {'script_path': '/tmp/test.rhai'},
        };

        final notification = JsonRpcNotification(
          jsonrpc: '2.0',
          method: 'event.state_change',
          params: params,
        );

        final event = DaemonEvent.fromNotification(notification);

        expect(event, isA<DaemonEvent>());
        event.map(
          input: (_) => fail('Should be stateChange'),
          output: (_) => fail('Should be stateChange'),
          stateChange: (e) {
            expect(e.event.type, 'script_reloaded');
            expect(e.event.timestampUs, 55555);
            expect(e.event.details?['script_path'], '/tmp/test.rhai');
          },
          decision: (_) => fail('Should be stateChange'),
          discovery: (_) => fail('Should be stateChange'),
        );
      },
    );
  });
}
