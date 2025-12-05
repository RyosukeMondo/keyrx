import 'package:flutter_test/flutter_test.dart';
import 'package:keyrx_ui/models/models.dart';

void main() {
  group('DeviceIdentity', () {
    test('creates instance with required fields', () {
      final identity = DeviceIdentity(
        vendorId: 0x046D,
        productId: 0xC52B,
        serialNumber: 'ABC123',
      );

      expect(identity.vendorId, 0x046D);
      expect(identity.productId, 0xC52B);
      expect(identity.serialNumber, 'ABC123');
      expect(identity.userLabel, null);
    });

    test('toKey returns correct format', () {
      final identity = DeviceIdentity(
        vendorId: 0x046D,
        productId: 0xC52B,
        serialNumber: 'ABC123',
      );

      expect(identity.toKey(), '046d:c52b:ABC123');
    });

    test('fromKey parses valid key', () {
      final identity = DeviceIdentity.fromKey('046d:c52b:ABC123');

      expect(identity, isNotNull);
      expect(identity!.vendorId, 0x046D);
      expect(identity.productId, 0xC52B);
      expect(identity.serialNumber, 'ABC123');
    });

    test('fromKey handles serial with colons', () {
      final identity = DeviceIdentity.fromKey('046d:c52b:ABC:123:XYZ');

      expect(identity, isNotNull);
      expect(identity!.serialNumber, 'ABC:123:XYZ');
    });

    test('fromKey returns null for invalid key', () {
      expect(DeviceIdentity.fromKey('invalid'), null);
      expect(DeviceIdentity.fromKey('046d:c52b'), null);
    });

    test('displayName returns user label when set', () {
      final identity = DeviceIdentity(
        vendorId: 0x046D,
        productId: 0xC52B,
        serialNumber: 'ABC123',
        userLabel: 'My Keyboard',
      );

      expect(identity.displayName, 'My Keyboard');
    });

    test('displayName returns VID:PID when no label', () {
      final identity = DeviceIdentity(
        vendorId: 0x046D,
        productId: 0xC52B,
        serialNumber: 'ABC123',
      );

      expect(identity.displayName, '046d:c52b');
    });

    test('serializes to JSON', () {
      final identity = DeviceIdentity(
        vendorId: 0x046D,
        productId: 0xC52B,
        serialNumber: 'ABC123',
        userLabel: 'Test Device',
      );

      final json = identity.toJson();
      expect(json['vendor_id'], 0x046D);
      expect(json['product_id'], 0xC52B);
      expect(json['serial_number'], 'ABC123');
      expect(json['user_label'], 'Test Device');
    });

    test('deserializes from JSON', () {
      final json = {
        'vendor_id': 0x046D,
        'product_id': 0xC52B,
        'serial_number': 'ABC123',
        'user_label': 'Test Device',
      };

      final identity = DeviceIdentity.fromJson(json);
      expect(identity.vendorId, 0x046D);
      expect(identity.productId, 0xC52B);
      expect(identity.serialNumber, 'ABC123');
      expect(identity.userLabel, 'Test Device');
    });

    test('serializes without user_label when null', () {
      final identity = DeviceIdentity(
        vendorId: 0x046D,
        productId: 0xC52B,
        serialNumber: 'ABC123',
      );

      final json = identity.toJson();
      expect(json.containsKey('user_label'), false);
    });
  });
}
