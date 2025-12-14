# KeyFlow Implementation - Tasks

- [x] 1. Repository Implementation
  - File: ui/lib/repositories/keyflow_repository.dart, ui/test/repositories/keyflow_repository_test.dart
  - Implement KeyFlowRepository with FFI integration: getRuntimeConfig() with 5min TTL caching, updateSlot(), addSlot() with auto-priority, removeSlot(), watchRuntimeConfig() stream for real-time updates, structured logging, comprehensive unit tests (80%+ coverage)
  - Purpose: Provide data layer for fetching and mutating RuntimeConfig via FFI bridge
  - _Leverage: existing ui/lib/ffi/bridge.dart, Rust RuntimeService_
  - _Requirements: FR1.1, FR1.2, FR1.3, NFR1.1, NFR2.1_
  - _Prompt: Implement the task for spec keyflow-implementation, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Flutter/Dart Developer with expertise in repository pattern and FFI integration | Task: Create ui/lib/repositories/keyflow_repository.dart implementing getRuntimeConfig() with 5min TTL caching and forceRefresh option, updateSlot() for mutations, addSlot() with auto-priority calculation, removeSlot() with confirmation, watchRuntimeConfig() stream for real-time updates, add structured logging to all methods using StructuredLogger. Create comprehensive unit tests in test/repositories/keyflow_repository_test.dart covering caching behavior, cache invalidation, error handling, stream subscriptions, achieve 80%+ coverage | Restrictions: Must use existing FFI bridge interface, cache should be thread-safe, ensure proper error handling for FFI failures, do not bypass validation | Success: Repository implements all CRUD operations, caching works correctly with TTL, stream provides real-time updates, 80%+ test coverage, all tests pass | After completion: (1) Edit tasks.md to change [-] to [x], (2) Use log-implementation tool with detailed artifacts (classes, functions, integrations)_

- [ ] 2. Data Models & Transformations
  - File: ui/lib/models/flow/flow_graph.dart, ui/test/models/flow/flow_graph_test.dart
  - Create FlowGraph, FlowNode, FlowEdge freezed models with fromRuntimeConfig() factory, validation logic, status enums, comprehensive unit tests for model transformations
  - Purpose: Transform RuntimeConfig data into flow graph representation for visualization
  - _Leverage: existing ui/lib/models/flow/flow_data.dart, ui/lib/models/runtime_config.dart_
  - _Requirements: FR3.1, FR3.2, NFR1.1_
  - _Prompt: Implement the task for spec keyflow-implementation, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Flutter Developer with expertise in data modeling and freezed | Task: Create ui/lib/models/flow/flow_graph.dart, define FlowGraph freezed class with nodes, edges, positions, define FlowNode union type (device, hardwareProfile, keymap), define FlowEdge freezed class (from, to, slot, priority, active, status), add EdgeStatus enum (valid, invalid, warning), implement FlowGraph.fromRuntimeConfig() factory method to transform RuntimeConfig into graph structure, write unit tests for all model transformations in test/models/flow/flow_graph_test.dart | Restrictions: Must use freezed for immutability, ensure type safety, validate all transformations, do not duplicate existing models | Success: All models defined with freezed, fromRuntimeConfig() correctly transforms data, EdgeStatus properly computed, unit tests pass with good coverage | After completion: (1) Edit tasks.md to change [-] to [x], (2) Use log-implementation tool with detailed artifacts_

- [ ] 3. Riverpod State Providers
  - File: ui/lib/state/keyflow_providers.dart, ui/test/state/keyflow_providers_test.dart
  - Create Riverpod providers: keyFlowRepositoryProvider, runtimeConfigProvider (async), flowGraphProvider (derived), nodePositionsProvider (persistent SharedPreferences), keyFlowActionsProvider for mutations, run build_runner, write provider tests with mocks
  - Purpose: Manage KeyFlow state with Riverpod, enable reactive updates across UI
  - _Leverage: existing Riverpod setup from ultra-speed spec, ui/lib/state/providers_setup.dart_
  - _Requirements: FR1.1, FR1.2, NFR2.1_
  - _Prompt: Implement the task for spec keyflow-implementation, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Flutter Developer with expertise in Riverpod and state management | Task: Create ui/lib/state/keyflow_providers.dart, implement keyFlowRepositoryProvider, runtimeConfigProvider (AsyncNotifierProvider fetching from repository), flowGraphProvider (derived from runtimeConfigProvider using FlowGraph.fromRuntimeConfig()), nodePositionsProvider (persistent using SharedPreferences for canvas positions), keyFlowActionsProvider for mutations (updateSlot, addSlot, removeSlot), run build_runner to generate code, write provider tests with mocked repository in test/state/keyflow_providers_test.dart | Restrictions: Must follow Riverpod best practices, ensure proper dependency injection, handle loading/error states, keep providers focused | Success: All providers implemented with code generation, state updates trigger UI rebuilds, node positions persist across sessions, mutations work correctly, provider tests pass | After completion: (1) Edit tasks.md to change [-] to [x], (2) Use log-implementation tool with detailed artifacts_

- [ ] 4. FlowEditorPage Refactoring
  - File: ui/lib/pages/flow_editor_page.dart, ui/test/pages/flow_editor_page_test.dart
  - Refactor FlowEditorPage from StatefulWidget to ConsumerStatefulWidget, replace hardcoded mock data with ref.watch(flowGraphProvider), add loading state (skeleton with shimmer), error state (ErrorView with retry), empty state ("No devices connected"), widget tests
  - Purpose: Connect UI to real data layer, handle async states properly
  - _Leverage: existing ui/lib/pages/flow_editor_page.dart prototype, Riverpod providers from task 3_
  - _Requirements: FR1.1, NFR3.2_
  - _Prompt: Implement the task for spec keyflow-implementation, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Flutter Developer with expertise in Riverpod and async UI patterns | Task: Refactor ui/lib/pages/flow_editor_page.dart to ConsumerStatefulWidget, replace hardcoded mock data with ref.watch(flowGraphProvider), implement when() pattern for AsyncValue handling (loading: skeleton with shimmer, error: ErrorView with retry button, data: render flow), add empty state when no devices connected, ensure InteractiveViewer still works, write widget tests covering all states in test/pages/flow_editor_page_test.dart | Restrictions: Must preserve existing UI layout, ensure smooth loading experience, do not break InteractiveViewer functionality, handle all error cases | Success: Page fetches real data from providers, loading state shows skeleton, error state has retry, empty state displayed when appropriate, all widget tests pass | After completion: (1) Edit tasks.md to change [-] to [x], (2) Use log-implementation tool with detailed artifacts_

- [ ] 5. Node Position Management
  - File: ui/lib/state/node_positions_provider.dart, ui/lib/models/flow/node_position.dart
  - Implement persistent node position storage using SharedPreferences, auto-layout algorithm for initial placement, save/restore positions across sessions
  - Purpose: Remember user's custom node arrangements, provide sensible default layout
  - _Leverage: nodePositionsProvider from task 3, shared_preferences package_
  - _Requirements: FR2.1_
  - _Prompt: Implement the task for spec keyflow-implementation, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Flutter Developer with expertise in state persistence and layout algorithms | Task: Implement ui/lib/state/node_positions_provider.dart using SharedPreferences for persistent storage, create ui/lib/models/flow/node_position.dart model, implement auto-layout algorithm for initial node placement (devices left, profiles middle, keymaps right with vertical spacing), save positions when user drags nodes, restore positions on page load, provide reset-to-default functionality | Restrictions: Positions must persist across app restarts, auto-layout should handle variable number of nodes, ensure efficient storage (don't save unchanged positions), gracefully handle missing/corrupted data | Success: Node positions persist across sessions, auto-layout provides sensible defaults, position updates are efficient, reset functionality works | After completion: (1) Edit tasks.md to change [-] to [x], (2) Use log-implementation tool with detailed artifacts_

- [ ] 6. Enhanced Edge Rendering
  - File: ui/lib/widgets/flow/edges_painter.dart, ui/lib/models/flow/edge_style.dart
  - Enhance EdgesPainter with visual encoding: solid lines for active slots, dashed for inactive, different colors for validation states (green=valid, red=invalid, yellow=warning), priority badges, hover effects
  - Purpose: Provide clear visual feedback about slot status and validation
  - _Leverage: existing ui/lib/widgets/flow/edges_painter.dart prototype_
  - _Requirements: FR3.1, FR3.2, FR3.3_
  - _Prompt: Implement the task for spec keyflow-implementation, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Flutter Developer with expertise in CustomPainter and canvas rendering | Task: Enhance ui/lib/widgets/flow/edges_painter.dart, create ui/lib/models/flow/edge_style.dart for styling configuration, implement solid lines for active slots (active: true), dashed lines for inactive slots, color coding based on validation (green=valid, red=invalid, yellow=warning), draw priority badges on edges showing priority number, add hover effect with thicker lines, optimize rendering for performance (use Path caching) | Restrictions: Must maintain 60fps during interactions, edges should follow Bezier curve path, ensure colors meet WCAG contrast requirements, do not sacrifice performance for visual effects | Success: Active/inactive slots visually distinguished, validation states clearly indicated, priority badges visible, hover effects smooth, rendering performs at 60fps | After completion: (1) Edit tasks.md to change [-] to [x], (2) Use log-implementation tool with detailed artifacts_

- [ ] 7. Static Widget Tests
  - File: ui/test/widgets/flow/device_node_widget_test.dart, ui/test/widgets/flow/layout_node_widget_test.dart, ui/test/widgets/flow/keymap_node_widget_test.dart
  - Write comprehensive widget tests for DeviceNodeWidget, LayoutNodeWidget, KeymapNodeWidget covering various states (connected/disconnected, valid/invalid, active/inactive)
  - Purpose: Ensure node widgets render correctly in all states
  - _Leverage: widget test harness from ultra-speed spec, existing node widgets_
  - _Requirements: NFR1.1_
  - _Prompt: Implement the task for spec keyflow-implementation, first run spec-workflow-guide to get the workflow guide then implement the task: Role: QA Engineer with expertise in Flutter widget testing | Task: Create comprehensive widget tests for all node widgets: test/widgets/flow/device_node_widget_test.dart (test connected/disconnected states, active/inactive slots, error states), test/widgets/flow/layout_node_widget_test.dart (test valid/invalid profiles), test/widgets/flow/keymap_node_widget_test.dart (test keymap loaded/compilation failed states), use widget test harness for provider mocking, verify text content, colors, icons, tooltips, achieve 80%+ coverage | Restrictions: Tests must be isolated, use mocked data, do not test CustomPainter internals, ensure fast test execution | Success: All node widgets tested thoroughly, all states covered, tests pass reliably, 80%+ coverage achieved | After completion: (1) Edit tasks.md to change [-] to [x], (2) Use log-implementation tool with detailed artifacts_

- [ ] 8. Node Dragging Implementation
  - File: ui/lib/widgets/flow/draggable_node.dart, ui/lib/state/drag_controller.dart
  - Implement draggable node widget with smooth dragging, position updates to nodePositionsProvider, constraints (keep within canvas bounds), visual feedback during drag
  - Purpose: Enable users to rearrange nodes for better visualization
  - _Leverage: existing InteractiveViewer in FlowEditorPage, nodePositionsProvider_
  - _Requirements: FR2.1, FR5.1_
  - _Prompt: Implement the task for spec keyflow-implementation, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Flutter Developer with expertise in gesture handling and animations | Task: Create ui/lib/widgets/flow/draggable_node.dart wrapping node widgets with GestureDetector, implement ui/lib/state/drag_controller.dart for drag state management, handle onPanStart/onPanUpdate/onPanEnd gestures, update node positions in nodePositionsProvider on drag end, add constraints to keep nodes within canvas bounds, provide visual feedback during drag (elevation, shadow), ensure smooth 60fps dragging performance | Restrictions: Dragging must work inside InteractiveViewer, ensure no gesture conflicts with pan/zoom, maintain 60fps performance, positions must update correctly with zoom/pan transforms | Success: Nodes can be dragged smoothly, positions persist after drag, constraints prevent nodes from going off-canvas, visual feedback clear, performance at 60fps | After completion: (1) Edit tasks.md to change [-] to [x], (2) Use log-implementation tool with detailed artifacts_

- [ ] 9. Edge Interaction & Selection
  - File: ui/lib/widgets/flow/interactive_edge.dart, ui/lib/state/selection_state.dart
  - Implement edge click detection, selection highlighting, hover effects, context menu for edge actions (delete, toggle active), keyboard support (Delete key)
  - Purpose: Enable users to interact with slot connections
  - _Leverage: enhanced EdgesPainter from task 6_
  - _Requirements: FR2.4, FR2.5, FR4.1_
  - _Prompt: Implement the task for spec keyflow-implementation, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Flutter Developer with expertise in CustomPainter hit detection and interaction | Task: Create ui/lib/widgets/flow/interactive_edge.dart with hit testing for Bezier curve paths, implement ui/lib/state/selection_state.dart for tracking selected edges, add click detection on edges with tolerance margin, implement selection highlighting (thicker border, different color), add hover effects, create context menu for edge actions (delete slot, toggle active/inactive), support Delete key for selected edges, integrate with keyFlowActionsProvider for mutations | Restrictions: Hit detection must be accurate for curved paths, ensure performance with many edges, context menu should be accessible, keyboard shortcuts must not conflict | Success: Edges can be clicked and selected, selection visually clear, hover effects work, context menu functional, keyboard shortcuts work, mutations trigger correctly | After completion: (1) Edit tasks.md to change [-] to [x], (2) Use log-implementation tool with detailed artifacts_

- [ ] 10. Slot Management (Add/Delete/Reorder)
  - File: ui/lib/dialogs/add_slot_dialog.dart, ui/lib/widgets/flow/slot_priority_adjuster.dart, ui/lib/state/slot_actions.dart
  - Implement Add Slot dialog (select profile + keymap), delete slot with confirmation, reorder slots by dragging priority badges, auto-save after mutations
  - Purpose: Enable full CRUD operations on device slots
  - _Leverage: keyFlowActionsProvider from task 3, existing profile/keymap data_
  - _Requirements: FR2.2, FR2.3, FR2.4, FR1.3_
  - _Prompt: Implement the task for spec keyflow-implementation, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Flutter Developer with expertise in dialogs and CRUD operations | Task: Create ui/lib/dialogs/add_slot_dialog.dart with dropdown selectors for hardware profile and keymap, implement add button on device nodes, create ui/lib/widgets/flow/slot_priority_adjuster.dart for dragging priority badges to reorder slots, implement ui/lib/state/slot_actions.dart wrapping keyFlowActionsProvider mutations, add confirmation dialog for delete operations, auto-save changes to RuntimeConfig via repository, show success/error toasts, validate inputs (no duplicate profile+keymap combos) | Restrictions: Dialog must validate selections, prevent invalid combinations, ensure atomic operations (rollback on failure), provide clear error messages, maintain undo capability | Success: Add slot dialog functional with validation, delete works with confirmation, priority reordering smooth, auto-save works, error handling robust | After completion: (1) Edit tasks.md to change [-] to [x], (2) Use log-implementation tool with detailed artifacts_

- [ ] 11. Interactive Widget Tests
  - File: ui/test/widgets/flow/draggable_node_test.dart, ui/test/widgets/flow/interactive_edge_test.dart, ui/test/dialogs/add_slot_dialog_test.dart
  - Write widget tests for drag interactions, edge selection, slot management dialogs covering user interactions and state updates
  - Purpose: Ensure interactive features work correctly
  - _Leverage: widget test harness, interactive components from tasks 8-10_
  - _Requirements: NFR1.1_
  - _Prompt: Implement the task for spec keyflow-implementation, first run spec-workflow-guide to get the workflow guide then implement the task: Role: QA Engineer with expertise in interaction testing | Task: Create widget tests for all interactive features: test/widgets/flow/draggable_node_test.dart (test drag gestures, position updates, constraints), test/widgets/flow/interactive_edge_test.dart (test click detection, selection, hover, context menu, keyboard shortcuts), test/dialogs/add_slot_dialog_test.dart (test form validation, profile/keymap selection, submission, cancellation), mock provider interactions, verify state updates, test error handling, achieve 80%+ coverage | Restrictions: Tests must simulate real user interactions, use WidgetTester.drag/tap methods, mock FFI calls, ensure tests are deterministic | Success: All interactive features tested, gesture handling verified, state updates confirmed, error cases covered, 80%+ coverage | After completion: (1) Edit tasks.md to change [-] to [x], (2) Use log-implementation tool with detailed artifacts_

- [ ] 12. Real-Time Device Updates
  - File: ui/lib/state/device_stream_provider.dart, ui/lib/services/device_event_handler.dart
  - Implement device state streaming from FFI (BridgeStateUpdate), update flow graph when devices connect/disconnect, animate node appearance/disappearance, update validation states in real-time
  - Purpose: Keep UI synchronized with device state changes without manual refresh
  - _Leverage: existing BridgeStateUpdate stream from FFI, runtimeConfigProvider_
  - _Requirements: FR1.2, US5_
  - _Prompt: Implement the task for spec keyflow-implementation, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Flutter Developer with expertise in reactive streams and real-time updates | Task: Create ui/lib/state/device_stream_provider.dart subscribing to BridgeStateUpdate stream from FFI bridge, implement ui/lib/services/device_event_handler.dart to process device connect/disconnect events, update runtimeConfigProvider when devices change, trigger flowGraphProvider recomputation, add fade-in animation for new devices, fade-out for disconnected devices, update node status colors in real-time, invalidate repository cache on changes, ensure <100ms latency from event to UI update | Restrictions: Stream must be properly disposed, handle backpressure, ensure no memory leaks, updates should be efficient (don't rebuild entire tree), maintain state consistency | Success: Devices appear/disappear automatically, animations smooth, status updates instant, <100ms latency, no memory leaks, stream properly managed | After completion: (1) Edit tasks.md to change [-] to [x], (2) Use log-implementation tool with detailed artifacts_

- [ ] 13. Auto-Save & Persistence
  - File: ui/lib/services/auto_save_service.dart, ui/lib/state/save_state_provider.dart
  - Implement auto-save with debouncing (500ms after last change), optimistic UI updates, rollback on failure, save state indicator (saving/saved/error), retry logic
  - Purpose: Persist changes automatically without manual save button
  - _Leverage: keyFlowRepository from task 1, keyFlowActionsProvider_
  - _Requirements: FR1.3, FR5.2_
  - _Prompt: Implement the task for spec keyflow-implementation, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Flutter Developer with expertise in state synchronization and debouncing | Task: Create ui/lib/services/auto_save_service.dart with 500ms debounced save after mutations, implement ui/lib/state/save_state_provider.dart tracking save status (idle/saving/saved/error), use optimistic UI updates (apply changes immediately, rollback on FFI failure), add inline save indicator in UI (spinner/checkmark/error icon), implement exponential backoff retry on failure (max 3 retries), show toast on persistent errors, ensure <200ms from user action to save confirmation, log all save operations | Restrictions: Must prevent save storms (debounce properly), handle concurrent mutations correctly, ensure data consistency, provide clear feedback to user, don't block UI during save | Success: Changes auto-save after 500ms, optimistic updates work, rollback on failure, save indicator clear, retry logic functional, <200ms save confirmation, no data loss | After completion: (1) Edit tasks.md to change [-] to [x], (2) Use log-implementation tool with detailed artifacts_

- [ ] 14. Node Visual Enhancements
  - File: ui/lib/widgets/flow/enhanced_device_node.dart, ui/lib/widgets/flow/enhanced_layout_node.dart, ui/lib/widgets/flow/enhanced_keymap_node.dart
  - Enhance node widgets with status colors (green/blue/red/grey), status icons, tooltips with detailed info, connection handles, visual hierarchy, shadows/elevation
  - Purpose: Improve visual feedback and information density
  - _Leverage: existing node widgets, design system from design.md_
  - _Requirements: FR3.1, FR3.4, NFR3.1_
  - _Prompt: Implement the task for spec keyflow-implementation, first run spec-workflow-guide to get the workflow guide then implement the task: Role: UI/UX Developer with expertise in Material Design and visual hierarchy | Task: Enhance all node widgets with status-based colors (device: green=connected active, blue=connected inactive, red=error, grey=disconnected; profile: blue=valid, red=invalid; keymap: green=valid, red=compilation failed), add status icons (checkmark, warning, error), implement rich tooltips with detailed information (device: VID/PID/serial, profile: layout type, keymap: layer count), add visible connection handles on node edges, improve visual hierarchy with proper typography and spacing, add subtle shadows/elevation, ensure Material Design 3 compliance | Restrictions: Must follow design system from design.md, ensure WCAG AA color contrast, keep nodes compact but readable, icons must be semantic, tooltips should not interfere with interactions | Success: Status colors implemented correctly, icons semantic and clear, tooltips informative, connection handles visible, visual hierarchy clear, Material Design 3 compliant, WCAG AA contrast met | After completion: (1) Edit tasks.md to change [-] to [x], (2) Use log-implementation tool with detailed artifacts_

- [ ] 15. Edge Visual Polish & Animations
  - File: ui/lib/widgets/flow/animated_edges_painter.dart, ui/lib/animations/edge_animations.dart
  - Add edge animations (pulse on active slot, fade in/out on create/delete), smooth transitions for validation state changes, improved Bezier curves, anti-aliasing, performance optimization
  - Purpose: Polish edge rendering with smooth animations and transitions
  - _Leverage: enhanced EdgesPainter from task 6_
  - _Requirements: FR3.2, FR5.1_
  - _Prompt: Implement the task for spec keyflow-implementation, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Flutter Developer with expertise in CustomPainter animations and performance | Task: Create ui/lib/widgets/flow/animated_edges_painter.dart extending enhanced EdgesPainter, implement ui/lib/animations/edge_animations.dart with subtle pulse animation for active slots, fade in/out animations for edge create/delete, smooth color transitions for validation state changes, optimize Bezier curve calculations, add anti-aliasing for smooth curves, implement Path caching to avoid recalculation, use RepaintBoundary for optimization, ensure 60fps during all animations | Restrictions: Animations must be subtle (not distracting), maintain 60fps performance even with 50+ edges, use efficient painting techniques, avoid unnecessary repaints, respect reduced motion preferences | Success: Animations smooth and subtle, validation transitions clear, Bezier curves smooth with anti-aliasing, performance at 60fps with many edges, reduced motion respected | After completion: (1) Edit tasks.md to change [-] to [x], (2) Use log-implementation tool with detailed artifacts_

- [ ] 16. Golden Tests
  - File: ui/test/golden/flow_nodes_golden_test.dart, ui/test/golden/flow_edges_golden_test.dart, ui/test/golden/flow_page_golden_test.dart
  - Create golden tests for all node widgets in various states, edge rendering in different styles, full page layout in different scenarios (empty, error, populated)
  - Purpose: Prevent visual regressions in UI components
  - _Leverage: golden test harness from ultra-speed spec, enhanced node widgets_
  - _Requirements: NFR1.1_
  - _Prompt: Implement the task for spec keyflow-implementation, first run spec-workflow-guide to get the workflow guide then implement the task: Role: QA Engineer with expertise in Flutter golden testing | Task: Create comprehensive golden tests: test/golden/flow_nodes_golden_test.dart (capture all node widgets in each state: connected/disconnected, valid/invalid, active/inactive, with/without errors), test/golden/flow_edges_golden_test.dart (capture edges with different styles: active/inactive, validation states, priority badges, hover), test/golden/flow_page_golden_test.dart (capture full page in different scenarios: empty state, error state, loading state, populated with various configurations), use golden_toolkit for multi-device testing, ensure deterministic rendering (fixed fonts, no animations), update goldens with just ui-golden-update | Restrictions: Golden tests must be deterministic, disable animations for captures, use consistent test data, ensure cross-platform compatibility, keep golden file sizes reasonable | Success: All visual states captured in goldens, tests pass on CI, goldens detect regressions, multi-device coverage, golden update workflow documented | After completion: (1) Edit tasks.md to change [-] to [x], (2) Use log-implementation tool with detailed artifacts_

- [ ] 17. Keyboard Navigation
  - File: ui/lib/widgets/flow/keyboard_navigable_canvas.dart, ui/lib/services/keyboard_shortcuts.dart
  - Implement keyboard navigation: Tab to navigate nodes, Arrow keys to move focused node, Enter for details, Delete for selected edge, Escape to deselect, Ctrl+0 to reset zoom, Ctrl+Plus/Minus for zoom
  - Purpose: Enable full keyboard-only navigation and control
  - _Leverage: existing InteractiveViewer, selection state_
  - _Requirements: FR4.1, FR4.2, NFR3.3_
  - _Prompt: Implement the task for spec keyflow-implementation, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Flutter Developer with expertise in accessibility and keyboard navigation | Task: Create ui/lib/widgets/flow/keyboard_navigable_canvas.dart wrapping flow canvas with FocusableActionDetector, implement ui/lib/services/keyboard_shortcuts.dart defining all shortcuts (Tab: cycle through nodes, Arrow keys: move focused node 10px, Enter: open node details dialog, Delete: remove selected edge, Escape: clear selection, Ctrl+0: reset zoom to 100%, Ctrl+Plus/Minus: zoom in/out by 10%), add visible focus indicators on nodes and edges, implement focus traversal order (devices → profiles → keymaps), ensure shortcuts don't conflict with browser/OS, provide shortcut help dialog (Ctrl+?) | Restrictions: Must work without mouse, focus indicators must meet WCAG visibility requirements, shortcuts should follow platform conventions, ensure no keyboard traps, provide escape routes | Success: All features accessible via keyboard, Tab order logical, Arrow keys move nodes, shortcuts work correctly, focus indicators visible, shortcut help available, WCAG compliance met | After completion: (1) Edit tasks.md to change [-] to [x], (2) Use log-implementation tool with detailed artifacts_

- [ ] 18. Accessibility (Screen Reader, Touch, Contrast)
  - File: ui/lib/widgets/flow/accessible_flow_canvas.dart, ui/lib/services/screen_reader_announcer.dart
  - Implement screen reader support with semantic labels and announcements, touch accessibility with larger hit targets, ensure WCAG 2.1 AA color contrast, add focus indicators
  - Purpose: Ensure KeyFlow is accessible to all users
  - _Leverage: enhanced node widgets, keyboard navigation from task 17_
  - _Requirements: FR4.3, NFR3.3_
  - _Prompt: Implement the task for spec keyflow-implementation, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Accessibility Specialist with Flutter expertise | Task: Create ui/lib/widgets/flow/accessible_flow_canvas.dart wrapping canvas with Semantics widgets, implement ui/lib/services/screen_reader_announcer.dart for state change announcements, add semantic labels to all nodes (e.g., "Keychron Q1 keyboard, connected, 2 active slots"), announce drag operations ("Node moved to new position"), add semantic labels to edges ("Active slot connecting Keychron Q1 to Ergo-60 profile"), increase touch target sizes to 48x48px minimum, verify all colors meet WCAG AA contrast (4.5:1 for text, 3:1 for UI), add clear focus indicators (2px outline, high contrast color), test with TalkBack/VoiceOver | Restrictions: Must not break existing functionality, semantic labels should be concise but descriptive, announcements should not be excessive, touch targets must not overlap, color contrast must meet WCAG AA | Success: Screen reader can navigate entire flow, all elements have meaningful labels, state changes announced appropriately, touch targets at least 48px, all color contrast meets WCAG AA, focus indicators clear, tested with real assistive tech | After completion: (1) Edit tasks.md to change [-] to [x], (2) Use log-implementation tool with detailed artifacts_

- [ ] 19. Rendering Performance Optimization
  - File: ui/lib/widgets/flow/optimized_flow_canvas.dart, ui/lib/utils/rendering_optimizer.dart
  - Optimize rendering for 60fps: use RepaintBoundary, implement custom RenderObject for edges, virtualize off-screen nodes, optimize CustomPainter, reduce widget rebuilds
  - Purpose: Ensure smooth 60fps performance with 50+ devices
  - _Leverage: existing canvas widgets, enhanced EdgesPainter_
  - _Requirements: FR5.1, NFR3.1_
  - _Prompt: Implement the task for spec keyflow-implementation, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Flutter Performance Engineer with expertise in rendering optimization | Task: Create ui/lib/widgets/flow/optimized_flow_canvas.dart, implement ui/lib/utils/rendering_optimizer.dart with optimization utilities, wrap each node in RepaintBoundary to isolate repaints, implement custom RenderObject for edge rendering to avoid CustomPainter overhead, virtualize nodes outside viewport (only render visible nodes + margin), optimize EdgesPainter with shouldRepaint logic, use const widgets where possible, reduce widget rebuilds with selective ref.watch, profile with Flutter DevTools Timeline to verify 60fps, benchmark with 50 devices × 5 slots (250 nodes), optimize Path calculations with caching | Restrictions: Must maintain 60fps during pan/zoom/drag operations, optimize without breaking functionality, ensure smooth animations, profile before/after to measure improvements, target <16ms frame time | Success: 60fps maintained with 250+ nodes, pan/zoom smooth, drag operations fluid, RepaintBoundary reducing repaints, custom RenderObject faster than CustomPainter, virtualization working, profiling confirms <16ms frames | After completion: (1) Edit tasks.md to change [-] to [x], (2) Use log-implementation tool with detailed artifacts_

- [ ] 20. Memory & Performance Benchmarks
  - File: ui/test/performance/keyflow_performance_test.dart, ui/test/performance/memory_profile_test.dart
  - Create performance benchmarks for rendering, interaction, data fetching, memory profiling, set regression thresholds, add to CI
  - Purpose: Prevent performance regressions and ensure memory efficiency
  - _Leverage: Flutter DevTools from ultra-speed spec, optimized canvas_
  - _Requirements: FR5.1, FR5.2, FR5.3_
  - _Prompt: Implement the task for spec keyflow-implementation, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Performance Engineer with expertise in Flutter benchmarking | Task: Create ui/test/performance/keyflow_performance_test.dart with benchmarks: widget rendering time (target <16ms per frame with 250 nodes), interaction latency (drag/zoom/pan <100ms), data fetching (initial load <500ms, updates <100ms), create ui/test/performance/memory_profile_test.dart testing memory usage (baseline <50MB, max <100MB with 50 devices), memory leak detection (check for leaking streams/controllers), set regression thresholds (fail if >10% slower or >20% more memory), document benchmarking in docs/PROFILING.md, optionally add to CI for automated regression detection | Restrictions: Benchmarks must be reproducible, use realistic test data, avoid flaky tests, thresholds should be achievable but strict, profile in release mode | Success: All performance benchmarks created, rendering <16ms per frame, interactions <100ms latency, initial load <500ms, memory <100MB max, no leaks detected, regression thresholds set, benchmarking documented | After completion: (1) Edit tasks.md to change [-] to [x], (2) Use log-implementation tool with detailed artifacts_

- [ ] 21. Error & Loading States
  - File: ui/lib/widgets/flow/error_view.dart, ui/lib/widgets/flow/loading_skeleton.dart, ui/lib/widgets/flow/save_indicator.dart
  - Implement polished error states (FFI errors, validation errors), loading states (skeleton with shimmer, inline spinners), save indicator (saving/saved/error with icons)
  - Purpose: Provide clear feedback during async operations and errors
  - _Leverage: existing error handling patterns, Material Design loading indicators_
  - _Requirements: NFR3.2_
  - _Prompt: Implement the task for spec keyflow-implementation, first run spec-workflow-guide to get the workflow guide then implement the task: Role: UI/UX Developer with expertise in error handling and loading states | Task: Create ui/lib/widgets/flow/error_view.dart with user-friendly error messages for different error types (FFI connection failed, runtime config not found, validation errors), add retry button, error icon, actionable error text, create ui/lib/widgets/flow/loading_skeleton.dart with shimmer effect mimicking flow layout (grey boxes for nodes, lines for edges, animated shimmer overlay), create ui/lib/widgets/flow/save_indicator.dart showing save status (idle: hidden, saving: spinner + "Saving...", saved: checkmark + "Saved", error: error icon + "Failed to save" with retry), add inline loading spinners for specific operations (deleting slot, adding slot), ensure all loading states are cancelable where appropriate | Restrictions: Error messages must be user-friendly (avoid technical jargon), provide actionable suggestions, loading states should not block entire UI unnecessarily, spinners should have semantic labels for screen readers, ensure smooth transitions between states | Success: Error states show clear messages with retry options, loading skeleton provides visual continuity, save indicator always visible and accurate, inline spinners for operations, all states accessible, smooth transitions | After completion: (1) Edit tasks.md to change [-] to [x], (2) Use log-implementation tool with detailed artifacts_

- [ ] 22. Validation Feedback & Empty States
  - File: ui/lib/widgets/flow/validation_overlay.dart, ui/lib/widgets/flow/empty_state.dart, ui/lib/services/flow_validator.dart
  - Implement validation overlay showing errors on nodes/edges, empty state with helpful onboarding ("Connect a device to get started"), validation service with clear error messages
  - Purpose: Guide users to fix configuration issues and understand empty states
  - _Leverage: validation logic from FlowGraph model, error handling patterns_
  - _Requirements: FR3.4, NFR3.2_
  - _Prompt: Implement the task for spec keyflow-implementation, first run spec-workflow-guide to get the workflow guide then implement the task: Role: UI/UX Developer with expertise in form validation and user guidance | Task: Create ui/lib/services/flow_validator.dart validating entire flow graph (check for missing profiles, invalid keymaps, orphaned slots, priority conflicts), implement ui/lib/widgets/flow/validation_overlay.dart displaying validation errors as badges on nodes (red exclamation icon, tooltip with error message and suggested fix), highlight invalid edges in red with error tooltip, create ui/lib/widgets/flow/empty_state.dart for no devices scenario (friendly illustration, "Connect a device to get started" message, link to device connection guide), empty state for disconnected devices ("No connected devices, waiting for connection..."), add inline validation for dialogs (prevent saving invalid slot configurations) | Restrictions: Validation should be real-time but efficient (debounce if needed), error messages must be actionable ("Hardware profile 'ergo-60' not found. Create it in Profiles page."), empty states should be helpful not discouraging, don't overwhelm user with too many errors at once | Success: Validation runs automatically on graph changes, errors displayed clearly with tooltips, suggested fixes provided, empty states helpful and guide user to action, inline validation prevents invalid submissions, user can navigate to fix issues | After completion: (1) Edit tasks.md to change [-] to [x], (2) Use log-implementation tool with detailed artifacts_

- [ ] 23. User Documentation & Onboarding
  - File: docs/USER_GUIDE_KEYFLOW.md, ui/lib/dialogs/keyflow_onboarding_dialog.dart, ui/lib/widgets/flow/feature_tour.dart
  - Create user documentation for KeyFlow page, onboarding dialog explaining concepts (devices, slots, priorities), optional feature tour with highlights
  - Purpose: Help users understand and use KeyFlow effectively
  - _Leverage: existing documentation structure, Material Design dialogs_
  - _Requirements: US1-US5 acceptance criteria, NFR3.2_
  - _Prompt: Implement the task for spec keyflow-implementation, first run spec-workflow-guide to get the workflow guide then implement the task: Role: Technical Writer and UI Developer | Task: Create docs/USER_GUIDE_KEYFLOW.md documenting KeyFlow features (what is KeyFlow, understanding the visualization, node types, edge meanings, how to add/delete/reorder slots, keyboard shortcuts, troubleshooting common errors), include screenshots and examples, create ui/lib/dialogs/keyflow_onboarding_dialog.dart shown on first visit explaining key concepts (devices connect to profiles via slots, active vs fallback slots, priority order, drag-drop to reorder), add "Don't show again" checkbox, optionally create ui/lib/widgets/flow/feature_tour.dart with step-by-step highlights (spotlight on device node, edge, add button, etc.), ensure onboarding is skippable and accessible | Restrictions: Documentation must be beginner-friendly, use clear language, include visual examples, onboarding should be concise (3-5 key points max), don't block user from using app, tour should be optional and easily dismissable | Success: User guide complete with screenshots, onboarding dialog explains core concepts clearly, users can skip/dismiss onboarding, feature tour enhances understanding (if implemented), documentation covers all features | After completion: (1) Edit tasks.md to change [-] to [x], (2) Use log-implementation tool with detailed artifacts_

- [ ] 24. Metrics, Observability & Feature Flags
  - File: ui/lib/services/keyflow_analytics.dart, ui/lib/config/feature_flags.dart
  - Implement analytics for KeyFlow (page views, interactions, errors), structured logging for debugging, feature flag to enable/disable KeyFlow, error tracking integration
  - Purpose: Monitor usage, debug issues, control feature rollout
  - _Leverage: structured logging from ultra-speed spec, existing analytics setup if any_
  - _Requirements: NFR2.2_
  - _Prompt: Implement the task for spec keyflow-implementation, first run spec-workflow-guide to get the workflow guide then implement the task: Role: DevOps Engineer with expertise in observability and feature management | Task: Create ui/lib/services/keyflow_analytics.dart tracking key events (page_view: KeyFlow opened, slot_added: new slot created, slot_deleted: slot removed, slot_reordered: priority changed, node_dragged: position updated, error_occurred: with error type), use structured logging (JSON format) for all events with context (user_id, device_count, slot_count, error_details), create ui/lib/config/feature_flags.dart with keyFlowEnabled flag (default: true), integrate with existing feature flag system or use simple SharedPreferences, conditionally show KeyFlow navigation based on flag, add error tracking integration (report FFI errors, validation failures to error tracking service), instrument performance metrics (page load time, interaction latency) | Restrictions: Analytics must respect user privacy, don't track PII, ensure analytics failures don't break app, feature flags should be easily configurable, logging should not impact performance | Success: Key events tracked with analytics, structured logs provide debugging context, feature flag controls KeyFlow visibility, error tracking reports issues automatically, performance metrics instrumented, privacy respected | After completion: (1) Edit tasks.md to change [-] to [x], (2) Use log-implementation tool with detailed artifacts_

- [ ] 25. Final Quality Gates & Deployment
  - File: .github/workflows/keyflow-ci.yml, docs/KEYFLOW_DEPLOYMENT.md
  - Configure CI quality gates (tests, coverage, lint, performance), create deployment checklist, verify all acceptance criteria met, prepare for production release
  - Purpose: Ensure KeyFlow meets all quality standards before release
  - _Leverage: CI setup from ultra-speed spec, all previous implementation_
  - _Requirements: All NFR requirements, all user story acceptance criteria_
  - _Prompt: Implement the task for spec keyflow-implementation, first run spec-workflow-guide to get the workflow guide then implement the task: Role: DevOps Engineer and QA Lead | Task: Configure KeyFlow-specific CI gates in .github/workflows/keyflow-ci.yml (run all KeyFlow tests, enforce 80% coverage minimum, check golden tests, run performance benchmarks, verify zero lint warnings), create docs/KEYFLOW_DEPLOYMENT.md with pre-deployment checklist (all tests passing, coverage >80%, performance benchmarks met, accessibility verified, documentation complete, error tracking configured, feature flag tested), verify all user story acceptance criteria met (US1-US5), verify all functional requirements implemented (FR1-FR5), verify all non-functional requirements met (NFR1-NFR3), perform final manual testing on all platforms (desktop/tablet), create release notes summarizing KeyFlow features, prepare rollout plan with gradual feature flag enablement | Restrictions: All quality gates must pass, no compromises on test coverage or performance, accessibility must be verified with real assistive tech, deployment must be reversible (feature flag), ensure backward compatibility | Success: All CI gates passing, 80%+ coverage achieved, performance targets met, all acceptance criteria verified, documentation complete, deployment checklist ready, feature flag tested, release notes prepared, ready for production | After completion: (1) Edit tasks.md to change [-] to [x], (2) Use log-implementation tool with detailed artifacts_

## Definition of Done (Per Task)

Each task is complete when:
- ✅ Implementation finished
- ✅ Tests written and passing (if applicable)
- ✅ Coverage ≥80% for new code
- ✅ All quality gates pass (lint, analyze, format)
- ✅ Documentation updated (if public API changed)
- ✅ Implementation logged with log-implementation tool
- ✅ Task marked [x] in tasks.md

## Success Metrics

Track these metrics before/after implementation:

| Metric | Target | Status |
|--------|--------|--------|
| Initial page load | <500ms | - |
| Device update latency | <100ms | - |
| Save operation | <200ms | - |
| Rendering fps | 60fps with 250 nodes | - |
| Memory usage | <100MB with 50 devices | - |
| Test coverage | 80% minimum | - |
| Keyboard accessibility | 100% features | - |
| WCAG compliance | AA level | - |
