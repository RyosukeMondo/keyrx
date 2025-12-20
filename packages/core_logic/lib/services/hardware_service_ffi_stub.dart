/// Stub that provides HardwareServiceFFI interface for web platform.
library;

import 'hardware_service.dart';

/// Stub class that matches FFI interface but throws on web.
class HardwareServiceFFI implements HardwareService {
  HardwareServiceFFI({required dynamic bridge}) {
    throw UnsupportedError(
      'HardwareServiceFFI not available on web. Use HardwareServiceWeb instead.',
    );
  }

  @override
  dynamic noSuchMethod(Invocation invocation) {
    throw UnsupportedError('HardwareServiceFFI not available on web');
  }
}
