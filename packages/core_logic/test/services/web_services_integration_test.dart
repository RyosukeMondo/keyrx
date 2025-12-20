/// Integration tests for web service implementations.
///
/// These tests verify that web services can correctly parse
/// backend API responses and catch format mismatches during build/test phase.
library;

import 'package:core_logic/models/hardware_profile.dart';
import 'package:core_logic/models/keymap.dart';
import 'package:core_logic/models/virtual_layout.dart';
import 'package:core_logic/services/hardware_service_web.dart';
import 'package:core_logic/services/keymap_service_web.dart';
import 'package:core_logic/services/layout_service_web.dart';
import 'package:core_logic/web/api_client.dart';
import 'package:http/http.dart' as http;
import 'package:http/testing.dart';
import 'package:test/test.dart';

void main() {
  group('Web Service API Contract Tests', () {
    group('LayoutServiceWeb', () {
      test('listLayouts parses backend response format correctly', () async {
        // Backend returns: {"layouts": [...]}
        final mockClient = MockClient((request) async {
          expect(request.url.path, '/api/layouts');
          expect(request.method, 'GET');

          return http.Response(
            '{"layouts": [{"id": "layout1", "name": "Test Layout", "layout_type": "matrix", "keys": []}]}',
            200,
            headers: {'content-type': 'application/json'},
          );
        });

        final wsClient = _MockWebSocketClient();
        final apiClient = WebApiClient(
          baseUrl: 'http://test',
          wsClient: wsClient,
        );
        // Inject mock HTTP client
        final service = _TestLayoutServiceWeb(apiClient, mockClient);

        final result = await service.listLayouts();

        expect(result.isSuccess, isTrue);
        expect(result.data, isNotNull);
        expect(result.data!.length, 1);
        expect(result.data![0].id, 'layout1');
        expect(result.data![0].name, 'Test Layout');
      });

      test('saveLayout sends and parses backend format correctly', () async {
        final mockClient = MockClient((request) async {
          expect(request.url.path, '/api/layouts');
          expect(request.method, 'POST');

          // Verify request body format
          final body = request.body;
          expect(body, contains('"id"'));
          expect(body, contains('"name"'));

          // Backend returns: {"layout": {...}}
          return http.Response(
            '{"layout": {"id": "layout1", "name": "Saved Layout", "layout_type": "matrix", "keys": []}}',
            200,
            headers: {'content-type': 'application/json'},
          );
        });

        final wsClient = _MockWebSocketClient();
        final apiClient = WebApiClient(
          baseUrl: 'http://test',
          wsClient: wsClient,
        );
        final service = _TestLayoutServiceWeb(apiClient, mockClient);

        final layout = VirtualLayout(
          id: 'layout1',
          name: 'Test',
          layoutType: LayoutType.matrix,
          keys: [],
        );

        final result = await service.saveLayout(layout);

        expect(result.isSuccess, isTrue);
        expect(result.data, isNotNull);
        expect(result.data!.name, 'Saved Layout');
      });

      test('handles error response format correctly', () async {
        final mockClient = MockClient((request) async {
          // Backend error format: {"error": "message"}
          return http.Response(
            '{"error": "Layout not found"}',
            404,
            headers: {'content-type': 'application/json'},
          );
        });

        final wsClient = _MockWebSocketClient();
        final apiClient = WebApiClient(
          baseUrl: 'http://test',
          wsClient: wsClient,
        );
        final service = _TestLayoutServiceWeb(apiClient, mockClient);

        final result = await service.listLayouts();

        expect(result.isError, isTrue);
        expect(result.errorMessage, 'Layout not found');
      });
    });

    group('HardwareServiceWeb', () {
      test('listProfiles parses backend response format correctly', () async {
        final mockClient = MockClient((request) async {
          expect(request.url.path, '/api/hardware/profiles');
          expect(request.method, 'GET');

          // Backend returns: {"profiles": [...]}
          return http.Response(
            '{"profiles": [{"id": "profile1", "name": "Test Profile", "layout_type": "matrix", "rows": 3, "columns": 5}]}',
            200,
            headers: {'content-type': 'application/json'},
          );
        });

        final wsClient = _MockWebSocketClient();
        final apiClient = WebApiClient(
          baseUrl: 'http://test',
          wsClient: wsClient,
        );
        final service = _TestHardwareServiceWeb(apiClient, mockClient);

        final result = await service.listProfiles();

        expect(result.isSuccess, isTrue);
        expect(result.data, isNotNull);
        expect(result.data!.length, 1);
        expect(result.data![0].id, 'profile1');
      });

      test('saveProfile parses backend response format correctly', () async {
        final mockClient = MockClient((request) async {
          // Backend returns: {"profile": {...}}
          return http.Response(
            '{"profile": {"id": "profile1", "name": "Saved Profile", "layout_type": "matrix", "rows": 3, "columns": 5}}',
            200,
            headers: {'content-type': 'application/json'},
          );
        });

        final wsClient = _MockWebSocketClient();
        final apiClient = WebApiClient(
          baseUrl: 'http://test',
          wsClient: wsClient,
        );
        final service = _TestHardwareServiceWeb(apiClient, mockClient);

        final profile = HardwareProfile(
          id: 'profile1',
          name: 'Test',
          layoutType: LayoutType.matrix,
          rows: 3,
          columns: 5,
        );

        final result = await service.saveProfile(profile);

        expect(result.isSuccess, isTrue);
        expect(result.data!.name, 'Saved Profile');
      });
    });

    group('KeymapServiceWeb', () {
      test('listKeymaps parses backend response format correctly', () async {
        final mockClient = MockClient((request) async {
          expect(request.url.path, '/api/keymaps');
          expect(request.method, 'GET');

          // Backend returns: {"keymaps": [...]}
          return http.Response(
            '{"keymaps": [{"id": "keymap1", "virtual_layout_id": "layout1", "name": "Test Keymap", "layers": [], "combos": []}]}',
            200,
            headers: {'content-type': 'application/json'},
          );
        });

        final wsClient = _MockWebSocketClient();
        final apiClient = WebApiClient(
          baseUrl: 'http://test',
          wsClient: wsClient,
        );
        final service = _TestKeymapServiceWeb(apiClient, mockClient);

        final result = await service.listKeymaps();

        expect(result.isSuccess, isTrue);
        expect(result.data, isNotNull);
        expect(result.data!.length, 1);
        expect(result.data![0].id, 'keymap1');
      });

      test('saveKeymap parses backend response format correctly', () async {
        final mockClient = MockClient((request) async {
          // Backend returns: {"keymap": {...}}
          return http.Response(
            '{"keymap": {"id": "keymap1", "virtual_layout_id": "layout1", "name": "Saved Keymap", "layers": [], "combos": []}}',
            200,
            headers: {'content-type': 'application/json'},
          );
        });

        final wsClient = _MockWebSocketClient();
        final apiClient = WebApiClient(
          baseUrl: 'http://test',
          wsClient: wsClient,
        );
        final service = _TestKeymapServiceWeb(apiClient, mockClient);

        final keymap = Keymap(
          id: 'keymap1',
          virtualLayoutId: 'layout1',
          name: 'Test',
          layers: [],
          combos: [],
        );

        final result = await service.saveKeymap(keymap);

        expect(result.isSuccess, isTrue);
        expect(result.data!.name, 'Saved Keymap');
      });
    });

    group('ApiResponse', () {
      test('parses success response with nested data', () {
        final response = http.Response(
          '{"layouts": [{"id": "1"}]}',
          200,
        );

        final apiResponse = ApiResponse.fromHttpResponse(response);

        expect(apiResponse.isSuccess, isTrue);
        expect(apiResponse.data, isA<Map<String, dynamic>>());
        expect((apiResponse.data as Map)['layouts'], isNotNull);
      });

      test('parses error response correctly', () {
        final response = http.Response(
          '{"error": "Something went wrong"}',
          400,
        );

        final apiResponse = ApiResponse.fromHttpResponse(response);

        expect(apiResponse.isError, isTrue);
        expect(apiResponse.error, 'Something went wrong');
      });

      test('handles malformed JSON gracefully', () {
        final response = http.Response(
          'not valid json',
          200,
        );

        final apiResponse = ApiResponse.fromHttpResponse(response);

        expect(apiResponse.isError, isTrue);
        expect(apiResponse.error, contains('Failed to parse response'));
      });
    });
  });
}

// Test-specific implementations that allow injecting mock HTTP client
class _TestLayoutServiceWeb extends LayoutServiceWeb {
  _TestLayoutServiceWeb(WebApiClient client, this._mockClient)
      : super(client: client);

  final http.Client _mockClient;

  @override
  Future<dynamic> get(String path) async {
    final response = await _mockClient.get(Uri.parse('http://test$path'));
    return ApiResponse.fromHttpResponse(response);
  }

  @override
  Future<dynamic> post(String path, {required Map<String, dynamic> body}) async {
    final response = await _mockClient.post(
      Uri.parse('http://test$path'),
      body: body,
    );
    return ApiResponse.fromHttpResponse(response);
  }
}

class _TestHardwareServiceWeb extends HardwareServiceWeb {
  _TestHardwareServiceWeb(WebApiClient client, this._mockClient)
      : super(client: client);

  final http.Client _mockClient;

  @override
  Future<dynamic> get(String path) async {
    final response = await _mockClient.get(Uri.parse('http://test$path'));
    return ApiResponse.fromHttpResponse(response);
  }

  @override
  Future<dynamic> post(String path, {required Map<String, dynamic> body}) async {
    final response = await _mockClient.post(
      Uri.parse('http://test$path'),
      body: body,
    );
    return ApiResponse.fromHttpResponse(response);
  }
}

class _TestKeymapServiceWeb extends KeymapServiceWeb {
  _TestKeymapServiceWeb(WebApiClient client, this._mockClient)
      : super(client: client);

  final http.Client _mockClient;

  @override
  Future<dynamic> get(String path) async {
    final response = await _mockClient.get(Uri.parse('http://test$path'));
    return ApiResponse.fromHttpResponse(response);
  }

  @override
  Future<dynamic> post(String path, {required Map<String, dynamic> body}) async {
    final response = await _mockClient.post(
      Uri.parse('http://test$path'),
      body: body,
    );
    return ApiResponse.fromHttpResponse(response);
  }
}

class _MockWebSocketClient implements WebSocketClient {
  @override
  Stream<T> subscribe<T>(String event, T Function(dynamic p1) parser) {
    return Stream.empty();
  }

  @override
  Stream<String> get errors => Stream.empty();

  @override
  void dispose() {}
}
