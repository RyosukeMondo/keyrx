/// UI wrapper for HardwareService that provides ChangeNotifier compatibility.
library;

import 'dart:async';

import 'package:core_logic/core_logic.dart' as core;
import 'package:flutter/foundation.dart';

/// UI wrapper for [core.HardwareService] that extends ChangeNotifier.
///
/// This wrapper listens to the core service's stream and calls notifyListeners()
/// to maintain compatibility with Riverpod providers that expect ChangeNotifier.
class HardwareServiceUI extends ChangeNotifier {
  HardwareServiceUI(this._core) {
    _subscription = _core.changes.listen((_) => notifyListeners());
  }

  final core.HardwareService _core;
  StreamSubscription<void>? _subscription;

  /// List all hardware profiles.
  Future<core.ConfigOperationResult<List<core.HardwareProfile>>>
  listProfiles() => _core.listProfiles();

  /// Persist or update a hardware profile.
  Future<core.ConfigOperationResult<core.HardwareProfile>> saveProfile(
    core.HardwareProfile profile,
  ) => _core.saveProfile(profile);

  /// Delete a hardware profile by id.
  Future<core.ConfigOperationResult<void>> deleteProfile(String id) =>
      _core.deleteProfile(id);

  @override
  void dispose() {
    _subscription?.cancel();
    _core.dispose();
    super.dispose();
  }
}
