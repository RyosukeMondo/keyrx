/// Mapping tab for binding virtual keys to actions using virtual layouts + keymaps.
library;

import 'package:flutter/material.dart';
import 'package:provider/provider.dart';

import '../models/action_binding.dart';
import '../models/keymap.dart';
import '../models/virtual_layout.dart';
import '../models/virtual_layout_type.dart';
import '../services/keymap_service.dart';
import '../services/layout_service.dart';
import '../services/service_registry.dart';
import '../widgets/soft_keyboard.dart';

class MappingPage extends StatefulWidget {
  const MappingPage({super.key});

  @override
  State<MappingPage> createState() => _MappingPageState();
}

class _MappingPageState extends State<MappingPage> {
  List<VirtualLayout> _layouts = const [];
  List<Keymap> _keymaps = const [];
  VirtualLayout? _selectedLayout;
  Keymap? _workingKeymap;
  int _selectedLayerIndex = 0;
  String? _selectedVirtualKeyId;
  bool _isLoading = true;
  bool _isSaving = false;
  String? _errorMessage;

  final TextEditingController _keymapIdController = TextEditingController();
  final TextEditingController _keymapNameController = TextEditingController();
  final TextEditingController _macroController = TextEditingController();

  LayoutService get _layoutService {
    try {
      final registry = Provider.of<ServiceRegistry>(context, listen: false);
      return registry.layoutService;
    } on ProviderNotFoundException {
      return Provider.of<LayoutService>(context, listen: false);
    }
  }

  KeymapService get _keymapService {
    try {
      final registry = Provider.of<ServiceRegistry>(context, listen: false);
      return registry.keymapService;
    } on ProviderNotFoundException {
      return Provider.of<KeymapService>(context, listen: false);
    }
  }

  @override
  void initState() {
    super.initState();
    _loadData();
  }

  @override
  void dispose() {
    _keymapIdController.dispose();
    _keymapNameController.dispose();
    _macroController.dispose();
    super.dispose();
  }

  Future<void> _loadData() async {
    setState(() {
      _isLoading = true;
      _errorMessage = null;
    });

    final layoutsResult = await _layoutService.listLayouts();
    final keymapsResult = await _keymapService.listKeymaps();

    if (!mounted) return;

    if (layoutsResult.hasError) {
      setState(() {
        _isLoading = false;
        _errorMessage = layoutsResult.errorMessage;
      });
      return;
    }

    if (keymapsResult.hasError) {
      setState(() {
        _isLoading = false;
        _errorMessage = keymapsResult.errorMessage;
      });
      return;
    }

    final layouts = layoutsResult.data ?? [];
    final keymaps = keymapsResult.data ?? [];

    setState(() {
      _layouts = layouts;
      _keymaps = keymaps;
      _isLoading = false;
    });

    if (layouts.isNotEmpty) {
      _selectLayout(layouts.first);
    }
  }

  void _selectLayout(VirtualLayout layout) {
    final matching = _keymaps
        .where((k) => k.virtualLayoutId == layout.id)
        .toList(growable: false);

    final keymap = matching.isNotEmpty
        ? matching.first
        : _buildDraftKeymap(layout);
    _setKeymap(keymap);

    setState(() {
      _selectedLayout = layout;
    });
  }

  void _setKeymap(Keymap keymap) {
    final withLayer = _ensureLayer(keymap);
    setState(() {
      _workingKeymap = withLayer;
      _selectedLayerIndex = 0;
      _selectedVirtualKeyId = null;
      _keymapIdController.text = withLayer.id;
      _keymapNameController.text = withLayer.name;
    });
  }

  Keymap _buildDraftKeymap(VirtualLayout layout) {
    final timestamp = DateTime.now().millisecondsSinceEpoch;
    return Keymap(
      id: 'keymap_$timestamp',
      name: '${layout.name} Keymap',
      virtualLayoutId: layout.id,
      layers: const [],
    );
  }

  Keymap _ensureLayer(Keymap keymap) {
    if (keymap.layers.isNotEmpty) return keymap;
    return keymap.copyWith(
      layers: [KeymapLayer(name: 'Layer 0', bindings: const {})],
    );
  }

  KeymapLayer? get _currentLayer {
    if (_workingKeymap == null) return null;
    if (_selectedLayerIndex < 0 ||
        _selectedLayerIndex >= _workingKeymap!.layers.length) {
      return null;
    }
    return _workingKeymap!.layers[_selectedLayerIndex];
  }

  Map<String, ActionBinding> get _bindingsForLayer {
    return _currentLayer?.bindings ?? const {};
  }

  void _applyBinding(ActionBinding binding) {
    final keyId = _selectedVirtualKeyId;
    if (keyId == null || _workingKeymap == null || _currentLayer == null) {
      return;
    }

    final layers = [..._workingKeymap!.layers];
    final layer = layers[_selectedLayerIndex];
    final updatedBindings = Map<String, ActionBinding>.from(layer.bindings);
    updatedBindings[keyId] = binding;
    layers[_selectedLayerIndex] = layer.copyWith(bindings: updatedBindings);

    setState(() {
      _workingKeymap = _workingKeymap!.copyWith(layers: layers);
    });

    _showSnack('Mapped $keyId to ${_bindingLabel(binding)}');
  }

  void _clearBinding() {
    final keyId = _selectedVirtualKeyId;
    if (keyId == null || _workingKeymap == null || _currentLayer == null) {
      return;
    }

    final layers = [..._workingKeymap!.layers];
    final layer = layers[_selectedLayerIndex];
    final updatedBindings = Map<String, ActionBinding>.from(layer.bindings);
    updatedBindings.remove(keyId);
    layers[_selectedLayerIndex] = layer.copyWith(bindings: updatedBindings);

    setState(() {
      _workingKeymap = _workingKeymap!.copyWith(layers: layers);
    });
  }

  Future<void> _saveKeymap() async {
    final layout = _selectedLayout;
    final keymap = _workingKeymap;
    if (layout == null || keymap == null) {
      _showSnack('Select a layout and keymap before saving', isError: true);
      return;
    }

    final id = _keymapIdController.text.trim().isEmpty
        ? keymap.id
        : _keymapIdController.text.trim();
    final name = _keymapNameController.text.trim().isEmpty
        ? keymap.name
        : _keymapNameController.text.trim();

    final toSave = keymap.copyWith(
      id: id,
      name: name,
      virtualLayoutId: layout.id,
    );

    setState(() {
      _isSaving = true;
      _errorMessage = null;
    });

    final result = await _keymapService.saveKeymap(toSave);
    if (!mounted) return;

    setState(() => _isSaving = false);

    if (result.hasError) {
      setState(() {
        _errorMessage = result.errorMessage;
      });
      _showSnack(
        'Failed to save keymap: ${result.errorMessage}',
        isError: true,
      );
      return;
    }

    final saved = result.data ?? toSave;
    final updated = [..._keymaps];
    final existingIndex = updated.indexWhere((k) => k.id == saved.id);
    if (existingIndex >= 0) {
      updated[existingIndex] = saved;
    } else {
      updated.add(saved);
    }

    setState(() {
      _keymaps = updated;
      _setKeymap(saved);
    });
    _showSnack('Keymap saved');
  }

  void _showSnack(String message, {bool isError = false}) {
    ScaffoldMessenger.of(context).showSnackBar(
      SnackBar(
        content: Text(message),
        backgroundColor: isError ? Theme.of(context).colorScheme.error : null,
      ),
    );
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

  Widget _buildLayoutSelector() {
    return DropdownButtonFormField<String>(
      value: _selectedLayout?.id,
      decoration: const InputDecoration(
        labelText: 'Virtual Layout',
        border: OutlineInputBorder(),
      ),
      items: _layouts
          .map(
            (layout) => DropdownMenuItem(
              value: layout.id,
              child: Text('${layout.name} (${layout.layoutType.label})'),
            ),
          )
          .toList(),
      onChanged: (value) {
        if (value == null) return;
        final layout = _layouts.firstWhere((l) => l.id == value);
        _selectLayout(layout);
      },
    );
  }

  Widget _buildKeymapSelector() {
    final layout = _selectedLayout;
    final options = layout == null
        ? <Keymap>[]
        : _keymaps.where((k) => k.virtualLayoutId == layout.id).toList();
    return DropdownButtonFormField<String>(
      value: _workingKeymap?.id,
      decoration: const InputDecoration(
        labelText: 'Keymap',
        border: OutlineInputBorder(),
      ),
      items: options
          .map((k) => DropdownMenuItem(value: k.id, child: Text(k.name)))
          .toList(),
      onChanged: (value) {
        if (value == null) return;
        final keymap = options.firstWhere((k) => k.id == value);
        _setKeymap(keymap);
      },
    );
  }

  Widget _buildLayerSelector() {
    final layers = _workingKeymap?.layers ?? const [];
    if (layers.isEmpty) {
      return const Text('No layers');
    }
    return DropdownButton<int>(
      value: _selectedLayerIndex < layers.length ? _selectedLayerIndex : 0,
      items: List.generate(
        layers.length,
        (i) => DropdownMenuItem(value: i, child: Text(layers[i].name)),
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

  Widget _buildHeader() {
    return Card(
      child: Padding(
        padding: const EdgeInsets.all(12),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Row(
              children: [
                Expanded(child: _buildLayoutSelector()),
                const SizedBox(width: 12),
                Expanded(child: _buildKeymapSelector()),
                const SizedBox(width: 12),
                IconButton(
                  tooltip: 'New keymap for layout',
                  onPressed: _selectedLayout == null
                      ? null
                      : () {
                          if (_selectedLayout == null) return;
                          final draft = _buildDraftKeymap(_selectedLayout!);
                          _setKeymap(draft);
                        },
                  icon: const Icon(Icons.add),
                ),
              ],
            ),
            const SizedBox(height: 12),
            Row(
              children: [
                Expanded(
                  child: TextField(
                    controller: _keymapIdController,
                    decoration: const InputDecoration(
                      labelText: 'Keymap ID',
                      border: OutlineInputBorder(),
                    ),
                  ),
                ),
                const SizedBox(width: 12),
                Expanded(
                  child: TextField(
                    controller: _keymapNameController,
                    decoration: const InputDecoration(
                      labelText: 'Display name',
                      border: OutlineInputBorder(),
                    ),
                  ),
                ),
                const SizedBox(width: 12),
                Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    const Text('Layer'),
                    const SizedBox(height: 4),
                    _buildLayerSelector(),
                  ],
                ),
                const Spacer(),
                FilledButton.icon(
                  onPressed: _isSaving ? null : _saveKeymap,
                  icon: _isSaving
                      ? const SizedBox(
                          width: 16,
                          height: 16,
                          child: CircularProgressIndicator(strokeWidth: 2),
                        )
                      : const Icon(Icons.save),
                  label: const Text('Save'),
                ),
              ],
            ),
            if (_errorMessage != null) ...[
              const SizedBox(height: 8),
              Text(
                _errorMessage!,
                style: TextStyle(color: Theme.of(context).colorScheme.error),
              ),
            ],
          ],
        ),
      ),
    );
  }

  Widget _buildVirtualLayoutView() {
    final layout = _selectedLayout;
    final keymap = _workingKeymap;

    if (layout == null) {
      return const Center(child: Text('Create a virtual layout to start.'));
    }
    if (keymap == null) {
      return const Center(child: Text('Create or select a keymap.'));
    }
    if (layout.keys.isEmpty) {
      return const Center(child: Text('This layout has no keys yet.'));
    }

    final bindings = _bindingsForLayer;
    final sortedKeys = [...layout.keys];
    sortedKeys.sort((a, b) {
      final ay = a.position?.y ?? 0;
      final by = b.position?.y ?? 0;
      final ax = a.position?.x ?? 0;
      final bx = b.position?.x ?? 0;
      final yCompare = ay.compareTo(by);
      if (yCompare != 0) return yCompare;
      return ax.compareTo(bx);
    });

    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Wrap(
          spacing: 8,
          runSpacing: 8,
          children: sortedKeys.map((key) {
            final binding = bindings[key.id];
            final selected = _selectedVirtualKeyId == key.id;
            return ChoiceChip(
              selected: selected,
              label: Column(
                mainAxisSize: MainAxisSize.min,
                children: [
                  Text(key.label, overflow: TextOverflow.ellipsis),
                  Text(
                    _bindingLabel(binding),
                    style: Theme.of(context).textTheme.bodySmall,
                    overflow: TextOverflow.ellipsis,
                  ),
                ],
              ),
              onSelected: (_) {
                setState(() {
                  _selectedVirtualKeyId = key.id;
                });
              },
            );
          }).toList(),
        ),
        const SizedBox(height: 12),
        Text(
          'Mapped keys: ${bindings.length} | Layer: ${_currentLayer?.name ?? '—'} | Layout: ${layout.name}',
          style: Theme.of(context).textTheme.bodySmall,
        ),
      ],
    );
  }

  Widget _buildActionPalette() {
    final selectedKeyLabel = _selectedVirtualKeyId == null
        ? 'Select a virtual key to map'
        : 'Map actions to ${_selectedVirtualKeyId}';

    return Card(
      elevation: 2,
      child: AnimatedContainer(
        duration: const Duration(milliseconds: 250),
        padding: const EdgeInsets.all(12),
        height: 320,
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Row(
              children: [
                const Icon(Icons.bolt_outlined, size: 18),
                const SizedBox(width: 8),
                Text(
                  selectedKeyLabel,
                  style: Theme.of(context).textTheme.titleSmall,
                ),
                const Spacer(),
                OutlinedButton.icon(
                  onPressed: _selectedVirtualKeyId == null
                      ? null
                      : _clearBinding,
                  icon: const Icon(Icons.backspace_outlined),
                  label: const Text('Clear mapping'),
                ),
              ],
            ),
            const SizedBox(height: 12),
            Expanded(
              child: Row(
                children: [
                  Expanded(
                    flex: 3,
                    child: SoftKeyboard(
                      selectedKey: null,
                      onKeySelected: (key) {
                        _applyBinding(ActionBinding.standardKey(value: key));
                      },
                      keySize: 48,
                      keySpacing: 6,
                    ),
                  ),
                  const SizedBox(width: 12),
                  Expanded(
                    flex: 2,
                    child: Column(
                      crossAxisAlignment: CrossAxisAlignment.stretch,
                      children: [
                        Text(
                          'Action palette',
                          style: Theme.of(context).textTheme.titleMedium,
                        ),
                        const SizedBox(height: 8),
                        TextField(
                          controller: _macroController,
                          decoration: const InputDecoration(
                            labelText: 'Macro text',
                            hintText: 'Enter macro / script name',
                            border: OutlineInputBorder(),
                          ),
                        ),
                        const SizedBox(height: 8),
                        Wrap(
                          spacing: 8,
                          runSpacing: 8,
                          children: [
                            FilledButton.tonalIcon(
                              icon: const Icon(Icons.play_circle_outline),
                              label: const Text('Set macro'),
                              onPressed: _selectedVirtualKeyId == null
                                  ? null
                                  : () {
                                      final text = _macroController.text.trim();
                                      if (text.isEmpty) return;
                                      _applyBinding(
                                        ActionBinding.macro(value: text),
                                      );
                                    },
                            ),
                            FilledButton.tonalIcon(
                              icon: const Icon(Icons.layers),
                              label: const Text('Toggle layer'),
                              onPressed: _selectedVirtualKeyId == null
                                  ? null
                                  : () {
                                      final target =
                                          _currentLayer?.name ?? 'Layer 0';
                                      _applyBinding(
                                        ActionBinding.layerToggle(
                                          value: target,
                                        ),
                                      );
                                    },
                            ),
                            FilledButton.tonalIcon(
                              icon: const Icon(Icons.transit_enterexit),
                              label: const Text('Transparent'),
                              onPressed: _selectedVirtualKeyId == null
                                  ? null
                                  : () => _applyBinding(
                                      const ActionBinding.transparent(),
                                    ),
                            ),
                          ],
                        ),
                        const SizedBox(height: 12),
                        Text(
                          'Tip: Click a virtual key above, then choose a standard key or macro below.',
                          style: Theme.of(context).textTheme.bodySmall,
                        ),
                      ],
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
      appBar: AppBar(title: const Text('Mapping')),
      body: SafeArea(
        child: Padding(
          padding: const EdgeInsets.all(12),
          child: _isLoading
              ? const Center(child: CircularProgressIndicator())
              : Column(
                  crossAxisAlignment: CrossAxisAlignment.stretch,
                  children: [
                    _buildHeader(),
                    const SizedBox(height: 12),
                    Expanded(
                      child: SingleChildScrollView(
                        child: _buildVirtualLayoutView(),
                      ),
                    ),
                    const SizedBox(height: 12),
                    _buildActionPalette(),
                  ],
                ),
        ),
      ),
    );
  }
}
