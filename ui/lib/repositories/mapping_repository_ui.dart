/// UI wrapper for MappingRepository that provides ChangeNotifier compatibility.
library;

import 'dart:async';

import 'package:core_logic/core_logic.dart' as core;
import 'package:core_logic/models/key_mapping.dart';
import 'package:flutter/foundation.dart';

import '../services/script_generator.dart';
import '../widgets/visual_keyboard.dart';

/// UI wrapper for [core.MappingRepository] that extends ChangeNotifier.
///
/// This wrapper listens to the core repository's stream and calls
/// notifyListeners() to maintain compatibility with Riverpod providers
/// that expect ChangeNotifier.
class MappingRepositoryUI extends ChangeNotifier {
  MappingRepositoryUI(this._core) {
    _subscription = _core.changes.listen((_) => notifyListeners());
  }

  final core.MappingRepository _core;
  StreamSubscription<void>? _subscription;

  /// All key mappings, keyed by source key.
  Map<String, KeyMapping> get mappings => _core.mappings;

  /// All combo configurations.
  List<ComboMapping> get combos => _core.combos;

  /// All tap-hold configurations.
  List<core.TapHoldMapping> get tapHolds => _core.tapHolds;

  /// Add or update a key mapping.
  void setMapping(String key, KeyMapping mapping) =>
      _core.setMapping(key, mapping);

  /// Remove a key mapping.
  void removeMapping(String key) => _core.removeMapping(key);

  /// Get a mapping by key.
  KeyMapping? getMapping(String key) => _core.getMapping(key);

  /// Add a combo configuration.
  void addCombo(ComboMapping combo) => _core.addCombo(combo);

  /// Remove a combo at index.
  void removeCombo(int index) => _core.removeCombo(index);

  /// Add a tap-hold configuration.
  void addTapHold(core.TapHoldMapping tapHold) => _core.addTapHold(tapHold);

  /// Remove a tap-hold at index.
  void removeTapHold(int index) => _core.removeTapHold(index);

  /// Clear all mappings, combos, and tap-holds.
  void clear() => _core.clear();

  /// Generate a Rhai script from current mappings.
  String generateScript() =>
      ScriptGenerator.build(mappings: mappings.values, combos: combos);

  /// Load mappings from visual editor remap configs.
  void loadFromRemapConfigs(List<RemapConfig> remaps) {
    _core.clear();
    for (final remap in remaps) {
      _core.setMapping(
        remap.sourceKeyId,
        KeyMapping(
          from: remap.sourceKeyId,
          type: KeyActionType.remap,
          to: remap.targetKeyId,
        ),
      );
    }
  }

  /// Export to visual editor remap configs.
  List<RemapConfig> toRemapConfigs() => mappings.values
      .where((m) => m.type == KeyActionType.remap && m.to != null)
      .map((m) => RemapConfig(sourceKeyId: m.from, targetKeyId: m.to!))
      .toList();

  @override
  void dispose() {
    _subscription?.cancel();
    _core.dispose();
    super.dispose();
  }
}
