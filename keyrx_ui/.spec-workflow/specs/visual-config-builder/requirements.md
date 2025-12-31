# Requirements Document

## Introduction

The Visual Configuration Builder provides a drag-and-drop GUI for creating keyboard remapping configurations without writing Rhai code. Currently, all configuration requires manual Rhai scripting, which creates a barrier for non-programmers and slows down even experienced users when creating simple mappings.

By providing a visual interface with drag-and-drop key mapping, layer management, and automatic Rhai code generation, we make KeyRx accessible to users who prefer GUI over code while maintaining the power of Rhai for advanced users.

## Requirements

### Requirement 1: Visual Key Mapping

**User Story:** As a non-programmer, I want to create key mappings by dragging keys from an on-screen keyboard, so that I don't need to learn Rhai syntax.

**Acceptance Criteria:**
1. WHEN user drags a source key onto a target key THEN a mapping SHALL be created
2. WHEN a mapping is created THEN the UI SHALL highlight both source and target keys
3. WHEN user hovers over a mapped key THEN a tooltip SHALL show the mapping (e.g., "A → B")
4. WHEN user right-clicks a mapping THEN a context menu SHALL allow delete/edit
5. WHEN changes are made THEN Rhai code SHALL be generated in real-time and displayed

### Requirement 2: Layer Management

**User Story:** As a power user, I want to create and switch between layers visually, so that I can organize complex mappings.

**Acceptance Criteria:**
1. WHEN user clicks "Add Layer" THEN a new layer SHALL be created with a default name
2. WHEN user selects a layer THEN the keyboard SHALL show mappings for that layer only
3. WHEN user creates a layer-switching mapping THEN the UI SHALL show layer activation arrows
4. WHEN layers are created THEN the layer list SHALL be sortable via drag-and-drop
5. WHEN user deletes a layer THEN all mappings for that layer SHALL be removed with confirmation

### Requirement 3: Modifier and Lock Management

**User Story:** As a user creating complex shortcuts, I want to configure modifiers and locks visually, so that I can build Shift+Ctrl combinations without code.

**Acceptance Criteria:**
1. WHEN user creates a modifier THEN it SHALL appear in the modifier panel with auto-generated ID
2. WHEN user drags a key to the modifier panel THEN it SHALL activate that modifier when pressed
3. WHEN user creates a lock THEN it SHALL toggle on/off behavior visually
4. WHEN modifiers are active THEN the keyboard SHALL show which keys are affected (color overlay)
5. WHEN user tests a modifier THEN the simulator SHALL validate behavior

### Requirement 4: Real-Time Rhai Code Generation

**User Story:** As a user learning Rhai, I want to see generated code update as I make visual changes, so that I can learn syntax by example.

**Acceptance Criteria:**
1. WHEN user makes ANY change THEN Rhai code SHALL regenerate within 100ms
2. WHEN generated code is displayed THEN it SHALL be syntax-highlighted and formatted
3. WHEN user clicks "Copy Code" THEN code SHALL be copied to clipboard
4. WHEN user switches to "Code View" THEN they SHALL see full generated Rhai configuration
5. WHEN generated code has syntax errors THEN the builder SHALL highlight the problematic visual element

### Requirement 5: Import/Export

**User Story:** As a user with existing Rhai configs, I want to import them into the visual builder, so that I can edit them visually.

**Acceptance Criteria:**
1. WHEN user uploads a .rhai file THEN it SHALL be parsed and visualized (if possible)
2. WHEN Rhai code contains unsupported features THEN a warning SHALL list them
3. WHEN user exports a config THEN it SHALL download as a .rhai file
4. WHEN user saves THEN the config SHALL be applied to the daemon
5. WHEN parsing fails THEN clear error messages SHALL explain what's unsupported

## Non-Functional Requirements

- **Architecture**: React with drag-and-drop library (react-dnd or dnd-kit)
- **Performance**: Rhai generation <100ms, keyboard rendering <16ms (60fps)
- **Code Quality**: File sizes ≤300 lines (components), TypeScript strict mode
- **Accessibility**: WCAG 2.1 AA (keyboard navigation, ARIA for drag-drop)

## Dependencies

- New: @dnd-kit/core, @dnd-kit/sortable (drag-and-drop)
- New: @monaco-editor/react (code display)
- Leverage: WASM validator (validate generated Rhai)

## Sources

- [15 Drag and Drop UI Design Tips That Actually Work in 2025](https://bricxlabs.com/blogs/drag-and-drop-ui)
- [Best Practices for Drag-and-Drop Workflow UI](https://latenode.com/blog/best-practices-for-drag-and-drop-workflow-ui)
- [Drag & Drop UX Design Best Practices](https://www.pencilandpaper.io/articles/ux-pattern-drag-and-drop)
