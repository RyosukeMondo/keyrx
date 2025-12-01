/// Real-time state visualizer/debugger page.
///
/// Shows the current engine state including active layers,
/// held modifiers, and why keys were blocked.

import 'package:flutter/material.dart';
import 'package:provider/provider.dart';

import '../services/engine_service.dart';
import '../services/service_registry.dart';

/// Real-time state debugger page.
class DebuggerPage extends StatefulWidget {
  const DebuggerPage({super.key});

  @override
  State<DebuggerPage> createState() => _DebuggerPageState();
}

class _DebuggerPageState extends State<DebuggerPage> {
  static const int _latencyWarningUs = 20000;
  static const int _latencyCautionUs = 10000;

  EngineService? _engine;
  Stream<EngineSnapshot>? _stateStream;
  final List<EngineSnapshot> _recent = [];
  bool _isRecording = true;
  final int _maxEvents = 300;

  @override
  void initState() {
    super.initState();
    final registry = Provider.of<ServiceRegistry>(context, listen: false);
    _engine = registry.engineService;
    _stateStream = _engine?.stateStream;
  }

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
            tooltip: _isRecording ? 'Pause Recording' : 'Start Recording',
          ),
          IconButton(
            icon: const Icon(Icons.clear_all),
            onPressed: _clearLog,
            tooltip: 'Clear Log',
          ),
        ],
      ),
      body: _stateStream == null
          ? const Center(child: Text('Engine stream unavailable.'))
          : StreamBuilder<EngineSnapshot>(
              stream: _stateStream,
              builder: (context, snapshot) {
                if (snapshot.hasData && _isRecording) {
                  _recent.insert(0, snapshot.data!);
                  if (_recent.length > _maxEvents) {
                    _recent.removeLast();
                  }
                }

                final latest = _recent.isNotEmpty ? _recent.first : null;
                return Row(
                  children: [
                    // State panel
                    Expanded(flex: 1, child: _buildStatePanel(latest)),
                    const VerticalDivider(),
                    // Event log
                    Expanded(
                      flex: 2,
                      child: Column(
                        children: [
                          Expanded(child: _buildEventLog()),
                          const Divider(height: 1),
                          SizedBox(
                            height: 200,
                            child: SingleChildScrollView(
                              child: _buildTimeline(),
                            ),
                          ),
                        ],
                      ),
                    ),
                  ],
                );
              },
            ),
    );
  }

  Widget _buildStatePanel(EngineSnapshot? snapshot) {
    final activeLayers = snapshot?.activeLayers ?? const [];
    final activeModifiers = snapshot?.activeModifiers ?? const [];
    final heldKeys = snapshot?.heldKeys ?? const [];
    final pending = snapshot?.pendingDecisions ?? const [];
    final latencyUs = snapshot?.latencyUs;
    final timing = snapshot?.timing;

    return SingleChildScrollView(
      padding: const EdgeInsets.all(16),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          if (snapshot?.lastEvent != null)
            Card(
              child: ListTile(
                leading: const Icon(Icons.event_note),
                title: const Text('Last Event'),
                subtitle: Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    Text(snapshot!.lastEvent!),
                    const SizedBox(height: 4),
                    Text(
                      snapshot.timestamp.toLocal().toIso8601String(),
                      style: Theme.of(context).textTheme.bodySmall,
                    ),
                  ],
                ),
              ),
            ),
          const SizedBox(height: 8),
          _buildLatencyCard(latencyUs),
          const SizedBox(height: 16),
          _buildTagSection('Active Layers', _formatList(activeLayers)),
          const SizedBox(height: 16),
          _buildTagSection('Active Modifiers', _formatList(activeModifiers)),
          const SizedBox(height: 16),
          _buildTagSection('Held Keys', _formatList(heldKeys)),
          const SizedBox(height: 16),
          _buildTagSection('Pending Decisions', _formatList(pending)),
          if (timing != null) ...[
            const SizedBox(height: 16),
            _buildTimingCard(timing),
          ],
        ],
      ),
    );
  }

  Widget _buildTagSection(String title, List<String> items) {
    final chips = items
        .map(
          (item) => Padding(
            padding: const EdgeInsets.symmetric(horizontal: 4, vertical: 4),
            child: Chip(
              label: Text(item),
              visualDensity: VisualDensity.compact,
            ),
          ),
        )
        .toList();

    return Card(
      child: Padding(
        padding: const EdgeInsets.all(12),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Text(title, style: Theme.of(context).textTheme.titleMedium),
            const Divider(),
            if (chips.isEmpty)
              const Text('None')
            else
              SingleChildScrollView(
                scrollDirection: Axis.horizontal,
                child: Row(children: chips),
              ),
          ],
        ),
      ),
    );
  }

  Widget _buildLatencyCard(int? latencyUs) {
    if (latencyUs == null) {
      return const Card(
        child: ListTile(
          leading: Icon(Icons.speed),
          title: Text('Latency'),
          subtitle: Text('No samples yet'),
        ),
      );
    }

    final color = _latencyColor(latencyUs);
    final label = latencyUs >= _latencyWarningUs
        ? 'High'
        : latencyUs >= _latencyCautionUs
        ? 'Caution'
        : 'Healthy';

    return Card(
      child: ListTile(
        leading: Icon(Icons.speed, color: color),
        title: const Text('Latency'),
        subtitle: Text('${latencyUs}µs per event'),
        trailing: Chip(
          backgroundColor: color.withOpacity(0.15),
          label: Text(label, style: TextStyle(color: color)),
        ),
      ),
    );
  }

  Widget _buildTimingCard(EngineTiming timing) {
    final items = <String>[
      if (timing.tapTimeoutMs != null) 'Tap timeout: ${timing.tapTimeoutMs}ms',
      if (timing.comboTimeoutMs != null)
        'Combo timeout: ${timing.comboTimeoutMs}ms',
      if (timing.holdDelayMs != null) 'Hold delay: ${timing.holdDelayMs}ms',
      if (timing.eagerTap != null) 'Eager tap: ${timing.eagerTap}',
      if (timing.permissiveHold != null)
        'Permissive hold: ${timing.permissiveHold}',
      if (timing.retroTap != null) 'Retro tap: ${timing.retroTap}',
    ];

    return Card(
      child: Padding(
        padding: const EdgeInsets.all(12),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Text('Timing', style: Theme.of(context).textTheme.titleMedium),
            const Divider(),
            if (items.isEmpty)
              const Text('No timing settings reported')
            else
              Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: items
                    .map(
                      (item) => Padding(
                        padding: const EdgeInsets.symmetric(vertical: 2),
                        child: Text(item),
                      ),
                    )
                    .toList(),
              ),
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
          child: _recent.isEmpty
              ? const Center(child: Text('Waiting for engine events...'))
              : ListView.builder(
                  itemCount: _recent.length,
                  itemBuilder: (context, index) {
                    final snap = _recent[index];
                    final ts = snap.timestamp.toLocal().toIso8601String();
                    return ListTile(
                      dense: true,
                      leading: Text('${index + 1}'),
                      title: Text(snap.lastEvent ?? 'Snapshot'),
                      subtitle: Text(ts),
                    );
                  },
                ),
        ),
      ],
    );
  }

  Widget _buildTimeline() {
    if (_recent.isEmpty) {
      return const SizedBox.shrink();
    }

    final latencies = _recent
        .where((e) => e.latencyUs != null)
        .map((e) => e.latencyUs!)
        .toList();
    final latestLatency = _recent.first.latencyUs;
    final avgLatency = latencies.isEmpty
        ? 0
        : latencies.reduce((a, b) => a + b) ~/ latencies.length;

    final bars = _recent.take(30).map((snap) {
      final value = snap.latencyUs?.toDouble() ?? 0;
      final widthNum = (value / (avgLatency == 0 ? 1 : avgLatency)).clamp(
        0.2,
        2,
      );
      final width = widthNum.toDouble();
      return Padding(
        padding: const EdgeInsets.symmetric(vertical: 2, horizontal: 12),
        child: Row(
          children: [
            Container(
              height: 6,
              width: 120 * width,
              color: _latencyColor(snap.latencyUs).withOpacity(0.7),
            ),
            const SizedBox(width: 8),
            Text(snap.latencyUs != null ? '${snap.latencyUs}µs' : '—'),
          ],
        ),
      );
    }).toList();

    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Padding(
          padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 8),
          child: Row(
            children: [
              const Icon(Icons.timeline),
              const SizedBox(width: 8),
              Expanded(
                child: Text(
                  'Latency Timeline (last ${bars.length})',
                  style: Theme.of(context).textTheme.titleMedium,
                  overflow: TextOverflow.ellipsis,
                ),
              ),
              const SizedBox(width: 8),
              Flexible(
                child: Text(
                  'Avg: ${avgLatency}µs  Latest: ${latestLatency ?? 0}µs',
                  style: Theme.of(context).textTheme.bodySmall,
                  textAlign: TextAlign.right,
                  overflow: TextOverflow.ellipsis,
                ),
              ),
            ],
          ),
        ),
        ...bars,
      ],
    );
  }

  List<String> _formatList(List<String> items) =>
      items.isEmpty ? const [] : items;

  void _toggleRecording() {
    setState(() {
      _isRecording = !_isRecording;
    });
  }

  void _clearLog() {
    setState(() {
      _recent.clear();
    });
  }

  Color _latencyColor(int? latencyUs) {
    if (latencyUs == null) return Colors.grey;
    if (latencyUs >= _latencyWarningUs) return Colors.redAccent;
    if (latencyUs >= _latencyCautionUs) return Colors.orangeAccent;
    return Colors.green;
  }
}
