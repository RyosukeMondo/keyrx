/// Stub that provides LayoutServiceFFI interface for web platform.
library;

import 'layout_service.dart';

/// Stub class that matches FFI interface but throws on web.
class LayoutServiceFFI implements LayoutService {
  LayoutServiceFFI({required dynamic bridge}) {
    throw UnsupportedError(
      'LayoutServiceFFI not available on web. Use LayoutServiceWeb instead.',
    );
  }

  @override
  dynamic noSuchMethod(Invocation invocation) {
    throw UnsupportedError('LayoutServiceFFI not available on web');
  }
}
