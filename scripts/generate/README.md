# Generation Scripts

Scripts that **create new files** from authoritative sources.

## Principle

Each script reads from a **single source of truth** and generates derived artifacts.

## Scripts

| Script | Generates | Source | Output |
|--------|-----------|--------|--------|
| `docs-errors.sh` | Error documentation | Rust error registry | `docs/errors/` |
| `coverage.sh` | Coverage report | `lcov.info` | `ui/coverage/html/` |
| `metrics-dashboard.sh` | Contract metrics | Contract JSON files | `.spec-workflow/analysis/` |
| `metrics-history.sh` | Velocity analysis | Implementation logs | `.spec-workflow/analysis/` |

## Usage

```bash
# Generate individual artifacts
./scripts/generate/docs-errors.sh
./scripts/generate/coverage.sh
./scripts/generate/metrics-dashboard.sh

# Via justfile
just gen-docs
just gen-coverage
just gen-metrics
```

## Notes

- **Bindings generation** is handled by the Rust tool in `core/tools/generate_dart_bindings/`
- Call it via `just gen-bindings` (not a script here)
