# KeyRx Project Configuration

## Rules

- Do what has been asked; nothing more, nothing less
- NEVER create files unless absolutely necessary for the goal
- ALWAYS prefer editing existing files over creating new ones
- NEVER proactively create documentation files (*.md) or README files
- NEVER save working files, text/mds, or tests to the root folder

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

See `.claude/CLAUDE.md` for the development guide (Linux host notes, critical constraints, shared utilities).
