import 'package:core_logic/models/daemon_protocol.dart';
import 'package:core_logic/models/generated_models.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:keyrx_ui/services/daemon_service.dart';
import 'package:keyrx_ui/services/generated/ipc_client.dart';
import 'package:mocktail/mocktail.dart';

class MockDaemonService extends Mock implements DaemonService {}

void main() {
  late MockDaemonService mockService;

  setUp(() {
    mockService = MockDaemonService();
  });

  group('Generated IPC Client Tests', () {
    test('BaseClient.version calls correct method and parses result', () async {
      final client = BaseClient(mockService);

      when(
        () => mockService.sendRequest('base.version', {}),
      ).thenAnswer((_) async => '1.0.0');

      final version = await client.version();

      expect(version, '1.0.0');
      verify(() => mockService.sendRequest('base.version', {})).called(1);
    });

    test('EngineClient.startLoop returns Map', () async {
      final client = EngineClient(mockService);
      final response = {'running': true};

      when(
        () => mockService.sendRequest('engine.start_loop', {}),
      ).thenAnswer((_) async => response);

      final result = await client.startLoop();

      expect(result, response);
    });

    test('DiscoveryClient parses events correctly', () {
      final client = DiscoveryClient(mockService);

      const event = JsonRpcNotification(
        jsonrpc: '2.0',
        method: 'discovery.discovery_summary',
        params: {
          'aliases': <String, dynamic>{},
          'captured': 10,
          'cols_per_row': <int>[10, 10],
          'device_id': {'product_id': 1, 'vendor_id': 1},
          'duplicates': <dynamic>[],
          'keymap': <String, dynamic>{},
          'message': 'Done',
          'rows': 2,
          'status': 'Completed',
          'total': 20,
          'unmapped': <int>[],
        },
      );

      when(
        () => mockService.notificationStream,
      ).thenAnswer((_) => Stream.value(event));

      expect(
        client.onDiscoverySummary,
        emits(
          isA<DiscoverySummary>()
              .having((s) => s.captured, 'captured', 10)
              .having((s) => s.status, 'status', SessionStatus.completed),
        ),
      );
    });
  });
}
