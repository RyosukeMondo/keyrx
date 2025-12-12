/// Converts Keymap models to VisualConfig for Rhai script generation.
library;

import '../models/keymap.dart';
import 'rhai_generator.dart';
import '../widgets/mapping_overlay.dart'; // Implements RemapConfig

/// Converts `Keymap` models to `VisualConfig` for script generation.
class KeymapConverter {
  const KeymapConverter();

  /// Convert a [Keymap] to a [VisualConfig].
  VisualConfig convert(Keymap keymap) {
    final mappings = <RemapConfig>[];
    final layerConfigs = <LayerConfig>[];
    final tapHoldConfigs = <TapHoldConfig>[];
    final comboConfigs = <ComboConfig>[];

    // Process all layers
    for (var i = 0; i < keymap.layers.length; i++) {
      final layer = keymap.layers[i];

      // Extract tap-hold configs from all layers (though usually they are on base)
      // If we encounter a tap-hold, we add it to tapHoldConfigs
      for (final entry in layer.bindings.entries) {
        final source = entry.key;
        final binding = entry.value;

        binding.maybeMap(
          tapHold: (b) {
            if (b.value.length == 2) {
              tapHoldConfigs.add(
                TapHoldConfig(
                  triggerKey: source,
                  tapAction: b.value[0],
                  holdAction: b.value[1],
                ),
              );
            }
          },
          orElse: () {},
        );
      }

      // Layer 0 is the base layer, treated as global remaps for simple cases
      if (i == 0) {
        mappings.addAll(_extractBaseMappings(layer));
      } else {
        layerConfigs.add(_extractLayerConfig(layer, i));
      }
    }

    // Extract combo configs
    for (final combo in keymap.combos) {
      comboConfigs.add(
        ComboConfig(
          keys: combo.keys,
          output: combo.output,
        ),
      );
    }

    return VisualConfig(
      mappings: mappings,
      layerConfigs: layerConfigs,
      tapHoldConfigs: tapHoldConfigs,
      comboConfigs: comboConfigs,
    );
  }

  List<RemapConfig> _extractBaseMappings(KeymapLayer layer) {
    final remaps = <RemapConfig>[];

    for (final entry in layer.bindings.entries) {
      final source = entry.key;
      final binding = entry.value;

      binding.map(
        standardKey: (b) {
          remaps.add(
            RemapConfig(
              sourceKeyId: source,
              targetKeyId: b.value,
              type: MappingType.simple,
            ),
          );
        },
        macro: (_) {
          // Macros not yet supported in simple remaps
          // TODO: Implement macro support
        },
        layerToggle: (_) {
          // Layer toggles on base layer not supported in simple remaps yet
        },
        tapHold: (_) {
          // Tap-Hold handled separately
        },
        transparent: (_) {
          // Transparent on base layer means no-op (pass-through)
        },
      );
    }
    return remaps;
  }

  LayerConfig _extractLayerConfig(KeymapLayer layer, int index) {
    final layerMappings = <LayerMapping>[];

    for (final entry in layer.bindings.entries) {
      final source = entry.key;
      final binding = entry.value;

      binding.maybeMap(
        standardKey: (b) {
          layerMappings.add(LayerMapping(sourceKey: source, action: b.value));
        },
        orElse: () {
          // TODO: Support other binding types in layers
        },
      );
    }

    return LayerConfig(
      name: layer.name,
      isTransparent: true, // Default to transparent for overlay layers
      mappings: layerMappings,
    );
  }
}
