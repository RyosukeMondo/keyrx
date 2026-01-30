# Tasks Document - Installer & Debuggability Enhancement

## Phase 1: Version Synchronization (SSOT)

- [x] 1. Create version synchronization script
  - File: scripts/sync-version.sh
  - Read version from Cargo.toml workspace.package.version (SSOT)
  - Auto-update package.json version
  - Auto-update keyrx_daemon/keyrx_installer.wxs Version attribute
  - Auto-update scripts/build_windows_installer.ps1 $Version variable
  - Validate all versions match before exit
  - Purpose: Enforce single source of truth for version across all files
  - _Leverage: Cargo.toml, keyrx_ui/package.json, existing SSOT_VERSION.md patterns_
  - _Requirements: 1.1 (SSOT enforcement)_
  - _Prompt: Role: DevOps Engineer with expertise in build automation and version management | Task: Create a shell script that reads version from Cargo.toml and automatically synchronizes it to package.json, keyrx_installer.wxs, and build_windows_installer.ps1, with validation to ensure all match before exit | Restrictions: Must use Cargo.toml as single source of truth, fail fast on any mismatch, provide clear error messages, support both Unix and Windows environments | Success: Script successfully syncs all version files, validates consistency, fails with clear errors on mismatch, works on Windows/Linux_

- [x] 2. Add build-time version validation to build.rs
  - File: keyrx_daemon/build.rs
  - Read Cargo.toml version via cargo metadata
  - Read keyrx_ui/package.json version
  - Compare and fail compilation with clear error if mismatch
  - Set compile-time environment variables for version validation
  - Purpose: Prevent building with mismatched versions
  - _Leverage: keyrx_daemon/build.rs (existing build script), Cargo.toml metadata_
  - _Requirements: 1.2 (build-time validation)_
  - _Prompt: Role: Rust Build Engineer with expertise in cargo build scripts and compile-time validation | Task: Enhance build.rs to validate version consistency between Cargo.toml and package.json at compile time, failing the build with a clear error message if versions don't match | Restrictions: Must use cargo metadata to read versions, fail compilation immediately on mismatch, provide actionable error message pointing to sync-version.sh, don't break existing build functionality | Success: Build fails with clear error on version mismatch, existing build functionality preserved, error message guides user to fix_

- [x] 3. Create version verification script
  - File: scripts/version-check.ps1
  - Check Cargo.toml version
  - Check package.json version
  - Check keyrx_installer.wxs version
  - Check build_windows_installer.ps1 version
  - Check installed binary version (if exists)
  - Check running daemon version via API (if running)
  - Report all versions and highlight any mismatches
  - Purpose: Runtime verification of version consistency
  - _Leverage: scripts/test_installation.ps1 (existing verification patterns)_
  - _Requirements: 1.3 (runtime verification)_
  - _Prompt: Role: DevOps Engineer with expertise in PowerShell and system diagnostics | Task: Create a PowerShell script that checks version consistency across all source files, installed binaries, and running daemon, reporting any mismatches with clear output | Restrictions: Must handle missing files gracefully, check API only if daemon is running, use structured output (table format), highlight mismatches in red, exit with non-zero code on mismatch | Success: Script checks all version sources, reports clearly in table format, highlights mismatches, handles missing components gracefully_

## Phase 2: Enhanced Health Checks

- [x] 4. Add /api/diagnostics endpoint to daemon
  - File: keyrx_daemon/src/web/api/diagnostics.rs (new file)
  - Create comprehensive diagnostics endpoint
  - Return version info (daemon version, build time, git hash)
  - Return binary timestamp
  - Return running-as-admin status
  - Return key blocker hook status
  - Return platform info
  - Return config validation status
  - Return memory usage
  - Purpose: Provide comprehensive runtime diagnostics via API
  - _Leverage: keyrx_daemon/src/web/api/config.rs (existing API pattern), keyrx_daemon/src/version.rs_
  - _Requirements: 2.1 (health check API)_
  - _Prompt: Role: Backend Rust Developer with expertise in Axum web framework and system diagnostics | Task: Create a new /api/diagnostics endpoint that returns comprehensive system health information including version, binary timestamp, admin status, hook status, and platform info in JSON format | Restrictions: Must follow existing API patterns from config.rs, use structured JSON response, handle errors gracefully, don't expose sensitive information, ensure response time < 100ms | Success: Endpoint returns comprehensive diagnostics in JSON, follows existing patterns, performs fast, handles errors gracefully_

- [x] 5. Enhance /api/health endpoint
  - File: keyrx_daemon/src/web/api/config.rs (modify existing)
  - Add version field to health response
  - Add build_time field
  - Add admin_rights field
  - Add hook_installed field
  - Maintain backward compatibility
  - Purpose: Provide essential health info in existing endpoint
  - _Leverage: keyrx_daemon/src/web/api/config.rs, keyrx_daemon/src/version.rs_
  - _Requirements: 2.2 (enhanced health endpoint)_
  - _Prompt: Role: Backend Rust Developer with expertise in API versioning and backward compatibility | Task: Enhance the existing /api/health endpoint to include version, build_time, admin_rights, and hook_installed fields while maintaining backward compatibility | Restrictions: Must not break existing clients, add new fields to existing response structure, ensure JSON serialization works correctly, test with existing integration tests | Success: Health endpoint includes new fields, existing tests pass, backward compatible, JSON structure valid_

- [x] 6. Add startup version validation to daemon
  - File: keyrx_daemon/src/main.rs
  - Validate version consistency at startup
  - Log version, build time, git hash on startup
  - Log admin rights status
  - Log hook installation success/failure
  - Warn if any version inconsistencies detected
  - Purpose: Immediate feedback on version issues at daemon startup
  - _Leverage: keyrx_daemon/src/main.rs (existing startup code), keyrx_daemon/src/version.rs_
  - _Requirements: 2.3 (startup validation)_
  - _Prompt: Role: System Integration Engineer with expertise in Rust application initialization and logging | Task: Add version validation and comprehensive logging to daemon startup, logging version info, admin status, and hook installation status, with warnings for any inconsistencies | Restrictions: Must not prevent daemon from starting on warnings, log to both console and daemon.log, use structured logging with log crate, don't duplicate existing startup code | Success: Daemon logs comprehensive version info on startup, warnings are clear, doesn't block startup on warnings, logs to appropriate destinations_

## Phase 3: Installer Enhancements

- [x] 7. Add version pre-flight check to installer
  - File: keyrx_daemon/keyrx_installer.wxs
  - Add CustomAction to validate binary version matches MSI version
  - Check binary timestamp is recent (within 24 hours)
  - Fail installation with clear error if mismatch
  - Add condition to verify build is fresh
  - Purpose: Prevent installing mismatched or stale binaries
  - _Leverage: keyrx_daemon/keyrx_installer.wxs (existing CustomActions), INSTALLER_FIX.md patterns_
  - _Requirements: 3.1 (installer validation)_
  - _Prompt: Role: Windows Installer Engineer with expertise in WiX toolset and MSI CustomActions | Task: Add a pre-flight CustomAction to the WiX installer that validates the binary version matches the MSI version and timestamp is recent, failing installation with clear error on mismatch | Restrictions: Must use WiX CustomAction, run before InstallFiles, provide user-friendly error message, don't break existing installation flow, handle errors gracefully | Success: Installer validates binary before installation, fails with clear message on mismatch, doesn't break existing functionality_

- [x] 8. Enhance daemon stop logic in installer
  - File: keyrx_daemon/keyrx_installer.wxs
  - Improve StopDaemonBeforeUpgrade CustomAction
  - Add retry logic (3 attempts with 2 second delay)
  - Add timeout handling (fail after 10 seconds)
  - Log stop attempts to MSI log
  - Gracefully handle already-stopped daemon
  - Purpose: Reliable daemon stopping during upgrades
  - _Leverage: keyrx_daemon/keyrx_installer.wxs (existing StopDaemonBeforeUpgrade)_
  - _Requirements: 3.2 (reliable daemon stop)_
  - _Prompt: Role: Windows System Administrator with expertise in process management and WiX CustomActions | Task: Enhance the StopDaemonBeforeUpgrade CustomAction with retry logic, timeout handling, and comprehensive logging to ensure reliable daemon stopping during upgrades | Restrictions: Must use WiX CustomAction Execute attribute, retry up to 3 times with delays, timeout after 10 seconds, log to MSI log, handle already-stopped gracefully, don't break existing upgrade flow | Success: Daemon stops reliably during upgrades, retries work correctly, timeout prevents hanging, logs are comprehensive_

- [x] 9. Add post-install health check to installer
  - File: keyrx_daemon/keyrx_installer.wxs
  - Add CustomAction to verify installation success
  - Check binary exists and is correct version
  - Check daemon starts successfully
  - Check API responds to /api/health
  - Report installation success/failure clearly
  - Purpose: Immediate feedback on installation success
  - _Leverage: keyrx_daemon/keyrx_installer.wxs, scripts/test_installation.ps1 patterns_
  - _Requirements: 3.3 (post-install validation)_
  - _Prompt: Role: Windows Installer Engineer with expertise in WiX CustomActions and API testing | Task: Add a post-install CustomAction that validates installation success by checking binary existence, daemon startup, and API health, reporting clear success or failure | Restrictions: Must run after InstallFinalize, use WiX CustomAction, don't fail installation on daemon startup issues (warn instead), provide clear user message, test API with timeout | Success: Post-install check validates installation, reports clearly to user, handles failures gracefully without rolling back_

## Phase 4: Diagnostic Scripts

- [x] 10. Create installer health check script
  - File: scripts/installer-health-check.ps1
  - Verify MSI file integrity
  - Check binary version matches MSI version
  - Validate all required files present in MSI
  - Check admin rights availability
  - Test daemon stop/start functionality
  - Generate comprehensive report
  - Purpose: Diagnose installer issues before installation
  - _Leverage: scripts/test_installation.ps1, DIAGNOSTIC_BUILD_REPORT.md patterns_
  - _Requirements: 4.1 (installer diagnostics)_
  - _Prompt: Role: DevOps Engineer with expertise in PowerShell, MSI validation, and system diagnostics | Task: Create a PowerShell script that performs comprehensive pre-installation health checks including MSI integrity, version matching, file presence, admin rights, and daemon control, generating a detailed report | Restrictions: Must use PowerShell 5.1+ cmdlets, output structured report (table or JSON), highlight issues in red, provide actionable recommendations, handle missing MSI gracefully | Success: Script performs all health checks, generates clear report, highlights issues, provides fix recommendations, exits with appropriate code_

- [x] 11. Create installation diagnostic script
  - File: scripts/diagnose-installation.ps1
  - Check all version sources for consistency
  - Check binary timestamps (source vs installed)
  - Check daemon running state
  - Check admin rights (current user and daemon process)
  - Check file locks on daemon binary
  - Check Windows event log for daemon errors
  - Suggest fixes for common issues
  - Purpose: Diagnose installation and runtime issues
  - _Leverage: scripts/version-check.ps1, CRITICAL_DIAGNOSIS.md patterns_
  - _Requirements: 4.2 (installation diagnostics)_
  - _Prompt: Role: Windows System Administrator with expertise in PowerShell, process diagnostics, and troubleshooting | Task: Create a comprehensive diagnostic script that checks version consistency, binary timestamps, daemon state, admin rights, file locks, and event logs, suggesting fixes for common issues | Restrictions: Must handle missing components gracefully, check event logs for daemon errors, use Get-Process and file cmdlets, provide fix suggestions for each issue found, output structured report | Success: Script diagnoses all common issues, provides actionable fix suggestions, handles missing components, outputs clear report_

- [x] 12. Create force clean reinstall script
  - File: scripts/force-clean-reinstall.ps1
  - Stop daemon forcefully (kill process if needed)
  - Uninstall MSI cleanly
  - Remove all state files (~/.keyrx/*)
  - Clean build artifacts (target/release/*)
  - Rebuild UI (npm run build)
  - Rebuild daemon (cargo build --release)
  - Build fresh installer
  - Install new MSI
  - Verify installation with health checks
  - Purpose: Complete clean reinstall automation
  - _Leverage: REBUILD_SSOT.bat, COMPLETE_REINSTALL.ps1 patterns_
  - _Requirements: 4.3 (clean reinstall automation)_
  - _Prompt: Role: DevOps Engineer with expertise in PowerShell, build automation, and clean state management | Task: Create a comprehensive script that performs a complete clean reinstall including daemon stop, MSI uninstall, state cleanup, full rebuild, fresh install, and verification | Restrictions: Must require admin rights, prompt user for confirmation before destructive operations, use -Force parameters carefully, verify each step succeeded before proceeding, provide progress feedback, rollback on critical failures | Success: Script performs complete clean reinstall, verifies each step, provides progress feedback, handles errors gracefully with rollback_

## Phase 5: Integration & Testing

- [x] 13. Create version management integration test
  - File: keyrx_daemon/tests/version_consistency_test.rs
  - Test sync-version.sh script functionality
  - Test build.rs validation catches mismatches
  - Test version constants are correct at runtime
  - Test API returns correct version info
  - Purpose: Ensure version management system works end-to-end
  - _Leverage: keyrx_daemon/tests/version_verification_test.rs (existing), Cargo.toml metadata_
  - _Requirements: 5.1 (version management testing)_
  - _Prompt: Role: Rust Test Engineer with expertise in integration testing and build script testing | Task: Create comprehensive integration tests for version management including script execution, build validation, runtime constants, and API responses | Restrictions: Must test actual script execution, use cargo metadata to verify versions, test API with actual HTTP requests, handle test isolation properly, achieve 90%+ coverage | Success: Tests verify entire version management flow, catch version mismatches, test API integration, run reliably in CI, achieve high coverage_

- [x] 14. Create installer validation test suite
  - File: tests/installer_validation_test.rs
  - Test installer pre-flight checks work
  - Test daemon stop logic with retry/timeout
  - Test admin rights detection
  - Test post-install verification
  - Use mock or test MSI for validation
  - Purpose: Ensure installer enhancements work correctly
  - _Leverage: scripts/test_installation.ps1 patterns, WiX testing best practices_
  - _Requirements: 5.2 (installer testing)_
  - _Prompt: Role: QA Automation Engineer with expertise in installer testing and WiX validation | Task: Create comprehensive test suite for installer enhancements including pre-flight checks, daemon stop logic, admin detection, and post-install verification using test MSI or mocks | Restrictions: Must test without requiring actual installation, use mocks for destructive operations, test both success and failure scenarios, ensure tests run on CI, don't require admin rights for tests | Success: Tests cover all installer enhancements, test success and failure paths, run on CI without admin, use appropriate mocks_

- [x] 15. Create diagnostic scripts test suite
  - File: tests/diagnostic_scripts_test.ps1
  - Test installer-health-check.ps1 output
  - Test diagnose-installation.ps1 detection accuracy
  - Test force-clean-reinstall.ps1 with dry-run mode
  - Verify all scripts handle errors gracefully
  - Purpose: Ensure diagnostic scripts work reliably
  - _Leverage: scripts/test_installation.ps1 testing patterns_
  - _Requirements: 5.3 (diagnostic testing)_
  - _Prompt: Role: PowerShell Test Engineer with expertise in script testing and Pester framework | Task: Create comprehensive test suite for diagnostic scripts using Pester, testing output accuracy, error handling, and dry-run modes where applicable | Restrictions: Must use Pester framework, test without actual installation/uninstallation, use mocks for destructive operations, verify script output format, test error scenarios | Success: Tests cover all diagnostic scripts, verify output accuracy, test error handling, use Pester best practices, run reliably_

- [x] 16. Add CI/CD version validation
  - File: .github/workflows/ci.yml (modify existing)
  - Add step to run sync-version.sh --check (validate only)
  - Add step to verify all versions match
  - Fail CI if version mismatch detected
  - Run on all branches and PRs
  - Purpose: Prevent merging code with version mismatches
  - _Leverage: .github/workflows/ci.yml (existing CI configuration)_
  - _Requirements: 5.4 (CI/CD integration)_
  - _Prompt: Role: DevOps Engineer with expertise in GitHub Actions and CI/CD pipelines | Task: Enhance CI workflow to validate version consistency across all source files, failing the build if any mismatches are detected | Restrictions: Must run on all branches and PRs, use existing CI structure, add as early step to fail fast, provide clear error message, don't break existing CI jobs | Success: CI validates version consistency, fails fast on mismatch, provides clear errors, doesn't break existing workflow_

- [x] 17. Final integration and documentation
  - File: .spec-workflow/specs/installer-debuggability-enhancement/README.md
  - Document complete workflow for version updates
  - Document troubleshooting guide
  - Document diagnostic script usage
  - Document installer validation process
  - Update main project README if needed
  - Purpose: Comprehensive documentation for maintainers
  - _Leverage: SSOT_VERSION.md, INSTALLER_FIX.md (existing docs)_
  - _Requirements: All_
  - _Prompt: Role: Technical Writer with expertise in software documentation and developer guides | Task: Create comprehensive documentation covering version update workflow, troubleshooting guide, diagnostic script usage, and installer validation process, consolidating learnings from past issues | Restrictions: Must include step-by-step procedures, troubleshooting decision trees, example outputs, common errors and fixes, reference existing docs where appropriate | Success: Documentation is clear and comprehensive, covers all scenarios, includes examples, provides troubleshooting guidance, accessible to all team members_
