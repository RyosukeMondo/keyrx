import 'dart:convert';
import 'dart:io';
import 'package:core_logic/core_logic.dart';

class CommandProcessor {
  final KeyrxBridge bridge;
  final DeviceRegistryService deviceRegistry;

  CommandProcessor(this.bridge, this.deviceRegistry);

  Future<void> process() async {
    try {
      await for (String? line in stdin
          .transform(utf8.decoder)
          .transform(const LineSplitter())) {
        if (line == null || line.isEmpty) continue;

        try {
          final request = jsonDecode(line) as Map<String, dynamic>;
          final response = await _handleRequest(request);
          stdout.writeln(jsonEncode(response));
        } catch (e) {
          stderr.writeln('Error processing line: $line\n$e');
          stdout.writeln(jsonEncode({
            'error': 'Invalid JSON or processing error: $e',
          }));
        }
      }
    } finally {
      await deviceRegistry.dispose();
      await bridge.dispose();
    }
  }

  Future<Map<String, dynamic>> _handleRequest(
    Map<String, dynamic> request,
  ) async {
    final command = request['command'] as String?;
    final id = request['id'];
    final payload = request['payload'];

    Map<String, dynamic> response = {'id': id};

    try {
      switch (command) {
        case 'version':
          response['result'] = bridge.version;
          break;
        case 'initialize':
          response['result'] = bridge.initialize();
          break;
        case 'isInitialized':
          response['result'] = bridge.isInitialized;
          break;
        case 'listDevices':
          final devices = await deviceRegistry.getDevices();
          response['result'] = devices.map((e) => e.toJson()).toList();
          break;
        case 'addVirtual':
          if (payload is Map<String, dynamic>) {
            final identity = DeviceIdentity.fromJson(payload);
            await deviceRegistry.addVirtualDevice(identity);
            response['result'] = 'ok';
          } else {
            response['error'] = 'Invalid payload for addVirtual';
          }
          break;
        case 'removeVirtual':
          if (payload is String) {
            await deviceRegistry.removeVirtualDevice(payload);
            response['result'] = 'ok';
          } else {
            response['error'] = 'Invalid payload for removeVirtual (expected key)';
          }
          break;
        case 'setLabel':
          if (payload is Map<String, dynamic>) {
            final key = payload['key'] as String;
            final label = payload['label'] as String?;
            final result = await deviceRegistry.setUserLabel(key, label);
            if (result.isSuccess) {
              response['result'] = 'ok';
            } else {
              response['error'] = result.errorMessage;
            }
          } else {
            response['error'] = 'Invalid payload for setLabel';
          }
          break;
        case 'echo':
          response['result'] = payload;
          break;
        case 'exit':
          response['result'] = 'Goodbye';
          stdout.writeln(jsonEncode(response));
          exit(0);
        default:
          response['error'] = 'Unknown command: $command';
      }
    } catch (e) {
      response['error'] = e.toString();
    }

    return response;
  }
}