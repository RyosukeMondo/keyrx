/// Device profile viewer page showing row-column layout and keymap.
///
/// Displays the physical keyboard layout information discovered during
/// the device discovery process.
library;

import 'package:flutter/material.dart';

import '../ffi/bridge.dart';
import '../services/device_profile_service.dart' show VisualKeyOverride;
import '../services/service_registry.dart';
import '../models/keyboard_layout.dart';
import '../widgets/visual_keyboard.dart';

/// Page for viewing device profile information.
class DeviceProfilePage extends StatefulWidget {
  const DeviceProfilePage({
    super.key,
    required this.vendorId,
    required this.productId,
    required this.deviceName,
    required this.services,
    this.initialProfile,
  });

  final int vendorId;
  final int productId;
  final String deviceName;
  final ServiceRegistry services;
  final DeviceProfile? initialProfile;

  @override
  State<DeviceProfilePage> createState() => _DeviceProfilePageState();
}

class _DeviceProfilePageState extends State<DeviceProfilePage> {
  DeviceProfile? _profile;
  Map<String, VisualKeyOverride> _visualOverrides = {};
  bool _isLoading = true;
  String? _error;

  // Edit Mode State
  bool _isEditing = false;
  late TextEditingController _nameController;
  Map<String, VisualKeyOverride> _tempOverrides = {};

  @override
  void initState() {
    super.initState();
    _nameController = TextEditingController();
    if (widget.initialProfile != null) {
      _profile = widget.initialProfile;
      _nameController.text = _profile?.name ?? '';
      // We still need overrides
      _loadOverrides();
    } else {
      _loadProfile();
    }
  }

  @override
  void dispose() {
    _nameController.dispose();
    super.dispose();
  }

  void _toggleEditMode() {
    setState(() {
      _isEditing = !_isEditing;
      if (_isEditing) {
        // Initialize temp state
        _tempOverrides = Map.from(_visualOverrides);
        _nameController.text = _profile?.name ?? '';
      } else {
        // Discard changes (revert to original)
        // If we wanted to confirm discard, we could show a dialog here.
      }
    });
  }

  Future<void> _saveChanges() async {
    if (_profile == null) return;

    setState(() => _isLoading = true);

    // 1. Update Profile Name
    final updatedProfile = DeviceProfile(
      vendorId: _profile!.vendorId,
      productId: _profile!.productId,
      name: _nameController.text,
      source: _profile!.source,
      schemaVersion: _profile!.schemaVersion,
      discoveredAt: _profile!.discoveredAt, // Keep ID same
      rows: _profile!.rows,
      colsPerRow: _profile!.colsPerRow,
      keymap: _profile!.keymap,
      aliases: _profile!.aliases,
    );

    // 2. Save Profile
    await widget.services.deviceProfileService.saveProfile(updatedProfile);

    // 3. Save Visual Overrides
    await widget.services.deviceProfileService.saveVisualOverrides(
      updatedProfile.vendorId,
      updatedProfile.productId,
      _tempOverrides,
    );

    setState(() {
      _profile = updatedProfile;
      _visualOverrides = Map.from(_tempOverrides);
      _isEditing = false;
      _isLoading = false;
    });

    if (mounted) {
      ScaffoldMessenger.of(context).showSnackBar(
        const SnackBar(content: Text('Profile updated')),
      );
    }
  }

  void _updateOverride(String keyId, {double? width, bool? isSkipped}) {
    if (!_isEditing) return;

    setState(() {
      final current = _tempOverrides[keyId] ?? const VisualKeyOverride();
      _tempOverrides[keyId] = VisualKeyOverride(
        width: width ?? current.width,
        isSkipped: isSkipped ?? current.isSkipped,
      );
    });
  }

  Future<void> _showKeyEditDialog(String keyId, int row, int col) async {
    if (!_isEditing) return;

    final current = _tempOverrides[keyId] ?? const VisualKeyOverride();
    double newWidth = current.width;
    bool newSkipped = current.isSkipped;

    await showDialog(
      context: context,
      builder: (context) => StatefulBuilder(
        builder: (context, setState) => AlertDialog(
          title: Text('Edit Key R${row}C${col}'),
          content: Column(
            mainAxisSize: MainAxisSize.min,
            children: [
              Row(
                children: [
                  const Text('Width: '),
                  Expanded(
                    child: Slider(
                      value: newWidth,
                      min: 0.25,
                      max: 4.0,
                      divisions: 15,
                      label: '${newWidth}u',
                      onChanged: (val) => setState(() => newWidth = val),
                    ),
                  ),
                  Text('${newWidth}u'),
                ],
              ),
              SwitchListTile(
                title: const Text('Skip (Gap/Spacer)'),
                value: newSkipped,
                onChanged: (val) => setState(() => newSkipped = val),
              ),
            ],
          ),
          actions: [
            TextButton(onPressed: () => Navigator.pop(context), child: const Text('Cancel')),
            FilledButton(
              onPressed: () {
                _updateOverride(keyId, width: newWidth, isSkipped: newSkipped);
                Navigator.pop(context);
              },
              child: const Text('Apply'),
            ),
          ],
        ),
      ),
    );
  }

  Future<void> _loadOverrides() async {
    final overrides = await widget.services.deviceProfileService
        .getVisualOverrides(widget.vendorId, widget.productId);

    if (mounted) {
      setState(() {
        _visualOverrides = overrides;
        _isLoading = false;
      });
    }
  }

  Future<void> _loadProfile() async {
    setState(() {
      _isLoading = true;
      _error = null;
    });

    final result = await widget.services.deviceProfileService
        .getProfile(widget.vendorId, widget.productId);

    final overrides = await widget.services.deviceProfileService
        .getVisualOverrides(widget.vendorId, widget.productId);

    if (!mounted) return;

    setState(() {
      _isLoading = false;
      if (result.isSuccess) {
        _profile = result.profile;
        _visualOverrides = overrides;
        _nameController.text = _profile?.name ?? '';
      } else {
        _error = result.errorMessage ?? 'Unknown error';
      }
    });
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        title: _isEditing
            ? TextField(
                controller: _nameController,
                decoration: const InputDecoration(
                  hintText: 'Profile Name',
                  border: InputBorder.none,
                ),
                style: const TextStyle(color: Colors.white, fontSize: 20),
                autofocus: true,
              )
            : Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  Text(_profile?.name ?? widget.deviceName),
                  Text(
                    '${widget.vendorId.toRadixString(16).padLeft(4, '0')}:'
                    '${widget.productId.toRadixString(16).padLeft(4, '0')}',
                    style: Theme.of(context).textTheme.bodySmall,
                  ),
                ],
              ),
        actions: [
          if (_isEditing) ...[
            IconButton(
              icon: const Icon(Icons.check),
              onPressed: _saveChanges,
              tooltip: 'Save Changes',
            ),
            IconButton(
              icon: const Icon(Icons.close),
              onPressed: _toggleEditMode,
              tooltip: 'Cancel',
            ),
          ] else
            IconButton(
              icon: const Icon(Icons.edit),
              onPressed: _toggleEditMode,
              tooltip: 'Edit Profile',
            ),
        ],
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
    final overrides = _isEditing ? _tempOverrides : _visualOverrides;
    final layout = _createLayoutFromProfile(profile, overrides);

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
                  'Visual Layout',
                  style: Theme.of(context).textTheme.titleMedium,
                ),
                if (_isEditing) ...[
                  const Spacer(),
                  const Text(
                    'Tap keys to edit',
                    style: TextStyle(fontStyle: FontStyle.italic, fontSize: 12),
                  ),
                ],
              ],
            ),
            const SizedBox(height: 16),
            SizedBox(
              height: 300,
              child: SingleChildScrollView(
                physics: const NeverScrollableScrollPhysics(),
                child: VisualKeyboard(
                  layout: layout,
                  enabled: true, // Always enable so we can tap
                  showMappingOverlay: false,
                  showSecondaryLabels: false,
                  enableDragDrop: false,
                  onKeyTap: _isEditing ? (key) => _showKeyEditDialog(key.id, key.row, 0) : null, // Pass row, logic needs fixing for col
                ),
              ),
            ),
          ],
        ),
      ),
    );
  }

  KeyboardLayout _createLayoutFromProfile(DeviceProfile profile, Map<String, VisualKeyOverride> overrides) {
    final rows = <KeyboardRow>[];

    for (int r = 0; r < profile.rows; r++) {
      final keys = <KeyDefinition>[];
      final cols = profile.colsPerRow.length > r ? profile.colsPerRow[r] : 0;

      for (int c = 0; c < cols; c++) {
        final keyId = 'r${r}_c${c}';
        final override = overrides[keyId];
        final width = override?.width ?? 1.0;
        final isSkipped = override?.isSkipped ?? false;

        keys.add(KeyDefinition(
          id: keyId,
          label: isSkipped ? '' : 'R${r}C${c}',
          row: r,
          column: c.toDouble(), // Placeholder, recalculated below
          width: width,
        ));
      }

      // Recalculate columns based on widths
      double currentX = 0.0;
      final positionedKeys = <KeyDefinition>[];
      for (final key in keys) {
        positionedKeys.add(KeyDefinition(
          id: key.id,
          label: key.label,
          row: key.row,
          column: currentX,
          width: key.width,
        ));
        currentX += key.width;
      }

      rows.add(KeyboardRow(keys: positionedKeys));
    }

    return KeyboardLayout(
      name: profile.name ?? 'Device Layout',
      rows: rows,
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
