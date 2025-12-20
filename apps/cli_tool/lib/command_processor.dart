import 'dart:convert';
import 'dart:io';
import 'package:core_logic/core_logic.dart';

class CommandProcessor {
  final KeyrxBridge bridge;
  final DeviceRegistryService deviceRegistry;
  final RuntimeService runtimeService;
  final HardwareService hardwareService;
  final LayoutService layoutService;
  final KeymapService keymapService;
  final ProfileRegistryService profileRegistry;
  final TestService testService;
  final ScriptFileService scriptFileService;

  CommandProcessor(
    this.bridge,
    this.deviceRegistry,
    this.runtimeService,
    this.hardwareService,
    this.layoutService,
    this.keymapService,
    this.profileRegistry,
    this.testService,
    this.scriptFileService,
  );

  Future<void> process() async {
    try {
      await for (String? line in stdin
          .transform(utf8.decoder)
          .transform(const LineSplitter())) {
        if (line == null || line.isEmpty) continue;

        try {
          final request = jsonDecode(line) as Map<String, dynamic>;
          final response = await handleRequest(request);
          stdout.writeln(jsonEncode(response));
        } catch (e) {
          stderr.writeln('Error processing line: $line\n$e');
          stdout.writeln(jsonEncode({
            'error': 'Invalid JSON or processing error: $e',
          }));
        }
      }
    } finally {
      await deviceRegistry.dispose();
      await bridge.dispose();
    }
  }

  Future<Map<String, dynamic>> handleRequest(
    Map<String, dynamic> request,
  ) async {
    final command = request['command'] as String?;
    final id = request['id'];
    final payload = request['payload'];

    Map<String, dynamic> response = {'id': id};

    try {
      switch (command) {
        case 'version':
          response['result'] = bridge.version;
          break;
        case 'initialize':
          response['result'] = bridge.initialize();
          break;
        case 'isInitialized':
          response['result'] = bridge.isInitialized;
          break;
        case 'listDevices':
          final devices = await deviceRegistry.getDevices();
          response['result'] = devices.map((e) => e.toJson()).toList();
          break;
        case 'addVirtual':
          if (payload is Map<String, dynamic>) {
            final identity = DeviceIdentity.fromJson(payload);
            await deviceRegistry.addVirtualDevice(identity);
            response['result'] = 'ok';
          } else {
            response['error'] = 'Invalid payload for addVirtual';
          }
          break;
        case 'removeVirtual':
          if (payload is String) {
            await deviceRegistry.removeVirtualDevice(payload);
            response['result'] = 'ok';
          } else {
            response['error'] = 'Invalid payload for removeVirtual (expected key)';
          }
          break;
        case 'setLabel':
          if (payload is Map<String, dynamic>) {
            final key = payload['key'] as String;
            final label = payload['label'] as String?;
            final result = await deviceRegistry.setUserLabel(key, label);
            if (result.isSuccess) {
              response['result'] = 'ok';
            } else {
              response['error'] = result.errorMessage;
            }
          } else {
            response['error'] = 'Invalid payload for setLabel';
          }
          break;
        case 'listRuntime':
          final result = await runtimeService.getConfig();
          if (result.isSuccess) {
            response['result'] = result.data?.toJson();
          } else {
            response['error'] = result.errorMessage;
          }
          break;
        case 'addSlot':
          if (payload is Map<String, dynamic>) {
            final deviceJson = payload['device'] as Map<String, dynamic>?;
            final slotJson = payload['slot'] as Map<String, dynamic>?;

            if (deviceJson == null || slotJson == null) {
              response['error'] = 'Invalid payload for addSlot: requires device and slot fields';
            } else {
              try {
                final device = DeviceInstanceId.fromJson(deviceJson);
                final slot = ProfileSlot.fromJson(slotJson);
                final result = await runtimeService.addSlot(device, slot);
                if (result.isSuccess) {
                  response['result'] = result.data?.toJson();
                } else {
                  response['error'] = result.errorMessage;
                }
              } catch (e) {
                response['error'] = 'Failed to parse addSlot payload: $e';
              }
            }
          } else {
            response['error'] = 'Invalid payload for addSlot';
          }
          break;
        case 'removeSlot':
          if (payload is Map<String, dynamic>) {
            final deviceJson = payload['device'] as Map<String, dynamic>?;
            final slotId = payload['slot_id'] as String?;

            if (deviceJson == null || slotId == null) {
              response['error'] = 'Invalid payload for removeSlot: requires device and slot_id fields';
            } else {
              try {
                final device = DeviceInstanceId.fromJson(deviceJson);
                final result = await runtimeService.removeSlot(device, slotId);
                if (result.isSuccess) {
                  response['result'] = result.data?.toJson();
                } else {
                  response['error'] = result.errorMessage;
                }
              } catch (e) {
                response['error'] = 'Failed to parse removeSlot payload: $e';
              }
            }
          } else {
            response['error'] = 'Invalid payload for removeSlot';
          }
          break;
        case 'activateSlot':
          if (payload is Map<String, dynamic>) {
            final deviceJson = payload['device'] as Map<String, dynamic>?;
            final slotId = payload['slot_id'] as String?;
            final active = payload['active'] as bool?;

            if (deviceJson == null || slotId == null || active == null) {
              response['error'] = 'Invalid payload for activateSlot: requires device, slot_id, and active fields';
            } else {
              try {
                final device = DeviceInstanceId.fromJson(deviceJson);
                final result = await runtimeService.setSlotActive(device, slotId, active);
                if (result.isSuccess) {
                  response['result'] = result.data?.toJson();
                } else {
                  response['error'] = result.errorMessage;
                }
              } catch (e) {
                response['error'] = 'Failed to parse activateSlot payload: $e';
              }
            }
          } else {
            response['error'] = 'Invalid payload for activateSlot';
          }
          break;
        case 'listHardware':
          final result = await hardwareService.listProfiles();
          if (result.isSuccess) {
            response['result'] = result.data?.map((p) => p.toJson()).toList();
          } else {
            response['error'] = result.errorMessage;
          }
          break;
        case 'saveHardware':
          if (payload is Map<String, dynamic>) {
            try {
              final profile = HardwareProfile.fromJson(payload);
              final result = await hardwareService.saveProfile(profile);
              if (result.isSuccess) {
                response['result'] = result.data?.toJson();
              } else {
                response['error'] = result.errorMessage;
              }
            } catch (e) {
              response['error'] = 'Failed to parse saveHardware payload: $e';
            }
          } else {
            response['error'] = 'Invalid payload for saveHardware';
          }
          break;
        case 'deleteHardware':
          if (payload is String) {
            final result = await hardwareService.deleteProfile(payload);
            if (result.isSuccess) {
              response['result'] = 'ok';
            } else {
              response['error'] = result.errorMessage;
            }
          } else {
            response['error'] = 'Invalid payload for deleteHardware (expected profile id)';
          }
          break;
        case 'listLayouts':
          final result = await layoutService.listLayouts();
          if (result.isSuccess) {
            response['result'] = result.data?.map((l) => l.toJson()).toList();
          } else {
            response['error'] = result.errorMessage;
          }
          break;
        case 'saveLayout':
          if (payload is Map<String, dynamic>) {
            try {
              final layout = VirtualLayout.fromJson(payload);
              final result = await layoutService.saveLayout(layout);
              if (result.isSuccess) {
                response['result'] = result.data?.toJson();
              } else {
                response['error'] = result.errorMessage;
              }
            } catch (e) {
              response['error'] = 'Failed to parse saveLayout payload: $e';
            }
          } else {
            response['error'] = 'Invalid payload for saveLayout';
          }
          break;
        case 'deleteLayout':
          if (payload is String) {
            final result = await layoutService.deleteLayout(payload);
            if (result.isSuccess) {
              response['result'] = 'ok';
            } else {
              response['error'] = result.errorMessage;
            }
          } else {
            response['error'] = 'Invalid payload for deleteLayout (expected layout id)';
          }
          break;
        case 'listKeymaps':
          final result = await keymapService.listKeymaps();
          if (result.isSuccess) {
            response['result'] = result.data?.map((k) => k.toJson()).toList();
          } else {
            response['error'] = result.errorMessage;
          }
          break;
        case 'saveKeymap':
          if (payload is Map<String, dynamic>) {
            try {
              final keymap = Keymap.fromJson(payload);
              final result = await keymapService.saveKeymap(keymap);
              if (result.isSuccess) {
                response['result'] = result.data?.toJson();
              } else {
                response['error'] = result.errorMessage;
              }
            } catch (e) {
              response['error'] = 'Failed to parse saveKeymap payload: $e';
            }
          } else {
            response['error'] = 'Invalid payload for saveKeymap';
          }
          break;
        case 'deleteKeymap':
          if (payload is String) {
            final result = await keymapService.deleteKeymap(payload);
            if (result.isSuccess) {
              response['result'] = 'ok';
            } else {
              response['error'] = result.errorMessage;
            }
          } else {
            response['error'] = 'Invalid payload for deleteKeymap (expected keymap id)';
          }
          break;
        case 'listProfiles':
          final profileIds = await profileRegistry.listProfiles();
          response['result'] = profileIds;
          break;
        case 'getProfile':
          if (payload is String) {
            final profile = await profileRegistry.getProfile(payload);
            if (profile != null) {
              response['result'] = profile.toJson();
            } else {
              response['error'] = 'Profile not found: $payload';
            }
          } else {
            response['error'] = 'Invalid payload for getProfile (expected profile id)';
          }
          break;
        case 'saveProfile':
          if (payload is Map<String, dynamic>) {
            try {
              final profile = Profile.fromJson(payload);
              final result = await profileRegistry.saveProfile(profile);
              if (result.success) {
                response['result'] = 'ok';
              } else {
                response['error'] = result.errorMessage;
              }
            } catch (e) {
              response['error'] = 'Failed to parse saveProfile payload: $e';
            }
          } else {
            response['error'] = 'Invalid payload for saveProfile';
          }
          break;
        case 'deleteProfile':
          if (payload is String) {
            final result = await profileRegistry.deleteProfile(payload);
            if (result.success) {
              response['result'] = 'ok';
            } else {
              response['error'] = result.errorMessage;
            }
          } else {
            response['error'] = 'Invalid payload for deleteProfile (expected profile id)';
          }
          break;
        case 'findCompatibleProfiles':
          if (payload is String) {
            LayoutType? layoutType;
            switch (payload.toLowerCase()) {
              case 'standard':
                layoutType = LayoutType.standard;
                break;
              case 'matrix':
                layoutType = LayoutType.matrix;
                break;
              case 'split':
                layoutType = LayoutType.split;
                break;
              default:
                response['error'] = 'Invalid layout type: $payload (must be standard, matrix, or split)';
            }
            if (layoutType != null) {
              final profiles = await profileRegistry.findCompatibleProfiles(layoutType);
              response['result'] = profiles.map((p) => p.toJson()).toList();
            }
          } else {
            response['error'] = 'Invalid payload for findCompatibleProfiles (expected layout type string)';
          }
          break;
        case 'validateScript':
          if (payload is String) {
            final result = await testService.discoverTests(payload);
            if (result.hasError) {
              response['error'] = result.errorMessage;
            } else {
              response['result'] = {
                'tests': result.tests.map((t) => {
                  'name': t.name,
                  'file': t.file,
                  'line': t.line,
                }).toList(),
              };
            }
          } else {
            response['error'] = 'Invalid payload for validateScript (expected script path)';
          }
          break;
        case 'runTests':
          if (payload is Map<String, dynamic>) {
            final scriptPath = payload['script'] as String?;
            final filter = payload['filter'] as String?;

            if (scriptPath == null) {
              response['error'] = 'Invalid payload for runTests: requires script field';
            } else {
              final result = await testService.runTests(scriptPath, filter: filter);
              if (result.hasError) {
                response['error'] = result.errorMessage;
              } else {
                response['result'] = {
                  'total': result.total,
                  'passed': result.passed,
                  'failed': result.failed,
                  'durationMs': result.durationMs,
                  'results': result.results.map((r) => {
                    'name': r.name,
                    'passed': r.passed,
                    'error': r.error,
                    'durationMs': r.durationMs,
                  }).toList(),
                };
              }
            }
          } else {
            response['error'] = 'Invalid payload for runTests (expected object with script field)';
          }
          break;
        case 'testScript':
          if (payload is Map<String, dynamic>) {
            final path = payload['path'] as String?;
            final content = payload['content'] as String?;

            if (path == null || content == null) {
              response['error'] = 'Invalid payload for testScript: requires path and content fields';
            } else {
              final saveResult = await scriptFileService.saveScript(path, content);
              if (!saveResult.success) {
                response['error'] = saveResult.errorMessage;
              } else {
                final testResult = await testService.runTests(path);
                if (testResult.hasError) {
                  response['error'] = testResult.errorMessage;
                } else {
                  response['result'] = {
                    'total': testResult.total,
                    'passed': testResult.passed,
                    'failed': testResult.failed,
                    'durationMs': testResult.durationMs,
                    'results': testResult.results.map((r) => {
                      'name': r.name,
                      'passed': r.passed,
                      'error': r.error,
                      'durationMs': r.durationMs,
                    }).toList(),
                  };
                }
              }
            }
          } else {
            response['error'] = 'Invalid payload for testScript (expected object with path and content fields)';
          }
          break;
        case 'echo':
          response['result'] = payload;
          break;
        case 'exit':
          response['result'] = 'Goodbye';
          stdout.writeln(jsonEncode(response));
          exit(0);
        default:
          response['error'] = 'Unknown command: $command';
      }
    } catch (e) {
      response['error'] = e.toString();
    }

    return response;
  }
}