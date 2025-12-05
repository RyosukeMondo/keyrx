/// Device management page for revolutionary mapping.
///
/// Displays all connected devices with their remap status, assigned profiles,
/// and provides controls for managing device configuration.
library;

import 'package:flutter/material.dart';
import '../models/device_state.dart';
import '../services/device_registry_service.dart';
import '../services/profile_registry_service.dart';
import '../widgets/device_card.dart';

/// Page for managing connected devices with revolutionary mapping.
///
/// Displays a list of all registered devices, allowing users to:
/// - Toggle remap enabled/disabled per device
/// - Assign profiles to devices
/// - Set user labels for easier identification
/// - Refresh the device list
class DevicesPage extends StatefulWidget {
  const DevicesPage({
    super.key,
    required this.deviceService,
    required this.profileService,
  });

  final DeviceRegistryService deviceService;
  final ProfileRegistryService profileService;

  @override
  State<DevicesPage> createState() => _DevicesPageState();
}

class _DevicesPageState extends State<DevicesPage> {
  // Key used to trigger FutureBuilder refresh
  int _refreshKey = 0;

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        title: const Text('Devices'),
        actions: [
          IconButton(
            icon: const Icon(Icons.refresh),
            onPressed: _refreshDevices,
            tooltip: 'Refresh device list',
          ),
        ],
      ),
      body: RefreshIndicator(
        onRefresh: () async {
          _refreshDevices();
        },
        child: FutureBuilder<List<DeviceState>>(
          key: ValueKey(_refreshKey),
          future: widget.deviceService.getDevices(),
          builder: (context, snapshot) {
            if (snapshot.connectionState == ConnectionState.waiting) {
              return const Center(child: CircularProgressIndicator());
            }

            if (snapshot.hasError) {
              return _buildErrorState(snapshot.error.toString());
            }

            final devices = snapshot.data ?? [];

            if (devices.isEmpty) {
              return _buildEmptyState();
            }

            return _buildDeviceList(devices);
          },
        ),
      ),
    );
  }

  /// Refresh the device list by incrementing the key
  void _refreshDevices() {
    setState(() {
      _refreshKey++;
    });
    widget.deviceService.refresh();
  }

  /// Build error state UI
  Widget _buildErrorState(String error) {
    return Center(
      child: Padding(
        padding: const EdgeInsets.all(24),
        child: Column(
          mainAxisSize: MainAxisSize.min,
          children: [
            const Icon(Icons.error_outline, size: 64, color: Colors.red),
            const SizedBox(height: 16),
            Text(
              'Error loading devices',
              style: Theme.of(context).textTheme.titleLarge,
            ),
            const SizedBox(height: 8),
            Text(
              error,
              textAlign: TextAlign.center,
              style: Theme.of(context).textTheme.bodyMedium,
            ),
            const SizedBox(height: 24),
            FilledButton.icon(
              onPressed: _refreshDevices,
              icon: const Icon(Icons.refresh),
              label: const Text('Retry'),
            ),
          ],
        ),
      ),
    );
  }

  /// Build empty state UI
  Widget _buildEmptyState() {
    return ListView(
      padding: const EdgeInsets.all(24),
      children: [
        const SizedBox(height: 48),
        const Icon(Icons.keyboard, size: 64, color: Colors.grey),
        const SizedBox(height: 16),
        Text(
          'No devices found',
          textAlign: TextAlign.center,
          style: Theme.of(context).textTheme.titleLarge,
        ),
        const SizedBox(height: 8),
        Text(
          'Connect a keyboard or other input device to get started.',
          textAlign: TextAlign.center,
          style: Theme.of(context).textTheme.bodyMedium,
        ),
        const SizedBox(height: 32),
        _buildTroubleshootingCard(),
      ],
    );
  }

  /// Build troubleshooting card for empty state
  Widget _buildTroubleshootingCard() {
    return Card(
      child: Padding(
        padding: const EdgeInsets.all(16),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Row(
              children: [
                const Icon(Icons.help_outline, size: 20),
                const SizedBox(width: 8),
                Text(
                  'Troubleshooting',
                  style: Theme.of(context).textTheme.titleMedium,
                ),
              ],
            ),
            const SizedBox(height: 12),
            const _TroubleshootingStep(
              number: '1',
              text: 'Check that your device is connected via USB',
            ),
            const _TroubleshootingStep(
              number: '2',
              text: 'Run "keyrx doctor" to diagnose permission issues',
            ),
            const _TroubleshootingStep(
              number: '3',
              text: 'Ensure your user is in the "input" group (Linux)',
            ),
            const _TroubleshootingStep(
              number: '4',
              text: 'Try running with elevated privileges if needed',
            ),
          ],
        ),
      ),
    );
  }

  /// Build list of device cards
  Widget _buildDeviceList(List<DeviceState> devices) {
    return ListView.builder(
      padding: const EdgeInsets.symmetric(vertical: 8),
      itemCount: devices.length,
      itemBuilder: (context, index) {
        final device = devices[index];
        return DeviceCard(
          deviceState: device,
          deviceService: widget.deviceService,
          profileService: widget.profileService,
          onEditLabel: () => _showEditLabelDialog(device),
          onManageProfiles: () => _showManageProfilesDialog(device),
        );
      },
    );
  }

  /// Show dialog to edit device label
  Future<void> _showEditLabelDialog(DeviceState device) async {
    final controller = TextEditingController(
      text: device.identity.userLabel ?? '',
    );

    final result = await showDialog<String>(
      context: context,
      builder: (context) => AlertDialog(
        title: const Text('Edit Device Label'),
        content: TextField(
          controller: controller,
          autofocus: true,
          decoration: const InputDecoration(
            labelText: 'Label',
            hintText: 'e.g., "Main Keyboard", "Gaming Keypad"',
            border: OutlineInputBorder(),
          ),
          onSubmitted: (value) => Navigator.of(context).pop(value),
        ),
        actions: [
          TextButton(
            onPressed: () => Navigator.of(context).pop(),
            child: const Text('Cancel'),
          ),
          TextButton(
            onPressed: () => Navigator.of(context).pop(null),
            child: const Text('Clear'),
          ),
          FilledButton(
            onPressed: () => Navigator.of(context).pop(controller.text),
            child: const Text('Save'),
          ),
        ],
      ),
    );

    if (result != null) {
      // User either submitted or pressed Save
      final label = result.isEmpty ? null : result;
      final opResult = await widget.deviceService.setUserLabel(
        device.identity.toKey(),
        label,
      );

      if (!mounted) return;

      if (opResult.success) {
        ScaffoldMessenger.of(context).showSnackBar(
          SnackBar(
            content: Text(
              label == null ? 'Label cleared' : 'Label updated to "$label"',
            ),
            backgroundColor: Colors.green,
          ),
        );
        _refreshDevices();
      } else {
        ScaffoldMessenger.of(context).showSnackBar(
          SnackBar(
            content: Text(
              'Failed to update label: ${opResult.errorMessage}',
            ),
            backgroundColor: Colors.red,
          ),
        );
      }
    }
  }

  /// Show dialog for managing profiles (placeholder for now)
  Future<void> _showManageProfilesDialog(DeviceState device) async {
    await showDialog(
      context: context,
      builder: (context) => AlertDialog(
        title: const Text('Manage Profiles'),
        content: Text(
          'Profile management for ${device.identity.displayName} will be available soon.\n\n'
          'For now, use the profile selector in the device card to assign profiles.',
        ),
        actions: [
          FilledButton(
            onPressed: () => Navigator.of(context).pop(),
            child: const Text('OK'),
          ),
        ],
      ),
    );
  }
}

/// Troubleshooting step widget
class _TroubleshootingStep extends StatelessWidget {
  const _TroubleshootingStep({
    required this.number,
    required this.text,
  });

  final String number;
  final String text;

  @override
  Widget build(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.symmetric(vertical: 4),
      child: Row(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Container(
            width: 20,
            height: 20,
            decoration: BoxDecoration(
              color: Theme.of(context).colorScheme.primaryContainer,
              shape: BoxShape.circle,
            ),
            child: Center(
              child: Text(
                number,
                style: TextStyle(
                  fontSize: 12,
                  fontWeight: FontWeight.bold,
                  color: Theme.of(context).colorScheme.onPrimaryContainer,
                ),
              ),
            ),
          ),
          const SizedBox(width: 12),
          Expanded(child: Text(text)),
        ],
      ),
    );
  }
}
