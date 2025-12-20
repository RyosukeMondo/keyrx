/// Service for managing Virtual Layouts.
library;

import 'dart:async';

import '../models/virtual_layout.dart';
import 'config_result.dart';

export 'layout_service_ffi.dart'
    if (dart.library.html) 'layout_service_web.dart';

/// Service for managing virtual layouts.
abstract class LayoutService {
  /// Stream of changes to virtual layouts.
  Stream<void> get changes;

  /// List all virtual layouts stored in the config directory.
  Future<ConfigOperationResult<List<VirtualLayout>>> listLayouts();

  /// Persist a virtual layout definition and return the saved value.
  Future<ConfigOperationResult<VirtualLayout>> saveLayout(
    VirtualLayout layout,
  );

  /// Delete a virtual layout by id.
  Future<ConfigOperationResult<void>> deleteLayout(String id);

  /// Dispose resources.
  void dispose();
}
