# Tasks Document

## Phase 1: Backend Macro Recording (Rust)

- [ ] 1. Create macro recorder in keyrx_daemon/src/macro_recorder.rs
  - Recording mode: capture all key events with timestamps
  - Store events in buffer
  - _Prompt: Role: Rust Developer | Task: Create macro recorder with event capture | Restrictions: File ≤400 lines, capture press/release with μs timestamps, recording mode toggleable | Success: ✅ Events captured_

- [ ] 2. Add macro API endpoints in keyrx_daemon/src/web/api.rs
  - POST /api/macros/start-recording, POST /api/macros/stop-recording, GET /api/macros/recorded-events
  - _Prompt: Role: Rust API Developer | Task: Add macro REST endpoints | Success: ✅ Recording controlled via API_

## Phase 2: React UI Components

- [ ] 3. Create MacroRecorderPage component in keyrx_ui/src/pages/MacroRecorderPage.tsx
  - Record/stop buttons
  - Display captured events
  - Generate Rhai code
  - _Prompt: Role: React Developer | Task: Create macro recorder page | Restrictions: File ≤400 lines, show recording indicator, list events, preview Rhai code | Success: ✅ Recording UI works_

- [ ] 4. Create EventTimeline component in keyrx_ui/src/components/EventTimeline.tsx
  - Visual timeline for event editing
  - Drag events to adjust timing
  - _Prompt: Role: React UI Developer | Task: Create event timeline editor | Restrictions: File ≤300 lines, draggable events, timestamp editing | Success: ✅ Timeline editable_

## Phase 3: Macro Code Generation

- [ ] 5. Implement Rhai macro generator in keyrx_ui/src/utils/macroGenerator.ts
  - Convert MacroEvent[] to Rhai macro syntax
  - _Prompt: Role: Code Generation Expert | Task: Generate Rhai macro code | Restrictions: File ≤300 lines, generate valid Rhai syntax | Success: ✅ Valid Rhai generated_

- [ ] 6. Add macro testing with simulator
  - Test macro playback in WASM simulator
  - _Prompt: Role: Integration Developer | Task: Test macro via simulator | Success: ✅ Testing works_

## Phase 4: Macro Templates

- [ ] 7. Implement text snippet template
  - Convert text string to key sequence
  - _Prompt: Role: Template Developer | Task: Create text snippet template | Success: ✅ Text converts to keys_

- [ ] 8. Create macro templates library
  - Pre-built templates (email signatures, common snippets)
  - _Prompt: Role: UX Developer | Task: Create macro template library | Success: ✅ Templates available_

## Phase 5: Testing & Documentation

- [ ] 9. Write unit tests for macro recorder (Rust)
  - Test event capture, timestamp accuracy
  - _Prompt: Role: Rust Test Engineer | Task: Test macro recorder | Success: ✅ Tests pass_

- [ ] 10. Write component tests for MacroRecorderPage
  - Test recording flow, editing
  - _Prompt: Role: React Test Engineer | Task: Test macro UI | Success: ✅ Tests pass_

- [ ] 11. Write E2E test for macro workflow
  - Record → edit → test → save
  - _Prompt: Role: QA Automation Engineer | Task: Test full macro workflow | Success: ✅ E2E test passes_

- [ ] 12. Create documentation in docs/macro-recorder.md
  - How to record and use macros
  - _Prompt: Role: Technical Writer | Task: Document macro recorder | Success: ✅ Docs complete_

- [ ] 13. Log implementation artifacts
  - Use spec-workflow log-implementation tool
  - _Prompt: Role: Documentation Engineer | Task: Log artifacts | Success: ✅ Artifacts logged_
