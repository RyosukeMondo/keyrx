# Design System & UI/UX Guidelines

## Overview

This document is the source of truth for the KeyRx visual system, Flutter widget conventions, and accessibility rules. It aligns the design language with the current implementation in `ui/lib/` and keeps desktop and mobile workflows consistent.

## Design Philosophy

1. **Keyboard-First, Visual-Second** – visuals clarify state; keyboard remains the fastest path.
2. **Fail Visible** – every error or mode change is immediately visible.
3. **Zero Surprise** – predictable layouts, no hidden actions.
4. **Accessibility First** – WCAG 2.1 AA, full keyboard navigation.
5. **Performance as Feature** – 60fps targets, <16ms frame budget, minimal overdraw.

User lenses: **Power Users** (speed/shortcuts), **Visual Thinkers** (spatial flows), **Debuggers** (state inspection, traces).

---

## Foundations

### Color Palette
- **Seed & scheme**: Material 3 scheme seeded with `#2196F3` (see `ui/lib/main.dart`). Default runtime uses dark mode; light mode must stay contrast-compliant.
- **Accent**: Amber `#FFC107` for highlights and warnings.
- **Semantic**: Success `#4CAF50`, Warning `#FFC107`, Error `#F44336`, Info `#64B5F6`.
- **Surfaces**: Use `Theme.of(context).colorScheme.surface{,ContainerHighest}` and `SurfaceStyles.surfaceColor(context)` for cards and panels to stay in sync with dark/light themes.
- **Device state tinting**: Active/Remapping `#4CAF50`, Connected `#2196F3`, Error `#F44336`, Disabled `#9E9E9E`.

```dart
final colorScheme = ColorScheme.fromSeed(
  seedColor: const Color(0xFF2196F3),
  brightness: Brightness.dark,
);
```

### Typography
- **Primary font**: System UI (Roboto/Segoe UI/SF) for speed and OS alignment.
- **Monospace**: JetBrains Mono for code, logs, and JSON previews.
- **Usage map** (Material 3 defaults):
  - `titleLarge` (22/500): Page titles, dialogs.
  - `titleMedium` (16/500): Section headers, cards.
  - `bodyLarge` (16/400): Primary reading text.
  - `bodyMedium` (14/400): Metadata, lists.
  - `labelLarge` (14/500): Buttons, chips.
  - `labelSmall` (11/500): Badges and dense UI.

### Spacing & Layout Grid
- **Base unit**: 4px grid; build multiples of 4.
- **Tokens (from `UiConstants`):**
  - Padding: `defaultPadding 16`, `smallPadding 8`, `tinyPadding 4`.
  - Margin: `defaultMargin 12`.
  - Radii: `defaultBorderRadius 4`, `keyBorderRadius 6`, `surfaceBorderRadius 12`.
  - Elevation: `defaultElevation 6`.
- **Common spacing**: Card/content padding 16, inter-component gaps 12–16, page gutters 24–32 on desktop, 16 on mobile.

### Elevation & Surfaces
- Prefer Material elevation over custom shadows. Defaults:
  - Cards/panels: elevation 2–6.
  - Dialogs: 8–12.
  - Drawers/modals: 16–24.
- Use `SurfaceContainer` (`ui/lib/ui/styles/surfaces.dart`) for theme-aware surfaces with consistent padding, margin, border, and tint.

---

## Component Library

| Area | Path | Guidance |
| --- | --- | --- |
| Surfaces | `ui/lib/ui/styles/surfaces.dart` | Use `SurfaceContainer` for cards/panels; keep padding/margin defaults unless a layout requires tighter density. |
| Buttons | `ui/lib/ui/assets/asset_button.dart` | Asset-backed hero buttons (primary/secondary/danger). For utility actions, use `FilledButton/OutlinedButton/TextButton` with `colorScheme`. Keep hit area ≥48px. |
| Device cards | `ui/lib/widgets/device_card.dart` | Standard device presentation (identity, remap toggle, profile actions). Keep status text short; align icons to `colorScheme.primary`/state colors. |
| Error dialogs | `ui/lib/ui/widgets/app_error_dialog.dart` | Default dialog for blocking errors. Include actionable primary CTA and optional secondary. |
| Navigation | `ui/lib/widgets/developer_drawer.dart`, `ui/lib/pages/layouts.dart` | Persist navigation labels; pair icons with text for clarity. |
| Asset icons | `ui/lib/ui/assets/asset_icon.dart` | Use for branded icons; respect theme contrast (avoid placing on low-contrast surfaces). |

### Buttons (Material examples)
```dart
FilledButton(
  onPressed: onTap,
  style: FilledButton.styleFrom(
    padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 12),
    shape: RoundedRectangleBorder(
      borderRadius: BorderRadius.circular(UiConstants.defaultBorderRadius),
    ),
  ),
  child: const Text('Primary'),
);

OutlinedButton(
  onPressed: onTap,
  child: const Text('Secondary'),
);
```

### Cards & Lists
- Wrap list-heavy pages in `SurfaceContainer` to separate from background.
- Use `ListTile` for dense data (title, subtitle, trailing icon). Maintain 12–16px vertical rhythm between tiles.
- For status indicators, prefer `Badge`/`Chip` using `colorScheme` semantic colors.

---

## Flutter Widget Patterns

- **StatelessWidget**: pure render; no internal state.
- **StatefulWidget**: local UI state (expansion, animations, form edits) before committing to global state.
- **Riverpod**: prefer `ConsumerWidget`/`WidgetRef.watch` for read-only, `ref.listen` for side effects. Keep rebuild surfaces small.

```dart
class DeviceListPage extends ConsumerWidget {
  const DeviceListPage({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final devices = ref.watch(devicesProvider);
    return devices.when(
      data: (items) => ListView.builder(
        itemCount: items.length,
        itemBuilder: (_, i) => DeviceCard(
          deviceState: items[i],
          deviceService: ref.read(deviceRegistryProvider),
          profileService: ref.read(profileRegistryProvider),
        ),
      ),
      loading: () => const Center(child: CircularProgressIndicator()),
      error: (error, stack) => AppErrorDialog.show(
        context,
        title: 'Unable to load devices',
        message: '$error',
      ),
    );
  }
}
```

### Composition Rules
- Favor composition over inheritance. Extract reusable surfaces, headers, and action rows.
- Keep widget trees shallow by splitting sub-sections (e.g., header, content, actions) into widgets.
- Use `const` constructors where possible to reduce rebuild cost.

---

## Layout Patterns

### Responsive Breakpoints
```dart
enum ScreenSize { mobile, tablet, desktop }

ScreenSize screenSize(BuildContext context) {
  final width = MediaQuery.of(context).size.width;
  if (width < 600) return ScreenSize.mobile;
  if (width < 1200) return ScreenSize.tablet;
  return ScreenSize.desktop;
}
```
- Desktop: master-detail (list 280–320px + content). Use `NavigationRail` or side drawers for tools.
- Tablet: two-column where possible; collapse tertiary panels.
- Mobile: single-column with `NavigationBar` and bottom sheets for secondary actions.

### Navigation
- Keep page titles visible on every top-level screen.
- Use `DeveloperDrawer` for advanced tools; keep primary navigation in the main scaffold.

---

## Accessibility

- **Keyboard**: All actions must be reachable via tab order; set `focusNode` on custom widgets.
- **Semantics**: Provide labels/descriptions for icons and stateful components. Announce state changes with `SemanticsService.announce`.
- **Contrast**: Maintain 4.5:1 for text on surfaces; validate gradients with overlays if needed.
- **Touch targets**: Minimum 48x48dp with 8dp spacing; do not shrink hit areas on dense lists.
- **Forms**: Associate labels and helper text; surface validation inline and in semantic labels.

---

## Motion & Microinteractions

- Durations: 100ms (tooltip/hover), 200ms (focus/selection), 300ms (page/modal), 500ms (complex sequences).
- Curves: `Curves.easeInOut` default, `Curves.easeOut` for entering, `Curves.easeIn` for exiting, `Curves.easeInOutCubicEmphasized` for critical actions.
- Respect reduced motion settings; avoid continuous animations if `MediaQuery.of(context).disableAnimations` is true.

---

## Dark Mode

- Default runtime uses dark theme with `ColorScheme.fromSeed` (blue seed). Light theme must preserve semantic colors and contrast.
- Surfaces: prefer `surfaceContainerHighest` tinted via `SurfaceStyles.surfaceColor(context)` instead of hard-coded greys.
- Gradients/assets: ensure text overlays reach contrast; add scrims at 60–70% if needed.
- Always set `themeMode: ThemeMode.system` to honor OS preference for end users; debugging screens can expose a toggle if needed.

---

## Error, Loading, and Empty States

- Use `CircularProgressIndicator` for blocking loads; prefer skeletons/placeholders for lists.
- Error dialogs: use `AppErrorDialog.show` with concise title, actionable primary button, optional secondary (Retry/Contact).
- Async data (`AsyncValue`): handle `data/loading/error` explicitly; include retry hooks where safe.
- Empty states: provide icon, single-line description, and a primary CTA to create/import data.

---

## Design Decisions (ADR-style)

- **Material 3, seed-based theming (2025-01)**: aligns with Flutter defaults, enables dynamic color tuning, keeps dark/light parity.
- **System UI + JetBrains Mono (2025-01)**: fastest text render path, consistent developer-facing surfaces.
- **4px grid with UiConstants tokens (2025-01)**: matches existing layout constants and keeps spacing predictable across platforms.
- **Dark-first runtime (2025-01)**: primary user base is desktop developers; dark reduces glare and matches terminal contexts.
- **Asset-backed hero buttons for marketing moments (2025-01)**: keep branded CTAs while everyday controls use Material buttons for accessibility and theming.

---

## References

- Material Design 3: https://m3.material.io/
- Flutter Widget Catalog: https://docs.flutter.dev/ui/widgets
- WCAG 2.1 Guidelines: https://www.w3.org/WAI/WCAG21/quickref/
- Riverpod Docs: https://riverpod.dev/
