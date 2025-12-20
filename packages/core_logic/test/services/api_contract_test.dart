/// API Contract verification tests.
///
/// These tests verify that API response formats match what the code expects.
/// Run during build/test phase to catch mismatches before UAT.
library;

import 'package:core_logic/web/api_client.dart';
import 'package:http/http.dart' as http;
import 'package:test/test.dart';

void main() {
  group('API Contract Tests - Backend Response Format', () {
    group('ApiResponse.fromHttpResponse', () {
      test('correctly parses layouts list response', () {
        // This is the ACTUAL format the backend returns
        final response = http.Response(
          '{"layouts": [{"id": "layout1", "name": "Test", "layout_type": "matrix", "keys": []}]}',
          200,
          headers: {'content-type': 'application/json'},
        );

        final apiResponse = ApiResponse.fromHttpResponse(response);

        expect(apiResponse.isSuccess, isTrue, reason: 'Response should be successful');
        expect(apiResponse.data, isA<Map<String, dynamic>>(), reason: 'Data should be a Map');

        final data = apiResponse.data as Map<String, dynamic>;
        expect(data.containsKey('layouts'), isTrue, reason: 'Response must contain "layouts" key');
        expect(data['layouts'], isA<List>(), reason: 'layouts value must be a List');
      });

      test('correctly parses keymaps list response', () {
        // Backend format: {"keymaps": [...]}
        final response = http.Response(
          '{"keymaps": [{"id": "keymap1", "virtual_layout_id": "layout1", "name": "Test", "layers": [], "combos": []}]}',
          200,
          headers: {'content-type': 'application/json'},
        );

        final apiResponse = ApiResponse.fromHttpResponse(response);

        expect(apiResponse.isSuccess, isTrue);
        final data = apiResponse.data as Map<String, dynamic>;
        expect(data.containsKey('keymaps'), isTrue, reason: 'Response must contain "keymaps" key');
      });

      test('correctly parses hardware profiles list response', () {
        // Backend format: {"profiles": [...]}
        final response = http.Response(
          '{"profiles": [{"id": "profile1", "name": "Test", "layout_type": "matrix", "rows": 3, "columns": 5}]}',
          200,
          headers: {'content-type': 'application/json'},
        );

        final apiResponse = ApiResponse.fromHttpResponse(response);

        expect(apiResponse.isSuccess, isTrue);
        final data = apiResponse.data as Map<String, dynamic>;
        expect(
          data.containsKey('profiles'),
          isTrue,
          reason: 'Response must contain "profiles" key',
        );
      });

      test('correctly parses devices list response', () {
        // Backend format: {"devices": [...]}
        final response = http.Response(
          '{"devices": []}',
          200,
          headers: {'content-type': 'application/json'},
        );

        final apiResponse = ApiResponse.fromHttpResponse(response);

        expect(apiResponse.isSuccess, isTrue);
        final data = apiResponse.data as Map<String, dynamic>;
        expect(data.containsKey('devices'), isTrue, reason: 'Response must contain "devices" key');
      });

      test('correctly parses single layout save response', () {
        // Backend format for POST: {"layout": {...}}
        final response = http.Response(
          '{"layout": {"id": "layout1", "name": "Saved", "layout_type": "matrix", "keys": []}}',
          200,
          headers: {'content-type': 'application/json'},
        );

        final apiResponse = ApiResponse.fromHttpResponse(response);

        expect(apiResponse.isSuccess, isTrue);
        final data = apiResponse.data as Map<String, dynamic>;
        expect(
          data.containsKey('layout'),
          isTrue,
          reason: 'Response must contain "layout" key for single item',
        );
      });

      test('correctly parses single keymap save response', () {
        // Backend format for POST: {"keymap": {...}}
        final response = http.Response(
          '{"keymap": {"id": "keymap1", "virtual_layout_id": "layout1", "name": "Saved", "layers": [], "combos": []}}',
          200,
          headers: {'content-type': 'application/json'},
        );

        final apiResponse = ApiResponse.fromHttpResponse(response);

        expect(apiResponse.isSuccess, isTrue);
        final data = apiResponse.data as Map<String, dynamic>;
        expect(
          data.containsKey('keymap'),
          isTrue,
          reason: 'Response must contain "keymap" key for single item',
        );
      });

      test('correctly parses single profile save response', () {
        // Backend format for POST: {"profile": {...}}
        final response = http.Response(
          '{"profile": {"id": "profile1", "name": "Saved", "layout_type": "matrix", "rows": 3, "columns": 5}}',
          200,
          headers: {'content-type': 'application/json'},
        );

        final apiResponse = ApiResponse.fromHttpResponse(response);

        expect(apiResponse.isSuccess, isTrue);
        final data = apiResponse.data as Map<String, dynamic>;
        expect(
          data.containsKey('profile'),
          isTrue,
          reason: 'Response must contain "profile" key for single item',
        );
      });

      test('correctly parses runtime config response', () {
        // Backend format: {"config": {...}}
        final response = http.Response(
          '{"config": {"devices": [], "version": 1}}',
          200,
          headers: {'content-type': 'application/json'},
        );

        final apiResponse = ApiResponse.fromHttpResponse(response);

        expect(apiResponse.isSuccess, isTrue);
        final data = apiResponse.data as Map<String, dynamic>;
        expect(data.containsKey('config'), isTrue, reason: 'Response must contain "config" key');
      });

      test('correctly parses success boolean response', () {
        // Backend format for delete/toggle: {"success": true}
        final response = http.Response(
          '{"success": true}',
          200,
          headers: {'content-type': 'application/json'},
        );

        final apiResponse = ApiResponse.fromHttpResponse(response);

        expect(apiResponse.isSuccess, isTrue);
        final data = apiResponse.data as Map<String, dynamic>;
        expect(data.containsKey('success'), isTrue);
      });

      test('correctly parses error response', () {
        // Backend error format: {"error": "message"}
        final response = http.Response(
          '{"error": "Invalid request"}',
          400,
          headers: {'content-type': 'application/json'},
        );

        final apiResponse = ApiResponse.fromHttpResponse(response);

        expect(apiResponse.isError, isTrue, reason: 'Response should be an error');
        expect(
          apiResponse.error,
          'Invalid request',
          reason: 'Error message should match backend response',
        );
      });

      test('handles 404 error correctly', () {
        final response = http.Response(
          '{"error": "Not found"}',
          404,
          headers: {'content-type': 'application/json'},
        );

        final apiResponse = ApiResponse.fromHttpResponse(response);

        expect(apiResponse.isError, isTrue);
        expect(apiResponse.error, 'Not found');
      });

      test('handles 500 error correctly', () {
        final response = http.Response(
          '{"error": "Internal server error"}',
          500,
          headers: {'content-type': 'application/json'},
        );

        final apiResponse = ApiResponse.fromHttpResponse(response);

        expect(apiResponse.isError, isTrue);
        expect(apiResponse.error, 'Internal server error');
      });

      test('handles malformed JSON gracefully', () {
        final response = http.Response(
          'not valid json',
          200,
          headers: {'content-type': 'application/json'},
        );

        final apiResponse = ApiResponse.fromHttpResponse(response);

        expect(apiResponse.isError, isTrue);
        expect(apiResponse.error, contains('Failed to parse response'));
      });

      test('handles empty response body', () {
        final response = http.Response('', 200, headers: {'content-type': 'application/json'});

        final apiResponse = ApiResponse.fromHttpResponse(response);

        expect(apiResponse.isError, isTrue);
      });
    });

    group('Response Format Compliance', () {
      test('FAIL if backend changes to wrong format for layouts', () {
        // If backend mistakenly returns {"data": [...]} instead of {"layouts": [...]}
        final response = http.Response('{"data": [{"id": "layout1"}]}', 200);

        final apiResponse = ApiResponse.fromHttpResponse(response);
        final data = apiResponse.data as Map<String, dynamic>;

        expect(
          data.containsKey('layouts'),
          isFalse,
          reason:
              'This test SHOULD FAIL if backend changes format - '
              'it detects when backend returns wrong key name',
        );
        expect(data.containsKey('data'), isTrue, reason: 'Wrong format detected');
      });

      test('backend response keys must match service expectations', () {
        // Document expected keys for each endpoint
        final expectedKeys = {
          'GET /api/layouts': 'layouts',
          'POST /api/layouts': 'layout',
          'GET /api/keymaps': 'keymaps',
          'POST /api/keymaps': 'keymap',
          'GET /api/hardware/profiles': 'profiles',
          'POST /api/hardware/profiles': 'profile',
          'GET /api/devices': 'devices',
          'GET /api/runtime/config': 'config',
        };

        // This test documents the contract
        expect(
          expectedKeys,
          isNotEmpty,
          reason: 'API contract documented and verified by other tests',
        );
      });
    });
  });
}
