# Tasks Document

## Phase 1: Foundation

- [x] 1. Create Result type and FacadeError
  - File: `ui/lib/services/facade/result.dart`
  - Define sealed `Result<T>` class with Ok and Err variants
  - Create `FacadeError` with code, message, userMessage fields
  - Add factory constructors for common error types
  - Purpose: Explicit error handling without exceptions
  - _Leverage: Rust Result pattern, existing ErrorTranslator_
  - _Requirements: 5.1, 5.2, 5.3, 5.4_
  - _Prompt: Implement the task for spec flutter-service-facade, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Dart Developer with functional programming experience | Task: Create Result<T> and FacadeError types in ui/lib/services/facade/result.dart following requirements 5.1-5.4 | Restrictions: Use freezed for immutability, follow Rust Result semantics, integrate with ErrorTranslator | _Leverage: ErrorTranslator in ui/lib/services/, Rust Result pattern | Success: Result type compiles, error factories work, integrates with existing error translation | After completion: Mark task [-] as in-progress before starting, use log-implementation tool to record artifacts, then mark [x] when complete_

- [x] 2. Create FacadeState aggregated state model
  - File: `ui/lib/services/facade/facade_state.dart`
  - Define `FacadeState` with engine, device, validation, discovery status
  - Create status enums: EngineStatus, DeviceStatus, ValidationStatus, DiscoveryStatus
  - Add `FacadeState.initial()` factory
  - Purpose: Unified state representation
  - _Leverage: freezed for immutability_
  - _Requirements: 3.1, 3.2, 3.3_
  - _Prompt: Implement the task for spec flutter-service-facade, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Dart Developer with state management expertise | Task: Create FacadeState and status enums in ui/lib/services/facade/facade_state.dart following requirements 3.1-3.3 | Restrictions: Use freezed, comprehensive status coverage, initial factory | _Leverage: Existing service state patterns | Success: State model compiles, status enums cover all cases, freezed generates correctly | After completion: Mark task [-] as in-progress before starting, use log-implementation tool to record artifacts, then mark [x] when complete_

- [x] 3. Create KeyrxFacade abstract interface
  - File: `ui/lib/services/facade/keyrx_facade.dart`
  - Define abstract class with all public methods
  - Add stateStream and currentState getters
  - Include factory constructors for real and mock
  - Purpose: Contract for facade implementations
  - _Leverage: Existing service interfaces pattern_
  - _Requirements: 1.1, 1.4, 2.1_
  - _Prompt: Implement the task for spec flutter-service-facade, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Dart Developer with interface design expertise | Task: Create KeyrxFacade abstract interface in ui/lib/services/facade/keyrx_facade.dart following requirements 1.1, 1.4, 2.1 | Restrictions: Abstract class with factory constructors, comprehensive method coverage, clear documentation | _Leverage: Existing service interfaces like EngineService | Success: Interface compiles, covers all major operations, factories defined | After completion: Mark task [-] as in-progress before starting, use log-implementation tool to record artifacts, then mark [x] when complete_

## Phase 2: Implementation

- [x] 4. Implement KeyrxFacadeImpl core structure
  - File: `ui/lib/services/facade/keyrx_facade_impl.dart`
  - Create class implementing KeyrxFacade
  - Accept ServiceRegistry in constructor
  - Set up BehaviorSubject for state stream
  - Implement dispose() method
  - Purpose: Foundation for facade implementation
  - _Leverage: ServiceRegistry, rxdart_
  - _Requirements: 1.1, 6.1_
  - _Prompt: Implement the task for spec flutter-service-facade, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Dart Developer with reactive programming experience | Task: Create KeyrxFacadeImpl core structure in ui/lib/services/facade/keyrx_facade_impl.dart following requirements 1.1, 6.1 | Restrictions: Accept ServiceRegistry, use BehaviorSubject for state, proper disposal | _Leverage: ServiceRegistry in ui/lib/services/, rxdart patterns | Success: Class compiles, state stream works, disposal cleans up resources | After completion: Mark task [-] as in-progress before starting, use log-implementation tool to record artifacts, then mark [x] when complete_

- [x] 5. Implement engine operations (start, stop, status)
  - File: `ui/lib/services/facade/keyrx_facade_impl.dart`
  - Implement startEngine() with validate → load → start sequence
  - Implement stopEngine() with proper cleanup
  - Implement getEngineStatus()
  - Update state on each step
  - Purpose: Core engine control
  - _Leverage: EngineService methods_
  - _Requirements: 4.1, 4.2, 4.3_
  - _Prompt: Implement the task for spec flutter-service-facade, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Dart Developer implementing engine control | Task: Implement engine operations in KeyrxFacadeImpl following requirements 4.1-4.3 with proper sequencing | Restrictions: Validate before start, update state at each step, handle errors with rollback | _Leverage: EngineService in ui/lib/services/engine_service.dart | Success: Engine starts/stops correctly, state updates properly, errors handled gracefully | After completion: Mark task [-] as in-progress before starting, use log-implementation tool to record artifacts, then mark [x] when complete_

- [x] 6. Implement script operations (validate, load, save)
  - File: `ui/lib/services/facade/keyrx_facade_impl.dart`
  - Implement validateScript() returning ValidationResult
  - Implement loadScript() and saveScript()
  - Update validation state appropriately
  - Purpose: Script management through facade
  - _Leverage: ScriptFileService, validation FFI_
  - _Requirements: 1.2_
  - _Prompt: Implement the task for spec flutter-service-facade, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Dart Developer implementing script operations | Task: Implement script operations in KeyrxFacadeImpl | Restrictions: Return structured ValidationResult, handle file errors, update validation state | _Leverage: ScriptFileService, validation FFI bindings | Success: Scripts load/save/validate correctly, validation state reflects results | After completion: Mark task [-] as in-progress before starting, use log-implementation tool to record artifacts, then mark [x] when complete_

- [x] 7. Implement device operations (list, discovery)
  - File: `ui/lib/services/facade/keyrx_facade_impl.dart`
  - Implement listDevices() returning DeviceInfo list
  - Implement startDiscovery() and cancelDiscovery()
  - Update discovery state during process
  - Purpose: Device management through facade
  - _Leverage: DeviceService methods_
  - _Requirements: 1.2_
  - _Prompt: Implement the task for spec flutter-service-facade, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Dart Developer implementing device operations | Task: Implement device operations in KeyrxFacadeImpl | Restrictions: Return DeviceInfo models, handle discovery state transitions, proper cancellation | _Leverage: DeviceService in ui/lib/services/device_service.dart | Success: Devices list correctly, discovery works with state updates, cancellation cleans up | After completion: Mark task [-] as in-progress before starting, use log-implementation tool to record artifacts, then mark [x] when complete_

- [x] 8. Implement test operations
  - File: `ui/lib/services/facade/keyrx_facade_impl.dart`
  - Implement runTests() returning TestResults
  - Implement cancelTests()
  - Handle test progress callbacks
  - Purpose: Test execution through facade
  - _Leverage: TestService methods_
  - _Requirements: 1.2_
  - _Prompt: Implement the task for spec flutter-service-facade, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Dart Developer implementing test operations | Task: Implement test operations in KeyrxFacadeImpl | Restrictions: Return structured TestResults, handle progress, proper cancellation | _Leverage: TestService in ui/lib/services/test_service.dart | Success: Tests run correctly, results structured properly, cancellation works | After completion: Mark task [-] as in-progress before starting, use log-implementation tool to record artifacts, then mark [x] when complete_

- [x] 9. Implement state stream aggregation
  - File: `ui/lib/services/facade/keyrx_facade_impl.dart`
  - Combine streams from engine, device, validation services
  - Add 100ms debounce for rapid state changes
  - Emit aggregated FacadeState
  - Purpose: Unified state observation
  - _Leverage: rxdart combineLatest, debounce_
  - _Requirements: 3.1, 3.2, 3.3, 3.4_
  - _Prompt: Implement the task for spec flutter-service-facade, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Dart Developer with reactive streams expertise | Task: Implement state stream aggregation combining multiple service streams with debounce | Restrictions: Use rxdart combineLatest, 100ms debounce, emit FacadeState | _Leverage: rxdart operators, individual service streams | Success: Aggregated stream emits correctly, debounce works, all sources combined | After completion: Mark task [-] as in-progress before starting, use log-implementation tool to record artifacts, then mark [x] when complete_

## Phase 3: Testing Infrastructure

- [x] 10. Create MockKeyrxFacade
  - File: `ui/test/mocks/mock_keyrx_facade.dart`
  - Implement KeyrxFacade with mockito
  - Add default stubbing for common operations
  - Include state stream mock
  - Purpose: Easy mocking for widget tests
  - _Leverage: mockito, KeyrxFacade interface_
  - _Requirements: 2.1, 2.2, 2.3_
  - _Prompt: Implement the task for spec flutter-service-facade, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Dart Test Developer | Task: Create MockKeyrxFacade with sensible defaults for widget testing | Restrictions: Use mockito, provide default stubs, mock state stream | _Leverage: mockito patterns, KeyrxFacade interface | Success: Mock can be used in widget tests, stubs are easy to override | After completion: Mark task [-] as in-progress before starting, use log-implementation tool to record artifacts, then mark [x] when complete_

- [x] 11. Add KeyrxFacade unit tests
  - File: `ui/test/services/facade/keyrx_facade_test.dart`
  - Test each operation with mocked ServiceRegistry
  - Verify state transitions
  - Test error handling paths
  - Purpose: Validate facade logic
  - _Leverage: MockKeyrxFacade, test utilities_
  - _Requirements: All_
  - _Prompt: Implement the task for spec flutter-service-facade, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Dart Test Developer | Task: Create comprehensive unit tests for KeyrxFacadeImpl | Restrictions: Mock ServiceRegistry, test all operations, verify state transitions | _Leverage: test utilities, MockKeyrxFacade | Success: All facade methods tested, state transitions verified, error handling covered | After completion: Mark task [-] as in-progress before starting, use log-implementation tool to record artifacts, then mark [x] when complete_

- [x] 12. Add integration tests
  - File: `ui/test/services/facade/keyrx_facade_integration_test.dart`
  - Test with real ServiceRegistry (mock FFI)
  - Test multi-step operations end-to-end
  - Verify service coordination
  - Purpose: Validate facade integrates correctly with services
  - _Leverage: Real ServiceRegistry, mock bridge_
  - _Requirements: 2.4_
  - _Prompt: Implement the task for spec flutter-service-facade, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Dart Integration Test Developer | Task: Create integration tests for KeyrxFacade with real ServiceRegistry | Restrictions: Use real services with mock FFI, test full operations, verify coordination | _Leverage: ServiceRegistry.real with mock bridge | Success: Integration tests pass, multi-step operations work, services coordinate correctly | After completion: Mark task [-] as in-progress before starting, use log-implementation tool to record artifacts, then mark [x] when complete_

## Phase 4: Migration

- [x] 13. Add KeyrxFacade Provider
  - File: `ui/lib/state/providers.dart`
  - Add KeyrxFacade provider using ServiceRegistry
  - Configure proper disposal
  - Make available throughout widget tree
  - Purpose: DI for facade in widgets
  - _Leverage: Provider package, existing providers_
  - _Requirements: 6.1_
  - _Prompt: Implement the task for spec flutter-service-facade, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Flutter Developer with state management expertise | Task: Add KeyrxFacade provider in ui/lib/state/providers.dart | Restrictions: Use existing provider pattern, proper disposal, available app-wide | _Leverage: Existing providers in ui/lib/state/ | Success: Provider compiles, facade accessible via context, disposal works | After completion: Mark task [-] as in-progress before starting, use log-implementation tool to record artifacts, then mark [x] when complete_

- [ ] 14. Migrate EditorPage to use facade
  - File: `ui/lib/pages/editor_page.dart`
  - Replace direct service injections with KeyrxFacade
  - Update state subscriptions to use facade stateStream
  - Keep backward compatibility (services still accessible)
  - Purpose: First page migration, validate pattern
  - _Leverage: KeyrxFacade, existing EditorPage_
  - _Requirements: 1.1, 6.3_
  - _Prompt: Implement the task for spec flutter-service-facade, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Flutter Developer migrating pages | Task: Migrate EditorPage to use KeyrxFacade instead of direct service injection | Restrictions: Maintain same functionality, use facade for operations, keep services accessible for edge cases | _Leverage: Current editor_page.dart, KeyrxFacade | Success: EditorPage uses facade, functionality unchanged, tests pass | After completion: Mark task [-] as in-progress before starting, use log-implementation tool to record artifacts, then mark [x] when complete_

- [ ] 15. Update EditorPage tests
  - File: `ui/test/pages/editor_page_test.dart`
  - Replace service mocks with MockKeyrxFacade
  - Simplify test setup
  - Verify same coverage with less code
  - Purpose: Validate testing simplification
  - _Leverage: MockKeyrxFacade, existing tests_
  - _Requirements: 2.1, 2.2_
  - _Prompt: Implement the task for spec flutter-service-facade, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Flutter Test Developer | Task: Update EditorPage tests to use MockKeyrxFacade | Restrictions: Maintain same test coverage, simplify setup, measure code reduction | _Leverage: MockKeyrxFacade, current editor_page_test.dart | Success: Tests pass with facade mock, setup simpler, coverage maintained | After completion: Mark task [-] as in-progress before starting, use log-implementation tool to record artifacts, then mark [x] when complete_

- [ ] 16. Migrate DiscoveryPage to use facade
  - File: `ui/lib/pages/discovery_page.dart`
  - Replace direct service usage with facade methods
  - Use facade state for discovery progress
  - Purpose: Second page migration
  - _Leverage: KeyrxFacade, existing DiscoveryPage_
  - _Requirements: 1.1_
  - _Prompt: Implement the task for spec flutter-service-facade, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Flutter Developer migrating pages | Task: Migrate DiscoveryPage to use KeyrxFacade | Restrictions: Maintain same functionality, use facade for all operations | _Leverage: Current discovery page, KeyrxFacade | Success: DiscoveryPage uses facade, functionality unchanged, tests pass | After completion: Mark task [-] as in-progress before starting, use log-implementation tool to record artifacts, then mark [x] when complete_

- [ ] 17. Migrate remaining pages
  - Files: `ui/lib/pages/*.dart`
  - Apply facade pattern to TestPage, SettingsPage, etc.
  - Update corresponding tests
  - Purpose: Complete migration
  - _Leverage: Established pattern_
  - _Requirements: 1.1_
  - _Prompt: Implement the task for spec flutter-service-facade, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Flutter Developer completing migration | Task: Migrate remaining pages to use KeyrxFacade | Restrictions: Apply consistent pattern, update all tests, maintain functionality | _Leverage: Established pattern from EditorPage/DiscoveryPage | Success: All pages use facade, all tests pass, consistent pattern | After completion: Mark task [-] as in-progress before starting, use log-implementation tool to record artifacts, then mark [x] when complete_

## Phase 5: Documentation & Cleanup

- [ ] 18. Add facade usage documentation
  - File: `docs/flutter-facade.md`
  - Document facade API and usage patterns
  - Include migration guide for existing code
  - Add testing examples
  - Purpose: Developer documentation
  - _Leverage: Implementation from previous tasks_
  - _Requirements: Non-functional (usability)_
  - _Prompt: Implement the task for spec flutter-service-facade, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Technical Writer with Flutter expertise | Task: Create facade documentation with usage patterns and migration guide | Restrictions: Cover all facade methods, include examples, migration steps | _Leverage: Implementation details from all previous tasks | Success: Documentation complete, examples work, migration path clear | After completion: Mark task [-] as in-progress before starting, use log-implementation tool to record artifacts, then mark [x] when complete_

- [ ] 19. Add code comments and dartdoc
  - Files: `ui/lib/services/facade/*.dart`
  - Add comprehensive dartdoc to all public APIs
  - Include usage examples in doc comments
  - Purpose: IDE discoverability
  - _Leverage: Dart documentation conventions_
  - _Requirements: Non-functional (usability)_
  - _Prompt: Implement the task for spec flutter-service-facade, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Dart Developer with documentation expertise | Task: Add comprehensive dartdoc to all facade files | Restrictions: Follow Dart conventions, include examples, all public APIs documented | _Leverage: Dart documentation conventions | Success: All public APIs have dartdoc, examples compile, IDE shows docs | After completion: Mark task [-] as in-progress before starting, use log-implementation tool to record artifacts, then mark [x] when complete_

- [ ] 20. Clean up deprecated direct service usage
  - Files: Review all pages
  - Remove unused direct service injections
  - Update ServiceRegistry if methods no longer needed publicly
  - Add @Deprecated annotations to legacy access patterns
  - Purpose: Code cleanup
  - _Leverage: Facade migration completed_
  - _Requirements: 6.4_
  - _Prompt: Implement the task for spec flutter-service-facade, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Flutter Developer completing cleanup | Task: Clean up deprecated service usage after facade migration | Restrictions: Only remove confirmed unused code, add deprecation annotations, don't break anything | _Leverage: Migration completed in previous tasks | Success: Unused service injections removed, deprecations added, all tests pass | After completion: Mark task [-] as in-progress before starting, use log-implementation tool to record artifacts, then mark [x] when complete_
