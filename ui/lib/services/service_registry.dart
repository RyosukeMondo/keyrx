import 'package:permission_handler/permission_handler.dart';

import '../ffi/bridge.dart';
import '../repositories/mapping_repository.dart';
import 'audio_service.dart';
import 'audio_service_impl.dart';
import 'device_service.dart';
import 'engine_service.dart';
import 'engine_service_impl.dart';
import 'error_translator.dart';
import 'error_translator_impl.dart';
import 'permission_service.dart';
import 'permission_service_impl.dart';

/// Simple service registry to construct and pass dependencies without globals.
class ServiceRegistry {
  ServiceRegistry({
    required this.permissionService,
    required this.audioService,
    required this.errorTranslator,
    required this.engineService,
    required this.mappingRepository,
    required this.deviceService,
    required this.bridge,
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
  }) {
    final translator = errorTranslator ?? const ErrorTranslatorImpl();
    final permissions =
        permissionService ??
        PermissionServiceImpl(microphonePermission: microphonePermission);
    final effectiveBridge = bridge ?? KeyrxBridge.open();
    final engine = EngineServiceImpl(bridge: effectiveBridge);
    final device = deviceService ?? DeviceServiceImpl(bridge: effectiveBridge);
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

    return ServiceRegistry(
      permissionService: permissions,
      audioService: audio,
      errorTranslator: translator,
      engineService: engine,
      mappingRepository: mappingRepository ?? MappingRepository(),
      deviceService: device,
      bridge: effectiveBridge,
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
    required KeyrxBridge bridge,
  }) {
    return ServiceRegistry(
      permissionService: permissionService,
      audioService: audioService,
      errorTranslator: errorTranslator,
      engineService: engineService,
      mappingRepository: mappingRepository,
      deviceService: deviceService,
      bridge: bridge,
    );
  }

  final PermissionService permissionService;
  final AudioService audioService;
  final ErrorTranslator errorTranslator;
  final EngineService engineService;
  final MappingRepository mappingRepository;
  final DeviceService deviceService;
  final KeyrxBridge bridge;

  /// Convenience for producing a registry with selective overrides.
  ServiceRegistry copyWith({
    PermissionService? permissionService,
    AudioService? audioService,
    ErrorTranslator? errorTranslator,
    EngineService? engineService,
    MappingRepository? mappingRepository,
    DeviceService? deviceService,
    KeyrxBridge? bridge,
  }) {
    return ServiceRegistry(
      permissionService: permissionService ?? this.permissionService,
      audioService: audioService ?? this.audioService,
      errorTranslator: errorTranslator ?? this.errorTranslator,
      engineService: engineService ?? this.engineService,
      mappingRepository: mappingRepository ?? this.mappingRepository,
      deviceService: deviceService ?? this.deviceService,
      bridge: bridge ?? this.bridge,
    );
  }

  /// Dispose any owned resources. Extend as more disposable services appear.
  Future<void> dispose() async {
    await audioService.dispose();
    await engineService.dispose();
    await deviceService.dispose();
    await bridge.dispose();
  }
}
