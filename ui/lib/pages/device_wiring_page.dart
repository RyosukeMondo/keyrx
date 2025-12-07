/// Device wiring editor page.
///
/// Provides a visual interface for wiring physical keys to the virtual matrix
/// (Device Profile Editor). Replaces the old VisualEditorPage.
library;

import 'dart:async';

import 'package:flutter/material.dart';
import 'package:provider/provider.dart';

import '../ffi/bridge_device_profile.dart';
import '../models/device_state.dart';
import '../models/keyboard_layout.dart';
import '../models/key_codes_windows.dart';
import '../services/device_profile_service.dart';
import '../services/device_registry_service.dart';
import '../services/service_registry.dart';
import '../widgets/visual_keyboard.dart';
import 'visual_editor_widgets.dart' show InlineMessage, InlineMessageVariant;

/// Page for wiring physical keys to the virtual matrix.
class DeviceWiringPage extends StatefulWidget {
  const DeviceWiringPage({super.key});

  @override
  State<DeviceWiringPage> createState() => _DeviceWiringPageState();
}

class _DeviceWiringPageState extends State<DeviceWiringPage> {
  List<DeviceState> _devices = [];
  DeviceState? _selectedDevice;
  DeviceProfile? _profile;
  bool _isLoading = false;
  String? _errorMessage;

  // Mapping State
  String? _selectedPhysicalKeyId;

  DeviceRegistryService get _deviceRegistry => Provider.of<ServiceRegistry>(
    context,
    listen: false,
  ).deviceRegistryService;

  DeviceProfileService get _profileService =>
      Provider.of<ServiceRegistry>(context, listen: false).deviceProfileService;

  @override
  void initState() {
    super.initState();
    _loadDevices();
  }

  Future<void> _loadDevices() async {
    setState(() {
      _isLoading = true;
      _errorMessage = null;
    });

    try {
      final devices = await _deviceRegistry.getDevices();
      setState(() {
        _devices = devices;
        _isLoading = false;

        // Auto-select first device if none selected
        if (_selectedDevice == null && devices.isNotEmpty) {
          _selectDevice(devices.first);
        } else if (_selectedDevice != null) {
          // Refresh selected device
          final updated = devices
              .where(
                (d) => d.identity.toKey() == _selectedDevice!.identity.toKey(),
              )
              .firstOrNull;
          if (updated != null) {
            _selectedDevice = updated;
          }
        }
      });
    } catch (e) {
      setState(() {
        _isLoading = false;
        _errorMessage = 'Failed to load devices: $e';
      });
    }
  }

  Future<void> _selectDevice(DeviceState device) async {
    setState(() {
      _selectedDevice = device;
      _profile = null;
      _isLoading = true;
    });

    try {
      final result = await _profileService.getProfile(
        device.identity.vendorId,
        device.identity.productId,
      );

      setState(() {
        _isLoading = false;
        if (result.isSuccess) {
          _profile = result.profile;
        } else {
          _errorMessage = result.errorMessage;
        }
      });
    } catch (e) {
      setState(() {
        _isLoading = false;
        _errorMessage = 'Failed to load profile: $e';
      });
    }
  }

  Future<void> _saveMapping(int scanCode, int row, int col) async {
    if (_profile == null || _selectedDevice == null) return;

    // Update keymap
    final updatedKeymap = Map<int, PhysicalKey>.from(_profile!.keymap);
    updatedKeymap[scanCode] = PhysicalKey(
      scanCode: scanCode,
      row: row,
      col: col,
    );

    final updatedProfile = DeviceProfile(
      schemaVersion: _profile!.schemaVersion,
      vendorId: _profile!.vendorId,
      productId: _profile!.productId,
      name: _profile!.name,
      discoveredAt: _profile!.discoveredAt,
      rows: _profile!.rows,
      colsPerRow: _profile!.colsPerRow,
      keymap: updatedKeymap,
      aliases: _profile!.aliases,
      source: _profile!.source,
    );

    await _profileService.saveProfile(updatedProfile);

    setState(() {
      _profile = updatedProfile;
      _selectedPhysicalKeyId = null; // Close palette
    });

    if (mounted) {
      ScaffoldMessenger.of(context).showSnackBar(
        const SnackBar(
          content: Text('Key mapped'),
          duration: Duration(milliseconds: 1000),
          behavior: SnackBarBehavior.floating,
        ),
      );
    }
  }

  Future<void> _clearMapping(int scanCode) async {
    if (_profile == null) return;

    final updatedKeymap = Map<int, PhysicalKey>.from(_profile!.keymap);
    updatedKeymap.remove(scanCode);

    final updatedProfile = DeviceProfile(
      schemaVersion: _profile!.schemaVersion,
      vendorId: _profile!.vendorId,
      productId: _profile!.productId,
      name: _profile!.name,
      discoveredAt: _profile!.discoveredAt,
      rows: _profile!.rows,
      colsPerRow: _profile!.colsPerRow,
      keymap: updatedKeymap,
      aliases: _profile!.aliases,
      source: _profile!.source,
    );

    await _profileService.saveProfile(updatedProfile);

    setState(() {
      _profile = updatedProfile;
      _selectedPhysicalKeyId = null;
    });
  }

  void _handlePhysicalKeyTap(KeyDefinition key) {
    if (keyIdToWindowsScanCode.containsKey(key.id)) {
      setState(() {
        _selectedPhysicalKeyId = key.id;
      });
    } else {
      ScaffoldMessenger.of(context).showSnackBar(
        SnackBar(content: Text('No scancode defined for ${key.label}')),
      );
    }
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        title: const Text('Profiles (Device Wiring)'),
        actions: [
          IconButton(icon: const Icon(Icons.refresh), onPressed: _loadDevices),
        ],
      ),
      body: Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          // Toolbar: Device Selector
          _buildToolbar(),

          if (_errorMessage != null)
            Padding(
              padding: const EdgeInsets.all(8.0),
              child: InlineMessage(
                message: _errorMessage!,
                variant: InlineMessageVariant.error,
              ),
            ),

          // Content
          Expanded(child: _buildContent()),

          // Palette
          _buildMatrixPalette(),
        ],
      ),
    );
  }

  Widget _buildToolbar() {
    return Container(
      padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 8),
      color: Theme.of(context).colorScheme.surfaceContainerHighest,
      child: Row(
        children: [
          const Icon(Icons.keyboard_outlined),
          const SizedBox(width: 12),
          const Text('Device:', style: TextStyle(fontWeight: FontWeight.w500)),
          const SizedBox(width: 12),
          Expanded(
            child: DropdownButton<String>(
              value: _selectedDevice?.identity.toKey(),
              hint: const Text('Select a device'),
              isExpanded: true,
              underline: Container(),
              items: _devices.map((d) {
                return DropdownMenuItem(
                  value: d.identity.toKey(),
                  child: Text(d.identity.displayName),
                );
              }).toList(),
              onChanged: (key) {
                if (key != null) {
                  final device = _devices.firstWhere(
                    (d) => d.identity.toKey() == key,
                  );
                  _selectDevice(device);
                }
              },
            ),
          ),
        ],
      ),
    );
  }

  Widget _buildContent() {
    if (_isLoading) return const Center(child: CircularProgressIndicator());
    if (_devices.isEmpty)
      return const Center(
        child: Text('No devices found. Connect a supported device.'),
      );
    if (_selectedDevice == null)
      return const Center(child: Text('Select a device to configure.'));
    if (_profile == null)
      return const Center(child: Text('No profile found for this device.'));

    return _buildPhysicalKeyboard();
  }

  Widget _buildPhysicalKeyboard() {
    final mappedScanCodes = _profile!.keymap.keys.toSet();
    final mappedKeyIds = <String>{};

    for (final entry in keyIdToWindowsScanCode.entries) {
      if (mappedScanCodes.contains(entry.value)) {
        mappedKeyIds.add(entry.key);
      }
    }

    return LayoutBuilder(
      builder: (context, constraints) {
        return Padding(
          padding: const EdgeInsets.all(16.0),
          child: Column(
            children: [
              Text(
                'Typical Key Layout (Physical)',
                style: Theme.of(context).textTheme.titleMedium,
              ),
              const SizedBox(height: 4),
              Text(
                'Tap a key to assign its matrix position (Row/Col).',
                style: Theme.of(context).textTheme.bodySmall,
              ),
              const SizedBox(height: 16),
              Expanded(
                child: Center(
                  child: VisualKeyboard(
                    layout: KeyboardLayout.ansi104(),
                    onKeyTap: _handlePhysicalKeyTap,
                    selectedKeys: _selectedPhysicalKeyId != null
                        ? {_selectedPhysicalKeyId!}
                        : {},
                    mappedKeys: mappedKeyIds,
                    showMappingOverlay: false,
                    enableDragDrop: false,
                  ),
                ),
              ),
            ],
          ),
        );
      },
    );
  }

  Widget _buildMatrixPalette() {
    final show = _selectedPhysicalKeyId != null;
    final theme = Theme.of(context);

    int? selectedScanCode;
    PhysicalKey? currentMapping;

    if (_selectedPhysicalKeyId != null) {
      selectedScanCode = keyIdToWindowsScanCode[_selectedPhysicalKeyId];
      if (selectedScanCode != null && _profile != null) {
        currentMapping = _profile!.keymap[selectedScanCode];
      }
    }

    return AnimatedContainer(
      duration: const Duration(milliseconds: 300),
      height: show ? 350 : 0,
      curve: Curves.easeInOutCubicEmphasized,
      decoration: BoxDecoration(
        color: theme.colorScheme.surface,
        boxShadow: [
          BoxShadow(
            color: Colors.black.withOpacity(0.1),
            blurRadius: 8,
            offset: const Offset(0, -2),
          ),
        ],
        borderRadius: const BorderRadius.vertical(top: Radius.circular(16)),
      ),
      child: ClipRRect(
        borderRadius: const BorderRadius.vertical(top: Radius.circular(16)),
        child: show
            ? Column(
                crossAxisAlignment: CrossAxisAlignment.stretch,
                children: [
                  Container(
                    padding: const EdgeInsets.symmetric(
                      horizontal: 16,
                      vertical: 12,
                    ),
                    color: theme.colorScheme.surfaceContainerHighest,
                    child: Row(
                      children: [
                        Icon(Icons.grid_4x4, color: theme.colorScheme.primary),
                        const SizedBox(width: 12),
                        Column(
                          crossAxisAlignment: CrossAxisAlignment.start,
                          children: [
                            Text(
                              'Select Matrix Position for $_selectedPhysicalKeyId',
                              style: theme.textTheme.titleSmall,
                            ),
                            if (currentMapping != null)
                              Text(
                                'Currently: R${currentMapping.row}C${currentMapping.col}',
                                style: theme.textTheme.bodySmall,
                              ),
                          ],
                        ),
                        const Spacer(),
                        if (currentMapping != null)
                          TextButton.icon(
                            onPressed: () => _clearMapping(selectedScanCode!),
                            icon: const Icon(Icons.clear, size: 18),
                            label: const Text('Unmap'),
                          ),
                        IconButton(
                          icon: const Icon(Icons.close),
                          onPressed: () =>
                              setState(() => _selectedPhysicalKeyId = null),
                        ),
                      ],
                    ),
                  ),
                  Expanded(
                    child: SingleChildScrollView(
                      padding: const EdgeInsets.all(16),
                      child: _buildMatrixGrid(selectedScanCode),
                    ),
                  ),
                ],
              )
            : const SizedBox.shrink(),
      ),
    );
  }

  Widget _buildMatrixGrid(int? scanCode) {
    if (_profile == null) return const SizedBox.shrink();

    final rows = <Widget>[];
    for (int r = 0; r < _profile!.rows; r++) {
      final cols = _profile!.colsPerRow.length > r
          ? _profile!.colsPerRow[r]
          : 0;
      final buttons = <Widget>[];

      for (int c = 0; c < cols; c++) {
        final isMapped = _profile!.keymap.values.any(
          (pk) => pk.row == r && pk.col == c && pk.scanCode != scanCode,
        );

        buttons.add(
          Padding(
            padding: const EdgeInsets.all(4.0),
            child: ActionChip(
              label: Text('R${r}C$c'),
              backgroundColor: isMapped
                  ? Theme.of(
                      context,
                    ).colorScheme.surfaceContainerHighest.withOpacity(0.5)
                  : null,
              onPressed: scanCode == null
                  ? null
                  : () => _saveMapping(scanCode, r, c),
            ),
          ),
        );
      }

      rows.add(
        Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Padding(
              padding: const EdgeInsets.only(bottom: 8.0, top: 8.0),
              child: Text(
                'Row $r',
                style: Theme.of(context).textTheme.labelMedium,
              ),
            ),
            Wrap(children: buttons),
          ],
        ),
      );
    }

    return Column(crossAxisAlignment: CrossAxisAlignment.start, children: rows);
  }
}
