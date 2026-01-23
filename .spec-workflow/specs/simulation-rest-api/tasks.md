# Tasks Document: Simulation REST API

- [x] 1. Fix WASM env-shim to provide `now()` function
  - File: keyrx_ui/src/wasm/env-shim.js
  - Add `now()` function that returns high-resolution timestamp in nanoseconds
  - Export as both named export and in default object
  - Purpose: Resolve WASM "Import #10 'env' 'now': function import requires a callable" error
  - _Leverage: performance.now() browser API, existing shim structure_
  - _Requirements: 1.1, 1.2, 1.3_
  - _Prompt: Implement the task for spec simulation-rest-api, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Frontend Developer specializing in WASM integration | Task: Fix the env-shim.js to provide a `now()` function that returns BigInt(Math.floor(performance.now() * 1_000_000)) for nanosecond precision timestamps required by Rust's Instant::now() in WASM. The current shim exports empty object and causes "env.now" import error. | Restrictions: Do not modify WASM build configuration, keep existing export default structure, use BigInt for precision | Success: WASM module loads without console errors, `wasm_init()` completes successfully, SimulatorPage renders without WASM error banner. Mark task as in-progress [-] before starting in tasks.md, log implementation with log-implementation tool after completion, then mark as complete [x]._

- [x] 2. Create SimulationService in services module
  - File: keyrx_daemon/src/services/simulation_service.rs
  - Wrap SimulationEngine with service layer providing profile loading, replay, and reset
  - Use Mutex<Option<SimulationEngine>> for thread-safe state
  - Purpose: Provide injectable simulation service for REST API endpoints
  - _Leverage: keyrx_daemon/src/config/simulation_engine.rs, existing service patterns_
  - _Requirements: 2.1, 2.2, 2.3, 2.4, 5.1_
  - _Prompt: Implement the task for spec simulation-rest-api, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Backend Developer with expertise in service layer patterns | Task: Create SimulationService struct with methods: load_profile(name), replay(sequence), run_scenario(scenario), run_all_scenarios(), replay_dsl(dsl, seed), reset(). Use Mutex<Option<SimulationEngine>> for state. Config directory path passed at construction. Follow existing service patterns in the codebase. | Restrictions: Do not modify SimulationEngine, maintain thread safety via Mutex, return proper SimulationError types | Success: Service compiles, all methods delegate to SimulationEngine correctly, reset clears state. Mark task as in-progress [-] before starting in tasks.md, log implementation with log-implementation tool after completion, then mark as complete [x]._

- [x] 3. Register SimulationService in services module
  - File: keyrx_daemon/src/services/mod.rs
  - Add `pub mod simulation_service;` and `pub use simulation_service::SimulationService;`
  - Purpose: Export SimulationService for use in AppState
  - _Leverage: existing module structure in services/mod.rs_
  - _Requirements: 2.1_
  - _Prompt: Implement the task for spec simulation-rest-api, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Developer | Task: Add simulation_service module export to services/mod.rs following existing pattern. Add both `pub mod` and `pub use` statements. | Restrictions: Do not remove existing exports, maintain alphabetical ordering if present | Success: SimulationService importable from crate::services. Mark task as in-progress [-] before starting in tasks.md, log implementation with log-implementation tool after completion, then mark as complete [x]._

- [x] 4. Extend AppState with SimulationService
  - File: keyrx_daemon/src/web/mod.rs
  - Add `simulation_service: Arc<SimulationService>` field to AppState struct
  - Update AppState::new() to accept and store SimulationService
  - Purpose: Enable dependency injection of SimulationService into API handlers
  - _Leverage: existing AppState pattern with Arc-wrapped services_
  - _Requirements: 2.1, 2.2_
  - _Prompt: Implement the task for spec simulation-rest-api, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Backend Developer with expertise in dependency injection | Task: Add simulation_service field to AppState struct, update constructor to accept Arc<SimulationService>, update all call sites. Follow existing pattern with Arc-wrapped services like profile_service, device_service. | Restrictions: Do not change existing fields, maintain constructor parameter order consistency | Success: AppState compiles with new field, simulation_service accessible from handlers. Mark task as in-progress [-] before starting in tasks.md, log implementation with log-implementation tool after completion, then mark as complete [x]._

- [ ] 5. Update simulator.rs endpoints to use SimulationService
  - File: keyrx_daemon/src/web/api/simulator.rs
  - Add State<Arc<AppState>> extraction to handlers
  - Add POST /api/simulator/load-profile endpoint
  - Add POST /api/simulator/scenarios/all endpoint
  - Update simulate_events to use SimulationService.replay/run_scenario
  - Update reset_simulator to call SimulationService.reset()
  - Purpose: Wire REST endpoints to actual simulation logic
  - _Leverage: existing axum handler patterns, ApiError for errors_
  - _Requirements: 2.1, 2.2, 2.3, 3.1, 3.2, 3.3, 4.1, 4.2, 5.1_
  - _Prompt: Implement the task for spec simulation-rest-api, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust API Developer with expertise in axum | Task: Update simulator.rs to use SimulationService. Add load-profile endpoint. Update simulate_events to support scenario, dsl, and events parameters. Add scenarios/all endpoint. All handlers extract State<Arc<AppState>> and call simulation_service methods. Return proper JSON responses with success/error. | Restrictions: Maintain backward compatibility with existing response format where possible, use ApiError for errors, do not bypass service layer | Success: All endpoints return structured JSON, load-profile stores state, simulate_events works with all input types, scenarios/all returns pass/fail counts. Mark task as in-progress [-] before starting in tasks.md, log implementation with log-implementation tool after completion, then mark as complete [x]._

- [ ] 6. Initialize SimulationService in main.rs
  - File: keyrx_daemon/src/main.rs
  - Create SimulationService with config directory path
  - Pass Arc<SimulationService> to AppState::new()
  - Purpose: Complete dependency injection chain
  - _Leverage: existing service initialization patterns in main.rs_
  - _Requirements: 2.1_
  - _Prompt: Implement the task for spec simulation-rest-api, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Application Developer | Task: Initialize SimulationService in main.rs with config_dir path (same as used by other services). Wrap in Arc and pass to AppState::new(). Follow existing service initialization patterns. | Restrictions: Do not modify config_dir resolution logic, maintain initialization order | Success: Daemon starts successfully, SimulationService available in handlers. Mark task as in-progress [-] before starting in tasks.md, log implementation with log-implementation tool after completion, then mark as complete [x]._

- [ ] 7. Update TestApp fixture with SimulationService
  - File: keyrx_daemon/tests/common/test_app.rs
  - Create SimulationService in TestApp::new()
  - Pass to AppState::new()
  - Purpose: Enable integration testing of simulation endpoints
  - _Leverage: existing TestApp patterns, temp directory for config_
  - _Requirements: 6.1_
  - _Prompt: Implement the task for spec simulation-rest-api, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Test Developer | Task: Update TestApp::new() to create SimulationService using the temp config directory already set up by TestApp. Wrap in Arc and pass to AppState::new(). Follow existing service initialization patterns in the fixture. | Restrictions: Use existing temp directory, do not change TestApp public interface | Success: TestApp compiles, integration tests can call simulation endpoints. Mark task as in-progress [-] before starting in tasks.md, log implementation with log-implementation tool after completion, then mark as complete [x]._

- [ ] 8. Create backend integration tests for simulation endpoints
  - File: keyrx_daemon/tests/simulator_api_test.rs
  - Test load-profile endpoint
  - Test simulate_events with scenario, DSL, and custom events
  - Test deterministic behavior with seed
  - Test error cases (no profile, unknown scenario)
  - Test reset endpoint
  - Purpose: Verify simulation API contract and error handling
  - _Leverage: keyrx_daemon/tests/common/test_app.rs, existing api test patterns_
  - _Requirements: 2.1, 2.2, 2.3, 3.1, 3.2, 3.3, 4.1, 4.2, 5.1, 5.2_
  - _Prompt: Implement the task for spec simulation-rest-api, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust Test Engineer | Task: Create comprehensive integration tests for simulation endpoints. Use TestApp fixture. Create test profile before testing. Test all endpoints: load-profile, simulate_events (scenario/dsl/events), scenarios/all, reset. Verify deterministic results with same seed. Test error responses. Use #[tokio::test] and #[serial_test::serial] attributes. | Restrictions: Use existing test patterns, do not modify TestApp public interface, clean up test profiles | Success: All tests pass, cover success and error scenarios, verify response structures. Mark task as in-progress [-] before starting in tasks.md, log implementation with log-implementation tool after completion, then mark as complete [x]._

- [ ] 9. Extend ApiHelpers with simulation methods
  - File: keyrx_ui/tests/e2e/fixtures/api.ts
  - Add TypeScript interfaces for simulation types
  - Add loadSimulatorProfile method
  - Add simulateEventsDsl method
  - Add simulateScenario method
  - Add runAllScenarios method
  - Purpose: Enable E2E testing of simulation endpoints
  - _Leverage: existing ApiHelpers patterns, Zod validation if applicable_
  - _Requirements: 6.1, 6.2_
  - _Prompt: Implement the task for spec simulation-rest-api, first run spec-workflow-guide to get the workflow guide then implement the task: Role: TypeScript Developer specializing in Playwright E2E testing | Task: Extend ApiHelpers class with simulation methods. Add interfaces: SimulationResponse, OutputEvent, AllScenariosResponse, ScenarioResult. Add methods: loadSimulatorProfile(name), simulateEventsDsl(dsl, seed?), simulateScenario(name), runAllScenarios(). Follow existing method patterns with proper error handling. | Restrictions: Do not modify existing methods, maintain consistent error handling patterns | Success: All methods compile, return typed responses, handle errors consistently. Mark task as in-progress [-] before starting in tasks.md, log implementation with log-implementation tool after completion, then mark as complete [x]._

- [ ] 10. Create Playwright E2E tests for simulation API
  - File: keyrx_ui/tests/e2e/api/simulator.spec.ts
  - Test load profile endpoint
  - Test built-in scenarios (tap-hold under/over threshold)
  - Test DSL event simulation
  - Test deterministic results with seed
  - Test error handling (unknown scenario, no profile)
  - Test reset endpoint
  - Purpose: Verify simulation API works end-to-end through Playwright
  - _Leverage: keyrx_ui/tests/e2e/fixtures/api.ts, daemon fixture patterns_
  - _Requirements: 6.1, 6.2, 6.3, 6.4_
  - _Prompt: Implement the task for spec simulation-rest-api, first run spec-workflow-guide to get the workflow guide then implement the task: Role: QA Engineer with expertise in Playwright E2E testing | Task: Create E2E tests for simulation API. Use daemon fixture to create test profiles. Test all endpoints via ApiHelpers. Verify scenario outputs contain expected keys (Escape for tap, Control for hold). Verify DSL produces deterministic results with same seed. Test error handling. Clean up test profiles in afterEach. | Restrictions: Follow existing E2E test patterns, use daemon fixture for profile management, ensure test isolation | Success: All tests pass, verify correct output events, verify deterministic behavior, verify error responses. Mark task as in-progress [-] before starting in tasks.md, log implementation with log-implementation tool after completion, then mark as complete [x]._

- [ ] 11. Verify WASM and run full test suite
  - Commands: npm run build:wasm, npm run dev, cargo test -p keyrx_daemon simulator, npm run test:e2e -- --grep "Simulator"
  - Verify browser console has no WASM errors
  - Verify all backend tests pass
  - Verify all E2E tests pass
  - Purpose: Final verification of complete implementation
  - _Leverage: existing build and test scripts_
  - _Requirements: All_
  - _Prompt: Implement the task for spec simulation-rest-api, first run spec-workflow-guide to get the workflow guide then implement the task: Role: DevOps Engineer | Task: Run full verification: 1) Build WASM with npm run build:wasm, 2) Start dev server and check browser console for no env.now errors, 3) Run cargo test -p keyrx_daemon simulator for backend tests, 4) Run npm run test:e2e -- --grep "Simulator" for E2E tests. Document any failures and fix if needed. | Restrictions: Do not skip any verification step, document all results | Success: WASM loads without errors, all backend tests pass, all E2E tests pass. Mark task as in-progress [-] before starting in tasks.md, log implementation with log-implementation tool after completion, then mark as complete [x]._
