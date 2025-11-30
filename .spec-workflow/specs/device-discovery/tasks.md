# Tasks Document

- [x] 1. Scaffold discovery module and shared types
  - File: core/src/discovery/mod.rs; core/src/discovery/types.rs
  - Create module skeleton with DeviceId helper, schema_version constant, DeviceProfile and PhysicalKey structs, ProfileSource enum, and config path helper for `~/.config/keyrx/devices`.
  - Purpose: Establish shared types/contracts for registry, storage, and session logic.
  - _Leverage: core/src/drivers/traits.rs (device identifiers), existing config/path helpers (CLI utilities), serde stack_
  - _Requirements: 1,3,5_
  - _Prompt: Implement the task for spec device-discovery, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust systems developer focused on keyboard input | Task: Scaffold discovery module and shared types for device discovery (DeviceId, DeviceProfile, PhysicalKey, ProfileSource, schema_version, config path helper) following requirements 1/3/5 | Restrictions: Keep files under core/src/discovery/, no engine wiring yet, ensure serde derives and defaults are explicit | _Leverage: core/src/drivers/traits.rs, CLI config path helpers, serde | _Requirements: 1,3,5 | Success: Types compile with no warnings, schema_version constant exposed, config path helper resolves to ~/.config/keyrx/devices, ready for storage/session use; mark task to [-] when starting, log implementation when done, then mark [x]_ 

- [x] 2. Implement profile storage with atomic writes and schema validation
  - File: core/src/discovery/storage.rs
  - Implement read/write/migrate functions for DeviceProfile JSON, using temp-file + rename for atomicity and validating schema_version with fallback to default profile.
  - Purpose: Persist per-device profiles safely and recover from corruption.
  - _Leverage: core/src/discovery/types.rs, serde_json, std::fs utilities_
  - _Requirements: 3,5_
  - _Prompt: Implement the task for spec device-discovery, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust engineer specializing in persistence | Task: Build profile storage layer (read/write/migrate) with atomic writes, schema_version checks, and fallback handling for device discovery | Restrictions: Do not panic on IO/parse errors, return actionable errors, keep default profile accessible | _Leverage: core/src/discovery/types.rs, serde_json, fs utilities | _Requirements: 3,5 | Success: Storage functions unit-tested (corrupt file, version mismatch, write failure), atomic writes verified, default fallback preserved; mark task to [-] when starting, log implementation when done, then mark [x]_ 

- [x] 3. Build discovery session state machine
  - File: core/src/discovery/session.rs
  - Implement state machine that sequences expected positions, records scan_code→position, handles duplicates/ambiguities, and emits progress/summary structs.
  - Purpose: Capture physical layout deterministically for each device.
  - _Leverage: core/src/drivers/traits.rs Input events, tracing utilities, core/src/discovery/types.rs_
  - _Requirements: 2,4,5_
  - _Prompt: Implement the task for spec device-discovery, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust engineer focused on state machines | Task: Implement discovery session to guide key presses, detect duplicates, emit progress/summary, and support cancel/retry | Restrictions: Keep pure/side-effect-free aside from event intake, expose JSON-serializable progress structs, ensure emergency-exit bypass hook is honored | _Leverage: driver traits for device_id filtering, tracing, discovery types | _Requirements: 2,4,5 | Success: Unit tests cover happy path, duplicate handling, cancel, and completion; mark task to [-] when starting, log implementation when done, then mark [x]_ 

- [x] 4. Integrate registry and engine load path
  - File: core/src/discovery/registry.rs; core/src/engine/mod.rs (or equivalent init path)
  - Wire registry lookup into engine initialization to load per-device profiles, fall back to default on errors, and expose `discover_needed` signal for unknown devices.
  - Purpose: Ensure runtime uses correct profile per device and prompts when unknown.
  - _Leverage: core/src/discovery/storage.rs, core/src/drivers/traits.rs, engine initialization patterns_
  - _Requirements: 1,3,5_
  - _Prompt: Implement the task for spec device-discovery, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust engineer integrating subsystems | Task: Add registry layer to engine startup to load profiles by device_id, expose unknown-device trigger, and keep default fallback safe | Restrictions: Do not block event loop; errors should degrade gracefully; keep emergency-exit behavior intact | _Leverage: discovery storage, driver traits, engine init patterns | _Requirements: 1,3,5 | Success: Engine loads profiles automatically on reconnect, unknown devices flagged for discovery without crashes, unit/integration tests cover load/fallback; mark task to [-] when starting, log implementation when done, then mark [x]_ 

- [ ] 5. Add CLI command `keyrx discover`
  - File: core/src/cli/commands/discover.rs; CLI wiring files as needed
  - Implement CLI entry to run discovery for unknown or specified device, showing progress, handling skip/confirm, writing profile on completion, and respecting JSON/yes flags.
  - Purpose: Provide user-facing workflow aligned with CLI-first principle.
  - _Leverage: core/src/cli/mod.rs scaffolding, discovery session/registry, JSON output formatter_
  - _Requirements: 1,2,3,4,5_
  - _Prompt: Implement the task for spec device-discovery, first run spec-workflow-guide to get the workflow guide then implement the task: Role: CLI-focused Rust developer | Task: Create `keyrx discover` command to prompt unknown devices, drive discovery session, show progress, and save profiles with confirmation | Restrictions: Keep non-blocking behavior for other devices, support --device/--force/--json/--yes flags, preserve exit codes 0/2/3 | _Leverage: CLI scaffolding, discovery session, registry/storage | _Requirements: 1,2,3,4,5 | Success: CLI command covered by integration tests (happy path, skip, cancel, corrupt profile), outputs JSON when requested, leaves default active on failure; mark task to [-] when starting, log implementation when done, then mark [x]_ 

- [ ] 6. Emit discovery progress to FFI for GUI hook
  - File: core/src/ffi/exports.rs; core/src/discovery/session.rs (event publisher)
  - Add optional FFI callbacks/events for discovery progress, duplicate warnings, summary, and completion, reusing session outputs.
  - Purpose: Enable Flutter UI to visualize discovery without duplicating logic.
  - _Leverage: existing FFI bridge patterns, discovery session structs_
  - _Requirements: 2,5_
  - _Prompt: Implement the task for spec device-discovery, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust/FFI engineer | Task: Expose discovery progress events through FFI callbacks for GUI consumption, reusing session outputs | Restrictions: Keep ABI-stable types, gate behind feature flag if needed, avoid blocking FFI threads | _Leverage: FFI exports, discovery session structs | _Requirements: 2,5 | Success: FFI functions exported and tested with dummy listener, no panic paths, GUI can subscribe to progress; mark task to [-] when starting, log implementation when done, then mark [x]_ 

- [ ] 7. Add automated tests for discovery workflows
  - File: core/tests/discovery_workflow.rs (or similar integration test location)
  - Implement integration tests simulating device connect → discovery run → profile load, covering duplicate handling, corruption fallback, re-discovery, and emergency-exit bypass.
  - Purpose: Ensure end-to-end reliability and regression coverage.
  - _Leverage: discovery session/registry/storage, CLI harness, simulation utilities_
  - _Requirements: 1,2,3,4,5_
  - _Prompt: Implement the task for spec device-discovery, first run spec-workflow-guide to get the workflow guide then implement the task: Role: QA automation engineer for Rust | Task: Write integration tests for discovery workflow covering happy path, duplicates, corruption fallback, re-discovery, and emergency-exit behavior | Restrictions: Use simulated events; do not require real hardware; keep tests deterministic and parallel-safe | _Leverage: discovery modules, CLI harness/simulation utils | _Requirements: 1,2,3,4,5 | Success: Tests run in CI, cover success and failure paths, assert profile persistence and fallback behaviors; mark task to [-] when starting, log implementation when done, then mark [x]_ 
