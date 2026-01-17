# Tasks Document - Fix WASM ConfigPage Error & Prevent Future Issues

## Investigation & Root Cause Analysis

- [ ] 1. Investigate current WASM error in ConfigPage
  - File: Investigation report to be created
  - Search web for React Error #185 solutions and WASM context issues
  - Analyze browser console errors in development mode
  - Check network tab for correct bundle loading
  - Purpose: Identify exact root cause of ConfigPage WASM error
  - _Leverage: Browser DevTools, Web search results_
  - _Requirements: 1.1 - Identify root cause_
  - _Prompt: Role: Senior Frontend Debugger with expertise in React errors and WASM integration | Task: Investigate the ConfigPage WASM error by running the dev server (npm run dev), checking browser console for unminified errors, verifying WasmProvider is correctly wrapping the component tree, and searching web for React Error #185 solutions | Restrictions: Must use development build (not production minified), must check actual network requests to verify correct bundle versions are loaded, must verify browser cache is cleared | Success: Exact error message identified from dev build, root cause documented with stack trace, web search findings summarized, clear action plan created_

- [ ] 2. Verify WasmProvider integration in component tree
  - File: keyrx_ui/src/App.tsx
  - Verify WasmProvider wraps all routes correctly
  - Check that WasmProvider is before LayoutPreviewProvider
  - Verify MonacoEditor has access to WasmContext
  - Purpose: Ensure WASM context is available to all components that need it
  - _Leverage: keyrx_ui/src/contexts/WasmContext.tsx, keyrx_ui/src/components/MonacoEditor.tsx_
  - _Requirements: 1.2 - Verify context provider hierarchy_
  - _Prompt: Role: React Architecture Specialist with expertise in Context API and provider patterns | Task: Verify WasmProvider is correctly positioned in App.tsx component tree, ensuring it wraps all components that use useWasmContext hook (MonacoEditor, CodePanelContainer, etc.), and validate the provider hierarchy order | Restrictions: Must not break existing LayoutPreviewProvider, must maintain current routing structure, must verify all WASM-dependent components are within provider scope | Success: WasmProvider verified in correct position, all WASM-dependent components identified and confirmed within provider scope, provider hierarchy diagram created_

- [ ] 3. Check for infinite re-render loops in ConfigPage
  - File: keyrx_ui/src/pages/ConfigPage.tsx
  - Audit all useEffect hooks for dependency array issues
  - Verify no functions are in dependency arrays (syncEngine.getCode(), etc.)
  - Check for setState calls that trigger their own useEffect
  - Purpose: Eliminate infinite re-render loops causing React Error #185
  - _Leverage: React DevTools Profiler, ESLint react-hooks/exhaustive-deps_
  - _Requirements: 1.3 - Fix infinite re-render loops_
  - _Prompt: Role: React Performance Engineer with expertise in hooks and re-render optimization | Task: Audit all useEffect hooks in ConfigPage.tsx for infinite loop patterns, specifically checking if functions like syncEngine.getCode() are in dependency arrays, if setState calls trigger their own effects, and if computed values cause cascading updates | Restrictions: Must not remove necessary dependencies, must not disable ESLint warnings without fixing root cause, must maintain functional correctness while fixing loops | Success: All infinite loop patterns identified and documented, dependency arrays corrected to use stable values, React DevTools shows no excessive re-renders, all ESLint warnings addressed_

## Development Environment Fixes

- [ ] 4. Set up development server workflow for debugging
  - Files: scripts/windows/Dev-Server.ps1 (already created), package.json
  - Ensure npm run dev starts Vite dev server with unminified code
  - Configure source maps for proper error stack traces
  - Add WASM development mode with verbose errors
  - Purpose: Enable proper debugging with readable error messages
  - _Leverage: Vite configuration, source map settings_
  - _Requirements: 2.1 - Enable development debugging_
  - _Prompt: Role: DevOps Engineer with expertise in Vite build configuration and debugging workflows | Task: Configure development environment to always provide unminified errors and proper source maps, ensuring Vite dev server (port 5173) serves unminified code while daemon (port 9871) handles API, and WASM modules load with detailed error messages | Restrictions: Must not affect production builds, must maintain hot module replacement (HMR) functionality, must not slow down development server significantly | Success: Dev server provides readable error messages with line numbers, source maps correctly map to TypeScript source, WASM errors show detailed stack traces, HMR works correctly_

- [ ] 5. Add development vs production build verification
  - Files: scripts/windows/UAT.ps1, scripts/windows/Verify-Build.ps1 (new)
  - Create script to verify production bundles match source files
  - Add hash verification for WASM modules
  - Check that all context providers are in production build
  - Purpose: Prevent production builds from missing critical code
  - _Leverage: keyrx_ui/dist/ build output, source file checksums_
  - _Requirements: 2.2 - Verify production builds_
  - _Prompt: Role: Build Systems Engineer with expertise in bundle analysis and verification | Task: Create verification script that checks production build integrity by verifying WasmProvider exists in bundle, WASM modules have correct hashes, bundle sizes are reasonable, and critical code paths are not tree-shaken away | Restrictions: Must catch missing providers before deployment, must verify bundle integrity without slowing build process significantly, must be automatable in CI/CD | Success: Verification script detects missing providers, WASM hash mismatches caught, bundle analysis shows all critical code included, script integrated into build process_

- [ ] 6. Configure browser cache busting for development
  - Files: keyrx_daemon/src/web/static_files.rs, vite.config.ts
  - Add cache-control headers to prevent stale UI in daemon
  - Configure Vite to use contenthash in filenames
  - Add version query parameter to assets
  - Purpose: Eliminate browser cache issues during development
  - _Leverage: Vite build configuration, Axum static file serving_
  - _Requirements: 2.3 - Prevent browser cache issues_
  - _Prompt: Role: Web Performance Engineer with expertise in HTTP caching and CDN configuration | Task: Configure proper cache-control headers in daemon's static file serving to prevent stale UI, ensure Vite generates contenthash-based filenames for cache busting, and add version query parameters to force cache invalidation on updates | Restrictions: Must not break production caching strategy, must maintain fast load times, must work with daemon's embedded static file serving | Success: Browser always loads latest UI without manual cache clear, contenthash filenames prevent stale assets, cache headers properly set for development and production_

## Preventive Measures

- [ ] 7. Add TypeScript strict mode for context usage
  - Files: tsconfig.json, keyrx_ui/src/contexts/*.tsx
  - Enable strict null checks for context values
  - Add ESLint rule to detect context usage outside providers
  - Create type guards for context values
  - Purpose: Catch context usage errors at compile time
  - _Leverage: TypeScript compiler, ESLint plugins_
  - _Requirements: 3.1 - Compile-time context validation_
  - _Prompt: Role: TypeScript Expert with expertise in strict typing and compiler configurations | Task: Enable TypeScript strict mode to catch context usage errors at compile time, add ESLint rules to detect useContext calls outside their providers, and create type guards that force null checks before using context values | Restrictions: Must not break existing code, must provide clear error messages for violations, must be enforceable in CI/CD | Success: Context usage outside providers fails compilation, ESLint catches improper context usage, type guards prevent null reference errors, all existing code compiles with strict mode_

- [ ] 8. Create automated tests for context provider hierarchy
  - Files: keyrx_ui/src/__tests__/App.context.test.tsx (new)
  - Write tests verifying WasmProvider wraps ConfigPage
  - Test that useWasmContext works in ConfigPage
  - Add smoke tests for all WASM-dependent components
  - Purpose: Prevent regression of provider hierarchy
  - _Leverage: React Testing Library, jest, testing utils_
  - _Requirements: 3.2 - Test provider hierarchy_
  - _Prompt: Role: QA Engineer with expertise in React Testing Library and integration testing | Task: Create comprehensive tests that verify WasmProvider correctly wraps all components using useWasmContext, test that context values are accessible in ConfigPage, and add smoke tests for MonacoEditor and CodePanelContainer to ensure WASM functionality works | Restrictions: Must test actual provider hierarchy, must fail if provider is removed, must not mock context in these tests | Success: Tests fail when WasmProvider is removed from App.tsx, tests verify context accessibility in all WASM-dependent components, tests run in CI/CD pipeline_

- [ ] 9. Add pre-commit hook to verify WASM build integrity
  - Files: .husky/pre-commit, scripts/verify-wasm-integrity.sh
  - Check WASM module exists before allowing commit
  - Verify WASM hash matches expected value
  - Ensure WasmProvider is in App.tsx
  - Purpose: Prevent commits that break WASM functionality
  - _Leverage: Git hooks, SHA256 verification, grep for WasmProvider_
  - _Requirements: 3.3 - Pre-commit WASM verification_
  - _Prompt: Role: DevOps Engineer with expertise in Git hooks and automated verification | Task: Create pre-commit hook that verifies WASM module exists in src/wasm/pkg/, checks WASM hash matches manifest, greps for WasmProvider in App.tsx, and fails commit if any check fails with clear error message | Restrictions: Must be fast enough for developer workflow, must provide actionable error messages, must work on both Windows and Linux | Success: Hook blocks commits with missing WASM, hash mismatches detected, missing WasmProvider caught, developers get clear fix instructions_

- [ ] 10. Document WASM troubleshooting guide
  - Files: docs/WASM_TROUBLESHOOTING.md (new), .claude/CLAUDE.md
  - Create step-by-step debugging guide for WASM errors
  - Document common error patterns and solutions
  - Add flowchart for diagnosing WASM issues
  - Purpose: Enable quick resolution of future WASM issues
  - _Leverage: This investigation's findings, web search results_
  - _Requirements: 3.4 - Create troubleshooting documentation_
  - _Prompt: Role: Technical Writer with expertise in developer documentation and troubleshooting guides | Task: Create comprehensive WASM troubleshooting guide documenting React Error #185, useWasmContext errors, infinite re-render loops, cache issues, and WASM build problems with step-by-step solutions and a diagnostic flowchart | Restrictions: Must be actionable with specific commands, must include both Windows and Linux solutions, must reference actual file paths in repo | Success: Guide enables developers to diagnose WASM issues independently, flowchart leads to correct solution, all known error patterns documented, added to .claude/CLAUDE.md for AI agent reference_

## Production Hardening

- [ ] 11. Add WASM loading error boundary with fallback UI
  - Files: keyrx_ui/src/components/WasmErrorBoundary.tsx (new)
  - Create error boundary specifically for WASM failures
  - Provide user-friendly error message with recovery steps
  - Add automatic retry mechanism for WASM loading
  - Purpose: Graceful degradation when WASM fails to load
  - _Leverage: React Error Boundary, retry logic_
  - _Requirements: 4.1 - Graceful WASM failure handling_
  - _Prompt: Role: Frontend Reliability Engineer with expertise in error boundaries and fallback UIs | Task: Create WasmErrorBoundary component that catches WASM loading failures, displays user-friendly error message explaining the issue, provides retry button, and optionally falls back to non-WASM mode for basic functionality | Restrictions: Must not crash entire app on WASM failure, must provide actionable user guidance, must log errors for debugging | Success: WASM failures show friendly error instead of blank page, users can retry loading, error details logged for debugging, basic functionality available without WASM_

- [ ] 12. Add production error reporting for minified errors
  - Files: keyrx_ui/src/utils/errorReporting.ts (new), App.tsx
  - Integrate error reporting service (Sentry or similar)
  - Map minified errors to source code using source maps
  - Add context information (browser, build version, user actions)
  - Purpose: Debug production issues with minified code
  - _Leverage: Source maps, error reporting SDKs_
  - _Requirements: 4.2 - Production error tracking_
  - _Prompt: Role: Production Support Engineer with expertise in error monitoring and observability | Task: Integrate error reporting service that uploads source maps, captures production errors with full stack traces, adds browser context and build version info, and enables debugging of minified React errors in production | Restrictions: Must not impact performance significantly, must protect user privacy (no PII), must work with embedded daemon UI | Success: Production errors captured with full stack traces, source maps correctly map to TypeScript, errors include browser and build context, team notified of critical errors_

- [ ] 13. Create automated UI health check after build
  - Files: scripts/windows/Health-Check.ps1 (new), .github/workflows/ci.yml
  - Launch daemon with production build
  - Check ConfigPage loads without errors
  - Verify WASM module initializes successfully
  - Purpose: Catch deployment issues before release
  - _Leverage: Headless browser testing, daemon API_
  - _Requirements: 4.3 - Automated health checks_
  - _Prompt: Role: QA Automation Engineer with expertise in headless browser testing and CI/CD | Task: Create automated health check script that starts daemon with production build, uses headless browser (Playwright) to navigate to ConfigPage, checks for React errors, verifies WASM loads, and fails CI if any issues detected | Restrictions: Must run in CI environment, must complete quickly (< 2 minutes), must provide clear failure messages | Success: Health check catches broken builds before merge, ConfigPage verified to load without errors, WASM initialization validated, integrated into CI/CD pipeline_

## Implementation and Validation

- [ ] 14. Fix identified issues in ConfigPage
  - Files: keyrx_ui/src/pages/ConfigPage.tsx, keyrx_ui/src/App.tsx
  - Apply fixes from investigation (tasks 1-3)
  - Remove function calls from useEffect dependency arrays
  - Ensure WasmProvider is correctly positioned
  - Purpose: Resolve current production issue
  - _Leverage: Investigation findings from tasks 1-3_
  - _Requirements: 5.1 - Apply fixes_
  - _Prompt: Role: Senior React Developer with expertise in hooks and performance optimization | Task: Apply all fixes identified in investigation including correcting useEffect dependency arrays (remove syncEngine.getCode()), verifying WasmProvider position in App.tsx, and fixing any infinite re-render loops, while maintaining functional correctness | Restrictions: Must maintain all existing functionality, must not introduce new bugs, must follow React best practices | Success: ConfigPage loads without errors, no infinite re-renders detected, WASM functionality works correctly, all existing features functional_

- [ ] 15. Verify fix with clean rebuild and UAT
  - Files: All build artifacts
  - Run Clean-All.ps1 to remove all artifacts
  - Rebuild with UAT.ps1
  - Test ConfigPage in both dev and production builds
  - Purpose: Validate fix works in clean environment
  - _Leverage: scripts/windows/Clean-All.ps1, scripts/windows/UAT.ps1_
  - _Requirements: 5.2 - Validate fix_
  - _Prompt: Role: QA Engineer with expertise in regression testing and UAT procedures | Task: Execute complete clean rebuild using Clean-All.ps1 followed by UAT.ps1, test ConfigPage functionality in both development server (port 5173) and production daemon (port 9871), verify WASM loads correctly, and confirm no React errors appear | Restrictions: Must test in fresh browser session (incognito), must verify network requests show correct bundle hashes, must test all ConfigPage features | Success: ConfigPage works in both dev and production, no React errors in console, WASM loads successfully, all features functional, network shows correct bundles_

- [ ] 16. Update project documentation and AI agent instructions
  - Files: .claude/CLAUDE.md, docs/WASM_TROUBLESHOOTING.md, README.md
  - Add WASM troubleshooting section to CLAUDE.md
  - Update UAT instructions with cache clearing steps
  - Document dev server workflow for debugging
  - Purpose: Prevent recurrence and enable quick resolution
  - _Leverage: Investigation findings, implemented solutions_
  - _Requirements: 5.3 - Update documentation_
  - _Prompt: Role: Technical Documentation Specialist with expertise in developer guides and AI agent instructions | Task: Update CLAUDE.md with WASM troubleshooting guide, add cache clearing instructions to UAT process, document dev server workflow for debugging minified errors, and add preventive measures to project documentation | Restrictions: Must be actionable and specific, must include Windows and Linux commands, must be concise for AI agent consumption | Success: CLAUDE.md includes WASM troubleshooting, UAT docs mention cache clearing, dev server workflow documented, future AI agents can prevent/fix this issue_
