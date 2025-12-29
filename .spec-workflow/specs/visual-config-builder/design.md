# Design Document

## Architecture

```
Visual UI → State Manager → Rhai Generator → Validation → Daemon
   ↓            ↓               ↓              ↓
Drag-Drop   ConfigState   generateRhai()  WasmCore
```

## Components

### 1. VisualConfigBuilder (keyrx_ui/src/pages/VisualConfigBuilder.tsx)
- Main page with keyboard, layers panel, modifiers panel, code preview
- DndContext provider from @dnd-kit

### 2. VirtualKeyboard (keyrx_ui/src/components/VirtualKeyboard.tsx)
- On-screen keyboard with draggable keys
- Highlight mapped keys with color coding

**UI Layout:**
```
+---------------------------------------------------------------+
| Layer: base                        [+ Add Layer]              |
+---------------------------------------------------------------+
| [Esc] [F1] [F2] [F3] ... [F12]                               |
| [`] [1] [2] ... [0] [-] [=] [Backspace]                      |
| [Tab] [Q→A] [W] [E] ... [P] [[] []]          ← mapped        |
| [Caps] [A] [S] [D] ... [;] ['] [Enter]                       |
| [Shift] [Z] [X] [C] ... [/] [Shift]                          |
+---------------------------------------------------------------+
| Drag from: Source Key     Drop on: Target Key                |
+---------------------------------------------------------------+
```

### 3. LayerPanel (keyrx_ui/src/components/LayerPanel.tsx)
- List of layers with drag-to-reorder
- Add/delete/rename layer buttons

### 4. ModifierPanel (keyrx_ui/src/components/ModifierPanel.tsx)
- List of modifiers/locks
- Drag key to panel to assign

### 5. CodePreview (keyrx_ui/src/components/CodePreview.tsx)
- Monaco editor showing generated Rhai code (read-only)
- Copy button

### 6. generateRhai() (keyrx_ui/src/utils/rhaiGenerator.ts)
- Convert ConfigState to Rhai syntax
- Format with proper indentation

## Data Models

```typescript
interface ConfigState {
  layers: Layer[];
  modifiers: Modifier[];
  locks: Lock[];
}

interface Layer {
  id: string;
  name: string;
  mappings: Mapping[];
}

interface Mapping {
  sourceKey: string;  // "KEY_Q"
  targetKey: string;  // "KEY_A"
  type: 'simple' | 'modifier_trigger' | 'layer_switch';
}
```

## Dependencies

- `@dnd-kit/core@^6.1.0`
- `@dnd-kit/sortable@^8.0.0`

## Sources

- [15 Drag and Drop UI Design Tips](https://bricxlabs.com/blogs/drag-and-drop-ui)
- [Best Practices for Drag-and-Drop Workflow UI](https://latenode.com/blog/best-practices-for-drag-and-drop-workflow-ui)
