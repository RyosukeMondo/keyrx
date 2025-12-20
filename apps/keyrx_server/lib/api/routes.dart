import 'dart:convert';
import 'package:shelf/shelf.dart';
import 'package:shelf_router/shelf_router.dart';
import 'package:core_logic/services/services.dart';
import 'package:core_logic/models/models.dart';
import 'package:core_logic/services/runtime_service.dart';
import 'package:core_logic/services/keymap_service.dart';
import 'websocket_handler.dart';

class ApiRoutes {
  ApiRoutes({
    required this.hardwareService,
    required this.keymapService,
    required this.layoutService,
    required this.deviceRegistryService,
    required this.profileRegistryService,
    required this.runtimeService,
    required this.testService,
    required this.scriptFileService,
    required this.wsHandler,
  });

  final HardwareService hardwareService;
  final KeymapService keymapService;
  final LayoutService layoutService;
  final DeviceRegistryService deviceRegistryService;
  final ProfileRegistryService profileRegistryService;
  final RuntimeService runtimeService;
  final TestService testService;
  final ScriptFileService scriptFileService;
  final WebSocketHandler wsHandler;

  Router get router {
    final router = Router();

    // WebSocket endpoint for real-time events
    router.get('/api/ws', wsHandler.handler);

    // Hardware Profile endpoints
    router.get('/api/hardware/profiles', _listHardwareProfiles);
    router.post('/api/hardware/profiles', _saveHardwareProfile);
    router.delete('/api/hardware/profiles/<id>', _deleteHardwareProfile);

    // Keymap endpoints
    router.get('/api/keymaps', _listKeymaps);
    router.post('/api/keymaps', _saveKeymap);
    router.delete('/api/keymaps/<id>', _deleteKeymap);

    // Layout endpoints
    router.get('/api/layouts', _listLayouts);
    router.post('/api/layouts', _saveLayout);
    router.delete('/api/layouts/<id>', _deleteLayout);

    // Device Registry endpoints
    router.get('/api/devices', _getDevices);
    router.post('/api/devices/virtual', _addVirtualDevice);
    router.delete('/api/devices/virtual/<key>', _removeVirtualDevice);
    router.post('/api/devices/<key>/toggle', _toggleDevice);
    router.post('/api/devices/<key>/toggle-remap', _toggleRemap);
    router.post('/api/devices/<key>/assign-profile', _assignProfile);
    router.post('/api/devices/<key>/set-label', _setUserLabel);
    router.post('/api/devices/refresh', _refreshDevices);

    // Profile Registry endpoints
    router.get('/api/profiles', _listProfiles);
    router.get('/api/profiles/<id>', _getProfile);
    router.post('/api/profiles', _saveProfile);
    router.delete('/api/profiles/<id>', _deleteProfile);
    router.get('/api/profiles/compatible/<layoutType>', _findCompatibleProfiles);
    router.post('/api/profiles/refresh', _refreshProfiles);

    // Runtime Service endpoints
    router.get('/api/runtime/config', _getRuntimeConfig);
    router.post('/api/runtime/slots/add', _addSlot);
    router.post('/api/runtime/slots/remove', _removeSlot);
    router.post('/api/runtime/slots/reorder', _reorderSlot);
    router.post('/api/runtime/slots/set-active', _setSlotActive);

    // Test Service endpoints
    router.post('/api/tests/discover', _discoverTests);
    router.post('/api/tests/run', _runTests);

    // Script File Service endpoints
    router.post('/api/scripts/save', _saveScript);
    router.post('/api/scripts/load', _loadScript);

    return router;
  }

  // Hardware Profile handlers
  Future<Response> _listHardwareProfiles(Request request) async {
    final result = await hardwareService.listProfiles();
    if (result.isSuccess) {
      return _jsonResponse(
        {'profiles': result.data!.map((HardwareProfile p) => p.toJson()).toList()},
      );
    }
    return _errorResponse(result.errorMessage ?? 'Failed to list profiles');
  }

  Future<Response> _saveHardwareProfile(Request request) async {
    final body = await _parseBody(request);
    if (body == null) {
      return _errorResponse('Invalid request body');
    }

    try {
      final profile = HardwareProfile.fromJson(body);
      final result = await hardwareService.saveProfile(profile);
      if (result.isSuccess) {
        return _jsonResponse({'profile': result.data!.toJson()});
      }
      return _errorResponse(result.errorMessage ?? 'Failed to save profile');
    } catch (e) {
      return _errorResponse('Invalid profile data: $e');
    }
  }

  Future<Response> _deleteHardwareProfile(Request request, String id) async {
    final result = await hardwareService.deleteProfile(id);
    if (result.isSuccess) {
      return _jsonResponse({'success': true});
    }
    return _errorResponse(result.errorMessage ?? 'Failed to delete profile');
  }

  // Keymap handlers
  Future<Response> _listKeymaps(Request request) async {
    final result = await keymapService.listKeymaps();
    if (result.isSuccess) {
      return _jsonResponse(
        {'keymaps': result.data!.map((Keymap k) => k.toJson()).toList()},
      );
    }
    return _errorResponse(result.errorMessage ?? 'Failed to list keymaps');
  }

  Future<Response> _saveKeymap(Request request) async {
    final body = await _parseBody(request);
    if (body == null) {
      return _errorResponse('Invalid request body');
    }

    try {
      final keymap = Keymap.fromJson(body);
      final result = await keymapService.saveKeymap(keymap);
      if (result.isSuccess) {
        return _jsonResponse({'keymap': result.data!.toJson()});
      }
      return _errorResponse(result.errorMessage ?? 'Failed to save keymap');
    } catch (e) {
      return _errorResponse('Invalid keymap data: $e');
    }
  }

  Future<Response> _deleteKeymap(Request request, String id) async {
    final result = await keymapService.deleteKeymap(id);
    if (result.isSuccess) {
      return _jsonResponse({'success': true});
    }
    return _errorResponse(result.errorMessage ?? 'Failed to delete keymap');
  }

  // Layout handlers
  Future<Response> _listLayouts(Request request) async {
    final result = await layoutService.listLayouts();
    if (result.isSuccess) {
      return _jsonResponse(
        {'layouts': result.data!.map((VirtualLayout l) => l.toJson()).toList()},
      );
    }
    return _errorResponse(result.errorMessage ?? 'Failed to list layouts');
  }

  Future<Response> _saveLayout(Request request) async {
    final body = await _parseBody(request);
    if (body == null) {
      return _errorResponse('Invalid request body');
    }

    try {
      final layout = VirtualLayout.fromJson(body);
      final result = await layoutService.saveLayout(layout);
      if (result.isSuccess) {
        return _jsonResponse({'layout': result.data!.toJson()});
      }
      return _errorResponse(result.errorMessage ?? 'Failed to save layout');
    } catch (e) {
      return _errorResponse('Invalid layout data: $e');
    }
  }

  Future<Response> _deleteLayout(Request request, String id) async {
    final result = await layoutService.deleteLayout(id);
    if (result.isSuccess) {
      return _jsonResponse({'success': true});
    }
    return _errorResponse(result.errorMessage ?? 'Failed to delete layout');
  }

  // Device Registry handlers
  Future<Response> _getDevices(Request request) async {
    final devices = await deviceRegistryService.getDevices();
    return _jsonResponse(
      {'devices': devices.map((DeviceState d) => d.toJson()).toList()},
    );
  }

  Future<Response> _addVirtualDevice(Request request) async {
    final body = await _parseBody(request);
    if (body == null) {
      return _errorResponse('Invalid request body');
    }

    try {
      final identity = DeviceIdentity.fromJson(body);
      await deviceRegistryService.addVirtualDevice(identity);
      return _jsonResponse({'success': true});
    } catch (e) {
      return _errorResponse('Invalid device identity: $e');
    }
  }

  Future<Response> _removeVirtualDevice(Request request, String key) async {
    await deviceRegistryService.removeVirtualDevice(key);
    return _jsonResponse({'success': true});
  }

  Future<Response> _toggleDevice(Request request, String key) async {
    final body = await _parseBody(request);
    if (body == null || body['enabled'] == null) {
      return _errorResponse('Missing enabled parameter');
    }

    final result = await deviceRegistryService.toggleDevice(
      key,
      body['enabled'] as bool,
    );
    if (result.isSuccess) {
      return _jsonResponse({'success': true});
    }
    return _errorResponse(result.errorMessage ?? 'Failed to toggle device');
  }

  Future<Response> _toggleRemap(Request request, String key) async {
    final body = await _parseBody(request);
    if (body == null || body['enabled'] == null) {
      return _errorResponse('Missing enabled parameter');
    }

    final result = await deviceRegistryService.toggleRemap(
      key,
      body['enabled'] as bool,
    );
    if (result.isSuccess) {
      return _jsonResponse({'success': true});
    }
    return _errorResponse(result.errorMessage ?? 'Failed to toggle remap');
  }

  Future<Response> _assignProfile(Request request, String key) async {
    final body = await _parseBody(request);
    if (body == null || body['profileId'] == null) {
      return _errorResponse('Missing profileId parameter');
    }

    final result = await deviceRegistryService.assignProfile(
      key,
      body['profileId'] as String,
    );
    if (result.isSuccess) {
      return _jsonResponse({'success': true});
    }
    return _errorResponse(result.errorMessage ?? 'Failed to assign profile');
  }

  Future<Response> _setUserLabel(Request request, String key) async {
    final body = await _parseBody(request);
    if (body == null) {
      return _errorResponse('Invalid request body');
    }

    final result = await deviceRegistryService.setUserLabel(
      key,
      body['label'] as String?,
    );
    if (result.isSuccess) {
      return _jsonResponse({'success': true});
    }
    return _errorResponse(result.errorMessage ?? 'Failed to set user label');
  }

  Future<Response> _refreshDevices(Request request) async {
    final devices = await deviceRegistryService.refresh();
    return _jsonResponse(
      {'devices': devices.map((DeviceState d) => d.toJson()).toList()},
    );
  }

  // Profile Registry handlers
  Future<Response> _listProfiles(Request request) async {
    final profiles = await profileRegistryService.listProfiles();
    return _jsonResponse({'profiles': profiles});
  }

  Future<Response> _getProfile(Request request, String id) async {
    final profile = await profileRegistryService.getProfile(id);
    if (profile != null) {
      return _jsonResponse({'profile': profile.toJson()});
    }
    return _errorResponse('Profile not found', statusCode: 404);
  }

  Future<Response> _saveProfile(Request request) async {
    final body = await _parseBody(request);
    if (body == null) {
      return _errorResponse('Invalid request body');
    }

    try {
      final profile = Profile.fromJson(body);
      final result = await profileRegistryService.saveProfile(profile);
      if (result.success) {
        return _jsonResponse({'success': true});
      }
      return _errorResponse(result.errorMessage ?? 'Failed to save profile');
    } catch (e) {
      return _errorResponse('Invalid profile data: $e');
    }
  }

  Future<Response> _deleteProfile(Request request, String id) async {
    final result = await profileRegistryService.deleteProfile(id);
    if (result.success) {
      return _jsonResponse({'success': true});
    }
    return _errorResponse(result.errorMessage ?? 'Failed to delete profile');
  }

  Future<Response> _findCompatibleProfiles(
    Request request,
    String layoutType,
  ) async {
    try {
      final layoutTypeEnum = LayoutType.values.firstWhere(
        (e) => e.name == layoutType,
        orElse: () => throw ArgumentError('Invalid layout type'),
      );
      final profiles = await profileRegistryService.findCompatibleProfiles(
        layoutTypeEnum,
      );
      return _jsonResponse(
        {'profiles': profiles.map((Profile p) => p.toJson()).toList()},
      );
    } catch (e) {
      return _errorResponse('Invalid layout type: $e');
    }
  }

  Future<Response> _refreshProfiles(Request request) async {
    final profiles = await profileRegistryService.refresh();
    return _jsonResponse({'profiles': profiles});
  }

  // Runtime Service handlers
  Future<Response> _getRuntimeConfig(Request request) async {
    final result = await runtimeService.getConfig();
    if (result.isSuccess) {
      return _jsonResponse({'config': result.data!.toJson()});
    }
    return _errorResponse(result.errorMessage ?? 'Failed to get config');
  }

  Future<Response> _addSlot(Request request) async {
    final body = await _parseBody(request);
    if (body == null || body['device'] == null || body['slot'] == null) {
      return _errorResponse('Missing device or slot parameter');
    }

    try {
      final device = DeviceInstanceId.fromJson(body['device'] as Map<String, dynamic>);
      final slot = ProfileSlot.fromJson(body['slot'] as Map<String, dynamic>);
      final result = await runtimeService.addSlot(device, slot);
      if (result.isSuccess) {
        return _jsonResponse({'config': result.data!.toJson()});
      }
      return _errorResponse(result.errorMessage ?? 'Failed to add slot');
    } catch (e) {
      return _errorResponse('Invalid device or slot data: $e');
    }
  }

  Future<Response> _removeSlot(Request request) async {
    final body = await _parseBody(request);
    if (body == null || body['device'] == null || body['slotId'] == null) {
      return _errorResponse('Missing device or slotId parameter');
    }

    try {
      final device = DeviceInstanceId.fromJson(body['device'] as Map<String, dynamic>);
      final result = await runtimeService.removeSlot(
        device,
        body['slotId'] as String,
      );
      if (result.isSuccess) {
        return _jsonResponse({'config': result.data!.toJson()});
      }
      return _errorResponse(result.errorMessage ?? 'Failed to remove slot');
    } catch (e) {
      return _errorResponse('Invalid device data: $e');
    }
  }

  Future<Response> _reorderSlot(Request request) async {
    final body = await _parseBody(request);
    if (body == null ||
        body['device'] == null ||
        body['slotId'] == null ||
        body['priority'] == null) {
      return _errorResponse('Missing device, slotId, or priority parameter');
    }

    try {
      final device = DeviceInstanceId.fromJson(body['device'] as Map<String, dynamic>);
      final result = await runtimeService.reorderSlot(
        device,
        body['slotId'] as String,
        body['priority'] as int,
      );
      if (result.isSuccess) {
        return _jsonResponse({'config': result.data!.toJson()});
      }
      return _errorResponse(result.errorMessage ?? 'Failed to reorder slot');
    } catch (e) {
      return _errorResponse('Invalid request data: $e');
    }
  }

  Future<Response> _setSlotActive(Request request) async {
    final body = await _parseBody(request);
    if (body == null ||
        body['device'] == null ||
        body['slotId'] == null ||
        body['active'] == null) {
      return _errorResponse('Missing device, slotId, or active parameter');
    }

    try {
      final device = DeviceInstanceId.fromJson(body['device'] as Map<String, dynamic>);
      final result = await runtimeService.setSlotActive(
        device,
        body['slotId'] as String,
        body['active'] as bool,
      );
      if (result.isSuccess) {
        return _jsonResponse({'config': result.data!.toJson()});
      }
      return _errorResponse(result.errorMessage ?? 'Failed to set slot active');
    } catch (e) {
      return _errorResponse('Invalid request data: $e');
    }
  }

  // Test Service handlers
  Future<Response> _discoverTests(Request request) async {
    final body = await _parseBody(request);
    if (body == null || body['scriptPath'] == null) {
      return _errorResponse('Missing scriptPath parameter');
    }

    final result = await testService.discoverTests(body['scriptPath'] as String);
    if (result.hasError) {
      return _errorResponse(result.errorMessage!);
    }

    return _jsonResponse({
      'tests': result.tests
          .map((t) => {
                'name': t.name,
                'file': t.file,
                'line': t.line,
              })
          .toList(),
    });
  }

  Future<Response> _runTests(Request request) async {
    final body = await _parseBody(request);
    if (body == null || body['scriptPath'] == null) {
      return _errorResponse('Missing scriptPath parameter');
    }

    final result = await testService.runTests(
      body['scriptPath'] as String,
      filter: body['filter'] as String?,
    );
    if (result.hasError) {
      return _errorResponse(result.errorMessage!);
    }

    return _jsonResponse({
      'total': result.total,
      'passed': result.passed,
      'failed': result.failed,
      'durationMs': result.durationMs,
      'results': result.results
          .map((r) => {
                'name': r.name,
                'passed': r.passed,
                'error': r.error,
                'durationMs': r.durationMs,
              })
          .toList(),
    });
  }

  // Script File Service handlers
  Future<Response> _saveScript(Request request) async {
    final body = await _parseBody(request);
    if (body == null || body['path'] == null || body['content'] == null) {
      return _errorResponse('Missing path or content parameter');
    }

    final result = await scriptFileService.saveScript(
      body['path'] as String,
      body['content'] as String,
    );
    if (result.success) {
      return _jsonResponse({'success': true});
    }
    return _errorResponse(result.errorMessage ?? 'Failed to save script');
  }

  Future<Response> _loadScript(Request request) async {
    final body = await _parseBody(request);
    if (body == null || body['path'] == null) {
      return _errorResponse('Missing path parameter');
    }

    final content = await scriptFileService.loadScript(body['path'] as String);
    if (content != null) {
      return _jsonResponse({'content': content});
    }
    return _errorResponse('Script not found', statusCode: 404);
  }

  // Helper methods
  Future<Map<String, dynamic>?> _parseBody(Request request) async {
    try {
      final body = await request.readAsString();
      if (body.isEmpty) {
        return null;
      }
      return jsonDecode(body) as Map<String, dynamic>;
    } catch (e) {
      return null;
    }
  }

  Response _jsonResponse(Map<String, dynamic> data) {
    return Response.ok(
      jsonEncode(data),
      headers: {'Content-Type': 'application/json'},
    );
  }

  Response _errorResponse(String message, {int statusCode = 400}) {
    return Response(
      statusCode,
      body: jsonEncode({'error': message}),
      headers: {'Content-Type': 'application/json'},
    );
  }
}
