# Requirements - Installer & Debuggability Enhancement

## 1. Version Synchronization (SSOT)

### 1.1 Single Source of Truth Enforcement
- **Requirement:** Cargo.toml `workspace.package.version` SHALL be the single source of truth
- **Rationale:** Eliminates manual sync errors that caused v0.1.3-v0.1.5 issues
- **Acceptance Criteria:**
  - Automated script syncs version to package.json, keyrx_installer.wxs, build_windows_installer.ps1
  - Script validates all versions match before exit
  - Script provides clear error on mismatch with fix instructions
  - Script works on both Windows (PowerShell/Git Bash) and Linux

### 1.2 Build-Time Validation
- **Requirement:** Build MUST fail if versions don't match across source files
- **Rationale:** Prevent building mismatched binaries that caused stale binary issues
- **Acceptance Criteria:**
  - build.rs reads and compares Cargo.toml vs package.json versions
  - Compilation fails immediately on mismatch
  - Error message is clear and actionable (points to sync-version.sh)
  - No silent failures or warnings that can be ignored

### 1.3 Runtime Verification
- **Requirement:** System SHALL detect and report version mismatches at runtime
- **Rationale:** Catch any deployment issues that slip through build validation
- **Acceptance Criteria:**
  - Script checks all version sources (files + binaries + running daemon)
  - Reports all versions in clear table format
  - Highlights mismatches in red with severity indicators
  - Provides fix recommendations for detected issues
  - Exits with non-zero code on mismatch for CI integration

## 2. Enhanced Health Checks

### 2.1 Comprehensive Diagnostics API
- **Requirement:** Daemon SHALL provide /api/diagnostics endpoint with full system state
- **Rationale:** Enable remote and automated health checking
- **Acceptance Criteria:**
  - Returns JSON with: version, build_time, git_hash, binary_timestamp, admin_status, hook_status, platform_info, memory_usage, config_validation_status
  - Response time < 100ms
  - Returns 200 OK when healthy, appropriate error codes when degraded
  - Follows existing API patterns (Axum, error handling)
  - No sensitive information exposed (no PII, secrets, internal paths)

### 2.2 Enhanced Health Endpoint
- **Requirement:** Existing /api/health endpoint SHALL include essential version info
- **Rationale:** Provide quick health check without breaking existing clients
- **Acceptance Criteria:**
  - Adds fields: version, build_time, admin_rights, hook_installed
  - Maintains backward compatibility (existing clients unaffected)
  - Follows semantic versioning for API changes
  - Existing integration tests pass without modification

### 2.3 Startup Validation
- **Requirement:** Daemon SHALL validate and log comprehensive version info on startup
- **Rationale:** Immediate visibility into potential issues
- **Acceptance Criteria:**
  - Logs version, build_time, git_hash on startup
  - Logs admin rights status (running as admin: yes/no)
  - Logs hook installation status (success/failure with reason)
  - Warns (not fails) on version inconsistencies
  - Logs to both console and daemon.log file
  - Uses structured logging (log crate with level/timestamp/component)

## 3. Installer Enhancements

### 3.1 Version Pre-Flight Validation
- **Requirement:** Installer SHALL validate binary version before installation
- **Rationale:** Prevent installing stale or mismatched binaries (v0.1.4 issue)
- **Acceptance Criteria:**
  - CustomAction validates binary version matches MSI version
  - CustomAction checks binary timestamp is recent (< 24 hours)
  - Installation fails with user-friendly error on mismatch
  - Error message includes binary version, MSI version, timestamp
  - Runs before InstallFiles phase (fail early)

### 3.2 Reliable Daemon Stopping
- **Requirement:** Installer SHALL reliably stop daemon during upgrades
- **Rationale:** Prevent file locking issues that blocked v0.1.4 updates
- **Acceptance Criteria:**
  - CustomAction attempts graceful stop (taskkill without /F)
  - Retries up to 3 times with 2-second delay between attempts
  - Uses force kill (/F) on final attempt
  - Timeout after 10 seconds total
  - Handles already-stopped daemon gracefully (not an error)
  - Logs all stop attempts to MSI log
  - Runs before RemoveExistingProducts (upgrade scenario)

### 3.3 Post-Install Validation
- **Requirement:** Installer SHALL verify installation success after completion
- **Rationale:** Immediate feedback on installation issues
- **Acceptance Criteria:**
  - CustomAction checks binary exists at install location
  - Verifies binary version matches MSI version
  - Attempts daemon startup and checks API health
  - Reports clear success/failure message to user
  - Does NOT rollback on daemon startup failure (warn only)
  - Runs after InstallFinalize phase
  - API health check has 10-second timeout

## 4. Diagnostic Scripts

### 4.1 Installer Health Check
- **Requirement:** Provide script to diagnose installer issues before installation
- **Rationale:** Enable users to self-diagnose and fix common problems
- **Acceptance Criteria:**
  - Verifies MSI file integrity (exists, size > 0, readable)
  - Checks binary version matches MSI expected version
  - Validates all required files present in MSI package
  - Checks current user has admin rights
  - Tests daemon stop/start functionality
  - Generates structured report (table or JSON format)
  - Highlights issues with color coding (red for errors, yellow for warnings)
  - Provides fix recommendations for each issue
  - Exits with code 0 (all checks pass) or 1 (issues found)

### 4.2 Installation Diagnostics
- **Requirement:** Provide script to diagnose runtime installation issues
- **Rationale:** Comprehensive troubleshooting for deployed systems
- **Acceptance Criteria:**
  - Checks version consistency across all sources
  - Compares binary timestamps (source vs installed)
  - Checks daemon running state (Get-Process)
  - Checks admin rights (current user + daemon process)
  - Checks file locks on daemon binary
  - Queries Windows Event Log for daemon errors
  - Suggests specific fixes for detected issues
  - Handles missing components gracefully (doesn't crash)
  - Outputs structured report with severity levels

### 4.3 Clean Reinstall Automation
- **Requirement:** Provide script for complete clean reinstall
- **Rationale:** Automate recovery from corrupted installations
- **Acceptance Criteria:**
  - Requires admin rights (checks and prompts if missing)
  - Prompts user for confirmation before destructive operations
  - Stops daemon forcefully (kill process if needed)
  - Uninstalls MSI cleanly (msiexec /x)
  - Removes all state files (~/.keyrx/*)
  - Cleans build artifacts (target/release/*)
  - Rebuilds UI (npm run build)
  - Rebuilds daemon (cargo build --release)
  - Builds fresh installer (build_windows_installer.ps1)
  - Installs new MSI
  - Verifies installation with health checks
  - Provides progress feedback throughout
  - Rolls back on critical failures (where possible)

## 5. Integration & Testing

### 5.1 Version Management Testing
- **Requirement:** Comprehensive tests for version management system
- **Rationale:** Ensure SSOT enforcement works reliably
- **Acceptance Criteria:**
  - Tests sync-version.sh script (success and failure cases)
  - Tests build.rs validation (mismatch detection)
  - Tests version constants at runtime
  - Tests API version endpoints
  - Achieves 90%+ code coverage
  - Tests run reliably on CI
  - Tests handle both Windows and Linux environments

### 5.2 Installer Validation Testing
- **Requirement:** Test suite for installer enhancements
- **Rationale:** Prevent installer regressions
- **Acceptance Criteria:**
  - Tests pre-flight version validation
  - Tests daemon stop logic (retry/timeout)
  - Tests admin rights detection
  - Tests post-install verification
  - Uses mocks to avoid requiring actual installation
  - Tests both success and failure scenarios
  - Runs on CI without admin rights
  - No destructive operations during testing

### 5.3 Diagnostic Scripts Testing
- **Requirement:** Test suite for diagnostic scripts
- **Rationale:** Ensure diagnostic scripts work reliably
- **Acceptance Criteria:**
  - Uses Pester framework for PowerShell testing
  - Tests installer-health-check.ps1 output accuracy
  - Tests diagnose-installation.ps1 detection accuracy
  - Tests force-clean-reinstall.ps1 in dry-run mode
  - Verifies error handling for all scripts
  - Uses mocks for destructive operations
  - Tests output format validation
  - Achieves 80%+ script coverage

### 5.4 CI/CD Integration
- **Requirement:** CI pipeline SHALL validate version consistency
- **Rationale:** Prevent merging code with version mismatches
- **Acceptance Criteria:**
  - Runs sync-version.sh --check on all PRs and branches
  - Fails CI build on version mismatch (fast fail)
  - Provides clear error message in CI logs
  - Runs as early step in CI pipeline
  - Doesn't break existing CI jobs
  - Works on GitHub Actions (existing platform)

## Non-Functional Requirements

### Performance
- Build-time validation adds < 1 second to build time
- API diagnostics endpoint responds in < 100ms
- Diagnostic scripts complete in < 30 seconds
- Version sync script completes in < 5 seconds

### Reliability
- All scripts handle errors gracefully (no crashes)
- Scripts work on clean install and dirty environments
- No silent failures (all errors logged and reported)
- Idempotent operations (safe to run multiple times)

### Usability
- Error messages are actionable (tell user what to do)
- Progress feedback for long-running operations
- Color coding for visual clarity (errors=red, warnings=yellow, success=green)
- Consistent output format (tables, JSON)

### Maintainability
- Code follows project guidelines (500 lines/file, 50 lines/function)
- Comprehensive inline documentation
- Clear separation of concerns
- DRY principle (no code duplication)
- Test coverage ≥ 80% (90% for critical paths)

### Security
- No secrets or PII in logs or diagnostics
- Admin rights required only where necessary
- Installer operations use least privilege principle
- API diagnostics don't expose internal paths

## Out of Scope

- Mac/Linux installer enhancements (Windows only for now)
- GUI for diagnostic tools (command-line only)
- Automatic version bumping (manual version updates remain)
- Rollback mechanisms for failed updates (future work)
- Remote update checking (future work)
