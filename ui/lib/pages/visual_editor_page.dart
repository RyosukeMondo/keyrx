/// Visual keymap editor page with drag-and-drop mapping and code view.
///
/// Provides a no-code interface for creating key mappings with the ability
/// to "eject to code" and see/edit the generated Rhai script.
library;

import 'dart:io';

import 'package:flutter/material.dart';
import 'package:provider/provider.dart';

import '../models/keyboard_layout.dart';
import '../services/rhai_generator.dart';
import '../services/service_registry.dart';
import '../widgets/visual_keyboard.dart';
import 'visual_editor_widgets.dart';

/// Visual editor page combining keyboard, mappings, and code view.
class VisualEditorPage extends StatefulWidget {
  const VisualEditorPage({super.key});

  @override
  State<VisualEditorPage> createState() => _VisualEditorPageState();
}

class _VisualEditorPageState extends State<VisualEditorPage> {
  final _generator = RhaiGenerator();
  final _codeController = TextEditingController();
  final _fileNameController = TextEditingController(text: 'config.rhai');

  List<RemapConfig> _mappings = [];
  List<TapHoldConfig> _tapHoldConfigs = [];
  bool _showCode = false;
  bool _codeModified = false;
  bool _hasAdvancedFeatures = false;
  bool _isSaving = false;
  String? _selectedKeyId;
  String? _lastSavedPath;

  VisualConfig get _visualConfig => VisualConfig(
        mappings: _mappings,
        tapHoldConfigs: _tapHoldConfigs,
      );

  @override
  void dispose() {
    _codeController.dispose();
    _fileNameController.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: _buildAppBar(context),
      body: Column(
        children: [
          if (_hasAdvancedFeatures)
            AdvancedFeaturesWarning(
              onViewCode: () => setState(() => _showCode = true),
            ),
          Expanded(
            child: _showCode ? _buildCodeView() : _buildVisualView(context),
          ),
        ],
      ),
    );
  }

  AppBar _buildAppBar(BuildContext context) {
    return AppBar(
      title: const Text('Visual Editor'),
      actions: [
        IconButton(
          icon: Icon(_showCode ? Icons.grid_view : Icons.code),
          tooltip: _showCode ? 'Visual View' : 'Show Code',
          onPressed: _toggleCodeView,
        ),
        IconButton(
          icon: const Icon(Icons.add),
          tooltip: 'New Configuration',
          onPressed: _confirmClear,
        ),
        IconButton(
          icon: const Icon(Icons.folder_open),
          tooltip: 'Load Script',
          onPressed: _loadScript,
        ),
        IconButton(
          icon: const Icon(Icons.save),
          tooltip: 'Save Script',
          onPressed: _isSaving ? null : _saveScript,
        ),
      ],
    );
  }

  Widget _buildVisualView(BuildContext context) {
    return Row(
      children: [
        Expanded(
          flex: 3,
          child: Padding(
            padding: const EdgeInsets.all(16),
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Text(
                  'Drag from one key to another to create a mapping',
                  style: Theme.of(context).textTheme.bodySmall?.copyWith(
                        color: Theme.of(context).colorScheme.onSurfaceVariant,
                      ),
                ),
                const SizedBox(height: 8),
                Expanded(
                  child: VisualKeyboard(
                    mappings: _mappings,
                    selectedKeys:
                        _selectedKeyId != null ? {_selectedKeyId!} : {},
                    mappedKeys: _getMappedKeys(),
                    onKeyTap: _handleKeyTap,
                    onMappingCreated: _handleMappingCreated,
                    onMappingDeleted: _handleMappingDeleted,
                  ),
                ),
              ],
            ),
          ),
        ),
        SizedBox(
          width: 280,
          child: MappingPanel(
            mappings: _mappings,
            tapHoldConfigs: _tapHoldConfigs,
            onMappingDeleted: _handleMappingDeleted,
            onTapHoldDeleted: (i) {
              setState(() {
                _tapHoldConfigs = List.from(_tapHoldConfigs)..removeAt(i);
              });
            },
            onMappingSelected: (keyId) => setState(() => _selectedKeyId = keyId),
            onClearAll: _confirmClear,
          ),
        ),
      ],
    );
  }

  Widget _buildCodeView() {
    return CodeEditorView(
      controller: _codeController,
      isModified: _codeModified,
      onCodeChanged: (value) {
        if (!_codeModified) setState(() => _codeModified = true);
      },
      onParseToVisual: _parseCodeToVisual,
    );
  }

  Set<String> _getMappedKeys() {
    final keys = <String>{};
    for (final mapping in _mappings) {
      keys.add(mapping.sourceKeyId);
    }
    for (final tapHold in _tapHoldConfigs) {
      keys.add(tapHold.triggerKey);
    }
    return keys;
  }

  void _handleKeyTap(KeyDefinition key) {
    setState(() {
      _selectedKeyId = _selectedKeyId == key.id ? null : key.id;
    });
  }

  void _handleMappingCreated(String sourceKeyId, String targetKeyId) {
    setState(() {
      final existingIndex =
          _mappings.indexWhere((m) => m.sourceKeyId == sourceKeyId);
      if (existingIndex >= 0) {
        _mappings = List.from(_mappings)
          ..[existingIndex] = RemapConfig(
            sourceKeyId: sourceKeyId,
            targetKeyId: targetKeyId,
          );
      } else {
        _mappings = [
          ..._mappings,
          RemapConfig(sourceKeyId: sourceKeyId, targetKeyId: targetKeyId),
        ];
      }
      _updateCodeFromVisual();
    });
  }

  void _handleMappingDeleted(int index) {
    setState(() {
      _mappings = List.from(_mappings)..removeAt(index);
      _updateCodeFromVisual();
    });
  }

  void _toggleCodeView() async {
    if (_showCode && _codeModified) {
      final action = await VisualEditorDialogs.showSyncWarning(context);
      if (!mounted || action == null) return;
      if (action == SyncAction.parse) {
        _parseCodeToVisual();
      }
      setState(() {
        _showCode = false;
        _codeModified = false;
      });
    } else {
      setState(() {
        _showCode = !_showCode;
        if (_showCode) _updateCodeFromVisual();
      });
    }
  }

  void _updateCodeFromVisual() {
    _codeController.text = _generator.generateScript(_visualConfig);
    _codeModified = false;
  }

  void _parseCodeToVisual() {
    final config = _generator.parseScript(_codeController.text);
    setState(() {
      _mappings = config.mappings;
      _tapHoldConfigs = config.tapHoldConfigs;
      _hasAdvancedFeatures = config.hasAdvancedFeatures;
      _codeModified = false;
    });
    if (config.hasAdvancedFeatures) {
      ScaffoldMessenger.of(context).showSnackBar(
        const SnackBar(
          content:
              Text('Some advanced features cannot be shown in visual mode.'),
        ),
      );
    }
  }

  Future<void> _confirmClear() async {
    if (_mappings.isEmpty && _tapHoldConfigs.isEmpty) return;
    final confirmed = await VisualEditorDialogs.showClearConfirmation(context);
    if (confirmed != true || !mounted) return;
    setState(() {
      _mappings = [];
      _tapHoldConfigs = [];
      _hasAdvancedFeatures = false;
      _codeModified = false;
      _updateCodeFromVisual();
    });
  }

  Future<void> _loadScript() async {
    final result = await VisualEditorDialogs.showLoadDialog(context);
    if (result == null || result.isEmpty || !mounted) return;

    try {
      final file = File(result);
      if (!await file.exists()) {
        if (mounted) {
          ScaffoldMessenger.of(context).showSnackBar(
            SnackBar(content: Text('File not found: $result')),
          );
        }
        return;
      }
      final code = await file.readAsString();
      final config = _generator.parseScript(code);
      setState(() {
        _mappings = config.mappings;
        _tapHoldConfigs = config.tapHoldConfigs;
        _hasAdvancedFeatures = config.hasAdvancedFeatures;
        _codeController.text = code;
        _codeModified = false;
        _lastSavedPath = result;
        _fileNameController.text = result.split('/').last;
      });
      if (mounted) {
        ScaffoldMessenger.of(context).showSnackBar(
          SnackBar(content: Text('Loaded: $result')),
        );
      }
    } catch (e) {
      if (mounted) {
        ScaffoldMessenger.of(context).showSnackBar(
          SnackBar(content: Text('Failed to load script: $e')),
        );
      }
    }
  }

  Future<void> _saveScript() async {
    final suggestedPath =
        _lastSavedPath ?? 'scripts/${_fileNameController.text}';
    final result =
        await VisualEditorDialogs.showSaveDialog(context, suggestedPath);
    if (result == null || result.isEmpty || !mounted) return;

    setState(() => _isSaving = true);
    try {
      final code = _showCode && _codeModified
          ? _codeController.text
          : _generator.generateScript(_visualConfig);
      final file = File(result);
      await file.parent.create(recursive: true);
      await file.writeAsString(code);
      _lastSavedPath = result;
      _fileNameController.text = result.split('/').last;

      var loaded = false;
      if (mounted) {
        final registry = Provider.of<ServiceRegistry>(context, listen: false);
        final engine = registry.engineService;
        loaded = engine.isInitialized ? await engine.loadScript(result) : false;
      }
      if (mounted) {
        ScaffoldMessenger.of(context).showSnackBar(
          SnackBar(
            content: Text(loaded
                ? 'Script saved and loaded: $result'
                : 'Script saved: $result'),
          ),
        );
      }
      setState(() => _codeModified = false);
    } catch (e) {
      if (mounted) {
        ScaffoldMessenger.of(context).showSnackBar(
          SnackBar(content: Text('Failed to save script: $e')),
        );
      }
    } finally {
      if (mounted) setState(() => _isSaving = false);
    }
  }
}
