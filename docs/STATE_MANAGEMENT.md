# State Management Guide

KeyRx standardizes on Riverpod for new development while keeping existing `provider` consumers alive through an adapter layer.

## Architecture
- `ProviderScope` wraps the app in `main.dart` to enable Riverpod.
- Core dependencies are created in `ui/lib/state/providers_setup.dart` and exposed to legacy widgets via `createProviders` in `ui/lib/state/providers.dart`.
- `keyFlowProvider` surfaces the aggregated `FacadeState` stream so widgets can watch runtime status without manual stream wiring.
- Generated code is minimal today; we keep providers hand-authored for clarity. Add `riverpod_annotation` if you need families/codegen, but prefer simple providers unless ergonomics demand macros.

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

## Best practices
- Prefer `ConsumerWidget`/`ConsumerStatefulWidget` over `Provider` context usage; move logic out of `initState` into providers where possible.
- Keep providers pure and side-effect free; place I/O inside async notifiers or facade calls, not in constructors.
- Keep provider names descriptive and in `snake_case` when exported (e.g., `devicesProvider`), matching the file name.
- Use `.when` for async providers to avoid uncaught loading/error states; never ignore the error branch.
- For imperative code paths (dialogs, navigation) pull values via `ref.read` to avoid rebuilds; otherwise use `ref.watch`.
- Dispose resources in provider tear-down (see `serviceRegistryProvider` pattern) rather than relying on widget disposal.

## Migration Notes
- Prefer Riverpod for all new state and dependency injection.
- For existing widgets that use `BuildContext.read/watch`, migrate them to `ConsumerWidget`/`ConsumerStatefulWidget` over time.
- Avoid creating new global singletons; place shared instances in `providers_setup.dart` so lifecycle is managed centrally.
