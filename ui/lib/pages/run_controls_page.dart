/// Central dashboard for daemon management and configuration.
///
/// Provides controls for the background daemon process, boot configuration,
/// and access to the passive monitor.
library;

import 'dart:io';

import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:provider/provider.dart' as provider;

import '../models/device_state.dart';
import '../services/daemon_service.dart';
import '../services/facade/keyrx_facade.dart';
import '../state/daemon_providers.dart';
import 'monitor_view.dart';

class RunControlsPage extends ConsumerStatefulWidget {
  const RunControlsPage({super.key});

  @override
  ConsumerState<RunControlsPage> createState() => _RunControlsPageState();
}

class _RunControlsPageState extends ConsumerState<RunControlsPage> {
  bool _runOnBoot = false; // Todo: Persist this preference
  bool _isStarting = false;

  @override
  void initState() {
    super.initState();
    WidgetsBinding.instance.addPostFrameCallback((_) {
      _initialize();
    });
  }

  Future<void> _initialize() async {
    final daemonService = ref.read(daemonServiceProvider);
    // Attempt to connect if not already connected
    if (daemonService.state == DaemonConnectionState.disconnected) {
      try {
        await daemonService.connect();
      } catch (_) {
        // Ignore, daemon might not be running
      }
    }
  }

  Future<void> _startDaemon() async {
    setState(() => _isStarting = true);
    try {
      // Start the daemon process in the background
      // Note: This assumes 'keyrx' is in the PATH or we should use absolute path
      await Process.start('keyrx', ['daemon'], mode: ProcessStartMode.detached);

      // Wait a bit for it to spin up
      await Future.delayed(const Duration(seconds: 1));

      // Try connecting
      final daemonService = ref.read(daemonServiceProvider);
      await daemonService.connect();

      if (mounted) {
        ScaffoldMessenger.of(context).showSnackBar(
          const SnackBar(
            content: Text('Daemon started'),
            backgroundColor: Colors.green,
          ),
        );
      }
    } catch (e) {
      if (mounted) {
        ScaffoldMessenger.of(context).showSnackBar(
          SnackBar(
            content: Text('Failed to start daemon: $e'),
            backgroundColor: Colors.red,
          ),
        );
      }
    } finally {
      if (mounted) {
        setState(() => _isStarting = false);
      }
    }
  }

  Future<void> _stopDaemon() async {
    try {
      // Try graceful shutdown via service first if available,
      // otherwise use shell command or just stop engine.
      // For now, we'll try to stop the engine via IPC.
      // Real daemon shutdown might require a specific IPC command 'shutdown'
      // or killing the process.

      // Fallback: Kill process by name (Linux specific for now)
      await Process.run('pkill', ['-f', 'keyrx daemon']);

      if (mounted) {
        ScaffoldMessenger.of(
          context,
        ).showSnackBar(const SnackBar(content: Text('Daemon stopped')));
      }
    } catch (e) {
      if (mounted) {
        ScaffoldMessenger.of(context).showSnackBar(
          SnackBar(
            content: Text('Failed to stop daemon: $e'),
            backgroundColor: Colors.red,
          ),
        );
      }
    }
  }

  Future<void> _toggleRunOnBoot(bool value) async {
    setState(() => _runOnBoot = value);
    // Todo: Implement platform-specific service installation
    // Linux: Create systemd unit
    // Mac: Create launchd plist
    ScaffoldMessenger.of(context).showSnackBar(
      const SnackBar(content: Text('Boot configuration updated (Mock)')),
    );
  }

  @override
  Widget build(BuildContext context) {
    final connectionState = ref.watch(daemonConnectionStateProvider);
    final isConnected =
        connectionState.value == DaemonConnectionState.connected;

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
                  isConnected ? Icons.check_circle : Icons.error_outline,
                  color: isConnected ? Colors.green : Colors.red,
                  size: 16,
                ),
                label: Text(
                  isConnected ? 'Daemon Active' : 'Daemon Stopped',
                  style: TextStyle(
                    color: isConnected ? Colors.green : Colors.red,
                    fontSize: 12,
                  ),
                ),
                backgroundColor: isConnected
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
                  _buildDaemonControlCard(isConnected),
                  const SizedBox(height: 24),
                  _buildConfigurationCard(isConnected),
                  const SizedBox(height: 24),
                  if (isConnected) _buildConnectedDevicesList(),
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

  Widget _buildDaemonControlCard(bool isConnected) => Card(
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
                'Daemon Service',
                style: Theme.of(context).textTheme.titleLarge,
              ),
            ],
          ),
          const SizedBox(height: 16),
          const Text(
            'The background service responsible for capturing and remapping input events. '
            'It must be running for remapping to work.',
            style: TextStyle(color: Colors.grey),
          ),
          const SizedBox(height: 24),
          Row(
            children: [
              Expanded(
                child: FilledButton.icon(
                  onPressed: isConnected
                      ? null
                      : (_isStarting ? null : _startDaemon),
                  icon: _isStarting
                      ? const SizedBox(
                          width: 16,
                          height: 16,
                          child: CircularProgressIndicator(strokeWidth: 2),
                        )
                      : const Icon(Icons.play_arrow),
                  label: const Text('Start Daemon'),
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
                  onPressed: isConnected ? _stopDaemon : null,
                  icon: const Icon(Icons.stop),
                  label: const Text('Stop Daemon'),
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

  Widget _buildConfigurationCard(bool isConnected) => Card(
    child: Padding(
      padding: const EdgeInsets.all(20),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Row(
            children: [
              Icon(
                Icons.settings,
                color: Theme.of(context).colorScheme.secondary,
              ),
              const SizedBox(width: 12),
              Text(
                'Configuration',
                style: Theme.of(context).textTheme.titleLarge,
              ),
            ],
          ),
          const SizedBox(height: 16),
          SwitchListTile(
            title: const Text('Remap on Boot'),
            subtitle: const Text(
              'Automatically start the daemon and apply mappings when the system starts.',
            ),
            value: _runOnBoot,
            onChanged: _toggleRunOnBoot,
            contentPadding: EdgeInsets.zero,
          ),
        ],
      ),
    ),
  );

  Widget _buildConnectedDevicesList() {
    // We can use KeyrxFacade for device list or fetch from Daemon
    // For now, let's use the Facade as it maintains the registry
    final facade = provider.Provider.of<KeyrxFacade>(context, listen: false);

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
            StreamBuilder<List<DeviceState>>(
              stream: facade.services.deviceRegistryService.devicesStream,
              builder: (context, snapshot) {
                if (!snapshot.hasData || snapshot.data!.isEmpty) {
                  return const Padding(
                    padding: EdgeInsets.symmetric(vertical: 8),
                    child: Text('No devices detected.'),
                  );
                }

                return Column(
                  children: snapshot.data!
                      .map(
                        (device) => ListTile(
                          contentPadding: EdgeInsets.zero,
                          leading: const Icon(Icons.keyboard_alt_outlined),
                          title: Text(device.identity.displayName),
                          subtitle: Text(device.profileId ?? 'No Profile'),
                          trailing: Container(
                            width: 8,
                            height: 8,
                            decoration: BoxDecoration(
                              color: device.remapEnabled
                                  ? Colors.green
                                  : Colors.grey,
                              shape: BoxShape.circle,
                            ),
                          ),
                        ),
                      )
                      .toList(),
                );
              },
            ),
          ],
        ),
      ),
    );
  }
}
