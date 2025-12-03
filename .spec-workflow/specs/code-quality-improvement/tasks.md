# Tasks Document

## Phase 1: Rust Testability

- [x] 1. Create RuntimeContext to replace global RUNTIME_SLOT
  - Files: core/src/scripting/context.rs (new), core/src/scripting/runtime.rs (modify)
  - Extract runtime state into injectable struct, remove global static
  - _Leverage: existing runtime.rs patterns_
  - _Requirements: 1_
  - _Prompt: Implement the task for spec code-quality-improvement, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust systems engineer | Task: Create RuntimeContext struct containing Engine, pending_ops Vec, and Registry. Remove RUNTIME_SLOT global from runtime.rs. Pass context as parameter instead of accessing global | Restrictions: ≤200 lines for context.rs; update all callers; maintain thread safety via Arc<Mutex> if needed | _Leverage: runtime.rs | _Requirements: 1 | Success: Tests can run in parallel without #[serial] attribute._

- [x] 2. Create BypassController to replace global bypass state
  - Files: core/src/drivers/bypass.rs (new), core/src/drivers/emergency_exit.rs (modify)
  - Extract bypass mode into injectable component
  - _Leverage: existing emergency_exit.rs_
  - _Requirements: 1_
  - _Prompt: Implement the task for spec code-quality-improvement, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust systems engineer | Task: Create BypassController struct with AtomicBool and optional callback. Remove BYPASS_MODE and BYPASS_CALLBACK globals. Inject controller into engine | Restrictions: ≤80 lines; maintain atomic operations; backward compatible API | _Leverage: emergency_exit.rs | _Requirements: 1 | Success: Bypass mode testable in isolation._

- [x] 3. Make FFI callbacks injectable
  - Files: core/src/ffi/exports.rs (modify), core/src/ffi/callbacks.rs (new)
  - Extract callback holders into injectable registry
  - _Leverage: existing exports.rs patterns_
  - _Requirements: 1_
  - _Prompt: Implement the task for spec code-quality-improvement, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust FFI engineer | Task: Create CallbackRegistry struct holding discovery and state callbacks. Replace static OnceLock with injectable registry. Pass registry to FFI init function | Restrictions: ≤100 lines; maintain C ABI compatibility | _Leverage: exports.rs | _Requirements: 1 | Success: FFI callbacks mockable in tests._

- [x] 4. Add engine dependency traits
  - Files: core/src/traits/state.rs (new), core/src/engine/advanced.rs (modify)
  - Create KeyStateProvider, ModifierProvider, LayerProvider traits
  - _Leverage: existing traits/mod.rs patterns_
  - _Requirements: 9_
  - _Prompt: Implement the task for spec code-quality-improvement, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust architect | Task: Create traits: KeyStateProvider (is_pressed, press, release), ModifierProvider (is_active, activate, deactivate), LayerProvider (active_layer, push, pop). Make AdvancedEngine generic over these traits | Restrictions: ≤150 lines; implement traits for existing concrete types | _Leverage: traits/mod.rs | _Requirements: 9 | Success: Engine accepts mock state providers._

## Phase 2: Rust Complexity Reduction

- [x] 5. Split process_event_traced into sub-functions
  - Files: core/src/engine/processing.rs (new), core/src/engine/advanced.rs (modify)
  - Extract 5 focused functions from 98-line function
  - _Leverage: existing advanced.rs_
  - _Requirements: 5_
  - _Prompt: Implement the task for spec code-quality-improvement, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust refactoring engineer | Task: Split process_event_traced into: validate_and_check_safe_mode(), update_key_state(), resolve_decision(), apply_decision(), trace_event(). Each ≤50 lines. Main function ≤20 lines orchestrating calls | Restrictions: Maintain exact behavior; add unit tests for each sub-function | _Leverage: advanced.rs | _Requirements: 5 | Success: All sub-functions ≤50 lines with tests._

- [x] 6. Split run.rs (713 lines)
  - Files: core/src/cli/commands/run.rs (modify), run_builder.rs (new), run_recorder.rs (new), run_tracer.rs (new)
  - Extract builder, recorder, tracer into separate modules
  - _Leverage: existing run.rs_
  - _Requirements: 6_
  - _Prompt: Implement the task for spec code-quality-improvement, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust modularization engineer | Task: Split run.rs into: run.rs (RunCommand, ≤200 lines), run_builder.rs (RuntimeBuilder, ≤200 lines), run_recorder.rs (RecordingConfig, ≤150 lines), run_tracer.rs (TracingConfig, ≤150 lines) | Restrictions: Each file ≤500 lines; maintain public API | _Leverage: run.rs | _Requirements: 6 | Success: All files ≤500 lines._

- [ ] 7. Split discover.rs (712 lines)
  - Files: core/src/cli/commands/discover.rs (modify), discover_session.rs (new), discover_validation.rs (new)
  - Extract session management and validation
  - _Leverage: existing discover.rs_
  - _Requirements: 6_
  - _Prompt: Implement the task for spec code-quality-improvement, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust modularization engineer | Task: Split discover.rs into: discover.rs (DiscoverCommand, ≤200 lines), discover_session.rs (DiscoverySession state machine, ≤250 lines), discover_validation.rs (validation logic, ≤200 lines) | Restrictions: Each file ≤500 lines; clear module boundaries | _Leverage: discover.rs | _Requirements: 6 | Success: All files ≤500 lines._

- [ ] 8. Split runtime.rs (683 lines)
  - Files: core/src/scripting/runtime.rs (modify), pending_ops.rs (new), registry_sync.rs (new)
  - Extract pending operations and registry sync
  - _Leverage: existing runtime.rs_
  - _Requirements: 6_
  - _Prompt: Implement the task for spec code-quality-improvement, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust modularization engineer | Task: Split runtime.rs into: runtime.rs (RhaiRuntime core, ≤250 lines), pending_ops.rs (PendingOp handling, ≤200 lines), registry_sync.rs (Registry synchronization, ≤200 lines) | Restrictions: Each file ≤500 lines | _Leverage: runtime.rs | _Requirements: 6 | Success: All files ≤500 lines._

## Phase 3: Rust DRY

- [ ] 9. Unify layer action handlers
  - Files: core/src/engine/layer_actions.rs (new), core/src/engine/decision_engine.rs (modify)
  - Merge handle_layer_action and execute_layer_action
  - _Leverage: existing decision_engine.rs_
  - _Requirements: 8_
  - _Prompt: Implement the task for spec code-quality-improvement, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust refactoring engineer | Task: Create apply_layer_action() taking optional event context. Implement handle_layer_action and execute_layer_action as thin wrappers. Remove 95% duplicate code | Restrictions: ≤120 lines; backward compatible; add tests | _Leverage: decision_engine.rs | _Requirements: 8 | Success: Single implementation for layer actions._

- [ ] 10. Extract error handling helper for pending ops
  - Files: core/src/scripting/pending_ops.rs (modify)
  - Create apply_op helper for repeated pattern
  - _Leverage: existing runtime.rs patterns_
  - _Requirements: 8_
  - _Prompt: Implement the task for spec code-quality-improvement, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust engineer | Task: Create apply_op<F, T>() helper that executes operation and logs errors. Replace 12 repetitions of if let Err pattern with helper calls | Restrictions: ≤30 lines for helper; maintain logging behavior | _Leverage: pending_ops.rs | _Requirements: 8 | Success: Error pattern used once, called 12 times._

## Phase 4: Flutter Testability

- [ ] 11. Refactor EditorPage for constructor injection
  - Files: ui/lib/pages/editor_page.dart (modify)
  - Accept required services via constructor, remove nullable fields
  - _Leverage: existing editor_page.dart_
  - _Requirements: 2_
  - _Prompt: Implement the task for spec code-quality-improvement, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Flutter engineer | Task: Change EditorPage to accept required EngineService and MappingRepository via constructor. Remove nullable _engine field. Remove Provider.of in initState. Update instantiation sites | Restrictions: ≤50 lines changed; maintain functionality | _Leverage: editor_page.dart | _Requirements: 2 | Success: EditorPage testable with mock services._

- [ ] 12. Refactor ConsolePage for constructor injection
  - Files: ui/lib/pages/console.dart (modify)
  - Accept required services via constructor
  - _Leverage: existing console.dart_
  - _Requirements: 2_
  - _Prompt: Implement the task for spec code-quality-improvement, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Flutter engineer | Task: Change ConsolePage to accept required EngineService via constructor. Remove nullable field and Provider.of in initState | Restrictions: ≤40 lines changed | _Leverage: console.dart | _Requirements: 2 | Success: ConsolePage testable with mock services._

- [ ] 13. Refactor DebuggerPage for constructor injection
  - Files: ui/lib/pages/debugger_page.dart (modify)
  - Accept required services via constructor
  - _Leverage: existing debugger_page.dart_
  - _Requirements: 2_
  - _Prompt: Implement the task for spec code-quality-improvement, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Flutter engineer | Task: Change DebuggerPage to accept required EngineService and state Stream via constructor. Remove nullable fields and Provider.of in initState | Restrictions: ≤40 lines changed | _Leverage: debugger_page.dart | _Requirements: 2 | Success: DebuggerPage testable with mock services._

- [ ] 14. Refactor TrainingScreen for constructor injection
  - Files: ui/lib/pages/training_screen.dart (modify)
  - Accept required services via constructor
  - _Leverage: existing training_screen.dart_
  - _Requirements: 2_
  - _Prompt: Implement the task for spec code-quality-improvement, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Flutter engineer | Task: Change TrainingScreen to accept required AudioService via constructor. Remove nullable fields and Provider.of in initState | Restrictions: ≤40 lines changed | _Leverage: training_screen.dart | _Requirements: 2 | Success: TrainingScreen testable with mock services._

## Phase 5: Flutter SSOT

- [ ] 15. Create MappingRepository
  - Files: ui/lib/repositories/mapping_repository.dart (new)
  - Single source of truth for key mappings
  - _Leverage: existing editor_page.dart mapping patterns_
  - _Requirements: 3_
  - _Prompt: Implement the task for spec code-quality-improvement, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Flutter architect | Task: Create MappingRepository extending ChangeNotifier with: _mappings Map, _combos List, _tapHolds List. Add CRUD methods, generateScript(). Register in ServiceRegistry | Restrictions: ≤120 lines; immutable getters | _Leverage: editor_page.dart | _Requirements: 3 | Success: Single source for all mapping data._

- [ ] 16. Consolidate layer state in AppState
  - Files: ui/lib/state/app_state.dart (modify), ui/lib/pages/editor_page.dart (modify)
  - Remove duplicate layer state from EditorPage
  - _Leverage: existing app_state.dart_
  - _Requirements: 3_
  - _Prompt: Implement the task for spec code-quality-improvement, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Flutter state engineer | Task: Ensure AppState has definitive layer list with add/remove/setActive methods. Remove _layers from EditorPage. Use context.watch<AppState>().layers in build | Restrictions: ≤60 lines changed total | _Leverage: app_state.dart | _Requirements: 3 | Success: Layers stored in one place only._

- [ ] 17. Unify mapping models between editors
  - Files: ui/lib/pages/editor_page.dart (modify), ui/lib/pages/visual_editor_page.dart (modify)
  - Both editors use MappingRepository
  - _Leverage: MappingRepository from task 15_
  - _Requirements: 3_
  - _Prompt: Implement the task for spec code-quality-improvement, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Flutter engineer | Task: Modify EditorPage and VisualEditorPage to use shared MappingRepository instead of local state. Remove local _mappings fields | Restrictions: ≤80 lines changed total; switching editors preserves data | _Leverage: mapping_repository.dart | _Requirements: 3 | Success: Both editors share same mapping data._

## Phase 6: Flutter Complexity Reduction

- [ ] 18. Extract MappingValidator service
  - Files: ui/lib/services/mapping_validator.dart (new), ui/lib/pages/editor_page.dart (modify)
  - Move validation logic out of EditorPage
  - _Leverage: existing editor_page.dart validation code_
  - _Requirements: 7_
  - _Prompt: Implement the task for spec code-quality-improvement, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Flutter service engineer | Task: Create MappingValidator with validate(fromKey, mapping) returning ValidationResult. Move all validation from EditorPage._applyMapping to validator. EditorPage calls validator | Restrictions: ≤80 lines; testable without UI | _Leverage: editor_page.dart | _Requirements: 7 | Success: Validation logic in isolated service._

- [ ] 19. Extract ConsoleParser service
  - Files: ui/lib/services/console_parser.dart (new), ui/lib/pages/console.dart (modify)
  - Move parsing/classification logic out of ConsolePage
  - _Leverage: existing console.dart parsing code_
  - _Requirements: 7_
  - _Prompt: Implement the task for spec code-quality-improvement, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Flutter service engineer | Task: Create ConsoleParser with classify(text) returning ConsoleEntryType, needsInitButton(text) returning bool. Move parsing from _buildEntry to parser | Restrictions: ≤60 lines; pure functions | _Leverage: console.dart | _Requirements: 7 | Success: Parsing logic in isolated service._

- [ ] 20. Extract ScriptFileService
  - Files: ui/lib/services/script_file_service.dart (new), ui/lib/pages/editor_page.dart (modify)
  - Move file I/O out of EditorPage
  - _Leverage: existing editor_page.dart file operations_
  - _Requirements: 7_
  - _Prompt: Implement the task for spec code-quality-improvement, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Flutter service engineer | Task: Create ScriptFileService with saveScript(path, content) and loadScript(path). Move file operations from EditorPage._saveScript. Add to ServiceRegistry | Restrictions: ≤60 lines; async methods | _Leverage: editor_page.dart | _Requirements: 7 | Success: File I/O in isolated service._

- [ ] 21. Split exports.rs (635 lines)
  - Files: core/src/ffi/exports.rs (modify), exports_device.rs (new), exports_session.rs (new), exports_engine.rs (new)
  - Split FFI exports by domain
  - _Leverage: existing exports.rs_
  - _Requirements: 6_
  - _Prompt: Implement the task for spec code-quality-improvement, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust modularization engineer | Task: Split exports.rs into: exports.rs (init/common, ≤200 lines), exports_device.rs (device functions, ≤150 lines), exports_session.rs (session/replay, ≤150 lines), exports_engine.rs (engine control, ≤150 lines) | Restrictions: Each ≤500 lines; re-export from exports.rs | _Leverage: exports.rs | _Requirements: 6 | Success: All files ≤500 lines._

## Phase 7: Flutter DRY

- [ ] 22. Create StreamSubscriber mixin
  - Files: ui/lib/mixins/stream_subscriber.dart (new)
  - Extract repeated stream subscription pattern
  - _Leverage: existing debugger_page.dart, training_screen.dart patterns_
  - _Requirements: 8_
  - _Prompt: Implement the task for spec code-quality-improvement, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Flutter engineer | Task: Create StreamSubscriber mixin with subscribe<S>() method handling mounted checks, error handling, and auto-dispose. Track subscriptions in list, cancel all in dispose | Restrictions: ≤50 lines; generic over stream type | _Leverage: debugger_page.dart | _Requirements: 8 | Success: Stream subscription pattern in one place._

- [ ] 23. Apply StreamSubscriber mixin to pages
  - Files: ui/lib/pages/debugger_page.dart (modify), ui/lib/pages/training_screen.dart (modify)
  - Replace manual subscription code with mixin
  - _Leverage: StreamSubscriber from task 22_
  - _Requirements: 8_
  - _Prompt: Implement the task for spec code-quality-improvement, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Flutter engineer | Task: Add StreamSubscriber mixin to DebuggerPage and TrainingScreen. Replace manual StreamSubscription fields and dispose logic with mixin subscribe() calls | Restrictions: ≤30 lines changed per file; maintain behavior | _Leverage: stream_subscriber.dart | _Requirements: 8 | Success: Subscription boilerplate removed._

## Phase 8: Verification

- [ ] 24. Add unit tests for extracted Rust functions
  - Files: core/src/engine/processing.rs (add tests), core/src/engine/layer_actions.rs (add tests)
  - Test each new sub-function
  - _Leverage: existing test patterns_
  - _Requirements: All_
  - _Prompt: Implement the task for spec code-quality-improvement, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust test engineer | Task: Add #[cfg(test)] module to processing.rs and layer_actions.rs. Write unit tests for each extracted function. Cover happy path and error cases | Restrictions: ≥80% coverage for new code | _Leverage: existing tests | _Requirements: All | Success: All new functions have tests._

- [ ] 25. Add widget tests for refactored Flutter pages
  - Files: ui/test/pages/editor_page_test.dart (new), ui/test/pages/console_test.dart (new)
  - Test pages with mock services
  - _Leverage: Flutter test patterns_
  - _Requirements: All_
  - _Prompt: Implement the task for spec code-quality-improvement, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Flutter test engineer | Task: Create widget tests for EditorPage and ConsolePage using mock services. Test key user flows: add mapping, validate error, execute command | Restrictions: ≥80% coverage; use mockito or manual mocks | _Leverage: Flutter testing | _Requirements: All | Success: Pages testable with mocks._

- [ ] 26. Verify parallel test execution
  - Files: N/A (CI verification)
  - Confirm tests run in parallel without failures
  - _Leverage: cargo nextest_
  - _Requirements: 1_
  - _Prompt: Implement the task for spec code-quality-improvement, first run spec-workflow-guide to get the workflow guide then implement the task: Role: QA engineer | Task: Remove all #[serial] attributes from Rust tests. Run cargo nextest run without --test-threads=1. Verify no race conditions or flaky tests | Restrictions: All tests must pass in parallel | _Leverage: nextest | _Requirements: 1 | Success: Tests pass with parallel execution._

- [ ] 27. Verify no performance regression
  - Files: N/A (benchmark verification)
  - Run benchmarks before/after, compare
  - _Leverage: existing benches/_
  - _Requirements: All NFRs_
  - _Prompt: Implement the task for spec code-quality-improvement, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Performance engineer | Task: Run cargo bench before starting refactoring (save baseline). Run again after completion. Verify no regression >5% on any metric | Restrictions: Document any performance changes | _Leverage: benches/ | _Requirements: All | Success: Performance within 5% of baseline._
