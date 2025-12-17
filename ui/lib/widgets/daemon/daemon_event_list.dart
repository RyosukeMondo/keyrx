// ignore_for_file: prefer_expression_function_bodies

import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
<<<<<<< HEAD
import 'package:intl/intl.dart';
import '../../models/daemon_protocol.dart';
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
=======
import 'package:keyrx_ui/models/daemon_protocol.dart';
import 'package:keyrx_ui/state/daemon_providers.dart';

/// Displays a scrolling list of daemon events.
///
/// Intended to be implemented by Jules Task C.
class DaemonEventList extends ConsumerStatefulWidget {
  const DaemonEventList({super.key});

  @override
  ConsumerState<DaemonEventList> createState() => _DaemonEventListState();
}

class _DaemonEventListState extends ConsumerState<DaemonEventList> {
  final List<DaemonEvent> _events = [];

  @override
  Widget build(BuildContext context) {
    ref.listen<AsyncValue<DaemonEvent>>(daemonEventsProvider, (_, asyncValue) {
      if (asyncValue is AsyncData<DaemonEvent>) {
        setState(() {
          _events.insert(0, asyncValue.value);
>>>>>>> 15e8196d (feat(ui): implement DaemonEventList widget)
          if (_events.length > 100) {
            _events.removeLast();
          }
        });
<<<<<<< HEAD
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
=======
      }
    });

    if (_events.isEmpty) {
      return const Center(child: Text('Awaiting daemon events...'));
    }

    return ListView.builder(
      itemCount: _events.length,
      itemBuilder: (context, index) {
        final event = _events[index];
        return event.when(
          input: (inputEvent) => ListTile(
            leading: const Icon(Icons.keyboard_arrow_down),
            title: Text('Input: ${inputEvent.keyName}'),
            subtitle: Text(
              '${inputEvent.eventType} on ${inputEvent.deviceName}',
            ),
          ),
          output: (outputEvent) => ListTile(
            leading: const Icon(Icons.keyboard_arrow_up),
            title: Text('Output: ${outputEvent.keyName}'),
            subtitle: Text('${outputEvent.actionType}'),
          ),
          stateChange: (stateChangeEvent) => ListTile(
            leading: const Icon(Icons.info_outline),
            title: Text('State Change: ${stateChangeEvent.type}'),
            subtitle: Text(stateChangeEvent.message),
          ),
        );
      },
>>>>>>> 15e8196d (feat(ui): implement DaemonEventList widget)
    );
  }
}
