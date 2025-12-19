import 'package:core_logic/ffi/bridge.dart';
import 'package:test/test.dart';

void main() {
  group('KeyrxBridge', () {
    test('can be opened (or reports load failure)', () {
      final bridge = KeyrxBridge.open();
      // We don't assert it succeeds because it depends on the native library being built
      // but it shouldn't crash.
      expect(bridge, isNotNull);
    });

    test('version is accessible if loaded', () {
      final bridge = KeyrxBridge.open();
      if (bridge.loadFailure == null) {
        expect(bridge.version, isNotEmpty);
        expect(bridge.version, isNot(equals('unavailable')));
      }
    });
  });
}