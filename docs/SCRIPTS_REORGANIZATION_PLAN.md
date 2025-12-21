# Scripts Reorganization Plan - MECE Aligned

## 🎯 Goal
Reorganize scripts into clear MECE hierarchy with consistent naming

## 📊 Current Problems

### Naming Inconsistencies
```
❌ verify_bindings.py        # verb_noun
❌ validate_contracts.sh      # different verb for same concept
❌ generate_errors.sh         # verb_noun
❌ pre-commit-ssot-check.sh   # context-noun-verb
❌ module_sizes.sh            # noun_noun (no verb!)
```

### Directory Chaos
```
❌ scripts/ffi/verify_bindings.py
❌ scripts/ffi/validate_contracts.sh
❌ scripts/docs/generate_errors.sh
❌ scripts/metrics/contract_dashboard.sh
❌ verify_fixes.sh (in root!)
```

---

## ✅ Proposed MECE Structure

### Principle: `<action>/<target>.sh`

```
scripts/
├── verify/                    # VERIFICATION (check state, fail if wrong)
│   ├── bindings.py           # FFI Dart bindings sync
│   ├── contracts.sh          # Rust #[ffi_export] → JSON contracts
│   ├── registry.sh           # CLI persistence
│   ├── ssot.sh               # SSOT violations
│   ├── breaking-changes.sh   # FFI breaking changes
│   ├── all.sh                # Run all verifications
│   └── README.md             # Explain verification pipeline
│
├── generate/                  # GENERATION (create new files)
│   ├── bindings.sh           # Generate Dart FFI bindings
│   ├── docs-errors.sh        # Generate error documentation
│   ├── docs-api.sh           # Generate API documentation
│   ├── coverage.sh           # Generate coverage reports
│   ├── metrics-dashboard.sh  # Generate contract metrics
│   ├── metrics-history.sh    # Generate velocity analysis
│   ├── all.sh                # Run all generators
│   └── README.md             # Explain generation pipeline
│
├── check/                     # STATIC ANALYSIS (report, don't fail)
│   ├── module-sizes.sh       # Check 500-line limit
│   ├── performance.sh        # Benchmark performance
│   └── README.md
│
├── test/                      # TEST EXECUTION
│   ├── daemon.sh             # Daemon integration tests
│   ├── e2e.sh                # End-to-end tests
│   ├── contracts.sh          # API contract tests
│   └── README.md
│
├── build/                     # BUILD ORCHESTRATION
│   ├── main.sh               # Main build script
│   ├── prebuild.sh           # Pre-build checks
│   ├── flutter.sh            # Flutter build
│   └── README.md
│
├── setup/                     # INSTALLATION/SETUP
│   ├── dev-environment.sh    # Setup dev environment
│   ├── ffi-tools.sh          # Setup FFI tools
│   └── ci/                   # CI-specific setup
│       ├── flutter.sh
│       └── jules.sh
│
├── dev/                       # DEVELOPMENT HELPERS
│   ├── watch-ffi.sh          # Watch FFI changes
│   ├── jules-manager.py      # Jules daemon manager
│   └── README.md
│
└── hooks/                     # GIT HOOKS
    ├── pre-commit.sh         # Pre-commit hook
    └── README.md
```

---

## 🔄 Migration Map

### Verification Scripts
```bash
scripts/ffi/verify_bindings.py              → scripts/verify/bindings.py
scripts/ffi/validate_contracts.sh           → scripts/verify/contracts.sh
scripts/verify_registry_consistency.sh      → scripts/verify/registry.sh
scripts/pre-commit-ssot-check.sh            → scripts/verify/ssot.sh
scripts/ffi/detect_breaking_changes.sh      → scripts/verify/breaking-changes.sh
verify_fixes.sh                             → scripts/verify/all.sh
```

### Generation Scripts
```bash
# Bindings generation now in Rust tool, script just calls it
core/tools/generate_dart_bindings/          → (keep as-is)
scripts/docs/generate_errors.sh             → scripts/generate/docs-errors.sh
scripts/docs/generate_api_docs.sh           → scripts/generate/docs-api.sh
ui/scripts/generate_coverage_report.sh      → scripts/generate/coverage.sh
scripts/metrics/contract_dashboard.sh       → scripts/generate/metrics-dashboard.sh
scripts/metrics/ffi_history.sh              → scripts/generate/metrics-history.sh
scripts/generate-zenn-articles.py           → DELETE (incomplete stub)
```

### Check Scripts
```bash
scripts/metrics/module_sizes.sh             → scripts/check/module-sizes.sh
scripts/metrics/benchmark.sh                → scripts/check/performance.sh
```

### Test Scripts
```bash
scripts/test/test_daemon.sh                 → scripts/test/daemon.sh
scripts/test/test_daemon_auto.sh            → scripts/test/daemon.sh (merge)
scripts/e2e_test_headless.sh                → scripts/test/e2e.sh
scripts/test-api-contracts.sh               → scripts/test/contracts.sh
```

### Build Scripts
```bash
scripts/build/main.sh                       → (keep)
scripts/pre-build.sh                        → scripts/build/prebuild.sh
scripts/build/flutter_prebuild.sh           → scripts/build/flutter.sh
```

### Setup Scripts
```bash
scripts/setup/dev_environment.sh            → (keep)
scripts/setup/ffi_tools.sh                  → (keep)
scripts/setup/ci/*                          → (keep)
```

### Dev Scripts
```bash
scripts/dev/ffi_watch.sh                    → scripts/dev/watch-ffi.sh
scripts/dev/jules_manager.py                → (keep)
scripts/dev/jules_safe.py                   → (merge into jules_manager.py)
```

---

## 📝 Updated Justfile Wiring

### Before (inconsistent)
```makefile
gen-dart-bindings:
    cargo run --release --manifest-path core/tools/generate_dart_bindings/Cargo.toml

verify-bindings:
    python3 scripts/ffi/verify_bindings.py

docs-errors:
    ./scripts/docs/generate_errors.sh

ui-coverage:
    ./ui/scripts/generate_coverage_report.sh
```

### After (consistent MECE)
```makefile
# VERIFICATION
verify-bindings:
    @./scripts/verify/bindings.py

verify-contracts:
    @./scripts/verify/contracts.sh

verify-ssot:
    @./scripts/verify/ssot.sh

verify-all:
    @./scripts/verify/all.sh

# GENERATION
gen-bindings:
    @./scripts/generate/bindings.sh

gen-docs:
    @./scripts/generate/docs-errors.sh
    @./scripts/generate/docs-api.sh

gen-coverage:
    @./scripts/generate/coverage.sh

gen-metrics:
    @./scripts/generate/metrics-dashboard.sh
    @./scripts/generate/metrics-history.sh

gen-all:
    @./scripts/generate/all.sh

# CHECKS
check-module-sizes:
    @./scripts/check/module-sizes.sh

check-performance:
    @./scripts/check/performance.sh

# TESTS
test-daemon:
    @./scripts/test/daemon.sh

test-e2e:
    @./scripts/test/e2e.sh

test-contracts:
    @./scripts/test/contracts.sh

# BUILD
build:
    @./scripts/build/main.sh

prebuild:
    @./scripts/build/prebuild.sh

# DEVELOPMENT
dev-watch:
    @./scripts/dev/watch-ffi.sh
```

---

## 🎯 Naming Convention Rules

### Format: `<action>/<target>.<ext>`

**Actions (Verbs):**
- `verify/` - Check correctness, fail if wrong (exit code != 0)
- `generate/` - Create new files from source
- `check/` - Analyze and report, don't fail (exit code = 0)
- `test/` - Execute tests
- `build/` - Compile/package
- `setup/` - Install/configure
- `dev/` - Development helpers

**Targets (Nouns):**
- Single word preferred: `bindings`, `contracts`, `coverage`
- Hyphenated for clarity: `breaking-changes`, `module-sizes`
- NO underscores in filenames (use hyphens)

**File Extensions:**
- `.sh` - Shell scripts
- `.py` - Python scripts
- `.js` - Node.js scripts (if any)

---

## 🔧 Implementation Steps

### Step 1: Create New Directory Structure
```bash
mkdir -p scripts/{verify,generate,check,test,hooks}
```

### Step 2: Move & Rename Files (with git mv)
```bash
# Verification
git mv scripts/ffi/verify_bindings.py scripts/verify/bindings.py
git mv scripts/ffi/validate_contracts.sh scripts/verify/contracts.sh
git mv scripts/verify_registry_consistency.sh scripts/verify/registry.sh
git mv scripts/pre-commit-ssot-check.sh scripts/verify/ssot.sh
git mv scripts/ffi/detect_breaking_changes.sh scripts/verify/breaking-changes.sh
git mv verify_fixes.sh scripts/verify/all.sh

# Generation
git mv scripts/docs/generate_errors.sh scripts/generate/docs-errors.sh
git mv ui/scripts/generate_coverage_report.sh scripts/generate/coverage.sh
git mv scripts/metrics/contract_dashboard.sh scripts/generate/metrics-dashboard.sh
git mv scripts/metrics/ffi_history.sh scripts/generate/metrics-history.sh
git rm scripts/generate-zenn-articles.py  # Remove stub

# Check
git mv scripts/metrics/module_sizes.sh scripts/check/module-sizes.sh
git mv scripts/metrics/benchmark.sh scripts/check/performance.sh

# Test
git mv scripts/test/test_daemon.sh scripts/test/daemon.sh
git mv scripts/e2e_test_headless.sh scripts/test/e2e.sh
git mv scripts/test-api-contracts.sh scripts/test/contracts.sh

# Build
git mv scripts/pre-build.sh scripts/build/prebuild.sh
git mv scripts/build/flutter_prebuild.sh scripts/build/flutter.sh

# Dev
git mv scripts/dev/ffi_watch.sh scripts/dev/watch-ffi.sh
```

### Step 3: Update All Script References
```bash
# Update justfile
# Update pre-commit hooks
# Update CI configs
# Update documentation
```

### Step 4: Add README.md to Each Directory
```bash
# scripts/verify/README.md - Explain verification pipeline
# scripts/generate/README.md - Explain generation pipeline
# etc.
```

### Step 5: Clean Up Empty Directories
```bash
rmdir scripts/ffi scripts/docs scripts/metrics
```

---

## 🔗 Script Dependencies Update

### Pre-commit Hook
```bash
# .git/hooks/pre-commit (update paths)
#!/bin/bash
./scripts/verify/contracts.sh || exit 1
./scripts/verify/ssot.sh || exit 1
```

### CI Pipeline
```yaml
# .github/workflows/ci.yml (update paths)
- name: Verify bindings
  run: ./scripts/verify/bindings.py

- name: Verify contracts
  run: ./scripts/verify/contracts.sh
```

---

## 📚 Documentation Updates

### Files to Update
- [ ] `README.md` - Update script paths
- [ ] `docs/ARCHITECTURE_VISUAL.md` - Update paths
- [ ] `docs/CONTRIBUTING.md` - Update paths
- [ ] `justfile` - Update all script paths
- [ ] `.github/workflows/*.yml` - Update CI paths
- [ ] Pre-commit hooks
- [ ] Each script that calls other scripts

---

## ✅ Benefits

### Before (Chaos)
```
❌ Inconsistent naming (verify/validate/check)
❌ Scattered across 5+ directories
❌ No clear categorization
❌ Hard to discover related scripts
❌ Duplicate concerns unclear
```

### After (MECE)
```
✅ Consistent verb/noun pattern
✅ Clear hierarchy by action type
✅ Easy to find: scripts/<action>/<target>.sh
✅ Related scripts grouped together
✅ MECE compliance obvious
```

---

## 🎯 Success Criteria

- [ ] All scripts follow `<action>/<target>.<ext>` pattern
- [ ] No underscores in filenames (use hyphens)
- [ ] Each action directory has README.md
- [ ] Justfile uses consistent naming
- [ ] All references updated
- [ ] Old directories removed
- [ ] Git history preserved (git mv)
- [ ] No broken links in documentation

---

**Estimated Time:** 2-3 hours for full reorganization + testing

**Risk:** LOW - Using `git mv` preserves history, old paths can be symlinked temporarily
