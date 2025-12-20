/// Service for managing Keymaps.
library;

import '../models/keymap.dart';
import 'config_result.dart';

export 'keymap_service_ffi.dart'
    if (dart.library.html) 'keymap_service_web.dart';

/// Service for managing keymaps.
abstract class KeymapService {
  /// List all keymaps.
  Future<ConfigOperationResult<List<Keymap>>> listKeymaps();

  /// Persist or update a keymap definition.
  Future<ConfigOperationResult<Keymap>> saveKeymap(Keymap keymap);

  /// Delete a keymap by id.
  Future<ConfigOperationResult<void>> deleteKeymap(String id);
}
