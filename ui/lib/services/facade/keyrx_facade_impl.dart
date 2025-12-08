/// Real implementation of the KeyRx facade.
///
/// Wraps the ServiceRegistry and provides a simplified, coordinated API surface.
/// Aggregates state from multiple services and handles multi-step operations.
library;

import 'dart:async';
import 'dart:convert';
import 'package:rxdart/rxdart.dart';

import '../service_registry.dart';
import '../test_service.dart';
import '../../ffi/bridge.dart' hide ScriptValidationResult;
import '../../models/log_entry.dart';
import '../../ffi/event_types.dart';
import 'facade_state.dart';
import 'keyrx_facade.dart';
import 'result.dart';

/// Real implementation of [KeyrxFacade].
///
/// This class:
/// - Wraps the ServiceRegistry and delegates operations to appropriate services
/// - Aggregates state from multiple service streams
/// - Coordinates multi-step operations atomically
/// - Translates technical errors to user-friendly messages
///
/// Example usage:
/// ```dart
/// final registry = ServiceRegistry.real();
/// final facade = KeyrxFacadeImpl(registry);
///
/// // Use facade operations
/// final result = await facade.startEngine('/path/to/script.rhai');
///
/// // Observe state
/// facade.stateStream.listen((state) {
///   print('Engine: ${state.engine}');
/// });
///
/// // Cleanup
/// await facade.dispose();
/// ```
class KeyrxFacadeImpl implements KeyrxFacade {
  /// Create a new facade implementation wrapping the given service registry.
  ///
  /// Initializes the state stream with [FacadeState.initial] and sets up
  /// subscriptions to underlying service streams for state aggregation.
  ///
  /// Remember to call [dispose] when the facade is no longer needed.
  KeyrxFacadeImpl(this._services)
    : _stateSubject = BehaviorSubject<FacadeState>.seeded(
        FacadeState.initial(),
      ) {
    _initializeStateAggregation();
  }

  final ServiceRegistry _services;
  final BehaviorSubject<FacadeState> _stateSubject;
  final List<StreamSubscription<dynamic>> _subscriptions = [];

  bool _disposed = false;

  /// Initialize state stream aggregation from underlying services.
  ///
  /// This sets up subscriptions to individual service state streams and
  /// combines them into the unified facade state. Updates are debounced
  /// by 100ms to avoid excessive emissions during rapid state changes.
  void _initializeStateAggregation() {
    // Subscribe to engine state stream and map to facade state updates
    final engineStateSub = _services.engineService.stateStream
        .debounceTime(const Duration(milliseconds: 100))
        .listen(
          (engineSnapshot) {
            // Map engine snapshot to engine status
            EngineStatus engineStatus = currentState.engine;

            // If we have active layers or held keys, the engine is actively running
            if (engineSnapshot.activeLayers.isNotEmpty ||
                engineSnapshot.heldKeys.isNotEmpty ||
                engineSnapshot.pendingDecisions.isNotEmpty) {
              engineStatus = EngineStatus.running;
            } else if (_services.engineService.isInitialized) {
              // If initialized but idle, mark as ready
              if (currentState.engine != EngineStatus.running &&
                  currentState.engine != EngineStatus.loading) {
                engineStatus = EngineStatus.ready;
              }
            }

            // Only update if status changed or we have new event data
            if (engineStatus != currentState.engine ||
                engineSnapshot.lastEvent != null) {
              _updateState(
                currentState.copyWith(
                  engine: engineStatus,
                  timestamp: engineSnapshot.timestamp,
                ),
              );
            }
          },
          onError: (error) {
            // On engine stream error, update state to reflect error
            _updateState(
              currentState.withEngineStatus(
                EngineStatus.error,
                error: 'Engine stream error: $error',
              ),
            );
          },
        );

    _subscriptions.add(engineStateSub);

    // Note: DeviceService, ValidationStatus, and DiscoveryStatus don't have
    // their own state streams. These states are managed through the facade's
    // operation methods (listDevices, validateScript, etc.)
    // and are already updated via _updateState() calls in those methods.
    //
    // The 100ms debounce on the state subject's stream (applied via the getter)
    // ensures that rapid manual state updates are also debounced appropriately.
  }

  @override
  Stream<FacadeState> get stateStream => _stateSubject.stream
      .debounceTime(const Duration(milliseconds: 100))
      .distinct((prev, next) => prev == next);

  @override
  FacadeState get currentState => _stateSubject.value;

  @override
  ServiceRegistry get services => _services;

  // === Engine Operations ===

  @override
  Future<Result<void>> startEngine(String scriptPath) async {
    _checkDisposed();

    try {
      // Step 1: Update state to loading
      _updateState(
        currentState.withEngineStatus(
          EngineStatus.loading,
          scriptPath: scriptPath,
        ),
      );

      // Step 2: Initialize engine (always ensure runtime is ready)
      // We call initialize() unconditionally to ensure the revolutionary runtime
      // is recovered if it was lost (e.g. during hot restart) even if the core bridge
      // thinks it's initialized. The bridge initialization is idempotent.
      _updateState(currentState.withEngineStatus(EngineStatus.initializing));
      final initialized = await _services.engineService.initialize();
      if (!initialized) {
        final error = FacadeError.operationFailed(
          'startEngine',
          'Engine initialization failed',
          userMessage: 'Failed to initialize the engine. Please try again.',
        );
        _updateState(
          currentState.withEngineStatus(
            EngineStatus.error,
            error: error.userMessage,
          ),
        );
        return Result.err(error);
      }
      _updateState(currentState.withEngineStatus(EngineStatus.ready));

      // Step 3: Load the script
      _updateState(
        currentState.withEngineStatus(
          EngineStatus.loading,
          scriptPath: scriptPath,
        ),
      );
      final loaded = await _services.engineService.loadScript(scriptPath);
      if (!loaded) {
        final error = FacadeError.operationFailed(
          'startEngine',
          'Script loading failed',
          userMessage: 'Failed to load the script. Please check the file path.',
        );
        _updateState(
          currentState.withEngineStatus(
            EngineStatus.error,
            error: error.userMessage,
          ),
        );
        return Result.err(error);
      }

      // Step 4: Mark engine as running
      _updateState(
        currentState.withEngineStatus(
          EngineStatus.running,
          scriptPath: scriptPath,
        ),
      );

      return const Result.ok(null);
    } catch (e) {
      final error = FacadeError.from(e, _services.errorTranslator);
      _updateState(
        currentState.withEngineStatus(
          EngineStatus.error,
          error: error.userMessage,
        ),
      );
      return Result.err(error);
    }
  }

  @override
  Future<Result<void>> stopEngine() async {
    _checkDisposed();

    try {
      // Check if engine is running
      if (currentState.engine != EngineStatus.running &&
          currentState.engine != EngineStatus.paused) {
        return const Result.ok(null);
      }

      // Update state to stopping
      _updateState(currentState.withEngineStatus(EngineStatus.stopping));

      // For now, just mark as ready since we don't have explicit stop methods
      // The engine service disposal will handle cleanup
      _updateState(currentState.withEngineStatus(EngineStatus.ready));

      return const Result.ok(null);
    } catch (e) {
      final error = FacadeError.from(e, _services.errorTranslator);
      _updateState(
        currentState.withEngineStatus(
          EngineStatus.error,
          error: error.userMessage,
        ),
      );
      return Result.err(error);
    }
  }

  @override
  Future<Result<EngineStatus>> getEngineStatus() async {
    _checkDisposed();

    try {
      // Determine status from engine service state
      final isInitialized = _services.engineService.isInitialized;

      if (!isInitialized) {
        return const Result.ok(EngineStatus.uninitialized);
      }

      // Return current state's engine status
      return Result.ok(currentState.engine);
    } catch (e) {
      final error = FacadeError.from(e, _services.errorTranslator);
      return Result.err(error);
    }
  }

  // === Script Operations ===

  @override
  Future<Result<ScriptValidationResult>> validateScript(
    String scriptPath,
  ) async {
    _checkDisposed();

    try {
      // Update state to validating
      _updateState(
        currentState.withValidationStatus(ValidationStatus.validating),
      );

      // Step 1: Load script content
      final content = await _services.scriptFileService.loadScript(scriptPath);
      if (content == null) {
        final error = FacadeError.operationFailed(
          'validateScript',
          'Script file not found or cannot be read',
          userMessage:
              'Could not read the script file. Please check the file path.',
        );
        _updateState(
          currentState.withValidationStatus(
            ValidationStatus.none,
            error: error.userMessage,
          ),
        );
        return Result.err(error);
      }

      // Step 2: Validate using the bridge
      final validationResult = _services.bridge.validateScript(content);

      // Step 3: Convert FFI ValidationResult to facade ScriptValidationResult
      final facadeResult = ScriptValidationResult(
        isValid: validationResult.isValid,
        errors: validationResult.errors
            .map(
              (e) => ValidationIssue(
                message: e.message,
                line: e.location?.line,
                column: e.location?.column,
                severity: IssueSeverity.error,
              ),
            )
            .toList(),
        warnings: validationResult.warnings
            .map(
              (w) => ValidationIssue(
                message: w.message,
                line: w.location?.line,
                column: w.location?.column,
                severity: IssueSeverity.warning,
              ),
            )
            .toList(),
        suggestions: validationResult.errors
            .where((e) => e.hasSuggestions)
            .expand((e) => e.suggestions)
            .toList(),
      );

      // Step 4: Update validation state based on result
      if (validationResult.isValid) {
        if (validationResult.hasWarnings) {
          _updateState(
            currentState.withValidationStatus(
              ValidationStatus.validWithWarnings,
              errorCount: 0,
              warningCount: validationResult.warnings.length,
            ),
          );
        } else {
          _updateState(
            currentState.withValidationStatus(
              ValidationStatus.valid,
              errorCount: 0,
              warningCount: 0,
            ),
          );
        }
      } else {
        _updateState(
          currentState.withValidationStatus(
            ValidationStatus.invalid,
            errorCount: validationResult.errors.length,
            warningCount: validationResult.warnings.length,
          ),
        );
      }

      return Result.ok(facadeResult);
    } catch (e) {
      final error = FacadeError.from(e, _services.errorTranslator);
      _updateState(
        currentState.withValidationStatus(
          ValidationStatus.none,
          error: error.userMessage,
        ),
      );
      return Result.err(error);
    }
  }

  @override
  Future<Result<String>> loadScriptContent(String path) async {
    _checkDisposed();

    try {
      final content = await _services.scriptFileService.loadScript(path);

      if (content == null) {
        final error = FacadeError.operationFailed(
          'loadScriptContent',
          'Script file not found',
          userMessage: 'Could not find the script file at the specified path.',
        );
        return Result.err(error);
      }

      return Result.ok(content);
    } catch (e) {
      final error = FacadeError.from(e, _services.errorTranslator);
      return Result.err(error);
    }
  }

  @override
  Future<Result<void>> saveScript(String path, String content) async {
    _checkDisposed();

    try {
      final result = await _services.scriptFileService.saveScript(
        path,
        content,
      );

      if (!result.success) {
        final error = FacadeError.operationFailed(
          'saveScript',
          result.errorMessage ?? 'Unknown error',
          userMessage:
              'Failed to save the script. ${result.errorMessage ?? "Please try again."}',
        );
        return Result.err(error);
      }

      return const Result.ok(null);
    } catch (e) {
      final error = FacadeError.from(e, _services.errorTranslator);
      return Result.err(error);
    }
  }

  // === Device Operations ===

  @override
  Future<Result<List<KeyboardDevice>>> listDevices() async {
    _checkDisposed();

    try {
      // Update state to scanning
      _updateState(currentState.withDeviceStatus(DeviceStatus.scanning));

      // List devices from device service
      // We call refresh() to ensure we get the latest list of devices from the OS,
      // rather than a cached list. This is critical for the "Reload" button in UI.
      final devices = await _services.deviceService.refresh();

      // Update state based on result
      if (devices.isEmpty) {
        _updateState(currentState.withDeviceStatus(DeviceStatus.none));
      } else {
        _updateState(currentState.withDeviceStatus(DeviceStatus.available));
      }

      return Result.ok(devices);
    } catch (e) {
      final error = FacadeError.from(e, _services.errorTranslator);
      _updateState(
        currentState.withDeviceStatus(
          DeviceStatus.error,
          error: error.userMessage,
        ),
      );
      return Result.err(error);
    }
  }

  @override
  Future<Result<void>> selectDevice(String devicePath) async {
    _checkDisposed();

    try {
      // Update state to selected
      _updateState(
        currentState.withDeviceStatus(
          DeviceStatus.selected,
          devicePath: devicePath,
        ),
      );

      // Select the device using device service
      final result = await _services.deviceService.selectDevice(devicePath);

      if (!result.success) {
        final error = FacadeError.operationFailed(
          'selectDevice',
          result.errorMessage ?? 'Unknown error',
          userMessage:
              'Failed to select device. ${result.errorMessage ?? "Please try again."}',
        );
        _updateState(
          currentState.withDeviceStatus(
            DeviceStatus.error,
            error: error.userMessage,
          ),
        );
        return Result.err(error);
      }

      // Update state to connected
      _updateState(
        currentState.withDeviceStatus(
          DeviceStatus.connected,
          devicePath: devicePath,
        ),
      );

      return const Result.ok(null);
    } catch (e) {
      final error = FacadeError.from(e, _services.errorTranslator);
      _updateState(
        currentState.withDeviceStatus(
          DeviceStatus.error,
          error: error.userMessage,
        ),
      );
      return Result.err(error);
    }
  }

  // === Testing Operations ===

  @override
  Future<Result<TestDiscoveryServiceResult>> discoverTests(
    String scriptPath,
  ) async {
    _checkDisposed();

    try {
      // Discover tests using the test service
      final result = await _services.testService.discoverTests(scriptPath);

      // Check if discovery had errors
      if (result.hasError) {
        final error = FacadeError.operationFailed(
          'discoverTests',
          result.errorMessage ?? 'Unknown error',
          userMessage:
              'Failed to discover tests. ${result.errorMessage ?? "Please check the script."}',
        );
        return Result.err(error);
      }

      return Result.ok(result);
    } catch (e) {
      final error = FacadeError.from(e, _services.errorTranslator);
      return Result.err(error);
    }
  }

  @override
  Future<Result<TestRunServiceResult>> runTests(
    String scriptPath, {
    String? filter,
  }) async {
    _checkDisposed();

    try {
      // Run tests using the test service
      final result = await _services.testService.runTests(
        scriptPath,
        filter: filter,
      );

      // Check if test execution had errors
      if (result.hasError) {
        final error = FacadeError.operationFailed(
          'runTests',
          result.errorMessage ?? 'Unknown error',
          userMessage:
              'Failed to run tests. ${result.errorMessage ?? "Please check the script."}',
        );
        return Result.err(error);
      }

      return Result.ok(result);
    } catch (e) {
      final error = FacadeError.from(e, _services.errorTranslator);
      return Result.err(error);
    }
  }

  @override
  Future<Result<void>> cancelTests() async {
    _checkDisposed();

    try {
      // Note: The current TestService implementation doesn't have a cancelTests method.
      // This is a placeholder for future implementation when cancellation support is added.
      // For now, tests run to completion and cannot be cancelled mid-execution.

      // Since there's no active cancellation mechanism in TestService yet,
      // we return success (no-op). When TestService adds cancellation support,
      // this should delegate to that method.

      return const Result.ok(null);
    } catch (e) {
      final error = FacadeError.from(e, _services.errorTranslator);
      return Result.err(error);
    }
  }

  // === Lifecycle ===

  @override
  Stream<dynamic> get logStream {
    if (_logController == null) {
      _logController = StreamController<dynamic>.broadcast(
        onListen: () {
          _services.bridge.registerEventCallback(EventType.diagnosticsLog, (
            payload,
          ) {
            try {
              // Payload is Uint8List, need to decode UTF-8
              final jsonString = utf8.decode(payload, allowMalformed: true);
              final json = jsonDecode(jsonString) as Map<String, dynamic>;
              try {
                final entry = LogEntry.fromJson(json);
                _logController?.add(entry);
              } catch (e) {
                print('Error parsing log entry: $e');
              }
            } catch (e) {
              // Fail silently to avoid crash loops
            }
          });
        },
      );
    }
    return _logController!.stream;
  }

  StreamController<dynamic>? _logController;

  @override
  Future<void> dispose() async {
    if (_disposed) return;
    _disposed = true;

    // Cancel all state stream subscriptions
    for (final subscription in _subscriptions) {
      await subscription.cancel();
    }
    _subscriptions.clear();

    // Close the state subject
    await _stateSubject.close();
    await _logController?.close();

    // Don't dispose the service registry - the caller owns it
    // and may be using it elsewhere
  }

  /// Update the current state and emit to stream.
  ///
  /// This method is called internally whenever an operation changes the facade state.
  /// The new state is emitted to [stateStream] where widgets can observe it.
  ///
  /// Does nothing if the facade has been disposed.
  void _updateState(FacadeState newState) {
    if (!_disposed) {
      _stateSubject.add(newState);
    }
  }

  /// Check if facade has been disposed.
  ///
  /// Throws a [StateError] if the facade has been disposed.
  /// All public methods call this to ensure the facade is still usable.
  void _checkDisposed() {
    if (_disposed) {
      throw StateError('KeyrxFacade has been disposed');
    }
  }
}
