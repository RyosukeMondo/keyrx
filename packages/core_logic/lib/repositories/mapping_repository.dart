/// Repository for key mappings as a single source of truth.
///
/// Provides CRUD operations for key mappings, combos, and tap-hold configs.
library;

import 'dart:async';

import '../models/key_mapping.dart';

/// Single source of truth for all mapping data.
class MappingRepository {
  final Map<String, KeyMapping> _mappings = {};
  final List<ComboMapping> _combos = [];
  final List<TapHoldMapping> _tapHolds = [];
  final StreamController<void> _changesController =
      StreamController.broadcast();

  /// Stream of changes to mappings.
  Stream<void> get changes => _changesController.stream;

  /// All key mappings, keyed by source key.
  Map<String, KeyMapping> get mappings => Map.unmodifiable(_mappings);

  /// All combo configurations.
  List<ComboMapping> get combos => List.unmodifiable(_combos);

  /// All tap-hold configurations.
  List<TapHoldMapping> get tapHolds => List.unmodifiable(_tapHolds);

  /// Add or update a key mapping.
  void setMapping(String key, KeyMapping mapping) {
    _mappings[key] = mapping;
    _changesController.add(null);
  }

  /// Remove a key mapping.
  void removeMapping(String key) {
    if (_mappings.remove(key) != null) {
      _changesController.add(null);
    }
  }

  /// Get a mapping by key.
  KeyMapping? getMapping(String key) => _mappings[key];

  /// Add a combo configuration.
  void addCombo(ComboMapping combo) {
    _combos.add(combo);
    _changesController.add(null);
  }

  /// Remove a combo at index.
  void removeCombo(int index) {
    if (index >= 0 && index < _combos.length) {
      _combos.removeAt(index);
      _changesController.add(null);
    }
  }

  /// Add a tap-hold configuration.
  void addTapHold(TapHoldMapping tapHold) {
    _tapHolds.add(tapHold);
    _changesController.add(null);
  }

  /// Remove a tap-hold at index.
  void removeTapHold(int index) {
    if (index >= 0 && index < _tapHolds.length) {
      _tapHolds.removeAt(index);
      _changesController.add(null);
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
      _changesController.add(null);
    }
  }

  /// Dispose resources.
  void dispose() {
    _changesController.close();
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
