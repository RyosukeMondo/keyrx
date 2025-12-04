import 'package:permission_handler/permission_handler.dart';

import '../ffi/bridge.dart';
import '../repositories/mapping_repository.dart';
import 'api_docs_service.dart';
import 'audio_service.dart';
import 'audio_service_impl.dart';
import 'device_service.dart';
import 'engine_service.dart';
import 'engine_service_impl.dart';
import 'error_translator.dart';
import 'error_translator_impl.dart';
import 'permission_service.dart';
import 'permission_service_impl.dart';
import 'script_file_service.dart';
import 'test_service.dart';

/// Simple service registry to construct and pass dependencies without globals.
class ServiceRegistry {
  ServiceRegistry({
    required this.permissionService,
    required this.audioService,
    required this.errorTranslator,
    required this.engineService,
    required this.mappingRepository,
    required this.deviceService,
    required this.scriptFileService,
    required this.testService,
    required this.bridge,
    required this.apiDocsService,
  });

  /// Build a registry with real implementations, allowing overrides for tests.
  factory ServiceRegistry.real({
    KeyrxBridge? bridge,
    Permission? microphonePermission,
    Stream<ClassificationResult>? classificationSource,
    ErrorTranslator? errorTranslator,
    PermissionService? permissionService,
    MappingRepository? mappingRepository,
    DeviceService? deviceService,
    ScriptFileService? scriptFileService,
    TestService? testService,
    ApiDocsService? apiDocsService,
  }) {
    final translator = errorTranslator ?? const ErrorTranslatorImpl();
    final permissions =
        permissionService ??
        PermissionServiceImpl(microphonePermission: microphonePermission);
    final effectiveBridge = bridge ?? KeyrxBridge.open();
    final engine = EngineServiceImpl(bridge: effectiveBridge);
    final device = deviceService ?? DeviceServiceImpl(bridge: effectiveBridge);
    final scriptFile = scriptFileService ?? const ScriptFileService();
    final tests = testService ?? TestServiceImpl(bridge: effectiveBridge);
    final mappedClassificationSource =
        classificationSource ??
        effectiveBridge.classificationStream?.map(
          (event) => ClassificationResult(
            label: event.label,
            confidence: event.confidence,
            timestamp: event.timestamp,
          ),
        );

    final audio = AudioServiceImpl(
      bridge: effectiveBridge,
      permissionService: permissions,
      errorTranslator: translator,
      classificationSource: mappedClassificationSource,
    );

    final docs = apiDocsService ?? ApiDocsServiceImpl();

    return ServiceRegistry(
      permissionService: permissions,
      audioService: audio,
      errorTranslator: translator,
      engineService: engine,
      mappingRepository: mappingRepository ?? MappingRepository(),
      deviceService: device,
      scriptFileService: scriptFile,
      testService: tests,
      bridge: effectiveBridge,
      apiDocsService: docs,
    );
  }

  /// Create a registry with explicit (often mocked) implementations.
  factory ServiceRegistry.withOverrides({
    required PermissionService permissionService,
    required AudioService audioService,
    required ErrorTranslator errorTranslator,
    required EngineService engineService,
    required MappingRepository mappingRepository,
    required DeviceService deviceService,
    required ScriptFileService scriptFileService,
    required TestService testService,
    required KeyrxBridge bridge,
    required ApiDocsService apiDocsService,
  }) {
    return ServiceRegistry(
      permissionService: permissionService,
      audioService: audioService,
      errorTranslator: errorTranslator,
      engineService: engineService,
      mappingRepository: mappingRepository,
      deviceService: deviceService,
      scriptFileService: scriptFileService,
      testService: testService,
      bridge: bridge,
      apiDocsService: apiDocsService,
    );
  }

  final PermissionService permissionService;
  final AudioService audioService;
  final ErrorTranslator errorTranslator;
  final EngineService engineService;
  final MappingRepository mappingRepository;
  final DeviceService deviceService;
  final ScriptFileService scriptFileService;
  final TestService testService;
  final KeyrxBridge bridge;
  final ApiDocsService apiDocsService;

  /// Convenience for producing a registry with selective overrides.
  ServiceRegistry copyWith({
    PermissionService? permissionService,
    AudioService? audioService,
    ErrorTranslator? errorTranslator,
    EngineService? engineService,
    MappingRepository? mappingRepository,
    DeviceService? deviceService,
    ScriptFileService? scriptFileService,
    TestService? testService,
    KeyrxBridge? bridge,
    ApiDocsService? apiDocsService,
  }) {
    return ServiceRegistry(
      permissionService: permissionService ?? this.permissionService,
      audioService: audioService ?? this.audioService,
      errorTranslator: errorTranslator ?? this.errorTranslator,
      engineService: engineService ?? this.engineService,
      mappingRepository: mappingRepository ?? this.mappingRepository,
      deviceService: deviceService ?? this.deviceService,
      scriptFileService: scriptFileService ?? this.scriptFileService,
      testService: testService ?? this.testService,
      bridge: bridge ?? this.bridge,
      apiDocsService: apiDocsService ?? this.apiDocsService,
    );
  }

  /// Dispose any owned resources. Extend as more disposable services appear.
  Future<void> dispose() async {
    await audioService.dispose();
    await engineService.dispose();
    await deviceService.dispose();
    await testService.dispose();
    await bridge.dispose();
  }
}
