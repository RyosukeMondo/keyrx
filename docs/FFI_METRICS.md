# FFI Metrics and Observability

This document describes the metrics, dashboards, and observability tools available for the FFI layer.

## Overview

The FFI layer provides comprehensive metrics and observability capabilities to monitor:
- Contract evolution and health
- Function call performance and reliability
- Breaking change detection
- Development velocity

## Contract Metrics Dashboard

### What It Provides

The contract metrics dashboard provides visibility into the overall health and evolution of the FFI layer:

**Summary Metrics:**
- Total FFI functions across all domains
- Number of domains
- Average parameters per function
- Return type distribution (Result vs other types)
- Breaking changes count from git history

**Visualizations:**
- Functions by Domain (bar chart) - shows distribution of functions across domains
- Evolution Timeline (line chart) - tracks functions added/removed over the last 12 weeks
- Return Type Distribution (pie chart) - shows proportion of Result-based vs other return types

### Usage

#### Generate Dashboard Locally

```bash
just ffi-dashboard
```

This generates:
- `.spec-workflow/analysis/contract-metrics.json` - Raw metrics data
- `.spec-workflow/analysis/contract-dashboard.html` - Interactive HTML dashboard

Open the dashboard in your browser:
```bash
open .spec-workflow/analysis/contract-dashboard.html
# or
xdg-open .spec-workflow/analysis/contract-dashboard.html
```

#### View in CI

The dashboard is automatically generated on every CI run and uploaded as a build artifact:

1. Navigate to the GitHub Actions run
2. Scroll to the **Artifacts** section
3. Download `ffi-contract-dashboard`
4. Open `contract-dashboard.html` in your browser

For pull requests, a summary is automatically posted as a comment.

### How It Works

The dashboard generation script (`scripts/generate_contract_metrics.sh`) performs the following:

1. **Contract Analysis**: Parses all `.ffi-contract.json` files in `core/src/ffi/contracts/`
   - Counts total functions
   - Groups functions by domain
   - Analyzes parameter counts
   - Classifies return types

2. **Git History Analysis**: Examines the last 12 weeks of commits
   - Tracks lines added/removed in contract files
   - Estimates functions added/removed (assumes ~15 lines per function)
   - Identifies breaking changes from commit messages

3. **Dashboard Generation**: Creates interactive HTML with Chart.js
   - Renders bar chart for domain distribution
   - Renders line chart for evolution timeline
   - Renders pie chart for return type distribution
   - All charts are interactive (hover for details)

## FFI Call Metrics

### Runtime Metrics

FFI calls are instrumented with tracing to collect runtime metrics:

```bash
just ffi-metrics          # Text format
just ffi-metrics json     # JSON format
```

**Available Metrics:**
- Call count per function
- Average latency per function
- Error rate per function
- Total calls across all functions
- Most frequently called functions
- Slowest functions

See `core/src/ffi/metrics.rs` for implementation details.

### Tracing Integration

All FFI functions are instrumented with the `#[instrument]` attribute from the `tracing` crate:

```rust
#[instrument(skip(ctx))]
#[ffi_export(domain = "engine", version = "1.0")]
pub fn start_loop(ctx: &FfiContext, profile_id: String) -> Result<EngineState> {
    // Function implementation
}
```

This provides:
- Structured logging of FFI calls
- Automatic error tracking
- Performance measurement
- Integration with Dart DevTools timeline

**Enable FFI tracing:**
```bash
just dev-trace
```

This runs the development environment with FFI tracing enabled at the `trace` level.

## Breaking Change Detection

The FFI layer includes automated breaking change detection:

```bash
just diff-ffi-contracts          # Compare with HEAD~1
just diff-ffi-contracts HEAD~5   # Compare with 5 commits ago
```

**Breaking changes detected:**
- Removed functions
- Changed parameter types
- Changed return types
- Renamed functions
- Changed domain names

Breaking changes are automatically detected in CI and will block PRs unless the version is bumped in `Cargo.toml`.

See `.github/workflows/ffi-breaking-changes.yml` for CI integration.

## Metrics Best Practices

### 1. Monitor Dashboard Regularly

Review the contract dashboard weekly to:
- Track FFI growth and complexity
- Identify domains that are becoming too large
- Monitor breaking change frequency
- Ensure balanced distribution across domains

### 2. Use Tracing in Development

Always run `just dev-trace` when:
- Debugging FFI issues
- Profiling performance
- Understanding call patterns
- Investigating errors

### 3. Check Metrics Before Refactoring

Before large refactoring efforts:
1. Generate current dashboard: `just ffi-dashboard`
2. Review function distribution by domain
3. Identify candidates for splitting or consolidation
4. After refactoring, regenerate dashboard to verify improvements

### 4. Performance Targets

Use `just ffi-metrics` to ensure:
- FFI call latency <1ms for simple functions
- FFI call latency <10ms for complex functions
- Error rate <1% under normal operation

### 5. Breaking Change Policy

- **Major version bump**: Required for any breaking changes
- **Minor version bump**: For new functions or optional parameters
- **Patch version bump**: For bug fixes with no API changes

Always use `just diff-ffi-contracts` before releasing to detect breaking changes.

## Metrics Data Schema

### contract-metrics.json

```json
{
  "generated_at": "2025-12-15T00:24:47Z",
  "summary": {
    "total_functions": 96,
    "total_domains": 14,
    "avg_params_per_function": 1.02,
    "result_return_type_count": 2,
    "non_result_return_type_count": 94,
    "breaking_changes_count": 0
  },
  "functions_by_domain": [
    {"domain": "engine", "count": 12},
    {"domain": "device_registry", "count": 8}
  ],
  "evolution": [
    {"week": "2025-12-08", "added": 5, "removed": 1},
    {"week": "2025-12-01", "added": 3, "removed": 0}
  ]
}
```

## Troubleshooting

### Dashboard generation fails

**Problem**: Script fails with "jq: command not found"
```bash
sudo apt-get install jq bc  # Ubuntu/Debian
brew install jq             # macOS
```

**Problem**: Git history analysis is slow
- The script analyzes the last 12 weeks by default
- For large repos, this can take 10-30 seconds
- This is expected behavior

### Metrics show unexpected values

**Problem**: Function count seems wrong
- Verify contract files are up-to-date: `just gen-dart-bindings`
- Check that all contracts are in `core/src/ffi/contracts/`
- Ensure JSON files are valid: `jq . core/src/ffi/contracts/*.json`

**Problem**: Evolution timeline shows no data
- Requires git history with at least one commit touching contracts
- Fresh repos may show all zeros
- Run `just ffi-dashboard` after making some commits

## Integration with Other Tools

### Module Size Analysis

Combine contract metrics with module size analysis:
```bash
just ffi-dashboard
./scripts/analyze_module_sizes.sh
```

Compare:
- Number of functions per domain (contract dashboard)
- Lines of code per domain (module size analysis)
- Identify domains with high function count but low LOC (simple functions)
- Identify domains with low function count but high LOC (complex functions)

### Implementation Logs

Contract metrics complement implementation logs:
- Contract metrics: What exists now
- Implementation logs: How we got here
- Together: Complete picture of FFI evolution

See `docs/FFI_DEVELOPMENT.md` for implementation log usage.

## Future Enhancements

Planned improvements to FFI metrics:

1. **Call Frequency Heatmap**: Show which functions are called most often in production
2. **Error Rate by Function**: Track which FFI functions fail most frequently
3. **Performance Regression Detection**: Automatically detect when FFI calls get slower
4. **Cross-platform Metrics**: Compare FFI performance across Linux/Windows/macOS
5. **Contract Complexity Score**: Rate contracts by complexity (params, return types, errors)

## References

- [FFI Development Guide](./FFI_DEVELOPMENT.md) - Complete FFI workflow
- [FFI Tracing Guide](./FFI_TRACING.md) - Detailed tracing and DevTools integration
- [FFI Architecture](./ffi-architecture.md) - High-level FFI design
- [Module Splitting Guide](./MODULE_SPLITTING.md) - Splitting large modules
