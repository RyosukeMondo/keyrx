/// Core logic for KeyRx, shared between Flutter UI and CLI.
library;

export 'ffi/bridge.dart' if (dart.library.html) 'ffi/bridge_stub.dart';
export 'models/models.dart';
export 'repositories/device_config_repository.dart';
export 'repositories/mapping_repository.dart';
export 'services/services.dart';
export 'services/config_result.dart';
export 'services/runtime_service.dart';
export 'services/keymap_service.dart';
export 'utils/logger.dart';
export 'utils/error_translator.dart';
export 'utils/error_translator_impl.dart';
export 'utils/storage_path_resolver.dart';