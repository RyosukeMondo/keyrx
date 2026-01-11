# Tasks: API Contract Testing

## Implementation Tasks

- [x] 1. Create contract validation script
  - File: keyrx_ui/scripts/validate-api-contracts.ts
  - Implement CLI script to validate API responses against Zod schemas
  - Support --base-url argument for configurable daemon URL
  - Exit codes: 0 (success), 1 (validation failure), 2 (connection error)
  - Purpose: Enable local contract validation before commits
  - _Leverage: keyrx_ui/src/api/schemas.ts_
  - _Requirements: 1.1, 1.2, 1.3, 1.4_
  - _Prompt: Implement the task for spec api-contract-testing, first run spec-workflow-guide to get the workflow guide then implement the task: Role: TypeScript Developer specializing in CLI tools and API testing | Task: Create a CLI script that validates API responses against Zod schemas from schemas.ts, supporting configurable base URL and clear error reporting | Restrictions: Do not modify existing schemas, use native fetch, maintain compatibility with npx tsx execution | Success: Script validates all endpoints, reports clear errors, exits with correct codes_

- [x] 2. Add npm script for contract validation
  - File: keyrx_ui/package.json
  - Add "validate:contracts" script that runs the validation script
  - Purpose: Make contract validation easily runnable
  - _Leverage: keyrx_ui/scripts/validate-api-contracts.ts_
  - _Requirements: 1.1_
  - _Prompt: Implement the task for spec api-contract-testing, first run spec-workflow-guide to get the workflow guide then implement the task: Role: DevOps Engineer | Task: Add npm script entry for validate:contracts | Restrictions: Follow existing script naming conventions | Success: npm run validate:contracts works correctly_

- [x] 3. Create CI workflow job for contract validation
  - File: .github/workflows/ci.yml
  - Add new job "api-contract-tests" that depends on build
  - Build daemon, start it, run contract validation
  - Fail CI if contracts mismatch
  - Purpose: Automatically catch contract mismatches on PRs
  - _Leverage: Existing CI workflow structure_
  - _Requirements: 2.1, 2.2, 2.3_
  - _Prompt: Implement the task for spec api-contract-testing, first run spec-workflow-guide to get the workflow guide then implement the task: Role: DevOps/CI Engineer with GitHub Actions expertise | Task: Add a new job to ci.yml that builds the daemon, creates a test profile, starts the daemon, waits for it to be ready, and runs npm run validate:contracts. Job should depend on existing build job. | Restrictions: Do not modify existing jobs, use Ubuntu runner, ensure daemon runs without keyboard permissions, cleanup daemon on failure | Success: CI job runs on PRs, fails if contracts mismatch, produces clear output, daemon starts and stops cleanly. After implementing, set this task to in-progress [-] in tasks.md, use log-implementation tool to record details, then mark as complete [x]_

- [x] 4. Add contract validation result artifact
  - File: .github/workflows/ci.yml (modify task 3)
  - Upload contract validation output as artifact
  - Include timestamp and commit SHA
  - Purpose: Enable debugging of CI failures
  - _Leverage: GitHub Actions artifacts_
  - _Requirements: 2.4_
  - _Prompt: Implement the task for spec api-contract-testing, first run spec-workflow-guide to get the workflow guide then implement the task: Role: DevOps Engineer | Task: Modify the CI job to capture validation output and upload as artifact | Restrictions: Use standard GitHub Actions artifact upload, keep artifact retention reasonable | Success: Artifacts are uploaded on both success and failure, include useful debugging info. After implementing, set this task to in-progress [-] in tasks.md, use log-implementation tool to record details, then mark as complete [x]_

- [x] 5. Create contract testing documentation
  - File: keyrx_ui/docs/contract-testing.md
  - Document how to run contract tests locally
  - Document how to add new endpoint validation
  - Include troubleshooting section
  - Purpose: Enable developers to maintain contract tests
  - _Leverage: keyrx_ui/scripts/validate-api-contracts.ts_
  - _Requirements: 3.1, 3.2, 3.3_
  - _Prompt: Implement the task for spec api-contract-testing, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Technical Writer with API documentation experience | Task: Create comprehensive documentation for the contract testing system including local usage, adding new endpoints, and troubleshooting | Restrictions: Follow existing documentation style, include code examples, keep concise | Success: Documentation covers all use cases, examples work correctly, troubleshooting helps resolve common issues. After implementing, set this task to in-progress [-] in tasks.md, use log-implementation tool to record details, then mark as complete [x]_

- [x] 6. Add validation for additional endpoints
  - File: keyrx_ui/scripts/validate-api-contracts.ts
  - Add validation for: GET /api/status, PATCH /api/devices/:id
  - Ensure all documented endpoints are covered
  - Purpose: Complete endpoint coverage
  - _Leverage: keyrx_ui/src/api/schemas.ts_
  - _Requirements: 3.1_
  - _Prompt: Implement the task for spec api-contract-testing, first run spec-workflow-guide to get the workflow guide then implement the task: Role: TypeScript Developer | Task: Extend the validation script to cover additional endpoints, adding appropriate schemas if needed | Restrictions: Follow existing validation patterns, add schemas to schemas.ts if missing | Success: All documented REST endpoints are validated. After implementing, set this task to in-progress [-] in tasks.md, use log-implementation tool to record details, then mark as complete [x]_

## Completed Tasks Summary

- Task 1: Created `keyrx_ui/scripts/validate-api-contracts.ts` - CLI validation script
- Task 2: Added `validate:contracts` npm script to package.json

## Notes

- Tasks 1-2 were completed as part of the initial investigation
- Tasks 3-6 remain for full CI integration
- Daemon must be built with web feature for API endpoints
- CI job runs on Ubuntu where keyboard capture will fail (expected - web API still works)
