/// UI wrapper for LayoutService that provides ChangeNotifier compatibility.
library;

import 'dart:async';

import 'package:core_logic/core_logic.dart' as core;
import 'package:flutter/foundation.dart';

/// UI wrapper for [core.LayoutService] that extends ChangeNotifier.
///
/// This wrapper listens to the core service's stream and calls notifyListeners()
/// to maintain compatibility with Riverpod providers that expect ChangeNotifier.
class LayoutServiceUI extends ChangeNotifier {
  LayoutServiceUI(this._core) {
    _subscription = _core.changes.listen((_) => notifyListeners());
  }

  final core.LayoutService _core;
  StreamSubscription<void>? _subscription;

  /// List all virtual layouts stored in the config directory.
  Future<core.ConfigOperationResult<List<core.VirtualLayout>>> listLayouts() =>
      _core.listLayouts();

  /// Persist a virtual layout definition and return the saved value.
  Future<core.ConfigOperationResult<core.VirtualLayout>> saveLayout(
    core.VirtualLayout layout,
  ) => _core.saveLayout(layout);

  /// Delete a virtual layout by id.
  Future<core.ConfigOperationResult<void>> deleteLayout(String id) =>
      _core.deleteLayout(id);

  @override
  void dispose() {
    _subscription?.cancel();
    _core.dispose();
    super.dispose();
  }
}
