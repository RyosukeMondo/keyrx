// Real-time state visualizer/debugger page.
//
// Shows the current engine state including active layers,
// held modifiers, and why keys were blocked.

import 'dart:async';

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

class _DebuggerPageState extends State<DebuggerPage>
    with SingleTickerProviderStateMixin {
  static const int _latencyWarningUs = 20000;
  static const int _latencyCautionUs = 10000;
  static const Duration _animationDuration = Duration(milliseconds: 150);

  EngineService? _engine;
  Stream<EngineSnapshot>? _stateStream;
  StreamSubscription<EngineSnapshot>? _streamSubscription;
  final List<EngineSnapshot> _recent = [];
  bool _isRecording = true;
  final int _maxEvents = 300;

  // Track previous state for change detection/animation
  Set<String> _previousLayers = {};
  Set<String> _previousModifiers = {};
  Set<String> _previousHeldKeys = {};
  Set<String> _previousPending = {};
  int? _previousLatency;

  // Animation controller for pulse effects
  late AnimationController _pulseController;
  late Animation<double> _pulseAnimation;

  @override
  void initState() {
    super.initState();

    // Set up pulse animation for state changes
    _pulseController = AnimationController(
      duration: const Duration(milliseconds: 300),
      vsync: this,
    );
    _pulseAnimation = Tween<double>(begin: 1.0, end: 1.1).animate(
      CurvedAnimation(parent: _pulseController, curve: Curves.easeInOut),
    );

    final registry = Provider.of<ServiceRegistry>(context, listen: false);
    _engine = registry.engineService;
    _stateStream = _engine?.stateStream;

    // Subscribe to state stream for immediate updates
    _subscribeToStateStream();
  }

  void _subscribeToStateStream() {
    _streamSubscription?.cancel();
    _streamSubscription = _stateStream?.listen((snapshot) {
      if (!_isRecording) return;

      setState(() {
        _recent.insert(0, snapshot);
        if (_recent.length > _maxEvents) {
          _recent.removeLast();
        }

        // Track changes for animation
        final newLayers = snapshot.activeLayers.toSet();
        final newModifiers = snapshot.activeModifiers.toSet();
        final newHeldKeys = snapshot.heldKeys.toSet();
        final newPending = snapshot.pendingDecisions.toSet();

        // Trigger pulse animation on significant changes
        if (_hasSignificantChange(newLayers, newModifiers, newHeldKeys)) {
          _pulseController.forward(from: 0);
        }

        _previousLayers = newLayers;
        _previousModifiers = newModifiers;
        _previousHeldKeys = newHeldKeys;
        _previousPending = newPending;
        _previousLatency = snapshot.latencyUs;
      });
    });
  }

  bool _hasSignificantChange(
    Set<String> newLayers,
    Set<String> newModifiers,
    Set<String> newHeldKeys,
  ) {
    return !_setEquals(newLayers, _previousLayers) ||
        !_setEquals(newModifiers, _previousModifiers) ||
        !_setEquals(newHeldKeys, _previousHeldKeys);
  }

  bool _setEquals(Set<String> a, Set<String> b) {
    if (a.length != b.length) return false;
    return a.every(b.contains);
  }

  @override
  void dispose() {
    _streamSubscription?.cancel();
    _pulseController.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final latest = _recent.isNotEmpty ? _recent.first : null;

    return Scaffold(
      appBar: AppBar(
        title: const Text('State Debugger'),
        actions: [
          // Live indicator
          AnimatedContainer(
            duration: _animationDuration,
            padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 4),
            decoration: BoxDecoration(
              color: _isRecording
                  ? Colors.green.withValues(alpha: 0.2)
                  : Colors.grey.withValues(alpha: 0.2),
              borderRadius: BorderRadius.circular(12),
            ),
            child: Row(
              mainAxisSize: MainAxisSize.min,
              children: [
                AnimatedContainer(
                  duration: _animationDuration,
                  width: 8,
                  height: 8,
                  decoration: BoxDecoration(
                    color: _isRecording ? Colors.green : Colors.grey,
                    shape: BoxShape.circle,
                  ),
                ),
                const SizedBox(width: 4),
                Text(
                  _isRecording ? 'LIVE' : 'PAUSED',
                  style: TextStyle(
                    fontSize: 10,
                    fontWeight: FontWeight.bold,
                    color: _isRecording ? Colors.green : Colors.grey,
                  ),
                ),
              ],
            ),
          ),
          const SizedBox(width: 8),
          IconButton(
            icon: Icon(_isRecording ? Icons.pause : Icons.play_arrow),
            color: _isRecording ? Colors.red : Colors.green,
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
          : Row(
              children: [
                // State panel with animation
                Expanded(
                  flex: 1,
                  child: AnimatedBuilder(
                    animation: _pulseAnimation,
                    builder: (context, child) => _buildStatePanel(latest),
                  ),
                ),
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
          // Last event card with animated highlight
          AnimatedContainer(
            duration: _animationDuration,
            decoration: BoxDecoration(
              borderRadius: BorderRadius.circular(12),
              border: snapshot?.lastEvent != null
                  ? Border.all(
                      color: Theme.of(context)
                          .colorScheme
                          .primary
                          .withValues(alpha: 0.3),
                      width: 1,
                    )
                  : null,
            ),
            child: snapshot?.lastEvent != null
                ? Card(
                    margin: EdgeInsets.zero,
                    child: ListTile(
                      leading: AnimatedScale(
                        scale: _pulseAnimation.value,
                        duration: _animationDuration,
                        child: const Icon(Icons.event_note),
                      ),
                      title: const Text('Last Event'),
                      subtitle: Column(
                        crossAxisAlignment: CrossAxisAlignment.start,
                        children: [
                          Text(
                            snapshot!.lastEvent!,
                            style: const TextStyle(fontWeight: FontWeight.bold),
                          ),
                          const SizedBox(height: 4),
                          Text(
                            snapshot.timestamp.toLocal().toIso8601String(),
                            style: Theme.of(context).textTheme.bodySmall,
                          ),
                        ],
                      ),
                    ),
                  )
                : Card(
                    margin: EdgeInsets.zero,
                    child: ListTile(
                      leading: const Icon(Icons.event_note, color: Colors.grey),
                      title: const Text('Last Event'),
                      subtitle: Text(
                        'Waiting for events...',
                        style: TextStyle(
                          color: Theme.of(context).textTheme.bodySmall?.color,
                          fontStyle: FontStyle.italic,
                        ),
                      ),
                    ),
                  ),
          ),
          const SizedBox(height: 8),
          _buildLatencyCard(latencyUs),
          const SizedBox(height: 16),
          _buildTagSection(
            'Active Layers',
            _formatList(activeLayers),
            previousItems: _previousLayers,
          ),
          const SizedBox(height: 16),
          _buildTagSection(
            'Active Modifiers',
            _formatList(activeModifiers),
            previousItems: _previousModifiers,
          ),
          const SizedBox(height: 16),
          _buildTagSection(
            'Held Keys',
            _formatList(heldKeys),
            previousItems: _previousHeldKeys,
          ),
          const SizedBox(height: 16),
          _buildTagSection(
            'Pending Decisions',
            _formatList(pending),
            previousItems: _previousPending,
          ),
          if (timing != null) ...[
            const SizedBox(height: 16),
            _buildTimingCard(timing),
          ],
        ],
      ),
    );
  }

  Widget _buildTagSection(
    String title,
    List<String> items, {
    Set<String>? previousItems,
  }) {
    final previousSet = previousItems ?? <String>{};
    final currentSet = items.toSet();

    final chips = items.map((item) {
      final isNew = !previousSet.contains(item);

      return AnimatedContainer(
        duration: _animationDuration,
        padding: const EdgeInsets.symmetric(horizontal: 4, vertical: 4),
        child: AnimatedScale(
          scale: isNew ? _pulseAnimation.value : 1.0,
          duration: _animationDuration,
          child: Chip(
            label: Text(item),
            visualDensity: VisualDensity.compact,
            backgroundColor: isNew
                ? Theme.of(context).colorScheme.primaryContainer
                : null,
            side: isNew
                ? BorderSide(
                    color: Theme.of(context).colorScheme.primary,
                    width: 2,
                  )
                : null,
          ),
        ),
      );
    }).toList();

    final removedCount = previousSet.difference(currentSet).length;

    return Card(
      child: Padding(
        padding: const EdgeInsets.all(12),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Row(
              children: [
                Expanded(
                  child: Text(
                    title,
                    style: Theme.of(context).textTheme.titleMedium,
                    overflow: TextOverflow.ellipsis,
                  ),
                ),
                const SizedBox(width: 8),
                AnimatedContainer(
                  duration: _animationDuration,
                  padding:
                      const EdgeInsets.symmetric(horizontal: 8, vertical: 2),
                  decoration: BoxDecoration(
                    color: items.isEmpty
                        ? Colors.grey.withValues(alpha: 0.2)
                        : Theme.of(context)
                            .colorScheme
                            .primary
                            .withValues(alpha: 0.2),
                    borderRadius: BorderRadius.circular(12),
                  ),
                  child: Text(
                    '${items.length}',
                    style: TextStyle(
                      fontSize: 12,
                      fontWeight: FontWeight.bold,
                      color: items.isEmpty
                          ? Colors.grey
                          : Theme.of(context).colorScheme.primary,
                    ),
                  ),
                ),
              ],
            ),
            const Divider(),
            if (chips.isEmpty)
              AnimatedOpacity(
                opacity: 1.0,
                duration: _animationDuration,
                child: Text(
                  'None',
                  style: TextStyle(
                    color: Theme.of(context).textTheme.bodySmall?.color,
                    fontStyle: FontStyle.italic,
                  ),
                ),
              )
            else
              SingleChildScrollView(
                scrollDirection: Axis.horizontal,
                child: Row(children: chips),
              ),
            if (removedCount > 0)
              AnimatedOpacity(
                opacity: 0.6,
                duration: _animationDuration,
                child: Padding(
                  padding: const EdgeInsets.only(top: 4),
                  child: Text(
                    '$removedCount removed',
                    style: TextStyle(
                      fontSize: 11,
                      color: Colors.red.shade300,
                      fontStyle: FontStyle.italic,
                    ),
                  ),
                ),
              ),
          ],
        ),
      ),
    );
  }

  Widget _buildLatencyCard(int? latencyUs) {
    if (latencyUs == null) {
      return Card(
        child: Padding(
          padding: const EdgeInsets.all(12),
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              Row(
                children: [
                  const Icon(Icons.speed, color: Colors.grey),
                  const SizedBox(width: 8),
                  Text(
                    'Latency',
                    style: Theme.of(context).textTheme.titleMedium,
                  ),
                ],
              ),
              const SizedBox(height: 8),
              Text(
                'Waiting for samples...',
                style: TextStyle(
                  color: Theme.of(context).textTheme.bodySmall?.color,
                  fontStyle: FontStyle.italic,
                ),
              ),
            ],
          ),
        ),
      );
    }

    final color = _latencyColor(latencyUs);
    final label = latencyUs >= _latencyWarningUs
        ? 'High'
        : latencyUs >= _latencyCautionUs
            ? 'Caution'
            : 'Healthy';

    // Calculate meter progress (0-1, capped at warning threshold * 2)
    final maxLatency = _latencyWarningUs * 2;
    final progress = (latencyUs / maxLatency).clamp(0.0, 1.0);

    // Detect latency change for animation
    final latencyChanged = _previousLatency != null && latencyUs != _previousLatency;
    final latencyIncreased =
        _previousLatency != null && latencyUs > _previousLatency!;

    return Card(
      child: Padding(
        padding: const EdgeInsets.all(12),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Row(
              children: [
                AnimatedContainer(
                  duration: _animationDuration,
                  child: Icon(Icons.speed, color: color),
                ),
                const SizedBox(width: 8),
                Expanded(
                  child: Text(
                    'Latency',
                    style: Theme.of(context).textTheme.titleMedium,
                    overflow: TextOverflow.ellipsis,
                  ),
                ),
                const SizedBox(width: 8),
                AnimatedContainer(
                  duration: _animationDuration,
                  padding:
                      const EdgeInsets.symmetric(horizontal: 10, vertical: 4),
                  decoration: BoxDecoration(
                    color: color.withValues(alpha: 0.15),
                    borderRadius: BorderRadius.circular(12),
                    border: Border.all(
                      color: color.withValues(alpha: 0.3),
                      width: 1,
                    ),
                  ),
                  child: Row(
                    mainAxisSize: MainAxisSize.min,
                    children: [
                      if (latencyChanged)
                        AnimatedOpacity(
                          opacity: 1.0,
                          duration: _animationDuration,
                          child: Icon(
                            latencyIncreased
                                ? Icons.arrow_upward
                                : Icons.arrow_downward,
                            size: 14,
                            color: latencyIncreased ? Colors.red : Colors.green,
                          ),
                        ),
                      Text(
                        label,
                        style: TextStyle(
                          color: color,
                          fontWeight: FontWeight.bold,
                          fontSize: 12,
                        ),
                      ),
                    ],
                  ),
                ),
              ],
            ),
            const SizedBox(height: 12),
            // Animated latency meter
            ClipRRect(
              borderRadius: BorderRadius.circular(4),
              child: AnimatedContainer(
                duration: _animationDuration,
                height: 8,
                child: LinearProgressIndicator(
                  value: progress,
                  backgroundColor: Colors.grey.withValues(alpha: 0.2),
                  valueColor: AlwaysStoppedAnimation<Color>(color),
                ),
              ),
            ),
            const SizedBox(height: 8),
            Row(
              children: [
                Expanded(
                  child: AnimatedDefaultTextStyle(
                    duration: _animationDuration,
                    style: TextStyle(
                      fontSize: 24,
                      fontWeight: FontWeight.bold,
                      color: color,
                    ),
                    child: Text('$latencyUs'),
                  ),
                ),
                Text(
                  'µs per event',
                  style: Theme.of(context).textTheme.bodySmall,
                ),
              ],
            ),
            // Threshold markers
            const SizedBox(height: 4),
            Wrap(
              alignment: WrapAlignment.spaceBetween,
              spacing: 8,
              runSpacing: 4,
              children: [
                _buildThresholdMarker('0', Colors.green),
                _buildThresholdMarker(
                    '${_latencyCautionUs ~/ 1000}k', Colors.orange),
                _buildThresholdMarker(
                    '${_latencyWarningUs ~/ 1000}k', Colors.red),
              ],
            ),
          ],
        ),
      ),
    );
  }

  Widget _buildThresholdMarker(String label, Color color) {
    return Row(
      mainAxisSize: MainAxisSize.min,
      children: [
        Container(
          width: 8,
          height: 8,
          decoration: BoxDecoration(
            color: color.withValues(alpha: 0.3),
            shape: BoxShape.circle,
            border: Border.all(color: color, width: 1),
          ),
        ),
        const SizedBox(width: 4),
        Text(
          label,
          style: TextStyle(fontSize: 10, color: color),
        ),
      ],
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
            AnimatedContainer(
              duration: _animationDuration,
              height: 6,
              width: 120 * width,
              color: _latencyColor(snap.latencyUs).withValues(alpha: 0.7),
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
                  'Avg: $avgLatency µs  Latest: ${latestLatency ?? 0}µs',
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
