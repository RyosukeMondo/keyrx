import 'package:flutter_test/flutter_test.dart';
import 'package:keyrx_ui/models/hardware_profile.dart';

void main() {
  test(
    'Should parse HardwareProfile with String keys in wiring map (JSON standard)',
    () {
      // JSON payload from the screenshot error message
      final json = {
        'id': 'hardware_1765152434347',
        'vendor_id': 2458,
        'product_id': 1592,
        'serial_number': '8&b1afabf&0&0000',
        'name': '3x5 wire',
        'virtual_layout_id': 'layout_1765161483118',
        'wiring': {
          '19': 'r1c4',
          '32': 'r2c3',
          '31': 'r2c2',
          '18': 'r1c3',
          '30': 'r2c1',
          '16': 'r1c1',
          '33': 'r2c4',
          '17': 'r1c2',
        },
      };

      try {
        final profile = HardwareProfile.fromJson(json);
        expect(profile.wiring[19], 'r1c4');
        expect(profile.wiring[32], 'r2c3');
      } catch (e) {
        fail('Parsing failed: $e');
      }
    },
  );
}
