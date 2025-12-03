// Real-time state visualizer/debugger page.
//
// Shows the current engine state including active layers,
// held modifiers, and why keys were blocked.

import 'package:flutter/material.dart';

import '../mixins/stream_subscriber.dart';
import '../services/engine_service.dart';
import 'debugger_meters.dart';
import 'debugger_widgets.dart';

/// Real-time state debugger page.
class DebuggerPage extends StatefulWidget {
  const DebuggerPage({
    super.key,
    required this.engineService,
  });

  /// The engine service for state stream access.
  final EngineService engineService;

  @override
  State<DebuggerPage> createState() => _DebuggerPageState();
}

class _DebuggerPageState extends State<DebuggerPage>
    with SingleTickerProviderStateMixin, StreamSubscriber<DebuggerPage> {
  static const Duration _animationDuration = Duration(milliseconds: 150);

  final List<EngineSnapshot> _recent = [];
  bool _isRecording = true;
  final int _maxEvents = 300;

  // Track previous state for change detection/animation
  Set<String> _previousLayers = {};
  Set<String> _previousModifiers = {};
  Set<String> _previousHeldKeys = {};
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

    // Subscribe to state stream for immediate updates
    _subscribeToStateStream();
  }

  void _subscribeToStateStream() {
    cancelAllSubscriptions();
    subscribe<EngineSnapshot>(
      widget.engineService.stateStream,
      onData: (snapshot) {
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

          // Trigger pulse animation on significant changes
          if (_hasSignificantChange(newLayers, newModifiers, newHeldKeys)) {
            _pulseController.forward(from: 0);
          }

          _previousLayers = newLayers;
          _previousModifiers = newModifiers;
          _previousHeldKeys = newHeldKeys;
          _previousLatency = snapshot.latencyUs;
        });
      },
    );
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
      body: Row(
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
                Expanded(child: EventLogWidget(events: _recent)),
                const Divider(height: 1),
                SizedBox(
                  height: 200,
                  child: SingleChildScrollView(
                    child: TimelineWidget(
                      events: _recent,
                      animationDuration: _animationDuration,
                    ),
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
          _buildLastEventCard(snapshot),
          const SizedBox(height: 8),
          LatencyMeterCard(
            latencyUs: latencyUs,
            previousLatency: _previousLatency,
            animationDuration: _animationDuration,
          ),
          const SizedBox(height: 16),
          TagSectionCard(
            title: 'Active Layers',
            items: activeLayers.toList(),
            previousItems: _previousLayers,
            pulseAnimation: _pulseAnimation,
            animationDuration: _animationDuration,
          ),
          const SizedBox(height: 16),
          TagSectionCard(
            title: 'Active Modifiers',
            items: activeModifiers.toList(),
            previousItems: _previousModifiers,
            pulseAnimation: _pulseAnimation,
            animationDuration: _animationDuration,
          ),
          const SizedBox(height: 16),
          TagSectionCard(
            title: 'Held Keys',
            items: heldKeys.toList(),
            previousItems: _previousHeldKeys,
            pulseAnimation: _pulseAnimation,
            animationDuration: _animationDuration,
          ),
          const SizedBox(height: 16),
          PendingDecisionsCard(
            pending: pending.toList(),
            timing: timing,
            pulseAnimation: _pulseAnimation,
            animationDuration: _animationDuration,
          ),
          if (timing != null) ...[
            const SizedBox(height: 16),
            TimingCard(timing: timing),
          ],
        ],
      ),
    );
  }

  Widget _buildLastEventCard(EngineSnapshot? snapshot) {
    return AnimatedContainer(
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
    );
  }

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
}
