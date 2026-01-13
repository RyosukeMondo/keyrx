# wasm-fix-verification - Tasks

## Overview
Fix WASM build issues, enhance build scripts for verification, and ensure WASM module is correctly built, loaded, and version-matched with the UI.

## Current Issues
- "WASM not available (run build:wasm)" error in UI
- "Using mock simulation (WASM not ready)" warning
- No automated verification of WASM build success
- No hash/version checking between WASM and UI

---

## Task 1: Diagnose WASM Build Issues

- [x] 1.1 Investigate current WASM build state
  - Files: `keyrx_ui/src/wasm/`, `keyrx_core/`, `scripts/lib/build-wasm.sh`
  - Check if WASM files exist in expected locations
  - Verify wasm-pack is installed and working
  - Check for build errors in WASM compilation
  - Document current WASM loading mechanism in useWasm.ts
  - Purpose: Understand why WASM is not loading
  - _Leverage: scripts/lib/build-wasm.sh, keyrx_ui/src/hooks/useWasm.ts_
  - _Requirements: Diagnose root cause of WASM failure_
  - _Prompt: Implement the task for spec wasm-fix-verification, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Build Engineer | Task: Investigate WASM build: 1) Check if keyrx_ui/src/wasm/pkg/ exists with .wasm file, 2) Run scripts/lib/build-wasm.sh and capture errors, 3) Check wasm-pack version, 4) Document findings in investigation report. | Restrictions: Don't modify code yet, diagnose only | Success: Root cause identified and documented | After completion: Mark task [-] as in-progress in tasks.md before starting, use log-implementation tool to record artifacts, then mark [x] complete_

- [x] 1.2 Fix WASM build configuration
  - Files: `keyrx_core/Cargo.toml`, `scripts/lib/build-wasm.sh`
  - Ensure crate-type includes "cdylib" for WASM
  - Fix any wasm-bindgen version mismatches
  - Update build script with proper error handling
  - Verify output directory structure
  - Purpose: Make WASM build succeed reliably
  - _Leverage: keyrx_core/Cargo.toml, wasm-pack documentation_
  - _Requirements: WASM builds successfully_
  - _Prompt: Implement the task for spec wasm-fix-verification, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust/WASM Developer | Task: Fix WASM build: 1) Check keyrx_core/Cargo.toml has crate-type = ["cdylib", "rlib"], 2) Add wasm-bindgen dependency if missing, 3) Fix build-wasm.sh to output to correct directory, 4) Test build succeeds. | Restrictions: Maintain compatibility with non-WASM builds | Success: scripts/lib/build-wasm.sh completes without errors | After completion: Mark task [-] as in-progress in tasks.md before starting, use log-implementation tool to record artifacts, then mark [x] complete_

---

## Task 2: Fix WASM Loading in UI

- [x] 2.1 Update useWasm hook for reliable loading
  - File: `keyrx_ui/src/hooks/useWasm.ts`
  - Fix import path to WASM module
  - Add proper async initialization
  - Handle loading states (loading, ready, error)
  - Add retry mechanism for transient failures
  - Expose detailed error messages for debugging
  - Purpose: Reliable WASM module loading
  - _Leverage: keyrx_ui/src/hooks/useWasm.ts existing implementation_
  - _Requirements: WASM loads reliably with proper error handling_
  - _Prompt: Implement the task for spec wasm-fix-verification, first run spec-workflow-guide to get the workflow guide then implement the task: Role: React/WASM Integration Developer | Task: Fix useWasm: 1) Update import path to built WASM module, 2) Use dynamic import with proper error catching, 3) Add loading/error/ready states, 4) Add retry logic (3 attempts, 1s delay), 5) Log detailed errors to console. | Restrictions: Keep hook interface backward compatible | Success: useWasm returns ready state when WASM loaded | After completion: Mark task [-] as in-progress in tasks.md before starting, use log-implementation tool to record artifacts, then mark [x] complete_

- [ ] 2.2 Add WASM status indicator to UI
  - File: `keyrx_ui/src/pages/SimulatorPage.tsx`, `ConfigPage.tsx`
  - Show WASM status: "Loading...", "Ready", "Error: [message]"
  - Green badge when ready, yellow when loading, red when error
  - Show helpful troubleshooting message on error
  - Purpose: Clear visibility of WASM state
  - _Leverage: keyrx_ui/src/hooks/useWasm.ts states_
  - _Requirements: Visual WASM status indicator_
  - _Prompt: Implement the task for spec wasm-fix-verification, first run spec-workflow-guide to get the workflow guide then implement the task: Role: React UI Developer | Task: Add WASM status badge to SimulatorPage and ConfigPage headers. Show: spinner + "Loading WASM...", green check + "WASM Ready", red X + "WASM Error: [msg]" + "Run: npm run build:wasm" hint. | Restrictions: Don't block page load on WASM, show graceful degradation | Success: Users see clear WASM status with actionable error messages | After completion: Mark task [-] as in-progress in tasks.md before starting, use log-implementation tool to record artifacts, then mark [x] complete_

---

## Task 3: Enhance Build Scripts

- [ ] 3.1 Add WASM build verification to build script
  - File: `scripts/lib/build-wasm.sh`
  - After build: verify .wasm file exists
  - Check file size is reasonable (> 100KB)
  - Generate SHA256 hash of .wasm file
  - Write hash to manifest file for version tracking
  - Exit with error if verification fails
  - Purpose: Ensure WASM build succeeded correctly
  - _Leverage: scripts/lib/build-wasm.sh, scripts/lib/common.sh_
  - _Requirements: Build script verifies WASM output_
  - _Prompt: Implement the task for spec wasm-fix-verification, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Build Automation Engineer | Task: Enhance build-wasm.sh: 1) Check .wasm file exists in output dir, 2) Verify size > 100KB, 3) Generate sha256sum, 4) Write to wasm-manifest.json with {hash, size, timestamp, version}. Exit 1 if any check fails. | Restrictions: Use standard tools (sha256sum), cross-platform compatible | Success: Build fails fast if WASM output is invalid | After completion: Mark task [-] as in-progress in tasks.md before starting, use log-implementation tool to record artifacts, then mark [x] complete_

- [ ] 3.2 Add WASM version matching check
  - File: `scripts/verify-wasm.sh` (create)
  - Compare WASM hash in UI bundle vs built WASM
  - Verify keyrx_core version matches WASM version
  - Check wasm-bindgen version compatibility
  - Report mismatches with clear error messages
  - Purpose: Catch version mismatches between components
  - _Leverage: scripts/lib/common.sh utilities_
  - _Requirements: Automated version matching verification_
  - _Prompt: Implement the task for spec wasm-fix-verification, first run spec-workflow-guide to get the workflow guide then implement the task: Role: DevOps Engineer | Task: Create verify-wasm.sh: 1) Read wasm-manifest.json, 2) Check WASM file hash matches manifest, 3) Extract version from keyrx_core/Cargo.toml, 4) Compare with WASM pkg/package.json version, 5) Report any mismatches. | Restrictions: Use existing script patterns, support --json output | Success: Script detects and reports version mismatches | After completion: Mark task [-] as in-progress in tasks.md before starting, use log-implementation tool to record artifacts, then mark [x] complete_

---

## Task 4: Add npm Scripts for WASM

- [ ] 4.1 Add convenient npm scripts for WASM operations
  - File: `keyrx_ui/package.json`
  - `build:wasm` - Build WASM module
  - `verify:wasm` - Run WASM verification
  - `clean:wasm` - Remove WASM build artifacts
  - `rebuild:wasm` - Clean + build
  - Update `build` script to include WASM build
  - Purpose: Easy WASM operations from npm
  - _Leverage: scripts/lib/build-wasm.sh, scripts/verify-wasm.sh_
  - _Requirements: npm scripts for all WASM operations_
  - _Prompt: Implement the task for spec wasm-fix-verification, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Frontend Build Config Developer | Task: Add npm scripts to keyrx_ui/package.json: "build:wasm": "../scripts/lib/build-wasm.sh", "verify:wasm": "../scripts/verify-wasm.sh", "clean:wasm": "rm -rf src/wasm/pkg", "rebuild:wasm": "npm run clean:wasm && npm run build:wasm", update "build" to include WASM. | Restrictions: Use relative paths to scripts, work on Linux/Mac | Success: npm run build:wasm works from keyrx_ui directory | After completion: Mark task [-] as in-progress in tasks.md before starting, use log-implementation tool to record artifacts, then mark [x] complete_

---

## Task 5: Integrate WASM Check into UAT

- [ ] 5.1 Add WASM verification step to uat.sh
  - File: `scripts/uat.sh`
  - After WASM build step: run verification
  - If verification fails: show clear error, abort UAT
  - If WASM hash changed: log informational message
  - Add `--skip-wasm` flag for quick UAT without WASM
  - Purpose: Ensure UAT always has working WASM
  - _Leverage: scripts/uat.sh, scripts/verify-wasm.sh_
  - _Requirements: UAT includes WASM verification_
  - _Prompt: Implement the task for spec wasm-fix-verification, first run spec-workflow-guide to get the workflow guide then implement the task: Role: CI/CD Engineer | Task: Update uat.sh: 1) After build_wasm(), call verify-wasm.sh, 2) If fail, show "WASM verification failed" and abort, 3) Add --skip-wasm flag that bypasses WASM steps, 4) Log WASM hash changes for tracking. | Restrictions: Keep existing UAT flow, fail fast on WASM issues | Success: UAT fails if WASM is broken, --skip-wasm works | After completion: Mark task [-] as in-progress in tasks.md before starting, use log-implementation tool to record artifacts, then mark [x] complete_

---

## Task 6: Add WASM Health Check CLI

- [ ] 6.1 Create CLI tool for WASM diagnostics
  - File: `scripts/wasm-health.sh` (create)
  - Check wasm-pack installation and version
  - Check Rust WASM target installed (wasm32-unknown-unknown)
  - Check keyrx_core builds for WASM target
  - Check WASM files exist in expected locations
  - Show summary: "WASM Health: OK" or "WASM Health: FAILED [reasons]"
  - Purpose: Quick diagnostics for WASM issues
  - _Leverage: scripts/lib/common.sh utilities_
  - _Requirements: Quick WASM health check command_
  - _Prompt: Implement the task for spec wasm-fix-verification, first run spec-workflow-guide to get the workflow guide then implement the task: Role: DevOps Tooling Developer | Task: Create wasm-health.sh: 1) Check wasm-pack --version, 2) Check rustup target list includes wasm32, 3) Check keyrx_core/Cargo.toml has cdylib, 4) Check WASM files exist, 5) Print summary with OK/FAIL status per check. | Restrictions: Use standard tools, provide actionable fix suggestions | Success: Running ./scripts/wasm-health.sh shows clear status | After completion: Mark task [-] as in-progress in tasks.md before starting, use log-implementation tool to record artifacts, then mark [x] complete_

---

## Task 7: Update Documentation

- [ ] 7.1 Update CLAUDE.md with WASM troubleshooting
  - File: `.claude/CLAUDE.md`
  - Add "WASM Troubleshooting" section
  - Document common issues and fixes
  - Document build scripts and npm commands
  - Add verification steps for AI agents
  - Purpose: Self-service WASM debugging
  - _Leverage: .claude/CLAUDE.md existing structure_
  - _Requirements: Comprehensive WASM documentation_
  - _Prompt: Implement the task for spec wasm-fix-verification, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Technical Writer | Task: Add "WASM Troubleshooting" section to CLAUDE.md: 1) Common errors and fixes table, 2) Build commands (npm run build:wasm), 3) Verification steps, 4) Health check usage. Place after "Troubleshooting" section. | Restrictions: Follow existing documentation style | Success: WASM issues can be self-diagnosed using docs | After completion: Mark task [-] as in-progress in tasks.md before starting, use log-implementation tool to record artifacts, then mark [x] complete_

---

## Task 8: Add Automated Tests

- [ ] 8.1 Add WASM integration tests
  - File: `keyrx_ui/src/__tests__/wasm.integration.test.ts` (create)
  - Test WASM module loads successfully
  - Test validation function works
  - Test simulation function works
  - Mock WASM for unit test isolation
  - Purpose: Catch WASM regressions automatically
  - _Leverage: tests/testUtils.tsx, Jest WASM mocking_
  - _Requirements: Automated WASM integration tests_
  - _Prompt: Implement the task for spec wasm-fix-verification, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Test Engineer | Task: Create wasm.integration.test.ts: 1) Test useWasm hook returns ready state, 2) Test validateConfig returns errors for invalid input, 3) Test runSimulation returns results, 4) Add mock WASM module for unit tests. | Restrictions: Tests must pass without real WASM (use mocks) | Success: CI runs WASM tests, catches loading failures | After completion: Mark task [-] as in-progress in tasks.md before starting, use log-implementation tool to record artifacts, then mark [x] complete_

- [ ] 8.2 Add build script tests
  - File: `scripts/tests/wasm-build.test.sh` (create)
  - Test build-wasm.sh produces output
  - Test verify-wasm.sh detects missing files
  - Test wasm-health.sh runs without error
  - Purpose: Catch script regressions
  - _Leverage: scripts/tests/ existing patterns_
  - _Requirements: Script test coverage_
  - _Prompt: Implement the task for spec wasm-fix-verification, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Shell Script Tester | Task: Create wasm-build.test.sh: 1) Run build-wasm.sh, check exit 0, 2) Delete .wasm file, run verify-wasm.sh, check exit 1, 3) Run wasm-health.sh, check output contains status. Use set -e for strict mode. | Restrictions: Clean up test artifacts, run in isolated env | Success: Script tests catch build/verify failures | After completion: Mark task [-] as in-progress in tasks.md before starting, use log-implementation tool to record artifacts, then mark [x] complete_

---

## Summary

| Task | Description | Files |
|------|-------------|-------|
| 1.1-1.2 | Diagnose and fix WASM build | keyrx_core/, build-wasm.sh |
| 2.1-2.2 | Fix WASM loading + status UI | useWasm.ts, SimulatorPage.tsx |
| 3.1-3.2 | Build verification + version matching | build-wasm.sh, verify-wasm.sh |
| 4.1 | npm scripts for WASM | package.json |
| 5.1 | Integrate into UAT | uat.sh |
| 6.1 | WASM health check CLI | wasm-health.sh |
| 7.1 | Documentation | CLAUDE.md |
| 8.1-8.2 | Automated tests | tests/ |

Total: 11 subtasks for comprehensive WASM fix and verification
