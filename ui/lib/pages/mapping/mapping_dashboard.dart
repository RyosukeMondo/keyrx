/// Dashboard for managing keymaps.
library;

import 'package:flutter/material.dart';
import 'package:provider/provider.dart';

import '../../models/hardware_profile.dart';
import '../../models/keymap.dart';
import '../../models/virtual_layout.dart';
import '../../services/hardware_service.dart';
import '../../services/keymap_service.dart';
import '../../services/layout_service.dart';
import '../../services/service_registry.dart';
import 'mapping_editor.dart';

class MappingDashboard extends StatefulWidget {
  const MappingDashboard({super.key});

  @override
  State<MappingDashboard> createState() => _MappingDashboardState();
}

class _MappingDashboardState extends State<MappingDashboard> {
  List<Keymap> _keymaps = [];
  List<HardwareProfile> _profiles = [];
  List<VirtualLayout> _layouts = [];
  bool _isLoading = true;
  String? _errorMessage;
  HardwareService? _cachedHardwareService;

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

  KeymapService get _keymapService {
    try {
      return Provider.of<ServiceRegistry>(context, listen: false).keymapService;
    } on ProviderNotFoundException {
      return Provider.of<KeymapService>(context, listen: false);
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
    WidgetsBinding.instance.addPostFrameCallback((_) {
      _loadData();
      _cachedHardwareService = _hardwareService;
      _cachedHardwareService?.addListener(_loadData);
    });
  }

  @override
  void dispose() {
    _cachedHardwareService?.removeListener(_loadData);
    super.dispose();
  }

  Future<void> _loadData() async {
    setState(() {
      _isLoading = true;
      _errorMessage = null;
    });

    final profilesResult = await _hardwareService.listProfiles();
    final layoutsResult = await _layoutService.listLayouts();
    final keymapsResult = await _keymapService.listKeymaps();

    if (!mounted) return;

    if (profilesResult.hasError ||
        layoutsResult.hasError ||
        keymapsResult.hasError) {
      setState(() {
        _isLoading = false;
        _errorMessage =
            profilesResult.errorMessage ??
            layoutsResult.errorMessage ??
            keymapsResult.errorMessage;
      });
      return;
    }

    setState(() {
      _profiles = profilesResult.data ?? [];
      _layouts = layoutsResult.data ?? [];
      _keymaps = keymapsResult.data ?? [];
      _isLoading = false;
    });
  }

  void _openEditor({
    required Keymap keymap,
    required HardwareProfile profile,
    required VirtualLayout layout,
  }) async {
    await Navigator.of(context).push(
      MaterialPageRoute(
        builder: (context) =>
            MappingEditor(keymap: keymap, profile: profile, layout: layout),
      ),
    );
    // Reload after returning from editor
    _loadData();
  }

  void _createNewKeymap() async {
    if (_profiles.isEmpty) {
      ScaffoldMessenger.of(context).showSnackBar(
        const SnackBar(
          content: Text('No wiring profiles found. Create one first.'),
        ),
      );
      return;
    }

    // Dialog to select profile
    final selectedProfile = await showDialog<HardwareProfile>(
      context: context,
      builder: (context) => _ProfileSelectionDialog(profiles: _profiles),
    );

    if (selectedProfile == null) return;

    final layout = _layouts
        .where((l) => l.id == selectedProfile.virtualLayoutId)
        .firstOrNull;

    if (layout == null) {
      if (mounted) {
        ScaffoldMessenger.of(context).showSnackBar(
          SnackBar(
            content: Text(
              'Layout ${selectedProfile.virtualLayoutId} not found.',
            ),
          ),
        );
      }
      return;
    }

    // Create draft
    final timestamp = DateTime.now().millisecondsSinceEpoch;
    final draft = Keymap(
      id: 'keymap_$timestamp',
      name: '${selectedProfile.name ?? "New"} Keymap',
      virtualLayoutId: layout.id,
      layers: [KeymapLayer(name: 'Layer 0', bindings: const {})],
    );

    if (!mounted) return;
    _openEditor(keymap: draft, profile: selectedProfile, layout: layout);
  }

  Future<void> _deleteKeymap(Keymap keymap) async {
    final confirm = await showDialog<bool>(
      context: context,
      builder: (context) => AlertDialog(
        title: const Text('Delete Keymap?'),
        content: Text('Are you sure you want to delete "${keymap.name}"?'),
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

    if (confirm != true) return;

    final result = await _keymapService.deleteKeymap(keymap.id);
    if (result.hasError) {
      if (mounted) {
        ScaffoldMessenger.of(context).showSnackBar(
          SnackBar(
            content: Text('Failed to delete: ${result.errorMessage}'),
            backgroundColor: Colors.red,
          ),
        );
      }
    } else {
      _loadData();
    }
  }

  @override
  Widget build(BuildContext context) {
    if (_isLoading) {
      return const Center(child: CircularProgressIndicator());
    }

    if (_errorMessage != null) {
      return Center(
        child: Column(
          mainAxisAlignment: MainAxisAlignment.center,
          children: [
            const Icon(Icons.error_outline, size: 48, color: Colors.red),
            const SizedBox(height: 16),
            Text(_errorMessage!),
            const SizedBox(height: 16),
            FilledButton.icon(
              onPressed: _loadData,
              icon: const Icon(Icons.refresh),
              label: const Text('Retry'),
            ),
          ],
        ),
      );
    }

    if (_keymaps.isEmpty) {
      return Center(
        child: Column(
          mainAxisAlignment: MainAxisAlignment.center,
          children: [
            const Icon(Icons.map, size: 64, color: Colors.grey),
            const SizedBox(height: 16),
            Text(
              'No Keymaps Found',
              style: Theme.of(context).textTheme.headlineSmall,
            ),
            const SizedBox(height: 8),
            const Text('Create a keymap to start mapping your device.'),
            const SizedBox(height: 24),
            FilledButton.icon(
              onPressed: _createNewKeymap,
              icon: const Icon(Icons.add),
              label: const Text('Create New Keymap'),
            ),
            if (_profiles.isEmpty) ...[
              const SizedBox(height: 16),
              const Text(
                '(You need a Wiring Profile first)',
                style: TextStyle(color: Colors.orange),
              ),
            ],
          ],
        ),
      );
    }

    return Scaffold(
      floatingActionButton: FloatingActionButton.extended(
        onPressed: _createNewKeymap,
        icon: const Icon(Icons.add),
        label: const Text('New Keymap'),
      ),
      body: RefreshIndicator(
        onRefresh: _loadData,
        child: ListView.builder(
          padding: const EdgeInsets.all(16),
          itemCount: _keymaps.length,
          itemBuilder: (context, index) {
            final keymap = _keymaps[index];
            // Infer profile name from layout ID (rough approximation if 1:1, else just show layout ID)
            final profile = _profiles
                .where((p) => p.virtualLayoutId == keymap.virtualLayoutId)
                .firstOrNull;
            final subtext = profile != null
                ? 'Profile: ${profile.name ?? "Unnamed"} • Layout: ${keymap.virtualLayoutId}'
                : 'Layout: ${keymap.virtualLayoutId}';

            return Card(
              child: ListTile(
                leading: const CircleAvatar(child: Icon(Icons.keyboard)),
                title: Text(keymap.name),
                subtitle: Text(subtext),
                trailing: IconButton(
                  icon: const Icon(Icons.delete_outline),
                  onPressed: () => _deleteKeymap(keymap),
                ),
                onTap: () {
                  if (profile == null) {
                    ScaffoldMessenger.of(context).showSnackBar(
                      const SnackBar(
                        content: Text(
                          'Hardware profile not found for this keymap (layout mismatch?)\nCannot open editor.',
                        ),
                      ),
                    );
                    return;
                  }
                  final layout = _layouts
                      .where((l) => l.id == keymap.virtualLayoutId)
                      .firstOrNull;
                  if (layout == null) {
                    ScaffoldMessenger.of(context).showSnackBar(
                      const SnackBar(
                        content: Text('Virtual Layout not found.'),
                      ),
                    );
                    return;
                  }
                  _openEditor(keymap: keymap, profile: profile, layout: layout);
                },
              ),
            );
          },
        ),
      ),
    );
  }
}

class _ProfileSelectionDialog extends StatefulWidget {
  const _ProfileSelectionDialog({required this.profiles});

  final List<HardwareProfile> profiles;

  @override
  State<_ProfileSelectionDialog> createState() =>
      _ProfileSelectionDialogState();
}

class _ProfileSelectionDialogState extends State<_ProfileSelectionDialog> {
  HardwareProfile? _selected;

  @override
  void initState() {
    super.initState();
    if (widget.profiles.isNotEmpty) {
      _selected = widget.profiles.first;
    }
  }

  @override
  Widget build(BuildContext context) {
    return AlertDialog(
      title: const Text('Select Wiring Profile'),
      content: DropdownButtonFormField<HardwareProfile>(
        initialValue: _selected,
        decoration: const InputDecoration(
          labelText: 'Profile',
          border: OutlineInputBorder(),
        ),
        items: widget.profiles.map((p) {
          return DropdownMenuItem(
            value: p,
            child: Text(
              '${p.name ?? "Unnamed"} (${p.virtualLayoutId})',
              overflow: TextOverflow.ellipsis,
            ),
          );
        }).toList(),
        onChanged: (val) {
          setState(() {
            _selected = val;
          });
        },
      ),
      actions: [
        TextButton(
          onPressed: () => Navigator.pop(context),
          child: const Text('Cancel'),
        ),
        FilledButton(
          onPressed: _selected == null
              ? null
              : () => Navigator.pop(context, _selected),
          child: const Text('Create'),
        ),
      ],
    );
  }
}
