/// Web API client for HTTP REST communication.
library;

import 'dart:convert';

import 'package:http/http.dart' as http;

import 'websocket_client.dart';

/// HTTP client for communicating with the backend API.
///
/// Provides HTTP methods (GET, POST, PUT, DELETE) and integrates
/// with WebSocketClient for real-time event subscriptions.
class WebApiClient {
  WebApiClient({
    required String baseUrl,
    required WebSocketClient wsClient,
    http.Client? httpClient,
  }) : _baseUrl = baseUrl,
       _wsClient = wsClient,
       _httpClient = httpClient ?? http.Client();

  final String _baseUrl;
  final WebSocketClient _wsClient;
  final http.Client _httpClient;

  /// Send a GET request to the specified path.
  Future<ApiResponse> get(String path) async {
    try {
      final url = Uri.parse('$_baseUrl$path');
      final response = await _httpClient.get(url, headers: {'Content-Type': 'application/json'});
      return ApiResponse.fromHttpResponse(response);
    } catch (e) {
      return ApiResponse(data: null, error: 'HTTP GET failed: $e');
    }
  }

  /// Send a POST request with a JSON body.
  Future<ApiResponse> post(String path, {required Map<String, dynamic> body}) async {
    try {
      final url = Uri.parse('$_baseUrl$path');
      final response = await _httpClient.post(
        url,
        body: jsonEncode(body),
        headers: {'Content-Type': 'application/json'},
      );
      return ApiResponse.fromHttpResponse(response);
    } catch (e) {
      return ApiResponse(data: null, error: 'HTTP POST failed: $e');
    }
  }

  /// Send a PUT request with a JSON body.
  Future<ApiResponse> put(String path, {required Map<String, dynamic> body}) async {
    try {
      final url = Uri.parse('$_baseUrl$path');
      final response = await _httpClient.put(
        url,
        body: jsonEncode(body),
        headers: {'Content-Type': 'application/json'},
      );
      return ApiResponse.fromHttpResponse(response);
    } catch (e) {
      return ApiResponse(data: null, error: 'HTTP PUT failed: $e');
    }
  }

  /// Send a DELETE request.
  Future<ApiResponse> delete(String path) async {
    try {
      final url = Uri.parse('$_baseUrl$path');
      final response = await _httpClient.delete(url, headers: {'Content-Type': 'application/json'});
      return ApiResponse.fromHttpResponse(response);
    } catch (e) {
      return ApiResponse(data: null, error: 'HTTP DELETE failed: $e');
    }
  }

  /// Subscribe to real-time events via WebSocket.
  ///
  /// [event] is the event name (e.g., 'device.updated').
  /// [parser] converts the raw event data to type [T].
  Stream<T> subscribe<T>(String event, T Function(dynamic) parser) {
    return _wsClient.subscribe<T>(event, parser);
  }

  /// Dispose resources.
  void dispose() {
    _wsClient.dispose();
    _httpClient.close();
  }
}

/// Response wrapper for HTTP API calls.
class ApiResponse {
  const ApiResponse({required this.data, this.error});

  /// Create an ApiResponse from an http.Response.
  factory ApiResponse.fromHttpResponse(http.Response response) {
    try {
      final json = jsonDecode(response.body) as Map<String, dynamic>;

      // Success: HTTP 2xx
      if (response.statusCode >= 200 && response.statusCode < 300) {
        // Return the entire JSON body for successful responses
        // Service implementations will extract the specific field they need
        return ApiResponse(data: json);
      }

      // Error: HTTP 4xx/5xx
      return ApiResponse(data: null, error: json['error'] as String? ?? 'Unknown server error');
    } catch (e) {
      // JSON parse error
      return ApiResponse(data: null, error: 'Failed to parse response: $e');
    }
  }

  /// Response data (null on error).
  final dynamic data;

  /// Error message (null on success).
  final String? error;

  /// True if this response represents an error.
  bool get isError => error != null;

  /// True if this response represents success.
  bool get isSuccess => !isError;
}
