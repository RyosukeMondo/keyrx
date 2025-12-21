# Check Scripts

Scripts that **analyze and report** without failing (exit code = 0).

## Principle

Check scripts provide insights and warnings but don't block workflows.

## Scripts

| Script | Analyzes | Reports |
|--------|----------|---------|
| `module-sizes.sh` | Rust module line counts | Files exceeding 500-line limit |
| `performance.sh` | Benchmark performance | Performance metrics |

## Usage

```bash
# Check module sizes
./scripts/check/module-sizes.sh

# Run performance benchmarks
./scripts/check/performance.sh

# Via justfile
just check-module-sizes
just check-performance
```

## Exit Codes

- Always returns `0` (informational only)
- Warnings are logged but don't fail the build
