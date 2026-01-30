# Claude-Flow Worker Scripts

Incremental analysis scripts to improve worker success rates and avoid timeouts.

## Problem

The `optimize` and `testgaps` workers were failing with ~1% success rates due to:
- **Timeout issues**: Workers trying to analyze entire codebase in 5 minutes
- **Scope too broad**: Single-shot analysis of all files at once
- **No state persistence**: Every run starts from scratch

## Solution

Incremental analysis scripts that:
- ✅ **Analyze in chunks**: One module/crate at a time
- ✅ **Track progress**: JSON state files remember what's been analyzed
- ✅ **Resume capability**: Each run picks up where the last left off
- ✅ **Fast execution**: Complete in <2 minutes per chunk
- ✅ **Actionable results**: Focused, specific suggestions per module

## Scripts

### optimize-incremental.sh

Analyzes performance optimization opportunities incrementally.

**Features:**
- Analyzes one module per run (keyrx_core, keyrx_daemon, keyrx_compiler, keyrx_ui)
- Checks for:
  - N+1 query patterns (database loops)
  - Excessive `.clone()` calls
  - String allocations in hot paths
  - Missing React memoization (useCallback/useMemo)
- Stores results in `.claude-flow/metrics/optimize-results.json`
- Tracks progress in `.claude-flow/state/optimize-progress.json`

**Usage:**
```bash
.claude-flow/scripts/optimize-incremental.sh
```

**Output:**
```json
{
  "success": true,
  "module": "keyrx_core",
  "suggestionCount": 3,
  "message": "Analyzed keyrx_core: found 3 optimization opportunities"
}
```

### testgaps-incremental.sh

Analyzes test coverage gaps incrementally.

**Features:**
- Analyzes one crate per run (keyrx_core → keyrx_daemon → keyrx_compiler)
- Prioritizes critical paths first
- Checks for:
  - Untested public functions
  - Missing error handling tests
  - Integration test gaps
  - Coverage percentage (using cargo-tarpaulin if available)
- Stores results in `.claude-flow/metrics/testgaps-results.json`
- Tracks progress in `.claude-flow/state/testgaps-progress.json`

**Usage:**
```bash
.claude-flow/scripts/testgaps-incremental.sh
```

**Output:**
```json
{
  "success": true,
  "crate": "keyrx_core",
  "coverage": 85.3,
  "gapCount": 2,
  "message": "Analyzed keyrx_core: 85.3% coverage, found 2 gaps"
}
```

## Integration with Claude-Flow Daemon

The daemon can be configured to use these scripts instead of the default prompts:

### Option 1: Manual Trigger
```bash
# Test the scripts manually
npx @claude-flow/cli@latest hooks worker dispatch --trigger optimize --script .claude-flow/scripts/optimize-incremental.sh
npx @claude-flow/cli@latest hooks worker dispatch --trigger testgaps --script .claude-flow/scripts/testgaps-incremental.sh
```

### Option 2: Daemon Configuration
Update `.claude-flow/daemon-state.json` to point to these scripts (requires claude-flow v3.0.0-alpha.13+):

```json
{
  "config": {
    "workers": [
      {
        "type": "optimize",
        "script": ".claude-flow/scripts/optimize-incremental.sh",
        "intervalMs": 600000,
        "enabled": true
      },
      {
        "type": "testgaps",
        "script": ".claude-flow/scripts/testgaps-incremental.sh",
        "intervalMs": 1200000,
        "enabled": true
      }
    ]
  }
}
```

## State Files

### optimize-progress.json
```json
{
  "lastAnalyzed": "2026-01-30T12:30:00Z",
  "completedModules": ["keyrx_core", "keyrx_daemon"],
  "currentModule": "keyrx_compiler",
  "startedAt": "2026-01-30T10:00:00Z"
}
```

### testgaps-progress.json
```json
{
  "lastAnalyzed": "2026-01-30T12:35:00Z",
  "completedCrates": ["keyrx_core"],
  "currentCrate": "keyrx_daemon",
  "startedAt": "2026-01-30T10:00:00Z"
}
```

## Results Files

### optimize-results.json
Array of analysis results per module:
```json
[
  {
    "module": "keyrx_core",
    "timestamp": "2026-01-30T12:30:00Z",
    "suggestions": [
      {
        "type": "excessive-clones",
        "severity": "medium",
        "count": 75,
        "suggestion": "Found 75 clone() calls. Review for unnecessary memory allocations."
      }
    ]
  }
]
```

### testgaps-results.json
Array of coverage analysis per crate:
```json
[
  {
    "crate": "keyrx_core",
    "timestamp": "2026-01-30T12:35:00Z",
    "coverage": 85.3,
    "gaps": [
      {
        "type": "missing-error-tests",
        "severity": "high",
        "errorHandlers": 45,
        "errorTests": 18,
        "suggestion": "Only 18 error tests for 45 Result types. Add error path tests."
      }
    ]
  }
]
```

## Viewing Results

```bash
# View optimization suggestions
cat .claude-flow/metrics/optimize-results.json | jq '.[-1]'

# View test coverage gaps
cat .claude-flow/metrics/testgaps-results.json | jq '.[-1]'

# Check progress
cat .claude-flow/state/optimize-progress.json
cat .claude-flow/state/testgaps-progress.json
```

## Expected Improvements

| Worker | Before | Target | Improvement |
|--------|--------|--------|-------------|
| optimize | 1.5% success (4/268) | >90% success | 60x better |
| testgaps | 0.5% success (1/211) | >90% success | 180x better |

**Why:**
- ✅ No more timeouts (2min vs 10min)
- ✅ Focused scope (1 module vs entire codebase)
- ✅ Incremental progress (state persists)
- ✅ Actionable output (specific suggestions)

## Troubleshooting

### Script fails with "jq: command not found"
```bash
# Install jq
sudo apt-get install jq  # Debian/Ubuntu
brew install jq          # macOS
```

### State file corruption
```bash
# Reset state files
rm .claude-flow/state/*.json
# Scripts will auto-recreate on next run
```

### No results appearing
```bash
# Check script output manually
bash -x .claude-flow/scripts/optimize-incremental.sh
bash -x .claude-flow/scripts/testgaps-incremental.sh
```

## Development

To modify the analysis logic:

1. Edit `.claude-flow/scripts/optimize-incremental.sh` or `testgaps-incremental.sh`
2. Test manually: `bash .claude-flow/scripts/optimize-incremental.sh`
3. Check output in `.claude-flow/metrics/`
4. Verify state tracking in `.claude-flow/state/`

## License

Same as keyrx project.
