import 'package:flutter_test/flutter_test.dart';
import 'package:keyrx_ui/models/action_binding.dart';
import 'package:keyrx_ui/models/keymap.dart';
import 'package:keyrx_ui/models/config_ids.dart';
import 'package:keyrx_ui/models/virtual_layout.dart';
import 'package:keyrx_ui/services/keymap_converter.dart';
import 'package:keyrx_ui/services/rhai_generator.dart';

void main() {
  group('KeymapConverter', () {
    const converter = KeymapConverter();

    test(
      'extracts complex mappings from Layer 0 into a separate LayerConfig',
      () {
        final keymap = Keymap(
          id: 'test_id',
          name: 'Test Keymap',
          virtualLayoutId: 'layout_id',
          layers: [
            KeymapLayer(
              name: 'Base',
              bindings: {
                'KEY_A': const ActionBinding.standardKey(value: 'KEY_B'),
                'KEY_M': const ActionBinding.macro(value: 'print("hello")'),
                'KEY_L': const ActionBinding.layerToggle(value: '1'),
              },
            ),
            KeymapLayer(
              name: 'Layer 1',
              bindings: {
                'KEY_C': const ActionBinding.standardKey(value: 'KEY_D'),
              },
            ),
          ],
        );

        final result = converter.convert(keymap);

        // Verify simple mappings (RemapConfig)
        expect(result.mappings, hasLength(1));
        expect(result.mappings.first.sourceKeyId, 'KEY_A');
        expect(result.mappings.first.targetKeyId, 'KEY_B');

        // Verify layer configs
        // Should have:
        // 1. Layer 0 complex mappings (generated)
        // 2. Layer 1 (explicit)
        expect(result.layerConfigs, hasLength(2));

        final baseComplex = result.layerConfigs.firstWhere(
          (l) => l.name == 'Base',
        );
        expect(baseComplex.mappings, hasLength(2));

        // Check Macro
        final macroMapping = baseComplex.mappings.firstWhere(
          (m) => m.sourceKey == 'KEY_M',
        );
        expect(macroMapping.action, 'macro:print("hello")');

        // Check Layer Toggle
        final toggleMapping = baseComplex.mappings.firstWhere(
          (m) => m.sourceKey == 'KEY_L',
        );
        expect(toggleMapping.action, 'layer_toggle:1');

        // Verify Layer 1 is preserved
        final layer1 = result.layerConfigs.firstWhere(
          (l) => l.name == 'Layer 1',
        );
        expect(layer1.mappings, hasLength(1));
        expect(layer1.mappings.first.sourceKey, 'KEY_C');
      },
    );

    test(
      'does not create LayerConfig for Layer 0 if only simple mappings exist',
      () {
        final keymap = Keymap(
          id: 'test_id',
          name: 'Simple Keymap',
          virtualLayoutId: 'layout_id',
          layers: [
            KeymapLayer(
              name: 'Base',
              bindings: {
                'KEY_A': const ActionBinding.standardKey(value: 'KEY_B'),
              },
            ),
          ],
        );

        final result = converter.convert(keymap);

        expect(result.mappings, hasLength(1));
        expect(result.layerConfigs, isEmpty);
      },
    );
  });
}
