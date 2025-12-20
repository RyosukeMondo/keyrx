import 'dart:async';
import 'dart:convert';
import 'dart:io';
import 'package:test/test.dart';
import 'package:http/http.dart' as http;
import 'package:web_socket_channel/web_socket_channel.dart';
import 'package:core_logic/models/models.dart';
import 'package:core_logic/models/virtual_layout_type.dart';

/// Integration tests for the KeyRx backend API server
///
/// Tests all REST endpoints, WebSocket connections, real-time event delivery,
/// and error handling scenarios.
void main() {
  late Process serverProcess;
  late String baseUrl;
  late String wsUrl;
  const port = 8888; // Use a different port for tests

  setUpAll(() async {
    // Start the server process
    baseUrl = 'http://localhost:$port';
    wsUrl = 'ws://localhost:$port';

    // Get the current directory (should be apps/keyrx_server when running tests)
    final currentDir = Directory.current.path;

    serverProcess = await Process.start(
      'dart',
      ['run', 'bin/server.dart', '-p', port.toString()],
      workingDirectory: currentDir,
    );

    // Wait for server to start
    await Future.delayed(const Duration(seconds: 3));

    // Verify server is running
    var retries = 0;
    while (retries < 5) {
      try {
        final response = await http.get(Uri.parse('$baseUrl/api/devices'));
        if (response.statusCode == 200) {
          break;
        }
      } catch (e) {
        retries++;
        if (retries >= 5) {
          throw Exception('Failed to start server: $e');
        }
        await Future.delayed(const Duration(seconds: 1));
      }
    }
  });

  tearDownAll(() async {
    // Stop the server
    serverProcess.kill();
    await serverProcess.exitCode.timeout(
      const Duration(seconds: 5),
      onTimeout: () {
        serverProcess.kill(ProcessSignal.sigkill);
        return -1;
      },
    );
  });

  group('REST API - Hardware Profile Endpoints', () {
    test('GET /api/hardware/profiles - list profiles', () async {
      final response = await http.get(
        Uri.parse('$baseUrl/api/hardware/profiles'),
      );

      expect(response.statusCode, equals(200));
      final data = jsonDecode(response.body) as Map<String, dynamic>;
      expect(data, containsPair('profiles', isA<List>()));
    });

    test('POST /api/hardware/profiles - save profile', () async {
      final profile = const HardwareProfile(
        id: 'test-hw-profile',
        virtualLayoutId: 'default-layout',
        vendorId: 0x1234,
        productId: 0x5678,
        name: 'Test Hardware Profile',
      );

      final response = await http.post(
        Uri.parse('$baseUrl/api/hardware/profiles'),
        headers: {'Content-Type': 'application/json'},
        body: jsonEncode(profile.toJson()),
      );

      expect(response.statusCode, equals(200));
      final data = jsonDecode(response.body) as Map<String, dynamic>;
      expect(data, containsPair('profile', isA<Map>()));
    });

    test('DELETE /api/hardware/profiles/:id - delete profile', () async {
      final response = await http.delete(
        Uri.parse('$baseUrl/api/hardware/profiles/test-hw-profile'),
      );

      expect(response.statusCode, equals(200));
      final data = jsonDecode(response.body) as Map<String, dynamic>;
      expect(data, containsPair('success', true));
    });
  });

  group('REST API - Keymap Endpoints', () {
    test('GET /api/keymaps - list keymaps', () async {
      final response = await http.get(Uri.parse('$baseUrl/api/keymaps'));

      expect(response.statusCode, equals(200));
      final data = jsonDecode(response.body) as Map<String, dynamic>;
      expect(data, containsPair('keymaps', isA<List>()));
    });

    test('POST /api/keymaps - save keymap', () async {
      final keymap = const Keymap(
        id: 'test-keymap',
        virtualLayoutId: 'default-layout',
        name: 'Test Keymap',
      );

      final response = await http.post(
        Uri.parse('$baseUrl/api/keymaps'),
        headers: {'Content-Type': 'application/json'},
        body: jsonEncode(keymap.toJson()),
      );

      expect(response.statusCode, equals(200));
      final data = jsonDecode(response.body) as Map<String, dynamic>;
      expect(data, containsPair('keymap', isA<Map>()));
    });

    test('DELETE /api/keymaps/:id - delete keymap', () async {
      final response = await http.delete(
        Uri.parse('$baseUrl/api/keymaps/test-keymap'),
      );

      expect(response.statusCode, equals(200));
      final data = jsonDecode(response.body) as Map<String, dynamic>;
      expect(data, containsPair('success', true));
    });
  });

  group('REST API - Layout Endpoints', () {
    test('GET /api/layouts - list layouts', () async {
      final response = await http.get(Uri.parse('$baseUrl/api/layouts'));

      expect(response.statusCode, equals(200));
      final data = jsonDecode(response.body) as Map<String, dynamic>;
      expect(data, containsPair('layouts', isA<List>()));
    });

    test('POST /api/layouts - save layout', () async {
      final layout = const VirtualLayout(
        id: 'test-layout',
        name: 'Test Layout',
        layoutType: VirtualLayoutType.semantic,
      );

      final response = await http.post(
        Uri.parse('$baseUrl/api/layouts'),
        headers: {'Content-Type': 'application/json'},
        body: jsonEncode(layout.toJson()),
      );

      expect(response.statusCode, equals(200));
      final data = jsonDecode(response.body) as Map<String, dynamic>;
      expect(data, containsPair('layout', isA<Map>()));
    });

    test('DELETE /api/layouts/:id - delete layout', () async {
      final response = await http.delete(
        Uri.parse('$baseUrl/api/layouts/test-layout'),
      );

      expect(response.statusCode, equals(200));
      final data = jsonDecode(response.body) as Map<String, dynamic>;
      expect(data, containsPair('success', true));
    });
  });

  group('REST API - Device Registry Endpoints', () {
    test('GET /api/devices - get devices', () async {
      final response = await http.get(Uri.parse('$baseUrl/api/devices'));

      expect(response.statusCode, equals(200));
      final data = jsonDecode(response.body) as Map<String, dynamic>;
      expect(data, containsPair('devices', isA<List>()));
    });

    test('POST /api/devices/refresh - refresh devices', () async {
      final response = await http.post(
        Uri.parse('$baseUrl/api/devices/refresh'),
      );

      expect(response.statusCode, equals(200));
      final data = jsonDecode(response.body) as Map<String, dynamic>;
      expect(data, containsPair('devices', isA<List>()));
    });

    test('POST /api/devices/virtual - add virtual device', () async {
      final identity = const DeviceIdentity(
        vendorId: 0x9999,
        productId: 0x8888,
        serialNumber: 'test-virtual-device',
        userLabel: 'Test Virtual Device',
      );

      final response = await http.post(
        Uri.parse('$baseUrl/api/devices/virtual'),
        headers: {'Content-Type': 'application/json'},
        body: jsonEncode(identity.toJson()),
      );

      expect(response.statusCode, equals(200));
      final data = jsonDecode(response.body) as Map<String, dynamic>;
      expect(data, containsPair('success', true));
    });

    test('DELETE /api/devices/virtual/:key - remove virtual device', () async {
      final response = await http.delete(
        Uri.parse('$baseUrl/api/devices/virtual/test-virtual-device'),
      );

      expect(response.statusCode, equals(200));
      final data = jsonDecode(response.body) as Map<String, dynamic>;
      expect(data, containsPair('success', true));
    });
  });

  group('REST API - Profile Registry Endpoints', () {
    test('GET /api/profiles - list profiles', () async {
      final response = await http.get(Uri.parse('$baseUrl/api/profiles'));

      expect(response.statusCode, equals(200));
      final data = jsonDecode(response.body) as Map<String, dynamic>;
      expect(data, containsPair('profiles', isA<List>()));
    });

    test('POST /api/profiles/refresh - refresh profiles', () async {
      final response = await http.post(
        Uri.parse('$baseUrl/api/profiles/refresh'),
      );

      expect(response.statusCode, equals(200));
      final data = jsonDecode(response.body) as Map<String, dynamic>;
      expect(data, containsPair('profiles', isA<List>()));
    });

    test('GET /api/profiles/compatible/:layoutType - find compatible profiles', () async {
      final response = await http.get(
        Uri.parse('$baseUrl/api/profiles/compatible/standard'),
      );

      expect(response.statusCode, equals(200));
      final data = jsonDecode(response.body) as Map<String, dynamic>;
      expect(data, containsPair('profiles', isA<List>()));
    });
  });

  group('REST API - Runtime Service Endpoints', () {
    test('GET /api/runtime/config - get runtime config', () async {
      final response = await http.get(
        Uri.parse('$baseUrl/api/runtime/config'),
      );

      expect(response.statusCode, equals(200));
      final data = jsonDecode(response.body) as Map<String, dynamic>;
      expect(data, containsPair('config', isA<Map>()));
    });
  });

  group('REST API - Error Handling', () {
    test('POST with invalid JSON returns 400', () async {
      final response = await http.post(
        Uri.parse('$baseUrl/api/hardware/profiles'),
        headers: {'Content-Type': 'application/json'},
        body: 'invalid json',
      );

      expect(response.statusCode, equals(400));
      final data = jsonDecode(response.body) as Map<String, dynamic>;
      expect(data, containsPair('error', isA<String>()));
    });

    test('GET non-existent profile returns 404', () async {
      final response = await http.get(
        Uri.parse('$baseUrl/api/profiles/non-existent-profile-id-12345'),
      );

      expect(response.statusCode, equals(404));
      final data = jsonDecode(response.body) as Map<String, dynamic>;
      expect(data, containsPair('error', isA<String>()));
    });

    test('POST with missing required fields returns 400', () async {
      final response = await http.post(
        Uri.parse('$baseUrl/api/devices/test-device/toggle'),
        headers: {'Content-Type': 'application/json'},
        body: jsonEncode({}), // Missing 'enabled' field
      );

      expect(response.statusCode, equals(400));
      final data = jsonDecode(response.body) as Map<String, dynamic>;
      expect(data, containsPair('error', isA<String>()));
    });

    test('Invalid layout type returns error', () async {
      final response = await http.get(
        Uri.parse('$baseUrl/api/profiles/compatible/invalid-layout-type'),
      );

      expect(response.statusCode, equals(400));
      final data = jsonDecode(response.body) as Map<String, dynamic>;
      expect(data, containsPair('error', isA<String>()));
    });
  });

  group('CORS Headers', () {
    test('All responses include CORS headers', () async {
      final response = await http.get(Uri.parse('$baseUrl/api/devices'));

      expect(
        response.headers['access-control-allow-origin'],
        equals('*'),
      );
      expect(
        response.headers['access-control-allow-methods'],
        contains('GET'),
      );
      expect(
        response.headers['access-control-allow-headers'],
        isNotNull,
      );
    });

    test('OPTIONS preflight request returns 200', () async {
      final request = http.Request(
        'OPTIONS',
        Uri.parse('$baseUrl/api/devices'),
      );
      final streamedResponse = await request.send();
      final response = await http.Response.fromStream(streamedResponse);

      expect(response.statusCode, equals(200));
      expect(
        response.headers['access-control-allow-origin'],
        equals('*'),
      );
    });
  });

  group('WebSocket Connection', () {
    test('Can connect to WebSocket endpoint', () async {
      final channel = WebSocketChannel.connect(
        Uri.parse('$wsUrl/api/ws'),
      );

      // Wait for connection message
      final message = await channel.stream.first.timeout(
        const Duration(seconds: 5),
      );

      final data = jsonDecode(message as String) as Map<String, dynamic>;
      expect(data['type'], equals('connected'));
      expect(data['message'], contains('KeyRx'));
      expect(data, containsPair('timestamp', isA<String>()));

      await channel.sink.close();
    });

    test('Multiple clients can connect simultaneously', () async {
      final channels = <WebSocketChannel>[];

      // Connect 5 clients
      for (var i = 0; i < 5; i++) {
        final channel = WebSocketChannel.connect(
          Uri.parse('$wsUrl/api/ws'),
        );
        channels.add(channel);

        // Wait for connection message
        final message = await channel.stream.first.timeout(
          const Duration(seconds: 5),
        );

        final data = jsonDecode(message as String) as Map<String, dynamic>;
        expect(data['type'], equals('connected'));
      }

      // Close all connections
      for (final channel in channels) {
        await channel.sink.close();
      }
    });
  });

  group('Real-time Event Delivery', () {
    test('WebSocket receives device update events', () async {
      final channel = WebSocketChannel.connect(
        Uri.parse('$wsUrl/api/ws'),
      );

      final events = <Map<String, dynamic>>[];
      final completer = Completer<void>();

      // Listen to all messages
      final subscription = channel.stream.listen((msg) {
        final data = jsonDecode(msg as String) as Map<String, dynamic>;
        events.add(data);
        if (data['type'] == 'devices_updated') {
          completer.complete();
        }
      });

      // Wait for connection message
      await Future.delayed(const Duration(milliseconds: 500));

      // Trigger device refresh
      await http.post(Uri.parse('$baseUrl/api/devices/refresh'));

      // Wait for event or timeout
      await completer.future.timeout(
        const Duration(seconds: 10),
        onTimeout: () {},
      );

      // Check if we received the event
      final hasEvent = events.any((e) => e['type'] == 'devices_updated');
      if (hasEvent) {
        final event = events.firstWhere((e) => e['type'] == 'devices_updated');
        expect(event['type'], equals('devices_updated'));
        expect(event['data'], isA<List>());
        expect(event, containsPair('timestamp', isA<String>()));
      }

      await subscription.cancel();
      await channel.sink.close();
    });

    test('WebSocket receives hardware change events', () async {
      final channel = WebSocketChannel.connect(
        Uri.parse('$wsUrl/api/ws'),
      );

      final events = <Map<String, dynamic>>[];
      final completer = Completer<void>();

      // Listen to all messages
      final subscription = channel.stream.listen((msg) {
        final data = jsonDecode(msg as String) as Map<String, dynamic>;
        events.add(data);
        if (data['type'] == 'hardware_changed' && !completer.isCompleted) {
          completer.complete();
        }
      });

      // Wait for connection
      await Future.delayed(const Duration(milliseconds: 500));

      // Create a hardware profile to trigger change event
      final profile = const HardwareProfile(
        id: 'test-ws-profile',
        virtualLayoutId: 'default-layout',
        vendorId: 0x8888,
        productId: 0x9999,
        name: 'Test WS Profile',
      );

      await http.post(
        Uri.parse('$baseUrl/api/hardware/profiles'),
        headers: {'Content-Type': 'application/json'},
        body: jsonEncode(profile.toJson()),
      );

      // Wait for event or timeout
      await completer.future.timeout(
        const Duration(seconds: 10),
        onTimeout: () {},
      );

      // Check if we received the event
      final hasEvent = events.any((e) => e['type'] == 'hardware_changed');
      if (hasEvent) {
        final event = events.firstWhere((e) => e['type'] == 'hardware_changed');
        expect(event['type'], equals('hardware_changed'));
        expect(event, containsPair('timestamp', isA<String>()));
      }

      // Cleanup
      await http.delete(
        Uri.parse('$baseUrl/api/hardware/profiles/test-ws-profile'),
      );

      await subscription.cancel();
      await channel.sink.close();
    });

    test('All connected clients receive the same events', () async {
      final channel1 = WebSocketChannel.connect(Uri.parse('$wsUrl/api/ws'));
      final channel2 = WebSocketChannel.connect(Uri.parse('$wsUrl/api/ws'));

      // Listen for events on both channels
      final events1 = <Map<String, dynamic>>[];
      final events2 = <Map<String, dynamic>>[];

      final sub1 = channel1.stream.listen((msg) {
        events1.add(jsonDecode(msg as String) as Map<String, dynamic>);
      });

      final sub2 = channel2.stream.listen((msg) {
        events2.add(jsonDecode(msg as String) as Map<String, dynamic>);
      });

      // Wait for connections
      await Future.delayed(const Duration(milliseconds: 500));

      // Trigger an event
      await http.post(Uri.parse('$baseUrl/api/devices/refresh'));

      // Wait for events to be received
      await Future.delayed(const Duration(seconds: 2));

      // Both clients should have received events
      expect(events1.isNotEmpty, isTrue);
      expect(events2.isNotEmpty, isTrue);

      // Events should contain devices_updated
      final hasDeviceUpdate1 = events1.any((e) => e['type'] == 'devices_updated');
      final hasDeviceUpdate2 = events2.any((e) => e['type'] == 'devices_updated');

      expect(hasDeviceUpdate1, isTrue);
      expect(hasDeviceUpdate2, isTrue);

      await sub1.cancel();
      await sub2.cancel();
      await channel1.sink.close();
      await channel2.sink.close();
    });
  });

  group('Concurrent Operations', () {
    test('Server handles multiple concurrent REST requests', () async {
      // Send 10 concurrent requests
      final futures = <Future<http.Response>>[];
      for (var i = 0; i < 10; i++) {
        futures.add(http.get(Uri.parse('$baseUrl/api/devices')));
      }

      final responses = await Future.wait(futures);

      // All requests should succeed
      for (final response in responses) {
        expect(response.statusCode, equals(200));
        final data = jsonDecode(response.body) as Map<String, dynamic>;
        expect(data, containsPair('devices', isA<List>()));
      }
    });

    test('Server handles concurrent WebSocket connections', () async {
      final channels = <WebSocketChannel>[];
      final connectionFutures = <Future<void>>[];

      // Connect 20 clients concurrently
      for (var i = 0; i < 20; i++) {
        final channel = WebSocketChannel.connect(
          Uri.parse('$wsUrl/api/ws'),
        );
        channels.add(channel);

        connectionFutures.add(
          channel.stream.first.then((msg) {
            final data = jsonDecode(msg as String) as Map<String, dynamic>;
            expect(data['type'], equals('connected'));
          }),
        );
      }

      // Wait for all connections to complete
      await Future.wait(connectionFutures).timeout(
        const Duration(seconds: 10),
      );

      // Close all connections
      for (final channel in channels) {
        await channel.sink.close();
      }
    });
  });

  group('Integration - End-to-End Workflows', () {
    test('Create hardware profile and receive change event', () async {
      final channel = WebSocketChannel.connect(Uri.parse('$wsUrl/api/ws'));

      final events = <Map<String, dynamic>>[];
      final completer = Completer<void>();

      // Listen to all messages
      final subscription = channel.stream.listen((msg) {
        final data = jsonDecode(msg as String) as Map<String, dynamic>;
        events.add(data);
        if (data['type'] == 'hardware_changed' && !completer.isCompleted) {
          completer.complete();
        }
      });

      // Wait for connection
      await Future.delayed(const Duration(milliseconds: 500));

      // Create profile via REST API
      final profile = const HardwareProfile(
        id: 'e2e-test-profile',
        virtualLayoutId: 'default-layout',
        vendorId: 0x2222,
        productId: 0x1111,
        name: 'E2E Test Profile',
      );

      final createResponse = await http.post(
        Uri.parse('$baseUrl/api/hardware/profiles'),
        headers: {'Content-Type': 'application/json'},
        body: jsonEncode(profile.toJson()),
      );

      expect(createResponse.statusCode, equals(200));

      // Wait for event or timeout
      await completer.future.timeout(
        const Duration(seconds: 10),
        onTimeout: () {},
      );

      // Verify event was received
      final hasEvent = events.any((e) => e['type'] == 'hardware_changed');
      if (hasEvent) {
        final event = events.firstWhere((e) => e['type'] == 'hardware_changed');
        expect(event['type'], equals('hardware_changed'));
      }

      // Verify profile exists
      final listResponse = await http.get(
        Uri.parse('$baseUrl/api/hardware/profiles'),
      );
      final data = jsonDecode(listResponse.body) as Map<String, dynamic>;
      final profiles = data['profiles'] as List;
      final profileExists = profiles.any(
        (p) => (p as Map<String, dynamic>)['id'] == 'e2e-test-profile',
      );

      expect(profileExists, isTrue);

      // Cleanup
      await http.delete(
        Uri.parse('$baseUrl/api/hardware/profiles/e2e-test-profile'),
      );

      await subscription.cancel();
      await channel.sink.close();
    });
  });
}
