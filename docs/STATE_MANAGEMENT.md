# State Management Guide

KeyRx UI now standardizes on Riverpod for new development while keeping the existing `provider` consumers running through an adapter layer.

## Architecture
- `ProviderScope` wraps the app in `main.dart` to enable Riverpod.
- Core dependencies are created in `ui/lib/state/providers_setup.dart` and exposed to legacy widgets through `createProviders` in `ui/lib/state/providers.dart`.
- `keyFlowProvider` surfaces the aggregated `FacadeState` stream so widgets can watch runtime status without manual stream wiring.

## Core Providers
- `bridgeProvider` – lifecycle-managed `KeyrxBridge`.
- `serviceRegistryProvider` – constructs and disposes `ServiceRegistry`.
- `keyrxFacadeProvider` – unified facade built from the registry.
- `apiDocsServiceProvider` – shared API docs service instance.
- `appStateProvider` – `ChangeNotifier` for UI state (developer mode, loaded script, etc.).
- `keyFlowProvider` – stream of `FacadeState` for KeyFlow/engine awareness.

## Usage Patterns
- Read dependencies with `WidgetRef`:
  ```dart
  class DevicesPage extends ConsumerWidget {
    @override
    Widget build(BuildContext context, WidgetRef ref) {
      final registry = ref.watch(serviceRegistryProvider);
      final facade = ref.watch(keyrxFacadeProvider);
      // use registry/facade here
    }
  }
  ```
- Listen to aggregated state:
  ```dart
  final keyFlow = ref.watch(keyFlowProvider);
  return keyFlow.when(
    data: (state) => Text(state.engine.toString()),
    loading: () => const CircularProgressIndicator(),
    error: (err, _) => Text('Error: $err'),
  );
  ```
- Legacy `provider` consumers continue to work because Riverpod instances are bridged via `createProviders`.

## Migration Notes
- Prefer Riverpod for all new state and dependency injection.
- For existing widgets that use `BuildContext.read/watch`, migrate them to `ConsumerWidget`/`ConsumerStatefulWidget` over time.
- Avoid creating new global singletons; place shared instances in `providers_setup.dart` so lifecycle is managed centrally.
