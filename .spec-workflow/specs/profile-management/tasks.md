# Tasks Document

## Phase 1: Backend Profile Management (Rust)

- [ ] 1. Create profile manager in keyrx_daemon/src/profile_manager.rs
  - CRUD operations for profiles
  - Load/save .krx files
  - _Prompt: Role: Rust Developer | Task: Create profile manager with CRUD operations | Restrictions: File ≤500 lines, store profiles in ~/.config/keyrx/profiles/, error handling | Success: ✅ Profiles load/save_

- [ ] 2. Add profile REST API endpoints in keyrx_daemon/src/web/api.rs
  - GET /api/profiles, POST /api/profiles, PUT /api/profiles/:id, DELETE /api/profiles/:id, POST /api/profiles/:id/activate
  - _Prompt: Role: Rust API Developer | Task: Add profile REST endpoints | Restrictions: Add to existing api.rs, validate inputs, return JSON | Success: ✅ All endpoints work_

- [ ] 3. Add profile CLI commands in keyrx_daemon/src/cli.rs
  - `keyrx profile list`, `keyrx profile activate <name>`, `keyrx profile save <name>`
  - _Prompt: Role: CLI Developer | Task: Add profile CLI commands | Success: ✅ CLI commands work_

## Phase 2: React UI Components

- [ ] 4. Create ProfilesPage component in keyrx_ui/src/pages/ProfilesPage.tsx
  - List all profiles with metadata
  - Create/activate/delete actions
  - _Prompt: Role: React Developer | Task: Create profiles management page | Restrictions: File ≤400 lines, call REST API, show loading states | Success: ✅ Profiles display_

- [ ] 5. Create ProfileCard component in keyrx_ui/src/components/ProfileCard.tsx
  - Display profile with action buttons
  - Hover to show actions
  - _Prompt: Role: React UI Developer | Task: Create profile card component | Restrictions: File ≤200 lines, actions: activate/rename/duplicate/delete | Success: ✅ Card renders_

- [ ] 6. Create ProfileDialog component in keyrx_ui/src/components/ProfileDialog.tsx
  - Modal for create/rename profile
  - Name and description inputs
  - _Prompt: Role: React Form Developer | Task: Create profile dialog | Restrictions: File ≤200 lines, validate name (no special chars), require name | Success: ✅ Dialog works_

## Phase 3: Profile Operations

- [ ] 7. Implement profile activation
  - Call POST /api/profiles/:id/activate
  - Show notification on success
  - _Prompt: Role: Integration Developer | Task: Wire up profile activation | Success: ✅ Activation switches daemon config_

- [ ] 8. Implement profile export/import
  - Export as .zip with .krx + metadata.json
  - Import from .zip
  - _Prompt: Role: File I/O Developer | Task: Add export/import functionality | Success: ✅ Export/import works_

## Phase 4: Testing & Documentation

- [ ] 9. Write unit tests for profile manager (Rust)
  - Test CRUD operations
  - _Prompt: Role: Rust Test Engineer | Task: Test profile manager | Success: ✅ All tests pass_

- [ ] 10. Write component tests for ProfilesPage
  - Test profile list, actions
  - _Prompt: Role: React Test Engineer | Task: Test profiles UI | Success: ✅ Tests pass_

- [ ] 11. Write E2E test for profile workflow
  - Create → activate → rename → delete
  - _Prompt: Role: QA Automation Engineer | Task: Test full profile workflow | Success: ✅ E2E test passes_

- [ ] 12. Create documentation in docs/profile-management.md
  - How to use profiles
  - _Prompt: Role: Technical Writer | Task: Document profile management | Success: ✅ Docs complete_

- [ ] 13. Log implementation artifacts
  - Use spec-workflow log-implementation tool
  - _Prompt: Role: Documentation Engineer | Task: Log artifacts | Success: ✅ Artifacts logged_
