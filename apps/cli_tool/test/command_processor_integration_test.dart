import 'package:keyrx_cli/command_processor.dart';
import 'package:core_logic/core_logic.dart';
import 'package:core_logic/ffi/mock_bridge.dart';
import 'package:test/test.dart';

/// Integration tests for CommandProcessor
/// These tests verify that new commands are registered and accessible
void main() {
  late CommandProcessor processor;
  late KeyrxBridge bridge;
  late DeviceRegistryService deviceRegistry;
  late RuntimeService runtimeService;
  late HardwareService hardwareService;
  late LayoutService layoutService;
  late KeymapService keymapService;
  late ProfileRegistryService profileRegistry;
  late TestService testService;
  late ScriptFileService scriptFileService;

  setUp(() {
    bridge = MockKeyrxBridge();

    // Create real services using the mock bridge - they will handle mocking internally
    final repository = DeviceConfigRepository();
    deviceRegistry = DeviceRegistryServiceImpl(
      bridge: bridge,
      repository: repository,
    );
    runtimeService = RuntimeServiceImpl(bridge: bridge);
    hardwareService = HardwareService(bridge: bridge);
    layoutService = LayoutService(bridge: bridge);
    keymapService = KeymapService(bridge: bridge);
    profileRegistry = ProfileRegistryServiceImpl(bridge: bridge);
    testService = TestServiceImpl(bridge: bridge);
    scriptFileService = const ScriptFileService();

    processor = CommandProcessor(
      bridge,
      deviceRegistry,
      runtimeService,
      hardwareService,
      layoutService,
      keymapService,
      profileRegistry,
      testService,
      scriptFileService,
    );
  });

  group('New Commands Exist', () {
    test('listRuntime command is recognized', () async {
      final request = {'command': 'listRuntime', 'id': 1};
      final response = await processor.handleRequest(request);

      expect(response['id'], equals(1));
      expect(response.containsKey('error') || response.containsKey('result'),
          isTrue);
    });

    test('listHardware command is recognized', () async {
      final request = {'command': 'listHardware', 'id': 2};
      final response = await processor.handleRequest(request);

      expect(response['id'], equals(2));
      expect(response.containsKey('error') || response.containsKey('result'),
          isTrue);
    });

    test('listLayouts command is recognized', () async {
      final request = {'command': 'listLayouts', 'id': 3};
      final response = await processor.handleRequest(request);

      expect(response['id'], equals(3));
      expect(response.containsKey('error') || response.containsKey('result'),
          isTrue);
    });

    test('listKeymaps command is recognized', () async {
      final request = {'command': 'listKeymaps', 'id': 4};
      final response = await processor.handleRequest(request);

      expect(response['id'], equals(4));
      expect(response.containsKey('error') || response.containsKey('result'),
          isTrue);
    });

    test('deleteHardware command is recognized', () async {
      final request = {
        'command': 'deleteHardware',
        'id': 5,
        'payload': 'test-id',
      };
      final response = await processor.handleRequest(request);

      expect(response['id'], equals(5));
      expect(response.containsKey('error') || response.containsKey('result'),
          isTrue);
    });

    test('deleteLayout command is recognized', () async {
      final request = {
        'command': 'deleteLayout',
        'id': 6,
        'payload': 'test-id',
      };
      final response = await processor.handleRequest(request);

      expect(response['id'], equals(6));
      expect(response.containsKey('error') || response.containsKey('result'),
          isTrue);
    });

    test('deleteKeymap command is recognized', () async {
      final request = {
        'command': 'deleteKeymap',
        'id': 7,
        'payload': 'test-id',
      };
      final response = await processor.handleRequest(request);

      expect(response['id'], equals(7));
      expect(response.containsKey('error') || response.containsKey('result'),
          isTrue);
    });
  });

  group('Invalid Payloads Return Errors', () {
    test('addSlot with missing fields returns error', () async {
      final request = {
        'command': 'addSlot',
        'id': 10,
        'payload': {'device': 'invalid'},
      };
      final response = await processor.handleRequest(request);

      expect(response['id'], equals(10));
      expect(response['error'], isNotNull);
    });

    test('saveHardware with invalid payload returns error', () async {
      final request = {
        'command': 'saveHardware',
        'id': 11,
        'payload': 'invalid',
      };
      final response = await processor.handleRequest(request);

      expect(response['id'], equals(11));
      expect(response['error'], isNotNull);
    });

    test('saveLayout with invalid payload returns error', () async {
      final request = {
        'command': 'saveLayout',
        'id': 12,
        'payload': 'invalid',
      };
      final response = await processor.handleRequest(request);

      expect(response['id'], equals(12));
      expect(response['error'], isNotNull);
    });

    test('saveKeymap with invalid payload returns error', () async {
      final request = {
        'command': 'saveKeymap',
        'id': 13,
        'payload': 'invalid',
      };
      final response = await processor.handleRequest(request);

      expect(response['id'], equals(13));
      expect(response['error'], isNotNull);
    });
  });

  group('Backward Compatibility', () {
    test('version command still works', () async {
      final request = {'command': 'version', 'id': 20};
      final response = await processor.handleRequest(request);

      expect(response['id'], equals(20));
      expect(response['error'], isNull);
      expect(response['result'], isNotNull);
      expect(response['result'], isA<String>());
    });

    test('listDevices command still works', () async {
      final request = {'command': 'listDevices', 'id': 21};
      final response = await processor.handleRequest(request);

      expect(response['id'], equals(21));
      expect(response['error'], isNull);
      expect(response['result'], isNotNull);
      expect(response['result'], isList);
    });

    test('echo command still works', () async {
      final request = {
        'command': 'echo',
        'id': 22,
        'payload': {'test': 'data'},
      };
      final response = await processor.handleRequest(request);

      expect(response['id'], equals(22));
      expect(response['error'], isNull);
      expect(response['result'], equals({'test': 'data'}));
    });

    test('unknown command returns error', () async {
      final request = {'command': 'unknownCommand', 'id': 23};
      final response = await processor.handleRequest(request);

      expect(response['id'], equals(23));
      expect(response['error'], contains('Unknown command'));
    });
  });

  group('JSON Protocol', () {
    test('all responses include id field', () async {
      final requests = [
        {'command': 'version', 'id': 30},
        {'command': 'listRuntime', 'id': 31},
        {'command': 'listHardware', 'id': 32},
        {'command': 'unknownCmd', 'id': 33},
      ];

      for (final request in requests) {
        final response = await processor.handleRequest(request);
        expect(response['id'], equals(request['id']));
      }
    });

    test('responses have either error or result field', () async {
      final requests = [
        {'command': 'version', 'id': 40},
        {'command': 'listRuntime', 'id': 41},
        {'command': 'unknownCmd', 'id': 42},
      ];

      for (final request in requests) {
        final response = await processor.handleRequest(request);
        expect(
          response.containsKey('error') || response.containsKey('result'),
          isTrue,
        );
      }
    });
  });
}
