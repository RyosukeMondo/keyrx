/// KeyFlow configuration viewer screen.
///
/// Composes device list, wiring graph, key details, and runtime strip
/// using the KeyFlow view model for live, read-only inspection.
library;

import 'package:flutter/material.dart';
import 'package:provider/provider.dart';

import '../models/action_binding.dart';
import '../models/device_identity.dart';
import '../models/hardware_profile.dart';
import '../models/keymap.dart';
import '../models/runtime_config.dart';
import '../models/virtual_layout.dart';
import '../services/facade/keyflow_repository.dart';
import '../services/service_registry.dart';
import '../state/keyflow_view_model.dart';
import '../widgets/virtual_layout_renderer.dart';

class KeyFlowPage extends StatefulWidget {
  const KeyFlowPage({super.key});

  @override
  State<KeyFlowPage> createState() => _KeyFlowPageState();
}

class _KeyFlowPageState extends State<KeyFlowPage> {
  late final KeyFlowRepository _repository;
  late final KeyFlowViewModel _viewModel;

  @override
  void initState() {
    super.initState();
    final registry = context.read<ServiceRegistry>();
    _repository = KeyFlowRepository(
      deviceRegistryService: registry.deviceRegistryService,
      hardwareService: registry.hardwareService,
      layoutService: registry.layoutService,
      keymapService: registry.keymapService,
      runtimeService: registry.runtimeService,
      runtimeUpdates: registry.bridge.stateStream,
    );
    _viewModel = KeyFlowViewModel(repository: _repository);
    _viewModel.load();
  }

  @override
  void dispose() {
    _viewModel.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    return ChangeNotifierProvider<KeyFlowViewModel>.value(
      value: _viewModel,
      child: Scaffold(
        appBar: AppBar(
          title: const Text('KeyFlow'),
          actions: [
            Consumer<KeyFlowViewModel>(
              builder: (context, vm, _) {
                final busy = vm.isLoading || vm.isRefreshing;
                return IconButton(
                  tooltip: 'Refresh devices',
                  onPressed: busy ? null : vm.refresh,
                  icon: busy
                      ? const SizedBox(
                          height: 18,
                          width: 18,
                          child: CircularProgressIndicator(strokeWidth: 2),
                        )
                      : const Icon(Icons.refresh),
                );
              },
            ),
          ],
        ),
        body: Consumer<KeyFlowViewModel>(
          builder: (context, vm, _) {
            final snapshot = vm.snapshot;
            final devices = snapshot?.devices ?? const [];
            final selectedDevice = vm.selectedDevice;
            final selectedProfile = _resolveProfile(snapshot, selectedDevice);
            final selectedLayout = _resolveLayout(snapshot, selectedDevice);
            final selectedKeymap = _resolveKeymap(snapshot, selectedDevice);
            final selectedKeyId = vm.selectedKeyId;

            if (snapshot == null && vm.isLoading) {
              return const Center(child: CircularProgressIndicator());
            }

            return RefreshIndicator(
              onRefresh: vm.refresh,
              color: Theme.of(context).colorScheme.primary,
              child: ListView(
                physics: const AlwaysScrollableScrollPhysics(),
                padding: const EdgeInsets.all(16),
                children: [
                  _HeaderRow(
                    generatedAt: snapshot?.generatedAt,
                    totalGaps: vm.totalGaps,
                    totalConflicts: vm.totalConflicts,
                    isStale: vm.isRuntimeStale,
                    runtimeUpdatedAt: vm.lastRuntimeUpdate,
                    hasRuntime: vm.runtimeOverlay != null,
                  ),
                  if (vm.errors.isNotEmpty) ...[
                    const SizedBox(height: 8),
                    _ErrorBanner(errors: vm.errors),
                  ],
                  if (vm.isRuntimeStale) ...[
                    const SizedBox(height: 8),
                    _RuntimeBanner(lastUpdate: vm.lastRuntimeUpdate),
                  ],
                  const SizedBox(height: 12),
                  LayoutBuilder(
                    builder: (context, constraints) {
                      final isWide = constraints.maxWidth > 1100;
                      final devicePane = _DeviceListPane(
                        devices: devices,
                        selectedDeviceId: selectedDevice?.id,
                        onSelect: vm.selectDevice,
                        isLoading: vm.isLoading,
                        isRefreshing: vm.isRefreshing,
                      );
                      final graphPane = _FlowGraphPane(
                        device: selectedDevice,
                        profile: selectedProfile,
                        layout: selectedLayout,
                        onKeySelect: vm.selectKey,
                        selectedKeyId: selectedKeyId,
                      );

                      if (isWide) {
                        return Row(
                          crossAxisAlignment: CrossAxisAlignment.start,
                          children: [
                            Expanded(flex: 2, child: devicePane),
                            const SizedBox(width: 12),
                            Expanded(flex: 3, child: graphPane),
                          ],
                        );
                      }

                      return Column(
                        children: [
                          devicePane,
                          const SizedBox(height: 12),
                          graphPane,
                        ],
                      );
                    },
                  ),
                  const SizedBox(height: 12),
                  _DetailsPane(
                    device: selectedDevice,
                    profile: selectedProfile,
                    layout: selectedLayout,
                    keymap: selectedKeymap,
                    selectedKeyId: selectedKeyId,
                  ),
                  const SizedBox(height: 12),
                  _RuntimeStrip(
                    overlay: vm.runtimeOverlay,
                    isStale: vm.isRuntimeStale,
                    errors: vm.errors,
                  ),
                  const SizedBox(height: 16),
                ],
              ),
            );
          },
        ),
      ),
    );
  }

  HardwareProfile? _resolveProfile(
    KeyFlowSnapshot? snapshot,
    KeyFlowDevice? device,
  ) {
    if (snapshot == null || device?.profileId == null) return null;
    return snapshot.profiles[device!.profileId!];
  }

  VirtualLayout? _resolveLayout(
    KeyFlowSnapshot? snapshot,
    KeyFlowDevice? device,
  ) {
    if (snapshot == null || device?.layoutId == null) return null;
    return snapshot.layouts[device!.layoutId!];
  }

  Keymap? _resolveKeymap(KeyFlowSnapshot? snapshot, KeyFlowDevice? device) {
    if (snapshot == null || device == null || snapshot.runtimeConfig == null) {
      return null;
    }

    final identity = DeviceIdentity.fromKey(device.id);
    if (identity == null) return null;

    final deviceSlots = snapshot.runtimeConfig!.devices.firstWhere(
      (entry) =>
          entry.device.vendorId == identity.vendorId &&
          entry.device.productId == identity.productId &&
          (entry.device.serial == null ||
              entry.device.serial!.isEmpty ||
              entry.device.serial == identity.serialNumber),
      orElse: () => const DeviceSlots(
        device: DeviceInstanceId(vendorId: 0, productId: 0),
      ),
    );

    if (deviceSlots.device.vendorId == 0 && deviceSlots.device.productId == 0) {
      return null;
    }

    ProfileSlot? slot = deviceSlots.slots.firstWhere(
      (s) => s.active,
      orElse: () => deviceSlots.slots.isNotEmpty
          ? deviceSlots.slots.first
          : const ProfileSlot(id: '', hardwareProfileId: '', keymapId: ''),
    );

    if (slot.keymapId.isEmpty) return null;
    return snapshot.keymaps[slot.keymapId];
  }
}

class _HeaderRow extends StatelessWidget {
  const _HeaderRow({
    required this.generatedAt,
    required this.totalGaps,
    required this.totalConflicts,
    required this.isStale,
    required this.runtimeUpdatedAt,
    required this.hasRuntime,
  });

  final DateTime? generatedAt;
  final int totalGaps;
  final int totalConflicts;
  final bool isStale;
  final DateTime? runtimeUpdatedAt;
  final bool hasRuntime;

  @override
  Widget build(BuildContext context) {
    return Card(
      child: Padding(
        padding: const EdgeInsets.all(12),
        child: Row(
          children: [
            const Icon(Icons.account_tree_outlined),
            const SizedBox(width: 8),
            Expanded(
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  Text(
                    'KeyFlow snapshot',
                    style: Theme.of(context).textTheme.titleMedium,
                  ),
                  if (generatedAt != null)
                    Text(
                      'Generated at ${generatedAt!.toLocal()}',
                      style: Theme.of(context).textTheme.bodySmall,
                    ),
                ],
              ),
            ),
            Wrap(
              spacing: 12,
              runSpacing: 8,
              children: [
                _Chip(
                  icon: Icons.report_gmailerrorred_outlined,
                  label: '$totalGaps gaps',
                  color: totalGaps > 0
                      ? Theme.of(context).colorScheme.errorContainer
                      : Theme.of(context).colorScheme.surfaceVariant,
                ),
                _Chip(
                  icon: Icons.link_off_outlined,
                  label: '$totalConflicts conflicts',
                  color: totalConflicts > 0
                      ? Theme.of(context).colorScheme.errorContainer
                      : Theme.of(context).colorScheme.surfaceVariant,
                ),
                _Chip(
                  icon: isStale
                      ? Icons.cloud_off
                      : (hasRuntime ? Icons.cloud_done : Icons.cloud_queue),
                  label: isStale
                      ? 'Runtime stale'
                      : hasRuntime
                      ? 'Runtime live'
                      : 'Runtime unavailable',
                  color: isStale
                      ? Theme.of(context).colorScheme.errorContainer
                      : Theme.of(context).colorScheme.surfaceVariant,
                  tooltip: runtimeUpdatedAt != null
                      ? 'Last update ${runtimeUpdatedAt!.toLocal()}'
                      : null,
                ),
              ],
            ),
          ],
        ),
      ),
    );
  }
}

class _DeviceListPane extends StatelessWidget {
  const _DeviceListPane({
    required this.devices,
    required this.selectedDeviceId,
    required this.onSelect,
    required this.isLoading,
    required this.isRefreshing,
  });

  final List<KeyFlowDevice> devices;
  final String? selectedDeviceId;
  final ValueChanged<String?> onSelect;
  final bool isLoading;
  final bool isRefreshing;

  @override
  Widget build(BuildContext context) {
    return Card(
      child: Padding(
        padding: const EdgeInsets.all(12),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Row(
              children: [
                const Icon(Icons.devices_other_outlined),
                const SizedBox(width: 8),
                Text('Devices', style: Theme.of(context).textTheme.titleMedium),
                const Spacer(),
                if (isLoading) const _TinySpinner(label: 'Loading...'),
                if (isRefreshing && !isLoading)
                  const _TinySpinner(label: 'Refreshing...'),
              ],
            ),
            const SizedBox(height: 8),
            if (devices.isEmpty)
              Padding(
                padding: const EdgeInsets.all(12),
                child: Text(
                  isLoading
                      ? 'Discovering devices...'
                      : 'No devices detected. Connect a device or load mock data.',
                ),
              )
            else
              ListView.separated(
                shrinkWrap: true,
                physics: const NeverScrollableScrollPhysics(),
                itemCount: devices.length,
                separatorBuilder: (_, __) => const SizedBox(height: 8),
                itemBuilder: (context, index) {
                  final device = devices[index];
                  final selected = device.id == selectedDeviceId;
                  return ListTile(
                    shape: RoundedRectangleBorder(
                      borderRadius: BorderRadius.circular(8),
                      side: BorderSide(
                        color: selected
                            ? Theme.of(context).colorScheme.primary
                            : Theme.of(context).dividerColor,
                      ),
                    ),
                    selected: selected,
                    onTap: () => onSelect(device.id),
                    title: Text(device.displayName),
                    subtitle: Text(
                      [
                        device.remapEnabled ? 'Remap enabled' : 'Passthrough',
                        if (device.lastUpdated != null)
                          'Updated ${device.lastUpdated}',
                        if (device.profileId != null)
                          'Profile ${device.profileId}',
                      ].join(' | '),
                    ),
                    trailing: Wrap(
                      spacing: 6,
                      children: [
                        _Badge(
                          icon: Icons.report_problem_outlined,
                          label: '${device.gaps}',
                          color: device.gaps > 0
                              ? Theme.of(context).colorScheme.errorContainer
                              : Theme.of(context).colorScheme.surfaceVariant,
                          tooltip: 'Unmapped keys',
                        ),
                        _Badge(
                          icon: Icons.scatter_plot_outlined,
                          label: '${device.conflicts}',
                          color: device.conflicts > 0
                              ? Theme.of(context).colorScheme.errorContainer
                              : Theme.of(context).colorScheme.surfaceVariant,
                          tooltip: 'Duplicate mappings',
                        ),
                      ],
                    ),
                  );
                },
              ),
          ],
        ),
      ),
    );
  }
}

class _FlowGraphPane extends StatelessWidget {
  const _FlowGraphPane({
    required this.device,
    required this.profile,
    required this.layout,
    required this.onKeySelect,
    required this.selectedKeyId,
  });

  final KeyFlowDevice? device;
  final HardwareProfile? profile;
  final VirtualLayout? layout;
  final ValueChanged<String?> onKeySelect;
  final String? selectedKeyId;

  @override
  Widget build(BuildContext context) {
    final mappedKeys = profile?.wiring.values.toSet() ?? const <String>{};
    return Card(
      child: Padding(
        padding: const EdgeInsets.all(12),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Row(
              children: [
                const Icon(Icons.schema_outlined),
                const SizedBox(width: 8),
                Text(
                  'Flow graph',
                  style: Theme.of(context).textTheme.titleMedium,
                ),
                const Spacer(),
                if (device != null && device!.layoutId != null)
                  Text(
                    'Layout ${device!.layoutId}',
                    style: Theme.of(context).textTheme.bodySmall,
                  ),
              ],
            ),
            const SizedBox(height: 8),
            if (device == null)
              const Padding(
                padding: EdgeInsets.all(12),
                child: Text('Select a device to view wiring and layout.'),
              )
            else if (layout == null)
              const Padding(
                padding: EdgeInsets.all(12),
                child: Text('No layout found for the selected device.'),
              )
            else
              ConstrainedBox(
                constraints: const BoxConstraints(minHeight: 320),
                child: VirtualLayoutRenderer(
                  layout: layout!,
                  selectedKeyId: selectedKeyId,
                  mappedKeyIds: mappedKeys,
                  onKeyTap: onKeySelect,
                ),
              ),
          ],
        ),
      ),
    );
  }
}

class _DetailsPane extends StatelessWidget {
  const _DetailsPane({
    required this.device,
    required this.profile,
    required this.layout,
    required this.keymap,
    required this.selectedKeyId,
  });

  final KeyFlowDevice? device;
  final HardwareProfile? profile;
  final VirtualLayout? layout;
  final Keymap? keymap;
  final String? selectedKeyId;

  @override
  Widget build(BuildContext context) {
    final keyBindings = _bindingForKey(keymap, selectedKeyId);
    final wiredFrom = _physicalSources(profile, selectedKeyId);
    return Card(
      child: Padding(
        padding: const EdgeInsets.all(12),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Row(
              children: [
                const Icon(Icons.info_outline),
                const SizedBox(width: 8),
                Text('Details', style: Theme.of(context).textTheme.titleMedium),
              ],
            ),
            const SizedBox(height: 12),
            if (device == null)
              const Text('Select a device to inspect wiring and keymaps.')
            else ...[
              Wrap(
                spacing: 12,
                runSpacing: 8,
                children: [
                  _DetailPill(label: 'Device', value: device!.displayName),
                  _DetailPill(
                    label: 'Profile',
                    value: profile?.name ?? device!.profileId ?? 'None',
                  ),
                  _DetailPill(
                    label: 'Layout',
                    value: layout?.name ?? device!.layoutId ?? 'None',
                  ),
                  _DetailPill(
                    label: 'Keymap',
                    value: keymap?.name ?? 'None attached',
                  ),
                ],
              ),
              const SizedBox(height: 12),
              if (selectedKeyId == null)
                const Text('Tap a key to view bindings.')
              else ...[
                Text(
                  'Selected key: $selectedKeyId',
                  style: Theme.of(context).textTheme.titleSmall,
                ),
                const SizedBox(height: 8),
                if (wiredFrom.isNotEmpty)
                  Text(
                    'Physical sources: ${wiredFrom.join(', ')}',
                    style: Theme.of(context).textTheme.bodySmall,
                  )
                else
                  Text(
                    'No physical wiring for this key.',
                    style: Theme.of(context).textTheme.bodySmall?.copyWith(
                      color: Theme.of(context).colorScheme.error,
                    ),
                  ),
                const SizedBox(height: 8),
                if (keyBindings.isEmpty)
                  Text(
                    'No bindings found in keymap.',
                    style: Theme.of(context).textTheme.bodySmall,
                  )
                else
                  Column(
                    crossAxisAlignment: CrossAxisAlignment.start,
                    children: keyBindings.entries
                        .map(
                          (entry) => Padding(
                            padding: const EdgeInsets.only(bottom: 4),
                            child: Text('Layer ${entry.key}: ${entry.value}'),
                          ),
                        )
                        .toList(),
                  ),
              ],
            ],
          ],
        ),
      ),
    );
  }

  Map<int, String> _bindingForKey(Keymap? keymap, String? keyId) {
    if (keymap == null || keyId == null) return const {};
    final bindings = <int, String>{};
    for (var i = 0; i < keymap.layers.length; i++) {
      final layer = keymap.layers[i];
      final action = layer.bindings[keyId];
      if (action != null) {
        bindings[i] = _describeBinding(action);
      }
    }
    return bindings;
  }

  List<int> _physicalSources(HardwareProfile? profile, String? keyId) {
    if (profile == null || keyId == null) return const [];
    final sources = <int>[];
    profile.wiring.forEach((code, virtualId) {
      if (virtualId == keyId) {
        sources.add(code);
      }
    });
    sources.sort();
    return sources;
  }

  String _describeBinding(ActionBinding binding) {
    return binding.when(
      standardKey: (value) => value.isEmpty ? 'Key' : 'Key: $value',
      macro: (value) => value.isEmpty ? 'Macro' : 'Macro: $value',
      layerToggle: (value) => value.isEmpty ? 'Layer toggle' : 'Layer: $value',
      tapHold: (value) =>
          value.isEmpty ? 'Tap/Hold' : 'Tap/Hold: ${value.join(' / ')}',
      transparent: () => 'Transparent',
    );
  }
}

class _RuntimeStrip extends StatelessWidget {
  const _RuntimeStrip({
    required this.overlay,
    required this.isStale,
    required this.errors,
  });

  final RuntimeOverlay? overlay;
  final bool isStale;
  final List<KeyFlowError> errors;

  @override
  Widget build(BuildContext context) {
    final mods = overlay?.activeModifiers ?? const [];
    final layers = overlay?.activeLayers ?? const [];
    final held = overlay?.heldKeys ?? const [];
    return Card(
      child: Padding(
        padding: const EdgeInsets.all(12),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Row(
              children: [
                Icon(
                  Icons.monitor_heart_outlined,
                  color: isStale ? Theme.of(context).colorScheme.error : null,
                ),
                const SizedBox(width: 8),
                Text(
                  isStale ? 'Runtime stale' : 'Runtime overlay',
                  style: Theme.of(context).textTheme.titleMedium,
                ),
                const Spacer(),
                if (overlay?.pendingCount != null)
                  Text(
                    'Pending: ${overlay!.pendingCount}',
                    style: Theme.of(context).textTheme.bodySmall,
                  ),
              ],
            ),
            const SizedBox(height: 8),
            if (overlay == null)
              const Text('No runtime data yet.')
            else ...[
              Wrap(
                spacing: 8,
                runSpacing: 8,
                children: [
                  _Chip(
                    icon: Icons.layers_outlined,
                    label: layers.isEmpty
                        ? 'Layers: none'
                        : 'Layers: ${layers.join(', ')}',
                    color: Theme.of(context).colorScheme.surfaceVariant,
                  ),
                  _Chip(
                    icon: Icons.keyboard_command_key_outlined,
                    label: mods.isEmpty
                        ? 'Modifiers: none'
                        : 'Modifiers: ${mods.join(', ')}',
                    color: Theme.of(context).colorScheme.surfaceVariant,
                  ),
                  _Chip(
                    icon: Icons.keyboard_double_arrow_down_outlined,
                    label: held.isEmpty
                        ? 'Held: none'
                        : 'Held: ${held.join(', ')}',
                    color: Theme.of(context).colorScheme.surfaceVariant,
                  ),
                  _Chip(
                    icon: Icons.timer_outlined,
                    label: 'Last event ${overlay!.lastUpdate.toLocal()}',
                    color: isStale
                        ? Theme.of(context).colorScheme.errorContainer
                        : Theme.of(context).colorScheme.surfaceVariant,
                  ),
                ],
              ),
              if (overlay!.lastEvent != null) ...[
                const SizedBox(height: 8),
                Text(
                  'Event: ${overlay!.lastEvent}',
                  style: Theme.of(context).textTheme.bodySmall,
                ),
              ],
              if (overlay!.latencyUs != null)
                Text(
                  'Latency: ${overlay!.latencyUs}us',
                  style: Theme.of(context).textTheme.bodySmall,
                ),
            ],
            if (errors.isNotEmpty) ...[
              const SizedBox(height: 8),
              Text(
                'Errors: ${errors.map((e) => e.source).join(', ')}',
                style: Theme.of(context).textTheme.bodySmall?.copyWith(
                  color: Theme.of(context).colorScheme.error,
                ),
              ),
            ],
          ],
        ),
      ),
    );
  }
}

class _RuntimeBanner extends StatelessWidget {
  const _RuntimeBanner({this.lastUpdate});

  final DateTime? lastUpdate;

  @override
  Widget build(BuildContext context) {
    return Card(
      color: Theme.of(context).colorScheme.errorContainer,
      child: Padding(
        padding: const EdgeInsets.all(12),
        child: Row(
          children: [
            Icon(
              Icons.cloud_off,
              color: Theme.of(context).colorScheme.onErrorContainer,
            ),
            const SizedBox(width: 8),
            Expanded(
              child: Text(
                lastUpdate == null
                    ? 'Runtime stream idle. Showing last known overlay.'
                    : 'Runtime stream idle since ${lastUpdate!.toLocal()}.',
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

class _ErrorBanner extends StatelessWidget {
  const _ErrorBanner({required this.errors});

  final List<KeyFlowError> errors;

  @override
  Widget build(BuildContext context) {
    return Card(
      color: Theme.of(context).colorScheme.errorContainer,
      child: Padding(
        padding: const EdgeInsets.all(12),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Row(
              children: [
                Icon(
                  Icons.error_outline,
                  color: Theme.of(context).colorScheme.onErrorContainer,
                ),
                const SizedBox(width: 8),
                Text(
                  'Issues detected',
                  style: TextStyle(
                    color: Theme.of(context).colorScheme.onErrorContainer,
                    fontWeight: FontWeight.bold,
                  ),
                ),
              ],
            ),
            const SizedBox(height: 8),
            ...errors.map(
              (e) => Text(
                '${e.source}: ${e.message}',
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

class _Chip extends StatelessWidget {
  const _Chip({
    required this.icon,
    required this.label,
    required this.color,
    this.tooltip,
  });

  final IconData icon;
  final String label;
  final Color color;
  final String? tooltip;

  @override
  Widget build(BuildContext context) {
    final content = Container(
      padding: const EdgeInsets.symmetric(horizontal: 10, vertical: 6),
      decoration: BoxDecoration(
        color: color,
        borderRadius: BorderRadius.circular(20),
      ),
      child: Row(
        mainAxisSize: MainAxisSize.min,
        children: [Icon(icon, size: 16), const SizedBox(width: 6), Text(label)],
      ),
    );
    if (tooltip != null && tooltip!.isNotEmpty) {
      return Tooltip(message: tooltip!, child: content);
    }
    return content;
  }
}

class _Badge extends StatelessWidget {
  const _Badge({
    required this.icon,
    required this.label,
    required this.color,
    this.tooltip,
  });

  final IconData icon;
  final String label;
  final Color color;
  final String? tooltip;

  @override
  Widget build(BuildContext context) {
    final child = Container(
      padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 4),
      decoration: BoxDecoration(
        color: color,
        borderRadius: BorderRadius.circular(8),
      ),
      child: Row(
        mainAxisSize: MainAxisSize.min,
        children: [Icon(icon, size: 14), const SizedBox(width: 4), Text(label)],
      ),
    );
    if (tooltip != null && tooltip!.isNotEmpty) {
      return Tooltip(message: tooltip!, child: child);
    }
    return child;
  }
}

class _DetailPill extends StatelessWidget {
  const _DetailPill({required this.label, required this.value});

  final String label;
  final String value;

  @override
  Widget build(BuildContext context) {
    return Container(
      padding: const EdgeInsets.symmetric(horizontal: 10, vertical: 6),
      decoration: BoxDecoration(
        color: Theme.of(context).colorScheme.surfaceVariant,
        borderRadius: BorderRadius.circular(10),
      ),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        mainAxisSize: MainAxisSize.min,
        children: [
          Text(
            label,
            style: Theme.of(context).textTheme.labelSmall?.copyWith(
              color: Theme.of(context).colorScheme.onSurfaceVariant,
            ),
          ),
          Text(value, style: Theme.of(context).textTheme.bodyMedium),
        ],
      ),
    );
  }
}

class _TinySpinner extends StatelessWidget {
  const _TinySpinner({required this.label});

  final String label;

  @override
  Widget build(BuildContext context) {
    return Row(
      mainAxisSize: MainAxisSize.min,
      children: [
        const SizedBox(
          width: 16,
          height: 16,
          child: CircularProgressIndicator(strokeWidth: 2),
        ),
        const SizedBox(width: 6),
        Text(label, style: Theme.of(context).textTheme.bodySmall),
      ],
    );
  }
}
