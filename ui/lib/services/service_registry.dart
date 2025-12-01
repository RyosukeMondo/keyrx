import 'package:permission_handler/permission_handler.dart';

import '../ffi/bridge.dart';
import 'audio_service.dart';
import 'audio_service_impl.dart';
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
  });

  /// Build a registry with real implementations, allowing overrides for tests.
  factory ServiceRegistry.real({
    KeyrxBridge? bridge,
    Permission? microphonePermission,
    Stream<ClassificationResult>? classificationSource,
    ErrorTranslator? errorTranslator,
    PermissionService? permissionService,
  }) {
    final translator = errorTranslator ?? const ErrorTranslatorImpl();
    final permissions =
        permissionService ??
        PermissionServiceImpl(microphonePermission: microphonePermission);
    final effectiveBridge = bridge ?? KeyrxBridge.instance;
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
    );
  }

  /// Create a registry with explicit (often mocked) implementations.
  factory ServiceRegistry.withOverrides({
    required PermissionService permissionService,
    required AudioService audioService,
    required ErrorTranslator errorTranslator,
  }) {
    return ServiceRegistry(
      permissionService: permissionService,
      audioService: audioService,
      errorTranslator: errorTranslator,
    );
  }

  final PermissionService permissionService;
  final AudioService audioService;
  final ErrorTranslator errorTranslator;

  /// Convenience for producing a registry with selective overrides.
  ServiceRegistry copyWith({
    PermissionService? permissionService,
    AudioService? audioService,
    ErrorTranslator? errorTranslator,
  }) {
    return ServiceRegistry(
      permissionService: permissionService ?? this.permissionService,
      audioService: audioService ?? this.audioService,
      errorTranslator: errorTranslator ?? this.errorTranslator,
    );
  }

  /// Dispose any owned resources. Extend as more disposable services appear.
  Future<void> dispose() async {
    await audioService.dispose();
  }
}
