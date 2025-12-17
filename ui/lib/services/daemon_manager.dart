// ignore_for_file: sort_constructors_first, flutter_style_todos

import 'package:flutter_riverpod/flutter_riverpod.dart';
import '../state/daemon_providers.dart';
import 'daemon_service.dart';

/// Manages high-level daemon lifecycle (auto-connect, retry logic).
class DaemonManager {
  final DaemonService _service;

  DaemonManager(this._service);

  Future<void> initialize() async {
    try {
      await _service.connect();
    } on DaemonException {
      // Ignore initial connection errors, will retry or show status
    }
  }

  Future<void> restartDaemon() async {
    if (_service.isConnected) {
      await _service.stopEngine();
      await _service.disconnect();
    }
    // TODO: Implement actual process restart if needed, for now just reconnect
    await _service.connect();
  }
}

final daemonManagerProvider = Provider<DaemonManager>((ref) {
  final service = ref.watch(daemonServiceProvider);
  return DaemonManager(service);
});
