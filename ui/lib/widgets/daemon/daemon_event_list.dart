// ignore_for_file: prefer_expression_function_bodies

import 'package:core_logic/models/daemon_protocol.dart';
import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:intl/intl.dart';

import '../../state/daemon_providers.dart';

/// Displays a scrolling list of daemon events.
class DaemonEventList extends ConsumerWidget {
  const DaemonEventList({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final eventsAsync = ref.watch(daemonEventsProvider);

    return eventsAsync.when(
      data: (event) => const _EventListHandler(),
      loading: () => const Center(child: CircularProgressIndicator()),
      error: (err, stack) => Center(child: Text('Error: $err')),
    );
  }
}

class _EventListHandler extends ConsumerStatefulWidget {
  const _EventListHandler();

  @override
  ConsumerState<_EventListHandler> createState() => _EventListHandlerState();
}

class _EventListHandlerState extends ConsumerState<_EventListHandler> {
  final List<DaemonEvent> _events = [];
  final ScrollController _scrollController = ScrollController();

  @override
  Widget build(BuildContext context) {
    ref.listen(daemonEventsProvider, (previous, next) {
      next.whenData((event) {
        setState(() {
          _events.insert(0, event);
          if (_events.length > 100) {
            _events.removeLast();
          }
        });
      });
    });

    if (_events.isEmpty) {
      return const Center(child: Text('No events yet'));
    }

    return ListView.separated(
      controller: _scrollController,
      itemCount: _events.length,
      separatorBuilder: (_, __) => const Divider(height: 1),
      itemBuilder: (context, index) => _buildEventTile(context, _events[index]),
    );
  }

  Widget _buildEventTile(BuildContext context, DaemonEvent event) {
    return event.when(
      input: (e) => ListTile(
        leading: const Icon(Icons.keyboard),
        title: Text('${e.keyName} (${e.eventType})'),
        subtitle: Text(e.deviceName),
        trailing: Text(
          DateFormat(
            'HH:mm:ss.SSS',
          ).format(DateTime.fromMillisecondsSinceEpoch(e.timestampUs ~/ 1000)),
        ),
      ),
      output: (e) => ListTile(
        leading: const Icon(Icons.output),
        title: Text('${e.keyName} (${e.actionType})'),
        trailing: Text(
          DateFormat(
            'HH:mm:ss.SSS',
          ).format(DateTime.fromMillisecondsSinceEpoch(e.timestampUs ~/ 1000)),
        ),
      ),
      decision: (e) => ListTile(
        leading: const Icon(Icons.psychology),
        title: Text('${e.keyName} -> ${e.decisionType}'),
        subtitle: Text('Outputs: ${e.outputs.join(', ')}'),
        trailing: Text(
          DateFormat(
            'HH:mm:ss.SSS',
          ).format(DateTime.fromMillisecondsSinceEpoch(e.timestampUs ~/ 1000)),
        ),
      ),
      discovery: (e) => ListTile(
        leading: const Icon(Icons.search),
        title: Text('Discovered: ${e.name}'),
        subtitle: Text('ID: ${e.deviceId}'),
      ),
      stateChange: (e) => ListTile(
        leading: const Icon(Icons.info),
        title: Text(e.message),
        subtitle: Text(e.type),
        trailing: Text(
          DateFormat(
            'HH:mm:ss.SSS',
          ).format(DateTime.fromMillisecondsSinceEpoch(e.timestampUs ~/ 1000)),
        ),
      ),
    );
  }
}
