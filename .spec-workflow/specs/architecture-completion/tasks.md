# Architecture Completion - Tasks

## Phase 1: Critical Blockers (6 hours)

### Task 1.1: Fix compilation errors
- [ ] 1.1.1 Add #[derive(Debug)] to 9 CLI Args structs
  - Find all Args structs missing Debug
  - Add derive attribute
  - Verify compilation

- [ ] 1.1.2 Fix DaemonError::Init pattern matching
  - Add DaemonError::Init(e) arms to 3 match expressions
  - Handle initialization errors properly
  - Test error paths

### Task 1.2: Retry TypeShare implementation
- [ ] 1.2.1 Add typeshare dependency
  - Add to Cargo.toml with version ^1.0
  - Annotate all RPC types with #[typeshare]
  - Generate TypeScript types

- [ ] 1.2.2 Replace frontend duplicates
  - Remove duplicate DeviceEntry, ProfileMetadata
  - Import from generated.ts
  - Verify TypeScript compilation without test execution

### Task 1.3: Complete main.rs refactoring
- [ ] 1.3.1 Extract Linux platform runner
  - Create platform_runners/linux.rs (~350 lines)
  - Move handle_run() Linux implementation
  - Wire through dispatcher

- [ ] 1.3.2 Extract Windows platform runner
  - Create platform_runners/windows.rs (~425 lines)
  - Move handle_run() Windows implementation
  - Wire through dispatcher

- [ ] 1.3.3 Finalize main.rs
  - Keep only: arg parsing, dispatcher call
  - Target: <200 lines total
  - Verify all platforms compile

### Task 1.4: Retry complexity reduction
- [ ] 1.4.1 Extract event loop helpers
  - Without running tests
  - Just refactor code
  - Defer verification

- [ ] 1.4.2 Split complex functions
  - run_event_loop: 18 → <10
  - execute_inner: 22 → <10
  - Use helper extraction pattern

### Task 1.5: Retry test file splitting
- [ ] 1.5.1 Split e2e_harness.rs
  - Without running npm test
  - Create module structure
  - Split into focused files

- [ ] 1.5.2 Split virtual_e2e_tests.rs
  - Create tests/virtual/* modules
  - Organize by scenario
  - Keep < 500 lines each

## Phase 2: Security Critical (2-3 days)

### Task 2.1: Implement authentication
- [ ] 2.1.1 Design auth system
  - JWT or session-based
  - Token storage strategy
  - Middleware architecture

- [ ] 2.1.2 Backend authentication
  - Auth middleware for Axum
  - Login/logout endpoints
  - Token validation
  - Protected route guards

- [ ] 2.1.3 Frontend authentication
  - Auth context provider
  - Token storage (httpOnly cookies)
  - Auto-redirect on 401
  - Login UI component

### Task 2.2: Fix CORS configuration
- [ ] 2.2.1 Proper origin validation
  - Remove allow-all CORS
  - Whitelist specific origins
  - Use daemon_config.rs

- [ ] 2.2.2 Security headers
  - Content-Security-Policy
  - X-Frame-Options
  - X-Content-Type-Options

### Task 2.3: Security hardening
- [ ] 2.3.1 Add rate limiting
  - Per-IP rate limits
  - API endpoint throttling
  - DDoS protection

- [ ] 2.3.2 Input sanitization audit
  - Verify all inputs validated
  - Check for injection vulnerabilities
  - Add input length limits

## Phase 3: Final Quality (1 week)

### Task 3.1: SSOT completion
- [ ] 3.1.1 Verify TypeShare integration
  - Zero duplicate types
  - All RPC types generated
  - Frontend uses generated types

- [ ] 3.1.2 Configuration verification
  - Zero hardcoded values
  - All config from environment
  - Validation on startup

### Task 3.2: File size final audit
- [ ] 3.2.1 Run file size check
  - scripts/verify_file_sizes.sh
  - Identify any remaining violations
  - Split if needed

### Task 3.3: Run all quality audits
- [ ] 3.3.1 Re-run SOLID audit
  - Target: A+ (95%+)
  - Verify ServiceContainer usage
  - Check dependency injection

- [ ] 3.3.2 Re-run KISS/SLAP audit
  - Target: 9/10
  - Verify file sizes
  - Check complexity metrics

- [ ] 3.3.3 Re-run Security audit
  - Target: A (95%+)
  - Verify auth implementation
  - Check for vulnerabilities

- [ ] 3.3.4 Re-run SSOT audit
  - Target: 0 violations
  - Verify TypeShare
  - Check configuration

### Task 3.4: Performance optimization
- [ ] 3.4.1 Profile hot paths
  - Event loop performance
  - Lookup performance
  - Identify bottlenecks

- [ ] 3.4.2 Optimize if needed
  - Cache frequently accessed data
  - Reduce allocations
  - Parallelize where safe

### Task 3.5: Final testing
- [ ] 3.5.1 All backend tests
  - cargo test --workspace
  - Target: 100% pass rate

- [ ] 3.5.2 All frontend tests
  - npm test
  - Target: 100% pass rate

- [ ] 3.5.3 E2E tests
  - Run full E2E suite
  - Verify all scenarios

## Summary

**Total Tasks:** 40+ subtasks
**Estimated Time:** 2-3 weeks
**Target Grade:** A+ (95%) across all categories
