/// Application state management.
///
/// Manages the global application state including
/// engine connection status, loaded scripts, and UI state.

import 'package:flutter/foundation.dart';
import 'package:shared_preferences/shared_preferences.dart';

import '../config/config.dart';
import '../services/engine_service.dart';
import '../services/error_translator.dart';
import '../widgets/editor/layer_panel.dart';

/// Global application state.
class AppState extends ChangeNotifier {
  AppState({
    required EngineService engineService,
    required ErrorTranslator errorTranslator,
  })  : _engineService = engineService,
        _errorTranslator = errorTranslator;

  final EngineService _engineService;
  final ErrorTranslator _errorTranslator;
  bool _initialized = false;
  String? _loadedScript;
  List<LayerInfo> _layers = [];
  String? _error;
  bool _isDeveloperMode = false;

  /// Whether the engine is initialized.
  bool get initialized => _initialized || _engineService.isInitialized;

  /// Currently loaded script path.
  String? get loadedScript => _loadedScript;

  /// Current layers.
  List<LayerInfo> get layers => List.unmodifiable(_layers);

  /// Current error message, if any.
  String? get error => _error;

  /// Whether developer mode is enabled.
  bool get isDeveloperMode => _isDeveloperMode;

  /// Core library version.
  String get version {
    if (!initialized) return 'Not initialized';
    return _engineService.version;
  }

  /// Initialize the engine.
  Future<bool> initialize() async {
    try {
      _error = null;
      _initialized = await _engineService.initialize();

      if (_initialized) {
        // Load default layers
        _layers = [
          const LayerInfo(name: 'base', active: true, priority: 0),
        ];
      } else {
        _error =
            _errorTranslator.translate(StateError('Engine not ready')).body;
      }

      notifyListeners();
      return _initialized;
    } catch (e) {
      _error = _errorTranslator.translate(e).body;
      notifyListeners();
      return false;
    }
  }

  /// Load a script file.
  Future<bool> loadScript(String path) async {
    if (!initialized) {
      _error =
          _errorTranslator.translate(StateError('Engine not ready')).body;
      notifyListeners();
      return false;
    }

    try {
      _error = null;
      final success = await _engineService.loadScript(path);

      if (success) {
        _loadedScript = path;
      } else {
        _error =
            _errorTranslator.translate(StateError('Failed to load $path')).body;
      }

      notifyListeners();
      return success;
    } catch (e) {
      _error = _errorTranslator.translate(e).body;
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

  /// Load developer mode setting from persistent storage.
  Future<void> loadDeveloperMode() async {
    final prefs = await SharedPreferences.getInstance();
    _isDeveloperMode = prefs.getBool(StorageKeys.developerModeKey) ?? false;
    notifyListeners();
  }

  /// Toggle developer mode on/off.
  Future<void> toggleDeveloperMode() async {
    _isDeveloperMode = !_isDeveloperMode;
    final prefs = await SharedPreferences.getInstance();
    await prefs.setBool(StorageKeys.developerModeKey, _isDeveloperMode);
    notifyListeners();
  }
}
