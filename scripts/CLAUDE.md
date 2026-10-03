# Scripts Documentation (MECE Reorganization)

## Introduction

This directory contains automation scripts for building, testing, verifying, and launching the keyrx workspace. Scripts have been reorganized following MECE (Mutually Exclusive, Collectively Exhaustive) and SRP (Single Responsibility Principle) principles.

**Design Principles:**
- **MECE**: Each script has a single, non-overlapping responsibility
- **SRP**: One command = one purpose
- **Consistent Interface**: All scripts support common flags (`--error`, `--json`, `--quiet`, `--log-file`)
- **Predictable Output**: Standardized status markers and log formats
- **Fail Fast**: Scripts abort on first error with clear error messages

## Script Reference

Every script documents its flags and usage: run `scripts/<name>.sh --help`.

## Output Format Specification

### Status Markers

```
=== accomplished ===  # Operation succeeded (green)
=== failed ===        # Operation failed (red)
=== warning ===       # Completed with warnings (yellow)
```

### Log Format

```
[YYYY-MM-DD HH:MM:SS] [LEVEL] message
```

Levels: `[INFO]` (blue), `[ERROR]` (red), `[WARN]` (yellow), `[DEBUG]` (no color)

### Exit Codes

| Code | Meaning |
|------|---------|
| 0 | Success |
| 1 | Failure |
| 2 | Missing required tool |

## Troubleshooting

### UAT shows old UI

This was the original problem. Use the new unified `uat.sh` which **always builds the UI**:

```bash
./scripts/uat.sh                      # Builds UI before starting daemon
./scripts/uat.sh --rebuild            # Force clean rebuild if needed
```

### Pre-commit hook blocks commit

```bash
./scripts/verify.sh                   # See detailed errors
cargo fmt                             # Auto-fix formatting
cargo clippy --fix                    # Auto-fix clippy warnings
```

### Setup issues

```bash
./scripts/setup.sh --check            # Check current status
./scripts/setup.sh --linux            # Fix Linux setup only
```

### Build failures

```bash
./scripts/build.sh                    # See build errors
cargo build --workspace 2>&1          # Direct cargo output
```

## For AI Agents

**Key Points:**
1. All scripts support `--json` for machine-parseable output
2. Exit codes: 0=success, 1=failure, 2=missing tool
3. Status markers (`=== accomplished ===`, `=== failed ===`)
4. Log files in `scripts/logs/` with epoch timestamps
5. **Use `uat.sh` for UAT** - it builds the UI unlike the old UAT.sh

**Recommended Workflow:**
```bash
./scripts/setup.sh --check            # 1. Verify environment
./scripts/verify.sh --skip-coverage   # 2. Quick quality check
./scripts/uat.sh                      # 3. Full UAT with fresh UI
./scripts/uat.sh --verify             # 4. Verify daemon is working
```

**Error Handling:**
- Always check exit codes
- Parse JSON output with `jq` for decision-making
- Read log files in `scripts/logs/` for debugging
