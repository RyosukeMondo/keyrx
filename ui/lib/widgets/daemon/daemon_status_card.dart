import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import '../../services/daemon_service.dart';
import '../../state/daemon_providers.dart';

/// Displays the current status of the daemon.
class DaemonStatusCard extends ConsumerWidget {
  const DaemonStatusCard({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final connectionState = ref.watch(daemonConnectionStateProvider);

    return Card(
      child: Padding(
        padding: const EdgeInsets.all(16),
        child: Row(
          children: [
            connectionState.when(
              data: (state) => Icon(
                state == DaemonConnectionState.connected
                    ? Icons.check_circle
                    : Icons.error_outline,
                color: state == DaemonConnectionState.connected
                    ? Colors.green
                    : Colors.red,
              ),
              loading: () => const SizedBox(
                width: 24,
                height: 24,
                child: CircularProgressIndicator(strokeWidth: 2),
              ),
              error: (_, __) => const Icon(Icons.error, color: Colors.red),
            ),
            const SizedBox(width: 16),
            Expanded(
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  const Text(
                    'Daemon Status',
                    style: TextStyle(fontWeight: FontWeight.bold),
                  ),
                  Text(
                    connectionState.value?.name.toUpperCase() ?? 'UNKNOWN',
                    style: Theme.of(context).textTheme.bodySmall,
                  ),
                ],
              ),
            ),
            if (connectionState.value != DaemonConnectionState.connected)
              OutlinedButton(
                onPressed: () {
                  ref.read(daemonServiceProvider).connect();
                },
                child: const Text('Reconnect'),
              ),
          ],
        ),
      ),
    );
  }
}
