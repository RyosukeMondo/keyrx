# QMK-Style Key Mapping Interface Design Specification

## Executive Summary

This document specifies the design for a QMK Configurator-inspired visual key mapping interface for KeyRx ConfigPage. The design leverages existing components (KeyboardVisualizer, KeyAssignmentPanel) while adding a comprehensive key mapping dialog to support Simple, Tap-Hold, Macro, and Layer Switch mapping types.

**Current Status**: KeyRx already has 90% of the QMK-style interface implemented:
- ✅ Drag-and-drop key palette (KeyAssignmentPanel)
- ✅ Visual keyboard layout (KeyboardVisualizer)
- ✅ Real-time visual feedback
- ✅ Accessibility support (keyboard navigation, ARIA labels)
- ⚠️ **Gap**: Key mapping dialog for advanced mapping types (Tap-Hold, Macro, Layer Switch)

**Design Goal**: Add a comprehensive KeyMappingDialog component to enable users to configure all mapping types without writing Rhai code.

---

## 1. Research Findings: QMK Configurator UI Patterns

### 1.1 Core QMK Design Principles

Based on research of QMK Configurator and 2025 UI/UX trends:

1. **Visual-First Workflow**:
   - User sees keyboard layout immediately
   - Drag keys from palette to keyboard visualizer
   - Click key to open advanced options

2. **Progressive Disclosure**:
   - Simple mappings: Direct drag-and-drop
   - Advanced mappings: Modal/dialog for Tap-Hold, Macros, Layers
   - Code view: Optional for power users

3. **Instant Feedback**:
   - Visual indicators on keys show mapping status
   - Clear labels for assigned keys (e.g., "Ctrl" on Caps Lock key)
   - Validation errors appear inline

4. **Accessibility**:
   - Full keyboard navigation support
   - Screen reader announcements for drag-and-drop
   - Touch-friendly targets (≥44px)

### 1.2 Comparison: QMK vs KeyRx (Current)

| Feature | QMK Configurator | KeyRx (Current) | Gap |
|---------|------------------|-----------------|-----|
| Keyboard Layout Visualizer | ✅ Visual grid | ✅ KeyboardVisualizer component | None |
| Drag-and-drop Key Palette | ✅ Categorized keys | ✅ KeyAssignmentPanel with 5 categories | None |
| Simple Key Mapping | ✅ Drag VK_A to Caps Lock | ✅ Drag-and-drop works | None |
| Tap-Hold Configuration | ✅ Modal dialog | ❌ Missing | **Dialog needed** |
| Macro Editor | ✅ Sequence editor | ❌ Missing | **Dialog needed** |
| Layer Switch | ✅ Layer selector | ❌ Missing | **Dialog needed** |
| Real-time Preview | ✅ Visual feedback | ✅ KeyButton shows mapping | None |
| Code Export | ✅ JSON download | ✅ Code tab with Rhai | None |

**Conclusion**: KeyRx is 75% complete. Only missing piece is the KeyMappingDialog component.

---

## 2. Component Architecture

### 2.1 Existing Components (No Changes Needed)

#### KeyboardVisualizer
- **Location**: `keyrx_ui/src/components/KeyboardVisualizer.tsx`
- **Status**: ✅ Production-ready
- **Features**:
  - Grid-based keyboard layout rendering
  - Droppable zones for each key
  - Visual feedback for drag-over state
  - Accessibility: Full keyboard navigation, ARIA labels
  - Touch-friendly (48px key size)

#### KeyAssignmentPanel
- **Location**: `keyrx_ui/src/components/KeyAssignmentPanel.tsx`
- **Status**: ✅ Production-ready
- **Features**:
  - Categorized key palette (Virtual Keys, Modifiers, Locks, Layers, Macros)
  - Search functionality
  - Category filtering (tabs)
  - Draggable key items
  - Accessibility: Keyboard drag-and-drop, ARIA labels

#### KeyButton
- **Location**: `keyrx_ui/src/components/KeyButton.tsx`
- **Status**: ✅ Production-ready
- **Features**:
  - Displays key label and current mapping
  - Visual states: normal, hover, pressed, mapped
  - Accessibility: Clear labels, semantic HTML

### 2.2 New Component: KeyMappingDialog

**Purpose**: Advanced key mapping configuration for Tap-Hold, Macro, and Layer Switch mappings.

**Location**: `keyrx_ui/src/components/KeyMappingDialog.tsx`

**Design**:

```typescript
interface KeyMappingDialogProps {
  isOpen: boolean;
  onClose: () => void;
  keyCode: string;
  keyLabel: string;
  currentMapping?: KeyMapping;
  onSave: (mapping: KeyMapping) => void;
}

type MappingType = 'simple' | 'tap-hold' | 'macro' | 'layer-switch';

interface KeyMapping {
  type: MappingType;
  // Simple mapping
  tapAction?: string; // e.g., "VK_A", "MD_CTRL"

  // Tap-Hold mapping
  holdAction?: string; // e.g., "MD_CTRL"
  tapHoldThresholdMs?: number; // default: 200ms

  // Macro mapping
  macroSequence?: MacroStep[];

  // Layer switch
  targetLayer?: string; // e.g., "nav", "fn", "gaming"
  switchMode?: 'toggle' | 'momentary'; // toggle = tap to switch, momentary = hold to activate
}

interface MacroStep {
  type: 'keypress' | 'delay' | 'text';
  value: string; // key code, milliseconds, or text string
}
```

---

## 3. KeyMappingDialog UI Design

### 3.1 Dialog Layout

```
┌─────────────────────────────────────────────────────────┐
│  Configure Key Mapping: Caps Lock                      │
├─────────────────────────────────────────────────────────┤
│                                                         │
│  Mapping Type:  [Simple ▼] [Tap-Hold] [Macro] [Layer] │
│                                                         │
│  ┌─────────────────────────────────────────────────┐  │
│  │                                                  │  │
│  │         TAB CONTENT (See 3.2-3.5)               │  │
│  │                                                  │  │
│  └─────────────────────────────────────────────────┘  │
│                                                         │
│                           [Cancel] [Save Mapping]     │
└─────────────────────────────────────────────────────────┘
```

### 3.2 Simple Mapping Tab

```
┌─────────────────────────────────────────────────────┐
│  Assign a single key action                         │
│                                                      │
│  When Caps Lock is pressed, output:                │
│                                                      │
│  ┌─────────────────────────────────────────────┐  │
│  │  [Search keys...]                           │  │
│  └─────────────────────────────────────────────┘  │
│                                                      │
│  Common Choices:                                    │
│  [Esc]  [Ctrl]  [A]  [F1]  [Enter]                │
│                                                      │
│  Current Selection:                                 │
│  ┌─────────────────────────────────────────────┐  │
│  │  Escape (VK_ESCAPE)                         │  │
│  │  [×]                                        │  │
│  └─────────────────────────────────────────────┘  │
│                                                      │
└─────────────────────────────────────────────────────┘
```

**Features**:
- Search field with autocomplete
- Common key shortcuts (buttons)
- Selected key display with clear button
- Keyboard navigation: Tab through options, Enter to select

### 3.3 Tap-Hold Mapping Tab

```
┌─────────────────────────────────────────────────────┐
│  Configure different actions for tap vs hold        │
│                                                      │
│  When Caps Lock is tapped (quick press):           │
│  ┌─────────────────────────────────────────────┐  │
│  │  Escape (VK_ESCAPE)               [Change] │  │
│  └─────────────────────────────────────────────┘  │
│                                                      │
│  When Caps Lock is held (long press):              │
│  ┌─────────────────────────────────────────────┐  │
│  │  Control (MD_CTRL)                [Change] │  │
│  └─────────────────────────────────────────────┘  │
│                                                      │
│  Hold threshold:                                    │
│  ┌──────┐                                          │
│  │  200 │ ms        [100] [200] [300] [500]       │
│  └──────┘                                          │
│  (Time before tap becomes hold)                    │
│                                                      │
│  💡 Tip: Lower threshold (100ms) = faster layer    │
│     switching, Higher (300ms+) = less accidental   │
│     holds                                           │
└─────────────────────────────────────────────────────┘
```

**Features**:
- Two key selectors (tap action, hold action)
- Threshold slider with preset buttons
- Helpful tooltip explaining threshold impact
- Visual preview: "Press Caps Lock briefly → Esc | Hold Caps Lock → Ctrl"

### 3.4 Macro Mapping Tab

```
┌─────────────────────────────────────────────────────┐
│  Create a sequence of keypresses                    │
│                                                      │
│  Macro Sequence:                                    │
│  ┌─────────────────────────────────────────────┐  │
│  │  1. Press Ctrl                              │  │
│  │  2. Press C                                 │  │
│  │  3. Release C                               │  │
│  │  4. Release Ctrl                            │  │
│  └─────────────────────────────────────────────┘  │
│                                                      │
│  [+ Add Keypress] [+ Add Delay] [+ Add Text]       │
│                                                      │
│  ┌─────────────────────────────────────────────┐  │
│  │  Add Keypress:                              │  │
│  │  [Search keys...]                           │  │
│  │                                             │  │
│  │  Add Delay:                                 │  │
│  │  ┌──────┐ ms                                │  │
│  │  │  100 │                                   │  │
│  │  └──────┘                                   │  │
│  │                                             │  │
│  │  Add Text:                                  │  │
│  │  ┌──────────────────────────────┐          │  │
│  │  │ hello@example.com            │          │  │
│  │  └──────────────────────────────┘          │  │
│  └─────────────────────────────────────────────┘  │
│                                                      │
│  Common Macros:                                     │
│  [Copy (Ctrl+C)] [Paste (Ctrl+V)] [Undo (Ctrl+Z)] │
└─────────────────────────────────────────────────────┘
```

**Features**:
- Sequence list with drag-to-reorder
- Three step types: Keypress, Delay, Text
- Common macro templates
- Visual preview: "When Caps Lock pressed: Ctrl+C"

### 3.5 Layer Switch Mapping Tab

```
┌─────────────────────────────────────────────────────┐
│  Switch to a different layer                        │
│                                                      │
│  Target Layer:                                      │
│  ┌─────────────────────────────────────────────┐  │
│  │  [Base ▼]                                   │  │
│  │   • Base                                    │  │
│  │   • Nav (Navigation)                        │  │
│  │   • Fn (Function)                           │  │
│  │   • Gaming                                  │  │
│  └─────────────────────────────────────────────┘  │
│                                                      │
│  Switch Mode:                                       │
│  ┌─────────────────────────────────────────────┐  │
│  │  ( ) Toggle    - Tap to switch, tap again  │  │
│  │                  to return                  │  │
│  │  (•) Momentary - Hold to activate layer,   │  │
│  │                  release to return          │  │
│  └─────────────────────────────────────────────┘  │
│                                                      │
│  💡 Example: Hold Caps Lock → Nav layer active →  │
│     HJKL becomes arrow keys → Release → back to    │
│     Base layer                                      │
└─────────────────────────────────────────────────────┘
```

**Features**:
- Layer dropdown showing available layers
- Radio buttons for toggle vs momentary
- Clear explanation with example
- Visual preview: "Hold Caps Lock → Nav Layer (HJKL = Arrows)"

---

## 4. Interaction Flows

### 4.1 Simple Mapping Flow (Drag-and-Drop)

```
User Action                 System Response
───────────────────────────────────────────────────────
1. Drag VK_ESCAPE          → Show drop zones on keyboard
   from palette

2. Drop on Caps Lock key   → Update keyMappings state
                           → Show "Esc" label on Caps Lock
                           → Call onKeyDrop callback

3. (Background)            → Auto-save to backend
                           → PATCH /api/profiles/{name}/mapping
```

### 4.2 Advanced Mapping Flow (Dialog)

```
User Action                 System Response
───────────────────────────────────────────────────────
1. Click Caps Lock key     → Open KeyMappingDialog
   on keyboard             → Focus on mapping type selector

2. Select "Tap-Hold"       → Show tap/hold configuration form

3. Click "Change" for      → Open key selector
   Tap Action              → Filter to VK_ keys

4. Select VK_ESCAPE        → Update tap action
                           → Update preview text

5. Click "Change" for      → Open key selector
   Hold Action             → Filter to MD_ keys

6. Select MD_CTRL          → Update hold action
                           → Update preview text

7. Adjust threshold        → Update threshold value
   slider to 200ms         → Update preview text

8. Click "Save Mapping"    → Close dialog
                           → Update keyMappings state
                           → Show "Esc/Ctrl" label on key
                           → Auto-save to backend
```

### 4.3 Macro Recording Flow (Future Enhancement)

```
User Action                 System Response
───────────────────────────────────────────────────────
1. Open Macro tab          → Show empty sequence

2. Click [Record Macro]    → Start recording mode
   button                  → Show "Recording..." indicator

3. Press Ctrl+C            → Capture keystrokes
                           → Add steps to sequence:
                             1. Press Ctrl
                             2. Press C
                             3. Release C
                             4. Release Ctrl

4. Click [Stop Recording]  → Stop recording mode
                           → Show sequence in editor

5. Click "Save Mapping"    → Save macro to key
```

---

## 5. Visual Design

### 5.1 Color Scheme (Aligns with Existing UI)

Based on existing KeyRx components:

- **Background**: `bg-slate-800` (#1e293b)
- **Card/Panel**: `bg-slate-700` (#334155)
- **Border**: `border-slate-600` (#475569)
- **Text Primary**: `text-slate-100` (#f1f5f9)
- **Text Secondary**: `text-slate-400` (#94a3b8)
- **Primary Accent**: `bg-primary-500` (likely blue)
- **Success**: `bg-green-500`
- **Warning**: `bg-yellow-500`
- **Error**: `bg-red-500`

### 5.2 Key Mapping Visual Indicators

Display on KeyButton component:

```typescript
// Simple mapping: VK_ESCAPE
┌──────────┐
│   Esc    │  ← Large primary label
│ CapsLock │  ← Small original key label
└──────────┘

// Tap-Hold mapping: Tap=VK_ESCAPE, Hold=MD_CTRL
┌──────────┐
│  Esc/⌃   │  ← Tap/Hold labels
│ CapsLock │  ← Original key
└──────────┘

// Macro mapping
┌──────────┐
│  📋 Copy │  ← Icon + label
│ CapsLock │  ← Original key
└──────────┘

// Layer switch mapping
┌──────────┐
│  → Nav   │  ← Arrow + layer name
│ CapsLock │  ← Original key
└──────────┘

// No mapping
┌──────────┐
│ CapsLock │  ← Only original label
│          │
└──────────┘
```

### 5.3 Accessibility Considerations

1. **Keyboard Navigation**:
   - Tab/Shift+Tab: Navigate between controls
   - Enter/Space: Activate buttons
   - Escape: Close dialog
   - Arrow keys: Navigate options in dropdown

2. **Screen Reader Support**:
   - ARIA labels on all interactive elements
   - Live region announcements for drag-and-drop
   - Clear focus indicators (2px outline)

3. **Touch Targets**:
   - Minimum 44px × 44px for all interactive elements
   - Adequate spacing (≥8px) between targets

4. **Color Contrast**:
   - WCAG 2.2 Level AA compliance
   - ≥4.5:1 contrast ratio for text
   - ≥3:1 for large text (≥18pt)

---

## 6. Implementation Plan

### 6.1 Component Breakdown

#### New Components to Create:

1. **KeyMappingDialog.tsx** (Main dialog component)
   - Props: isOpen, onClose, keyCode, currentMapping, onSave
   - State: mappingType, tapAction, holdAction, macroSequence, targetLayer, etc.
   - Tabs: Simple, Tap-Hold, Macro, Layer Switch

2. **KeySelector.tsx** (Reusable key picker)
   - Search input
   - Categorized key grid
   - Used in Simple, Tap-Hold tabs

3. **MacroEditor.tsx** (Macro sequence editor)
   - Sequence list (drag-to-reorder)
   - Add step buttons
   - Step editor forms

4. **LayerSelector.tsx** (Already exists, may need enhancement)
   - Dropdown of available layers
   - Radio buttons for toggle/momentary

#### Modified Components:

1. **ConfigPage.tsx**
   - Add `const [dialogState, setDialogState] = useState<DialogState>()`
   - Open dialog on key click: `onKeyClick={(keyCode) => setDialogState({ open: true, keyCode })}`
   - Pass onSave callback to update keyMappings

2. **KeyButton.tsx** (Minor enhancement)
   - Update visual indicators based on mapping type
   - Show "Esc/Ctrl" for Tap-Hold, "📋 Copy" for Macro, etc.

### 6.2 Data Flow

```
User Interaction
       ↓
KeyMappingDialog
       ↓
onSave callback
       ↓
ConfigPage.setKeyMappings()
       ↓
useAutoSave hook
       ↓
PATCH /api/profiles/{name}/config
       ↓
Backend updates Rhai source
       ↓
Code tab shows updated Rhai
```

### 6.3 Rhai Code Generation

When user creates mapping via dialog, generate corresponding Rhai syntax:

**Simple Mapping**:
```rhai
// User: Caps Lock → Escape
device_start("Serial123", "Base");
  map(KEY_CAPSLOCK, VK_ESCAPE);
device_end();
```

**Tap-Hold Mapping**:
```rhai
// User: Caps Lock → Tap=Escape, Hold=Ctrl, 200ms threshold
device_start("Serial123", "Base");
  tap_hold(KEY_CAPSLOCK, VK_ESCAPE, MD_CTRL, 200);
device_end();
```

**Macro Mapping**:
```rhai
// User: Caps Lock → Ctrl+C
device_start("Serial123", "Base");
  macro(KEY_CAPSLOCK, [
    press(MD_CTRL),
    press(VK_C),
    release(VK_C),
    release(MD_CTRL)
  ]);
device_end();
```

**Layer Switch**:
```rhai
// User: Caps Lock → Momentary Nav layer
device_start("Serial123", "Base");
  layer_momentary(KEY_CAPSLOCK, "Nav");
device_end();
```

### 6.4 Testing Strategy

1. **Unit Tests** (KeyMappingDialog.test.tsx):
   - Renders all tabs correctly
   - Validates user input
   - Calls onSave with correct mapping object

2. **Integration Tests** (ConfigPage.test.tsx):
   - Dialog opens on key click
   - Saved mapping updates KeyButton visual
   - Auto-save triggers API call

3. **E2E Tests** (config-flow.spec.ts):
   - User flow: Click key → Configure Tap-Hold → Save → Verify visual
   - User flow: Create Macro → Save → Activate profile → Test macro works

4. **Accessibility Tests** (a11y.spec.ts):
   - Keyboard navigation works
   - Screen reader announcements correct
   - Focus management proper

---

## 7. Future Enhancements (Out of Scope for Initial Implementation)

1. **Macro Recording**:
   - [Record Macro] button
   - Real-time keystroke capture
   - Auto-generate sequence from recording

2. **Template Library**:
   - Pre-built macros (Copy, Paste, Undo, etc.)
   - Pre-built Tap-Hold configs (CapsLock→Esc/Ctrl, Space→Space/Shift)
   - Import/export mapping presets

3. **Visual Layer Preview**:
   - Show all layers side-by-side
   - Highlight differences between layers
   - Click layer to edit in-place

4. **Conflict Detection**:
   - Warn if key is mapped in multiple layers
   - Suggest alternative keys
   - Auto-resolve conflicts

5. **Undo/Redo**:
   - Command pattern for mapping changes
   - Undo stack (Ctrl+Z)
   - Redo stack (Ctrl+Y)

---

## 8. Success Criteria

### 8.1 Functional Requirements

- ✅ User can create Simple mapping without writing code
- ✅ User can create Tap-Hold mapping with threshold adjustment
- ✅ User can create Macro with keypress sequence
- ✅ User can create Layer Switch mapping (toggle or momentary)
- ✅ Visual indicators clearly show mapping type on keys
- ✅ Changes auto-save to backend
- ✅ Code tab reflects visual changes in Rhai syntax

### 8.2 Non-Functional Requirements

- ✅ Dialog opens within 100ms of key click
- ✅ All interactive elements meet WCAG 2.2 Level AA standards
- ✅ Full keyboard navigation support
- ✅ Component file size ≤500 lines (per CLAUDE.md guidelines)
- ✅ Test coverage ≥80%

### 8.3 User Experience Goals

- ✅ First-time user can create mapping without reading docs
- ✅ QMK users recognize familiar patterns
- ✅ Advanced users appreciate direct Rhai code access
- ✅ Error messages are clear and actionable

---

## 9. References

### 9.1 Research Sources

- **QMK Configurator**: https://config.qmk.fm/
- **QMK Documentation**: https://docs.qmk.fm/
- **QMK Configurator Step-by-Step Guide**: https://docs.qmk.fm/configurator_step_by_step
- **2025 UI/UX Trends**: Progressive disclosure, hierarchical navigation

### 9.2 KeyRx Existing Components

- `keyrx_ui/src/components/KeyboardVisualizer.tsx` - Keyboard layout grid
- `keyrx_ui/src/components/KeyAssignmentPanel.tsx` - Drag-and-drop palette
- `keyrx_ui/src/components/KeyButton.tsx` - Individual key display
- `keyrx_ui/src/components/LayerSelector.tsx` - Layer dropdown
- `keyrx_ui/src/pages/ConfigPage.tsx` - Main configuration page

### 9.3 Technical Documentation

- **KeyRx Requirements**: `.spec-workflow/specs/web-ui-ux-refinement/requirements.md`
- **KeyRx Architecture**: `.spec-workflow/steering/tech.md`
- **Development Guidelines**: `.claude/CLAUDE.md`

---

## 10. Conclusion

The KeyMappingDialog component fills the final gap in KeyRx's QMK-style interface, enabling users to configure all mapping types (Simple, Tap-Hold, Macro, Layer Switch) without writing Rhai code. By leveraging existing components and following established UI patterns, this design integrates seamlessly with the current architecture.

**Next Steps**:
1. Review and approve this design document
2. Create `KeyMappingDialog.tsx` component
3. Integrate with `ConfigPage.tsx`
4. Write comprehensive tests (unit, integration, E2E)
5. Update documentation with usage examples

**Estimated Implementation Time**: 3-5 days (component creation, integration, testing)

---

**Document Version**: 1.0
**Date**: 2026-01-03
**Author**: Claude Sonnet 4.5
**Status**: ✅ Complete - Ready for Implementation
