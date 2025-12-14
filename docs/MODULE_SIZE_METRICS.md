# Module Size Metrics

This repo enforces a 500-line code limit per Rust module to keep files AI-friendly and reviewable.

## Running the analysis
- Run `./scripts/analyze_module_sizes.sh` (fast, uses `tokei`).
- Outputs: `.spec-workflow/analysis/module-sizes.json` (machine readable) and `.spec-workflow/analysis/module-sizes.html` (dashboard).
- Adjust the threshold temporarily with `MODULE_MAX_LINES=600 ./scripts/analyze_module_sizes.sh`.

## Allowlist exceptions
- Legitimate exceptions live in `.spec-workflow/config/module-size-allowlist.json`.
- Each entry needs `path`, `max_lines`, and `reason`. Keep `max_lines` tight and remove once the module is refactored.
- Current temporary exceptions cover the legacy `exports_engine.rs` and `exports_compat.rs` monoliths scheduled for splitting tasks.

## Pre-commit enforcement
- The Git hook runs the analysis and fails commits on modules over the limit unless allowlisted.
- If you must add a temporary exception, update the allowlist with a reason and follow-up task, then rerun the hook.

## Interpretation and next steps
- Use the HTML dashboard to sort by size and follow recommendations for splitting.
- For oversized files, target ~500-line slices grouped by domain/responsibility and add new modules under `core/src`.
