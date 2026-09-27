# Architecture Completion - Tasks

**E1 verdict summary (2026-09-27):** 21 DONE-ELSEWHERE, 5 STILL-NEEDED, 4 OBSOLETE (30 tasks total). Verified against current code (grep/read, no full builds) by 4 parallel research agents cross-checked against .spec-workflow/RESUME_PLAN.md Phase E1. Genuinely-needed items are promoted to concrete tasks in RESUME_PLAN.md Phase F.

## Phase 1: Critical Blockers (6 hours)

### Task 1.1: Fix compilation errors
- [ ] 1.1.1 Add #[derive(Debug)] to 9 CLI Args structs — **[DONE-ELSEWHERE]** all 11 clap Args structs already have #[derive(Args, Debug)] (cli/{devices,test,layouts,layers,state,status,simulate,metrics}.rs, cli/config/mod.rs:18, cli/profiles/args.rs:8).
  - Find all Args structs missing Debug
  - Add derive attribute
  - Verify compilation

- [ ] 1.1.2 Fix DaemonError::Init pattern matching — **[DONE-ELSEWHERE]** DaemonError::Init (error/mod.rs:48) is handled in all 3 downstream matches: cli/error/mod.rs:218, cli/error/codes.rs:82 (code 9001), web/error.rs:71 (INIT_ERROR/500).
  - Add DaemonError::Init(e) arms to 3 match expressions
  - Handle initialization errors properly
  - Test error paths

### Task 1.2: Retry TypeShare implementation
- [ ] 1.2.1 Add typeshare dependency — **[DONE-ELSEWHERE]** typeshare="1.0" in Cargo.toml:40 + [package.metadata.typeshare]; #[typeshare] on DeviceEntry/ProfileMetadata and RPC/event types, all present in generated.ts.
  - Add to Cargo.toml with version ^1.0
  - Annotate all RPC types with #[typeshare]
  - Generate TypeScript types

- [ ] 1.2.2 Replace frontend duplicates — **[DONE-ELSEWHERE]** keyrx_ui/src/types/index.ts re-exports generated RPC types from ./generated; its own DeviceEntry/ProfileMetadata are deliberate, documented post-mapping frontend view-models, not stale duplicates of the wire type.
  - Remove duplicate DeviceEntry, ProfileMetadata
  - Import from generated.ts
  - Verify TypeScript compilation without test execution

### Task 1.3: Complete main.rs refactoring
- [ ] 1.3.1 Extract Linux platform runner — **[DONE-ELSEWHERE]** keyrx_daemon/src/daemon/platform_runners/linux.rs exists (427 lines), wired via cli/dispatcher.rs.
  - Create platform_runners/linux.rs (~350 lines)
  - Move handle_run() Linux implementation
  - Wire through dispatcher

- [ ] 1.3.2 Extract Windows platform runner — **[DONE-ELSEWHERE]** keyrx_daemon/src/daemon/platform_runners/windows.rs exists (738 lines — over target/gate, but tracked separately as RESUME_PLAN G3/G5, not reopened here).
  - Create platform_runners/windows.rs (~425 lines)
  - Move handle_run() Windows implementation
  - Wire through dispatcher

- [ ] 1.3.3 Finalize main.rs — **[DONE-ELSEWHERE]** keyrx_daemon/src/main.rs is 172 lines: only Cli/Commands arg structs + dispatcher::dispatch(command) call.
  - Keep only: arg parsing, dispatcher call
  - Target: <200 lines total
  - Verify all platforms compile

### Task 1.4: Retry complexity reduction
- [ ] 1.4.1 Extract event loop helpers — **[DONE-ELSEWHERE]** daemon/event_loop.rs has 13 extracted helpers (format_output_description, handle_timeout_events, process_input_event, process_remapping, inject_output_events, log_reload_error, ...).
  - Without running tests
  - Just refactor code
  - Defer verification

- [ ] 1.4.2 Split complex functions — **[DONE-ELSEWHERE]** run_event_loop is a ~95-line orchestrator over those helpers; execute_inner is now a thin function in cli/config/mod.rs and cli/devices.rs — duplicates architecture-remediation 4.4.1/4.4.2, same verdict applies.
  - run_event_loop: 18 → <10
  - execute_inner: 22 → <10
  - Use helper extraction pattern

### Task 1.5: Retry test file splitting
- [ ] 1.5.1 Split e2e_harness.rs — **[STILL-NEEDED]** tests/e2e_harness.rs (3386 lines) is the live monolith (`mod e2e_harness;` in 4 test files); a parallel split tree at tests/harness/{mod,harness,error,config,assertions}.rs has equivalent content but is referenced nowhere (orphaned since commit 90834f9d) — finish wiring the split or delete the dead tree.
  - Without running npm test
  - Create module structure
  - Split into focused files

- [ ] 1.5.2 Split virtual_e2e_tests.rs — **[STILL-NEEDED]** same pattern: tests/virtual_e2e_test.rs (1904 lines) is live; tests/virtual/{mod,basic,complex,layers,passthrough,advanced_output,advanced_sequences}.rs is an orphaned, unreferenced split tree.
  - Create tests/virtual/* modules
  - Organize by scenario
  - Keep < 500 lines each

## Phase 2: Security Critical (2-3 days)

### Task 2.1: Implement authentication
- [ ] 2.1.1 Design auth system — **[DONE-ELSEWHERE]** keyrx_daemon/src/auth/mod.rs: JWT (15min access/7d refresh) + Argon2id hashing, AuthMode::{DevMode,Jwt,Password} from_env().
  - JWT or session-based
  - Token storage strategy
  - Middleware architecture

- [ ] 2.1.2 Backend authentication — **[DONE-ELSEWHERE]** keyrx_daemon/src/auth/handlers.rs (login/logout/refresh/validate) + web/middleware/auth.rs:52 auth_middleware guards routes, 401 on invalid/expired token.
  - Auth middleware for Axum
  - Login/logout endpoints
  - Token validation
  - Protected route guards

- [ ] 2.1.3 Frontend authentication — **[DONE-ELSEWHERE]** keyrx_ui/src/contexts/AuthContext.tsx + components/LoginForm.tsx exist and are wired into Header/Page components.
  - Auth context provider
  - Token storage (httpOnly cookies)
  - Auto-redirect on 401
  - Login UI component

### Task 2.2: Fix CORS configuration
- [ ] 2.2.1 Proper origin validation — **[DONE-ELSEWHERE]** web/mod.rs build_cors_layer(): CorsLayer::new().allow_origin(AllowOrigin::list(...)) from config.cors_origins() (KEYRX_ALLOWED_ORIGINS env) — real whitelist, not Any.
  - Remove allow-all CORS
  - Whitelist specific origins
  - Use daemon_config.rs

- [ ] 2.2.2 Security headers — **[DONE-ELSEWHERE]** web/middleware/security_headers.rs implements Content-Security-Policy, X-Content-Type-Options, X-Frame-Options.
  - Content-Security-Policy
  - X-Frame-Options
  - X-Content-Type-Options

### Task 2.3: Security hardening
- [ ] 2.3.1 Add rate limiting — **[DONE-ELSEWHERE]** web/middleware/rate_limit.rs (387 lines, per-IP + per-endpoint) plus auth/rate_limit.rs (199 lines, LoginRateLimiter: 5 attempts/min/IP) — two complementary limiters.
  - Per-IP rate limits
  - API endpoint throttling
  - DDoS protection

- [ ] 2.3.2 Input sanitization audit — **[DONE-ELSEWHERE]** keyrx_daemon/src/validation/ module + web/middleware/input_validation.rs + web/api/validation.rs (MAX_BODY_SIZE=1MB, MAX_CONFIG_LENGTH=512KB, profile_name::MAX_NAME_LENGTH enforced).
  - Verify all inputs validated
  - Check for injection vulnerabilities
  - Add input length limits

## Phase 3: Final Quality (1 week)

### Task 3.1: SSOT completion
- [ ] 3.1.1 Verify TypeShare integration — **[DONE-ELSEWHERE]** typeshare wired (10 #[typeshare] files), generated.ts real; index.ts's separate DeviceEntry/ProfileMetadata are deliberate view-models (see 1.2.2), not the literal duplication this task worried about.
  - Zero duplicate types
  - All RPC types generated
  - Frontend uses generated types

- [ ] 3.1.2 Configuration verification — **[DONE-ELSEWHERE]** keyrx_daemon/src/daemon_config.rs::from_env() reads KEYRX_BIND_HOST/PORT/LOG_LEVEL/DEBUG/TEST_MODE/ALLOWED_ORIGINS with defaults + validation.
  - Zero hardcoded values
  - All config from environment
  - Validation on startup

### Task 3.2: File size final audit
- [ ] 3.2.1 Run file size check — **[DONE-ELSEWHERE]** scripts/verify/file-sizes.sh is a real working gate; current violations are only 3 files: profile_manager.rs (712), platform_runners/windows.rs (537, delegated to a Windows session), simulation_engine.rs (529).
  - scripts/verify_file_sizes.sh
  - Identify any remaining violations
  - Split if needed

### Task 3.3: Run all quality audits
- [ ] 3.3.1 Re-run SOLID audit — **[OBSOLETE]** "re-run SOLID audit, target A+95%" references the one-off grading rubric from comprehensive-architecture-refactoring/audit-report.md; no such audit tool exists today to re-run.
  - Target: A+ (95%+)
  - Verify ServiceContainer usage
  - Check dependency injection

- [ ] 3.3.2 Re-run KISS/SLAP audit — **[OBSOLETE]** same rubric problem; the real, current file-size/complexity gate is scripts/verify/file-sizes.sh (task 3.2.1), not a scored audit.
  - Target: 9/10
  - Verify file sizes
  - Check complexity metrics

- [ ] 3.3.3 Re-run Security audit — **[OBSOLETE]** same rubric problem; auth/CORS/rate-limit/input-validation are all implemented (2.1-2.3) but there's no scored "security audit" tool to re-run.
  - Target: A (95%+)
  - Verify auth implementation
  - Check for vulnerabilities

- [ ] 3.3.4 Re-run SSOT audit — **[OBSOLETE]** scripts/verify/ssot.sh is a literal `exit 0` stub not even called by verify.sh's CHECK_ORDER; no live SSOT audit exists to re-run.
  - Target: 0 violations
  - Verify TypeShare
  - Check configuration

### Task 3.4: Performance optimization
- [ ] 3.4.1 Profile hot paths — **[DONE-ELSEWHERE]** keyrx_daemon/benches/{input_latency,ipc_latency,profile_activation,device_registry}.rs profile the relevant hot paths.
  - Event loop performance
  - Lookup performance
  - Identify bottlenecks

- [ ] 3.4.2 Optimize if needed — **[DONE-ELSEWHERE]** RESUME_PLAN.md Phase G/G4: /api/devices per-node open+close (RCU-wait ~15ms×32) removed → 440ms to <1ms; no other flagged bottleneck remains undocumented.
  - Cache frequently accessed data
  - Reduce allocations
  - Parallelize where safe

### Task 3.5: Final testing
- [ ] 3.5.1 All backend tests — **[STILL-NEEDED]** .claude/CLAUDE.md Quality Gates table: Linux run 2026-09-27 = 2020 pass/4 fail/91 ignored — literal "100% pass" criterion unmet (4 known failures catalogued under RESUME_PLAN G4, not yet all fixed/reverified).
  - cargo test --workspace
  - Target: 100% pass rate

- [ ] 3.5.2 All frontend tests — **[STILL-NEEDED]** RESUME_PLAN.md Phase G/G2 note: "UI suite 1916 pass / 2 fail (pre-existing DevicesPage, same at HEAD)" — 100%-pass target unmet by 2 known frontend failures.
  - npm test
  - Target: 100% pass rate

- [ ] 3.5.3 E2E tests — **[STILL-NEEDED]** Playwright E2E infra exists (playwright.config.ts, npm scripts) but no record in RESUME_PLAN.md/.claude/CLAUDE.md of a full E2E run/pass — the task's actual bar has no recent evidence of execution.
  - Run full E2E suite
  - Verify all scenarios

## Summary

**Total Tasks:** 40+ subtasks
**Estimated Time:** 2-3 weeks
**Target Grade:** A+ (95%) across all categories
