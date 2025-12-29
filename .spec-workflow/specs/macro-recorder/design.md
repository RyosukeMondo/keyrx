# Design Document

## Architecture

```
User Keys → Daemon Capture → Event Buffer → UI Editor → Rhai Generator → Config
                                                ↓
                                          Simulator Test
```

## Components

### 1. Macro Recorder (Rust - keyrx_daemon/src/macro_recorder.rs)
- Capture key events during recording mode
- Store events with precise timestamps

### 2. MacroRecorderPage (keyrx_ui/src/pages/MacroRecorderPage.tsx)

**UI Layout:**
```
+---------------------------------------------------------------+
| Macro Recorder                         [●  Recording 00:05]  |
+---------------------------------------------------------------+
| [🔴 Start Recording]  [⏹️ Stop]  [▶️  Test Macro]           |
+---------------------------------------------------------------+
| Recorded Events:                                              |
| 0ms    KEY_H ↓                                    [Edit]      |
| 50ms   KEY_H ↑                                    [Delete]    |
| 100ms  KEY_E ↓                                    [Edit]      |
| 150ms  KEY_E ↑                                    [Delete]    |
| 200ms  KEY_L ↓                                    [Edit]      |
| 250ms  KEY_L ↑                                    [Delete]    |
| 300ms  KEY_L ↓                                    [Edit]      |
| 350ms  KEY_L ↑                                    [Delete]    |
| 400ms  KEY_O ↓                                    [Edit]      |
| 450ms  KEY_O ↑                                    [Delete]    |
+---------------------------------------------------------------+
| Generated Rhai Code:                              [Copy]      |
| macro "hello_macro" {                                         |
|     trigger KEY_F1                                            |
|     sequence [                                                 |
|         (KEY_H, Press, 0),                                    |
|         (KEY_H, Release, 50),                                 |
|         ...                                                    |
|     ]                                                          |
| }                                                              |
+---------------------------------------------------------------+
| [Save Macro]                                                  |
+---------------------------------------------------------------+
```

### 3. EventTimeline (keyrx_ui/src/components/EventTimeline.tsx)
- Visual timeline for editing event timing
- Drag events to adjust timestamps

### 4. generateMacroRhai() (keyrx_ui/src/utils/macroGenerator.ts)
- Convert recorded events to Rhai macro syntax

## Data Models

```typescript
interface MacroEvent {
  keyCode: string;
  eventType: 'press' | 'release';
  timestamp: number;  // Relative to recording start (ms)
}

interface Macro {
  name: string;
  trigger: string;  // Hotkey that activates macro
  events: MacroEvent[];
  description?: string;
}
```

## Dependencies

- No new dependencies (use existing React)

## Sources

- [Keyboard Shortcuts design pattern](https://ui-patterns.com/patterns/keyboard-shortcuts)
- [Macro Recorder](https://www.keysmith.app/)
