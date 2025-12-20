/// Stub for MockKeyrxBridge on web platform.
///
/// MockKeyrxBridge is not available on web since it depends on FFI types.
/// Web applications should use ServiceRegistry.real() without mock mode.
library;

import 'bridge_stub.dart';

/// Stub that throws UnsupportedError.
/// Web applications should not use MOCK_FFI mode.
class MockKeyrxBridge implements KeyrxBridge {
  factory MockKeyrxBridge() {
    throw UnsupportedError(
      'MockKeyrxBridge is not available on web platform. '
      'Use ServiceRegistry.real() without MOCK_FFI=true.',
    );
  }

  @override
  dynamic get bindings => throw UnsupportedError('FFI not available on web');

  @override
  dynamic get runtimeBindings => throw UnsupportedError('FFI not available on web');

  @override
  String? get loadFailure => null;

  @override
  dynamic noSuchMethod(Invocation invocation) {
    throw UnsupportedError('FFI method ${invocation.memberName} not available on web platform.');
  }
}
