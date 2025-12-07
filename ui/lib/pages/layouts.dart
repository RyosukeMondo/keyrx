/// Layouts page for creating and editing virtual layouts.
library;

import 'package:flutter/material.dart';
import 'package:provider/provider.dart';

import '../models/virtual_layout.dart';
import '../models/virtual_layout_type.dart';
import '../services/layout_service.dart';
import '../services/service_registry.dart';

class LayoutsPage extends StatefulWidget {
  const LayoutsPage({super.key});

  @override
  State<LayoutsPage> createState() => _LayoutsPageState();
}

class _LayoutsPageState extends State<LayoutsPage> {
  final TextEditingController _idController = TextEditingController();
  final TextEditingController _nameController = TextEditingController();
  final GlobalKey<FormState> _formKey = GlobalKey<FormState>();

  List<VirtualLayout> _layouts = const [];
  List<_KeyDraft> _keyDrafts = [];
  VirtualLayoutType _selectedType = VirtualLayoutType.matrix;
  String? _selectedLayoutId;
  bool _isLoading = true;
  bool _isSaving = false;
  String? _errorMessage;

  LayoutService get _layoutService {
    try {
      return Provider.of<ServiceRegistry>(context, listen: false).layoutService;
    } on ProviderNotFoundException {
      return Provider.of<LayoutService>(context, listen: false);
    }
  }

  @override
  void initState() {
    super.initState();
    _resetDraft();
    _loadLayouts();
  }

  @override
  void dispose() {
    _idController.dispose();
    _nameController.dispose();
    for (final draft in _keyDrafts) {
      draft.dispose();
    }
    super.dispose();
  }

  Future<void> _loadLayouts() async {
    setState(() {
      _isLoading = true;
      _errorMessage = null;
    });

    final result = await _layoutService.listLayouts();
    if (!mounted) return;

    if (result.hasError) {
      setState(() {
        _isLoading = false;
        _errorMessage = result.errorMessage;
        _layouts = const [];
      });
      return;
    }

    final layouts = result.data ?? [];
    setState(() {
      _layouts = layouts;
      _isLoading = false;
      _errorMessage = null;
    });

    if (_selectedLayoutId != null) {
      VirtualLayout? existing;
      try {
        existing = layouts.firstWhere((l) => l.id == _selectedLayoutId);
      } catch (_) {
        existing = null;
      }
      if (existing != null) {
        _setDraftFromLayout(existing);
        return;
      }
    }

    if (layouts.isNotEmpty) {
      _selectLayout(layouts.first);
    } else {
      _resetDraft();
    }
  }

  void _resetDraft() {
    _setDraftFromLayout(
      VirtualLayout(
        id: _generateLayoutId(),
        name: 'New Layout',
        layoutType: VirtualLayoutType.matrix,
        keys: const [],
      ),
    );
    setState(() {
      _selectedLayoutId = null;
    });
  }

  void _setDraftFromLayout(VirtualLayout layout) {
    for (final draft in _keyDrafts) {
      draft.dispose();
    }

    final drafts = layout.keys
        .map(
          (key) => _KeyDraft(
            id: key.id,
            label: key.label,
            positionX: key.position?.x,
            positionY: key.position?.y,
            width: key.size?.width,
            height: key.size?.height,
          ),
        )
        .toList();

    setState(() {
      _keyDrafts = drafts;
      _selectedLayoutId = layout.id;
      _idController.text = layout.id;
      _nameController.text = layout.name;
      _selectedType = layout.layoutType;
    });
  }

  void _selectLayout(VirtualLayout layout) {
    _setDraftFromLayout(layout);
  }

  Future<void> _saveLayout() async {
    final formValid = _formKey.currentState?.validate() ?? false;
    if (!formValid) return;

    final id = _idController.text.trim();
    final name = _nameController.text.trim();

    final keys = _keyDrafts
        .map((draft) => draft.toVirtualKey())
        .where((k) => k != null)
        .cast<VirtualKeyDef>()
        .toList();

    final layout = VirtualLayout(
      id: id,
      name: name,
      layoutType: _selectedType,
      keys: keys,
    );

    setState(() {
      _isSaving = true;
      _errorMessage = null;
    });

    final result = await _layoutService.saveLayout(layout);
    if (!mounted) return;

    setState(() => _isSaving = false);

    if (result.hasError) {
      setState(() {
        _errorMessage = result.errorMessage;
      });
      _showSnack(
        'Failed to save layout: ${result.errorMessage}',
        isError: true,
      );
      return;
    }

    final saved = result.data ?? layout;
    final updated = [..._layouts];
    final existingIndex = updated.indexWhere((item) => item.id == saved.id);
    if (existingIndex >= 0) {
      updated[existingIndex] = saved;
    } else {
      updated.add(saved);
    }

    setState(() {
      _layouts = updated;
      _selectedLayoutId = saved.id;
    });
    _setDraftFromLayout(saved);
    _showSnack('Layout saved');
  }

  Future<void> _deleteLayout() async {
    final id = _selectedLayoutId;
    if (id == null) return;

    final confirmed = await showDialog<bool>(
      context: context,
      builder: (context) => AlertDialog(
        title: const Text('Delete layout?'),
        content: const Text('This will remove the layout permanently.'),
        actions: [
          TextButton(
            onPressed: () => Navigator.pop(context, false),
            child: const Text('Cancel'),
          ),
          FilledButton(
            onPressed: () => Navigator.pop(context, true),
            style: FilledButton.styleFrom(backgroundColor: Colors.red),
            child: const Text('Delete'),
          ),
        ],
      ),
    );

    if (confirmed != true) return;

    setState(() {
      _isSaving = true;
      _errorMessage = null;
    });

    final result = await _layoutService.deleteLayout(id);
    if (!mounted) return;

    setState(() => _isSaving = false);

    if (result.hasError) {
      setState(() {
        _errorMessage = result.errorMessage;
      });
      _showSnack('Failed to delete: ${result.errorMessage}', isError: true);
      return;
    }

    final remaining = _layouts.where((item) => item.id != id).toList();
    setState(() {
      _layouts = remaining;
      _selectedLayoutId = null;
    });
    _resetDraft();
    _showSnack('Layout deleted');
  }

  void _addKeyDraft() {
    setState(() {
      _keyDrafts.add(_KeyDraft(id: 'key_${_keyDrafts.length + 1}', label: ''));
    });
  }

  void _removeKeyDraft(int index) {
    if (index < 0 || index >= _keyDrafts.length) return;
    final draft = _keyDrafts.removeAt(index);
    draft.dispose();
    setState(() {});
  }

  String _generateLayoutId() {
    final timestamp = DateTime.now().millisecondsSinceEpoch;
    return 'layout_$timestamp';
  }

  void _showSnack(String message, {bool isError = false}) {
    ScaffoldMessenger.of(context).showSnackBar(
      SnackBar(
        content: Text(message),
        backgroundColor: isError ? Colors.red : null,
      ),
    );
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        title: const Text('Layouts'),
        actions: [
          IconButton(
            icon: const Icon(Icons.refresh),
            tooltip: 'Reload layouts',
            onPressed: _isLoading ? null : _loadLayouts,
          ),
          IconButton(
            icon: const Icon(Icons.add),
            tooltip: 'New layout',
            onPressed: _isSaving ? null : _resetDraft,
          ),
        ],
      ),
      body: _isLoading
          ? const Center(child: CircularProgressIndicator())
          : LayoutBuilder(
              builder: (context, constraints) {
                final isWide = constraints.maxWidth > 900;
                final list = _buildLayoutList();
                final editor = _buildEditor();

                if (isWide) {
                  return Row(
                    crossAxisAlignment: CrossAxisAlignment.start,
                    children: [
                      Flexible(flex: 2, child: list),
                      const VerticalDivider(width: 1),
                      Flexible(flex: 3, child: editor),
                    ],
                  );
                }

                return ListView(
                  children: [list, const Divider(height: 1), editor],
                );
              },
            ),
    );
  }

  Widget _buildLayoutList() {
    return Padding(
      padding: const EdgeInsets.all(12),
      child: Card(
        elevation: 1,
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.stretch,
          children: [
            ListTile(
              title: const Text('Saved layouts'),
              subtitle: Text(
                _errorMessage ?? '${_layouts.length} layouts',
                style: TextStyle(
                  color: _errorMessage == null
                      ? null
                      : Theme.of(context).colorScheme.error,
                ),
              ),
              trailing: IconButton(
                icon: const Icon(Icons.refresh),
                onPressed: _isLoading ? null : _loadLayouts,
              ),
            ),
            if (_layouts.isEmpty)
              Padding(
                padding: const EdgeInsets.symmetric(
                  horizontal: 16,
                  vertical: 20,
                ),
                child: Column(
                  children: [
                    const Text('No layouts yet. Create one to get started.'),
                    const SizedBox(height: 12),
                    FilledButton.icon(
                      onPressed: _isSaving ? null : _resetDraft,
                      icon: const Icon(Icons.add),
                      label: const Text('New layout'),
                    ),
                  ],
                ),
              )
            else
              ListView.separated(
                shrinkWrap: true,
                physics: const NeverScrollableScrollPhysics(),
                itemBuilder: (context, index) {
                  final layout = _layouts[index];
                  final selected = layout.id == _selectedLayoutId;
                  return ListTile(
                    selected: selected,
                    title: Text(layout.name),
                    subtitle: Text(
                      '${layout.layoutType.label} • ${layout.id}',
                      maxLines: 1,
                      overflow: TextOverflow.ellipsis,
                    ),
                    onTap: () => _selectLayout(layout),
                    trailing: IconButton(
                      icon: const Icon(Icons.delete_outline),
                      tooltip: 'Delete layout',
                      onPressed: selected ? _deleteLayout : null,
                    ),
                  );
                },
                separatorBuilder: (_, __) => const Divider(height: 1),
                itemCount: _layouts.length,
              ),
          ],
        ),
      ),
    );
  }

  Widget _buildEditor() {
    return Form(
      key: _formKey,
      child: SingleChildScrollView(
        padding: const EdgeInsets.all(16),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Row(
              children: [
                Expanded(
                  child: TextFormField(
                    controller: _idController,
                    decoration: const InputDecoration(
                      labelText: 'Layout ID',
                      helperText: 'Used for wiring and keymaps',
                    ),
                    validator: (value) {
                      if (value == null || value.trim().isEmpty) {
                        return 'ID is required';
                      }
                      return null;
                    },
                  ),
                ),
                const SizedBox(width: 16),
                Expanded(
                  child: TextFormField(
                    controller: _nameController,
                    decoration: const InputDecoration(
                      labelText: 'Display name',
                      helperText: 'Shown in wiring/mapping selectors',
                    ),
                    validator: (value) {
                      if (value == null || value.trim().isEmpty) {
                        return 'Name is required';
                      }
                      return null;
                    },
                  ),
                ),
              ],
            ),
            const SizedBox(height: 12),
            DropdownButtonFormField<VirtualLayoutType>(
              key: ValueKey(_selectedLayoutId ?? _selectedType.name),
              initialValue: _selectedType,
              decoration: const InputDecoration(labelText: 'Layout type'),
              items: VirtualLayoutType.values
                  .map(
                    (type) =>
                        DropdownMenuItem(value: type, child: Text(type.label)),
                  )
                  .toList(),
              onChanged: (type) {
                if (type == null) return;
                setState(() => _selectedType = type);
              },
            ),
            const SizedBox(height: 16),
            Row(
              mainAxisAlignment: MainAxisAlignment.spaceBetween,
              children: [
                const Text(
                  'Keys',
                  style: TextStyle(fontSize: 16, fontWeight: FontWeight.bold),
                ),
                TextButton.icon(
                  onPressed: _isSaving ? null : _addKeyDraft,
                  icon: const Icon(Icons.add),
                  label: const Text('Add key'),
                ),
              ],
            ),
            const SizedBox(height: 8),
            if (_keyDrafts.isEmpty)
              Padding(
                padding: const EdgeInsets.symmetric(
                  horizontal: 8,
                  vertical: 12,
                ),
                child: Text(
                  'No keys defined. Add keys to build your layout.',
                  style: Theme.of(context).textTheme.bodyMedium,
                ),
              )
            else
              Column(
                children: [
                  for (int i = 0; i < _keyDrafts.length; i++)
                    _KeyEditorRow(
                      keyDraft: _keyDrafts[i],
                      index: i,
                      onRemove: () => _removeKeyDraft(i),
                    ),
                ],
              ),
            const SizedBox(height: 24),
            Row(
              children: [
                FilledButton.icon(
                  onPressed: _isSaving ? null : _saveLayout,
                  icon: const Icon(Icons.save),
                  label: const Text('Save layout'),
                ),
                const SizedBox(width: 12),
                OutlinedButton(
                  onPressed: _isSaving ? null : _resetDraft,
                  child: const Text('Reset'),
                ),
              ],
            ),
            if (_errorMessage != null) ...[
              const SizedBox(height: 12),
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
}

class _KeyDraft {
  _KeyDraft({
    required String id,
    required String label,
    double? positionX,
    double? positionY,
    double? width,
    double? height,
  }) : idController = TextEditingController(text: id),
       labelController = TextEditingController(text: label),
       posXController = TextEditingController(
         text: positionX != null ? positionX.toString() : '',
       ),
       posYController = TextEditingController(
         text: positionY != null ? positionY.toString() : '',
       ),
       widthController = TextEditingController(
         text: width != null ? width.toString() : '',
       ),
       heightController = TextEditingController(
         text: height != null ? height.toString() : '',
       );

  final TextEditingController idController;
  final TextEditingController labelController;
  final TextEditingController posXController;
  final TextEditingController posYController;
  final TextEditingController widthController;
  final TextEditingController heightController;

  VirtualKeyDef? toVirtualKey() {
    final id = idController.text.trim();
    final label = labelController.text.trim();
    if (id.isEmpty || label.isEmpty) return null;

    final x = double.tryParse(posXController.text.trim());
    final y = double.tryParse(posYController.text.trim());
    final width = double.tryParse(widthController.text.trim());
    final height = double.tryParse(heightController.text.trim());

    return VirtualKeyDef(
      id: id,
      label: label,
      position: x != null && y != null ? KeyPosition(x: x, y: y) : null,
      size: width != null && height != null
          ? KeySize(width: width, height: height)
          : null,
    );
  }

  void dispose() {
    idController.dispose();
    labelController.dispose();
    posXController.dispose();
    posYController.dispose();
    widthController.dispose();
    heightController.dispose();
  }
}

class _KeyEditorRow extends StatelessWidget {
  const _KeyEditorRow({
    required this.keyDraft,
    required this.index,
    required this.onRemove,
  });

  final _KeyDraft keyDraft;
  final int index;
  final VoidCallback onRemove;

  @override
  Widget build(BuildContext context) {
    return Card(
      margin: const EdgeInsets.symmetric(vertical: 6),
      child: Padding(
        padding: const EdgeInsets.all(12),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Row(
              children: [
                Text(
                  'Key ${index + 1}',
                  style: Theme.of(context).textTheme.titleSmall,
                ),
                const Spacer(),
                IconButton(
                  icon: const Icon(Icons.delete_outline),
                  tooltip: 'Remove key',
                  onPressed: onRemove,
                ),
              ],
            ),
            Row(
              children: [
                Expanded(
                  child: TextField(
                    controller: keyDraft.idController,
                    decoration: const InputDecoration(
                      labelText: 'Virtual key ID',
                      helperText: 'e.g., K00 or ESC',
                    ),
                  ),
                ),
                const SizedBox(width: 12),
                Expanded(
                  child: TextField(
                    controller: keyDraft.labelController,
                    decoration: const InputDecoration(
                      labelText: 'Label',
                      helperText: 'Shown in editors',
                    ),
                  ),
                ),
              ],
            ),
            const SizedBox(height: 8),
            Row(
              children: [
                Expanded(
                  child: TextField(
                    controller: keyDraft.posXController,
                    decoration: const InputDecoration(
                      labelText: 'Pos X',
                      helperText: 'Optional',
                    ),
                    keyboardType: const TextInputType.numberWithOptions(
                      decimal: true,
                    ),
                  ),
                ),
                const SizedBox(width: 8),
                Expanded(
                  child: TextField(
                    controller: keyDraft.posYController,
                    decoration: const InputDecoration(
                      labelText: 'Pos Y',
                      helperText: 'Optional',
                    ),
                    keyboardType: const TextInputType.numberWithOptions(
                      decimal: true,
                    ),
                  ),
                ),
                const SizedBox(width: 8),
                Expanded(
                  child: TextField(
                    controller: keyDraft.widthController,
                    decoration: const InputDecoration(
                      labelText: 'Width',
                      helperText: 'Optional',
                    ),
                    keyboardType: const TextInputType.numberWithOptions(
                      decimal: true,
                    ),
                  ),
                ),
                const SizedBox(width: 8),
                Expanded(
                  child: TextField(
                    controller: keyDraft.heightController,
                    decoration: const InputDecoration(
                      labelText: 'Height',
                      helperText: 'Optional',
                    ),
                    keyboardType: const TextInputType.numberWithOptions(
                      decimal: true,
                    ),
                  ),
                ),
              ],
            ),
          ],
        ),
      ),
    );
  }
}
