// Unit tests for the Rhai script generator service.
//
// Tests code generation, script parsing, and advanced feature detection.

import 'package:flutter_test/flutter_test.dart';
import 'package:keyrx_ui/services/rhai_generator.dart';
import 'package:keyrx_ui/widgets/mapping_overlay.dart';

void main() {
  late RhaiGenerator generator;

  setUp(() {
    generator = RhaiGenerator();
  });

  group('Script generation', () {
    group('generates valid remap syntax', () {
      test('generates single remap', () {
        const config = VisualConfig(
          mappings: [
            RemapConfig(
              sourceKeyId: 'KeyA',
              targetKeyId: 'KeyB',
              type: MappingType.simple,
            ),
          ],
        );

        final script = generator.generateScript(config);

        expect(script, contains('remap("KeyA", "KeyB");'));
      });

      test('generates multiple remaps', () {
        const config = VisualConfig(
          mappings: [
            RemapConfig(
              sourceKeyId: 'KeyA',
              targetKeyId: 'KeyB',
              type: MappingType.simple,
            ),
            RemapConfig(
              sourceKeyId: 'CapsLock',
              targetKeyId: 'Escape',
              type: MappingType.simple,
            ),
          ],
        );

        final script = generator.generateScript(config);

        expect(script, contains('remap("KeyA", "KeyB");'));
        expect(script, contains('remap("CapsLock", "Escape");'));
      });

      test('escapes special characters in key names', () {
        const config = VisualConfig(
          mappings: [
            RemapConfig(
              sourceKeyId: 'Key"Quote',
              targetKeyId: 'Key\\Slash',
              type: MappingType.simple,
            ),
          ],
        );

        final script = generator.generateScript(config);

        // The escape function escapes quotes then backslashes, so:
        // Key"Quote -> Key\"Quote -> Key\\"Quote (backslash from quote escaping is re-escaped)
        // Key\Slash -> Key\\Slash (original backslash is escaped)
        expect(script, contains(r'remap("Key\\"Quote", "Key\\Slash");'));
      });
    });

    group('generates tap_hold syntax', () {
      test('generates single tap-hold', () {
        const config = VisualConfig(
          tapHoldConfigs: [
            TapHoldConfig(
              triggerKey: 'KeyA',
              tapAction: 'KeyA',
              holdAction: 'ControlLeft',
            ),
          ],
        );

        final script = generator.generateScript(config);

        expect(script, contains('tap_hold("KeyA", "KeyA", "ControlLeft");'));
      });

      test('generates multiple tap-holds', () {
        const config = VisualConfig(
          tapHoldConfigs: [
            TapHoldConfig(
              triggerKey: 'KeyA',
              tapAction: 'KeyA',
              holdAction: 'ControlLeft',
            ),
            TapHoldConfig(
              triggerKey: 'KeyS',
              tapAction: 'KeyS',
              holdAction: 'ShiftLeft',
            ),
          ],
        );

        final script = generator.generateScript(config);

        expect(script, contains('tap_hold("KeyA", "KeyA", "ControlLeft");'));
        expect(script, contains('tap_hold("KeyS", "KeyS", "ShiftLeft");'));
      });
    });

    group('generates layer syntax', () {
      test('generates layer definition', () {
        const config = VisualConfig(
          layerConfigs: [
            LayerConfig(name: 'nav', isTransparent: true, mappings: []),
          ],
        );

        final script = generator.generateScript(config);

        expect(script, contains('layer_define("nav", true);'));
      });

      test('generates layer with mappings', () {
        const config = VisualConfig(
          layerConfigs: [
            LayerConfig(
              name: 'symbols',
              isTransparent: false,
              mappings: [
                LayerMapping(sourceKey: 'KeyA', action: 'Key1'),
                LayerMapping(sourceKey: 'KeyS', action: 'Key2'),
              ],
            ),
          ],
        );

        final script = generator.generateScript(config);

        expect(script, contains('layer_define("symbols", false);'));
        expect(script, contains('layer_map("symbols", "KeyA", "Key1");'));
        expect(script, contains('layer_map("symbols", "KeyS", "Key2");'));
      });
    });

    group('generates combo syntax', () {
      test('generates combo with two keys', () {
        const config = VisualConfig(
          comboConfigs: [
            ComboConfig(keys: ['KeyJ', 'KeyK'], output: 'Escape'),
          ],
        );

        final script = generator.generateScript(config);

        expect(script, contains('combo(["KeyJ", "KeyK"], "Escape");'));
      });

      test('generates combo with multiple keys', () {
        const config = VisualConfig(
          comboConfigs: [
            ComboConfig(keys: ['KeyA', 'KeyS', 'KeyD'], output: 'F1'),
          ],
        );

        final script = generator.generateScript(config);

        expect(script, contains('combo(["KeyA", "KeyS", "KeyD"], "F1");'));
      });
    });

    group('generates timing config', () {
      test('generates tap timeout', () {
        const config = VisualConfig(
          timingConfig: TimingConfig(tapTimeoutMs: 200),
        );

        final script = generator.generateScript(config);

        expect(script, contains('set_tap_timeout(200);'));
      });

      test('generates hold delay', () {
        const config = VisualConfig(
          timingConfig: TimingConfig(holdDelayMs: 150),
        );

        final script = generator.generateScript(config);

        expect(script, contains('set_hold_delay(150);'));
      });

      test('generates combo timeout', () {
        const config = VisualConfig(
          timingConfig: TimingConfig(comboTimeoutMs: 50),
        );

        final script = generator.generateScript(config);

        expect(script, contains('set_combo_timeout(50);'));
      });

      test('generates boolean timing options', () {
        const config = VisualConfig(
          timingConfig: TimingConfig(
            permissiveHold: true,
            eagerTap: false,
            retroTap: true,
          ),
        );

        final script = generator.generateScript(config);

        expect(script, contains('set_permissive_hold(true);'));
        expect(script, contains('set_eager_tap(false);'));
        expect(script, contains('set_retro_tap(true);'));
      });

      test('generates all timing options together', () {
        const config = VisualConfig(
          timingConfig: TimingConfig(
            tapTimeoutMs: 200,
            holdDelayMs: 150,
            comboTimeoutMs: 50,
            permissiveHold: true,
            eagerTap: true,
            retroTap: false,
          ),
        );

        final script = generator.generateScript(config);

        expect(script, contains('set_tap_timeout(200);'));
        expect(script, contains('set_hold_delay(150);'));
        expect(script, contains('set_combo_timeout(50);'));
        expect(script, contains('set_permissive_hold(true);'));
        expect(script, contains('set_eager_tap(true);'));
        expect(script, contains('set_retro_tap(false);'));
      });
    });

    group('script structure', () {
      test('includes header comment with timestamp', () {
        const config = VisualConfig();

        final script = generator.generateScript(config);

        expect(script, contains('// KeyRx Configuration'));
        expect(script, contains('// Generated:'));
      });

      test('wraps in on_init function', () {
        const config = VisualConfig();

        final script = generator.generateScript(config);

        expect(script, contains('fn on_init() {'));
        expect(script, contains('}'));
      });

      test('includes debug print at end', () {
        const config = VisualConfig();

        final script = generator.generateScript(config);

        expect(script, contains('print_debug("Configuration loaded");'));
      });

      test('generates complete script with all features', () {
        const config = VisualConfig(
          mappings: [
            RemapConfig(
              sourceKeyId: 'CapsLock',
              targetKeyId: 'Escape',
              type: MappingType.simple,
            ),
          ],
          tapHoldConfigs: [
            TapHoldConfig(
              triggerKey: 'Space',
              tapAction: 'Space',
              holdAction: 'ShiftLeft',
            ),
          ],
          layerConfigs: [
            LayerConfig(
              name: 'nav',
              isTransparent: true,
              mappings: [LayerMapping(sourceKey: 'KeyH', action: 'Left')],
            ),
          ],
          comboConfigs: [
            ComboConfig(keys: ['KeyJ', 'KeyK'], output: 'Escape'),
          ],
          timingConfig: TimingConfig(tapTimeoutMs: 200),
        );

        final script = generator.generateScript(config);

        // Verify all sections are present
        expect(script, contains('// Key remappings'));
        expect(script, contains('remap("CapsLock", "Escape");'));
        expect(script, contains('// Tap-hold bindings'));
        expect(script, contains('tap_hold("Space", "Space", "ShiftLeft");'));
        expect(script, contains('// Layer definitions'));
        expect(script, contains('layer_define("nav", true);'));
        expect(script, contains('layer_map("nav", "KeyH", "Left");'));
        expect(script, contains('// Combo bindings'));
        expect(script, contains('combo(["KeyJ", "KeyK"], "Escape");'));
        expect(script, contains('set_tap_timeout(200);'));
      });
    });
  });

  group('Script parsing', () {
    group('parses simple script', () {
      test('parses single remap', () {
        const script = '''
fn on_init() {
    remap("KeyA", "KeyB");
}
''';

        final config = generator.parseScript(script);

        expect(config.mappings.length, 1);
        expect(config.mappings[0].sourceKeyId, 'KeyA');
        expect(config.mappings[0].targetKeyId, 'KeyB');
        expect(config.mappings[0].type, MappingType.simple);
      });

      test('parses multiple remaps', () {
        const script = '''
fn on_init() {
    remap("KeyA", "KeyB");
    remap("CapsLock", "Escape");
}
''';

        final config = generator.parseScript(script);

        expect(config.mappings.length, 2);
        expect(config.mappings[0].sourceKeyId, 'KeyA');
        expect(config.mappings[1].sourceKeyId, 'CapsLock');
        expect(config.mappings[1].targetKeyId, 'Escape');
      });

      test('parses tap-hold', () {
        const script = '''
fn on_init() {
    tap_hold("KeyA", "KeyA", "ControlLeft");
}
''';

        final config = generator.parseScript(script);

        expect(config.tapHoldConfigs.length, 1);
        expect(config.tapHoldConfigs[0].triggerKey, 'KeyA');
        expect(config.tapHoldConfigs[0].tapAction, 'KeyA');
        expect(config.tapHoldConfigs[0].holdAction, 'ControlLeft');
      });

      test('parses layer definition', () {
        const script = '''
fn on_init() {
    layer_define("nav", true);
    layer_map("nav", "KeyH", "Left");
    layer_map("nav", "KeyL", "Right");
}
''';

        final config = generator.parseScript(script);

        expect(config.layerConfigs.length, 1);
        expect(config.layerConfigs[0].name, 'nav');
        expect(config.layerConfigs[0].isTransparent, true);
        expect(config.layerConfigs[0].mappings.length, 2);
        expect(config.layerConfigs[0].mappings[0].sourceKey, 'KeyH');
        expect(config.layerConfigs[0].mappings[0].action, 'Left');
      });

      test('parses combo', () {
        const script = '''
fn on_init() {
    combo(["KeyJ", "KeyK"], "Escape");
}
''';

        final config = generator.parseScript(script);

        expect(config.comboConfigs.length, 1);
        expect(config.comboConfigs[0].keys, ['KeyJ', 'KeyK']);
        expect(config.comboConfigs[0].output, 'Escape');
      });

      test('parses combo with multiple keys', () {
        const script = '''
fn on_init() {
    combo(["KeyA", "KeyS", "KeyD", "KeyF"], "F12");
}
''';

        final config = generator.parseScript(script);

        expect(config.comboConfigs[0].keys, ['KeyA', 'KeyS', 'KeyD', 'KeyF']);
        expect(config.comboConfigs[0].output, 'F12');
      });

      test('parses timing configuration', () {
        const script = '''
fn on_init() {
    set_tap_timeout(200);
    set_hold_delay(150);
    set_combo_timeout(50);
    set_permissive_hold(true);
    set_eager_tap(false);
    set_retro_tap(true);
}
''';

        final config = generator.parseScript(script);

        expect(config.timingConfig, isNotNull);
        expect(config.timingConfig!.tapTimeoutMs, 200);
        expect(config.timingConfig!.holdDelayMs, 150);
        expect(config.timingConfig!.comboTimeoutMs, 50);
        expect(config.timingConfig!.permissiveHold, true);
        expect(config.timingConfig!.eagerTap, false);
        expect(config.timingConfig!.retroTap, true);
      });

      test('returns null timing config when no timing calls present', () {
        const script = '''
fn on_init() {
    remap("KeyA", "KeyB");
}
''';

        final config = generator.parseScript(script);

        expect(config.timingConfig, isNull);
      });
    });

    group('detects advanced features', () {
      test('detects custom functions', () {
        const script = '''
fn my_custom_function() {
    return true;
}

fn on_init() {
    remap("KeyA", "KeyB");
}
''';

        final config = generator.parseScript(script);

        expect(config.hasAdvancedFeatures, true);
      });

      test('does not flag standard functions as advanced', () {
        const script = '''
fn on_init() {
    remap("KeyA", "KeyB");
}

fn on_key() {
}
''';

        final config = generator.parseScript(script);

        expect(config.hasAdvancedFeatures, false);
      });

      test('detects let variable definitions', () {
        const script = '''
fn on_init() {
    let counter = 0;
    remap("KeyA", "KeyB");
}
''';

        final config = generator.parseScript(script);

        expect(config.hasAdvancedFeatures, true);
      });

      test('detects if statements', () {
        const script = '''
fn on_init() {
    if true {
        remap("KeyA", "KeyB");
    }
}
''';

        final config = generator.parseScript(script);

        expect(config.hasAdvancedFeatures, true);
      });

      test('detects for loops', () {
        const script = '''
fn on_init() {
    for i in 1..10 {
        print(i);
    }
}
''';

        final config = generator.parseScript(script);

        expect(config.hasAdvancedFeatures, true);
      });

      test('detects while loops', () {
        const script = '''
fn on_init() {
    while true {
        break;
    }
}
''';

        final config = generator.parseScript(script);

        expect(config.hasAdvancedFeatures, true);
      });

      test('detects loop keyword', () {
        const script = '''
fn on_init() {
    loop {
        break;
    }
}
''';

        final config = generator.parseScript(script);

        expect(config.hasAdvancedFeatures, true);
      });

      test('detects import statements', () {
        const script = '''
import "other_script.rhai";

fn on_init() {
    remap("KeyA", "KeyB");
}
''';

        final config = generator.parseScript(script);

        expect(config.hasAdvancedFeatures, true);
      });

      test('detects define_modifier', () {
        const script = '''
fn on_init() {
    define_modifier("hyper");
    remap("KeyA", "KeyB");
}
''';

        final config = generator.parseScript(script);

        expect(config.hasAdvancedFeatures, true);
      });

      test('detects tap_hold_mod', () {
        const script = '''
fn on_init() {
    tap_hold_mod("KeyA", "KeyA", "hyper");
}
''';

        final config = generator.parseScript(script);

        expect(config.hasAdvancedFeatures, true);
      });

      test('detects layer_map without layer_define', () {
        const script = '''
fn on_init() {
    layer_map("unknown_layer", "KeyA", "KeyB");
}
''';

        final config = generator.parseScript(script);

        // Should detect this as advanced because the layer is defined elsewhere
        expect(config.hasAdvancedFeatures, true);
      });

      test('simple script without advanced features', () {
        const script = '''
fn on_init() {
    remap("CapsLock", "Escape");
    tap_hold("Space", "Space", "ShiftLeft");
    layer_define("nav", true);
    layer_map("nav", "KeyH", "Left");
    combo(["KeyJ", "KeyK"], "Escape");
    set_tap_timeout(200);
}
''';

        final config = generator.parseScript(script);

        expect(config.hasAdvancedFeatures, false);
        expect(config.mappings.length, 1);
        expect(config.tapHoldConfigs.length, 1);
        expect(config.layerConfigs.length, 1);
        expect(config.comboConfigs.length, 1);
      });
    });

    group('roundtrip generation and parsing', () {
      test('simple config survives roundtrip', () {
        const original = VisualConfig(
          mappings: [
            RemapConfig(
              sourceKeyId: 'CapsLock',
              targetKeyId: 'Escape',
              type: MappingType.simple,
            ),
          ],
        );

        final script = generator.generateScript(original);
        final parsed = generator.parseScript(script);

        expect(parsed.mappings.length, 1);
        expect(parsed.mappings[0].sourceKeyId, 'CapsLock');
        expect(parsed.mappings[0].targetKeyId, 'Escape');
      });

      test('tap-hold config survives roundtrip', () {
        const original = VisualConfig(
          tapHoldConfigs: [
            TapHoldConfig(
              triggerKey: 'KeyA',
              tapAction: 'KeyA',
              holdAction: 'ControlLeft',
            ),
          ],
        );

        final script = generator.generateScript(original);
        final parsed = generator.parseScript(script);

        expect(parsed.tapHoldConfigs.length, 1);
        expect(parsed.tapHoldConfigs[0].triggerKey, 'KeyA');
        expect(parsed.tapHoldConfigs[0].tapAction, 'KeyA');
        expect(parsed.tapHoldConfigs[0].holdAction, 'ControlLeft');
      });

      test('layer config survives roundtrip', () {
        const original = VisualConfig(
          layerConfigs: [
            LayerConfig(
              name: 'nav',
              isTransparent: true,
              mappings: [LayerMapping(sourceKey: 'KeyH', action: 'Left')],
            ),
          ],
        );

        final script = generator.generateScript(original);
        final parsed = generator.parseScript(script);

        expect(parsed.layerConfigs.length, 1);
        expect(parsed.layerConfigs[0].name, 'nav');
        expect(parsed.layerConfigs[0].isTransparent, true);
        expect(parsed.layerConfigs[0].mappings.length, 1);
      });

      test('combo config survives roundtrip', () {
        const original = VisualConfig(
          comboConfigs: [
            ComboConfig(keys: ['KeyJ', 'KeyK'], output: 'Escape'),
          ],
        );

        final script = generator.generateScript(original);
        final parsed = generator.parseScript(script);

        expect(parsed.comboConfigs.length, 1);
        expect(parsed.comboConfigs[0].keys, ['KeyJ', 'KeyK']);
        expect(parsed.comboConfigs[0].output, 'Escape');
      });

      test('timing config survives roundtrip', () {
        const original = VisualConfig(
          timingConfig: TimingConfig(
            tapTimeoutMs: 200,
            holdDelayMs: 150,
            permissiveHold: true,
          ),
        );

        final script = generator.generateScript(original);
        final parsed = generator.parseScript(script);

        expect(parsed.timingConfig, isNotNull);
        expect(parsed.timingConfig!.tapTimeoutMs, 200);
        expect(parsed.timingConfig!.holdDelayMs, 150);
        expect(parsed.timingConfig!.permissiveHold, true);
      });
    });
  });

  group('VisualConfig model', () {
    test('isEmpty returns true for empty config', () {
      const config = VisualConfig();
      expect(config.isEmpty, true);
    });

    test('isEmpty returns false when mappings exist', () {
      const config = VisualConfig(
        mappings: [
          RemapConfig(
            sourceKeyId: 'KeyA',
            targetKeyId: 'KeyB',
            type: MappingType.simple,
          ),
        ],
      );
      expect(config.isEmpty, false);
    });

    test('isEmpty returns false when tap-holds exist', () {
      const config = VisualConfig(
        tapHoldConfigs: [
          TapHoldConfig(
            triggerKey: 'KeyA',
            tapAction: 'KeyA',
            holdAction: 'Ctrl',
          ),
        ],
      );
      expect(config.isEmpty, false);
    });

    test('isEmpty returns false when layers exist', () {
      const config = VisualConfig(
        layerConfigs: [LayerConfig(name: 'test', mappings: [])],
      );
      expect(config.isEmpty, false);
    });

    test('isEmpty returns false when combos exist', () {
      const config = VisualConfig(
        comboConfigs: [
          ComboConfig(keys: ['KeyA', 'KeyB'], output: 'KeyC'),
        ],
      );
      expect(config.isEmpty, false);
    });

    test('copyWith creates modified copy', () {
      const original = VisualConfig(
        mappings: [
          RemapConfig(
            sourceKeyId: 'KeyA',
            targetKeyId: 'KeyB',
            type: MappingType.simple,
          ),
        ],
        hasAdvancedFeatures: false,
      );

      final modified = original.copyWith(hasAdvancedFeatures: true);

      expect(original.hasAdvancedFeatures, false);
      expect(modified.hasAdvancedFeatures, true);
      expect(modified.mappings.length, 1);
    });

    test('copyWith preserves unmodified fields', () {
      const original = VisualConfig(
        mappings: [
          RemapConfig(
            sourceKeyId: 'KeyA',
            targetKeyId: 'KeyB',
            type: MappingType.simple,
          ),
        ],
        timingConfig: TimingConfig(tapTimeoutMs: 200),
      );

      final modified = original.copyWith(
        tapHoldConfigs: [
          TapHoldConfig(
            triggerKey: 'KeyC',
            tapAction: 'KeyC',
            holdAction: 'Ctrl',
          ),
        ],
      );

      expect(modified.mappings.length, 1);
      expect(modified.tapHoldConfigs.length, 1);
      expect(modified.timingConfig?.tapTimeoutMs, 200);
    });
  });

  group('TapHoldConfig model', () {
    test('equality based on all fields', () {
      const config1 = TapHoldConfig(
        triggerKey: 'KeyA',
        tapAction: 'KeyA',
        holdAction: 'Ctrl',
      );
      const config2 = TapHoldConfig(
        triggerKey: 'KeyA',
        tapAction: 'KeyA',
        holdAction: 'Ctrl',
      );
      const config3 = TapHoldConfig(
        triggerKey: 'KeyA',
        tapAction: 'KeyA',
        holdAction: 'Shift',
      );

      expect(config1, equals(config2));
      expect(config1, isNot(equals(config3)));
    });

    test('hashCode is consistent', () {
      const config1 = TapHoldConfig(
        triggerKey: 'KeyA',
        tapAction: 'KeyA',
        holdAction: 'Ctrl',
      );
      const config2 = TapHoldConfig(
        triggerKey: 'KeyA',
        tapAction: 'KeyA',
        holdAction: 'Ctrl',
      );

      expect(config1.hashCode, equals(config2.hashCode));
    });
  });
}
