// ignore_for_file: unused_field, flutter_style_todos, sort_constructors_first

import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'daemon_service.dart';

/// Manages high-level daemon lifecycle (auto-connect, retry logic).
///
/// Intended to be implemented by Jules Task B.
class DaemonManager {
  final DaemonService _service;

  DaemonManager(this._service);

  Future<void> initialize() async {
    // TODO: Jules Task B - Implement auto-connect logic
  }

  Future<void> restartDaemon() async {
    // TODO: Jules Task B - Implement restart logic
  }
}

final daemonManagerProvider = Provider<DaemonManager>((ref) {
  // TODO: Jules Task B - Wiring
  throw UnimplementedError();
});
