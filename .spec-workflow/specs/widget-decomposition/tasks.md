# Tasks Document

## Phase 1: Directory Structure

- [x] 1. Create widget directory hierarchy
  - Files: `ui/lib/widgets/{common,editor,preview,settings}/`
  - Create directories for each feature area
  - Add barrel export files
  - Purpose: Foundation for widget organization
  - _Leverage: Flutter project structure patterns_
  - _Requirements: 2.1, 2.3_
  - _Prompt: Implement the task for spec widget-decomposition, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Flutter Developer setting up structure | Task: Create widget directory hierarchy with barrel exports | Restrictions: Follow Flutter conventions, add index files | _Leverage: Flutter project patterns | Success: Directory structure ready, exports work | After completion: Mark task [-] as in-progress before starting, use log-implementation tool to record artifacts, then mark [x] when complete_

- [x] 2. Analyze editor_widgets.dart for extraction
  - File: `ui/lib/pages/editor_widgets.dart`
  - Document all widgets and their dependencies
  - Create extraction plan
  - Purpose: Plan decomposition
  - _Leverage: Code analysis_
  - _Requirements: 1.1_
  - _Prompt: Implement the task for spec widget-decomposition, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Flutter Developer analyzing code | Task: Analyze editor_widgets.dart and document widgets | Restrictions: List all widgets, dependencies, line counts | _Leverage: Code analysis | Success: Complete widget inventory documented | After completion: Mark task [-] as in-progress before starting, use log-implementation tool to record artifacts, then mark [x] when complete_

## Phase 2: Common Widget Extraction

- [x] 3. Extract IconButton variants
  - File: `ui/lib/widgets/common/styled_icon_button.dart`
  - Extract custom icon button implementations
  - Add hover and press states
  - Purpose: Reusable icon buttons
  - _Leverage: Existing icon button code_
  - _Requirements: 1.1, 2.3_
  - _Prompt: Implement the task for spec widget-decomposition, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Flutter Developer extracting widgets | Task: Extract IconButton variants to common/styled_icon_button.dart | Restrictions: Support all existing use cases, add documentation | _Leverage: Existing implementations | Success: IconButton works in all contexts | After completion: Mark task [-] as in-progress before starting, use log-implementation tool to record artifacts, then mark [x] when complete_

- [ ] 4. Extract TextField variants
  - File: `ui/lib/widgets/common/styled_text_field.dart`
  - Extract custom text field implementations
  - Unify styling and validation
  - Purpose: Reusable text inputs
  - _Leverage: Existing text field code_
  - _Requirements: 1.1, 2.3_
  - _Prompt: Implement the task for spec widget-decomposition, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Flutter Developer extracting widgets | Task: Extract TextField variants to common/styled_text_field.dart | Restrictions: Preserve validation, unify styling | _Leverage: Existing implementations | Success: TextField works everywhere | After completion: Mark task [-] as in-progress before starting, use log-implementation tool to record artifacts, then mark [x] when complete_

- [ ] 5. Extract Dialog widgets
  - File: `ui/lib/widgets/common/dialogs/`
  - Extract confirmation, input, and selection dialogs
  - Create dialog builder helpers
  - Purpose: Reusable dialogs
  - _Leverage: Existing dialog code_
  - _Requirements: 1.1, 2.3_
  - _Prompt: Implement the task for spec widget-decomposition, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Flutter Developer extracting widgets | Task: Extract Dialog widgets to common/dialogs/ | Restrictions: Preserve behavior, add builders | _Leverage: Existing implementations | Success: Dialogs reusable across app | After completion: Mark task [-] as in-progress before starting, use log-implementation tool to record artifacts, then mark [x] when complete_

## Phase 3: Editor Widget Extraction

- [ ] 6. Extract KeyButton widget
  - File: `ui/lib/widgets/editor/key_button.dart`
  - Extract key button with all states
  - Add size variants
  - Purpose: Core key rendering
  - _Leverage: Existing key rendering_
  - _Requirements: 1.1, 1.4_
  - _Prompt: Implement the task for spec widget-decomposition, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Flutter Developer extracting widgets | Task: Extract KeyButton to editor/key_button.dart | Restrictions: Support all states, sizes, interactions | _Leverage: Existing key button | Success: KeyButton works in grid | After completion: Mark task [-] as in-progress before starting, use log-implementation tool to record artifacts, then mark [x] when complete_

- [ ] 7. Extract KeyGrid widget
  - File: `ui/lib/widgets/editor/key_grid.dart`
  - Extract keyboard grid layout
  - Use KeyButton for individual keys
  - Purpose: Keyboard visualization
  - _Leverage: Existing grid layout_
  - _Requirements: 1.1, 3.2_
  - _Prompt: Implement the task for spec widget-decomposition, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Flutter Developer extracting widgets | Task: Extract KeyGrid to editor/key_grid.dart | Restrictions: Use KeyButton, efficient rebuilds | _Leverage: Existing grid | Success: KeyGrid renders keyboard correctly | After completion: Mark task [-] as in-progress before starting, use log-implementation tool to record artifacts, then mark [x] when complete_

- [ ] 8. Extract KeyLegend widget
  - File: `ui/lib/widgets/editor/key_legend.dart`
  - Extract legend display
  - Support horizontal and vertical layouts
  - Purpose: Key color legend
  - _Leverage: Existing legend code_
  - _Requirements: 1.1_
  - _Prompt: Implement the task for spec widget-decomposition, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Flutter Developer extracting widgets | Task: Extract KeyLegend to editor/key_legend.dart | Restrictions: Both orientations, themeable | _Leverage: Existing legend | Success: Legend displays correctly | After completion: Mark task [-] as in-progress before starting, use log-implementation tool to record artifacts, then mark [x] when complete_

- [ ] 9. Extract LayerPanel widget
  - File: `ui/lib/widgets/editor/layer_panel.dart`
  - Extract layer selection panel
  - Include add/delete/reorder
  - Purpose: Layer management UI
  - _Leverage: Existing layer panel_
  - _Requirements: 1.1, 3.3_
  - _Prompt: Implement the task for spec widget-decomposition, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Flutter Developer extracting widgets | Task: Extract LayerPanel to editor/layer_panel.dart | Restrictions: All operations, reorderable | _Leverage: Existing panel | Success: Layer management works | After completion: Mark task [-] as in-progress before starting, use log-implementation tool to record artifacts, then mark [x] when complete_

- [ ] 10. Extract BindingPanel widget
  - File: `ui/lib/widgets/editor/binding_panel.dart`
  - Extract binding configuration form
  - Split into sub-components if large
  - Purpose: Binding editing UI
  - _Leverage: Existing binding form_
  - _Requirements: 1.1, 1.2_
  - _Prompt: Implement the task for spec widget-decomposition, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Flutter Developer extracting widgets | Task: Extract BindingPanel to editor/binding_panel.dart | Restrictions: All binding types, validation | _Leverage: Existing form | Success: Binding configuration works | After completion: Mark task [-] as in-progress before starting, use log-implementation tool to record artifacts, then mark [x] when complete_

## Phase 4: Integration and Cleanup

- [ ] 11. Update EditorPage imports
  - File: `ui/lib/pages/editor_page.dart`
  - Replace editor_widgets import with barrel import
  - Verify all widgets still accessible
  - Purpose: Use new widget structure
  - _Leverage: Barrel exports_
  - _Requirements: 1.3_
  - _Prompt: Implement the task for spec widget-decomposition, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Flutter Developer updating imports | Task: Update EditorPage to use barrel imports | Restrictions: No functionality change, verify all widgets | _Leverage: Barrel exports | Success: EditorPage works with new imports | After completion: Mark task [-] as in-progress before starting, use log-implementation tool to record artifacts, then mark [x] when complete_

- [ ] 12. Update other pages with extracted widgets
  - Files: All pages using editor widgets
  - Update imports to use common widgets
  - Remove duplicated widget code
  - Purpose: Consistent widget usage
  - _Leverage: Common widgets_
  - _Requirements: 2.3_
  - _Prompt: Implement the task for spec widget-decomposition, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Flutter Developer updating imports | Task: Update all pages to use extracted widgets | Restrictions: Use common widgets, remove duplicates | _Leverage: Common widgets | Success: All pages use shared widgets | After completion: Mark task [-] as in-progress before starting, use log-implementation tool to record artifacts, then mark [x] when complete_

- [ ] 13. Delete editor_widgets.dart
  - File: `ui/lib/pages/editor_widgets.dart`
  - Remove after all extractions complete
  - Verify no remaining references
  - Purpose: Complete migration
  - _Leverage: Extraction complete_
  - _Requirements: 1.1_
  - _Prompt: Implement the task for spec widget-decomposition, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Flutter Developer cleaning up | Task: Delete editor_widgets.dart after verification | Restrictions: No references remain, all tests pass | _Leverage: Complete extraction | Success: Old file removed, no regressions | After completion: Mark task [-] as in-progress before starting, use log-implementation tool to record artifacts, then mark [x] when complete_

## Phase 5: Testing

- [ ] 14. Add widget unit tests
  - Files: `ui/test/widgets/`
  - Create tests for each extracted widget
  - Test states and interactions
  - Purpose: Widget verification
  - _Leverage: Flutter test framework_
  - _Requirements: 4.1, 4.2_
  - _Prompt: Implement the task for spec widget-decomposition, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Flutter Test Developer | Task: Create unit tests for extracted widgets | Restrictions: Test all states, mock dependencies | _Leverage: Flutter testing | Success: High test coverage for widgets | After completion: Mark task [-] as in-progress before starting, use log-implementation tool to record artifacts, then mark [x] when complete_

- [ ] 15. Add golden tests for visual widgets
  - Files: `ui/test/widgets/golden/`
  - Create screenshot tests for key widgets
  - Test different themes and sizes
  - Purpose: Visual regression testing
  - _Leverage: Flutter golden testing_
  - _Requirements: 4.4_
  - _Prompt: Implement the task for spec widget-decomposition, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Flutter Test Developer | Task: Create golden tests for visual widgets | Restrictions: Test themes, sizes, states | _Leverage: Golden testing | Success: Visual regressions caught | After completion: Mark task [-] as in-progress before starting, use log-implementation tool to record artifacts, then mark [x] when complete_

- [ ] 16. Add integration tests for widget interactions
  - Files: `ui/test/widgets/integration/`
  - Test widget composition and callbacks
  - Verify state flows correctly
  - Purpose: Integration verification
  - _Leverage: Flutter integration testing_
  - _Requirements: 4.3_
  - _Prompt: Implement the task for spec widget-decomposition, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Flutter Test Developer | Task: Create integration tests for widget interactions | Restrictions: Test composition, callbacks, state | _Leverage: Flutter testing | Success: Widget interactions verified | After completion: Mark task [-] as in-progress before starting, use log-implementation tool to record artifacts, then mark [x] when complete_
