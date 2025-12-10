/// Wiring page for mapping physical scancodes to virtual layout keys.
///
/// Interaction model:
/// - Dashboard: List of hardware profiles.
/// - Editor: Detailed view for wiring a specific profile.
library;

import 'dart:async';

import 'package:flutter/material.dart';
import 'package:provider/provider.dart';

import '../models/device_state.dart';
import '../models/hardware_profile.dart';
import '../models/keyboard_layout.dart';
import '../models/key_codes_windows.dart';
import '../models/virtual_layout.dart';
import '../services/device_registry_service.dart';
import '../services/hardware_service.dart';
import '../services/layout_service.dart';
import '../services/service_registry.dart';
import '../widgets/visual_keyboard.dart';
import '../widgets/virtual_layout_renderer.dart';

class WiringPage extends StatefulWidget {
  const WiringPage({super.key});

  @override
  State<WiringPage> createState() => _WiringPageState();
}

class _WiringPageState extends State<WiringPage> {
  final TextEditingController _idController = TextEditingController();
  final TextEditingController _nameController = TextEditingController();
  final TextEditingController _vendorController = TextEditingController();
  final TextEditingController _productController = TextEditingController();
  final TextEditingController _serialController = TextEditingController();

  List<HardwareProfile> _profiles = const [];
  List<VirtualLayout> _layouts = const [];
  List<DeviceState> _knownDevices = const [];
  Map<int, String> _wiringDraft = {};

  HardwareProfile? _selectedProfile;
  String? _selectedPhysicalKeyId;
  String? _selectedVirtualKeyId;
  String? _selectedLayoutId;
  bool _isLoading = true;
  bool _isEditing = false;
  bool _isSaving = false;
  String? _errorMessage;
  StreamSubscription<List<DeviceState>>? _deviceSubscription;

  HardwareService get _hardwareService {
    try {
      return Provider.of<ServiceRegistry>(
        context,
        listen: false,
      ).hardwareService;
    } on ProviderNotFoundException {
      return Provider.of<HardwareService>(context, listen: false);
    }
  }

  LayoutService get _layoutService {
    try {
      return Provider.of<ServiceRegistry>(context, listen: false).layoutService;
    } on ProviderNotFoundException {
      return Provider.of<LayoutService>(context, listen: false);
    }
  }

  DeviceRegistryService get _deviceRegistryService {
    try {
      return Provider.of<ServiceRegistry>(
        context,
        listen: false,
      ).deviceRegistryService;
    } on ProviderNotFoundException {
      return Provider.of<DeviceRegistryService>(context, listen: false);
    }
  }

  @override
  void initState() {
    super.initState();
    WidgetsBinding.instance.addPostFrameCallback((_) {
      _loadData();
      _subscribeToDevices();
    });
  }

  void _subscribeToDevices() {
    _deviceSubscription?.cancel();
    _deviceSubscription = _deviceRegistryService.devicesStream.listen((
      devices,
    ) {
      if (mounted) {
        setState(() {
          _knownDevices = devices;
        });
      }
    });
  }

  @override
  void dispose() {
    _deviceSubscription?.cancel();
    _idController.dispose();
    _nameController.dispose();
    _vendorController.dispose();
    _productController.dispose();
    _serialController.dispose();
    super.dispose();
  }

  Future<void> _loadData() async {
    setState(() {
      _isLoading = true;
      _errorMessage = null;
    });

    final profilesResult = await _hardwareService.listProfiles();
    final layoutsResult = await _layoutService.listLayouts();
    List<DeviceState> devices = [];
    try {
      devices = await _deviceRegistryService.getDevices();
    } catch (e) {
      debugPrint('Failed to fetch devices: $e');
    }

    if (!mounted) return;

    if (profilesResult.hasError) {
      setState(() {
        _isLoading = false;
        _errorMessage = profilesResult.errorMessage;
      });
      return;
    }

    if (layoutsResult.hasError) {
      setState(() {
        _isLoading = false;
        _errorMessage = layoutsResult.errorMessage;
      });
      return;
    }

    setState(() {
      _profiles = profilesResult.data ?? [];
      _layouts = layoutsResult.data ?? [];
      _knownDevices = devices;
      _isLoading = false;
    });
  }

  void _selectProfile(HardwareProfile profile) {
    _idController.text = profile.id;
    _nameController.text = profile.name ?? '';
    _vendorController.text = profile.vendorId.toString();
    _productController.text = profile.productId.toString();
    _serialController.text = profile.serialNumber ?? '';
    _selectedLayoutId = profile.virtualLayoutId;
    _wiringDraft = Map<int, String>.from(profile.wiring);

    setState(() {
      _selectedProfile = profile;
      _selectedPhysicalKeyId = null;
      _selectedVirtualKeyId = null;
      _isEditing = true;
    });
  }

  void _startNewProfile() {
    _idController.text = 'hardware_${DateTime.now().millisecondsSinceEpoch}';
    _nameController.text = 'New Hardware';
    _vendorController.text = '0';
    _productController.text = '0';
    _serialController.clear();
    _selectedLayoutId = _layouts.isNotEmpty ? _layouts.first.id : null;
    _wiringDraft = {};

    setState(() {
      _selectedProfile = null;
      _selectedPhysicalKeyId = null;
      _selectedVirtualKeyId = null;
      _isEditing = true;
    });
  }

  VirtualLayout? get _selectedLayout {
    if (_selectedLayoutId == null) return null;
    for (final layout in _layouts) {
      if (layout.id == _selectedLayoutId) return layout;
    }
    return _layouts.isNotEmpty ? _layouts.first : null;
  }

  Future<void> _saveProfile() async {
    final vendorId = int.tryParse(_vendorController.text.trim()) ?? 0;
    final productId = int.tryParse(_productController.text.trim()) ?? 0;
    final layoutId = _selectedLayoutId;
    final layout = _selectedLayout;

    if (layoutId == null || layout == null) {
      _showSnack('Select a virtual layout first', isError: true);
      return;
    }

    final allowedKeys = layout.keys.map((k) => k.id).toSet();
    final filteredWiring = Map<int, String>.fromEntries(
      _wiringDraft.entries.where((entry) => allowedKeys.contains(entry.value)),
    );

    final profile = HardwareProfile(
      id: _idController.text.trim(),
      vendorId: vendorId,
      productId: productId,
      serialNumber: _serialController.text.trim().isEmpty
          ? null
          : _serialController.text.trim(),
      name: _nameController.text.trim().isEmpty
          ? null
          : _nameController.text.trim(),
      virtualLayoutId: layoutId,
      wiring: filteredWiring,
    );

    setState(() {
      _isSaving = true;
      _errorMessage = null;
    });

    final result = await _hardwareService.saveProfile(profile);
    if (!mounted) return;

    setState(() => _isSaving = false);

    if (result.hasError) {
      setState(() {
        _errorMessage = result.errorMessage;
      });
      _showSnack(
        'Failed to save profile: ${result.errorMessage}',
        isError: true,
      );
      return;
    }

    // Reload list and exit edit mode
    await _loadData();
    setState(() {
      _isEditing = false;
    });
    _showSnack('Profile saved');
  }

  Future<void> _confirmDelete(HardwareProfile profile) async {
    final confirmed = await showDialog<bool>(
      context: context,
      builder: (context) => AlertDialog(
        title: const Text('Delete Profile?'),
        content: Text(
          'Are you sure you want to delete "${profile.name ?? profile.id}"?',
        ),
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
      final result = await _hardwareService.deleteProfile(profile.id);
      if (result.isSuccess) {
        await _loadData();
        _showSnack('Profile deleted');
      } else {
        _showSnack('Failed to delete: ${result.errorMessage}', isError: true);
      }
    }
  }

  void _handlePhysicalKeyTap(KeyDefinition key) {
    setState(() {
      _selectedPhysicalKeyId = key.id;
      final scanCode = keyIdToWindowsScanCode[key.id];
      _selectedVirtualKeyId = scanCode != null ? _wiringDraft[scanCode] : null;
    });
  }

  void _handleVirtualKeyTap(String virtualKeyId) {
    if (_selectedPhysicalKeyId == null) {
      setState(() {
        _selectedVirtualKeyId = virtualKeyId;
      });
      return;
    }

    final scanCode = keyIdToWindowsScanCode[_selectedPhysicalKeyId];
    if (scanCode == null) {
      _showSnack(
        'No scan code found for $_selectedPhysicalKeyId',
        isError: true,
      );
      return;
    }

    final updated = Map<int, String>.from(_wiringDraft);
    updated[scanCode] = virtualKeyId;

    setState(() {
      _wiringDraft = updated;
      _selectedVirtualKeyId = virtualKeyId;
    });

    _showSnack('Mapped $_selectedPhysicalKeyId → $virtualKeyId');
  }

  void _clearMappingForSelectedPhysical() {
    if (_selectedPhysicalKeyId == null) return;
    final scanCode = keyIdToWindowsScanCode[_selectedPhysicalKeyId];
    if (scanCode == null) return;

    final updated = Map<int, String>.from(_wiringDraft);
    updated.remove(scanCode);

    setState(() {
      _wiringDraft = updated;
      _selectedVirtualKeyId = null;
    });
  }

  Set<String> get _mappedPhysicalKeyIds {
    final inverse = <int, String>{};
    for (final entry in keyIdToWindowsScanCode.entries) {
      inverse[entry.value] = entry.key;
    }

    final mapped = <String>{};
    for (final scanCode in _wiringDraft.keys) {
      final keyId = inverse[scanCode];
      if (keyId != null) mapped.add(keyId);
    }
    return mapped;
  }

  void _showSnack(String message, {bool isError = false}) {
    ScaffoldMessenger.of(context).showSnackBar(
      SnackBar(
        content: Text(message),
        backgroundColor: isError ? Colors.red : null,
      ),
    );
  }

  // --- Views ---

  Widget _buildDashboard() {
    if (_profiles.isEmpty) {
      return Scaffold(
        appBar: AppBar(title: const Text('Hardware Profiles')),
        body: Center(
          child: Column(
            mainAxisAlignment: MainAxisAlignment.center,
            children: [
              const Icon(Icons.cable, size: 64, color: Colors.grey),
              const SizedBox(height: 16),
              Text(
                'No Hardware Profiles',
                style: Theme.of(context).textTheme.headlineSmall,
              ),
              const SizedBox(height: 8),
              const Text('Create a profile to wire your device.'),
              const SizedBox(height: 24),
              FilledButton.icon(
                onPressed: _startNewProfile,
                icon: const Icon(Icons.add),
                label: const Text('Create New Profile'),
              ),
            ],
          ),
        ),
      );
    }

    return Scaffold(
      appBar: AppBar(
        title: const Text('Hardware Profiles'),
        actions: [
          IconButton(
            icon: const Icon(Icons.refresh),
            onPressed: _isLoading ? null : _loadData,
          ),
          IconButton(
            onPressed: _startNewProfile,
            icon: const Icon(Icons.add),
            tooltip: 'New Profile',
          ),
        ],
      ),
      body: GridView.builder(
        padding: const EdgeInsets.all(16),
        gridDelegate: const SliverGridDelegateWithMaxCrossAxisExtent(
          maxCrossAxisExtent: 400,
          childAspectRatio: 1.5,
          crossAxisSpacing: 16,
          mainAxisSpacing: 16,
        ),
        itemCount: _profiles.length,
        itemBuilder: (context, index) {
          final profile = _profiles[index];
          return Card(
            clipBehavior: Clip.antiAlias,
            child: InkWell(
              onTap: () => _selectProfile(profile),
              child: Padding(
                padding: const EdgeInsets.all(16),
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    Row(
                      children: [
                        const Icon(Icons.keyboard),
                        const SizedBox(width: 8),
                        Expanded(
                          child: Text(
                            profile.name ?? 'Unnamed Profile',
                            style: Theme.of(context).textTheme.titleMedium,
                            overflow: TextOverflow.ellipsis,
                          ),
                        ),
                        PopupMenuButton<String>(
                          onSelected: (value) async {
                            if (value == 'delete') {
                              _confirmDelete(profile);
                            }
                          },
                          itemBuilder: (context) => [
                            const PopupMenuItem(
                              value: 'delete',
                              child: Row(
                                children: [
                                  Icon(Icons.delete, color: Colors.red),
                                  SizedBox(width: 8),
                                  Text(
                                    'Delete',
                                    style: TextStyle(color: Colors.red),
                                  ),
                                ],
                              ),
                            ),
                          ],
                        ),
                      ],
                    ),
                    const Divider(),
                    Text(
                      'ID: ${profile.id}',
                      style: Theme.of(context).textTheme.bodySmall,
                    ),
                    const SizedBox(height: 4),
                    Text(
                      'VID: ${profile.vendorId.toRadixString(16).padLeft(4, '0')}  '
                      'PID: ${profile.productId.toRadixString(16).padLeft(4, '0')}',
                      style: Theme.of(
                        context,
                      ).textTheme.bodySmall?.copyWith(fontFamily: 'monospace'),
                    ),
                    if (profile.serialNumber != null) ...[
                      const SizedBox(height: 4),
                      Text(
                        'Serial: ${profile.serialNumber}',
                        style: Theme.of(context).textTheme.bodySmall?.copyWith(
                          fontFamily: 'monospace',
                        ),
                      ),
                    ],
                    const Spacer(),
                    Container(
                      padding: const EdgeInsets.symmetric(
                        horizontal: 8,
                        vertical: 4,
                      ),
                      decoration: BoxDecoration(
                        color: Theme.of(
                          context,
                        ).colorScheme.surfaceContainerHighest,
                        borderRadius: BorderRadius.circular(4),
                      ),
                      child: Text(
                        'Layout: ${profile.virtualLayoutId}',
                        style: Theme.of(context).textTheme.labelSmall,
                      ),
                    ),
                  ],
                ),
              ),
            ),
          );
        },
      ),
    );
  }

  Widget _buildEditor() {
    return Scaffold(
      appBar: AppBar(
        leading: IconButton(
          icon: const Icon(Icons.arrow_back),
          onPressed: () {
            setState(() {
              _isEditing = false;
              _selectedProfile = null;
            });
          },
        ),
        title: Text(_selectedProfile == null ? 'New Profile' : 'Edit Profile'),
        actions: [
          IconButton(
            icon: const Icon(Icons.save),
            onPressed: _isSaving ? null : _saveProfile,
          ),
        ],
      ),
      body: LayoutBuilder(
        builder: (context, constraints) {
          final isWide = constraints.maxWidth > 1000;
          final panels = [
            Expanded(child: _buildPhysicalPanel()),
            Expanded(child: _buildVirtualPanel()),
          ];

          if (isWide) {
            return Column(
              children: [
                _buildProfilePicker(), // Collapsible form
                Expanded(
                  child: Row(
                    crossAxisAlignment: CrossAxisAlignment.start,
                    children: panels,
                  ),
                ),
              ],
            );
          } else {
            return ListView(
              children: [
                _buildProfilePicker(),
                _buildPhysicalPanel(),
                _buildVirtualPanel(),
              ],
            );
          }
        },
      ),
    );
  }

  // --- Helper Widgets for Editor ---

  Widget _buildProfilePicker() {
    return Card(
      elevation: 1,
      margin: const EdgeInsets.all(12),
      child: ExpansionTile(
        initiallyExpanded:
            _selectedProfile == null, // Expand only if creating new
        shape: Border.all(color: Colors.transparent),
        title: Text(
          _nameController.text.isEmpty
              ? 'Profile Settings'
              : _nameController.text,
          style: const TextStyle(fontWeight: FontWeight.w600),
        ),
        subtitle: Text(
          '${_vendorController.text.padLeft(4, '0')}:${_productController.text.padLeft(4, '0')} • ${_selectedLayoutId ?? "No Layout"}',
          style: Theme.of(context).textTheme.bodySmall,
        ),
        childrenPadding: const EdgeInsets.all(16),
        children: [
          // 1. Display Name
          TextField(
            controller: _nameController,
            decoration: const InputDecoration(
              labelText: 'Display Name',
              border: OutlineInputBorder(),
            ),
          ),
          const SizedBox(height: 12),

          // 2. Row: Target Device | Hardware Identity
          Row(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              Expanded(
                flex: 5,
                child: DropdownButtonFormField<String>(
                  initialValue: _findMatchingDeviceKey(),
                  isExpanded: true,
                  decoration: const InputDecoration(
                    labelText: 'Target Device',
                    border: OutlineInputBorder(),
                    helperText: 'Select a connected or virtual device',
                  ),
                  items: _knownDevices.map((d) {
                    final key = d.identity.toKey();
                    final label =
                        d.identity.userLabel ??
                        '${d.identity.vendorId.toRadixString(16).padLeft(4, '0')}:${d.identity.productId.toRadixString(16).padLeft(4, '0')}';
                    final serial = d.identity.serialNumber.isNotEmpty
                        ? ' (${d.identity.serialNumber})'
                        : '';
                    return DropdownMenuItem(
                      value: key,
                      child: Text(
                        '$label$serial',
                        overflow: TextOverflow.ellipsis,
                      ),
                    );
                  }).toList(),
                  onChanged: (value) {
                    if (value == null) return;
                    final device = _knownDevices.firstWhere(
                      (d) => d.identity.toKey() == value,
                      orElse: () => _knownDevices.first,
                    );
                    setState(() {
                      _vendorController.text = device.identity.vendorId
                          .toString();
                      _productController.text = device.identity.productId
                          .toString();
                      _serialController.text = device.identity.serialNumber;

                      // Auto-name if empty or default
                      if (_nameController.text.isEmpty ||
                          _nameController.text == 'New Hardware') {
                        _nameController.text =
                            device.identity.userLabel ??
                            '${device.identity.toKey()} Profile';
                      }
                    });
                  },
                  validator: (value) =>
                      value == null ? 'Please select a device' : null,
                ),
              ),
              const SizedBox(width: 12),
              Expanded(
                flex: 4,
                child: Container(
                  padding: const EdgeInsets.all(12),
                  decoration: BoxDecoration(
                    color: Theme.of(
                      context,
                    ).colorScheme.surfaceContainerHighest,
                    borderRadius: BorderRadius.circular(8),
                    border: Border.all(color: Theme.of(context).dividerColor),
                  ),
                  child: Column(
                    crossAxisAlignment: CrossAxisAlignment.start,
                    children: [
                      Text(
                        'Hardware Identity',
                        style: Theme.of(context).textTheme.labelMedium,
                      ),
                      const SizedBox(height: 4),
                      Wrap(
                        spacing: 8,
                        runSpacing: 4,
                        children: [
                          _InfoChip(
                            label:
                                'VID: ${_vendorController.text.isEmpty ? '?' : int.tryParse(_vendorController.text)?.toRadixString(16).padLeft(4, '0').toUpperCase()}',
                          ),
                          _InfoChip(
                            label:
                                'PID: ${_productController.text.isEmpty ? '?' : int.tryParse(_productController.text)?.toRadixString(16).padLeft(4, '0').toUpperCase()}',
                          ),
                          _InfoChip(
                            label:
                                'SN: ${_serialController.text.isEmpty ? 'None' : _serialController.text}',
                          ),
                        ],
                      ),
                    ],
                  ),
                ),
              ),
            ],
          ),
          const SizedBox(height: 12),

          // 3. Virtual Layout
          DropdownButtonFormField<String>(
            initialValue: _selectedLayoutId,
            isExpanded: true,
            decoration: const InputDecoration(
              labelText: 'Virtual layout',
              border: OutlineInputBorder(),
            ),
            items: _layouts
                .map(
                  (layout) => DropdownMenuItem(
                    value: layout.id,
                    child: Text('${layout.name} • ${layout.id}'),
                  ),
                )
                .toList(),
            onChanged: (value) {
              setState(() {
                _selectedLayoutId = value;
                _selectedVirtualKeyId = null;
              });
            },
          ),
          const SizedBox(height: 16),
          if (_selectedPhysicalKeyId != null)
            TextButton.icon(
              onPressed: _clearMappingForSelectedPhysical,
              icon: const Icon(Icons.link_off),
              label: const Text('Unmap selected key'),
            ),
          if (_errorMessage != null) ...[
            const SizedBox(height: 8),
            Text(
              _errorMessage!,
              style: TextStyle(color: Theme.of(context).colorScheme.error),
            ),
          ],
        ],
      ),
    );
  }

  Widget _buildPhysicalPanel() {
    return Card(
      margin: const EdgeInsets.all(12),
      child: Padding(
        padding: const EdgeInsets.all(12),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            const Text(
              'Physical Keys',
              style: TextStyle(fontSize: 16, fontWeight: FontWeight.bold),
            ),
            const SizedBox(height: 4),
            Text(
              _selectedPhysicalKeyId == null
                  ? 'Tap a physical key to start wiring'
                  : 'Selected: $_selectedPhysicalKeyId',
              style: Theme.of(context).textTheme.bodySmall,
            ),
            const SizedBox(height: 12),
            SizedBox(
              height: 280,
              child: VisualKeyboard(
                layout: KeyboardLayout.full(),
                onKeyTap: _handlePhysicalKeyTap,
                selectedKeys: _selectedPhysicalKeyId == null
                    ? {}
                    : {_selectedPhysicalKeyId!},
                mappedKeys: _mappedPhysicalKeyIds,
                showMappingOverlay: false,
                enableDragDrop: false,
              ),
            ),
          ],
        ),
      ),
    );
  }

  Widget _buildVirtualPanel() {
    final layout = _selectedLayout;
    if (layout == null) {
      return const Card(
        margin: EdgeInsets.all(12),
        child: Padding(
          padding: EdgeInsets.all(16),
          child: Text('Select or create a virtual layout to wire keys.'),
        ),
      );
    }

    // final buttons = _buildVirtualKeyButtons(layout); // Removed
    final mappedCount = _wiringDraft.length;

    return Card(
      margin: const EdgeInsets.all(12),
      child: Padding(
        padding: const EdgeInsets.all(12),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Text(
              'Virtual Layout: ${layout.name}',
              style: const TextStyle(fontSize: 16, fontWeight: FontWeight.bold),
            ),
            const SizedBox(height: 4),
            Text(
              'Tap a virtual key to map the selected physical key. Mapped: $mappedCount',
              style: Theme.of(context).textTheme.bodySmall,
            ),
            const SizedBox(height: 12),
            if (layout.keys.isEmpty)
              const Text('No keys defined for this layout.')
            else
              Expanded(
                child: VirtualLayoutRenderer(
                  layout: layout,
                  selectedKeyId: _selectedVirtualKeyId,
                  mappedKeyIds: _allMappedVirtualKeyIds,
                  onKeyTap: _handleVirtualKeyTap,
                ),
              ),
          ],
        ),
      ),
    );
  }

  // Get all virtual keys that are target of some mapping
  Set<String> get _allMappedVirtualKeyIds {
    return _wiringDraft.values.toSet();
  }

  @override
  Widget build(BuildContext context) {
    if (_isLoading && !_isEditing) {
      return const Scaffold(body: Center(child: CircularProgressIndicator()));
    }

    if (_isEditing) {
      return _buildEditor();
    }

    return _buildDashboard();
  }

  String? _findMatchingDeviceKey() {
    final vid = int.tryParse(_vendorController.text);
    final pid = int.tryParse(_productController.text);
    final serial = _serialController.text;

    if (vid == null || pid == null) return null;

    try {
      final match = _knownDevices.firstWhere((d) {
        return d.identity.vendorId == vid &&
            d.identity.productId == pid &&
            d.identity.serialNumber == serial;
      });
      return match.identity.toKey();
    } catch (_) {
      return null;
    }
  }
}

class _InfoChip extends StatelessWidget {
  const _InfoChip({required this.label});

  final String label;

  @override
  Widget build(BuildContext context) {
    return Container(
      padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 4),
      decoration: BoxDecoration(
        color: Theme.of(context).colorScheme.surface,
        borderRadius: BorderRadius.circular(4),
        border: Border.all(color: Theme.of(context).dividerColor),
      ),
      child: Text(
        label,
        style: TextStyle(
          fontFamily: 'monospace',
          fontSize: 12,
          color: Theme.of(context).colorScheme.onSurface,
        ),
      ),
    );
  }
}
