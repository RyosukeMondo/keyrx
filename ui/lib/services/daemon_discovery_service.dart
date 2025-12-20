import 'dart:async';

import 'package:core_logic/models/daemon_protocol.dart';
import 'package:flutter/foundation.dart';

import 'daemon_service.dart';

/// Service for device discovery.
class DaemonDiscoveryService {
  /// Creates a new [DaemonDiscoveryService].
  DaemonDiscoveryService(this._daemonService) {
    _eventSubscription = _daemonService.events.listen(_handleDaemonEvent);
  }

  final DaemonService _daemonService;
  late final StreamSubscription<DaemonEvent> _eventSubscription;
  bool _isDiscovering = false;

  final StreamController<DiscoveryCandidate> _discoveryController =
      StreamController<DiscoveryCandidate>.broadcast();

  /// Stream of discovered devices.
  Stream<DiscoveryCandidate> get discoveryEvents => _discoveryController.stream;

  /// Whether discovery is currently active.
  bool get isDiscovering => _isDiscovering;

  /// Starts device discovery.
  Future<void> startDiscovery() async {
    if (_isDiscovering) {
      return;
    }

    try {
      await _daemonService.sendRequest(DaemonMethods.discoveryStart);
      _isDiscovering = true;
    } on DaemonException catch (e) {
      debugPrint('Failed to start discovery: $e');
      rethrow;
    }
  }

  /// Stops device discovery.
  Future<void> stopDiscovery() async {
    if (!_isDiscovering) {
      return;
    }

    try {
      await _daemonService.sendRequest(DaemonMethods.discoveryStop);
      _isDiscovering = false;
    } on DaemonException catch (e) {
      debugPrint('Failed to stop discovery: $e');
      rethrow;
    }
  }

  void _handleDaemonEvent(DaemonEvent event) {
    event.when(
      discovery: (candidate) {
        _discoveryController.add(candidate);
      },
      input: (_) {},
      output: (_) {},
      decision: (_) {},
      stateChange: (_) {},
    );
  }

  /// Disposes the service.
  @mustCallSuper
  void dispose() {
    unawaited(_eventSubscription.cancel());
    unawaited(_discoveryController.close());
  }
}
