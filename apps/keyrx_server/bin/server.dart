import 'dart:io';
import 'package:args/args.dart';
import 'package:shelf/shelf.dart';
import 'package:shelf/shelf_io.dart' as shelf_io;
import 'package:core_logic/ffi/bridge.dart';
import 'package:core_logic/repositories/device_config_repository.dart';
import 'package:core_logic/services/services.dart';
import 'package:core_logic/services/runtime_service.dart';
import 'package:core_logic/services/keymap_service_ffi.dart';
import 'package:keyrx_server/api/routes.dart';
import 'package:keyrx_server/api/websocket_handler.dart';

void main(List<String> args) async {
  final parser = ArgParser()
    ..addOption('port', abbr: 'p', defaultsTo: '8080', help: 'Server port')
    ..addOption('host', defaultsTo: '0.0.0.0', help: 'Server host')
    ..addFlag('help', abbr: 'h', negatable: false, help: 'Show help');

  final argResults = parser.parse(args);

  if (argResults['help'] as bool) {
    // ignore: avoid_print
    print('KeyRx Backend API Server');
    // ignore: avoid_print
    print('');
    // ignore: avoid_print
    print('Usage: keyrx_server [options]');
    // ignore: avoid_print
    print('');
    // ignore: avoid_print
    print(parser.usage);
    exit(0);
  }

  final port = int.parse(argResults['port'] as String);
  final host = argResults['host'] as String;

  // ignore: avoid_print
  print('Initializing KeyRx services...');

  // Initialize FFI bridge
  final bridge = KeyrxBridge.open();

  // Initialize services
  final deviceConfigRepo = DeviceConfigRepository();
  final deviceRegistryService = DeviceRegistryServiceFFI(
    bridge: bridge,
    repository: deviceConfigRepo,
  );
  final profileRegistryService = ProfileRegistryServiceImpl(bridge: bridge);
  final runtimeService = RuntimeServiceImpl(bridge: bridge);
  final testService = TestServiceImpl(bridge: bridge);
  const scriptFileService = ScriptFileService();

  // Create platform-specific service implementations
  // For now, we use FFI implementations since we're running on desktop
  // In the future, we might want to make these configurable
  final hardwareService = HardwareServiceFFI(bridge: bridge);
  final keymapService = KeymapServiceFFI(bridge: bridge);
  final layoutService = LayoutServiceFFI(bridge: bridge);

  // Create WebSocket handler for real-time events
  final wsHandler = WebSocketHandler(
    bridge: bridge,
    deviceRegistryService: deviceRegistryService,
    hardwareService: hardwareService,
    layoutService: layoutService,
  );

  // Create API routes
  final apiRoutes = ApiRoutes(
    hardwareService: hardwareService,
    keymapService: keymapService,
    layoutService: layoutService,
    deviceRegistryService: deviceRegistryService,
    profileRegistryService: profileRegistryService,
    runtimeService: runtimeService,
    testService: testService,
    scriptFileService: scriptFileService,
    wsHandler: wsHandler,
  );

  // Create middleware pipeline
  final handler = const Pipeline()
      .addMiddleware(logRequests())
      .addMiddleware(_corsMiddleware())
      .addHandler(apiRoutes.router.call);

  // Start server
  final server = await shelf_io.serve(handler, host, port);
  // ignore: avoid_print
  print('KeyRx API Server listening on http://${server.address.host}:${server.port}');
  // ignore: avoid_print
  print('CORS enabled for all origins');

  // Handle shutdown gracefully
  ProcessSignal.sigint.watch().listen((signal) async {
    // ignore: avoid_print
    print('\nShutting down server...');
    await wsHandler.dispose();
    await deviceRegistryService.dispose();
    await profileRegistryService.dispose();
    await testService.dispose();
    hardwareService.dispose();
    layoutService.dispose();
    await server.close();
    // ignore: avoid_print
    print('Server stopped');
    exit(0);
  });
}

/// CORS middleware to allow cross-origin requests from web clients
Middleware _corsMiddleware() {
  return (Handler innerHandler) {
    return (Request request) async {
      // Handle preflight requests
      if (request.method == 'OPTIONS') {
        return Response.ok('', headers: _corsHeaders);
      }

      // Add CORS headers to all responses
      final response = await innerHandler(request);
      return response.change(headers: _corsHeaders);
    };
  };
}

const _corsHeaders = {
  'Access-Control-Allow-Origin': '*',
  'Access-Control-Allow-Methods': 'GET, POST, PUT, DELETE, OPTIONS',
  'Access-Control-Allow-Headers': 'Origin, Content-Type, Accept, Authorization',
  'Access-Control-Max-Age': '86400',
};
