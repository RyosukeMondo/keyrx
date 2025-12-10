import 'package:flutter_test/flutter_test.dart';
import 'package:keyrx_ui/models/action_binding.dart';
import 'package:keyrx_ui/models/keymap.dart';
import 'package:keyrx_ui/services/keymap_converter.dart';
import 'package:keyrx_ui/services/rhai_generator.dart';
import 'package:keyrx_ui/widgets/mapping_overlay.dart';

void main() {
  const converter = KeymapConverter();
  final generator = RhaiGenerator();

  group('KeymapConverter', () {
    test('converts simple base layer remappings', () {
      final keymap = Keymap(
        id: 'test_keymap',
        name: 'Test Keymap',
        virtualLayoutId: 'ansi_104',
        layers: [
          KeymapLayer(
            name: 'Base',
            bindings: {
              'KeyA': const ActionBinding.standardKey(value: 'KeyB'),
              'CapsLock': const ActionBinding.standardKey(value: 'Escape'),
            },
          ),
        ],
      );

      final config = converter.convert(keymap);

      // Verify VisualConfig structure
      expect(config.mappings.length, 2);
      expect(
        config.mappings.contains(
          const RemapConfig(
            sourceKeyId: 'KeyA',
            targetKeyId: 'KeyB',
            type: MappingType.simple,
          ),
        ),
        isTrue,
      );
      expect(
        config.mappings.contains(
          const RemapConfig(
            sourceKeyId: 'CapsLock',
            targetKeyId: 'Escape',
            type: MappingType.simple,
          ),
        ),
        isTrue,
      );

      // Verify generated script
      final script = generator.generateScript(config);
      expect(script, contains('remap("KeyA", "KeyB");'));
      expect(script, contains('remap("CapsLock", "Escape");'));
    });

    test('converts overlay layers', () {
      final keymap = Keymap(
        id: 'test_layers',
        name: 'Test Layers',
        virtualLayoutId: 'ansi_104',
        layers: [
          KeymapLayer(name: 'Base', bindings: {}), // Empty base
          KeymapLayer(
            name: 'Nav',
            bindings: {
              'KeyH': const ActionBinding.standardKey(value: 'Left'),
              'KeyL': const ActionBinding.standardKey(value: 'Right'),
            },
          ),
        ],
      );

      final config = converter.convert(keymap);

      expect(config.mappings.isEmpty, isTrue); // No base mappings
      expect(config.layerConfigs.length, 1);

      final layerConfig = config.layerConfigs.first;
      expect(layerConfig.name, 'Nav');
      expect(layerConfig.isTransparent, isTrue);
      expect(layerConfig.mappings.length, 2);

      // Verify generated script
      final script = generator.generateScript(config);
      expect(script, contains('layer_define("Nav", true);'));
      expect(script, contains('layer_map("Nav", "KeyH", "Left");'));
      expect(script, contains('layer_map("Nav", "KeyL", "Right");'));
    });

    test('ignores unsupported bindings for now', () {
      // Currently only standardKey is supported in the converter
      final keymap = Keymap(
        id: 'test_unsupported',
        name: 'Test Unsupported',
        virtualLayoutId: 'ansi_104',
        layers: [
          KeymapLayer(
            name: 'Base',
            bindings: {
              'KeyA':
                  const ActionBinding.transparent(), // Should be ignored on base
            },
          ),
        ],
      );

      final config = converter.convert(keymap);
      expect(config.mappings.isEmpty, isTrue);
    });
  });
}
