/// Central dashboard for daemon management and configuration.
///
/// Provides controls for the background daemon process, boot configuration,
/// and access to the passive monitor.
library;

import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import '../services/daemon_service.dart';
import '../state/daemon_providers.dart';
import 'monitor_view.dart';

class RunControlsPage extends ConsumerStatefulWidget {
  const RunControlsPage({super.key});

  @override
  ConsumerState<RunControlsPage> createState() => _RunControlsPageState();
}

class _RunControlsPageState extends ConsumerState<RunControlsPage> {

  @override

  Widget build(BuildContext context) {
    final connectionState = ref.watch(daemonConnectionStateProvider);
    final isConnected =
        connectionState.value == DaemonConnectionState.connected;
    final statusAsync = ref.watch(daemonStatusProvider);

    final isEngineRunning = statusAsync.when(
      data: (status) => status.status == 'running',
      loading: () => false,
      error: (err, stack) => false,
    );

    return DefaultTabController(
      length: 2,
      child: Scaffold(
        appBar: AppBar(
          title: const Text('Dashboard'),
          bottom: const TabBar(
            tabs: [
              Tab(text: 'Controls', icon: Icon(Icons.tune)),
              Tab(text: 'Monitor', icon: Icon(Icons.monitor_heart)),
            ],
          ),
          actions: [
            // Status Chip
            Padding(
              padding: const EdgeInsets.only(right: 16),
              child: Chip(
                avatar: Icon(
                  isEngineRunning ? Icons.check_circle : Icons.error_outline,
                  color: isEngineRunning ? Colors.green : Colors.red,
                  size: 16,
                ),
                label: Text(
                  isEngineRunning ? 'Engine Running' : 'Engine Stopped',
                  style: TextStyle(
                    color: isEngineRunning ? Colors.green : Colors.red,
                    fontSize: 12,
                  ),
                ),
                backgroundColor: isEngineRunning
                    ? Colors.green.withValues(alpha: 0.1)
                    : Colors.red.withValues(alpha: 0.1),
                side: BorderSide.none,
              ),
            ),
          ],
        ),
        body: TabBarView(
          children: [
            // Tab 1: Controls
            SingleChildScrollView(
              padding: const EdgeInsets.all(24),
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.stretch,
                children: [
                  _buildDaemonControlCard(isConnected, ref),
                  const SizedBox(height: 24),
                  if (isConnected) _buildConnectedDevicesList(ref),
                ],
              ),
            ),

            // Tab 2: Monitor (Passive)
            const MonitorView(),
          ],
        ),
      ),
    );
  }

  Widget _buildDaemonControlCard(bool isConnected, WidgetRef ref) => Card(
    child: Padding(
      padding: const EdgeInsets.all(20),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Row(
            children: [
              Icon(Icons.dns, color: Theme.of(context).colorScheme.primary),
              const SizedBox(width: 12),
              Text(
                'Engine Control',
                style: Theme.of(context).textTheme.titleLarge,
              ),
            ],
          ),
          const SizedBox(height: 16),
          const Text(
            'Start or stop the remapping engine. The daemon must be running.',
            style: TextStyle(color: Colors.grey),
          ),
          const SizedBox(height: 24),
          Row(
            children: [
              Expanded(
                child: FilledButton.icon(
                  onPressed: isConnected ? () {
                    ref.read(daemonServiceProvider).startEngine();
                  } : null,
                  icon: const Icon(Icons.play_arrow),
                  label: const Text('Start Engine'),
                  style: FilledButton.styleFrom(
                    backgroundColor: Colors.green,
                    foregroundColor: Colors.white,
                    padding: const EdgeInsets.symmetric(vertical: 16),
                  ),
                ),
              ),
              const SizedBox(width: 16),
              Expanded(
                child: OutlinedButton.icon(
                  onPressed: isConnected ? () {
                    ref.read(daemonServiceProvider).stopEngine();
                  } : null,
                  icon: const Icon(Icons.stop),
                  label: const Text('Stop Engine'),
                  style: OutlinedButton.styleFrom(
                    foregroundColor: Colors.red,
                    padding: const EdgeInsets.symmetric(vertical: 16),
                  ),
                ),
              ),
            ],
          ),
        ],
      ),
    ),
  );



  Widget _buildConnectedDevicesList(WidgetRef ref) {
    final statusAsync = ref.watch(daemonStatusProvider);

    return Card(
      child: Padding(
        padding: const EdgeInsets.all(20),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Row(
              children: [
                const Icon(Icons.keyboard),
                const SizedBox(width: 12),
                Text(
                  'Managed Devices',
                  style: Theme.of(context).textTheme.titleLarge,
                ),
              ],
            ),
            const SizedBox(height: 16),
            statusAsync.when(
              data: (status) {
                if (status.devices.isEmpty) {
                  return const Padding(
                    padding: EdgeInsets.symmetric(vertical: 8),
                    child: Text('No devices detected.'),
                  );
                }
                return Column(
                  children: status.devices
                      .map(
                        (device) => ListTile(
                          contentPadding: EdgeInsets.zero,
                          leading: const Icon(Icons.keyboard_alt_outlined),
                          title: Text(device.name),
                          subtitle: Text(device.path),
                        ),
                      )
                      .toList(),
                );
              },
              loading: () => const Center(child: CircularProgressIndicator()),
              error: (err, stack) => Center(child: Text('Error: $err')),
            ),
          ],
        ),
      ),
    );
  }
}
