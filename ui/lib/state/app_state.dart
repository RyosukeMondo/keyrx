/// Application state management.
///
/// Manages the global application state including
/// engine connection status, loaded scripts, and UI state.

import 'package:flutter/foundation.dart';

import '../ffi/bridge.dart';
import '../widgets/layer_panel.dart';

/// Global application state.
class AppState extends ChangeNotifier {
  bool _initialized = false;
  String? _loadedScript;
  List<LayerInfo> _layers = [];
  String? _error;

  /// Whether the engine is initialized.
  bool get initialized => _initialized;

  /// Currently loaded script path.
  String? get loadedScript => _loadedScript;

  /// Current layers.
  List<LayerInfo> get layers => List.unmodifiable(_layers);

  /// Current error message, if any.
  String? get error => _error;

  /// Core library version.
  String get version {
    if (!_initialized) return 'Not initialized';
    return KeyrxBridge.instance.version;
  }

  /// Initialize the engine.
  Future<bool> initialize() async {
    try {
      _error = null;
      _initialized = KeyrxBridge.instance.initialize();

      if (_initialized) {
        // Load default layers
        _layers = [
          const LayerInfo(name: 'base', active: true, priority: 0),
        ];
      }

      notifyListeners();
      return _initialized;
    } catch (e) {
      _error = e.toString();
      notifyListeners();
      return false;
    }
  }

  /// Load a script file.
  Future<bool> loadScript(String path) async {
    if (!_initialized) {
      _error = 'Engine not initialized';
      notifyListeners();
      return false;
    }

    try {
      _error = null;
      final success = KeyrxBridge.instance.loadScript(path);

      if (success) {
        _loadedScript = path;
      } else {
        _error = 'Failed to load script';
      }

      notifyListeners();
      return success;
    } catch (e) {
      _error = e.toString();
      notifyListeners();
      return false;
    }
  }

  /// Toggle a layer's active state.
  void toggleLayer(String name, bool active) {
    final index = _layers.indexWhere((l) => l.name == name);
    if (index >= 0) {
      _layers[index] = LayerInfo(
        name: name,
        active: active,
        priority: _layers[index].priority,
      );
      notifyListeners();
    }
  }

  /// Add a new layer.
  void addLayer(String name, {int priority = 0}) {
    _layers.add(LayerInfo(
      name: name,
      active: false,
      priority: priority,
    ));
    notifyListeners();
  }

  /// Clear any error.
  void clearError() {
    _error = null;
    notifyListeners();
  }
}
