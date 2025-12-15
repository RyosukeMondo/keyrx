# FFI CI/CD Integration

This document describes the Continuous Integration and Continuous Deployment (CI/CD) setup for the KeyRX FFI layer, including quality gates, automated validation, and enforcement of FFI quality standards.

## Overview

The FFI CI/CD system ensures code quality through automated checks and quality gates. It validates contracts, enforces module size limits, detects breaking changes, and maintains test coverage standards.

## Workflows

### 1. FFI CI Workflow (`.github/workflows/ffi-ci.yml`)

The main FFI CI workflow runs comprehensive validation and testing on all FFI changes.

**Triggers:**
- Push to `main` branch (FFI-related files)
- Pull requests (FFI-related files)

**Jobs:**

#### Contract Validation
- Validates all `#[ffi_export]` annotations have matching contract JSON
- Ensures contracts match Rust function signatures
- Detects orphaned contracts (JSON without Rust code)
- Fails if contracts are out of sync

**Command:** `scripts/validate_ffi_contracts.sh --check --verbose`

#### Module Size Check
- Analyzes all FFI modules with `tokei`
- Enforces 500-line limit per module (excluding comments/blanks)
- Checks against allowlist for legitimate exceptions
- Generates HTML dashboard with violations

**Command:** `scripts/analyze_module_sizes.sh`

**Environment:** `MODULE_MAX_LINES=500`

#### FFI Tests
- Runs all FFI unit tests on Ubuntu and Windows
- Runs FFI integration tests
- Uses `cargo nextest` for parallel execution

**Commands:**
```bash
cargo nextest run --filter-expr 'package(keyrx) and test(ffi)' --profile ci-unit
cargo nextest run --filter-expr 'package(keyrx) and test(integration::ffi)' --profile ci-integration
```

#### Test Coverage
- Generates FFI test coverage report with `cargo-llvm-cov`
- Enforces minimum 80% line coverage
- Uploads coverage report as artifact
- Comments PR with coverage summary

**Command:**
```bash
cargo llvm-cov --all-features --lcov --output-path ffi-lcov.info -- --test-threads=1 ffi
```

**Threshold:** 80% minimum coverage

#### Tracing Overhead Check
- Runs tracing overhead benchmarks (if implemented)
- Ensures tracing adds <5% latency overhead
- Currently in development

**Target:** <5% performance overhead

#### Performance Regression
- Compares FFI benchmark results between PR and base branch
- Detects performance regressions (>10% slower)
- Currently in development

**Threshold:** <10% regression allowed

### 2. FFI Quality Gates (`.github/workflows/ffi-quality-gates.yml`)

Strict quality gates that must pass before a PR can be merged.

**Triggers:**
- Pull requests (FFI-related files)

**Quality Checks:**

| Check | Description | Required |
|-------|-------------|----------|
| **Formatting** | `cargo fmt --check` | ✅ Yes |
| **Clippy** | No warnings or errors | ✅ Yes |
| **Contract Validation** | All contracts valid | ✅ Yes |
| **Module Sizes** | All modules ≤500 lines | ✅ Yes |
| **Breaking Changes** | Version bump or `allow-breaking` label | ✅ Yes |
| **FFI Tests** | All tests pass | ✅ Yes |
| **Test Coverage** | ≥80% coverage | ✅ Yes |
| **Dart Bindings** | Bindings up-to-date | ✅ Yes |

**PR Comment:**
The workflow automatically comments on PRs with a quality gate report showing pass/fail status for each check.

### 3. FFI Breaking Changes (`.github/workflows/ffi-breaking-changes.yml`)

Detects and reports breaking changes in FFI contracts.

**Triggers:**
- Pull requests (FFI files or Cargo.toml)
- Push to `main` branch

**Detection:**
- Compares contract JSON between base and current branch
- Classifies changes as major/minor/patch
- Reports breaking changes with impact analysis

**Enforcement:**
- Blocks PR if major breaking changes detected
- Requires version bump in `core/Cargo.toml`
- OR requires `allow-breaking` label on PR

**Override:**
Add the `allow-breaking` label to a PR to explicitly acknowledge breaking changes without a version bump.

## Local Validation

Before pushing code, run these commands locally to validate:

```bash
# Validate all FFI contracts
./scripts/validate_ffi_contracts.sh --check --verbose

# Check module sizes
./scripts/analyze_module_sizes.sh

# Detect breaking changes (against main)
./scripts/detect_ffi_breaking_changes.sh --base origin/main --format markdown

# Run FFI tests
cd core && cargo nextest run --filter-expr 'test(ffi)'

# Check test coverage
cd core && cargo llvm-cov --all-features --text -- ffi

# Verify Dart bindings are up-to-date
just gen-dart-bindings
git diff --exit-code ui/lib  # Should show no changes
```

## CI Commands Reference

### Validation Scripts

All validation scripts support CI-optimized flags:

```bash
# Contract validation
scripts/validate_ffi_contracts.sh --check          # CI mode (strict)
scripts/validate_ffi_contracts.sh --fix            # Auto-fix contracts
scripts/validate_ffi_contracts.sh --verbose        # Detailed output

# Module size analysis
scripts/analyze_module_sizes.sh                    # Check sizes, fail on violations
MODULE_MAX_LINES=500 scripts/analyze_module_sizes.sh

# Breaking change detection
scripts/detect_ffi_breaking_changes.sh --ci                    # CI mode (JSON)
scripts/detect_ffi_breaking_changes.sh --base origin/main      # Compare against main
scripts/detect_ffi_breaking_changes.sh --format markdown       # Human-readable report

# Dart bindings generation
cargo run --manifest-path core/tools/generate_dart_bindings/Cargo.toml -- --check  # Validation only
cargo run --manifest-path core/tools/generate_dart_bindings/Cargo.toml -- --verbose  # Detailed logs
```

### Just Commands

Convenient `just` commands for local development:

```bash
just gen-dart-bindings      # Generate Dart FFI bindings
just ffi-watch              # Watch mode for FFI changes
just ffi-metrics            # Show FFI metrics
just ffi-dashboard          # Generate contract dashboard
just analyze-ffi            # Analyze implementation history
just diff-ffi-contracts     # Show contract differences
```

## Quality Standards

### Contract Validation

**Requirements:**
- Every `#[ffi_export]` function must have a matching contract JSON entry
- Contract JSON must match Rust function signature exactly
- No orphaned contracts (JSON without Rust code)

**Validation:**
- Runs on every PR
- Uses Python parser to extract annotations
- Validates JSON structure and completeness

**Fix:**
```bash
./scripts/validate_ffi_contracts.sh --fix
# OR
just gen-dart-bindings
```

### Module Size Limits

**Requirements:**
- All FFI modules must be ≤500 code lines (excluding comments/blanks)
- Exceptions must be in `.spec-workflow/config/module-size-allowlist.json`

**Allowlist Format:**
```json
{
  "allow": [
    {
      "path": "core/src/ffi/domains/observability.rs",
      "max_lines": 586,
      "reason": "Complex tracing setup, will split in future refactor"
    }
  ]
}
```

**Fix:**
Use the module splitter tool to split oversized modules:
```bash
# Preview split
just split-module core/src/ffi/exports_large.rs --preview

# Execute split
just split-module core/src/ffi/exports_large.rs --apply
```

### Breaking Changes

**Classification:**

| Change Type | Examples | Version Bump |
|-------------|----------|--------------|
| **Major (Breaking)** | Remove function, change signature, rename, change types | Major version |
| **Minor (Compatible)** | Add new function, add optional param | Minor version |
| **Patch** | Fix bugs, update docs | Patch version |

**Enforcement:**
- PRs with breaking changes require version bump in `core/Cargo.toml`
- OR explicit `allow-breaking` label
- CI blocks merge otherwise

**Guidance:**
See [FFI Versioning Policy](FFI_VERSIONING.md) for detailed guidance.

### Test Coverage

**Requirements:**
- Minimum 80% line coverage for FFI code
- All new FFI functions must have tests
- Integration tests for complex FFI interactions

**Measurement:**
```bash
cd core
cargo llvm-cov --all-features --html -- ffi
# Open target/llvm-cov/html/index.html
```

**Coverage by Type:**
- Unit tests: 90%+ for individual functions
- Integration tests: 80%+ for FFI boundary
- Property tests: For invariants (contracts match functions, etc.)

## Branch Protection

### Recommended Settings

Configure branch protection rules for `main` branch:

**Required Status Checks:**
- `FFI Quality Gate / quality-gate-check`
- `FFI Breaking Changes Detection / detect-breaking-changes`
- `FFI CI / contract-validation`
- `FFI CI / module-size-check`
- `FFI CI / ffi-tests`
- `FFI CI / test-coverage`

**Settings:**
- ✅ Require status checks to pass before merging
- ✅ Require branches to be up to date before merging
- ✅ Require conversation resolution before merging
- ✅ Do not allow bypassing the above settings

**Configuration (GitHub UI):**
1. Go to Repository Settings → Branches
2. Add branch protection rule for `main`
3. Enable "Require status checks to pass before merging"
4. Search and select the required checks above
5. Enable "Require branches to be up to date"
6. Save changes

**Configuration (GitHub CLI):**
```bash
# Enable branch protection with required checks
gh api repos/:owner/:repo/branches/main/protection -X PUT -f required_status_checks='{
  "strict": true,
  "contexts": [
    "FFI Quality Gate / quality-gate-check",
    "FFI Breaking Changes Detection / detect-breaking-changes",
    "FFI CI / contract-validation",
    "FFI CI / module-size-check",
    "FFI CI / ffi-tests",
    "FFI CI / test-coverage"
  ]
}'
```

## Artifacts and Reports

### Generated Artifacts

Each CI run generates artifacts available for download:

**FFI CI Workflow:**
- `contract-validation-report` - Validation errors and details
- `module-size-dashboard` - HTML dashboard with size violations
- `ffi-coverage-report` - LCOV coverage data and summary
- `ffi-contract-dashboard` - Contract metrics and visualizations

**Quality Gates Workflow:**
- `quality-gate-report` - Comprehensive quality report (Markdown)
- `breaking-changes-report` - Breaking change analysis (JSON + Markdown)
- `module-sizes.json` - Module size analysis data

**Breaking Changes Workflow:**
- `breaking-changes-report` - Breaking change detection results

### Viewing Reports

**In GitHub UI:**
1. Navigate to Actions → Select workflow run
2. Scroll to "Artifacts" section
3. Download and extract artifact
4. Open HTML files in browser

**Dashboard URLs:**
- Module Size Dashboard: `.spec-workflow/analysis/module-sizes.html`
- Contract Dashboard: `.spec-workflow/analysis/contract-dashboard.html`

**PR Comments:**
Automated comments on PRs include:
- Quality gate summary
- Module size violations
- Coverage percentage
- Breaking change warnings

## Troubleshooting

### Common Issues

#### ❌ Contract validation failed

**Error:** "Missing contract entry for function X"

**Fix:**
```bash
./scripts/validate_ffi_contracts.sh --fix
# OR
just gen-dart-bindings
git add -u && git commit -m "Regenerate FFI contracts"
```

#### ❌ Module size violation

**Error:** "Module exceeds 500 lines"

**Fix:** Split the module into smaller files:
```bash
# Use module splitter tool
just split-module path/to/large/module.rs --preview
just split-module path/to/large/module.rs --apply

# OR add to allowlist temporarily
# Edit .spec-workflow/config/module-size-allowlist.json
```

#### ❌ Breaking changes detected

**Error:** "Breaking changes detected without version bump"

**Fix (option 1 - bump version):**
```bash
# Edit core/Cargo.toml - bump version
# For breaking changes, bump major version
git add core/Cargo.toml
git commit -m "Bump version for breaking FFI changes"
```

**Fix (option 2 - add label):**
```bash
# Add 'allow-breaking' label to PR via GitHub UI
# This explicitly acknowledges the breaking change
```

#### ❌ Coverage below threshold

**Error:** "FFI test coverage is 72%, below 80% threshold"

**Fix:** Add tests for uncovered code:
```bash
# Generate coverage report locally
cd core
cargo llvm-cov --all-features --html -- ffi
# Open target/llvm-cov/html/index.html
# Write tests for red (uncovered) lines
```

#### ❌ Dart bindings out of date

**Error:** "Dart FFI bindings are out of sync with contracts"

**Fix:**
```bash
just gen-dart-bindings
git add ui/lib
git commit -m "Update Dart FFI bindings"
```

### CI Failures

If CI is failing but local validation passes:

1. **Check CI logs** - Click "Details" on failed check
2. **Verify tool versions** - CI may use different versions
3. **Run in clean environment** - Try in Docker or fresh clone
4. **Check file permissions** - Scripts must be executable
5. **Verify paths** - Some scripts expect to run from repo root

### Performance Issues

If CI runs are slow:

1. **Cache is working** - Check Rust cache hit/miss rates
2. **Parallel jobs** - Ensure jobs run in parallel where possible
3. **Optimize filters** - Only run on relevant file changes
4. **Benchmark locally** - Compare local vs CI execution time

## Status Badges

Add these badges to your README to show CI status:

```markdown
[![FFI CI](https://github.com/YOUR_ORG/keyrx/actions/workflows/ffi-ci.yml/badge.svg)](https://github.com/YOUR_ORG/keyrx/actions/workflows/ffi-ci.yml)
[![FFI Quality Gates](https://github.com/YOUR_ORG/keyrx/actions/workflows/ffi-quality-gates.yml/badge.svg)](https://github.com/YOUR_ORG/keyrx/actions/workflows/ffi-quality-gates.yml)
[![FFI Breaking Changes](https://github.com/YOUR_ORG/keyrx/actions/workflows/ffi-breaking-changes.yml/badge.svg)](https://github.com/YOUR_ORG/keyrx/actions/workflows/ffi-breaking-changes.yml)
```

Replace `YOUR_ORG` with your GitHub organization or username.

## Performance Targets

These are the performance targets enforced by CI:

| Metric | Target | Enforcement |
|--------|--------|-------------|
| Contract validation | <2s | Warning if slower |
| Module size analysis | <5s | Warning if slower |
| FFI tests (full suite) | <30s | Warning if slower |
| Coverage generation | <2min | Warning if slower |
| Codegen (full) | <5s | Future: regression check |
| Codegen (incremental) | <1s | Future: regression check |
| FFI call latency | <1ms | Future: regression check |
| Tracing overhead | <5% | Future: regression check |

## Future Enhancements

Planned improvements to FFI CI/CD:

1. **Performance Regression Detection**
   - Automated benchmarking on every PR
   - Historical tracking of performance metrics
   - Block PRs with >10% regression

2. **Automated Benchmarking**
   - Run `cargo criterion` on FFI benchmarks
   - Compare against baseline
   - Generate performance reports

3. **Security Scanning**
   - Scan FFI code for common vulnerabilities
   - Check for unsafe code patterns
   - Validate memory safety

4. **Documentation Generation**
   - Auto-generate FFI API docs
   - Publish to GitHub Pages
   - Update on every release

5. **Contract Evolution Dashboard**
   - Track FFI contract changes over time
   - Visualize API surface growth
   - Breaking change history

## Related Documentation

- [FFI Development Guide](FFI_DEVELOPMENT.md) - Development workflow
- [FFI Migration Guide](FFI_MIGRATION_GUIDE.md) - Migrating to AI-native workflow
- [FFI Versioning Policy](FFI_VERSIONING.md) - Breaking change guidelines
- [FFI Testing Strategy](FFI_TESTING.md) - Testing best practices
- [FFI Tracing Guide](FFI_TRACING.md) - Observability and debugging

## Support

If you encounter issues with FFI CI/CD:

1. Check this documentation first
2. Review CI logs for error messages
3. Run validation locally to reproduce
4. File an issue with details (logs, error messages, steps to reproduce)
5. Ask in team chat for quick help

## Changelog

- **2025-12-15**: Initial FFI CI/CD setup
  - Created ffi-ci.yml workflow
  - Created ffi-quality-gates.yml workflow
  - Integrated all FFI validation tools
  - Added comprehensive documentation
