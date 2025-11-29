/// Real-time state visualizer/debugger page.
///
/// Shows the current engine state including active layers,
/// held modifiers, and why keys were blocked.

import 'package:flutter/material.dart';

/// Real-time state debugger page.
class DebuggerPage extends StatefulWidget {
  const DebuggerPage({super.key});

  @override
  State<DebuggerPage> createState() => _DebuggerPageState();
}

class _DebuggerPageState extends State<DebuggerPage> {
  final List<String> _eventLog = [];
  bool _isRecording = false;

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        title: const Text('State Debugger'),
        actions: [
          IconButton(
            icon: Icon(_isRecording ? Icons.stop : Icons.fiber_manual_record),
            color: _isRecording ? Colors.red : null,
            onPressed: _toggleRecording,
            tooltip: _isRecording ? 'Stop Recording' : 'Start Recording',
          ),
          IconButton(
            icon: const Icon(Icons.clear_all),
            onPressed: _clearLog,
            tooltip: 'Clear Log',
          ),
        ],
      ),
      body: Row(
        children: [
          // State panel
          Expanded(
            flex: 1,
            child: _buildStatePanel(),
          ),
          const VerticalDivider(),
          // Event log
          Expanded(
            flex: 2,
            child: _buildEventLog(),
          ),
        ],
      ),
    );
  }

  Widget _buildStatePanel() {
    return ListView(
      padding: const EdgeInsets.all(16),
      children: [
        _buildSection('Active Layers', [
          'base (priority: 0)',
        ]),
        const SizedBox(height: 16),
        _buildSection('Active Modifiers', [
          'None',
        ]),
        const SizedBox(height: 16),
        _buildSection('Held Keys', [
          'None',
        ]),
      ],
    );
  }

  Widget _buildSection(String title, List<String> items) {
    return Card(
      child: Padding(
        padding: const EdgeInsets.all(12),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Text(
              title,
              style: Theme.of(context).textTheme.titleMedium,
            ),
            const Divider(),
            ...items.map((item) => Padding(
                  padding: const EdgeInsets.symmetric(vertical: 4),
                  child: Text(item),
                )),
          ],
        ),
      ),
    );
  }

  Widget _buildEventLog() {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Padding(
          padding: const EdgeInsets.all(16),
          child: Text(
            'Event Log',
            style: Theme.of(context).textTheme.titleLarge,
          ),
        ),
        Expanded(
          child: _eventLog.isEmpty
              ? const Center(
                  child: Text('No events recorded. Start recording to see events.'),
                )
              : ListView.builder(
                  itemCount: _eventLog.length,
                  itemBuilder: (context, index) {
                    return ListTile(
                      dense: true,
                      leading: Text('${index + 1}'),
                      title: Text(_eventLog[index]),
                    );
                  },
                ),
        ),
      ],
    );
  }

  void _toggleRecording() {
    setState(() {
      _isRecording = !_isRecording;
    });
    // TODO: Start/stop event recording from core
  }

  void _clearLog() {
    setState(() {
      _eventLog.clear();
    });
  }
}
