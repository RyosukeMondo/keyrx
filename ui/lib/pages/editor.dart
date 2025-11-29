/// Visual keymap editor page.
///
/// Provides a drag-and-drop interface for creating key mappings
/// that generates the underlying Rhai script automatically.

import 'package:flutter/material.dart';

import '../widgets/keyboard.dart';

/// Visual keymap editor page.
class EditorPage extends StatefulWidget {
  const EditorPage({super.key});

  @override
  State<EditorPage> createState() => _EditorPageState();
}

class _EditorPageState extends State<EditorPage> {
  String? _selectedKey;

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        title: const Text('Keymap Editor'),
        actions: [
          IconButton(
            icon: const Icon(Icons.save),
            onPressed: _saveScript,
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
          // Keyboard visualization
          Expanded(
            flex: 2,
            child: KeyboardWidget(
              onKeySelected: (key) {
                setState(() {
                  _selectedKey = key;
                });
              },
              selectedKey: _selectedKey,
            ),
          ),
          // Key configuration panel
          Expanded(
            flex: 1,
            child: _buildConfigPanel(),
          ),
        ],
      ),
    );
  }

  Widget _buildConfigPanel() {
    if (_selectedKey == null) {
      return const Center(
        child: Text('Select a key to configure'),
      );
    }

    return Card(
      margin: const EdgeInsets.all(16),
      child: Padding(
        padding: const EdgeInsets.all(16),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Text(
              'Configuring: $_selectedKey',
              style: Theme.of(context).textTheme.titleLarge,
            ),
            const SizedBox(height: 16),
            // TODO: Add key configuration options
            const Text('Key configuration options will appear here'),
          ],
        ),
      ),
    );
  }

  void _saveScript() {
    // TODO: Save generated script
    ScaffoldMessenger.of(context).showSnackBar(
      const SnackBar(content: Text('Script saved')),
    );
  }

  void _viewScript() {
    // TODO: Show generated script dialog
    showDialog(
      context: context,
      builder: (context) => AlertDialog(
        title: const Text('Generated Script'),
        content: const SingleChildScrollView(
          child: Text('// Generated Rhai script will appear here'),
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
}
