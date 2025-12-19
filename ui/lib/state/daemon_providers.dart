/// Providers for daemon service.
library;

import 'package:core_logic/models/daemon_protocol.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../services/daemon_service.dart';

/// Provider for the daemon service singleton.
///
/// This provides a single instance of DaemonService that can be used
/// throughout the app to communicate with the keyrx daemon.
final daemonServiceProvider = Provider<DaemonService>((ref) {
  final service = DaemonService();

  // Dispose the service when the provider is disposed
  ref.onDispose(service.dispose);

  return service;
});

/// Provider for daemon connection state.
final daemonConnectionStateProvider = StreamProvider<DaemonConnectionState>((
  ref,
) {
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
