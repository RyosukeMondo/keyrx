/// Layouts page for creating and editing virtual layouts.
library;

import 'package:flutter/material.dart';
import 'package:provider/provider.dart';

import '../models/virtual_layout.dart';
import '../models/virtual_layout_type.dart';
import '../services/layout_service.dart';
import '../services/service_registry.dart';
import 'layouts/layout_selector.dart';
import 'layouts/layout_visual_editor.dart';

class LayoutsPage extends StatefulWidget {
  const LayoutsPage({super.key});

  @override
  State<LayoutsPage> createState() => _LayoutsPageState();
}

class _LayoutsPageState extends State<LayoutsPage> {
  final TextEditingController _idController = TextEditingController();
  final TextEditingController _nameController = TextEditingController();

  List<VirtualLayout> _layouts = const [];
  VirtualLayout? _activeLayout;

  // Temporary storage for keys being edited before saving
  List<VirtualKeyDef> _editingKeys = [];

  // Metadata being edited
  VirtualLayoutType _editingType = VirtualLayoutType.matrix;

  bool _isLoading = true;
  bool _isSaving = false;

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
    _loadLayouts();
  }

  @override
  void dispose() {
    _idController.dispose();
    _nameController.dispose();
    super.dispose();
  }

  Future<void> _loadLayouts() async {
    setState(() {
      _isLoading = true;
    });

    final result = await _layoutService.listLayouts();
    if (!mounted) return;

    if (result.hasError) {
      setState(() {
        _isLoading = false;
        _layouts = const [];
      });
      return;
    }

    setState(() {
      _layouts = result.data ?? [];
      _isLoading = false;
    });
  }

  void _openLayout(VirtualLayout layout) {
    setState(() {
      _activeLayout = layout;
      _idController.text = layout.id;
      _nameController.text = layout.name;
      _editingType = layout.layoutType;
      _editingKeys = List.from(layout.keys);
    });
  }

  void _createFreeform() {
    _openLayout(
      VirtualLayout(
        id: _generateLayoutId(),
        name: 'New Custom Layout',
        layoutType: VirtualLayoutType.matrix,
        keys: const [],
      ),
    );
  }

  void _createGrid(List<VirtualKeyDef> keys) {
    _openLayout(
      VirtualLayout(
        id: _generateLayoutId(),
        name: 'New Grid Layout',
        layoutType: VirtualLayoutType.matrix,
        keys: keys,
      ),
    );
  }

  void _closeEditor() {
    setState(() {
      _activeLayout = null;
      _editingKeys = [];
    });
  }

  Future<void> _saveLayout() async {
    // If ID/Name changed in UI (we need UI for that in editor), update here.
    // For now assuming the top bar handles it or we show a dialog.
    // Let's create a simple dialog for metadata if saving for the first time?
    // Or just use the controller values.

    final id = _idController.text.trim();
    final name = _nameController.text.trim();

    if (id.isEmpty || name.isEmpty) {
      ScaffoldMessenger.of(context).showSnackBar(
        const SnackBar(
          content: Text('Layout ID and Name are required'),
          backgroundColor: Colors.red,
        ),
      );
      return;
    }

    final layout = VirtualLayout(
      id: id,
      name: name,
      layoutType: _editingType,
      keys: _editingKeys,
    );

    setState(() {
      _isSaving = true;
    });

    final result = await _layoutService.saveLayout(layout);
    if (!mounted) return;

    setState(() => _isSaving = false);

    if (result.hasError) {
      ScaffoldMessenger.of(context).showSnackBar(
        SnackBar(
          content: Text('Error: ${result.errorMessage}'),
          backgroundColor: Colors.red,
        ),
      );
      return;
    }

    final saved = result.data ?? layout;

    // Refresh list
    await _loadLayouts();
    if (!mounted) return;

    // Stay open with saved data
    _openLayout(saved);
    ScaffoldMessenger.of(
      context,
    ).showSnackBar(const SnackBar(content: Text('Layout saved')));
  }

  Future<void> _deleteLayout([String? id]) async {
    final targetId = id ?? _activeLayout?.id;
    if (targetId == null) return;

    final confirmed = await showDialog<bool>(
      context: context,
      builder: (context) => AlertDialog(
        title: const Text('Delete layout?'),
        content: const Text('This cannot be undone.'),
        actions: [
          TextButton(
            onPressed: () => Navigator.pop(context, false),
            child: const Text('Cancel'),
          ),
          TextButton(
            onPressed: () => Navigator.pop(context, true),
            child: const Text('Delete', style: TextStyle(color: Colors.red)),
          ),
        ],
      ),
    );

    if (confirmed != true) return;

    await _layoutService.deleteLayout(targetId);
    await _loadLayouts();
    if (_activeLayout?.id == targetId) {
      _closeEditor();
    }
  }

  String _generateLayoutId() {
    final timestamp = DateTime.now().millisecondsSinceEpoch;
    return 'layout_$timestamp';
  }

  @override
  Widget build(BuildContext context) {
    // If no layout is active, show the Selector
    if (_activeLayout == null) {
      if (_isLoading) return const Center(child: CircularProgressIndicator());

      return Scaffold(
        appBar: AppBar(
          title: const Text('Layouts'),
          actions: [
            IconButton(
              icon: const Icon(Icons.refresh),
              onPressed: _loadLayouts,
            ),
          ],
        ),
        body: LayoutSelector(
          layouts: _layouts,
          onSelectLayout: _openLayout,
          onDeleteLayout: _deleteLayout,
          onCreateFreeform: _createFreeform,
          onCreateGrid: _createGrid,
        ),
      );
    }

    // Otherwise show the Editor
    return Scaffold(
      appBar: AppBar(
        leading: IconButton(
          icon: const Icon(Icons.arrow_back),
          onPressed: _closeEditor,
        ),
        title: Row(
          children: [
            // Editable Title? For now just Text
            Expanded(
              child: TextField(
                controller: _nameController,
                decoration: const InputDecoration(
                  border: InputBorder.none,
                  hintText: 'Layout Name',
                ),
                style: const TextStyle(
                  fontWeight: FontWeight.bold,
                  fontSize: 18,
                ),
              ),
            ),
          ],
        ),
        actions: [
          IconButton(
            icon: const Icon(Icons.delete_outline),
            onPressed: () => _deleteLayout(),
            tooltip: 'Delete',
          ),
          const SizedBox(width: 8),
          FilledButton.icon(
            onPressed: _isSaving ? null : _saveLayout,
            icon: const Icon(Icons.save),
            label: const Text('Save'),
          ),
          const SizedBox(width: 16),
        ],
      ),
      body: Column(
        children: [
          // Metadata Bar (ID, Type)
          // Metadata Bar removed (ID is hidden)
          /*
          Padding(
            padding: const EdgeInsets.symmetric(horizontal: 16.0, vertical: 8),
            child: Row(
              children: [
                Expanded(
                  child: TextField(
                    controller: _idController,
                    decoration: const InputDecoration(
                      labelText: 'Layout ID',
                      isDense: true,
                    ),
                  ),
                ),
                const SizedBox(width: 16),
                // Layout Type Dropdown could go here
              ],
            ),
          ),
          const Divider(height: 1),
          */
          const Divider(height: 1),
          // Visual Editor
          Expanded(
            child: LayoutVisualEditor(
              keys: _editingKeys,
              onKeysChanged: (keys) => setState(() => _editingKeys = keys),
            ),
          ),
        ],
      ),
    );
  }
}
