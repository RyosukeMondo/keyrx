/// Device profile viewer page showing row-column layout and keymap.
///
/// Displays the physical keyboard layout information discovered during
/// the device discovery process.
library;

import 'package:flutter/material.dart';

import '../ffi/bridge.dart';
import '../services/service_registry.dart';

/// Page for viewing device profile information.
class DeviceProfilePage extends StatefulWidget {
  const DeviceProfilePage({
    super.key,
    required this.vendorId,
    required this.productId,
    required this.deviceName,
    required this.services,
  });

  final int vendorId;
  final int productId;
  final String deviceName;
  final ServiceRegistry services;

  @override
  State<DeviceProfilePage> createState() => _DeviceProfilePageState();
}

class _DeviceProfilePageState extends State<DeviceProfilePage> {
  DeviceProfile? _profile;
  bool _isLoading = true;
  String? _error;

  @override
  void initState() {
    super.initState();
    _loadProfile();
  }

  Future<void> _loadProfile() async {
    setState(() {
      _isLoading = true;
      _error = null;
    });

    final result = await widget.services.deviceProfileService
        .getProfile(widget.vendorId, widget.productId);

    if (!mounted) return;

    setState(() {
      _isLoading = false;
      if (result.isSuccess) {
        _profile = result.profile;
      } else {
        _error = result.errorMessage ?? 'Unknown error';
      }
    });
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        title: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Text(widget.deviceName),
            Text(
              '${widget.vendorId.toRadixString(16).padLeft(4, '0')}:'
              '${widget.productId.toRadixString(16).padLeft(4, '0')}',
              style: Theme.of(context).textTheme.bodySmall,
            ),
          ],
        ),
      ),
      body: _buildBody(),
    );
  }

  Widget _buildBody() {
    if (_isLoading) {
      return const Center(child: CircularProgressIndicator());
    }

    if (_error != null) {
      return _buildErrorState();
    }

    if (_profile == null) {
      return const Center(child: Text('No profile data'));
    }

    return _buildProfileView();
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
              'Error loading profile',
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
              onPressed: _loadProfile,
              icon: const Icon(Icons.refresh),
              label: const Text('Retry'),
            ),
          ],
        ),
      ),
    );
  }

  Widget _buildProfileView() {
    final profile = _profile!;

    return ListView(
      padding: const EdgeInsets.all(16),
      children: [
        _buildInfoCard(profile),
        const SizedBox(height: 16),
        _buildLayoutCard(profile),
        const SizedBox(height: 16),
        _buildKeymapCard(profile),
        if (profile.aliases.isNotEmpty) ...[
          const SizedBox(height: 16),
          _buildAliasesCard(profile),
        ],
      ],
    );
  }

  Widget _buildInfoCard(DeviceProfile profile) {
    final discoveredDate = profile.discoveredAt.toLocal();
    final dateStr =
        '${discoveredDate.year}-${discoveredDate.month.toString().padLeft(2, '0')}-${discoveredDate.day.toString().padLeft(2, '0')} '
        '${discoveredDate.hour.toString().padLeft(2, '0')}:${discoveredDate.minute.toString().padLeft(2, '0')}';

    return Card(
      child: Padding(
        padding: const EdgeInsets.all(16),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Row(
              children: [
                const Icon(Icons.info_outline),
                const SizedBox(width: 8),
                Text(
                  'Profile Information',
                  style: Theme.of(context).textTheme.titleMedium,
                ),
              ],
            ),
            const SizedBox(height: 16),
            _buildInfoRow('Device Name', profile.name ?? 'Unknown'),
            _buildInfoRow('Device ID', profile.deviceId),
            _buildInfoRow('Source', profile.source.label),
            _buildInfoRow('Discovered', dateStr),
            _buildInfoRow('Schema Version', '${profile.schemaVersion}'),
            _buildInfoRow('Total Keys', '${profile.totalKeys}'),
          ],
        ),
      ),
    );
  }

  Widget _buildLayoutCard(DeviceProfile profile) {
    return Card(
      child: Padding(
        padding: const EdgeInsets.all(16),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Row(
              children: [
                const Icon(Icons.grid_4x4),
                const SizedBox(width: 8),
                Text(
                  'Physical Layout',
                  style: Theme.of(context).textTheme.titleMedium,
                ),
              ],
            ),
            const SizedBox(height: 16),
            _buildInfoRow('Rows', '${profile.rows}'),
            const SizedBox(height: 12),
            Text(
              'Columns per Row:',
              style: Theme.of(context).textTheme.bodyMedium?.copyWith(
                    fontWeight: FontWeight.w500,
                  ),
            ),
            const SizedBox(height: 8),
            ...List.generate(profile.colsPerRow.length, (index) {
              return Padding(
                padding: const EdgeInsets.symmetric(vertical: 4),
                child: Row(
                  children: [
                    Container(
                      width: 32,
                      height: 32,
                      decoration: BoxDecoration(
                        color: Theme.of(context).colorScheme.primaryContainer,
                        borderRadius: BorderRadius.circular(4),
                      ),
                      child: Center(
                        child: Text(
                          'R$index',
                          style: TextStyle(
                            fontSize: 12,
                            fontWeight: FontWeight.bold,
                            color: Theme.of(context)
                                .colorScheme
                                .onPrimaryContainer,
                          ),
                        ),
                      ),
                    ),
                    const SizedBox(width: 12),
                    Text('${profile.colsPerRow[index]} columns'),
                  ],
                ),
              );
            }),
          ],
        ),
      ),
    );
  }

  Widget _buildKeymapCard(DeviceProfile profile) {
    final sortedEntries = profile.keymap.entries.toList()
      ..sort((a, b) {
        // Sort by row, then column
        final rowCompare = a.value.row.compareTo(b.value.row);
        if (rowCompare != 0) return rowCompare;
        return a.value.col.compareTo(b.value.col);
      });

    return Card(
      child: Padding(
        padding: const EdgeInsets.all(16),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Row(
              children: [
                const Icon(Icons.keyboard),
                const SizedBox(width: 8),
                Text(
                  'Key Mapping',
                  style: Theme.of(context).textTheme.titleMedium,
                ),
              ],
            ),
            const SizedBox(height: 8),
            Text(
              '${profile.keymap.length} keys mapped',
              style: Theme.of(context).textTheme.bodySmall,
            ),
            const SizedBox(height: 16),
            ...sortedEntries.map((entry) {
              final scanCode = entry.key;
              final physicalKey = entry.value;
              return _buildKeyMapRow(scanCode, physicalKey);
            }),
          ],
        ),
      ),
    );
  }

  Widget _buildKeyMapRow(int scanCode, PhysicalKey key) {
    return Padding(
      padding: const EdgeInsets.symmetric(vertical: 6),
      child: Row(
        children: [
          Container(
            padding: const EdgeInsets.symmetric(horizontal: 10, vertical: 6),
            decoration: BoxDecoration(
              color: Theme.of(context).colorScheme.surfaceContainerHighest,
              borderRadius: BorderRadius.circular(4),
            ),
            child: Text(
              'R${key.row}C${key.col}',
              style: const TextStyle(
                fontFamily: 'monospace',
                fontWeight: FontWeight.bold,
              ),
            ),
          ),
          const SizedBox(width: 12),
          const Icon(Icons.arrow_forward, size: 16),
          const SizedBox(width: 12),
          Container(
            padding: const EdgeInsets.symmetric(horizontal: 10, vertical: 6),
            decoration: BoxDecoration(
              color: Theme.of(context).colorScheme.primaryContainer,
              borderRadius: BorderRadius.circular(4),
            ),
            child: Text(
              'Scan: 0x${scanCode.toRadixString(16).padLeft(2, '0').toUpperCase()}',
              style: TextStyle(
                fontFamily: 'monospace',
                color: Theme.of(context).colorScheme.onPrimaryContainer,
              ),
            ),
          ),
          if (key.alias != null) ...[
            const SizedBox(width: 12),
            Expanded(
              child: Text(
                key.alias!,
                style: Theme.of(context).textTheme.bodyMedium?.copyWith(
                      fontStyle: FontStyle.italic,
                    ),
                overflow: TextOverflow.ellipsis,
              ),
            ),
          ],
        ],
      ),
    );
  }

  Widget _buildAliasesCard(DeviceProfile profile) {
    return Card(
      child: Padding(
        padding: const EdgeInsets.all(16),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Row(
              children: [
                const Icon(Icons.label),
                const SizedBox(width: 8),
                Text(
                  'Key Aliases',
                  style: Theme.of(context).textTheme.titleMedium,
                ),
              ],
            ),
            const SizedBox(height: 16),
            ...profile.aliases.entries.map((entry) {
              return Padding(
                padding: const EdgeInsets.symmetric(vertical: 4),
                child: Row(
                  children: [
                    Expanded(
                      child: Text(
                        entry.key,
                        style: Theme.of(context).textTheme.bodyMedium,
                      ),
                    ),
                    const Icon(Icons.arrow_forward, size: 16),
                    const SizedBox(width: 8),
                    Text(
                      'Scan: 0x${entry.value.toRadixString(16).padLeft(2, '0').toUpperCase()}',
                      style: const TextStyle(fontFamily: 'monospace'),
                    ),
                  ],
                ),
              );
            }),
          ],
        ),
      ),
    );
  }

  Widget _buildInfoRow(String label, String value) {
    return Padding(
      padding: const EdgeInsets.symmetric(vertical: 4),
      child: Row(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          SizedBox(
            width: 140,
            child: Text(
              label,
              style: Theme.of(context).textTheme.bodyMedium?.copyWith(
                    fontWeight: FontWeight.w500,
                  ),
            ),
          ),
          Expanded(
            child: Text(
              value,
              style: Theme.of(context).textTheme.bodyMedium,
            ),
          ),
        ],
      ),
    );
  }
}
