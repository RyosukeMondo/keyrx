/// Real implementation of the KeyRx facade.
///
/// Wraps the ServiceRegistry and provides a simplified, coordinated API surface.
/// Aggregates state from multiple services and handles multi-step operations.
library;

import 'dart:async';
import 'package:rxdart/rxdart.dart';

import '../service_registry.dart';
import '../test_service.dart';
import '../../ffi/bridge.dart' hide ScriptValidationResult;
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
    // TODO: In task 9, we'll implement proper state stream aggregation
    // from individual services. For now, we just maintain the initial state.
    // This will use rxdart's combineLatest and debounce operators to
    // merge engine, device, validation, and discovery state streams.
  }

  @override
  Stream<FacadeState> get stateStream => _stateSubject.stream;

  @override
  FacadeState get currentState => _stateSubject.value;

  @override
  ServiceRegistry get services => _services;

  // === Engine Operations ===

  @override
  Future<Result<void>> startEngine(String scriptPath) async {
    // TODO: Implement in task 5
    throw UnimplementedError('startEngine will be implemented in task 5');
  }

  @override
  Future<Result<void>> stopEngine() async {
    // TODO: Implement in task 5
    throw UnimplementedError('stopEngine will be implemented in task 5');
  }

  @override
  Future<Result<EngineStatus>> getEngineStatus() async {
    // TODO: Implement in task 5
    throw UnimplementedError('getEngineStatus will be implemented in task 5');
  }

  // === Script Operations ===

  @override
  Future<Result<ScriptValidationResult>> validateScript(String scriptPath) async {
    // TODO: Implement in task 6
    throw UnimplementedError('validateScript will be implemented in task 6');
  }

  @override
  Future<Result<String>> loadScriptContent(String path) async {
    // TODO: Implement in task 6
    throw UnimplementedError('loadScriptContent will be implemented in task 6');
  }

  @override
  Future<Result<void>> saveScript(String path, String content) async {
    // TODO: Implement in task 6
    throw UnimplementedError('saveScript will be implemented in task 6');
  }

  // === Device Operations ===

  @override
  Future<Result<List<KeyboardDevice>>> listDevices() async {
    // TODO: Implement in task 7
    throw UnimplementedError('listDevices will be implemented in task 7');
  }

  @override
  Future<Result<void>> selectDevice(String devicePath) async {
    // TODO: Implement in task 7
    throw UnimplementedError('selectDevice will be implemented in task 7');
  }

  @override
  Future<Result<void>> startDiscovery({
    required KeyboardDevice device,
    required int rows,
    required List<int> colsPerRow,
  }) async {
    // TODO: Implement in task 7
    throw UnimplementedError('startDiscovery will be implemented in task 7');
  }

  @override
  Future<Result<void>> cancelDiscovery() async {
    // TODO: Implement in task 7
    throw UnimplementedError('cancelDiscovery will be implemented in task 7');
  }

  // === Testing Operations ===

  @override
  Future<Result<TestDiscoveryServiceResult>> discoverTests(String scriptPath) async {
    // TODO: Implement in task 8
    throw UnimplementedError('discoverTests will be implemented in task 8');
  }

  @override
  Future<Result<TestRunServiceResult>> runTests(
    String scriptPath, {
    String? filter,
  }) async {
    // TODO: Implement in task 8
    throw UnimplementedError('runTests will be implemented in task 8');
  }

  @override
  Future<Result<void>> cancelTests() async {
    // TODO: Implement in task 8
    throw UnimplementedError('cancelTests will be implemented in task 8');
  }

  // === Lifecycle ===

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

    // Don't dispose the service registry - the caller owns it
    // and may be using it elsewhere
  }

  /// Update the current state and emit to stream.
  void _updateState(FacadeState newState) {
    if (!_disposed) {
      _stateSubject.add(newState);
    }
  }

  /// Check if facade has been disposed.
  void _checkDisposed() {
    if (_disposed) {
      throw StateError('KeyrxFacade has been disposed');
    }
  }
}
