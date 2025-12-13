import 'package:flutter_test/flutter_test.dart';
import 'package:keyrx_ui/models/hardware_profile.dart';
import 'package:keyrx_ui/models/keymap.dart';
import 'package:keyrx_ui/models/action_binding.dart';

void main() {
  group('HardwareProfile Deserialization Repro', () {
    test('Should parse full valid JSON', () {
      final json = {
        'id': 'test-profile',
        'vendor_id': 1234,
        'product_id': 5678,
        'serial_number': 'SN123',
        'name': 'Test Profile',
        'virtual_layout_id': 'virtual-layout-1',
        'wiring': {'10': 'KEY_A', '20': 'KEY_B'},
      };

      final profile = HardwareProfile.fromJson(json);
      expect(profile.id, 'test-profile');
      expect(profile.vendorId, 1234);
      expect(profile.wiring[10], 'KEY_A');
    });

    test('Should handle missing vendor_id gracefully', () {
      final json = {
        'id': 'test-profile',
        // 'vendor_id': missing
        'product_id': 5678,
        'virtual_layout_id': 'virtual-layout-1',
      };
      final profile = HardwareProfile.fromJson(json);
      expect(profile.vendorId, 0); // Default value
    });

    test('Should handle missing product_id gracefully', () {
      final json = {
        'id': 'test-profile',
        'vendor_id': 1234,
        // 'product_id': missing
        'virtual_layout_id': 'virtual-layout-1',
      };
      final profile = HardwareProfile.fromJson(json);
      expect(profile.productId, 0); // Default value
    });

    test('Should fail if wiring value is null', () {
      final json = {
        'id': 'test-profile',
        'vendor_id': 1234,
        'product_id': 5678,
        'virtual_layout_id': 'virtual-layout-1',
        'wiring': {
          '10': null, // Invalid null value
        },
      };

      try {
        HardwareProfile.fromJson(json);
        fail('Should have thrown TypeError');
      } catch (e) {
        expect(e.toString(), contains('subtype'));
      }
    });
  });

  group('Keymap Deserialization Repro', () {
    test('Should parse valid Keymap', () {
      final json = {
        'id': 'test-keymap',
        'name': 'Test Keymap',
        'virtual_layout_id': 'layout-1',
        'layers': [
          {
            'name': 'Base',
            'bindings': {
              'KEY_A': {'type': 'standard_key', 'value': 'A'},
              'KEY_B': {'type': 'transparent'},
            },
          },
        ],
        'combos': [],
      };
      Keymap.fromJson(json);
    });

    test('Should handle missing StandardKey value gracefully', () {
      final json = {'type': 'standard_key'}; // value missing
      final binding = ActionBinding.fromJson(json);
      binding.map(
        standardKey: (b) => expect(b.value, ''),
        macro: (_) => fail('wrong type'),
        layerToggle: (_) => fail('wrong type'),
        tapHold: (_) => fail('wrong type'),
        transparent: (_) => fail('wrong type'),
      );
    });

    test('Should handle missing Keymap name gracefully', () {
      final json = {
        'id': 'test-keymap',
        // 'name': 'Test Keymap', // Missing
        'virtual_layout_id': 'layout-1',
        'layers': [],
        'combos': [],
      };
      final keymap = Keymap.fromJson(json);
      expect(keymap.name, 'Untitled Keymap');
    });
  });
}
