# IDE Setup (VS Code)

## Quick Start
- Install VS Code and ensure the Flutter and Dart SDKs are available (FVM-managed at `.fvm/versions/stable`).
- Open the repository root in VS Code so workspace settings and tasks load automatically.
- Accept recommended extensions when prompted for full Flutter/Dart support.

## Workspace Settings
- Format on save and organize imports on save are enabled; Dart line length is set to 100 characters.
- Default formatters: Dart (`Dart-Code.dart-code`), Rust (`rust-analyzer`), TOML (`even-better-toml`).
- Flutter SDK path is pinned to `.fvm/versions/stable`. Adjust only if using a different FVM channel.
- UI guides and function call completions are enabled to speed navigation and autocompletion.

## Recommended Extensions
- `Dart-Code.flutter`, `Dart-Code.dart-code`
- `robert-brunhage.flutter-riverpod-snippets` (Riverpod helpers)
- `usernamehw.errorlens` (inline diagnostics)
- Rust tooling already recommended: `rust-lang.rust-analyzer`, `tamasfe.even-better-toml`, `serayuzgur.crates`

## Launch Configurations
Defined in `.vscode/launch.json`:
- **Flutter: KeyRx UI** — Runs the app from `ui/lib/main.dart` after building Rust core.
- **Flutter: KeyRx UI (Mock FFI)** — Runs the app with `--dart-define=MOCK_FFI=true` to use the mock bridge.
- **Flutter: All Tests** — Runs all Flutter tests in `ui/test`.
- **Flutter: Widget & Golden Tests** — Runs tests tagged `widget` for focused UI checks.

## Tasks and Automation
- Build tasks in `.vscode/tasks.json` handle Rust core builds and Dart binding verification. Ensure the workspace root is selected so tasks resolve paths correctly.
- Pre-launch tasks for app configs depend on Rust build to keep FFI bindings current.

## Snippets
Custom snippets in `.vscode/snippets.json`:
- `riverpodProvider` / `rpProvider` — Annotated Riverpod provider with generated part directive.
- `widgetHarnessTest` / `wthTest` — Widget test skeleton using `WidgetTestHarness`.
- `goldenHarnessTest` / `goldenTest` — Golden test skeleton using `GoldenTestHarness`.

## Troubleshooting
- If the Flutter SDK path is not detected, run `fvm use stable` in `ui/` or update `dart.flutterSdkPath`.
- For hot reload issues, confirm you launched via the provided Flutter configurations (not a generic Dart config).
- If tests fail to discover, ensure you run from the repo root so the `ui` `cwd` resolves correctly.
