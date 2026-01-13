# CI Caching Strategy

This document explains the caching strategy used in the CI/CD pipeline to optimize build and test times.

## Overview

The CI pipeline implements multi-level caching to reduce redundant work and speed up incremental builds. Proper cache invalidation ensures that changes are detected while maximizing cache reuse.

## Cache Types

### 1. Rust Cargo Cache

**Cached Paths:**
- `~/.cargo/registry` - Downloaded crate registry
- `~/.cargo/git` - Git dependencies
- `~/.cargo/bin` - Installed binaries (cargo-nextest, cargo-tarpaulin, etc.)
- `target/` - Compiled artifacts

**Cache Key Strategy:**
```yaml
key: ${{ runner.os }}-cargo-<job>-${{ hashFiles('**/Cargo.lock') }}-${{ hashFiles('**/Cargo.toml') }}
restore-keys:
  - ${{ runner.os }}-cargo-<job>-${{ hashFiles('**/Cargo.lock') }}-
  - ${{ runner.os }}-cargo-<job>-
  - ${{ runner.os }}-cargo-
```

**Invalidation:**
- Primary: Changes to `Cargo.lock` or `Cargo.toml` files
- Fallback: Restore from partial matches (same lock file, same job)
- Final fallback: Any cargo cache for the OS

**Jobs Using Cargo Cache:**
- `build-and-verify` - Main build and verification
- `e2e-playwright-tests` - E2E test daemon builds
- `test-docs` - Documentation testing
- `virtual-e2e-tests` - Virtual E2E tests
- `api-contract-tests` - API contract validation

### 2. npm Dependencies Cache

**Cached Paths:**
- `keyrx_ui/node_modules` - Installed npm packages

**Cache Key Strategy:**
```yaml
key: ${{ runner.os }}-npm-${{ hashFiles('keyrx_ui/package-lock.json') }}
restore-keys:
  - ${{ runner.os }}-npm-
```

**Invalidation:**
- Primary: Changes to `package-lock.json`
- Fallback: Any npm cache for the OS

**Jobs Using npm Cache:**
- `frontend-tests` - Unit/integration tests (all shards)
- `frontend-accessibility` - Accessibility audits
- `frontend-coverage` - Coverage analysis

**Note:** Node.js setup also uses built-in npm caching via `cache: 'npm'` parameter.

### 3. Vitest Test Results Cache

**Cached Paths:**
- `keyrx_ui/node_modules/.vitest` - Vitest internal cache
- `keyrx_ui/.vitest` - Test results and analysis

**Cache Key Strategy:**
```yaml
key: ${{ runner.os }}-vitest-<job>-${{ hashFiles('keyrx_ui/package-lock.json') }}-${{ hashFiles('keyrx_ui/vitest*.config.ts') }}-${{ hashFiles('keyrx_ui/src/**/*.{ts,tsx}') }}
restore-keys:
  - ${{ runner.os }}-vitest-<job>-${{ hashFiles('keyrx_ui/package-lock.json') }}-${{ hashFiles('keyrx_ui/vitest*.config.ts') }}-
  - ${{ runner.os }}-vitest-<job>-${{ hashFiles('keyrx_ui/package-lock.json') }}-
  - ${{ runner.os }}-vitest-<job>-
```

**Invalidation:**
- Primary: Changes to source files (`src/**/*.{ts,tsx}`)
- Secondary: Changes to Vitest config files
- Tertiary: Changes to package-lock.json
- Fallback: Restore from partial matches

**Benefits:**
- Vitest can skip unchanged test files
- Module transformation cache is preserved
- Faster test execution on incremental changes

**Jobs Using Vitest Cache:**
- `frontend-tests` - Separate cache per shard
- `frontend-accessibility` - Dedicated a11y cache
- `frontend-coverage` - Dedicated coverage cache

### 4. Playwright Browser Binaries Cache

**Cached Paths:**
- `~/.cache/ms-playwright` - Downloaded browser binaries

**Cache Key Strategy:**
```yaml
key: ${{ runner.os }}-playwright-${{ hashFiles('keyrx_ui/package-lock.json') }}
restore-keys:
  - ${{ runner.os }}-playwright-
```

**Invalidation:**
- Primary: Changes to `package-lock.json` (Playwright version change)
- Fallback: Any playwright cache for the OS

**Installation Logic:**
```yaml
- name: Install Playwright browsers
  if: steps.playwright-cache.outputs.cache-hit != 'true'
  run: npx playwright install chromium --with-deps

- name: Install Playwright dependencies (cache hit)
  if: steps.playwright-cache.outputs.cache-hit == 'true'
  run: npx playwright install-deps chromium
```

**Benefits:**
- Skips downloading ~200MB Chromium binary on cache hit
- Still installs system dependencies (libglib, etc.) when needed
- Reduces E2E test job startup time by ~2-3 minutes

**Jobs Using Playwright Cache:**
- `e2e-playwright-tests` - E2E tests with Playwright

## Cache Size Considerations

### Estimated Cache Sizes
- Cargo cache: ~500MB - 2GB (varies by dependencies)
- npm node_modules: ~300MB - 500MB
- Vitest cache: ~10MB - 50MB
- Playwright browsers: ~200MB - 300MB per browser

### GitHub Actions Cache Limits
- **Total cache limit per repository:** 10GB
- **Unused caches are evicted after 7 days**
- **Caches are scoped by branch** (main branch caches available to all PRs)

### Cache Management
- Each job uses a unique cache key prefix to prevent conflicts
- Restore keys allow fallback to similar caches
- Cargo caches include job name to avoid cross-contamination
- Source file hashes in Vitest cache ensure invalidation on code changes

## Performance Impact

### Expected Speedup (Cache Hit vs Cold)

| Cache Type | Cold Build | Cache Hit | Speedup |
|------------|-----------|-----------|---------|
| Cargo (full) | ~10-15 min | ~2-3 min | 5-7x |
| npm | ~2-3 min | ~30s | 4-6x |
| Vitest | ~30s | ~10s | 3x |
| Playwright | ~3 min | ~30s | 6x |

### Overall CI Impact
- **Cold run (no cache):** ~45 minutes
- **Warm run (all caches hit):** ~20-25 minutes
- **Target:** ≤30 minutes average

## Cache Invalidation Rules

### When Caches Are Invalidated

1. **Dependency Changes**
   - `Cargo.lock` or `Cargo.toml` modified → Cargo cache miss
   - `package-lock.json` modified → npm cache miss

2. **Configuration Changes**
   - `vitest*.config.ts` modified → Vitest cache miss
   - Playwright version in package.json → Browser cache miss

3. **Source Code Changes**
   - `src/**/*.{ts,tsx}` modified → Vitest cache partial miss (affected files only)
   - Rust source changes don't affect Cargo cache (recompiles from cache)

4. **Manual Invalidation**
   - Delete cache via GitHub UI: Settings → Actions → Caches
   - Update cache key version in workflow file

## Troubleshooting

### Cache Not Restoring

**Symptoms:** Jobs always run as "cold" builds despite no changes.

**Possible Causes:**
1. Cache keys have changed (check workflow file)
2. Cache expired (7-day limit)
3. Cache storage exceeded 10GB limit
4. Branch isolation (PR from fork doesn't have access to main cache)

**Solutions:**
1. Check GitHub Actions cache tab for available caches
2. Ensure `restore-keys` provides fallback options
3. Verify hashFiles() patterns match actual file locations
4. Push to main branch to populate cache for PRs

### Cache Stale or Corrupted

**Symptoms:** Builds fail with unexpected errors after cache restore.

**Possible Causes:**
1. Cargo cache contains outdated artifacts
2. node_modules corrupted during cache save
3. Vitest cache referencing old file locations

**Solutions:**
1. Clear cache and rebuild from scratch
2. Add cache version suffix to key (e.g., `v2-cargo-...`)
3. For Cargo: `cargo clean` before build
4. For npm: Use `npm ci` instead of `npm install`

### Excessive Cache Misses

**Symptoms:** Cache hit rate is low despite few changes.

**Possible Causes:**
1. Cache key too specific (includes volatile data)
2. Missing restore-keys fallbacks
3. File hash patterns too broad

**Solutions:**
1. Review cache key strategy
2. Add more restore-keys for partial matches
3. Use `hashFiles('**/Cargo.lock')` instead of `hashFiles('**/*')`
4. Monitor cache hit rates in CI logs

## Monitoring Cache Performance

### Check Cache Hit Rate

View cache restore logs in GitHub Actions:
```
Cache restored from key: Linux-cargo-main-abc123...
```

vs

```
Cache not found for input keys: Linux-cargo-main-abc123...
Attempting restore from restore keys...
Cache restored from key: Linux-cargo-main-
```

### Cache Metrics to Track
1. **Hit rate:** Percentage of jobs with cache hits
2. **Size:** Total cache size per job/branch
3. **Build time:** Compare cold vs warm builds
4. **Eviction rate:** How often caches are evicted

### GitHub Actions Cache API

Query cache statistics:
```bash
gh api repos/:owner/:repo/actions/caches
```

## Best Practices

1. **Use specific cache keys with fallbacks**
   - Primary key should be specific (all relevant hashes)
   - Restore keys provide progressive fallback
   - Never use generic keys like `cargo-cache`

2. **Separate caches by job type**
   - Different jobs have different needs
   - Prevents one job from corrupting another's cache
   - Allows independent eviction

3. **Include configuration in cache key**
   - Vitest config changes should invalidate test cache
   - Cargo.toml changes should trigger rebuild

4. **Monitor cache sizes**
   - Large caches slow down restore
   - Consider excluding non-essential paths
   - Use `--locked` flags to ensure reproducibility

5. **Document cache strategy**
   - Keep this file updated
   - Explain non-obvious cache keys
   - Document expected performance impact

## Future Improvements

### Potential Optimizations
1. **Rust incremental compilation cache**
   - Cache `target/debug/incremental` separately
   - Requires careful invalidation strategy

2. **Test result caching**
   - Store test results by commit SHA
   - Skip unchanged tests entirely

3. **Cross-platform cache sharing**
   - Platform-independent artifacts (wasm)
   - Requires careful path handling

4. **Cache compression**
   - Use zstd compression for large caches
   - Trade CPU time for network transfer

### Metrics to Track
- Cache hit rate over time
- Average cache restore time
- Cache size growth trends
- CI time reduction from caching

## References

- [GitHub Actions Cache Documentation](https://docs.github.com/en/actions/using-workflows/caching-dependencies-to-speed-up-workflows)
- [Cargo Cache Best Practices](https://doc.rust-lang.org/cargo/guide/cargo-home.html)
- [Playwright Browser Cache](https://playwright.dev/docs/ci#caching-browsers)
- [Vitest Performance](https://vitest.dev/guide/improving-performance.html)
