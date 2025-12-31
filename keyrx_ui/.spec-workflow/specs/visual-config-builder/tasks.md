# Tasks Document

## Phase 1: Drag-and-Drop Foundation

- [x] 1. Install drag-and-drop dependencies
  - Add @dnd-kit/core@^6.1.0, @dnd-kit/sortable@^8.0.0
  - _Prompt: Role: Frontend Build Engineer | Task: Add dnd-kit dependencies | Success: ✅ Installed_

- [x] 2. Create ConfigState type definitions in keyrx_ui/src/types/configBuilder.ts
  - Define ConfigState, Layer, Mapping, Modifier types
  - _Prompt: Role: TypeScript Types Architect | Task: Define config builder state types | Restrictions: File ≤200 lines, strict mode | Success: ✅ Types defined_

- [x] 3. Create Zustand config builder store in keyrx_ui/src/store/configBuilderStore.ts
  - State: ConfigState with layers, modifiers, locks
  - Actions: addLayer, removeLayer, addMapping, removeMapping
  - _Prompt: Role: React State Expert | Task: Create Zustand store for visual builder | Restrictions: File ≤300 lines | Success: ✅ Store works_

## Phase 2: Visual Components

- [x] 4. Create VirtualKeyboard component in keyrx_ui/src/components/VirtualKeyboard.tsx
  - Render keyboard layout with draggable keys
  - Highlight mapped keys
  - _Prompt: Role: React UI Developer | Task: Create virtual keyboard with drag-and-drop | Restrictions: File ≤400 lines, use @dnd-kit, QWERTY layout, highlight mappings | Success: ✅ Keyboard renders, ✅ Drag works_

- [x] 5. Create LayerPanel component in keyrx_ui/src/components/LayerPanel.tsx
  - List layers with drag-to-reorder
  - Add/delete/rename buttons
  - _Prompt: Role: React UI Developer | Task: Create layer management panel | Restrictions: File ≤250 lines, use @dnd-kit/sortable | Success: ✅ Layers manageable_

- [x] 6. Create ModifierPanel component in keyrx_ui/src/components/ModifierPanel.tsx
  - List modifiers/locks
  - Drag key to assign
  - _Prompt: Role: React UI Developer | Task: Create modifier management panel | Restrictions: File ≤200 lines | Success: ✅ Modifiers work_

- [x] 7. Create CodePreview component in keyrx_ui/src/components/CodePreview.tsx
  - Monaco editor (read-only) showing generated Rhai
  - Copy button
  - _Prompt: Role: React Developer | Task: Create code preview with Monaco | Restrictions: File ≤200 lines, syntax highlight Rhai | Success: ✅ Code displays_

## Phase 3: Rhai Code Generation

- [x] 8. Implement Rhai generator in keyrx_ui/src/utils/rhaiGenerator.ts
  - Convert ConfigState to Rhai syntax
  - Generate layer definitions, mappings, modifiers
  - _Prompt: Role: Code Generation Expert | Task: Implement Rhai code generator | Restrictions: File ≤400 lines, generate valid Rhai syntax, format with indentation | Success: ✅ Valid Rhai generated_

- [x] 9. Integrate validator with code preview
  - Validate generated Rhai with WASM validator
  - Show errors if invalid
  - _Prompt: Role: Integration Developer | Task: Validate generated Rhai | Success: ✅ Validation works_

## Phase 4: Import/Export

- [x] 10. Implement Rhai parser in keyrx_ui/src/utils/rhaiParser.ts
  - Parse Rhai to ConfigState (basic support)
  - Warn on unsupported features
  - _Prompt: Role: Parser Developer | Task: Parse Rhai to visual state | Restrictions: File ≤400 lines, handle basic mappings only | Success: ✅ Basic parsing works_

- [x] 11. Add import/export buttons
  - Import .rhai file → parse → visualize
  - Export ConfigState → generate Rhai → download
  - _Prompt: Role: File I/O Developer | Task: Add import/export functionality | Success: ✅ Import/export works_

## Phase 5: Testing & Documentation

- [x] 12. Write unit tests for Rhai generator
  - Test simple mappings, layers, modifiers
  - _Prompt: Role: Test Engineer | Task: Test Rhai generator | Success: ✅ All cases tested_

- [x] 13. Write component tests
  - Test drag-and-drop interactions
  - _Prompt: Role: React Test Engineer | Task: Test visual builder UI | Success: ✅ Tests pass_

- [x] 14. Write accessibility tests
  - Test keyboard navigation for drag-and-drop
  - _Prompt: Role: Accessibility QA | Task: Test WCAG compliance | Success: ✅ 0 axe violations_

- [x] 15. Create documentation in docs/visual-config-builder.md
  - How to use visual builder
  - _Prompt: Role: Technical Writer | Task: Document visual builder | Success: ✅ Docs complete_

- [x] 16. Log implementation artifacts
  - Use spec-workflow log-implementation tool
  - _Prompt: Role: Documentation Engineer | Task: Log artifacts | Success: ✅ Artifacts logged_
