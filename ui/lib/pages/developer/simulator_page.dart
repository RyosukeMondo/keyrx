/// Simulator page for key sequence testing.
///
/// Provides an interactive virtual keyboard for building key sequences
/// and simulating them through the mapping engine.
library;

import 'package:flutter/material.dart';

import '../../services/simulation_service.dart';
import '../../widgets/keyboard.dart';

/// A key in the simulation sequence with optional hold duration.
class _SimKey {
  _SimKey({required this.code, required this.holdMs});

  final String code;
  int holdMs;
}

/// Simulator page for key sequence testing.
class SimulatorPage extends StatefulWidget {
  const SimulatorPage({super.key, required this.simulationService});

  final SimulationService simulationService;

  @override
  State<SimulatorPage> createState() => _SimulatorPageState();
}

class _SimulatorPageState extends State<SimulatorPage> {
  final List<_SimKey> _keySequence = [];
  final _scriptPathController = TextEditingController();

  bool _comboMode = false;
  bool _isSimulating = false;
  SimulationServiceResult? _result;
  String? _error;

  @override
  void dispose() {
    _scriptPathController.dispose();
    super.dispose();
  }

  void _addKey(String code) {
    setState(() {
      _keySequence.add(_SimKey(code: code, holdMs: 50));
      _result = null;
    });
  }

  void _removeKey(int index) {
    setState(() {
      _keySequence.removeAt(index);
      _result = null;
    });
  }

  void _updateHoldDuration(int index, int holdMs) {
    setState(() {
      _keySequence[index].holdMs = holdMs;
      _result = null;
    });
  }

  void _clearSequence() {
    setState(() {
      _keySequence.clear();
      _result = null;
      _error = null;
    });
  }

  Future<void> _simulate() async {
    if (_keySequence.isEmpty) {
      setState(() => _error = 'Add keys to simulate');
      return;
    }

    setState(() {
      _isSimulating = true;
      _error = null;
    });

    final keys = _keySequence.map((k) {
      return SimulationKeyInput(code: k.code, holdMs: k.holdMs);
    }).toList();

    final scriptPath = _scriptPathController.text.trim();
    final result = await widget.simulationService.simulate(
      keys,
      scriptPath: scriptPath.isNotEmpty ? scriptPath : null,
      comboMode: _comboMode,
    );

    setState(() {
      _isSimulating = false;
      if (result.hasError) {
        _error = result.errorMessage;
        _result = null;
      } else {
        _result = result;
      }
    });
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        title: const Text('Key Simulator'),
        actions: [
          if (_keySequence.isNotEmpty)
            IconButton(
              icon: const Icon(Icons.clear_all),
              tooltip: 'Clear sequence',
              onPressed: _clearSequence,
            ),
        ],
      ),
      body: Column(
        children: [
          _buildScriptInput(),
          const Divider(height: 1),
          _buildKeySequence(),
          const Divider(height: 1),
          _buildControls(),
          const Divider(height: 1),
          Expanded(child: _buildResultsArea()),
          const Divider(height: 1),
          _buildKeyboard(),
        ],
      ),
    );
  }

  Widget _buildScriptInput() {
    return Padding(
      padding: const EdgeInsets.all(8),
      child: TextField(
        controller: _scriptPathController,
        decoration: const InputDecoration(
          labelText: 'Script Path (optional)',
          hintText: 'Leave empty for active script',
          border: OutlineInputBorder(),
          prefixIcon: Icon(Icons.description),
          isDense: true,
        ),
      ),
    );
  }

  Widget _buildKeySequence() {
    return Container(
      height: 80,
      padding: const EdgeInsets.symmetric(horizontal: 8),
      child: _keySequence.isEmpty
          ? Center(
              child: Text(
                'Click keys below to build a sequence',
                style: TextStyle(color: Colors.grey[500]),
              ),
            )
          : ListView.builder(
              scrollDirection: Axis.horizontal,
              itemCount: _keySequence.length,
              itemBuilder: (context, index) {
                return _buildKeyChip(index, _keySequence[index]);
              },
            ),
    );
  }

  Widget _buildKeyChip(int index, _SimKey simKey) {
    return Padding(
      padding: const EdgeInsets.all(4),
      child: Chip(
        label: Column(
          mainAxisSize: MainAxisSize.min,
          children: [
            Text(
              simKey.code,
              style: const TextStyle(fontWeight: FontWeight.bold),
            ),
            GestureDetector(
              onTap: () => _showHoldDurationDialog(index, simKey),
              child: Text(
                '${simKey.holdMs}ms',
                style: TextStyle(fontSize: 10, color: Colors.grey[400]),
              ),
            ),
          ],
        ),
        onDeleted: () => _removeKey(index),
        deleteIcon: const Icon(Icons.close, size: 16),
      ),
    );
  }

  Future<void> _showHoldDurationDialog(int index, _SimKey simKey) async {
    final controller = TextEditingController(text: simKey.holdMs.toString());
    final result = await showDialog<int>(
      context: context,
      builder: (context) => AlertDialog(
        title: Text('Hold duration for ${simKey.code}'),
        content: TextField(
          controller: controller,
          keyboardType: TextInputType.number,
          decoration: const InputDecoration(
            labelText: 'Duration (ms)',
            suffixText: 'ms',
          ),
          autofocus: true,
        ),
        actions: [
          TextButton(
            onPressed: () => Navigator.pop(context),
            child: const Text('Cancel'),
          ),
          FilledButton(
            onPressed: () {
              final value = int.tryParse(controller.text) ?? simKey.holdMs;
              Navigator.pop(context, value.clamp(1, 10000));
            },
            child: const Text('Set'),
          ),
        ],
      ),
    );
    if (result != null) {
      _updateHoldDuration(index, result);
    }
  }

  Widget _buildControls() {
    return Padding(
      padding: const EdgeInsets.all(8),
      child: Row(
        children: [
          FilterChip(
            label: const Text('Combo Mode'),
            selected: _comboMode,
            onSelected: (v) => setState(() => _comboMode = v),
            tooltip: 'Press keys simultaneously instead of sequentially',
          ),
          const Spacer(),
          FilledButton.icon(
            onPressed: _keySequence.isEmpty || _isSimulating ? null : _simulate,
            icon: _isSimulating
                ? const SizedBox(
                    width: 16,
                    height: 16,
                    child: CircularProgressIndicator(strokeWidth: 2),
                  )
                : const Icon(Icons.play_arrow),
            label: const Text('Simulate'),
          ),
        ],
      ),
    );
  }

  Widget _buildResultsArea() {
    if (_error != null) {
      return _buildErrorBanner();
    }
    if (_result == null) {
      return Center(
        child: Column(
          mainAxisAlignment: MainAxisAlignment.center,
          children: [
            Icon(
              Icons.keyboard_alt_outlined,
              size: 48,
              color: Colors.grey[600],
            ),
            const SizedBox(height: 8),
            Text(
              'Results will appear here',
              style: TextStyle(color: Colors.grey[500]),
            ),
          ],
        ),
      );
    }
    return _buildResults();
  }

  Widget _buildErrorBanner() {
    return Center(
      child: Card(
        color: Theme.of(context).colorScheme.errorContainer,
        child: Padding(
          padding: const EdgeInsets.all(16),
          child: Row(
            mainAxisSize: MainAxisSize.min,
            children: [
              Icon(Icons.error, color: Theme.of(context).colorScheme.error),
              const SizedBox(width: 12),
              Text(
                _error!,
                style: TextStyle(
                  color: Theme.of(context).colorScheme.onErrorContainer,
                ),
              ),
            ],
          ),
        ),
      ),
    );
  }

  Widget _buildResults() {
    final result = _result!;
    return Padding(
      padding: const EdgeInsets.all(8),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          _buildStateRow(result),
          const SizedBox(height: 8),
          Expanded(child: _buildMappingsList(result.mappings)),
        ],
      ),
    );
  }

  Widget _buildStateRow(SimulationServiceResult result) {
    return Row(
      children: [
        if (result.activeLayers.isNotEmpty) ...[
          const Icon(Icons.layers, size: 16),
          const SizedBox(width: 4),
          Text('Layers: ${result.activeLayers.join(", ")}'),
          const SizedBox(width: 16),
        ],
        if (result.pending.isNotEmpty) ...[
          const Icon(Icons.pending, size: 16),
          const SizedBox(width: 4),
          Text('Pending: ${result.pending.join(", ")}'),
        ],
        if (result.activeLayers.isEmpty && result.pending.isEmpty)
          Text(
            'Base layer, no pending keys',
            style: TextStyle(color: Colors.grey[500]),
          ),
      ],
    );
  }

  Widget _buildMappingsList(List<SimulationKeyMapping> mappings) {
    if (mappings.isEmpty) {
      return Center(
        child: Text('No mappings', style: TextStyle(color: Colors.grey[500])),
      );
    }
    return ListView.builder(
      itemCount: mappings.length,
      itemBuilder: (context, index) {
        final m = mappings[index];
        return Card(
          child: ListTile(
            leading: _decisionIcon(m.decision),
            title: Row(
              children: [
                Text(m.input, style: const TextStyle(fontFamily: 'monospace')),
                const Icon(Icons.arrow_forward, size: 16),
                Text(m.output, style: const TextStyle(fontFamily: 'monospace')),
              ],
            ),
            subtitle: Text(m.decision),
          ),
        );
      },
    );
  }

  Widget _decisionIcon(String decision) {
    final lower = decision.toLowerCase();
    if (lower.contains('pass')) {
      return const Icon(Icons.arrow_forward, color: Colors.green);
    }
    if (lower.contains('remap') || lower.contains('map')) {
      return const Icon(Icons.swap_horiz, color: Colors.blue);
    }
    if (lower.contains('block') || lower.contains('drop')) {
      return const Icon(Icons.block, color: Colors.red);
    }
    return const Icon(Icons.help_outline, color: Colors.grey);
  }

  Widget _buildKeyboard() {
    return SizedBox(height: 280, child: KeyboardWidget(onKeySelected: _addKey));
  }
}
