# KeyFlow Implementation - Requirements

## Overview

Transform the existing KeyFlow prototype into a fully functional, production-ready visual configuration flow editor. Users should be able to visualize, understand, and modify device-to-profile-to-keymap mappings through an interactive node-based interface.

## Business Context

### Problem Statement

**Current State**: Users must edit RuntimeConfig JSON manually or use CLI commands to configure device mappings. This is error-prone and doesn't provide visual feedback about the configuration structure.

**Example Complexity**:
```json
{
  "devices": [
    {
      "device": { "vendor_id": 1234, "product_id": 5678, "serial": "ABC123" },
      "slots": [
        { "id": "slot-1", "hardware_profile_id": "ergo-60", "keymap_id": "colemak-dh", "active": true, "priority": 0 },
        { "id": "slot-2", "hardware_profile_id": "ergo-60", "keymap_id": "qwerty", "active": false, "priority": 1 }
      ]
    }
  ]
}
```

**User Pain Points**:
- ❌ Can't visualize which device uses which profile
- ❌ Don't understand profile slot priority
- ❌ Can't see fallback chains
- ❌ JSON editing is error-prone
- ❌ No validation feedback

### Desired Outcome

**KeyFlow Visualization**:
```
[Keyboard Device]     [Hardware Profile]     [Keymap]
    VID:1234    ━━━━▶    Ergo-60     ━━━━▶  Colemak-DH  (Active, Priority 0)
    PID:5678           │
    Serial: ABC        └━━━━▶    Ergo-60     ━━━━▶  QWERTY      (Inactive, Priority 1)
```

**User Benefits**:
- ✅ See entire configuration at a glance
- ✅ Understand active vs fallback profiles
- ✅ Drag-drop to reorder priorities
- ✅ Real-time validation (invalid connections show errors)
- ✅ Auto-save to RuntimeConfig

## Current Implementation Status

### ✅ Already Implemented (Prototype)

**Models** (`ui/lib/models/flow/flow_data.dart`):
- `FlowNodeType` enum (device, layout, keymap)
- `DeviceNodeData` - device info + wirings
- `DeviceWiring` - slot configuration
- `LayoutNodeData` - layout type info
- `KeymapNodeData` - keymap layers

**UI Components**:
- `FlowEditorPage` - main canvas with InteractiveViewer
- `DeviceNodeWidget` - displays device with status
- `LayoutNodeWidget` - shows layout preview
- `KeymapNodeWidget` - shows keymap info
- `GridPainter` - background grid
- `EdgesPainter` - bezier curve connections

**Dependencies**:
- `fl_nodes: ^0.5.0+1` added to pubspec.yaml

### ❌ Missing (To Be Implemented)

**Data Layer**:
- `KeyFlowRepository` - fetch/update RuntimeConfig via FFI
- FFI integration for runtime config queries
- Stream for real-time device state updates

**State Management**:
- Riverpod provider for KeyFlow state
- View model for business logic
- State synchronization with Rust

**Interactive Features**:
- Node dragging (currently static positions)
- Edge creation/deletion
- Profile slot reordering
- Validation feedback
- Auto-save

**Real-Time Updates**:
- Device connect/disconnect events
- Profile changes from other sources
- Conflict detection (multiple devices, same profile)

## Functional Requirements

### FR1: Data Fetching & Synchronization

#### FR1.1: Fetch RuntimeConfig
- **Input**: None (query current state from Rust)
- **Output**: `RuntimeConfig` with all devices and slots
- **FFI Call**: Use existing `RuntimeService` via bridge
- **Error Handling**: Show error toast if FFI fails

#### FR1.2: Real-Time Device Updates
- **Input**: Stream of `BridgeStateUpdate` events
- **Output**: Updated KeyFlow visualization
- **Behavior**: When device connects/disconnects, update flow automatically
- **Implementation**: Subscribe to device registry stream

#### FR1.3: Persist Changes
- **Input**: Modified `RuntimeConfig` (after drag-drop, reordering)
- **Output**: Saved to Rust, persisted to disk
- **FFI Call**: Use `RuntimeService::update_config()`
- **Validation**: Rust validates before persisting

### FR2: Interactive Node Manipulation

#### FR2.1: Drag Nodes
- **Behavior**: User can drag any node to reposition it
- **Constraints**: None (free positioning on canvas)
- **Persistence**: Node positions saved to local storage (not RuntimeConfig)

#### FR2.2: Reorder Profile Slots
- **Behavior**: Drag slot connection to change priority
- **Visual Feedback**: Show priority number (0 = highest)
- **Validation**: Can't have duplicate priorities for same device

#### FR2.3: Create New Slot
- **Trigger**: Click "Add Slot" on device node
- **Action**: Open dialog to select hardware profile + keymap
- **Result**: New slot added with next available priority

#### FR2.4: Delete Slot
- **Trigger**: Click delete icon on slot edge
- **Action**: Show confirmation dialog
- **Result**: Slot removed from device, RuntimeConfig updated

#### FR2.5: Toggle Slot Active/Inactive
- **Trigger**: Click slot connection or node
- **Visual**: Active = solid line, Inactive = dashed line
- **Behavior**: Only one slot can be active per device

### FR3: Visual Encoding & Feedback

#### FR3.1: Node Status Colors
- **Device Node**:
  - Green: Connected & remapping active
  - Blue: Connected, no active slots
  - Red: Error state (profile not found, etc.)
  - Grey: Disconnected

- **Profile Node**:
  - Blue: Valid hardware profile
  - Red: Profile not found / invalid

- **Keymap Node**:
  - Green: Keymap loaded and valid
  - Red: Keymap compilation failed

#### FR3.2: Edge Visual Encoding
- **Solid Line**: Active slot (priority 0, active: true)
- **Dashed Line**: Fallback slot (active: false)
- **Thick Line**: Hover state
- **Red Line**: Invalid connection (profile/keymap not found)

#### FR3.3: Priority Indicators
- **Badge on Edge**: Show priority number (0, 1, 2, ...)
- **Tooltip**: "Active (Priority 0)" or "Fallback (Priority 1)"

#### FR3.4: Validation Errors
- **Toast**: Show error message when validation fails
- **Node Badge**: Red exclamation icon on invalid nodes
- **Tooltip**: Explain error (e.g., "Hardware profile 'ergo-60' not found")

### FR4: Keyboard Shortcuts & Accessibility

#### FR4.1: Keyboard Navigation
- **Tab**: Navigate between nodes
- **Arrow Keys**: Move focused node
- **Enter**: Open node details dialog
- **Delete**: Delete selected edge/slot
- **Escape**: Deselect all

#### FR4.2: Zoom & Pan
- **Mouse Wheel**: Zoom in/out
- **Middle Click Drag**: Pan canvas
- **Ctrl + 0**: Reset zoom to 100%
- **Ctrl + Plus/Minus**: Zoom in/out

#### FR4.3: Screen Reader Support
- **Semantic Labels**: Each node has meaningful label
- **Announcements**: State changes announced
- **Focus Indicators**: Clear focus outlines

### FR5: Performance Requirements

#### FR5.1: Rendering Performance
- **Target**: 60fps during interactions
- **Max Nodes**: Support 50+ devices with 5 slots each (250+ nodes)
- **Optimization**: Use `CustomPainter` for edges, avoid rebuilds

#### FR5.2: Data Fetching
- **Initial Load**: <500ms from page open to first render
- **Updates**: <100ms from device event to UI update
- **Persistence**: <200ms from user action to save confirmation

#### FR5.3: Memory Usage
- **Baseline**: <50MB for flow visualization
- **Max**: <100MB with 50 devices

## Non-Functional Requirements

### NFR1: Code Quality

#### NFR1.1: Test Coverage
- **Target**: 80% minimum (90% for business logic)
- **Unit Tests**: Repository, view model logic
- **Widget Tests**: All interactive widgets
- **Golden Tests**: Node visual appearance

#### NFR1.2: Code Metrics
- **Max File Size**: 500 lines (excluding comments)
- **Max Function Size**: 50 lines
- **Lint**: Zero warnings in `flutter analyze`

### NFR2: Architecture

#### NFR2.1: State Management
- **Pattern**: Riverpod with code generation
- **Separation**: UI ↔ ViewModel ↔ Repository ↔ FFI
- **Testability**: All dependencies injected

#### NFR2.2: Error Handling
- **FFI Errors**: Translated to user-friendly messages
- **Validation Errors**: Shown in-line with actionable fixes
- **Logging**: Structured JSON logs for debugging

### NFR3: User Experience

#### NFR3.1: Responsive Design
- **Desktop**: Full-featured with drag-drop
- **Tablet**: Touch-friendly (larger hit targets)
- **Mobile**: Read-only view (editing via dialogs)

#### NFR3.2: Loading States
- **Initial Load**: Skeleton loading with shimmer
- **Saving**: Inline spinner, disable interactions
- **Error**: Clear error message + retry button

#### NFR3.3: Accessibility
- **WCAG 2.1 AA**: Minimum compliance
- **Keyboard Only**: Fully navigable without mouse
- **Screen Reader**: All content accessible

## User Stories

### US1: Visualize Current Configuration
**As a** keyboard enthusiast
**I want to** see my current device-to-keymap mappings visually
**So that** I understand which keyboards use which layouts

**Acceptance Criteria**:
- Open KeyFlow page shows all connected devices
- Each device shows its active profile and keymap
- Fallback slots are clearly distinguished (dashed lines)

### US2: Reorder Fallback Priority
**As a** power user
**I want to** drag-drop profile slots to change priority
**So that** I can control fallback order without editing JSON

**Acceptance Criteria**:
- Drag a slot connection to reorder
- Priority numbers update automatically
- Changes persist to RuntimeConfig immediately

### US3: Add New Profile Slot
**As a** experimenter
**I want to** add a new profile slot to a device
**So that** I can test different keymaps without removing existing ones

**Acceptance Criteria**:
- Click "Add Slot" button on device node
- Dialog shows list of available profiles + keymaps
- New slot appears with lowest priority (inactive)

### US4: Debug Configuration Errors
**As a** troubleshooter
**I want to** see which profiles/keymaps are invalid
**So that** I can fix configuration issues quickly

**Acceptance Criteria**:
- Invalid nodes show red color
- Error icon with tooltip explaining the issue
- Clicking error opens relevant config page

### US5: Real-Time Device Updates
**As a** multi-device user
**I want to** see new devices appear automatically
**So that** I don't have to refresh the page manually

**Acceptance Criteria**:
- Plug in keyboard → appears in flow within 1s
- Unplug keyboard → node fades out, connections removed
- No page reload required

## Success Criteria

### Phase 1: Data Integration (Week 1)
- ✅ KeyFlowRepository implemented with FFI integration
- ✅ RuntimeConfig fetched and displayed correctly
- ✅ Riverpod provider wired up
- ✅ Real-time device updates working
- ✅ Unit tests for repository (80% coverage)

### Phase 2: Interactivity (Week 2)
- ✅ Node dragging works smoothly (60fps)
- ✅ Profile slot reordering functional
- ✅ Add/delete slots working
- ✅ Active/inactive toggling works
- ✅ Changes persist to Rust
- ✅ Widget tests for all interactions

### Phase 3: Visual Polish (Week 3)
- ✅ Status colors implemented
- ✅ Edge visual encoding (solid/dashed)
- ✅ Priority badges on connections
- ✅ Validation error indicators
- ✅ Loading/error states
- ✅ Golden tests for visual appearance

### Phase 4: Accessibility & Performance (Week 4)
- ✅ Keyboard navigation working
- ✅ Screen reader support
- ✅ 60fps during interactions
- ✅ <500ms initial load time
- ✅ WCAG 2.1 AA compliance
- ✅ Performance benchmarks passing

## Out of Scope

### Won't Do in v1
- ❌ Auto-layout (nodes positioned manually)
- ❌ Undo/redo (use manual save/revert)
- ❌ Export to image (use screenshot)
- ❌ Collaborative editing (single user only)
- ❌ Custom node shapes (use predefined widgets)
- ❌ Animation on data changes (instant updates)

### Future Considerations
- Auto-layout algorithms (force-directed, hierarchical)
- Timeline view (historical configuration changes)
- Diff view (compare two RuntimeConfigs)
- Bulk operations (apply profile to multiple devices)

## Dependencies

### Required Packages
- ✅ `fl_nodes: ^0.5.0+1` (already added)
- ✅ `flutter_riverpod: ^2.5.0` (from ultra-speed spec)
- ✅ `riverpod_annotation: ^2.3.0` (code generation)

### FFI Integration
- ✅ `RuntimeService` (existing, in `core/src/services/runtime.rs`)
- ✅ `DeviceRegistryService` (existing)
- ✅ `ProfileRegistryService` (existing)
- ✅ `BridgeStateUpdate` stream (existing)

### Steering Documents
- ✅ `design.md` - UI patterns, colors, typography (created)
- ✅ `tech.md` - Architecture, FFI, testing standards (existing)
- ✅ `structure.md` - File organization (existing)

## Acceptance Criteria (Overall)

This spec is complete when:

1. **Functionality**:
   - ✅ All 5 user stories implemented and tested
   - ✅ Real-time updates working
   - ✅ Data persists correctly
   - ✅ Validation errors shown clearly

2. **Quality**:
   - ✅ 80%+ test coverage
   - ✅ Zero lint warnings
   - ✅ All quality gates pass
   - ✅ Performance targets met

3. **User Experience**:
   - ✅ Keyboard-accessible
   - ✅ Screen reader support
   - ✅ Loading/error states polished
   - ✅ Responsive on desktop/tablet

4. **Documentation**:
   - ✅ User guide for KeyFlow page
   - ✅ Architecture decision records (ADRs)
   - ✅ API documentation for repository

5. **Deployment**:
   - ✅ Feature flag available (optional: enable/disable KeyFlow)
   - ✅ Metrics instrumented (page views, interactions)
   - ✅ Error tracking configured

## References

- AUTONOMOUS-DEV-GUIDE.md - Development workflow
- .spec-workflow/steering/design.md - UI patterns
- .spec-workflow/steering/tech.md - Technical standards
- ui/lib/models/runtime_config.dart - Data models
- core/src/services/runtime.rs - FFI interface
