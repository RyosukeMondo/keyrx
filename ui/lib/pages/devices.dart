/// Devices tab for runtime profile slot management.
library;

import 'dart:math' as math;

import 'package:flutter/material.dart';

import '../models/device_state.dart';
import '../models/device_identity.dart';
import '../models/hardware_profile.dart';
import '../models/keymap.dart';
import '../models/runtime_config.dart';
import '../services/config_result.dart';
import '../services/device_registry_service.dart';
import '../services/hardware_service.dart';
import '../services/keymap_service.dart';
import '../services/runtime_service.dart';

class DevicesPage extends StatefulWidget {
  const DevicesPage({
    super.key,
    required this.runtimeService,
    required this.hardwareService,
    required this.keymapService,
    required this.deviceRegistryService,
  });

  final RuntimeService runtimeService;
  final HardwareService hardwareService;
  final KeymapService keymapService;
  final DeviceRegistryService deviceRegistryService;

  @override
  State<DevicesPage> createState() => _DevicesPageState();
}

class _DevicesPageState extends State<DevicesPage> {
  RuntimeConfig _runtime = const RuntimeConfig();
  List<HardwareProfile> _hardwareProfiles = const [];
  List<Keymap> _keymaps = const [];
  final Map<String, DeviceState> _registryDevices = {};
  final Set<String> _busySlots = {};

  bool _isLoading = true;
  bool _isMutating = false;
  String? _error;

  @override
  void initState() {
    super.initState();
    _loadAll();
  }

  Future<void> _loadAll() async {
    setState(() {
      _isLoading = true;
      _error = null;
    });

    try {
      final runtimeResult = await widget.runtimeService.getConfig();
      final hardwareResult = await widget.hardwareService.listProfiles();
      final keymapResult = await widget.keymapService.listKeymaps();

      List<DeviceState> devices = const [];
      try {
        devices = await widget.deviceRegistryService.getDevices();
      } catch (e) {
        _error = 'Device discovery failed: $e';
      }

      if (!mounted) return;

      final errors = [
        if (runtimeResult.hasError) runtimeResult.errorMessage,
        if (hardwareResult.hasError) hardwareResult.errorMessage,
        if (keymapResult.hasError) keymapResult.errorMessage,
        if (_error != null) _error,
      ].whereType<String>().toList();

      setState(() {
        _runtime = runtimeResult.data ?? const RuntimeConfig();
        _hardwareProfiles = hardwareResult.data ?? const [];
        _keymaps = keymapResult.data ?? const [];
        _registryDevices
          ..clear()
          ..addEntries(devices.map((d) => MapEntry(d.identity.toKey(), d)));
        _error = errors.isEmpty ? null : errors.join(' • ');
        _isLoading = false;
      });
    } catch (e) {
      if (!mounted) return;
      setState(() {
        _isLoading = false;
        _error = 'Failed to load runtime: $e';
      });
    }
  }

  Future<void> _mutate(
    Future<ConfigOperationResult<RuntimeConfig>> Function() action, {
    String? slotId,
  }) async {
    ConfigOperationResult<RuntimeConfig>? result;
    setState(() {
      _isMutating = true;
      if (slotId != null) {
        _busySlots.add(slotId);
      }
    });

    try {
      result = await action();
    } catch (e) {
      result = ConfigOperationResult.error(e.toString());
    } finally {
      if (mounted) {
        setState(() {
          _isMutating = false;
          if (slotId != null) {
            _busySlots.remove(slotId);
          }
        });
      }
    }

    if (!mounted) return;

    _applyResult(result);
  }

  void _applyResult(ConfigOperationResult<RuntimeConfig> result) {
    if (result.hasError) {
      _showSnack('Operation failed: ${result.errorMessage}', isError: true);
      return;
    }
    if (result.data != null) {
      setState(() {
        _runtime = result.data!;
      });
    }
  }

  DeviceState? _findRegistryDevice(DeviceInstanceId device) {
    for (final entry in _registryDevices.values) {
      final identity = entry.identity;
      final serialMatches =
          device.serial == null ||
          device.serial!.isEmpty ||
          identity.serialNumber == device.serial;
      if (identity.vendorId == device.vendorId &&
          identity.productId == device.productId &&
          serialMatches) {
        return entry;
      }
    }
    return null;
  }

  String _deviceTitle(DeviceSlots device) {
    final registry = _findRegistryDevice(device.device);
    if (registry?.identity.userLabel != null &&
        registry!.identity.userLabel!.isNotEmpty) {
      return registry.identity.userLabel!;
    }
    final vid = device.device.vendorId.toRadixString(16).padLeft(4, '0');
    final pid = device.device.productId.toRadixString(16).padLeft(4, '0');
    final serial = device.device.serial;
    if (serial != null && serial.isNotEmpty) {
      return '0x$vid:0x$pid ($serial)';
    }
    return '0x$vid:0x$pid';
  }

  String _deviceSubTitle(DeviceSlots device) {
    final registry = _findRegistryDevice(device.device);
    final serial = registry?.identity.serialNumber ?? device.device.serial;
    final key = _deviceKey(device.device);
    if (serial == null || serial.isEmpty) {
      return 'Key: $key';
    }
    return 'Serial: $serial • Key: $key';
  }

  String _deviceKey(DeviceInstanceId device) {
    final vid = device.vendorId.toRadixString(16).padLeft(4, '0');
    final pid = device.productId.toRadixString(16).padLeft(4, '0');
    final serial = device.serial;
    if (serial == null || serial.isEmpty) {
      return '$vid:$pid';
    }
    return '$vid:$pid:$serial';
  }

  List<ProfileSlot> _orderedSlots(DeviceSlots device) {
    final slots = [...device.slots];
    slots.sort((a, b) => b.priority.compareTo(a.priority));
    return slots;
  }

  List<HardwareProfile> _hardwareOptionsFor(DeviceSlots device) {
    final filtered = _hardwareProfiles.where((profile) {
      return profile.vendorId == device.device.vendorId &&
          profile.productId == device.device.productId;
    }).toList();
    return filtered.isEmpty ? _hardwareProfiles : filtered;
  }

  List<DeviceSlots> get _mergedDevices {
    final knownDevices = _runtime.devices.toList();
    final knownKeys = knownDevices.map((d) => _deviceKey(d.device)).toSet();

    for (final entry in _registryDevices.values) {
      final identity = entry.identity;
      final instanceId = DeviceInstanceId(
        vendorId: identity.vendorId,
        productId: identity.productId,
        serial: identity.serialNumber,
      );
      final key = _deviceKey(instanceId);

      if (!knownKeys.contains(key)) {
        knownDevices.add(DeviceSlots(device: instanceId, slots: []));
      }
    }
    return knownDevices;
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        title: const Text('Devices'),
        actions: [
          IconButton(
            icon: const Icon(Icons.add),
            tooltip: 'Add Virtual Device',
            onPressed: _isMutating ? null : _showAddVirtualDeviceDialog,
          ),
          IconButton(
            icon: const Icon(Icons.refresh),
            tooltip: 'Reload runtime',
            onPressed: _isMutating ? null : _loadAll,
          ),
        ],
      ),
      body: RefreshIndicator(onRefresh: _loadAll, child: _buildBody()),
    );
  }

  Widget _buildBody() {
    if (_isLoading) {
      return const Center(child: CircularProgressIndicator());
    }

    final devices = _mergedDevices;

    if (devices.isEmpty) {
      return ListView(
        physics: const AlwaysScrollableScrollPhysics(),
        padding: const EdgeInsets.all(24),
        children: [
          if (_error != null) _InlineErrorBanner(message: _error!),
          _EmptyState(onRefresh: _loadAll),
        ],
      );
    }

    return ListView(
      physics: const AlwaysScrollableScrollPhysics(),
      padding: const EdgeInsets.all(12),
      children: [
        if (_error != null) ...[
          _InlineErrorBanner(message: _error!),
          const SizedBox(height: 12),
        ],
        ...devices.map((device) => _buildDeviceCard(device)),
        const SizedBox(height: 12),
      ],
    );
  }

  Widget _buildDeviceCard(DeviceSlots device) {
    final slots = _orderedSlots(device);
    final hardwareOptions = _hardwareOptionsFor(device);
    final hasWiring = hardwareOptions.isNotEmpty;
    final hasKeymaps = _keymaps.isNotEmpty;
    final disabled = _isMutating;

    return Card(
      margin: const EdgeInsets.symmetric(vertical: 8),
      child: Padding(
        padding: const EdgeInsets.all(16),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Row(
              children: [
                Icon(
                  Icons.keyboard,
                  color: Theme.of(context).colorScheme.primary,
                ),
                const SizedBox(width: 12),
                Expanded(
                  child: Column(
                    crossAxisAlignment: CrossAxisAlignment.start,
                    children: [
                      Row(
                        children: [
                          Flexible(
                            child: Text(
                              _deviceTitle(device),
                              style: Theme.of(context).textTheme.titleMedium
                                  ?.copyWith(fontWeight: FontWeight.bold),
                            ),
                          ),
                          IconButton(
                            icon: const Icon(Icons.edit, size: 16),
                            tooltip: 'Rename device',
                            onPressed: disabled
                                ? null
                                : () => _showRenameDialog(device),
                          ),
                        ],
                      ),
                      const SizedBox(height: 4),
                      Text(
                        _deviceSubTitle(device),
                        style: Theme.of(context).textTheme.bodySmall,
                      ),
                    ],
                  ),
                ),
                FilledButton.icon(
                  onPressed: disabled || !hasWiring || !hasKeymaps
                      ? null
                      : () => _addSlot(device),
                  icon: const Icon(Icons.add),
                  label: const Text('Add Slot'),
                ),
              ],
            ),
            const SizedBox(height: 8),
            // Remap Toggle Row
            Row(
              children: [
                const Spacer(),
                const Text('Remap Enabled'),
                const SizedBox(width: 8),
                Switch(
                  value:
                      _findRegistryDevice(device.device)?.remapEnabled ?? false,
                  onChanged: disabled
                      ? null
                      : (value) async {
                          final registryDevice = _findRegistryDevice(
                            device.device,
                          );
                          if (registryDevice != null) {
                            await widget.deviceRegistryService.toggleRemap(
                              registryDevice.identity.toKey(),
                              value,
                            );
                            _loadAll();
                          }
                        },
                ),
              ],
            ),
            const SizedBox(height: 12),
            if (!hasWiring || !hasKeymaps)
              Padding(
                padding: const EdgeInsets.only(bottom: 12),
                child: _InlineInfoBanner(
                  message: !hasWiring
                      ? 'Create a wiring profile in the Wiring tab before adding slots.'
                      : 'Create a keymap in the Mapping tab before adding slots.',
                ),
              ),
            if (slots.isEmpty)
              Padding(
                padding: const EdgeInsets.symmetric(vertical: 8),
                child: Text(
                  'No profile slots yet. Add at least one wiring + keymap pair for this device.',
                  style: Theme.of(context).textTheme.bodyMedium,
                ),
              )
            else
              Column(
                children: [
                  for (var i = 0; i < slots.length; i++)
                    _buildSlotCard(
                      device: device,
                      slot: slots[i],
                      index: i,
                      total: slots.length,
                      hardwareOptions: hardwareOptions,
                    ),
                ],
              ),
          ],
        ),
      ),
    );
  }

  Widget _buildSlotCard({
    required DeviceSlots device,
    required ProfileSlot slot,
    required int index,
    required int total,
    required List<HardwareProfile> hardwareOptions,
  }) {
    final slotBusy = _busySlots.contains(slot.id) || _isMutating;
    final hardware = hardwareOptions.isNotEmpty
        ? hardwareOptions.firstWhere(
            (h) => h.id == slot.hardwareProfileId,
            orElse: () => hardwareOptions.first,
          )
        : null;
    final hasKeymap = _keymaps.any((k) => k.id == slot.keymapId);
    final keymapValue = hasKeymap
        ? slot.keymapId
        : (_keymaps.isNotEmpty ? _keymaps.first.id : null);

    return Container(
      margin: const EdgeInsets.symmetric(vertical: 6),
      decoration: BoxDecoration(
        borderRadius: BorderRadius.circular(12),
        color: Theme.of(context).colorScheme.surfaceContainerHighest,
        border: Border.all(
          color: slot.active
              ? Theme.of(context).colorScheme.primary
              : Theme.of(context).dividerColor,
        ),
      ),
      child: Padding(
        padding: const EdgeInsets.all(12),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Row(
              children: [
                Text(
                  'Slot ${index + 1}',
                  style: Theme.of(context).textTheme.titleSmall,
                ),
                const SizedBox(width: 8),
                Chip(
                  label: Text('Priority ${slot.priority}'),
                  padding: EdgeInsets.zero,
                  materialTapTargetSize: MaterialTapTargetSize.shrinkWrap,
                ),
                const Spacer(),
                IconButton(
                  tooltip: 'Move up',
                  onPressed: slotBusy || index == 0
                      ? null
                      : () => _moveSlot(device, slot, -1),
                  icon: const Icon(Icons.arrow_upward),
                ),
                IconButton(
                  tooltip: 'Move down',
                  onPressed: slotBusy || index == total - 1
                      ? null
                      : () => _moveSlot(device, slot, 1),
                  icon: const Icon(Icons.arrow_downward),
                ),
                Switch(
                  value: slot.active,
                  onChanged: slotBusy
                      ? null
                      : (value) => _mutate(
                          () => widget.runtimeService.setSlotActive(
                            device.device,
                            slot.id,
                            value,
                          ),
                          slotId: slot.id,
                        ),
                ),
              ],
            ),
            const SizedBox(height: 8),
            Row(
              children: [
                Expanded(
                  child: DropdownButtonFormField<String>(
                    initialValue: hardware?.id,
                    isExpanded: true,
                    decoration: const InputDecoration(
                      labelText: 'Wiring (Hardware Profile)',
                      border: OutlineInputBorder(),
                    ),
                    items: hardwareOptions
                        .map(
                          (h) => DropdownMenuItem(
                            value: h.id,
                            child: Text(h.name ?? h.id),
                          ),
                        )
                        .toList(),
                    onChanged: slotBusy || hardwareOptions.isEmpty
                        ? null
                        : (value) {
                            if (value != null &&
                                value != slot.hardwareProfileId) {
                              _mutate(
                                () => widget.runtimeService.addSlot(
                                  device.device,
                                  slot.copyWith(hardwareProfileId: value),
                                ),
                                slotId: slot.id,
                              );
                            }
                          },
                  ),
                ),
                const SizedBox(width: 12),
                Expanded(
                  child: DropdownButtonFormField<String>(
                    initialValue: keymapValue,
                    isExpanded: true,
                    decoration: const InputDecoration(
                      labelText: 'Keymap',
                      border: OutlineInputBorder(),
                    ),
                    items: _keymaps
                        .map(
                          (k) => DropdownMenuItem(
                            value: k.id,
                            child: Text(k.name),
                          ),
                        )
                        .toList(),
                    onChanged: slotBusy || _keymaps.isEmpty
                        ? null
                        : (value) {
                            if (value != null && value != slot.keymapId) {
                              _mutate(
                                () => widget.runtimeService.addSlot(
                                  device.device,
                                  slot.copyWith(keymapId: value),
                                ),
                                slotId: slot.id,
                              );
                            }
                          },
                  ),
                ),
              ],
            ),
            const SizedBox(height: 12),
            Align(
              alignment: Alignment.centerRight,
              child: TextButton.icon(
                onPressed: slotBusy
                    ? null
                    : () => _mutate(
                        () => widget.runtimeService.removeSlot(
                          device.device,
                          slot.id,
                        ),
                        slotId: slot.id,
                      ),
                icon: const Icon(Icons.delete),
                label: const Text('Remove Slot'),
              ),
            ),
          ],
        ),
      ),
    );
  }

  Future<void> _addSlot(DeviceSlots device) async {
    final hardwareOptions = _hardwareOptionsFor(device);
    if (hardwareOptions.isEmpty) {
      _showSnack(
        'Add a wiring profile for this device in the Wiring tab before creating a slot.',
        isError: true,
      );
      return;
    }
    if (_keymaps.isEmpty) {
      _showSnack(
        'Create a keymap in the Mapping tab before creating a slot.',
        isError: true,
      );
      return;
    }

    HardwareProfile selectedHardware = hardwareOptions.first;
    Keymap selectedKeymap = _keymaps.first;

    final confirmed = await showDialog<bool>(
      context: context,
      builder: (context) {
        return StatefulBuilder(
          builder: (context, setDialogState) {
            return AlertDialog(
              title: const Text('Add Profile Slot'),
              content: Column(
                mainAxisSize: MainAxisSize.min,
                children: [
                  DropdownButtonFormField<String>(
                    initialValue: selectedHardware.id,
                    decoration: const InputDecoration(
                      labelText: 'Wiring (Hardware Profile)',
                      border: OutlineInputBorder(),
                    ),
                    items: hardwareOptions
                        .map(
                          (h) => DropdownMenuItem(
                            value: h.id,
                            child: Text(h.name ?? h.id),
                          ),
                        )
                        .toList(),
                    onChanged: (value) {
                      if (value == null) return;
                      setDialogState(() {
                        selectedHardware = hardwareOptions.firstWhere(
                          (h) => h.id == value,
                        );
                      });
                    },
                  ),
                  const SizedBox(height: 12),
                  DropdownButtonFormField<String>(
                    initialValue: selectedKeymap.id,
                    decoration: const InputDecoration(
                      labelText: 'Keymap',
                      border: OutlineInputBorder(),
                    ),
                    items: _keymaps
                        .map(
                          (k) => DropdownMenuItem(
                            value: k.id,
                            child: Text(k.name),
                          ),
                        )
                        .toList(),
                    onChanged: (value) {
                      if (value == null) return;
                      setDialogState(() {
                        selectedKeymap = _keymaps.firstWhere(
                          (k) => k.id == value,
                        );
                      });
                    },
                  ),
                ],
              ),
              actions: [
                TextButton(
                  onPressed: () => Navigator.of(context).pop(false),
                  child: const Text('Cancel'),
                ),
                FilledButton(
                  onPressed: () => Navigator.of(context).pop(true),
                  child: const Text('Add'),
                ),
              ],
            );
          },
        );
      },
    );

    if (confirmed != true) return;

    final nextPriority = _nextPriority(device.slots);
    final slotId =
        'slot-${_deviceKey(device.device).replaceAll(':', '-')}-${DateTime.now().millisecondsSinceEpoch}';

    final newSlot = ProfileSlot(
      id: slotId,
      hardwareProfileId: selectedHardware.id,
      keymapId: selectedKeymap.id,
      active: true,
      priority: nextPriority,
    );

    await _mutate(
      () => widget.runtimeService.addSlot(device.device, newSlot),
      slotId: slotId,
    );
  }

  int _nextPriority(List<ProfileSlot> slots) {
    if (slots.isEmpty) return 1;
    final highest = slots.map((s) => s.priority).reduce(math.max);
    return highest + 1;
  }

  Future<void> _moveSlot(
    DeviceSlots device,
    ProfileSlot slot,
    int delta,
  ) async {
    setState(() {
      _isMutating = true;
      _busySlots.add(slot.id);
    });

    try {
      final ordered = _orderedSlots(device);
      final currentIndex = ordered.indexWhere((s) => s.id == slot.id);
      if (currentIndex == -1) return;

      final newIndex = (currentIndex + delta).clamp(0, ordered.length - 1);
      if (newIndex == currentIndex) return;

      final reordered = List<ProfileSlot>.from(ordered)
        ..removeAt(currentIndex)
        ..insert(newIndex, slot);

      ConfigOperationResult<RuntimeConfig>? lastResult;
      for (var i = 0; i < reordered.length; i++) {
        final priority = reordered.length - i;
        lastResult = await widget.runtimeService.reorderSlot(
          device.device,
          reordered[i].id,
          priority,
        );
        if (lastResult.hasError) {
          _showSnack(
            'Failed to reorder: ${lastResult.errorMessage}',
            isError: true,
          );
          await _loadAll();
          return;
        }
      }

      if (lastResult?.data != null) {
        setState(() {
          _runtime = lastResult!.data!;
        });
      }
    } finally {
      if (mounted) {
        setState(() {
          _isMutating = false;
          _busySlots.remove(slot.id);
        });
      }
    }
  }

  Future<void> _showAddVirtualDeviceDialog() async {
    final vidController = TextEditingController();
    final pidController = TextEditingController();
    final serialController = TextEditingController();
    final labelController = TextEditingController();

    final identity = await showDialog<DeviceIdentity>(
      context: context,
      builder: (context) {
        return AlertDialog(
          title: const Text('Add Virtual Device'),
          content: SingleChildScrollView(
            child: Column(
              mainAxisSize: MainAxisSize.min,
              children: [
                const Text(
                  'Enter device details to create a virtual hardware profile.',
                  style: TextStyle(fontSize: 13),
                ),
                const SizedBox(height: 16),
                Row(
                  children: [
                    Expanded(
                      child: TextField(
                        controller: vidController,
                        decoration: const InputDecoration(
                          labelText: 'Vendor ID (Hex)',
                          hintText: 'e.g. 046D',
                          isDense: true,
                          border: OutlineInputBorder(),
                        ),
                      ),
                    ),
                    const SizedBox(width: 12),
                    Expanded(
                      child: TextField(
                        controller: pidController,
                        decoration: const InputDecoration(
                          labelText: 'Product ID (Hex)',
                          hintText: 'e.g. C52B',
                          isDense: true,
                          border: OutlineInputBorder(),
                        ),
                      ),
                    ),
                  ],
                ),
                const SizedBox(height: 12),
                TextField(
                  controller: serialController,
                  decoration: const InputDecoration(
                    labelText: 'Serial Number',
                    hintText: 'e.g. SN12345678',
                    isDense: true,
                    border: OutlineInputBorder(),
                  ),
                ),
                const SizedBox(height: 12),
                TextField(
                  controller: labelController,
                  decoration: const InputDecoration(
                    labelText: 'Label (Optional)',
                    hintText: 'My Custom Keyboard',
                    isDense: true,
                    border: OutlineInputBorder(),
                  ),
                ),
              ],
            ),
          ),
          actions: [
            TextButton(
              onPressed: () => Navigator.of(context).pop(),
              child: const Text('Cancel'),
            ),
            FilledButton(
              onPressed: () {
                // Basic validation
                final vidStr = vidController.text.trim();
                final pidStr = pidController.text.trim();
                final serial = serialController.text.trim();

                if (vidStr.isEmpty || pidStr.isEmpty || serial.isEmpty) {
                  return; // Should probably show error but simple return for now
                }

                int? vid = int.tryParse(vidStr, radix: 16);
                int? pid = int.tryParse(pidStr, radix: 16);

                if (vid == null || pid == null) {
                  return;
                }

                Navigator.of(context).pop(
                  DeviceIdentity(
                    vendorId: vid,
                    productId: pid,
                    serialNumber: serial,
                    userLabel: labelController.text.trim().isEmpty
                        ? null
                        : labelController.text.trim(),
                  ),
                );
              },
              child: const Text('Add Device'),
            ),
          ],
        );
      },
    );

    if (identity == null) return;

    setState(() => _isMutating = true);
    try {
      await widget.deviceRegistryService.addVirtualDevice(identity);
      await _loadAll();
      _showSnack('Virtual device added', isError: false);
    } catch (e) {
      _showSnack('Failed to add virtual device: $e', isError: true);
    } finally {
      if (mounted) {
        setState(() => _isMutating = false);
      }
    }
  }

  Future<void> _showRenameDialog(DeviceSlots device) async {
    final registry = _findRegistryDevice(device.device);
    final key = registry?.identity.toKey() ?? _deviceKey(device.device);
    final currentLabel = registry?.identity.userLabel ?? '';
    final controller = TextEditingController(text: currentLabel);

    final newLabel = await showDialog<String>(
      context: context,
      builder: (context) {
        return AlertDialog(
          title: const Text('Rename Device'),
          content: TextField(
            controller: controller,
            decoration: const InputDecoration(
              labelText: 'Device Name',
              hintText: 'Enter a custom name for this device',
            ),
            autofocus: true,
          ),
          actions: [
            TextButton(
              onPressed: () => Navigator.of(context).pop(),
              child: const Text('Cancel'),
            ),
            FilledButton(
              onPressed: () =>
                  Navigator.of(context).pop(controller.text.trim()),
              child: const Text('Save'),
            ),
          ],
        );
      },
    );

    if (newLabel == null || newLabel == currentLabel) return;

    setState(() {
      _isMutating = true;
    });

    try {
      final result = await widget.deviceRegistryService.setUserLabel(
        key,
        newLabel.isEmpty ? null : newLabel,
      );

      if (!result.success) {
        _showSnack('Failed to rename: ${result.errorMessage}', isError: true);
      } else {
        await _loadAll();
      }
    } finally {
      if (mounted) {
        setState(() {
          _isMutating = false;
        });
      }
    }
  }

  void _showSnack(String message, {bool isError = false}) {
    ScaffoldMessenger.of(context).showSnackBar(
      SnackBar(
        content: Text(message),
        backgroundColor: isError
            ? Theme.of(context).colorScheme.errorContainer
            : null,
      ),
    );
  }
} // End of State

class _InlineErrorBanner extends StatelessWidget {
  const _InlineErrorBanner({required this.message});

  final String message;

  @override
  Widget build(BuildContext context) {
    return Material(
      color: Theme.of(context).colorScheme.errorContainer,
      borderRadius: BorderRadius.circular(12),
      child: Padding(
        padding: const EdgeInsets.all(12),
        child: Row(
          children: [
            Icon(
              Icons.error_outline,
              color: Theme.of(context).colorScheme.onErrorContainer,
            ),
            const SizedBox(width: 8),
            Expanded(
              child: Text(
                message,
                style: TextStyle(
                  color: Theme.of(context).colorScheme.onErrorContainer,
                ),
              ),
            ),
          ],
        ),
      ),
    );
  }
}

class _InlineInfoBanner extends StatelessWidget {
  const _InlineInfoBanner({required this.message});

  final String message;

  @override
  Widget build(BuildContext context) {
    return Container(
      width: double.infinity,
      decoration: BoxDecoration(
        color: Theme.of(context).colorScheme.surfaceContainerHighest,
        borderRadius: BorderRadius.circular(10),
      ),
      padding: const EdgeInsets.all(12),
      child: Row(
        children: [
          Icon(
            Icons.info_outline,
            color: Theme.of(context).colorScheme.onSurfaceVariant,
          ),
          const SizedBox(width: 8),
          Expanded(child: Text(message)),
        ],
      ),
    );
  }
}

class _EmptyState extends StatelessWidget {
  const _EmptyState({required this.onRefresh});

  final Future<void> Function() onRefresh;

  @override
  Widget build(BuildContext context) {
    return Column(
      mainAxisSize: MainAxisSize.min,
      children: [
        const SizedBox(height: 24),
        const Icon(Icons.devices_other_outlined, size: 64),
        const SizedBox(height: 16),
        Text(
          'No connected devices with runtime slots',
          style: Theme.of(context).textTheme.titleMedium,
          textAlign: TextAlign.center,
        ),
        const SizedBox(height: 8),
        Text(
          'Connect a device, then add wiring/keymap slots to control it.',
          style: Theme.of(context).textTheme.bodyMedium,
          textAlign: TextAlign.center,
        ),
        const SizedBox(height: 16),
        FilledButton.icon(
          onPressed: onRefresh,
          icon: const Icon(Icons.refresh),
          label: const Text('Refresh devices'),
        ),
      ],
    );
  }
}
