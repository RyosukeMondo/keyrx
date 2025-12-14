# FFI Development Workflow (AI-Native)

This workflow keeps FFI changes traceable and fast for humans and AI agents.

## Logging Every FFI Change

1) After completing work (ideally right after a commit), generate a log:

```
just ffi-log <task-number>
```

Optional: compare against a base ref for clearer diffs:

```
just ffi-log 3 main
```

2) The script writes a pre-filled entry at `.spec-workflow/specs/rust-first-ai-native-ffi/Implementation Logs/`.
   - Template: `.spec-workflow/templates/ffi-implementation-log-template.md`
   - Sections cover: functions added/modified, contract changes, files touched, tests, performance, AI notes, follow-ups.

3) Open the generated file, fill the placeholders, and commit it with your code.

## Automation Helpers

- `scripts/log_ffi_implementation.sh` populates:
  - Timestamp, log id, task id
  - `git status` snapshot and `git diff --stat` (or `--base-ref` diff)
- `.githooks/post-commit` reminds you to log after each commit. Install hooks once:

```
./scripts/install-hooks.sh
```

## When to Log

- Any FFI-facing change: new or modified exports, contract/schema updates, codegen changes, watch-mode/tracing tweaks.
- Include tests you ran and performance impacts (codegen time, watch latency, tracing overhead).
- Capture AI notes: prompts that worked/failed and constraints to remember.

## Tips for Fast, Good Logs

- Keep bullets short; prefer concrete facts over prose.
- Note contract compatibility (breaking vs compatible) and follow-up actions.
- If nothing changed since last commit, the generated diff/stat blocks will show “no diff” — leave them as-is.

## Reference

- Spec: `rust-first-ai-native-ffi`
- Log directory: `.spec-workflow/specs/rust-first-ai-native-ffi/Implementation Logs/`
- Template: `.spec-workflow/templates/ffi-implementation-log-template.md`
- Script: `scripts/log_ffi_implementation.sh`
- Just command: `just ffi-log <id> [<base_ref>]`
