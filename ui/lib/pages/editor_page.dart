/// Visual keymap editor page.
///
/// Provides a drag-and-drop interface for creating key mappings
/// that generates the underlying Rhai script automatically.

import 'dart:io';

import 'package:flutter/material.dart';

import '../services/engine_service.dart';
import '../widgets/keyboard.dart';
import '../widgets/layer_panel.dart';
import 'editor_widgets.dart';

/// Visual keymap editor page.
class EditorPage extends StatefulWidget {
  const EditorPage({
    super.key,
    required this.engineService,
  });

  /// The engine service for key registry and script loading.
  final EngineService engineService;

  @override
  State<EditorPage> createState() => _EditorPageState();
}

class _EditorPageState extends State<EditorPage> {
  String? _selectedKey;
  KeyActionType _selectedAction = KeyActionType.remap;
  final TextEditingController _outputController = TextEditingController();
  final TextEditingController _layerController = TextEditingController();
  final TextEditingController _tapOutputController = TextEditingController();
  final TextEditingController _holdOutputController = TextEditingController();
  final TextEditingController _comboKeysController = TextEditingController();
  final TextEditingController _comboOutputController = TextEditingController();
  final Map<String, KeyMapping> _mappings = {};
  final List<ComboMapping> _combos = [];
  bool _isSaving = false;
  bool _isFetchingKeys = false;
  bool _usingFallbackKeys = false;
  String? _registryError;
  List<String> _canonicalKeys = KeyMappings.allowedKeys;
  static const String _defaultScriptPath = 'scripts/generated.rhai';
  List<LayerInfo> _layers = const [
    LayerInfo(name: 'base', active: true, priority: 0),
  ];

  @override
  void initState() {
    super.initState();
    WidgetsBinding.instance.addPostFrameCallback((_) {
      _fetchKeyRegistry();
    });
  }

  @override
  void dispose() {
    _outputController.dispose();
    _layerController.dispose();
    _tapOutputController.dispose();
    _holdOutputController.dispose();
    _comboKeysController.dispose();
    _comboOutputController.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        title: const Text('Keymap Editor'),
        actions: [
          IconButton(
            icon: const Icon(Icons.save),
            onPressed: _isSaving ? null : _saveScript,
            tooltip: 'Save Script',
          ),
          IconButton(
            icon: const Icon(Icons.code),
            onPressed: _viewScript,
            tooltip: 'View Generated Script',
          ),
        ],
      ),
      body: Column(
        children: [
          KeyRegistryBanner(
            isFetchingKeys: _isFetchingKeys,
            usingFallbackKeys: _usingFallbackKeys,
            canonicalKeysCount: _canonicalKeys.length,
            registryError: _registryError,
            onRefresh: _fetchKeyRegistry,
          ),
          Expanded(
            flex: 2,
            child: KeyboardWidget(
              onKeySelected: _handleKeySelected,
              selectedKey: _selectedKey,
            ),
          ),
          Expanded(flex: 1, child: _buildConfigPanel()),
        ],
      ),
    );
  }

  Widget _buildConfigPanel() {
    return Column(
      children: [
        KeyConfigPanel(
          selectedKey: _selectedKey,
          selectedAction: _selectedAction,
          outputController: _outputController,
          layerController: _layerController,
          tapOutputController: _tapOutputController,
          holdOutputController: _holdOutputController,
          onActionChanged: (value) {
            if (value == null) return;
            setState(() => _selectedAction = value);
          },
          onApply: _applyMapping,
        ),
        Expanded(
          child: Padding(
            padding: const EdgeInsets.symmetric(horizontal: 16),
            child: MappingListPanel(
              mappings: _mappings,
              layers: _layers,
              onRemoveMapping: _removeMapping,
              onAddLayer: _addLayer,
              onToggleLayer: _toggleLayer,
            ),
          ),
        ),
        Padding(
          padding: const EdgeInsets.all(16),
          child: ComboConfigRow(
            comboKeysController: _comboKeysController,
            comboOutputController: _comboOutputController,
            combos: _combos,
            onAddCombo: _addCombo,
            onRemoveCombo: (index) => setState(() => _combos.removeAt(index)),
          ),
        ),
      ],
    );
  }

  void _handleKeySelected(String key) {
    setState(() {
      _selectedKey = key;
      _outputController.text = _mappings[key]?.to ?? '';
      _layerController.text = _mappings[key]?.layer ?? '';
      _tapOutputController.text = _mappings[key]?.tapHoldTap ?? '';
      _holdOutputController.text = _mappings[key]?.tapHoldHold ?? '';
      _selectedAction = _mappings[key]?.type ?? KeyActionType.remap;
    });
  }

  void _applyMapping() {
    if (_selectedKey == null) return;
    if (_selectedAction == KeyActionType.remap &&
        _outputController.text.trim().isEmpty) {
      ScaffoldMessenger.of(context).showSnackBar(
        const SnackBar(content: Text('Provide a target key for remap.')),
      );
      return;
    }

    final fromKey = _selectedKey!;
    if (!KeyMappings.isKnownKey(fromKey)) {
      ScaffoldMessenger.of(
        context,
      ).showSnackBar(SnackBar(content: Text('Unknown key: $fromKey')));
      return;
    }

    if (_selectedAction == KeyActionType.remap &&
        !KeyMappings.isKnownKey(_outputController.text.trim())) {
      ScaffoldMessenger.of(context).showSnackBar(
        SnackBar(
          content: Text('Unknown target key: ${_outputController.text.trim()}'),
        ),
      );
      return;
    }

    final mapping = KeyMapping(
      from: fromKey,
      type: _selectedAction,
      to: _selectedAction == KeyActionType.remap
          ? _outputController.text.trim()
          : null,
      layer: _layerController.text.trim().isEmpty
          ? null
          : _layerController.text.trim(),
      tapHoldTap: _tapOutputController.text.trim().isEmpty
          ? null
          : _tapOutputController.text.trim(),
      tapHoldHold: _holdOutputController.text.trim().isEmpty
          ? null
          : _holdOutputController.text.trim(),
    );

    setState(() {
      _mappings[_selectedKey!] = mapping;
    });
  }

  Future<void> _saveScript() async {
    if (_mappings.isEmpty) {
      ScaffoldMessenger.of(context).showSnackBar(
        const SnackBar(
          content: Text('Add at least one mapping before saving.'),
        ),
      );
      return;
    }

    setState(() => _isSaving = true);
    final script = ScriptGenerator.build(
      mappings: _mappings.values,
      combos: _combos,
    );

    try {
      final file = File(_defaultScriptPath);
      await file.parent.create(recursive: true);
      await file.writeAsString(script);

      final engine = widget.engineService;
      final loaded = engine.isInitialized
          ? await engine.loadScript(_defaultScriptPath)
          : false;

      ScaffoldMessenger.of(context).showSnackBar(
        SnackBar(
          content: Text(
            loaded
                ? 'Script saved and loaded from $_defaultScriptPath'
                : 'Script saved to $_defaultScriptPath',
          ),
        ),
      );
    } catch (e) {
      ScaffoldMessenger.of(
        context,
      ).showSnackBar(SnackBar(content: Text('Failed to save script: $e')));
    } finally {
      if (mounted) {
        setState(() => _isSaving = false);
      }
    }
  }

  void _viewScript() {
    final script = ScriptGenerator.build(
      mappings: _mappings.values,
      combos: _combos,
    );
    showDialog(
      context: context,
      builder: (context) => AlertDialog(
        title: const Text('Generated Script'),
        content: SingleChildScrollView(child: SelectableText(script)),
        actions: [
          TextButton(
            onPressed: () => Navigator.pop(context),
            child: const Text('Close'),
          ),
        ],
      ),
    );
  }

  void _removeMapping(String key) {
    setState(() {
      _mappings.remove(key);
      if (_selectedKey == key) {
        _outputController.clear();
      }
    });
  }

  void _addCombo() {
    final keys = _comboKeysController.text
        .split(',')
        .map((k) => k.trim())
        .where((k) => k.isNotEmpty)
        .toList();
    final output = _comboOutputController.text.trim();

    if (keys.length < 2 || output.isEmpty) {
      ScaffoldMessenger.of(context).showSnackBar(
        const SnackBar(content: Text('Provide at least 2 keys and an output.')),
      );
      return;
    }

    if (keys.any((k) => !KeyMappings.isKnownKey(k))) {
      ScaffoldMessenger.of(
        context,
      ).showSnackBar(const SnackBar(content: Text('Unknown key in combo.')));
      return;
    }

    setState(() {
      _combos.add(ComboMapping(keys: keys, output: output));
      _comboKeysController.clear();
      _comboOutputController.clear();
    });
  }

  void _addLayer() {
    setState(() {
      final next = 'layer_${_layers.length}';
      _layers = [
        ..._layers,
        LayerInfo(name: next, active: false, priority: _layers.length),
      ];
    });
  }

  void _toggleLayer(String name, bool active) {
    setState(() {
      final idx = _layers.indexWhere((l) => l.name == name);
      if (idx >= 0) {
        _layers = [
          ..._layers.sublist(0, idx),
          LayerInfo(
            name: name,
            active: active,
            priority: _layers[idx].priority,
          ),
          ..._layers.sublist(idx + 1),
        ];
      }
    });
  }

  Future<void> _fetchKeyRegistry() async {
    setState(() {
      _isFetchingKeys = true;
      _registryError = null;
    });

    final result = await widget.engineService.fetchKeyRegistry();
    if (!mounted) return;

    final canonical = result.entries
        .expand((entry) => [entry.name, ...entry.aliases])
        .map((k) => k.toLowerCase())
        .where((k) => k.isNotEmpty)
        .toList();
    if (canonical.isNotEmpty) {
      KeyMappings.updateAllowedKeys(canonical);
    }

    setState(() {
      _canonicalKeys = KeyMappings.allowedKeys;
      _usingFallbackKeys = result.usedFallback;
      _registryError = result.error;
      _isFetchingKeys = false;
    });
  }
}
