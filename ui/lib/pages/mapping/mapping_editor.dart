/// Visual editor for keymaps.
library;

import 'package:flutter/material.dart';
import 'package:provider/provider.dart';

import '../../models/action_binding.dart';
import '../../models/hardware_profile.dart';
import '../../models/keymap.dart';
import '../../models/virtual_layout.dart';
import '../../services/keymap_service.dart';
import '../../services/service_registry.dart';
import '../../widgets/soft_keyboard.dart';
import '../../widgets/virtual_layout_renderer.dart';

class MappingEditor extends StatefulWidget {
  const MappingEditor({
    super.key,
    required this.keymap,
    required this.profile,
    required this.layout,
  });

  final Keymap keymap;
  final HardwareProfile profile;
  final VirtualLayout layout;

  @override
  State<MappingEditor> createState() => _MappingEditorState();
}

class _MappingEditorState extends State<MappingEditor> {
  late Keymap _workingKeymap;
  late TextEditingController _nameController;
  late TextEditingController _macroController;
  int _selectedLayerIndex = 0;
  String? _selectedVirtualKeyId;
  bool _isSaving = false;
  bool _isDirty = false;

  KeymapService get _keymapService {
    try {
      return Provider.of<ServiceRegistry>(context, listen: false).keymapService;
    } on ProviderNotFoundException {
      return Provider.of<KeymapService>(context, listen: false);
    }
  }

  @override
  void initState() {
    super.initState();
    _workingKeymap = widget.keymap;
    _nameController = TextEditingController(text: widget.keymap.name);
    _macroController = TextEditingController();
  }

  @override
  void dispose() {
    _nameController.dispose();
    _macroController.dispose();
    super.dispose();
  }

  KeymapLayer? get _currentLayer {
    if (_selectedLayerIndex < 0 ||
        _selectedLayerIndex >= _workingKeymap.layers.length) {
      return null;
    }
    return _workingKeymap.layers[_selectedLayerIndex];
  }

  Map<String, ActionBinding> get _bindingsForLayer {
    return _currentLayer?.bindings ?? const {};
  }

  String _bindingLabel(ActionBinding? binding) {
    if (binding == null) return 'Unmapped';
    return binding.map(
      standardKey: (b) => b.value,
      macro: (b) => 'Macro: ${b.value}',
      layerToggle: (b) => 'Layer → ${b.value}',
      transparent: (_) => 'Transparent',
    );
  }

  void _applyBinding(ActionBinding binding) {
    if (_selectedVirtualKeyId == null || _currentLayer == null) return;

    final layers = [..._workingKeymap.layers];
    final layer = layers[_selectedLayerIndex];
    final updatedBindings = Map<String, ActionBinding>.from(layer.bindings);
    updatedBindings[_selectedVirtualKeyId!] = binding;
    layers[_selectedLayerIndex] = layer.copyWith(bindings: updatedBindings);

    setState(() {
      _workingKeymap = _workingKeymap.copyWith(layers: layers);
      _isDirty = true;
    });

    ScaffoldMessenger.of(context).showSnackBar(
      SnackBar(
        content: Text(
          'Mapped $_selectedVirtualKeyId to ${_bindingLabel(binding)}',
        ),
        duration: const Duration(milliseconds: 500),
      ),
    );
  }

  void _clearBinding() {
    if (_selectedVirtualKeyId == null || _currentLayer == null) return;

    final layers = [..._workingKeymap.layers];
    final layer = layers[_selectedLayerIndex];
    final updatedBindings = Map<String, ActionBinding>.from(layer.bindings);
    updatedBindings.remove(_selectedVirtualKeyId!);
    layers[_selectedLayerIndex] = layer.copyWith(bindings: updatedBindings);

    setState(() {
      _workingKeymap = _workingKeymap.copyWith(layers: layers);
      _isDirty = true;
    });
  }

  Future<void> _saveKeymap() async {
    setState(() => _isSaving = true);

    final toSave = _workingKeymap.copyWith(name: _nameController.text.trim());

    final result = await _keymapService.saveKeymap(toSave);

    if (!mounted) return;
    setState(() => _isSaving = false);

    if (result.hasError) {
      ScaffoldMessenger.of(context).showSnackBar(
        SnackBar(
          content: Text('Failed to save: ${result.errorMessage}'),
          backgroundColor: Colors.red,
        ),
      );
    } else {
      setState(() {
        _workingKeymap = result.data ?? toSave;
        _isDirty = false;
      });
      ScaffoldMessenger.of(
        context,
      ).showSnackBar(const SnackBar(content: Text('Keymap saved')));
    }
  }

  // --- UI Construction ---

  Widget _buildTopBar() {
    return Card(
      margin: const EdgeInsets.all(8),
      child: Padding(
        padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 12),
        child: Row(
          children: [
            Expanded(
              child: TextField(
                controller: _nameController,
                decoration: const InputDecoration(
                  labelText: 'Keymap Name',
                  border: OutlineInputBorder(),
                  isDense: true,
                ),
                onChanged: (_) => setState(() => _isDirty = true),
              ),
            ),
            const SizedBox(width: 16),
            _buildLayerSelector(),
            const SizedBox(width: 16),
            FilledButton.icon(
              onPressed: _isSaving ? null : _saveKeymap,
              icon: _isSaving
                  ? const SizedBox(
                      width: 16,
                      height: 16,
                      child: CircularProgressIndicator(strokeWidth: 2),
                    )
                  : const Icon(Icons.save),
              label: Text(_isDirty ? 'Save*' : 'Save'),
            ),
          ],
        ),
      ),
    );
  }

  Widget _buildLayerSelector() {
    final layers = _workingKeymap.layers;
    return DropdownButton<int>(
      value: _selectedLayerIndex < layers.length ? _selectedLayerIndex : 0,
      underline: Container(), // Clean look
      items: List.generate(
        layers.length,
        (i) => DropdownMenuItem(
          value: i,
          child: Text('Layer $i: ${layers[i].name}'),
        ),
      ),
      onChanged: (value) {
        if (value == null) return;
        setState(() {
          _selectedLayerIndex = value;
          _selectedVirtualKeyId = null;
        });
      },
    );
  }

  Widget _buildCanvas() {
    return Expanded(
      child: Card(
        margin: const EdgeInsets.fromLTRB(8, 0, 8, 8),
        clipBehavior: Clip.antiAlias,
        child: Stack(
          children: [
            // Using a container to provide background
            Positioned.fill(
              child: ColoredBox(
                color: Theme.of(context).colorScheme.surfaceContainer,
              ),
            ),
            VirtualLayoutRenderer(
              layout: widget.layout,
              selectedKeyId: _selectedVirtualKeyId,
              mappedKeyIds: _bindingsForLayer.keys.toSet(),
              onKeyTap: (keyId) {
                setState(() {
                  _selectedVirtualKeyId = keyId;
                });
              },
            ),
          ],
        ),
      ),
    );
  }

  Widget _buildActionPalette() {
    return Card(
      margin: const EdgeInsets.all(8),
      elevation: 4,
      child: SizedBox(
        height: 280,
        child: Column(
          children: [
            Container(
              padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 8),
              width: double.infinity,
              color: Theme.of(context).colorScheme.surfaceContainerHighest,
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  Row(
                    children: [
                      const Icon(Icons.bolt, size: 16),
                      const SizedBox(width: 8),
                      Text(
                        'Action Palette',
                        style: Theme.of(context).textTheme.labelSmall?.copyWith(
                          fontWeight: FontWeight.bold,
                          color: Theme.of(context).colorScheme.primary,
                        ),
                      ),
                      const Spacer(),
                      if (_selectedVirtualKeyId != null)
                        TextButton.icon(
                          onPressed: _clearBinding,
                          icon: const Icon(Icons.backspace, size: 16),
                          label: const Text('Clear'),
                          style: TextButton.styleFrom(
                            padding: const EdgeInsets.symmetric(horizontal: 8),
                            minimumSize: Size.zero,
                            tapTargetSize: MaterialTapTargetSize.shrinkWrap,
                          ),
                        ),
                    ],
                  ),
                  if (_selectedVirtualKeyId != null) ...[
                    const SizedBox(height: 4),
                    RichText(
                      text: TextSpan(
                        style: Theme.of(context).textTheme.bodyMedium,
                        children: [
                          const TextSpan(text: 'Map '),
                          TextSpan(
                            text: _selectedVirtualKeyId,
                            style: const TextStyle(fontWeight: FontWeight.bold),
                          ),
                          const TextSpan(text: ' to: '),
                          TextSpan(
                            text: _bindingLabel(
                              _bindingsForLayer[_selectedVirtualKeyId],
                            ),
                            style: TextStyle(
                              fontWeight: FontWeight.bold,
                              color: Theme.of(context).colorScheme.secondary,
                            ),
                          ),
                        ],
                      ),
                    ),
                  ] else
                    Padding(
                      padding: const EdgeInsets.only(top: 4.0),
                      child: Text(
                        'Select a virtual key to map',
                        style: Theme.of(context).textTheme.bodyMedium,
                      ),
                    ),
                ],
              ),
            ),
            Expanded(
              child: Row(
                children: [
                  Expanded(
                    flex: 3,
                    child: SoftKeyboard(
                      onKeySelected: (key) =>
                          _applyBinding(ActionBinding.standardKey(value: key)),
                      keySize: 42,
                    ),
                  ),
                  const VerticalDivider(width: 1),
                  Expanded(
                    flex: 2,
                    child: Padding(
                      padding: const EdgeInsets.all(8.0),
                      child: Column(
                        crossAxisAlignment: CrossAxisAlignment.stretch,
                        children: [
                          TextField(
                            controller: _macroController,
                            decoration: const InputDecoration(
                              labelText: 'Macro',
                              isDense: true,
                              border: OutlineInputBorder(),
                            ),
                          ),
                          const SizedBox(height: 8),
                          Wrap(
                            spacing: 8,
                            runSpacing: 8,
                            children: [
                              ActionChip(
                                label: const Text('Macro'),
                                avatar: const Icon(
                                  Icons.play_circle_outline,
                                  size: 16,
                                ),
                                onPressed: () {
                                  if (_macroController.text.isNotEmpty) {
                                    _applyBinding(
                                      ActionBinding.macro(
                                        value: _macroController.text,
                                      ),
                                    );
                                  }
                                },
                              ),
                              ActionChip(
                                label: const Text('Transparent'),
                                avatar: const Icon(
                                  Icons.transit_enterexit,
                                  size: 16,
                                ),
                                onPressed: () => _applyBinding(
                                  const ActionBinding.transparent(),
                                ),
                              ),
                              // TODO: Add proper layer toggle UI
                              ActionChip(
                                label: const Text('To Layer 1'),
                                avatar: const Icon(Icons.layers, size: 16),
                                onPressed: () => _applyBinding(
                                  const ActionBinding.layerToggle(
                                    value: 'Layer 1',
                                  ),
                                ),
                              ),
                            ],
                          ),
                        ],
                      ),
                    ),
                  ),
                ],
              ),
            ),
          ],
        ),
      ),
    );
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        title: Text('Edit ${widget.keymap.name}'),
        leading: BackButton(
          onPressed: () async {
            if (_isDirty) {
              final discard = await showDialog<bool>(
                context: context,
                builder: (context) => AlertDialog(
                  title: const Text('Discard changes?'),
                  content: const Text('You have unsaved changes.'),
                  actions: [
                    TextButton(
                      onPressed: () => Navigator.pop(context, false),
                      child: const Text('Cancel'),
                    ),
                    TextButton(
                      onPressed: () => Navigator.pop(context, true),
                      child: const Text('Discard'),
                    ),
                  ],
                ),
              );
              if (discard != true) return;
            }
            // ignore: use_build_context_synchronously
            if (mounted) Navigator.pop(context);
          },
        ),
      ),
      body: Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [_buildTopBar(), _buildCanvas(), _buildActionPalette()],
      ),
    );
  }
}
