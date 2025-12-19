import 'dart:convert';
import 'dart:io';
import 'package:core_logic/ffi/bridge.dart';

class CommandProcessor {
  final KeyrxBridge bridge;

  CommandProcessor(this.bridge);

  Future<void> process() async {
    try {
      await for (String? line in stdin.transform(utf8.decoder).transform(const LineSplitter())) {
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
      await bridge.dispose();
    }
  }

  Future<Map<String, dynamic>> _handleRequest(Map<String, dynamic> request) async {
    final command = request['command'] as String?;
    final id = request['id'];

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
          final result = bridge.listRegisteredDevices();
          if (result.hasError) {
            response['error'] = result.errorMessage;
          } else {
            response['result'] = result.data?.map((e) => e.toJson()).toList();
          }
          break;
        case 'echo':
          response['result'] = request['payload'];
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