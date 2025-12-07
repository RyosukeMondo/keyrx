# Task Completion Steps
- For spec tasks: mark selected task in `.spec-workflow/specs/<spec>/tasks.md` from `[ ]` to `[-]` before coding; mark `[x]` after completion and logging.
- Before implementing, search existing Implementation Logs under `.spec-workflow/specs/<spec>/Implementation Logs/` to avoid duplication (grep for api/component/function/integration terms).
- Run relevant checks (prefer `just check`, or targeted tests/formatters per change). Follow project pre-commit expectations (cargo fmt, cargo clippy -D warnings, cargo test --lib).
- After finishing work, use `log-implementation` tool with artifacts (apiEndpoints/components/functions/classes/integrations), files modified/created, stats, summary, and taskId.
- Commit changes with clear message once a logical unit is done; avoid reverting unrelated dirty tree changes.
