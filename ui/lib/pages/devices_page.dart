/// Device selection page for listing and selecting keyboard devices.
///
/// Shows available devices with name, path, and profile status.
/// Supports selection, refresh, and troubleshooting guidance.
library;

import 'package:flutter/material.dart';

import '../ffi/bridge.dart';
import '../services/device_service.dart';

/// Page for listing and selecting keyboard devices.
class DevicesPage extends StatefulWidget {
  const DevicesPage({
    super.key,
    required this.deviceService,
  });

  final DeviceService deviceService;

  @override
  State<DevicesPage> createState() => _DevicesPageState();
}

class _DevicesPageState extends State<DevicesPage> {
  List<KeyboardDevice> _devices = [];
  String? _selectedPath;
  bool _isLoading = true;
  String? _error;

  @override
  void initState() {
    super.initState();
    _loadDevices();
  }

  Future<void> _loadDevices() async {
    setState(() {
      _isLoading = true;
      _error = null;
    });

    try {
      final devices = await widget.deviceService.listDevices();
      setState(() {
        _devices = devices;
        _isLoading = false;
      });
    } catch (e) {
      setState(() {
        _error = e.toString();
        _isLoading = false;
      });
    }
  }

  Future<void> _refreshDevices() async {
    setState(() {
      _error = null;
    });

    try {
      final devices = await widget.deviceService.refresh();
      setState(() {
        _devices = devices;
      });
    } catch (e) {
      setState(() {
        _error = e.toString();
      });
    }
  }

  Future<void> _selectDevice(KeyboardDevice device) async {
    final result = await widget.deviceService.selectDevice(device.path);

    if (!mounted) return;

    if (result.success) {
      setState(() {
        _selectedPath = device.path;
      });
      ScaffoldMessenger.of(context).showSnackBar(
        SnackBar(
          content: Text('Selected: ${device.name}'),
          backgroundColor: Colors.green,
        ),
      );
    } else {
      ScaffoldMessenger.of(context).showSnackBar(
        SnackBar(
          content: Text('Failed to select device: ${result.errorMessage}'),
          backgroundColor: Colors.red,
        ),
      );
    }
  }

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
        onRefresh: _refreshDevices,
        child: _buildBody(),
      ),
    );
  }

  Widget _buildBody() {
    if (_isLoading) {
      return const Center(child: CircularProgressIndicator());
    }

    if (_error != null) {
      return _buildErrorState();
    }

    if (_devices.isEmpty) {
      return _buildEmptyState();
    }

    return _buildDeviceList();
  }

  Widget _buildErrorState() {
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
              _error ?? 'Unknown error',
              textAlign: TextAlign.center,
              style: Theme.of(context).textTheme.bodyMedium,
            ),
            const SizedBox(height: 24),
            FilledButton.icon(
              onPressed: _loadDevices,
              icon: const Icon(Icons.refresh),
              label: const Text('Retry'),
            ),
          ],
        ),
      ),
    );
  }

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
          'Make sure your keyboard is connected and you have the necessary permissions.',
          textAlign: TextAlign.center,
          style: Theme.of(context).textTheme.bodyMedium,
        ),
        const SizedBox(height: 32),
        _buildTroubleshootingCard(),
      ],
    );
  }

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
              text: 'Check that your keyboard is connected via USB',
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

  Widget _buildDeviceList() {
    return ListView.builder(
      padding: const EdgeInsets.symmetric(vertical: 8),
      itemCount: _devices.length,
      itemBuilder: (context, index) {
        final device = _devices[index];
        final isSelected = device.path == _selectedPath;

        return _DeviceListTile(
          device: device,
          isSelected: isSelected,
          onTap: () => _selectDevice(device),
        );
      },
    );
  }
}

class _DeviceListTile extends StatelessWidget {
  const _DeviceListTile({
    required this.device,
    required this.isSelected,
    required this.onTap,
  });

  final KeyboardDevice device;
  final bool isSelected;
  final VoidCallback onTap;

  @override
  Widget build(BuildContext context) {
    final vendorId = device.vendorId.toRadixString(16).padLeft(4, '0');
    final productId = device.productId.toRadixString(16).padLeft(4, '0');

    return ListTile(
      leading: Icon(
        Icons.keyboard,
        color: isSelected ? Theme.of(context).colorScheme.primary : null,
      ),
      title: Text(
        device.name,
        style: TextStyle(
          fontWeight: isSelected ? FontWeight.bold : FontWeight.normal,
        ),
      ),
      subtitle: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Text('$vendorId:$productId'),
          Text(
            device.path,
            style: Theme.of(context).textTheme.bodySmall?.copyWith(
                  color: Theme.of(context).colorScheme.outline,
                  fontFamily: 'monospace',
                ),
          ),
        ],
      ),
      trailing: Row(
        mainAxisSize: MainAxisSize.min,
        children: [
          if (device.hasProfile)
            Tooltip(
              message: 'Profile available',
              child: Container(
                padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 4),
                decoration: BoxDecoration(
                  color: Colors.green.withValues(alpha: 0.15),
                  borderRadius: BorderRadius.circular(12),
                ),
                child: const Row(
                  mainAxisSize: MainAxisSize.min,
                  children: [
                    Icon(Icons.check_circle, size: 16, color: Colors.green),
                    SizedBox(width: 4),
                    Text(
                      'Profile',
                      style: TextStyle(fontSize: 12, color: Colors.green),
                    ),
                  ],
                ),
              ),
            ),
          if (isSelected) ...[
            const SizedBox(width: 8),
            const Icon(Icons.check, color: Colors.green),
          ],
        ],
      ),
      selected: isSelected,
      onTap: onTap,
      isThreeLine: true,
    );
  }
}

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
