/// Page for managing multiple profiles for a device.
library;

import 'package:flutter/material.dart';

import '../ffi/bridge.dart';
import '../services/facade/keyrx_facade.dart';
import '../services/service_registry.dart';
import 'device_discovery_page.dart';
import 'device_profile_page.dart';

/// Page for managing device profiles (create, select, delete).
class DeviceProfilesPage extends StatefulWidget {
  const DeviceProfilesPage({
    super.key,
    required this.vendorId,
    required this.productId,
    required this.deviceName,
    required this.devicePath,
    required this.facade,
    required this.services,
  });

  final int vendorId;
  final int productId;
  final String deviceName;
  final String devicePath;
  final KeyrxFacade facade;
  final ServiceRegistry services;

  @override
  State<DeviceProfilesPage> createState() => _DeviceProfilesPageState();
}

class _DeviceProfilesPageState extends State<DeviceProfilesPage> {
  List<DeviceProfile> _profiles = [];
  String? _activeProfileId;
  bool _isLoading = true;

  @override
  void initState() {
    super.initState();
    _loadProfiles();
  }

  Future<void> _loadProfiles() async {
    setState(() => _isLoading = true);

    final profiles = await widget.services.deviceProfileService.listProfiles(
      widget.vendorId,
      widget.productId,
    );

    final activeId = await widget.services.deviceProfileService.getActiveProfileId(
      widget.vendorId,
      widget.productId,
    );

    if (mounted) {
      setState(() {
        _profiles = profiles;
        _activeProfileId = activeId;
        _isLoading = false;
      });
    }
  }

  Future<void> _setActive(DeviceProfile profile) async {
    await widget.services.deviceProfileService.setActiveProfile(profile);
    if (mounted) {
      ScaffoldMessenger.of(context).showSnackBar(
        SnackBar(content: Text('Set active profile: ${profile.name ?? "Unnamed"}')),
      );
      _loadProfiles();
    }
  }

  Future<void> _deleteProfile(DeviceProfile profile) async {
    final confirmed = await showDialog<bool>(
      context: context,
      builder: (context) => AlertDialog(
        title: const Text('Delete Profile?'),
        content: const Text('This action cannot be undone.'),
        actions: [
          TextButton(
            onPressed: () => Navigator.pop(context, false),
            child: const Text('Cancel'),
          ),
          FilledButton(
            onPressed: () => Navigator.pop(context, true),
            style: FilledButton.styleFrom(backgroundColor: Colors.red),
            child: const Text('Delete'),
          ),
        ],
      ),
    );

    if (confirmed == true) {
      await widget.services.deviceProfileService.deleteProfile(profile);
      if (mounted) {
        _loadProfiles();
      }
    }
  }

  void _viewProfile(DeviceProfile profile) {
    Navigator.of(context).push(
      MaterialPageRoute(
        builder: (context) => DeviceProfilePage(
          vendorId: widget.vendorId,
          productId: widget.productId,
          deviceName: widget.deviceName,
          services: widget.services,
          initialProfile: profile,
        ),
      ),
    );
  }

  void _createNewProfile() {
    Navigator.of(context).push(
      MaterialPageRoute(
        builder: (context) => DeviceDiscoveryPage(
          device: KeyboardDevice(
            path: widget.devicePath,
            name: widget.deviceName,
            vendorId: widget.vendorId,
            productId: widget.productId,
            hasProfile: false, // Not used in discovery
          ),
          facade: widget.facade,
          services: widget.services,
        ),
      ),
    ).then((_) => _loadProfiles());
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        title: const Text('Device Profiles'),
        actions: [
          IconButton(
            icon: const Icon(Icons.add),
            onPressed: _createNewProfile,
            tooltip: 'Create New Profile',
          ),
        ],
      ),
      body: _isLoading
          ? const Center(child: CircularProgressIndicator())
          : _profiles.isEmpty
              ? _buildEmptyState()
              : _buildProfileList(),
    );
  }

  Widget _buildEmptyState() {
    return Center(
      child: Column(
        mainAxisAlignment: MainAxisAlignment.center,
        children: [
          const Icon(Icons.keyboard_alt_outlined, size: 64, color: Colors.grey),
          const SizedBox(height: 16),
          const Text('No profiles found'),
          const SizedBox(height: 24),
          FilledButton.icon(
            onPressed: _createNewProfile,
            icon: const Icon(Icons.add),
            label: const Text('Create Profile'),
          ),
        ],
      ),
    );
  }

  Widget _buildProfileList() {
    // Sort: Active first, then by date descending
    final sortedProfiles = List<DeviceProfile>.from(_profiles);
    sortedProfiles.sort((a, b) {
      final aIsActive = a.discoveredAt.toIso8601String() == _activeProfileId;
      final bIsActive = b.discoveredAt.toIso8601String() == _activeProfileId;
      if (aIsActive && !bIsActive) return -1;
      if (!aIsActive && bIsActive) return 1;
      return b.discoveredAt.compareTo(a.discoveredAt);
    });

    return ListView.builder(
      padding: const EdgeInsets.all(8),
      itemCount: sortedProfiles.length,
      itemBuilder: (context, index) {
        final profile = sortedProfiles[index];
        final isActive = profile.discoveredAt.toIso8601String() == _activeProfileId;

        return Card(
          elevation: isActive ? 2 : 1,
          color: isActive ? Theme.of(context).colorScheme.primaryContainer.withOpacity(0.3) : null,
          child: ListTile(
            leading: CircleAvatar(
              backgroundColor: isActive
                  ? Theme.of(context).colorScheme.primary
                  : Theme.of(context).colorScheme.surfaceContainerHighest,
              child: Icon(
                isActive ? Icons.check : Icons.keyboard,
                color: isActive ? Theme.of(context).colorScheme.onPrimary : null,
              ),
            ),
            title: Text(
              profile.name ?? 'Unnamed Profile',
              style: TextStyle(fontWeight: isActive ? FontWeight.bold : FontWeight.normal),
            ),
            subtitle: Text(
              'Created: ${profile.discoveredAt.toLocal().toString().split('.')[0]}\n'
              '${profile.totalKeys} keys, ${profile.rows} rows',
            ),
            isThreeLine: true,
            onTap: () => _viewProfile(profile),
            trailing: PopupMenuButton<String>(
              onSelected: (value) {
                switch (value) {
                  case 'activate':
                    _setActive(profile);
                    break;
                  case 'delete':
                    _deleteProfile(profile);
                    break;
                  case 'view':
                    _viewProfile(profile);
                    break;
                }
              },
              itemBuilder: (context) => [
                if (!isActive)
                  const PopupMenuItem(
                    value: 'activate',
                    child: Row(
                      children: [
                        Icon(Icons.check_circle_outline, size: 20),
                        SizedBox(width: 8),
                        Text('Set Active'),
                      ],
                    ),
                  ),
                const PopupMenuItem(
                  value: 'view',
                  child: Row(
                    children: [
                      Icon(Icons.visibility_outlined, size: 20),
                      SizedBox(width: 8),
                      Text('View Details'),
                    ],
                  ),
                ),
                const PopupMenuDivider(),
                const PopupMenuItem(
                  value: 'delete',
                  child: Row(
                    children: [
                      Icon(Icons.delete_outline, color: Colors.red, size: 20),
                      SizedBox(width: 8),
                      Text('Delete', style: TextStyle(color: Colors.red)),
                    ],
                  ),
                ),
              ],
            ),
          ),
        );
      },
    );
  }
}
