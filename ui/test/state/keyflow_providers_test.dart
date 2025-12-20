import 'dart:async';

import 'package:core_logic/models/flow/flow_graph.dart';
import 'package:core_logic/models/hardware_profile.dart';
import 'package:core_logic/models/runtime_config.dart';
import 'package:core_logic/services/config_result.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:keyrx_ui/repositories/keyflow_repository.dart';
import 'package:keyrx_ui/state/keyflow_providers.dart';
import 'package:keyrx_ui/state/node_positions_provider.dart';
import 'package:keyrx_ui/utils/core_ui_extensions.dart';
import 'package:mocktail/mocktail.dart';
import 'package:shared_preferences/shared_preferences.dart';

class _MockKeyFlowRepository extends Mock implements KeyFlowRepository {}

void main() {
  TestWidgetsFlutterBinding.ensureInitialized();

  const device = DeviceInstanceId(vendorId: 1, productId: 2, serial: 'abc');
  const baseConfig = RuntimeConfig(devices: [DeviceSlots(device: device)]);

  setUp(() {
    SharedPreferences.setMockInitialValues({});
  });

  test(
    'runtimeConfigProvider loads initial config and listens for updates',
    () async {
      final controller = StreamController<RuntimeConfig>();
      final repository = _MockKeyFlowRepository();

      when(repository.watchRuntimeConfig).thenAnswer((_) => controller.stream);
      when(
        () => repository.getRuntimeConfig(
          forceRefresh: any(named: 'forceRefresh'),
        ),
      ).thenAnswer((_) async => ConfigOperationResult.success(baseConfig));
      when(repository.dispose).thenAnswer((_) async {});

      final container = ProviderContainer(
        overrides: [keyFlowRepositoryProvider.overrideWithValue(repository)],
      );
      addTearDown(container.dispose);

      final initial = await container.read(runtimeConfigProvider.future);
      expect(initial, baseConfig);

      final listener = container.listen(
        runtimeConfigProvider,
        (_, __) {},
        fireImmediately: true,
      );

      const updated = RuntimeConfig(
        devices: [
          DeviceSlots(
            device: device,
            slots: [
              ProfileSlot(
                id: 'slot-a',
                hardwareProfileId: 'hp-a',
                keymapId: 'km-a',
                active: true,
              ),
            ],
          ),
        ],
      );

      controller.add(updated);
      await Future<void>.delayed(Duration.zero);

      expect(listener.read().value, updated);

      await controller.close();
      listener.close();
    },
  );

  test('flowGraphProvider derives graph from runtime config', () async {
    final repository = _MockKeyFlowRepository();
    when(repository.watchRuntimeConfig).thenAnswer((_) => const Stream.empty());

    const config = RuntimeConfig(
      devices: [
        DeviceSlots(
          device: device,
          slots: [
            ProfileSlot(
              id: 'slot-a',
              hardwareProfileId: 'hp-a',
              keymapId: 'km-a',
              priority: 1,
            ),
          ],
        ),
      ],
    );

    when(
      () =>
          repository.getRuntimeConfig(forceRefresh: any(named: 'forceRefresh')),
    ).thenAnswer((_) async => ConfigOperationResult.success(config));
    when(repository.dispose).thenAnswer((_) async {});

    final container = ProviderContainer(
      overrides: [keyFlowRepositoryProvider.overrideWithValue(repository)],
    );
    addTearDown(container.dispose);

    final graph = await container.read(flowGraphProvider.future);

    expect(graph.nodes.length, 3);
    expect(graph.edges.length, 2);
    expect(graph.edges.every((edge) => edge.priority == 1), isTrue);
  });

  test('nodePositionsProvider persists and restores positions', () async {
    final overrides = [
      flowGraphProvider.overrideWith(
        (_) async => const FlowGraph(
          nodes: [FlowNode.device(id: 'node-a', deviceId: device)],
        ),
      ),
    ];
    final container = ProviderContainer(overrides: overrides);
    addTearDown(container.dispose);

    final notifier = container.read(nodePositionsProvider.notifier);
    await notifier.setPosition('node-a', const Offset(120.5, 40.25));

    final stored = await container.read(nodePositionsProvider.future);
    expect(stored['node-a']?.offset, const Offset(120.5, 40.25));
    expect(stored['node-a']?.isManual, isTrue);

    final secondContainer = ProviderContainer(overrides: overrides);
    addTearDown(secondContainer.dispose);
    final restored = await secondContainer.read(nodePositionsProvider.future);
    expect(restored['node-a']?.offset, const Offset(120.5, 40.25));
  });

  test('nodePositionsProvider auto layouts nodes by type', () async {
    final container = ProviderContainer(
      overrides: [
        flowGraphProvider.overrideWith(
          (_) async => const FlowGraph(
            nodes: [
              FlowNode.device(id: 'device-1', deviceId: device),
              FlowNode.hardwareProfile(id: 'hp-1'),
              FlowNode.keymap(id: 'km-1'),
            ],
          ),
        ),
      ],
    );
    addTearDown(container.dispose);

    final positions = await container.read(nodePositionsProvider.future);

    expect(positions['device-1']?.x, 80);
    expect(positions['hp-1']?.x, 460);
    expect(positions['km-1']?.x, 820);
    expect(positions['device-1']?.isManual, isFalse);
  });

  test(
    'keyFlowActionsProvider updates runtime state after mutations',
    () async {
      final repository = _MockKeyFlowRepository();
      final controller = StreamController<RuntimeConfig>();
      const initialConfig = RuntimeConfig(
        devices: [
          DeviceSlots(
            device: device,
            slots: [
              ProfileSlot(
                id: 'slot-a',
                hardwareProfileId: 'hp-a',
                keymapId: 'km-a',
              ),
            ],
          ),
        ],
      );
      const updatedConfig = RuntimeConfig(
        devices: [
          DeviceSlots(
            device: device,
            slots: [
              ProfileSlot(
                id: 'slot-a',
                hardwareProfileId: 'hp-a',
                keymapId: 'km-a',
              ),
              ProfileSlot(
                id: 'slot-b',
                hardwareProfileId: 'hp-b',
                keymapId: 'km-b',
                priority: 1,
              ),
            ],
          ),
        ],
      );

      when(repository.watchRuntimeConfig).thenAnswer((_) => controller.stream);
      when(
        () => repository.getRuntimeConfig(
          forceRefresh: any(named: 'forceRefresh'),
        ),
      ).thenAnswer((_) async => ConfigOperationResult.success(initialConfig));
      when(
        () => repository.addSlot(
          device,
          const ProfileSlot(
            id: 'slot-b',
            hardwareProfileId: 'hp-b',
            keymapId: 'km-b',
            priority: 1,
          ),
        ),
      ).thenAnswer((_) async => ConfigOperationResult.success(updatedConfig));
      when(repository.dispose).thenAnswer((_) async {});

      final container = ProviderContainer(
        overrides: [keyFlowRepositoryProvider.overrideWithValue(repository)],
      );
      addTearDown(container.dispose);

      await container.read(runtimeConfigProvider.future);
      container
          .read(keyFlowActionsProvider)
          .addSlot(
            device,
            const ProfileSlot(
              id: 'slot-b',
              hardwareProfileId: 'hp-b',
              keymapId: 'km-b',
              priority: 1,
            ),
          );

      final runtimeValue = container.read(runtimeConfigNotifierProvider).value;
      expect(runtimeValue, updatedConfig);

      await controller.close();
    },
  );
}
