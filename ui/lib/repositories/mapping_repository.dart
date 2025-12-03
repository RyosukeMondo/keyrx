/// Repository for key mappings as a single source of truth.
///
/// Provides CRUD operations for key mappings, combos, and tap-hold configs.
/// Both EditorPage and VisualEditorPage can share this repository.
library;

import 'package:flutter/foundation.dart';

import '../pages/editor_widgets.dart';
import '../widgets/visual_keyboard.dart';

/// Single source of truth for all mapping data.
class MappingRepository extends ChangeNotifier {
  final Map<String, KeyMapping> _mappings = {};
  final List<ComboMapping> _combos = [];
  final List<TapHoldMapping> _tapHolds = [];

  /// All key mappings, keyed by source key.
  Map<String, KeyMapping> get mappings => Map.unmodifiable(_mappings);

  /// All combo configurations.
  List<ComboMapping> get combos => List.unmodifiable(_combos);

  /// All tap-hold configurations.
  List<TapHoldMapping> get tapHolds => List.unmodifiable(_tapHolds);

  /// Add or update a key mapping.
  void setMapping(String key, KeyMapping mapping) {
    _mappings[key] = mapping;
    notifyListeners();
  }

  /// Remove a key mapping.
  void removeMapping(String key) {
    if (_mappings.remove(key) != null) {
      notifyListeners();
    }
  }

  /// Get a mapping by key.
  KeyMapping? getMapping(String key) => _mappings[key];

  /// Add a combo configuration.
  void addCombo(ComboMapping combo) {
    _combos.add(combo);
    notifyListeners();
  }

  /// Remove a combo at index.
  void removeCombo(int index) {
    if (index >= 0 && index < _combos.length) {
      _combos.removeAt(index);
      notifyListeners();
    }
  }

  /// Add a tap-hold configuration.
  void addTapHold(TapHoldMapping tapHold) {
    _tapHolds.add(tapHold);
    notifyListeners();
  }

  /// Remove a tap-hold at index.
  void removeTapHold(int index) {
    if (index >= 0 && index < _tapHolds.length) {
      _tapHolds.removeAt(index);
      notifyListeners();
    }
  }

  /// Clear all mappings, combos, and tap-holds.
  void clear() {
    final hadData =
        _mappings.isNotEmpty || _combos.isNotEmpty || _tapHolds.isNotEmpty;
    _mappings.clear();
    _combos.clear();
    _tapHolds.clear();
    if (hadData) {
      notifyListeners();
    }
  }

  /// Generate a Rhai script from current mappings.
  String generateScript() {
    return ScriptGenerator.build(mappings: _mappings.values, combos: _combos);
  }

  /// Load mappings from visual editor remap configs.
  void loadFromRemapConfigs(List<RemapConfig> remaps) {
    _mappings.clear();
    for (final remap in remaps) {
      _mappings[remap.sourceKeyId] = KeyMapping(
        from: remap.sourceKeyId,
        type: KeyActionType.remap,
        to: remap.targetKeyId,
      );
    }
    notifyListeners();
  }

  /// Export to visual editor remap configs.
  List<RemapConfig> toRemapConfigs() {
    return _mappings.values
        .where((m) => m.type == KeyActionType.remap && m.to != null)
        .map((m) => RemapConfig(sourceKeyId: m.from, targetKeyId: m.to!))
        .toList();
  }
}

/// Tap-hold mapping configuration.
class TapHoldMapping {
  const TapHoldMapping({
    required this.triggerKey,
    required this.tapAction,
    required this.holdAction,
  });

  final String triggerKey;
  final String tapAction;
  final String holdAction;
}
