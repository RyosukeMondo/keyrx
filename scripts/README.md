# Scripts Directory

MECE-compliant script organization for KeyRx development workflows.

## 📁 Directory Structure

```
scripts/
├── verify/      # Verification (fail if wrong, exit != 0)
├── generate/    # Generation (create files from SSOT)
├── check/       # Analysis (report only, exit = 0)
├── test/        # Test execution
├── build/       # Build orchestration
├── dev/         # Development helpers
├── utils/       # Manual utilities
└── setup/       # Installation/setup
```

## 🎯 Quick Reference

### Verification Scripts

Verify correctness and fail if something is wrong (exit code != 0).

```bash
just verify-bindings    # Verify Dart FFI bindings are in sync
just verify-contracts   # Verify JSON contracts match Rust code
just verify-ssot        # Check for SSOT violations
just verify-registry    # Verify CLI persistence
just verify-all         # Run all verifications
```

**Details:** See [verify/README.md](verify/README.md)

### Generation Scripts

Create files from authoritative sources (SSOT).

```bash
just gen-dart-bindings  # Generate Dart FFI bindings
just gen-docs           # Generate error documentation
just gen-coverage       # Generate coverage report
just gen-metrics        # Generate metrics dashboard
```

**Details:** See [generate/README.md](generate/README.md)

### Check Scripts

Analyze and report without failing (exit code = 0).

```bash
just check-module-sizes    # Check 500-line limit
just check-performance     # Run benchmarks
```

**Details:** See [check/README.md](check/README.md)

### Test Scripts

Execute test suites.

```bash
just test-daemon      # Daemon integration tests
just test-e2e         # End-to-end tests
just test-contracts   # API contract tests
```

**Details:** See [test/README.md](test/README.md)

### Development Scripts

Helper scripts for development workflows.

```bash
just dev-watch        # Watch FFI files and auto-regenerate
./scripts/dev/log-implementation.sh  # Log FFI implementation
```

### Setup Scripts

Environment and tool installation.

```bash
just setup            # Full development environment setup
just setup-ffi-tools  # Install FFI-specific tools
```

## 📚 Naming Convention

All scripts follow the `<action>/<target>.<ext>` pattern:

- **Actions**: `verify/`, `generate/`, `check/`, `test/`, `build/`, `dev/`, `utils/`, `setup/`
- **Targets**: Hyphenated names (e.g., `module-sizes.sh`, `breaking-changes.sh`)
- **Extensions**: `.sh` (shell), `.py` (Python)

## 🔍 Finding Scripts

**By Category:**
```bash
ls scripts/verify/     # All verification scripts
ls scripts/generate/   # All generation scripts
```

**By Function:**
- Bindings verification: `scripts/verify/bindings.py`
- Contract validation: `scripts/verify/contracts.sh`
- Module size check: `scripts/check/module-sizes.sh`
- FFI watch mode: `scripts/dev/watch-ffi.sh`

## 📖 Documentation

Each category has its own README with detailed information:

- [verify/README.md](verify/README.md) - Verification pipeline layers
- [generate/README.md](generate/README.md) - Generation from SSOT
- [check/README.md](check/README.md) - Analysis and reporting
- [test/README.md](test/README.md) - Test execution
- [utils/README.md](utils/README.md) - Manual utilities

## 🎨 MECE Principles

This organization follows MECE (Mutually Exclusive, Collectively Exhaustive):

- **Mutually Exclusive**: Each script has a single, clear purpose
- **Collectively Exhaustive**: All development workflows are covered
- **No Duplication**: Each concern is addressed exactly once

## 🔗 Related Documentation

- [SCRIPTS_REORGANIZATION_COMPLETE.md](../docs/SCRIPTS_REORGANIZATION_COMPLETE.md) - Full reorganization summary
- [SCRIPTS_MECE_ANALYSIS.md](../docs/SCRIPTS_MECE_ANALYSIS.md) - MECE compliance analysis
- [CONTRIBUTING.md](../CONTRIBUTING.md) - Development workflow guide

---

**Note:** This organization was established 2025-12-21. All scripts follow consistent naming and organizational patterns for easy discovery and maintenance.
