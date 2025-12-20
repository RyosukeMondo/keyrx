/// Providers for daemon service.
library;

import 'dart:async';

import 'package:core_logic/models/daemon_protocol.dart';
import 'package:flutter/foundation.dart' show kIsWeb;
import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../services/daemon_service.dart';

/// Web stub implementation of DaemonService.
///
/// Provides safe no-op implementations for web platform where daemon
/// IPC is not available.
class _WebStubDaemonService implements DaemonService {
  _WebStubDaemonService() {
    _eventController = StreamController<DaemonEvent>.broadcast();
    _notificationController = StreamController<JsonRpcNotification>.broadcast();
    _stateController = StreamController<DaemonConnectionState>.broadcast();
    _statusController = StreamController<EngineStatus>.broadcast();
  }

  late final StreamController<DaemonEvent> _eventController;
  late final StreamController<JsonRpcNotification> _notificationController;
  late final StreamController<DaemonConnectionState> _stateController;
  late final StreamController<EngineStatus> _statusController;

  @override
  Stream<DaemonEvent> get events => _eventController.stream;

  @override
  Stream<JsonRpcNotification> get notificationStream => _notificationController.stream;

  @override
  Stream<DaemonConnectionState> get connectionState => _stateController.stream;

  @override
  Stream<EngineStatus> get statusStream => _statusController.stream;

  @override
  EngineStatus get currentStatus => const EngineStatus(status: 'unavailable');

  @override
  DaemonConnectionState get state => DaemonConnectionState.disconnected;

  @override
  bool get isConnected => false;

  @override
  Future<void> connect() async {
    // No-op on web
  }

  @override
  Future<void> disconnect() async {
    // No-op on web
  }

  @override
  Future<void> subscribeEvents({
    List<String> eventTypes = const ['input', 'output', 'state_change'],
  }) async {
    // No-op on web - events come via WebSocket
  }

  @override
  Future<void> unsubscribeEvents() async {
    // No-op on web
  }

  @override
  Future<List<DaemonDeviceInfo>> startEngine({
    String? configPath,
    String? scriptPath,
    bool useAllDevices = true,
  }) async {
    // No-op on web - engine controlled via backend API
    return [];
  }

  @override
  Future<void> stopEngine() async {
    // No-op on web
  }

  @override
  Future<EngineStatus> getStatus() async {
    // Return unavailable status on web
    return const EngineStatus(status: 'unavailable');
  }

  @override
  Future<void> reloadConfig({String? configPath, String? scriptPath}) async {
    // No-op on web
  }

  @override
  Future<dynamic> sendRequest(String method, [Map<String, dynamic>? params]) async {
    // No-op on web - API calls go through WebApiClient
    return null;
  }

  @override
  Future<void> dispose() async {
    await _eventController.close();
    await _notificationController.close();
    await _stateController.close();
    await _statusController.close();
  }

  @override
  dynamic noSuchMethod(Invocation invocation) {
    // Gracefully handle any other methods
    print('[WARN] DaemonService method called on web (not supported): ${invocation.memberName}');
    return null;
  }
}

/// Provider for the daemon service singleton.
///
/// This provides a single instance of DaemonService that can be used
/// throughout the app to communicate with the keyrx daemon.
///
/// On web platform, returns a stub implementation.
final daemonServiceProvider = Provider<DaemonService>((ref) {
  print('[DEBUG] daemonServiceProvider: initializing (kIsWeb=$kIsWeb)');

  // On web, create stub implementation
  if (kIsWeb) {
    print('[DEBUG] daemonServiceProvider: creating stub for web platform');
    final stub = _WebStubDaemonService();
    ref.onDispose(stub.dispose);
    print('[DEBUG] daemonServiceProvider: stub initialized successfully');
    return stub;
  }

  // On desktop, create real daemon service
  print('[DEBUG] daemonServiceProvider: creating real DaemonService for desktop');
  final service = DaemonService();

  // Dispose the service when the provider is disposed
  ref.onDispose(service.dispose);

  print('[DEBUG] daemonServiceProvider: initialized successfully');
  return service;
});

/// Provider for daemon connection state.
final daemonConnectionStateProvider = StreamProvider<DaemonConnectionState>((ref) {
  final service = ref.watch(daemonServiceProvider);
  return service.connectionState;
});

/// Provider for daemon status.
final daemonStatusProvider = StreamProvider<EngineStatus>((ref) {
  final service = ref.watch(daemonServiceProvider);
  return service.statusStream;
});

/// Provider for daemon events.
final daemonEventsProvider = StreamProvider((ref) {
  final service = ref.watch(daemonServiceProvider);
  return service.events;
});
