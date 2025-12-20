import 'dart:io';
import 'package:args/args.dart';
import 'package:core_logic/core_logic.dart';
import 'package:core_logic/ffi/library_loader.dart';
import 'package:keyrx_cli/command_processor.dart';

Future<void> main(List<String> arguments) async {
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
      // Set override if provided
      if (results['lib-path'] != null) {
        LibraryLoader.overridePath = results['lib-path'];
      }
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

    // Set override if provided
    if (results['lib-path'] != null) {
      LibraryLoader.overridePath = results['lib-path'];
    }

    final bridge = KeyrxBridge.open();
    if (bridge.loadFailure != null) {
      stderr.writeln('Failed to load bridge: ${bridge.loadFailure}');
      exit(1);
    }

    if (results['config-root'] != null) {
      bridge.setConfigRoot(results['config-root']);
    }

    final repository = DeviceConfigRepository(configDir: results['config-root']);
    final deviceService =
        DeviceRegistryServiceFFI(bridge: bridge, repository: repository);
    final runtimeService = RuntimeServiceImpl(bridge: bridge);
    final hardwareService = HardwareServiceFFI(bridge: bridge);
    final layoutService = LayoutServiceFFI(bridge: bridge);
    final keymapService = KeymapServiceFFI(bridge: bridge);
    final profileRegistry = ProfileRegistryServiceFFI(bridge: bridge);
    final testService = TestServiceImpl(bridge: bridge);
    final scriptFileService = const ScriptFileService();

    final processor = CommandProcessor(
      bridge,
      deviceService,
      runtimeService,
      hardwareService,
      layoutService,
      keymapService,
      profileRegistry,
      testService,
      scriptFileService,
    );
    await processor.process();
    
  } catch (e) {
    stderr.writeln('Error: $e');
    exit(1);
  }
}
