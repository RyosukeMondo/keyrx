# KeyRx Project Configuration

## Project Overview

KeyRx is a keyboard remapping system with a 4-crate Rust workspace + React/TypeScript UI.

## Rules

- Do what has been asked; nothing more, nothing less
- NEVER create files unless absolutely necessary for the goal
- ALWAYS prefer editing existing files over creating new ones
- NEVER proactively create documentation files (*.md) or README files
- NEVER save working files, text/mds, or tests to the root folder

## File Organization

Save files in appropriate subdirectories:
- `keyrx_core/src/` - Core library (no_std, platform-agnostic)
- `keyrx_compiler/src/` - Config compiler
- `keyrx_daemon/src/` - Daemon + web server (axum)
- `keyrx_daemon/tests/` - Integration/e2e tests
- `keyrx_ui/src/` - React frontend
- `scripts/` - Build/test automation
- `docs/` - Documentation (only when requested)

## Workflow

- For complex tasks (3+ files, new features, refactoring): use Task tool to spawn parallel agents
- Spawn all agents in ONE message with `run_in_background: true`
- After spawning, tell the user what's working and wait for results
- For simple tasks (1-2 file edits, bug fixes): work directly without agents

## Quality Gates

- `cargo clippy --workspace -- -D warnings` (zero warnings)
- `cargo fmt --check`
- `cargo test --workspace` (all pass)
- 80% test coverage minimum (90% for keyrx_core)
- Max 500 lines/file, max 50 lines/function

## Full Reference

See `.claude/CLAUDE.md` for complete development guide (architecture, naming, troubleshooting, shared utilities).
