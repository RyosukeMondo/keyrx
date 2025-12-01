# Tasks Document

- [x] 1. Wire state snapshot stream with latency/timing (Rust)
  - Files: core/src/engine/state.rs; core/src/engine/event_loop.rs; core/src/ffi/exports.rs
  - Publish full state (layers, modifiers, held, pending, event summary, latency_us, timing) on each event/state change; skip safely if no callback; ensure initial snapshot delivered.
  - _Leverage: existing StateStore getters; process_event timing; keyrx_on_state callback plumbing._
  - _Requirements: 1,4_
  - _Prompt: Implement the task for spec live-ui-ffi-hardening, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust systems engineer focused on low-latency async engines | Task: Add state snapshot construction with latency measurement and timing fields, serialized to JSON and emitted via keyrx_on_state; include initial snapshot and safe no-callback handling | Restrictions: Keep added overhead <1ms total processing, avoid panics across FFI, reuse existing state accessors, do not regress thread safety | _Leverage: StateStore, event loop timing hooks, FFI callback utilities | _Requirements: 1,4 | Success: Snapshot JSON includes required fields, delivers on each event/change, initial snapshot sent, latency measured from ingress to post-decision, skipped cleanly when no callback, tests planned._

- [x] 2. Implement shared eval against active runtime (Rust)
  - Files: core/src/scripting/runtime.rs; core/src/ffi/exports.rs
  - Route keyrx_eval to the live RhaiRuntime via synchronized handle/channel; return ok:/error: strings; guard when engine not initialized.
  - _Leverage: existing RhaiRuntime setup; existing FFI string helpers._
  - _Requirements: 2,4_
  - _Prompt: Implement the task for spec live-ui-ffi-hardening, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust/Rhai runtime engineer | Task: Expose thread-safe eval on the running runtime, serializing access and returning ok:<value> or error:<message>; handle engine-not-initialized gracefully | Restrictions: No unsynchronized mutable access, no panics across FFI, respect Rhai sandbox limits, keep API compatible with existing FFI string ownership | _Leverage: runtime handle/channel patterns, FFI string alloc/free helpers | _Requirements: 2,4 | Success: keyrx_eval executes on live runtime, concurrent calls serialized, engine-missing returns error:, responses prefixed correctly._

- [x] 3. Export canonical key registry via FFI (Rust)
  - Files: core/src/ffi/exports.rs; core/src/drivers/keycodes/definitions.rs (or registry source)
  - Extend keyrx_list_keys to return JSON objects {name, aliases, evdev, vk}; ensure aliases/metadata mirror engine registry.
  - _Leverage: existing key definitions and alias tables; existing JSON serialization._
  - _Requirements: 3,4_
  - _Prompt: Implement the task for spec live-ui-ffi-hardening, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust FFI engineer | Task: Build canonical key registry payload with names/aliases/evdev/vk codes and expose via keyrx_list_keys JSON; standardize ok:/error: | Restrictions: Do not change canonical names; keep allocation patterns FFI-safe; avoid large regressions in startup or list time | _Leverage: key definitions/aliases tables, serde JSON, FFI string helpers | _Requirements: 3,4 | Success: keyrx_list_keys returns correct JSON array with metadata, errors return error:, compatible with keyrx_free_string._

- [x] 4. Extend Dart bridge for state/eval/key registry
  - Files: ui/lib/ffi/bridge.dart; ui/lib/ffi/bindings.dart (if needed); ui/lib/state/engine_snapshot.dart (or equivalent)
  - Parse modifiers/pending/timing in BridgeState/EngineSnapshot; normalize ok:/error: for eval; add listKeys fetch with fallback behavior.
  - _Leverage: existing bridge decoding and state stream handling._
  - _Requirements: 1,2,3,4_
  - _Prompt: Implement the task for spec live-ui-ffi-hardening, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Flutter/Dart FFI bridge engineer | Task: Update bridge models to include modifiers/pending/timing, standardize ok:/error: eval handling, add listKeys bridge method with fallback on failure | Restrictions: Do not break existing stream wiring; keep models immutable-friendly; handle null/absent timing gracefully | _Leverage: current bridge.dart stream mapping, existing EngineSnapshot models | _Requirements: 1,2,3,4 | Success: New fields parsed and exposed to UI, eval errors surfaced with error:, key list fetched on init with graceful fallback._

- [x] 5. Update Flutter UI (debugger, console, editor)
  - Files: ui/lib/pages/debugger.dart; ui/lib/pages/console.dart; ui/lib/pages/editor.dart (or keymapping widgets)
  - Render modifiers/pending/timing in debugger with scrollable layout and thresholds; style console responses by ok:/error:; fetch canonical keys on init and show inline invalid key badges.
  - _Leverage: existing debugger timeline components; console widgets; key mappings manager._
  - _Requirements: 1,3,4_
  - _Prompt: Implement the task for spec live-ui-ffi-hardening, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Flutter UI engineer | Task: Enhance debugger to display modifiers/pending/timing (including thresholds), update console styling for ok:/error:, load canonical key list into KeyMappings and show inline invalid indicators | Restrictions: Maintain responsiveness and scrolling to avoid overflow; preserve existing visual language; avoid blocking UI on key fetch (fallback allowed) | _Leverage: existing debugger timeline/widget structure, console output handling, key mapping editor state | _Requirements: 1,3,4 | Success: UI shows new state fields without overflow, console visually distinguishes errors including engine not initialized, editor uses fetched keys and flags invalid inline._

- [x] 6. Add Rust tests for FFI/state/eval/registry
  - Files: core/tests/ffi_state_tests.rs (or similar); core/src/ffi/exports.rs (unit modules)
  - Tests for keyrx_eval ok/error, keyrx_list_keys schema/content, keyrx_on_state serialization with timing/latency.
  - _Leverage: existing test harnesses/mocks; serde_json assertions._
  - _Requirements: 1,2,3,5_
  - _Prompt: Implement the task for spec live-ui-ffi-hardening, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Rust test engineer | Task: Write unit/integration tests covering eval success/error paths, key registry JSON schema/fields, and state snapshot serialization with timing/latency | Restrictions: Keep tests deterministic; avoid long-running async; no reliance on real hardware | _Leverage: current ffi tests/mocks, serde_json, engine state builders | _Requirements: 1,2,3,5 | Success: Tests fail on schema/timing regressions, cover ok/error prefixes, run in CI quickly._

- [ ] 7. Add Flutter tests for debugger/console/editor
  - Files: ui/test/debugger_test.dart; ui/test/console_test.dart; ui/test/editor_key_validation_test.dart (or equivalents)
  - Widget tests for rendering modifiers/pending/timing; console error display; editor key validation using fetched key list; integration smoke with mocked stateStream including timing.
  - _Leverage: existing test utils/mocks; fake stateStream fixtures._
  - _Requirements: 1,3,4,5_
  - _Prompt: Implement the task for spec live-ui-ffi-hardening, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Flutter QA engineer | Task: Add widget/integration tests to validate debugger rendering of new fields, console error styling on error:, editor validation from fetched keys, and mocked stateStream with timing | Restrictions: Keep tests deterministic and non-flaky; avoid real FFI; use mocks/fakes for streams and bridge calls | _Leverage: Flutter test framework, existing mocks/fakes, stateStream fixtures | _Requirements: 1,3,4,5 | Success: Tests assert presence of modifiers/pending/timing, console shows error style, editor flags invalid keys, mocked stream passes._
