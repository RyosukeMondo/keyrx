/// Visual keymap editor page.
///
/// Provides a drag-and-drop interface for creating key mappings
/// that generates the underlying Rhai script automatically.
library;

import 'dart:async';
import 'dart:io';

import 'package:flutter/material.dart';
import 'package:provider/provider.dart';

import '../ffi/bridge.dart';
import '../repositories/mapping_repository.dart';
import '../services/engine_service.dart';
import '../services/mapping_validator.dart';
import '../services/script_file_service.dart';
import '../state/app_state.dart';
import '../widgets/keyboard.dart';
import 'editor_widgets.dart';

/// Visual keymap editor page.
class EditorPage extends StatefulWidget {
  const EditorPage({
    super.key,
    required this.engineService,
    required this.mappingRepository,
    required this.bridge,
    this.validator = const MappingValidator(),
    this.scriptFileService = const ScriptFileService(),
  });

  /// The engine service for key registry and script loading.
  final EngineService engineService;

  /// The shared mapping repository for key mappings.
  final MappingRepository mappingRepository;

  /// The FFI bridge for script validation.
  final KeyrxBridge bridge;

  /// The validator for key mappings and combos.
  final MappingValidator validator;

  /// The service for script file I/O operations.
  final ScriptFileService scriptFileService;

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
  bool _isSaving = false;
  bool _isFetchingKeys = false;
  bool _usingFallbackKeys = false;
  String? _registryError;
  List<String> _canonicalKeys = KeyMappings.allowedKeys;
  static const String _defaultScriptPath = 'scripts/generated.rhai';
  static const String _tempValidationPath = '/tmp/keyrx_validation.rhai';

  // Script validation state
  Timer? _validationDebounce;
  bool _isValidating = false;
  ScriptValidationResult? _validationResult;
  static const _validationDebounceMs = 500;

  @override
  void initState() {
    super.initState();
    widget.mappingRepository.addListener(_onMappingsChanged);
    WidgetsBinding.instance.addPostFrameCallback((_) {
      _fetchKeyRegistry();
    });
  }

  @override
  void dispose() {
    _validationDebounce?.cancel();
    widget.mappingRepository.removeListener(_onMappingsChanged);
    _outputController.dispose();
    _layerController.dispose();
    _tapOutputController.dispose();
    _holdOutputController.dispose();
    _comboKeysController.dispose();
    _comboOutputController.dispose();
    super.dispose();
  }

  void _onMappingsChanged() {
    if (mounted) {
      setState(() {});
      _scheduleValidation();
    }
  }

  void _scheduleValidation() {
    _validationDebounce?.cancel();
    _validationDebounce = Timer(
      const Duration(milliseconds: _validationDebounceMs),
      _validateScript,
    );
  }

  Future<void> _validateScript() async {
    final repo = widget.mappingRepository;
    if (repo.mappings.isEmpty) {
      setState(() {
        _validationResult = null;
        _isValidating = false;
      });
      return;
    }

    setState(() => _isValidating = true);

    final script = repo.generateScript();
    try {
      final file = File(_tempValidationPath);
      await file.parent.create(recursive: true);
      await file.writeAsString(script);

      final result = widget.bridge.checkScript(_tempValidationPath);

      if (mounted) {
        setState(() {
          _validationResult = result;
          _isValidating = false;
        });
      }
    } catch (e) {
      if (mounted) {
        setState(() {
          _validationResult = ScriptValidationResult.error('$e');
          _isValidating = false;
        });
      }
    }
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
          _buildValidationBanner(),
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

  Widget _buildValidationBanner() {
    if (_isValidating) {
      return Container(
        color: Colors.blue.withValues(alpha: 0.1),
        padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 8),
        child: const Row(
          children: [
            SizedBox(
              width: 16,
              height: 16,
              child: CircularProgressIndicator(strokeWidth: 2),
            ),
            SizedBox(width: 8),
            Text('Validating script...'),
          ],
        ),
      );
    }

    final result = _validationResult;
    if (result == null || result.valid) {
      return const SizedBox.shrink();
    }

    final errors = result.errors;
    final errorMessage = result.errorMessage;

    return Material(
      color: Colors.red.withValues(alpha: 0.15),
      child: InkWell(
        onTap: errors.isNotEmpty ? () => _showValidationErrors(errors) : null,
        child: Padding(
          padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 8),
          child: Row(
            children: [
              const Icon(Icons.error_outline, color: Colors.red, size: 20),
              const SizedBox(width: 8),
              Expanded(
                child: Text(
                  errorMessage ??
                      (errors.isNotEmpty
                          ? 'Script has ${errors.length} error${errors.length > 1 ? 's' : ''}: ${errors.first.message}'
                          : 'Script validation failed'),
                  style: const TextStyle(color: Colors.red),
                  overflow: TextOverflow.ellipsis,
                ),
              ),
              if (errors.isNotEmpty) ...[
                const SizedBox(width: 8),
                const Icon(Icons.chevron_right, color: Colors.red),
              ],
            ],
          ),
        ),
      ),
    );
  }

  void _showValidationErrors(List<ScriptValidationError> errors) {
    showDialog(
      context: context,
      builder: (context) => AlertDialog(
        title: const Text('Script Validation Errors'),
        content: SizedBox(
          width: double.maxFinite,
          child: ListView.separated(
            shrinkWrap: true,
            itemCount: errors.length,
            separatorBuilder: (_, __) => const Divider(),
            itemBuilder: (context, index) {
              final error = errors[index];
              return ListTile(
                leading: const Icon(Icons.error, color: Colors.red),
                title: Text(error.message),
                subtitle: error.line != null
                    ? Text('Line ${error.line}${error.column != null ? ', Column ${error.column}' : ''}')
                    : null,
                dense: true,
              );
            },
          ),
        ),
        actions: [
          TextButton(
            onPressed: () => Navigator.pop(context),
            child: const Text('Close'),
          ),
        ],
      ),
    );
  }

  Widget _buildConfigPanel() {
    final appState = context.watch<AppState>();
    final repo = widget.mappingRepository;
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
              mappings: repo.mappings,
              layers: appState.layers,
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
            combos: repo.combos,
            onAddCombo: _addCombo,
            onRemoveCombo: (index) => repo.removeCombo(index),
          ),
        ),
      ],
    );
  }

  void _handleKeySelected(String key) {
    final mapping = widget.mappingRepository.getMapping(key);
    setState(() {
      _selectedKey = key;
      _outputController.text = mapping?.to ?? '';
      _layerController.text = mapping?.layer ?? '';
      _tapOutputController.text = mapping?.tapHoldTap ?? '';
      _holdOutputController.text = mapping?.tapHoldHold ?? '';
      _selectedAction = mapping?.type ?? KeyActionType.remap;
    });
  }

  void _applyMapping() {
    if (_selectedKey == null) return;

    final mapping = KeyMapping(
      from: _selectedKey!,
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

    final result = widget.validator.validateMapping(_selectedKey, mapping);
    if (!result.isValid) {
      ScaffoldMessenger.of(context).showSnackBar(
        SnackBar(content: Text(result.errorMessage ?? 'Invalid mapping')),
      );
      return;
    }

    widget.mappingRepository.setMapping(_selectedKey!, mapping);
  }

  Future<void> _saveScript() async {
    final repo = widget.mappingRepository;
    if (repo.mappings.isEmpty) {
      ScaffoldMessenger.of(context).showSnackBar(
        const SnackBar(
          content: Text('Add at least one mapping before saving.'),
        ),
      );
      return;
    }

    // Block save if script validation failed
    final validationResult = _validationResult;
    if (validationResult != null && !validationResult.valid) {
      ScaffoldMessenger.of(context).showSnackBar(
        const SnackBar(
          content: Text('Fix script validation errors before saving.'),
          backgroundColor: Colors.red,
        ),
      );
      return;
    }

    setState(() => _isSaving = true);
    final script = repo.generateScript();

    final result = await widget.scriptFileService.saveScript(
      _defaultScriptPath,
      script,
    );

    if (!result.success) {
      if (mounted) {
        ScaffoldMessenger.of(context).showSnackBar(
          SnackBar(content: Text('Failed to save script: ${result.errorMessage}')),
        );
        setState(() => _isSaving = false);
      }
      return;
    }

    // Only load into engine if validation passed
    final engine = widget.engineService;
    final loaded = engine.isInitialized
        ? await engine.loadScript(_defaultScriptPath)
        : false;

    if (mounted) {
      ScaffoldMessenger.of(context).showSnackBar(
        SnackBar(
          content: Text(
            loaded
                ? 'Script saved and loaded from $_defaultScriptPath'
                : 'Script saved to $_defaultScriptPath',
          ),
        ),
      );
      setState(() => _isSaving = false);
    }
  }

  void _viewScript() {
    final script = widget.mappingRepository.generateScript();
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
    widget.mappingRepository.removeMapping(key);
    if (_selectedKey == key) {
      _outputController.clear();
    }
  }

  void _addCombo() {
    final keys = _comboKeysController.text
        .split(',')
        .map((k) => k.trim())
        .where((k) => k.isNotEmpty)
        .toList();
    final output = _comboOutputController.text.trim();

    final result = widget.validator.validateCombo(keys, output);
    if (!result.isValid) {
      ScaffoldMessenger.of(context).showSnackBar(
        SnackBar(content: Text(result.errorMessage ?? 'Invalid combo')),
      );
      return;
    }

    widget.mappingRepository.addCombo(ComboMapping(keys: keys, output: output));
    _comboKeysController.clear();
    _comboOutputController.clear();
  }

  void _addLayer() {
    final appState = context.read<AppState>();
    final next = 'layer_${appState.layers.length}';
    appState.addLayer(next, priority: appState.layers.length);
  }

  void _toggleLayer(String name, bool active) {
    context.read<AppState>().toggleLayer(name, active);
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
