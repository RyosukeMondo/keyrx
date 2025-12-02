/// Rhai script generator service for visual editor.
///
/// Generates Rhai scripts from visual configuration and parses
/// simple scripts back to visual configuration (best-effort).
library;

import '../widgets/mapping_overlay.dart';

/// Configuration from visual editor that can be converted to/from Rhai.
class VisualConfig {
  const VisualConfig({
    this.mappings = const [],
    this.tapHoldConfigs = const [],
    this.layerConfigs = const [],
    this.comboConfigs = const [],
    this.timingConfig,
    this.hasAdvancedFeatures = false,
  });

  /// Simple key remappings.
  final List<RemapConfig> mappings;

  /// Tap-hold configurations.
  final List<TapHoldConfig> tapHoldConfigs;

  /// Layer configurations.
  final List<LayerConfig> layerConfigs;

  /// Combo configurations.
  final List<ComboConfig> comboConfigs;

  /// Timing configuration.
  final TimingConfig? timingConfig;

  /// True if the script contains features that cannot be visually edited.
  final bool hasAdvancedFeatures;

  /// Create a copy with modified fields.
  VisualConfig copyWith({
    List<RemapConfig>? mappings,
    List<TapHoldConfig>? tapHoldConfigs,
    List<LayerConfig>? layerConfigs,
    List<ComboConfig>? comboConfigs,
    TimingConfig? timingConfig,
    bool? hasAdvancedFeatures,
  }) {
    return VisualConfig(
      mappings: mappings ?? this.mappings,
      tapHoldConfigs: tapHoldConfigs ?? this.tapHoldConfigs,
      layerConfigs: layerConfigs ?? this.layerConfigs,
      comboConfigs: comboConfigs ?? this.comboConfigs,
      timingConfig: timingConfig ?? this.timingConfig,
      hasAdvancedFeatures: hasAdvancedFeatures ?? this.hasAdvancedFeatures,
    );
  }

  /// Check if config is empty.
  bool get isEmpty =>
      mappings.isEmpty &&
      tapHoldConfigs.isEmpty &&
      layerConfigs.isEmpty &&
      comboConfigs.isEmpty;
}

/// Tap-hold key configuration.
class TapHoldConfig {
  const TapHoldConfig({
    required this.triggerKey,
    required this.tapAction,
    required this.holdAction,
  });

  final String triggerKey;
  final String tapAction;
  final String holdAction;

  @override
  bool operator ==(Object other) =>
      identical(this, other) ||
      other is TapHoldConfig &&
          triggerKey == other.triggerKey &&
          tapAction == other.tapAction &&
          holdAction == other.holdAction;

  @override
  int get hashCode => Object.hash(triggerKey, tapAction, holdAction);
}

/// Layer configuration.
class LayerConfig {
  const LayerConfig({
    required this.name,
    this.isTransparent = true,
    this.mappings = const [],
  });

  final String name;
  final bool isTransparent;
  final List<LayerMapping> mappings;
}

/// A mapping within a layer.
class LayerMapping {
  const LayerMapping({
    required this.sourceKey,
    required this.action,
  });

  final String sourceKey;
  final String action;
}

/// Combo (chord) configuration.
class ComboConfig {
  const ComboConfig({
    required this.keys,
    required this.output,
  });

  final List<String> keys;
  final String output;
}

/// Timing configuration.
class TimingConfig {
  const TimingConfig({
    this.tapTimeoutMs,
    this.holdDelayMs,
    this.comboTimeoutMs,
    this.permissiveHold,
    this.eagerTap,
    this.retroTap,
  });

  final int? tapTimeoutMs;
  final int? holdDelayMs;
  final int? comboTimeoutMs;
  final bool? permissiveHold;
  final bool? eagerTap;
  final bool? retroTap;
}

/// Rhai script generator and parser.
class RhaiGenerator {
  RhaiGenerator();

  /// Generate Rhai script from visual configuration.
  String generateScript(VisualConfig config) {
    final buffer = StringBuffer();

    // Header comment
    buffer.writeln('// KeyRx Configuration');
    buffer.writeln('// Generated: ${DateTime.now().toIso8601String()}');
    buffer.writeln('// Edit visually or modify this script directly.');
    buffer.writeln();

    // Main init function
    buffer.writeln('fn on_init() {');

    // Timing configuration
    if (config.timingConfig != null) {
      _generateTimingConfig(buffer, config.timingConfig!);
    }

    // Simple remaps
    if (config.mappings.isNotEmpty) {
      buffer.writeln('    // Key remappings');
      for (final mapping in config.mappings) {
        if (mapping.type == MappingType.simple) {
          _generateRemap(buffer, mapping);
        }
      }
      buffer.writeln();
    }

    // Tap-hold configurations
    if (config.tapHoldConfigs.isNotEmpty) {
      buffer.writeln('    // Tap-hold bindings');
      for (final tapHold in config.tapHoldConfigs) {
        _generateTapHold(buffer, tapHold);
      }
      buffer.writeln();
    }

    // Layer definitions
    if (config.layerConfigs.isNotEmpty) {
      buffer.writeln('    // Layer definitions');
      for (final layer in config.layerConfigs) {
        _generateLayer(buffer, layer);
      }
      buffer.writeln();
    }

    // Combo configurations
    if (config.comboConfigs.isNotEmpty) {
      buffer.writeln('    // Combo bindings');
      for (final combo in config.comboConfigs) {
        _generateCombo(buffer, combo);
      }
      buffer.writeln();
    }

    buffer.writeln('    print_debug("Configuration loaded");');
    buffer.writeln('}');

    return buffer.toString();
  }

  void _generateTimingConfig(StringBuffer buffer, TimingConfig timing) {
    buffer.writeln('    // Timing configuration');
    if (timing.tapTimeoutMs != null) {
      buffer.writeln('    set_tap_timeout(${timing.tapTimeoutMs});');
    }
    if (timing.holdDelayMs != null) {
      buffer.writeln('    set_hold_delay(${timing.holdDelayMs});');
    }
    if (timing.comboTimeoutMs != null) {
      buffer.writeln('    set_combo_timeout(${timing.comboTimeoutMs});');
    }
    if (timing.permissiveHold != null) {
      buffer.writeln('    set_permissive_hold(${timing.permissiveHold});');
    }
    if (timing.eagerTap != null) {
      buffer.writeln('    set_eager_tap(${timing.eagerTap});');
    }
    if (timing.retroTap != null) {
      buffer.writeln('    set_retro_tap(${timing.retroTap});');
    }
    buffer.writeln();
  }

  void _generateRemap(StringBuffer buffer, RemapConfig mapping) {
    final source = _escapeKeyName(mapping.sourceKeyId);
    final target = _escapeKeyName(mapping.targetKeyId);
    buffer.writeln('    remap("$source", "$target");');
  }

  void _generateTapHold(StringBuffer buffer, TapHoldConfig tapHold) {
    final trigger = _escapeKeyName(tapHold.triggerKey);
    final tap = _escapeKeyName(tapHold.tapAction);
    final hold = _escapeKeyName(tapHold.holdAction);
    buffer.writeln('    tap_hold("$trigger", "$tap", "$hold");');
  }

  void _generateLayer(StringBuffer buffer, LayerConfig layer) {
    final name = _escapeString(layer.name);
    buffer.writeln('    layer_define("$name", ${layer.isTransparent});');

    for (final mapping in layer.mappings) {
      final key = _escapeKeyName(mapping.sourceKey);
      final action = _escapeString(mapping.action);
      buffer.writeln('    layer_map("$name", "$key", "$action");');
    }
  }

  void _generateCombo(StringBuffer buffer, ComboConfig combo) {
    final keys = combo.keys.map((k) => '"${_escapeKeyName(k)}"').join(', ');
    final output = _escapeKeyName(combo.output);
    buffer.writeln('    combo([$keys], "$output");');
  }

  String _escapeKeyName(String key) {
    return key.replaceAll('"', r'\"').replaceAll(r'\', r'\\');
  }

  String _escapeString(String s) {
    return s.replaceAll(r'\', r'\\').replaceAll('"', r'\"');
  }

  /// Parse a Rhai script and extract visual configuration (best-effort).
  ///
  /// Returns a [VisualConfig] with [hasAdvancedFeatures] set to true if
  /// the script contains constructs that cannot be represented visually.
  VisualConfig parseScript(String script) {
    final mappings = <RemapConfig>[];
    final tapHoldConfigs = <TapHoldConfig>[];
    final layerConfigs = <LayerConfig>[];
    final comboConfigs = <ComboConfig>[];
    TimingConfig? timingConfig;
    var hasAdvancedFeatures = false;

    // Track layer definitions for later mapping association
    final layerDefs = <String, LayerConfig>{};

    // Parse timing configuration
    timingConfig = _parseTimingConfig(script);

    // Parse simple remaps: remap("KeyA", "KeyB")
    final remapRegex = RegExp(r'remap\s*\(\s*"([^"]+)"\s*,\s*"([^"]+)"\s*\)');
    for (final match in remapRegex.allMatches(script)) {
      mappings.add(RemapConfig(
        sourceKeyId: match.group(1)!,
        targetKeyId: match.group(2)!,
        type: MappingType.simple,
      ));
    }

    // Parse tap_hold: tap_hold("Key", "TapAction", "HoldAction")
    final tapHoldRegex = RegExp(
      r'tap_hold\s*\(\s*"([^"]+)"\s*,\s*"([^"]+)"\s*,\s*"([^"]+)"\s*\)',
    );
    for (final match in tapHoldRegex.allMatches(script)) {
      tapHoldConfigs.add(TapHoldConfig(
        triggerKey: match.group(1)!,
        tapAction: match.group(2)!,
        holdAction: match.group(3)!,
      ));
    }

    // Parse layer_define: layer_define("name", true/false)
    final layerDefRegex = RegExp(
      r'layer_define\s*\(\s*"([^"]+)"\s*,\s*(true|false)\s*\)',
    );
    for (final match in layerDefRegex.allMatches(script)) {
      final name = match.group(1)!;
      final isTransparent = match.group(2) == 'true';
      layerDefs[name] = LayerConfig(
        name: name,
        isTransparent: isTransparent,
        mappings: [],
      );
    }

    // Parse layer_map: layer_map("layerName", "Key", "Action")
    final layerMapRegex = RegExp(
      r'layer_map\s*\(\s*"([^"]+)"\s*,\s*"([^"]+)"\s*,\s*"([^"]+)"\s*\)',
    );
    for (final match in layerMapRegex.allMatches(script)) {
      final layerName = match.group(1)!;
      final sourceKey = match.group(2)!;
      final action = match.group(3)!;

      if (layerDefs.containsKey(layerName)) {
        final current = layerDefs[layerName]!;
        layerDefs[layerName] = LayerConfig(
          name: current.name,
          isTransparent: current.isTransparent,
          mappings: [
            ...current.mappings,
            LayerMapping(sourceKey: sourceKey, action: action),
          ],
        );
      } else {
        // Layer defined elsewhere or dynamically
        hasAdvancedFeatures = true;
      }
    }

    layerConfigs.addAll(layerDefs.values);

    // Parse combo: combo(["K1", "K2"], "Output")
    final comboRegex = RegExp(
      r'combo\s*\(\s*\[\s*([^\]]+)\s*\]\s*,\s*"([^"]+)"\s*\)',
    );
    for (final match in comboRegex.allMatches(script)) {
      final keysStr = match.group(1)!;
      final output = match.group(2)!;

      // Parse individual keys from the array
      final keyRegex = RegExp(r'"([^"]+)"');
      final keys = keyRegex
          .allMatches(keysStr)
          .map((m) => m.group(1)!)
          .toList();

      if (keys.isNotEmpty) {
        comboConfigs.add(ComboConfig(keys: keys, output: output));
      }
    }

    // Check for advanced features that can't be represented visually
    hasAdvancedFeatures = hasAdvancedFeatures || _hasAdvancedFeatures(script);

    return VisualConfig(
      mappings: mappings,
      tapHoldConfigs: tapHoldConfigs,
      layerConfigs: layerConfigs,
      comboConfigs: comboConfigs,
      timingConfig: timingConfig,
      hasAdvancedFeatures: hasAdvancedFeatures,
    );
  }

  TimingConfig? _parseTimingConfig(String script) {
    int? tapTimeoutMs;
    int? holdDelayMs;
    int? comboTimeoutMs;
    bool? permissiveHold;
    bool? eagerTap;
    bool? retroTap;

    // Parse set_tap_timeout(value)
    final tapTimeoutRegex = RegExp(r'set_tap_timeout\s*\(\s*(\d+)\s*\)');
    final tapTimeoutMatch = tapTimeoutRegex.firstMatch(script);
    if (tapTimeoutMatch != null) {
      tapTimeoutMs = int.tryParse(tapTimeoutMatch.group(1)!);
    }

    // Parse set_hold_delay(value)
    final holdDelayRegex = RegExp(r'set_hold_delay\s*\(\s*(\d+)\s*\)');
    final holdDelayMatch = holdDelayRegex.firstMatch(script);
    if (holdDelayMatch != null) {
      holdDelayMs = int.tryParse(holdDelayMatch.group(1)!);
    }

    // Parse set_combo_timeout(value)
    final comboTimeoutRegex = RegExp(r'set_combo_timeout\s*\(\s*(\d+)\s*\)');
    final comboTimeoutMatch = comboTimeoutRegex.firstMatch(script);
    if (comboTimeoutMatch != null) {
      comboTimeoutMs = int.tryParse(comboTimeoutMatch.group(1)!);
    }

    // Parse set_permissive_hold(bool)
    final permissiveHoldRegex =
        RegExp(r'set_permissive_hold\s*\(\s*(true|false)\s*\)');
    final permissiveHoldMatch = permissiveHoldRegex.firstMatch(script);
    if (permissiveHoldMatch != null) {
      permissiveHold = permissiveHoldMatch.group(1) == 'true';
    }

    // Parse set_eager_tap(bool)
    final eagerTapRegex = RegExp(r'set_eager_tap\s*\(\s*(true|false)\s*\)');
    final eagerTapMatch = eagerTapRegex.firstMatch(script);
    if (eagerTapMatch != null) {
      eagerTap = eagerTapMatch.group(1) == 'true';
    }

    // Parse set_retro_tap(bool)
    final retroTapRegex = RegExp(r'set_retro_tap\s*\(\s*(true|false)\s*\)');
    final retroTapMatch = retroTapRegex.firstMatch(script);
    if (retroTapMatch != null) {
      retroTap = retroTapMatch.group(1) == 'true';
    }

    // Return null if no timing config found
    if (tapTimeoutMs == null &&
        holdDelayMs == null &&
        comboTimeoutMs == null &&
        permissiveHold == null &&
        eagerTap == null &&
        retroTap == null) {
      return null;
    }

    return TimingConfig(
      tapTimeoutMs: tapTimeoutMs,
      holdDelayMs: holdDelayMs,
      comboTimeoutMs: comboTimeoutMs,
      permissiveHold: permissiveHold,
      eagerTap: eagerTap,
      retroTap: retroTap,
    );
  }

  bool _hasAdvancedFeatures(String script) {
    // Check for constructs that cannot be represented in the visual editor

    // Custom functions (other than on_init, init, on_key, on_window_change)
    final fnRegex = RegExp(r'\bfn\s+(\w+)\s*\(');
    final knownFunctions = {'on_init', 'init', 'on_key', 'on_window_change'};
    for (final match in fnRegex.allMatches(script)) {
      final funcName = match.group(1)!;
      if (!knownFunctions.contains(funcName)) {
        return true;
      }
    }

    // Variable definitions with let
    if (RegExp(r'\blet\s+\w+\s*=').hasMatch(script)) {
      return true;
    }

    // Conditional logic
    if (RegExp(r'\bif\s+').hasMatch(script)) {
      return true;
    }

    // Loops
    if (RegExp(r'\b(for|while|loop)\s+').hasMatch(script)) {
      return true;
    }

    // Import statements
    if (RegExp(r'\bimport\s+').hasMatch(script)) {
      return true;
    }

    // define_modifier (custom virtual modifiers)
    if (RegExp(r'\bdefine_modifier\s*\(').hasMatch(script)) {
      return true;
    }

    // tap_hold_mod (uses custom modifiers)
    if (RegExp(r'\btap_hold_mod\s*\(').hasMatch(script)) {
      return true;
    }

    return false;
  }
}
