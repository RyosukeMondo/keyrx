import 'dart:io';
import 'package:args/args.dart';
import 'package:core_logic/ffi/bridge.dart';

void main(List<String> arguments) {
  final parser = ArgParser()
    ..addFlag('help', abbr: 'h', negatable: false, help: 'Show this help.')
    ..addFlag('version', negatable: false, help: 'Show version information.')
    ..addOption('config-root',
        abbr: 'c', help: 'Path to the configuration root directory.')
    ..addOption('lib-path', abbr: 'l', help: 'Explicit path to libkeyrx_core.so');

  try {
    final results = parser.parse(arguments);

    if (results['help']) {
      print('KeyRX CLI Harness');
      print('Usage: keyrx_cli [options]');
      print(parser.usage);
      exit(0);
    }

    if (results['version']) {
      print('KeyRX CLI v1.0.0');
      // If bridge can be opened, try to get core version too
      try {
        final bridge = KeyrxBridge.open();
        if (bridge.loadFailure == null) {
          print('Core Version: ${bridge.version}');
        } else {
          print('Core Version: unavailable (${bridge.loadFailure})');
        }
      } catch (e) {
        print('Core Version: error ($e)');
      }
      exit(0);
    }

    print('KeyRX CLI started. Use --help for usage.');
    
    // Future: Task 6 will implement the command processor here.
    
  } catch (e) {
    stderr.writeln('Error: $e');
    exit(1);
  }
}