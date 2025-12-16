/// Monitor view for viewing real-time daemon events.
///
/// This replaces the old CLI/FFI monitor with a passive listener connected
/// to the `keyrx daemon` via IPC.
library;

import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:intl/intl.dart';

import '../models/daemon_protocol.dart';
import '../services/daemon_service.dart';
import '../state/daemon_providers.dart';

class MonitorView extends ConsumerStatefulWidget {
  const MonitorView({super.key});

  @override
  ConsumerState<MonitorView> createState() => _MonitorViewState();
}

class _MonitorViewState extends ConsumerState<MonitorView>
    with AutomaticKeepAliveClientMixin {
  final List<_EventDisplay> _events = [];
  final int _maxEvents = 500; // Buffer limit

  DaemonConnectionState _connectionState = DaemonConnectionState.disconnected;

  // Event filtering
  bool _showInput = true;
  bool _showOutput = true;
  bool _showStateChange = true;

  StreamSubscription<DaemonEvent>? _eventSubscription;
  StreamSubscription<DaemonConnectionState>? _stateSubscription;

  final DateFormat _timeFormat = DateFormat('HH:mm:ss.SSS');

  @override
  bool get wantKeepAlive => true;

  @override
  void initState() {
    super.initState();
    WidgetsBinding.instance.addPostFrameCallback((_) => _initialize());
  }

  @override
  void dispose() {
    _eventSubscription?.cancel();
    _stateSubscription?.cancel();
    super.dispose();
  }

  Future<void> _initialize() async {
    final daemonService = ref.read(daemonServiceProvider);

    // Listen to connection state
    _stateSubscription = daemonService.connectionState.listen((state) {
      if (mounted) {
        setState(() {
          _connectionState = state;
        });
      }
    });

    // Listen to events
    _eventSubscription = daemonService.events.listen((event) {
      if (mounted) {
        _addEvent(event);
      }
    });

    // Try to connect if not already connected
    if (_connectionState == DaemonConnectionState.disconnected) {
      try {
        await daemonService.connect();
        await daemonService.subscribeEvents();
      } catch (e) {
        // Silent failure on init, user can see status
      }
    }
  }

  void _addEvent(DaemonEvent event) {
    setState(() {
      _events.insert(
        0,
        _EventDisplay(
          event: event,
          timestamp: DateTime.now(),
        ),
      );

      // Limit buffer size
      if (_events.length > _maxEvents) {
        _events.removeRange(_maxEvents, _events.length);
      }
    });
  }

  void _clearEvents() {
    setState(() {
      _events.clear();
    });
  }

  @override
  Widget build(BuildContext context) {
    super.build(context);
    return Column(
      children: [
        _buildToolbar(),
        Expanded(child: _buildEventList()),
      ],
    );
  }

  Widget _buildToolbar() {
    return Container(
      padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 8),
      decoration: BoxDecoration(
        color: Theme.of(context).colorScheme.surfaceContainerHighest,
        border: Border(
          bottom: BorderSide(
            color: Theme.of(context).dividerColor,
          ),
        ),
      ),
      child: Row(
        children: [
          // Connection Status
          Icon(
            _connectionState == DaemonConnectionState.connected
                ? Icons.check_circle
                : Icons.error_outline,
            size: 16,
            color: _connectionState == DaemonConnectionState.connected
                ? Colors.green
                : Colors.red,
          ),
          const SizedBox(width: 8),
          Text(
            _connectionState == DaemonConnectionState.connected
                ? 'Connected'
                : 'Disconnected',
            style: Theme.of(context).textTheme.bodySmall,
          ),
          const Spacer(),
          // Filter Button
          IconButton(
            icon: Icon(
              Icons.filter_list,
              size: 20,
              color: (_showInput && _showOutput && _showStateChange)
                  ? null
                  : Theme.of(context).colorScheme.primary,
            ),
            tooltip: 'Filter events',
            onPressed: _showFilterDialog,
          ),
          // Clear Button
          IconButton(
            icon: const Icon(Icons.clear_all, size: 20),
            tooltip: 'Clear events',
            onPressed: _events.isEmpty ? null : _clearEvents,
          ),
        ],
      ),
    );
  }

  Widget _buildEventList() {
    final filteredEvents = _events.where((e) {
      return e.event.when(
        input: (_) => _showInput,
        output: (_) => _showOutput,
        stateChange: (_) => _showStateChange,
      );
    }).toList();

    if (filteredEvents.isEmpty) {
      return Center(
        child: Column(
          mainAxisAlignment: MainAxisAlignment.center,
          children: [
            Icon(
              Icons.visibility_off,
              size: 48,
              color: Theme.of(context).colorScheme.outline,
            ),
            const SizedBox(height: 16),
            Text(
              _events.isEmpty ? 'No events yet' : 'No events match filter',
              style: Theme.of(context).textTheme.titleMedium,
            ),
            const SizedBox(height: 8),
            Text(
              _events.isEmpty
                  ? 'Listening for daemon events...'
                  : 'Try adjusting the filter',
              style: Theme.of(context).textTheme.bodySmall,
            ),
          ],
        ),
      );
    }

    return ListView.separated(
      padding: const EdgeInsets.all(8),
      itemCount: filteredEvents.length,
      separatorBuilder: (context, index) => const Divider(height: 1),
      itemBuilder: (context, index) {
        final eventDisplay = filteredEvents[index];
        return _buildEventTile(eventDisplay);
      },
    );
  }

  Widget _buildEventTile(_EventDisplay eventDisplay) {
    return eventDisplay.event.when(
      input: (event) => _buildInputEventTile(event, eventDisplay.timestamp),
      output: (event) => _buildOutputEventTile(event, eventDisplay.timestamp),
      stateChange: (event) =>
          _buildStateChangeEventTile(event, eventDisplay.timestamp),
    );
  }

  Widget _buildInputEventTile(InputEvent event, DateTime timestamp) {
    final isPress = event.eventType == 'key_press';

    return ListTile(
      dense: true,
      visualDensity: VisualDensity.compact,
      leading: CircleAvatar(
        radius: 12,
        backgroundColor: Colors.blue.withValues(alpha: 0.2),
        child: Icon(
          isPress ? Icons.keyboard_arrow_down : Icons.keyboard_arrow_up,
          color: Colors.blue,
          size: 14,
        ),
      ),
      title: Row(
        children: [
          Text(
            event.keyName,
            style: const TextStyle(fontWeight: FontWeight.bold, fontSize: 13),
          ),
          const SizedBox(width: 8),
          Container(
            padding: const EdgeInsets.symmetric(horizontal: 4, vertical: 2),
            decoration: BoxDecoration(
              color: isPress
                  ? Colors.green.withValues(alpha: 0.1)
                  : Colors.red.withValues(alpha: 0.1),
              borderRadius: BorderRadius.circular(4),
            ),
            child: Text(
              isPress ? 'PRESS' : 'RELEASE',
              style: TextStyle(
                fontSize: 10,
                color: isPress ? Colors.green : Colors.red,
                fontWeight: FontWeight.bold,
              ),
            ),
          ),
        ],
      ),
      subtitle: Text(
        '${event.deviceName} • ${_timeFormat.format(timestamp)}',
        style: Theme.of(context).textTheme.bodySmall?.copyWith(fontSize: 11),
      ),
      trailing: Text(
        '${event.keyCode}',
        style: Theme.of(context).textTheme.bodySmall?.copyWith(
              fontFamily: 'monospace',
              fontSize: 11,
            ),
      ),
    );
  }

  Widget _buildOutputEventTile(OutputEvent event, DateTime timestamp) {
    final isPress = event.actionType == 'key_press';

    return ListTile(
      dense: true,
      visualDensity: VisualDensity.compact,
      leading: CircleAvatar(
        radius: 12,
        backgroundColor: Colors.orange.withValues(alpha: 0.2),
        child: Icon(
          isPress ? Icons.keyboard_arrow_down : Icons.keyboard_arrow_up,
          color: Colors.orange,
          size: 14,
        ),
      ),
      title: Row(
        children: [
          const Icon(Icons.output, size: 12, color: Colors.grey),
          const SizedBox(width: 4),
          Text(
            event.keyName,
            style: const TextStyle(fontWeight: FontWeight.bold, fontSize: 13),
          ),
          const SizedBox(width: 8),
          Container(
            padding: const EdgeInsets.symmetric(horizontal: 4, vertical: 2),
            decoration: BoxDecoration(
              color: isPress
                  ? Colors.green.withValues(alpha: 0.1)
                  : Colors.red.withValues(alpha: 0.1),
              borderRadius: BorderRadius.circular(4),
            ),
            child: Text(
              isPress ? 'PRESS' : 'RELEASE',
              style: TextStyle(
                fontSize: 10,
                color: isPress ? Colors.green : Colors.red,
                fontWeight: FontWeight.bold,
              ),
            ),
          ),
        ],
      ),
      subtitle: Text(
        'Output • ${_timeFormat.format(timestamp)}',
        style: Theme.of(context).textTheme.bodySmall?.copyWith(fontSize: 11),
      ),
      trailing: Text(
        '${event.keyCode}',
        style: Theme.of(context).textTheme.bodySmall?.copyWith(
              fontFamily: 'monospace',
              fontSize: 11,
            ),
      ),
    );
  }

  Widget _buildStateChangeEventTile(
    StateChangeEvent event,
    DateTime timestamp,
  ) {
    return ListTile(
      dense: true,
      visualDensity: VisualDensity.compact,
      leading: CircleAvatar(
        radius: 12,
        backgroundColor: Colors.purple.withValues(alpha: 0.2),
        child: const Icon(
          Icons.info_outline,
          color: Colors.purple,
          size: 14,
        ),
      ),
      title: Text(
        event.message,
        style: const TextStyle(fontWeight: FontWeight.w500, fontSize: 13),
      ),
      subtitle: Text(
        '${event.type} • ${_timeFormat.format(timestamp)}',
        style: Theme.of(context).textTheme.bodySmall?.copyWith(fontSize: 11),
      ),
    );
  }

  void _showFilterDialog() {
    showDialog<void>(
      context: context,
      builder: (context) {
        return StatefulBuilder(
          builder: (context, setDialogState) {
            return AlertDialog(
              title: const Text('Filter Events'),
              content: Column(
                mainAxisSize: MainAxisSize.min,
                children: [
                  CheckboxListTile(
                    title: const Text('Input Events'),
                    subtitle: const Text('Key press/release from devices'),
                    value: _showInput,
                    onChanged: (value) {
                      setDialogState(() => _showInput = value ?? true);
                      setState(() => _showInput = value ?? true);
                    },
                  ),
                  CheckboxListTile(
                    title: const Text('Output Events'),
                    subtitle: const Text('Virtual device output'),
                    value: _showOutput,
                    onChanged: (value) {
                      setDialogState(() => _showOutput = value ?? true);
                      setState(() => _showOutput = value ?? true);
                    },
                  ),
                  CheckboxListTile(
                    title: const Text('State Changes'),
                    subtitle: const Text('Engine state changes'),
                    value: _showStateChange,
                    onChanged: (value) {
                      setDialogState(() => _showStateChange = value ?? true);
                      setState(() => _showStateChange = value ?? true);
                    },
                  ),
                ],
              ),
              actions: [
                TextButton(
                  onPressed: () => Navigator.of(context).pop(),
                  child: const Text('Close'),
                ),
              ],
            );
          },
        );
      },
    );
  }
}

/// Internal event display wrapper.
class _EventDisplay {
  const _EventDisplay({
    required this.event,
    required this.timestamp,
  });

  final DaemonEvent event;
  final DateTime timestamp;
}