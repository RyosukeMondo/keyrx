# KeyRx Overview
- Purpose: Cross-platform keyboard remapping engine with Rhai scripting, reactive/event-sourced core, Flutter UI for configuration/debugging. Current spec: advanced profiling & flamegraph support.
- Tech stack: Rust core (Tokio, Rhai scripting), Flutter/Dart UI, optional OpenTelemetry. Uses just for task running; cargo for core builds; Flutter for UI.
- Structure: core/ (Rust engine & CLI), ui/ (Flutter app), docs/ (architecture, guides), scripts/ (Rhai examples), tests/ (integration/UAT). Spec artifacts live under .spec-workflow/specs/<name>/.
- Notable docs: README.md (usage/commands), CONTRIBUTING.md (dev commands/checks), docs/ARCHITECTURE.md (principles & QA).
- Platforms: Linux & Windows targets; UI uses Flutter hot reload; core binaries built with cargo.
