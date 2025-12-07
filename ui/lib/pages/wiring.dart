/// Wiring page for mapping physical scancodes to virtual layout keys.
///
/// Interaction model:
/// - Physical source (ANSI keyboard) displayed on top.
/// - Virtual layout target displayed on bottom.
/// - Tap a physical key, then tap a virtual key to assign the wiring.
library;

import 'package:flutter/material.dart';
import 'package:provider/provider.dart';

import '../models/hardware_profile.dart';
import '../models/keyboard_layout.dart';
import '../models/key_codes_windows.dart';
import '../models/virtual_layout.dart';
import '../services/hardware_service.dart';
import '../services/layout_service.dart';
import '../services/service_registry.dart';
import '../widgets/visual_keyboard.dart';

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

  List<HardwareProfile> _profiles = const [];
  List<VirtualLayout> _layouts = const [];
  Map<int, String> _wiringDraft = {};

  HardwareProfile? _selectedProfile;
  String? _selectedPhysicalKeyId;
  String? _selectedVirtualKeyId;
  String? _selectedLayoutId;
  bool _isLoading = true;
  bool _isSaving = false;
  String? _errorMessage;

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

  @override
  void initState() {
    super.initState();
    _loadData();
  }

  @override
  void dispose() {
    _idController.dispose();
    _nameController.dispose();
    _vendorController.dispose();
    _productController.dispose();
    super.dispose();
  }

  Future<void> _loadData() async {
    setState(() {
      _isLoading = true;
      _errorMessage = null;
    });

    final profilesResult = await _hardwareService.listProfiles();
    final layoutsResult = await _layoutService.listLayouts();

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

    final profiles = profilesResult.data ?? [];
    final layouts = layoutsResult.data ?? [];
    setState(() {
      _profiles = profiles;
      _layouts = layouts;
      _isLoading = false;
    });

    if (profiles.isNotEmpty) {
      _selectProfile(profiles.first);
    } else {
      _startNewProfile();
    }
  }

  void _selectProfile(HardwareProfile profile) {
    _idController.text = profile.id;
    _nameController.text = profile.name ?? '';
    _vendorController.text = profile.vendorId.toString();
    _productController.text = profile.productId.toString();
    _selectedLayoutId = profile.virtualLayoutId;
    _wiringDraft = Map<int, String>.from(profile.wiring);

    setState(() {
      _selectedProfile = profile;
      _selectedPhysicalKeyId = null;
      _selectedVirtualKeyId = null;
    });
  }

  void _startNewProfile() {
    _idController.text = 'hardware_${DateTime.now().millisecondsSinceEpoch}';
    _nameController.text = 'New Hardware';
    _vendorController.text = '0';
    _productController.text = '0';
    _selectedLayoutId = _layouts.isNotEmpty ? _layouts.first.id : null;
    _wiringDraft = {};

    setState(() {
      _selectedProfile = null;
      _selectedPhysicalKeyId = null;
      _selectedVirtualKeyId = null;
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

    final saved = result.data ?? profile;
    final updated = [..._profiles];
    final existingIndex = updated.indexWhere((p) => p.id == saved.id);
    if (existingIndex >= 0) {
      updated[existingIndex] = saved;
    } else {
      updated.add(saved);
    }

    setState(() {
      _profiles = updated;
      _selectedProfile = saved;
    });

    _showSnack('Profile saved');
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

  List<Widget> _buildVirtualKeyButtons(VirtualLayout layout) {
    final mappedForSelectedPhysical = _selectedPhysicalKeyId != null
        ? _wiringDraft[keyIdToWindowsScanCode[_selectedPhysicalKeyId] ?? -1]
        : null;

    final keys = [...layout.keys];
    keys.sort((a, b) {
      final ay = a.position?.y ?? 0;
      final by = b.position?.y ?? 0;
      final ax = a.position?.x ?? 0;
      final bx = b.position?.x ?? 0;
      return ay != by ? ay.compareTo(by) : ax.compareTo(bx);
    });

    return keys
        .map(
          (key) => Padding(
            padding: const EdgeInsets.all(4),
            child: ChoiceChip(
              label: Text(key.label),
              selected:
                  _selectedVirtualKeyId == key.id ||
                  mappedForSelectedPhysical == key.id,
              onSelected: (_) => _handleVirtualKeyTap(key.id),
            ),
          ),
        )
        .toList();
  }

  Widget _buildProfilePicker() {
    return Card(
      elevation: 1,
      margin: const EdgeInsets.all(12),
      child: Padding(
        padding: const EdgeInsets.all(16),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Row(
              children: [
                const Text(
                  'Hardware Profiles',
                  style: TextStyle(fontSize: 18, fontWeight: FontWeight.w600),
                ),
                const Spacer(),
                IconButton(
                  tooltip: 'Refresh',
                  icon: const Icon(Icons.refresh),
                  onPressed: _isLoading ? null : _loadData,
                ),
                FilledButton.icon(
                  icon: const Icon(Icons.add),
                  label: const Text('New'),
                  onPressed: _isSaving ? null : _startNewProfile,
                ),
              ],
            ),
            const SizedBox(height: 12),
            DropdownButtonFormField<String>(
              value: _selectedProfile?.id,
              isExpanded: true,
              hint: const Text('Select a profile'),
              decoration: const InputDecoration(
                labelText: 'Profile',
                border: OutlineInputBorder(),
              ),
              items: _profiles
                  .map(
                    (profile) => DropdownMenuItem(
                      value: profile.id,
                      child: Text(profile.name ?? profile.id),
                    ),
                  )
                  .toList(),
              onChanged: (value) {
                if (value == null) return;
                final profile = _profiles.firstWhere(
                  (p) => p.id == value,
                  orElse: () => _profiles.first,
                );
                _selectProfile(profile);
              },
            ),
            const SizedBox(height: 16),
            Row(
              children: [
                Expanded(
                  child: TextField(
                    controller: _idController,
                    decoration: const InputDecoration(
                      labelText: 'Profile ID',
                      border: OutlineInputBorder(),
                    ),
                  ),
                ),
                const SizedBox(width: 12),
                Expanded(
                  child: TextField(
                    controller: _nameController,
                    decoration: const InputDecoration(
                      labelText: 'Display name',
                      border: OutlineInputBorder(),
                    ),
                  ),
                ),
              ],
            ),
            const SizedBox(height: 12),
            Row(
              children: [
                Expanded(
                  child: TextField(
                    controller: _vendorController,
                    decoration: const InputDecoration(
                      labelText: 'Vendor ID',
                      border: OutlineInputBorder(),
                    ),
                    keyboardType: TextInputType.number,
                  ),
                ),
                const SizedBox(width: 12),
                Expanded(
                  child: TextField(
                    controller: _productController,
                    decoration: const InputDecoration(
                      labelText: 'Product ID',
                      border: OutlineInputBorder(),
                    ),
                    keyboardType: TextInputType.number,
                  ),
                ),
              ],
            ),
            const SizedBox(height: 12),
            DropdownButtonFormField<String>(
              value: _selectedLayoutId,
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
            Row(
              children: [
                FilledButton.icon(
                  icon: const Icon(Icons.save),
                  label: const Text('Save Profile'),
                  onPressed: _isSaving ? null : _saveProfile,
                ),
                const SizedBox(width: 12),
                if (_selectedPhysicalKeyId != null)
                  TextButton.icon(
                    onPressed: _clearMappingForSelectedPhysical,
                    icon: const Icon(Icons.link_off),
                    label: const Text('Unmap selected key'),
                  ),
              ],
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

    final buttons = _buildVirtualKeyButtons(layout);
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
              Wrap(children: buttons),
          ],
        ),
      ),
    );
  }

  void _showSnack(String message, {bool isError = false}) {
    ScaffoldMessenger.of(context).showSnackBar(
      SnackBar(
        content: Text(message),
        backgroundColor: isError ? Colors.red : null,
      ),
    );
  }

  @override
  Widget build(BuildContext context) {
    if (_isLoading) {
      return const Scaffold(body: Center(child: CircularProgressIndicator()));
    }

    return Scaffold(
      appBar: AppBar(
        title: const Text('Wiring'),
        actions: [
          IconButton(
            icon: const Icon(Icons.refresh),
            onPressed: _isLoading ? null : _loadData,
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

          return Column(
            children: [
              _buildProfilePicker(),
              if (isWide)
                Expanded(
                  child: Row(
                    crossAxisAlignment: CrossAxisAlignment.start,
                    children: panels,
                  ),
                )
              else
                Expanded(
                  child: ListView(
                    children: [_buildPhysicalPanel(), _buildVirtualPanel()],
                  ),
                ),
            ],
          );
        },
      ),
    );
  }
}
