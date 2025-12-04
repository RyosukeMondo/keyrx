/// Visual keymap editor page.
///
/// Provides a drag-and-drop interface for creating key mappings
/// that generates the underlying Rhai script automatically.
library;

import 'dart:async';

import 'package:flutter/material.dart';
import 'package:provider/provider.dart';

import '../models/key_mapping.dart';
import '../models/validation.dart' as validation_models;
import '../repositories/mapping_repository.dart';
import '../services/key_mappings_util.dart';
import '../services/mapping_validator.dart';
import '../services/facade/keyrx_facade.dart';
import '../state/app_state.dart';
import '../config/config.dart';
import '../widgets/keyboard.dart';
import '../widgets/editor/editor.dart';

/// Visual keymap editor page.
class EditorPage extends StatefulWidget {
  const EditorPage({
    super.key,
    required this.facade,
    required this.mappingRepository,
    this.validator = const MappingValidator(),
  });

  /// The KeyRx facade for all service operations.
  final KeyrxFacade facade;

  /// The shared mapping repository for key mappings.
  final MappingRepository mappingRepository;

  /// The validator for key mappings and combos.
  final MappingValidator validator;

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

  // Script validation state
  Timer? _validationDebounce;
  bool _isValidating = false;
  validation_models.ValidationResult? _validationResult;

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
      const Duration(milliseconds: TimingConfig.debounceMs),
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
      // Use bridge directly for in-memory validation (facade.validateScript requires a file path)
      final result = widget.facade.services.bridge.validateScript(
        script,
        const validation_models.ValidationOptions(includeCoverage: true),
      );

      if (mounted) {
        setState(() {
          _validationResult = result;
          _isValidating = false;
        });
      }
    } catch (e) {
      if (mounted) {
        setState(() {
          _validationResult = validation_models.ValidationResult.error('$e');
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
          ValidationBannerRich(
            isValidating: _isValidating,
            validationResult: _validationResult,
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
    final appState = context.watch<AppState>();
    final repo = widget.mappingRepository;
    return LayoutBuilder(
      builder: (context, constraints) {
        final availableHeight = constraints.maxHeight;
        final configPanelHeight = availableHeight * 0.4;
        final comboHeight = availableHeight * 0.3;
        final listHeight = availableHeight - configPanelHeight - comboHeight;

        return Column(
          mainAxisSize: MainAxisSize.min,
          children: [
            ConstrainedBox(
              constraints: BoxConstraints(maxHeight: configPanelHeight),
              child: SingleChildScrollView(
                child: KeyConfigPanel(
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
              ),
            ),
            SizedBox(
              height: listHeight,
              child: Padding(
                padding: const EdgeInsets.symmetric(
                  horizontal: UiConstants.defaultPadding,
                ),
                child: MappingListPanel(
                  mappings: repo.mappings,
                  layers: appState.layers,
                  onRemoveMapping: _removeMapping,
                  onAddLayer: _addLayer,
                  onToggleLayer: _toggleLayer,
                ),
              ),
            ),
            ConstrainedBox(
              constraints: BoxConstraints(maxHeight: comboHeight),
              child: SingleChildScrollView(
                child: Padding(
                  padding: const EdgeInsets.all(UiConstants.defaultPadding),
                  child: ComboConfigRow(
                    comboKeysController: _comboKeysController,
                    comboOutputController: _comboOutputController,
                    combos: repo.combos,
                    onAddCombo: _addCombo,
                    onRemoveCombo: (index) => repo.removeCombo(index),
                  ),
                ),
              ),
            ),
          ],
        );
      },
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

    // Check validation result
    final validationResult = _validationResult;
    if (validationResult != null && !validationResult.isValid) {
      // Has errors - block save
      if (validationResult.hasErrors) {
        ScaffoldMessenger.of(context).showSnackBar(
          const SnackBar(
            content: Text('Fix script validation errors before saving.'),
            backgroundColor: Colors.red,
          ),
        );
        return;
      }
    }

    // Has warnings - confirm save
    if (validationResult != null && validationResult.hasWarnings) {
      final confirmed = await _showSaveWithWarningsDialog(validationResult);
      if (!confirmed) return;
    }

    setState(() => _isSaving = true);
    final script = repo.generateScript();

    // Use facade for script operations
    final saveResult = await widget.facade.saveScript(
      PathConstants.defaultScriptPath,
      script,
    );

    if (saveResult.isErr) {
      if (mounted) {
        final error = saveResult.errOrNull!;
        ScaffoldMessenger.of(context).showSnackBar(
          SnackBar(content: Text('Failed to save script: ${error.userMessage}')),
        );
        setState(() => _isSaving = false);
      }
      return;
    }

    // Load into engine if initialized
    final engine = widget.facade.services.engineService;
    final loaded = engine.isInitialized
        ? await engine.loadScript(PathConstants.defaultScriptPath)
        : false;

    if (mounted) {
      ScaffoldMessenger.of(context).showSnackBar(
        SnackBar(
          content: Text(
            loaded
                ? 'Script saved and loaded from ${PathConstants.defaultScriptPath}'
                : 'Script saved to ${PathConstants.defaultScriptPath}',
          ),
        ),
      );
      setState(() => _isSaving = false);
    }
  }

  Future<bool> _showSaveWithWarningsDialog(
    validation_models.ValidationResult result,
  ) async {
    final confirmed = await showDialog<bool>(
      context: context,
      builder: (ctx) => AlertDialog(
        title: const Row(
          children: [
            Icon(Icons.warning_amber, color: Colors.orange),
            SizedBox(width: 8),
            Text('Save with warnings?'),
          ],
        ),
        content: SizedBox(
          width: double.maxFinite,
          child: Column(
            mainAxisSize: MainAxisSize.min,
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              Text(
                'Your script has ${result.warnings.length} warning${result.warnings.length > 1 ? 's' : ''}:',
              ),
              const SizedBox(height: 12),
              ConstrainedBox(
                constraints: const BoxConstraints(maxHeight: 200),
                child: ListView.builder(
                  shrinkWrap: true,
                  itemCount: result.warnings.length,
                  itemBuilder: (_, index) {
                    final w = result.warnings[index];
                    return Padding(
                      padding: const EdgeInsets.symmetric(vertical: 4),
                      child: Row(
                        crossAxisAlignment: CrossAxisAlignment.start,
                        children: [
                          Icon(
                            _iconForCategory(w.category),
                            size: 16,
                            color: Colors.orange,
                          ),
                          const SizedBox(width: 8),
                          Expanded(child: Text(w.message)),
                        ],
                      ),
                    );
                  },
                ),
              ),
              const SizedBox(height: 12),
              const Text(
                'Do you want to save anyway?',
                style: TextStyle(fontWeight: FontWeight.w500),
              ),
            ],
          ),
        ),
        actions: [
          TextButton(
            onPressed: () => Navigator.pop(ctx, false),
            child: const Text('Cancel'),
          ),
          FilledButton(
            onPressed: () => Navigator.pop(ctx, true),
            child: const Text('Save anyway'),
          ),
        ],
      ),
    );
    return confirmed ?? false;
  }

  IconData _iconForCategory(validation_models.WarningCategory category) {
    return switch (category) {
      validation_models.WarningCategory.conflict => Icons.compare_arrows,
      validation_models.WarningCategory.safety => Icons.warning_amber,
      validation_models.WarningCategory.performance => Icons.speed,
    };
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

    // Access engine service through facade for key registry operations
    final result = await widget.facade.services.engineService.fetchKeyRegistry();
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
