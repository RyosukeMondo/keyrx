/// Stub that provides KeymapServiceFFI interface for web platform.
library;

import 'keymap_service.dart';

/// Stub class that matches FFI interface but throws on web.
class KeymapServiceFFI implements KeymapService {
  KeymapServiceFFI({required dynamic bridge}) {
    throw UnsupportedError(
      'KeymapServiceFFI not available on web. Use KeymapServiceWeb instead.',
    );
  }

  @override
  dynamic noSuchMethod(Invocation invocation) {
    throw UnsupportedError('KeymapServiceFFI not available on web');
  }
}
