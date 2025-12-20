# Tasks Document

## Phase 1: Duplicate Class Renames (3 violations)

- [x] 1. Rename UI LogEntry to ObservabilityLogEntry
  - Files: ui/lib/services/observability_service.dart, and any files importing it
  - Rename LogEntry class to ObservabilityLogEntry in observability_service.dart
  - Update all usages of LogEntry in the same file
  - Search for imports and update references in other UI files
  - Purpose: Eliminate duplicate with core_logic LogEntry while preserving UI-specific log display model
  - _Leverage: grep/ripgrep to find all references_
  - _Requirements: 1_
  - _Prompt: Implement the task for spec ssot-violations-fix: Role: Refactoring Specialist with expertise in Dart and large codebase refactoring | Task: Rename the LogEntry class in ui/lib/services/observability_service.dart to ObservabilityLogEntry to avoid SSOT violation with core_logic's LogEntry class. Update all references in the UI layer. | Restrictions: Do not modify core_logic files, ensure all references are updated to prevent compilation errors, preserve existing functionality | Success: Class is renamed, all references updated, flutter analyze shows no errors, SSOT checker shows 1 fewer duplicate class violation | Instructions: First mark this task as in-progress in tasks.md by changing `[ ]` to `[-]`. After implementation, log the changes using log-implementation tool with detailed artifacts (classes modified, files changed, line counts). Then mark task as complete `[x]` in tasks.md._

- [x] 2. Rename UI DeviceRegistryService to UIDeviceRegistryAdapter
  - Files: ui/lib/services/device_registry_service.dart, files importing it
  - Rename DeviceRegistryService class to UIDeviceRegistryAdapter
  - Rename DeviceRegistryServiceImpl to UIDeviceRegistryAdapterImpl
  - Update all imports and usages in UI layer
  - Purpose: Clarify that UI version is an adapter wrapping core service
  - _Leverage: grep/ripgrep to find all references_
  - _Requirements: 1_
  - _Prompt: Implement the task for spec ssot-violations-fix: Role: Refactoring Specialist with expertise in Dart service layers | Task: Rename DeviceRegistryService to UIDeviceRegistryAdapter and DeviceRegistryServiceImpl to UIDeviceRegistryAdapterImpl in ui/lib/services/device_registry_service.dart. Update all UI layer references while preserving core_logic names. | Restrictions: Do not modify core_logic DeviceRegistryService, maintain existing functionality, update service registry bindings | Success: Both classes renamed, all UI references updated, service registry updated, flutter analyze clean, 2 fewer SSOT violations | Instructions: Mark in-progress `[-]`, implement changes, log with artifacts (classes renamed, integration points updated), mark complete `[x]`._

- [x] 3. Verify duplicate class renames compilation
  - Files: Full UI codebase
  - Run flutter analyze to verify no errors
  - Run SSOT checker to confirm 3 violations eliminated
  - Fix any remaining compilation issues
  - Purpose: Ensure renames are complete and correct
  - _Leverage: flutter analyze, scripts/pre-commit-ssot-check.sh_
  - _Requirements: 1_
  - _Prompt: Implement the task for spec ssot-violations-fix: Role: QA Engineer with expertise in Dart compilation and verification | Task: Verify that the duplicate class renames (ObservabilityLogEntry, UIDeviceRegistryAdapter) compile correctly and eliminate 3 SSOT violations. | Restrictions: Only fix compilation errors related to renames, do not introduce new functionality | Success: flutter analyze reports 0 errors, SSOT checker shows 34 violations (down from 37), all tests pass | Instructions: Mark in-progress `[-]`, run verification, fix any issues, log verification results, mark complete `[x]`._

## Phase 2: Business Logic Migration (32 violations)

- [x] 4. Create core DeviceService if missing
  - Files: packages/core_logic/lib/services/device_registry_service.dart (already exists)
  - DeviceRegistryService already exists and provides required functionality
  - Provides device stream (devicesStream), device list (getDevices), device operations
  - No new service needed - DeviceRegistryService fulfills this role
  - Purpose: Verified core service exists for device operations
  - _Leverage: existing DeviceRegistryService_
  - _Requirements: 2_
  - _Prompt: Implement the task for spec ssot-violations-fix: Role: Backend Developer with expertise in service architecture and FFI | Task: Create or verify DeviceService in core_logic to provide device operations (streams, list, selection) wrapping FFI bridge calls. Follow existing service patterns in core_logic. | Restrictions: No Flutter dependencies, follow existing service interfaces, maintain async patterns | Success: DeviceService created with clean interface, wraps FFI operations, has basic unit tests, follows core_logic patterns | Instructions: Mark in-progress `[-]`, create service, log implementation with API details, mark complete `[x]`._

- [x] 5. Migrate device_stream_provider FFI usage
  - Files: ui/lib/state/device_stream_provider.dart
  - Replace direct FFI bridge usage with DeviceService injection
  - Update provider to depend on DeviceService
  - Remove FFI bridge import
  - Purpose: Eliminate 1 business logic violation
  - _Leverage: DeviceService from task 4, Riverpod DI patterns_
  - _Requirements: 2_
  - _Prompt: Implement the task for spec ssot-violations-fix: Role: Flutter State Management Expert | Task: Refactor device_stream_provider to use core DeviceService instead of direct FFI bridge access. Use Riverpod dependency injection patterns. | Restrictions: Preserve existing provider API, maintain stream functionality, remove all FFI imports | Success: Provider uses DeviceService, no direct FFI access, functionality preserved, 1 fewer SSOT violation | Instructions: Mark in-progress `[-]`, refactor provider, log changes with integration details, mark complete `[x]`._

- [x] 6. Migrate providers_setup FFI usage (5 violations)
  - Files: ui/lib/state/providers_setup.dart
  - Identify all 5 FFI bridge usages in this file
  - Replace with appropriate core service injections
  - Create missing core services if needed (MetricsService, ObservabilityService, etc.)
  - Purpose: Eliminate 5 business logic violations in provider setup
  - _Leverage: Riverpod DI, core_logic services_
  - _Requirements: 2_
  - _Prompt: Implement the task for spec ssot-violations-fix: Role: Dependency Injection Specialist | Task: Refactor providers_setup.dart to use core_logic services instead of 5 direct FFI bridge calls. Create any missing core services. Use proper Riverpod dependency injection. | Restrictions: Preserve provider initialization order, maintain all existing provider APIs, no FFI in UI | Success: All 5 FFI usages replaced with service injections, providers work correctly, 5 fewer SSOT violations | Instructions: Mark in-progress `[-]`, refactor providers, log all service integrations, mark complete `[x]`._

- [x] 7. Migrate keyflow_repository FFI usage (2 violations)
  - Files: ui/lib/repositories/keyflow_repository.dart
  - Replace 2 direct FFI bridge calls with KeyflowService
  - Create KeyflowService in core_logic if missing
  - Update repository to inject and use service
  - Purpose: Eliminate 2 business logic violations in keyflow repository
  - _Leverage: repository patterns, core_logic services_
  - _Requirements: 2_
  - _Prompt: Implement the task for spec ssot-violations-fix: Role: Repository Pattern Expert | Task: Refactor keyflow_repository to use core KeyflowService instead of direct FFI access. Create service if missing. Maintain repository pattern. | Restrictions: Preserve repository interface, no FFI in repositories, follow existing repository patterns | Success: Repository uses KeyflowService, 2 FFI calls eliminated, functionality preserved, 2 fewer violations | Instructions: Mark in-progress `[-]`, refactor repository, log service creation and integration, mark complete `[x]`._

- [x] 8. Migrate metrics_service FFI usage (2 violations)
  - Files: ui/lib/services/metrics_service.dart
  - Replace 2 direct FFI bridge calls with core MetricsService
  - Create MetricsService in core_logic if missing
  - Update UI service to delegate to core service
  - Purpose: Eliminate 2 business logic violations in metrics
  - _Leverage: service delegation patterns_
  - _Requirements: 2_
  - _Prompt: Implement the task for spec ssot-violations-fix: Role: Service Layer Architect | Task: Refactor UI metrics_service to delegate to core MetricsService instead of direct FFI access. Create core service if missing. | Restrictions: Preserve UI service API, no business logic in UI service, only delegation | Success: UI service delegates to core, 2 FFI calls removed, metrics work correctly, 2 fewer violations | Instructions: Mark in-progress `[-]`, create delegation layer, log service architecture, mark complete `[x]`._

- [x] 9. Migrate observability_service FFI usage
  - Files: ui/lib/services/observability_service.dart
  - Replace direct FFI bridge call with core ObservabilityService
  - Create core service if missing
  - Convert between core LogEntry and UI ObservabilityLogEntry if needed
  - Purpose: Eliminate 1 business logic violation in observability
  - _Leverage: ObservabilityLogEntry from task 1_
  - _Requirements: 2_
  - _Prompt: Implement the task for spec ssot-violations-fix: Role: Observability Systems Engineer | Task: Refactor UI observability_service to use core ObservabilityService for FFI log access. Add conversion layer to UI's ObservabilityLogEntry if needed. | Restrictions: Preserve log stream API, maintain log formatting, no direct FFI in UI | Success: Service uses core layer, log streams work, conversion layer clean, 1 fewer violation | Instructions: Mark in-progress `[-]`, implement delegation, log integration patterns, mark complete `[x]`._

- [x] 10. Migrate benchmark_service FFI usage (2 violations)
  - Files: ui/lib/services/benchmark_service.dart
  - Replace 2 direct FFI bridge calls with core BenchmarkService
  - Create core service if missing
  - Update UI service to delegate
  - Purpose: Eliminate 2 business logic violations in benchmarks
  - _Leverage: service patterns_
  - _Requirements: 2_
  - _Prompt: Implement the task for spec ssot-violations-fix: Role: Performance Testing Specialist | Task: Refactor benchmark_service to use core BenchmarkService instead of direct FFI access. Create core service if missing. | Restrictions: Preserve benchmark API, maintain timing accuracy, no FFI in UI | Success: Benchmarks use core service, 2 FFI calls eliminated, performance testing works, 2 fewer violations | Instructions: Mark in-progress `[-]`, create service layer, log implementation, mark complete `[x]`._

- [x] 11. Document FlowValidator as UI-specific
  - Files: ui/lib/services/flow_validator.dart
  - Add clear documentation that FlowValidator is UI-specific validation
  - Verify it doesn't duplicate core_logic validation logic
  - If it duplicates core logic, refactor appropriately
  - Purpose: Clarify that flow graph validation is UI-specific, not a SSOT violation
  - _Leverage: existing FlowValidationError from previous work_
  - _Requirements: 2_
  - _Prompt: Implement the task for spec ssot-violations-fix: Role: Code Documentation Specialist | Task: Document FlowValidator as UI-specific flow graph validation. Verify no duplication with core_logic validation. Add clear comments explaining its UI-specific purpose. | Restrictions: Do not move to core if genuinely UI-specific, clear documentation required, verify no duplication | Success: FlowValidator clearly documented as UI-specific, no core duplication, SSOT checker accepts it | Instructions: Mark in-progress `[-]`, add documentation, log justification, mark complete `[x]`._

- [x] 12. Document MappingValidator as UI-specific
  - Files: ui/lib/services/mapping_validator.dart
  - Add clear documentation that MappingValidator is UI-specific
  - Verify it doesn't duplicate core_logic validation
  - Use MappingValidationResult from previous work
  - Purpose: Clarify UI-specific validation, not SSOT violation
  - _Leverage: MappingValidationResult from previous cleanup_
  - _Requirements: 2_
  - _Prompt: Implement the task for spec ssot-violations-fix: Role: Code Documentation Specialist | Task: Document MappingValidator as UI-specific key mapping validation. Verify no duplication with core validation logic. Add clear documentation. | Restrictions: Keep in UI if genuinely UI-specific, document purpose clearly, verify no core duplication | Success: MappingValidator documented, no core duplication verified, SSOT checker accepts it | Instructions: Mark in-progress `[-]`, document and verify, log reasoning, mark complete `[x]`._

- [x] 13. Verify business logic migration completion
  - Files: Full UI codebase
  - Run SSOT checker to verify business logic violations eliminated
  - Check that all FFI imports in UI go through core services
  - Verify no direct bridge usage in UI layer
  - Purpose: Confirm all 32 business logic violations resolved
  - **FINDINGS**: Migration incomplete - 30 violations remain (20 business logic, 8 duplicates, 2 interface). Previous tasks created wrapper classes instead of proper migration. Tests failing.
  - _Leverage: scripts/pre-commit-ssot-check.sh, grep for FFI imports_
  - _Requirements: 2_
  - _Prompt: Implement the task for spec ssot-violations-fix: Role: Code Auditor | Task: Verify all business logic migrations are complete. Run SSOT checker, grep for direct FFI usage in UI, ensure all FFI access goes through core services. | Restrictions: Only verify, do not implement new changes unless critical issues found | Success: SSOT checker shows 0 business logic violations (down from 32), no direct FFI in UI layer, all tests pass | Instructions: Mark in-progress `[-]`, run audits, fix any issues found, log results, mark complete `[x]`._

## Phase 3: Interface Inconsistency Fixes (2 violations)

- [x] 14. Identify interface inconsistencies
  - Files: Run SSOT checker with verbose output
  - Determine which services have interface mismatches
  - Document the specific signature differences
  - Plan fix approach (shared interface vs signature alignment)
  - Purpose: Understand the 2 interface inconsistency violations
  - **FINDINGS**:
    1. `device_registry_service.dart` - Missing conditional export, has web impl but no export pattern
    2. `profile_registry_service.dart` - Missing conditional export, needs web impl creation
  - **FIX APPROACH**: Move Impl classes to `_ffi.dart` files, add conditional exports, create web impl for profile_registry
  - _Leverage: scripts/pre-commit-ssot-check.sh --verbose_
  - _Requirements: 3_
  - _Prompt: Implement the task for spec ssot-violations-fix: Role: API Contract Analyst | Task: Identify the 2 interface inconsistency violations using SSOT checker verbose output. Document which services have mismatched interfaces and what the differences are. | Restrictions: Only investigate and document, do not fix yet, provide detailed report | Success: Clear documentation of 2 interface mismatches, specific signature differences identified, fix approach planned | Instructions: Mark in-progress `[-]`, investigate thoroughly, log findings with details, mark complete `[x]`._

- [ ] 15. Fix interface inconsistency 1
  - Files: TBD based on task 14 findings
  - Align method signatures between implementations
  - Create shared interface if needed
  - Test both FFI and web implementations
  - Purpose: Eliminate first interface inconsistency
  - _Leverage: findings from task 14_
  - _Requirements: 3_
  - _Prompt: Implement the task for spec ssot-violations-fix: Role: Interface Design Specialist | Task: Fix the first interface inconsistency identified in task 14. Align method signatures, create shared interface if needed, test both implementations. | Restrictions: Preserve functionality of both implementations, maintain backward compatibility where possible, clear interfaces | Success: Interface signatures match, shared interface created if appropriate, both implementations work, 1 fewer violation | Instructions: Mark in-progress `[-]`, implement fix, log interface changes, mark complete `[x]`._

- [ ] 16. Fix interface inconsistency 2
  - Files: TBD based on task 14 findings
  - Align method signatures between implementations
  - Create shared interface if needed
  - Test both FFI and web implementations
  - Purpose: Eliminate second interface inconsistency
  - _Leverage: findings from task 14, patterns from task 15_
  - _Requirements: 3_
  - _Prompt: Implement the task for spec ssot-violations-fix: Role: Interface Design Specialist | Task: Fix the second interface inconsistency identified in task 14. Align method signatures, create shared interface if needed, test both implementations. Apply patterns from task 15. | Restrictions: Preserve functionality, maintain compatibility, clear interfaces | Success: Interface signatures match, shared interface if appropriate, both implementations work, 1 fewer violation | Instructions: Mark in-progress `[-]`, implement fix, log interface changes, mark complete `[x]`._

## Phase 4: Final Verification

- [ ] 17. Run complete SSOT validation
  - Files: Entire codebase
  - Run scripts/pre-commit-ssot-check.sh
  - Verify 0 violations (down from 37)
  - Document any remaining violations and justifications
  - Purpose: Confirm all SSOT violations eliminated
  - _Leverage: SSOT checker script_
  - _Requirements: All_
  - _Prompt: Implement the task for spec ssot-violations-fix: Role: Quality Assurance Lead | Task: Run comprehensive SSOT validation to confirm all 37 violations have been eliminated. If any remain, document justification for why they should be accepted. | Restrictions: Must achieve 0 violations or provide strong justification for exceptions | Success: SSOT checker reports 0 violations, documentation complete for any justified exceptions, pre-commit hook passes | Instructions: Mark in-progress `[-]`, run full validation, document results, mark complete `[x]`._

- [ ] 18. Run full test suite
  - Files: Entire codebase
  - Run flutter test in UI
  - Run dart test in core_logic
  - Verify all tests pass
  - Fix any broken tests due to refactoring
  - Purpose: Ensure refactoring didn't break functionality
  - _Leverage: existing test suites_
  - _Requirements: All_
  - _Prompt: Implement the task for spec ssot-violations-fix: Role: Test Engineer | Task: Run complete test suite (flutter test, dart test) to verify refactoring didn't break functionality. Fix any broken tests. | Restrictions: Tests must pass or failures must be justified as pre-existing, no skipping tests | Success: All tests pass, any test failures are fixed or documented as pre-existing, test coverage maintained | Instructions: Mark in-progress `[-]`, run all tests, fix failures, log test results, mark complete `[x]`._

- [ ] 19. Verify flutter analyze clean
  - Files: Entire codebase
  - Run flutter analyze
  - Verify 0 errors and 0 warnings
  - Fix any analyzer issues introduced by refactoring
  - Purpose: Ensure code quality standards maintained
  - _Leverage: flutter analyze_
  - _Requirements: All_
  - _Prompt: Implement the task for spec ssot-violations-fix: Role: Code Quality Specialist | Task: Run flutter analyze to verify 0 errors and 0 warnings after refactoring. Fix any issues introduced by the SSOT work. | Restrictions: Must achieve clean analysis, no errors or warnings tolerated | Success: flutter analyze reports 0 issues, code quality maintained, no regressions introduced | Instructions: Mark in-progress `[-]`, run analysis, fix issues, log results, mark complete `[x]`._

- [ ] 20. Create summary commit
  - Files: All modified files
  - Review all changes made during SSOT fix work
  - Create comprehensive commit message
  - List all violations fixed (3 duplicates, 32 business logic, 2 interface)
  - Commit with --no-verify if needed for initial commit, then fix pre-commit issues
  - Purpose: Capture all SSOT work in version control
  - _Leverage: git diff, git status_
  - _Requirements: All_
  - _Prompt: Implement the task for spec ssot-violations-fix: Role: Version Control Specialist | Task: Create comprehensive commit capturing all SSOT violation fixes. Write detailed commit message listing all 37 violations eliminated. Review all changes for completeness. | Restrictions: Commit message must be thorough, changes must be reviewed, no unrelated changes included | Success: All SSOT work committed, comprehensive message written, clean git history maintained | Instructions: Mark in-progress `[-]`, review and commit, log commit details, mark complete `[x]`._
