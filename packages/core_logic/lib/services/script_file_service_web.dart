/// Web implementation of ScriptFileService using HTTP API.
library;

import '../web/api_client.dart';
import 'script_file_service.dart';

/// Web-based implementation of ScriptFileService.
///
/// Uses HTTP API endpoints to communicate with backend server for script file operations.
class ScriptFileServiceWeb implements ScriptFileService {
  const ScriptFileServiceWeb({required WebApiClient client}) : _client = client;

  final WebApiClient _client;

  @override
  Future<ScriptFileResult> saveScript(String path, String content) async {
    final response = await _client.post(
      '/api/scripts/save',
      body: {
        'path': path,
        'content': content,
      },
    );

    if (response.isError) {
      return ScriptFileResult(
        success: false,
        errorMessage: response.error,
      );
    }

    return const ScriptFileResult(success: true);
  }

  @override
  Future<String?> loadScript(String path) async {
    final response = await _client.post(
      '/api/scripts/load',
      body: {'path': path},
    );

    if (response.isError) {
      return null;
    }

    try {
      final json = response.data as Map<String, dynamic>;
      return json['content'] as String?;
    } catch (e) {
      return null;
    }
  }
}
