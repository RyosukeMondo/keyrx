# Legendary Transition: Gap Analysis

> Making KeyRx ever-lasting through comprehensive feature coverage.

## Gap Analysis: What Makes Software Legendary?

| Dimension | Current Coverage | Gap |
|-----------|------------------|-----|
| **Core Functionality** | Strong | - |
| **Performance** | Sub-1ms | - |
| **Debuggability** | Added | - |
| **Extensibility** | Rhai only | Plugin system? |
| **Adoption/Migration** | Missing | Import from competitors |
| **Learning Curve** | Missing | Progressive complexity |
| **Self-Documentation** | Missing | Auto-generated cheat sheets |
| **Safety/Recovery** | Partial | Emergency exit, safe mode |
| **Community/Sharing** | Phase 4 | Module system, trust model |
| **User Understanding** | Missing | Typing analysis, suggestions |

---

## Missing Perspectives

### 1. Script Testing Framework (Unique Differentiator)

No other remapper lets users write tests for their configs:

```rhai
#[test]
fn capslock_becomes_escape() {
    simulate_tap("CapsLock");
    assert_output("Escape");
}

#[test]
fn hold_capslock_is_ctrl() {
    simulate_hold("CapsLock", 300); // ms
    simulate_tap("C");
    assert_output("Ctrl+C");
}
```

**Why legendary**: Users can refactor configs with confidence. AI agents can verify changes.

---

### 2. Conflict Detection & Resolution

Visual conflict graph before problems occur:

```
Warning: Conflict Detected:

  CapsLock -> Escape (line 12)
  CapsLock -> Ctrl (hold) (line 45)

  These overlap. Which takes priority?

  [Tap-Hold Resolution] [Keep First] [Keep Second]
```

**Why legendary**: Prevents frustration. Other tools fail silently.

---

### 3. Migration Importers (Adoption Accelerator)

```bash
# Import from competitors
keyrx import --from karabiner ~/.config/karabiner/karabiner.json
keyrx import --from autohotkey script.ahk
keyrx import --from keyd /etc/keyd/default.conf
keyrx import --from kanata config.kbd
```

**Why legendary**: Zero friction adoption. Meet users where they are.

---

### 4. Progressive Complexity (Onboarding)

```
+-------------------------------------------------------------+
| SIMPLE MODE (90% of users)                                  |
| +-------------+    +-------------+                          |
| | CapsLock    | -> | Escape      |                          |
| +-------------+    +-------------+                          |
| [+ Add Remap]                                               |
+-------------------------------------------------------------+
| ADVANCED MODE (power users)                                 |
| tap_hold("CapsLock", tap: "Escape", hold: "Ctrl");          |
+-------------------------------------------------------------+
| EXPERT MODE (full Rhai scripting)                           |
| on_key("CapsLock", |event| { ... });                        |
+-------------------------------------------------------------+
```

**Why legendary**: Don't overwhelm beginners, don't limit experts.

---

### 5. Auto-Generated Documentation

From your config, generate a visual keyboard cheat sheet:

- Visual keyboard layout with your mappings
- Layer legend and switching instructions
- Export to PDF for printing
- Shareable with team members

**Why legendary**: Users can print and stick on monitor. Shareable.

---

### 6. Composable Module System

```rhai
// Like npm for keyboard configs
import "std:home-row-mods";      // Official standard library
import "community:vim-arrows";    // Community vetted
import "local:my-macros";         // User's own modules

// Compose with overrides
home_row_mods::configure(
    tap_timeout: 180,  // Override default
);
```

**Why legendary**: Standing on shoulders of giants. Community compounds value.

---

### 7. Typing Analysis & Suggestions

```
+-------------------------------------------------------------+
| TYPING INSIGHTS (last 7 days)                               |
+-------------------------------------------------------------+
|                                                             |
| [!] CapsLock: Never used (0 presses)                        |
|     -> Suggestion: Remap to Escape or Ctrl                  |
|                                                             |
| [~] Right Shift: Rarely used (12 presses vs 2,847 Left)     |
|     -> Suggestion: Consider one-shot or different mapping   |
|                                                             |
| [+] Home Row: 78% of keypresses (efficient!)                |
|                                                             |
| [!] Pinky strain detected: 340 stretches to Backspace       |
|     -> Suggestion: Remap Backspace closer to home row       |
|                                                             |
+-------------------------------------------------------------+
```

**Why legendary**: The app understands YOU and helps you improve.

---

### 8. Safe Mode / Emergency Exit

```
ALWAYS WORKS, NEVER REMAPPED:

  Ctrl + Alt + Shift + Escape = Disable KeyRx instantly

  Physical indicator: System tray icon changes to red

  Recovery: Press same combo to re-enable
```

**Why legendary**: Users never fear getting locked out. Trust.

---

### 9. Keyboard Layout Awareness

```rhai
// Config works regardless of physical layout
layout("colemak");  // or "dvorak", "qwerty", "azerty"

// Mappings use logical positions, not physical keys
remap("home_row_left_index", "Ctrl");  // 'F' on QWERTY, 'T' on Colemak
```

**Why legendary**: One config, any layout. True portability.

---

### 10. Living Documentation / Changelog

```
+-------------------------------------------------------------+
| CONFIG HISTORY                                              |
+-------------------------------------------------------------+
| Today 14:32 - Added Vim navigation layer                    |
| Yesterday - Changed tap_timeout from 200ms to 180ms         |
| 3 days ago - Imported from Karabiner                        |
|                                                             |
| [Undo] [Compare Versions] [Restore]                         |
+-------------------------------------------------------------+
```

**Why legendary**: Never lose work. Fearless experimentation.

---

## Priority Recommendation

| Priority | Feature | Why |
|----------|---------|-----|
| **P0** | Safe Mode / Emergency Exit | Trust & Safety |
| **P0** | Progressive Complexity | Adoption |
| **P1** | Script Testing Framework | Quality & AI-friendly |
| **P1** | Migration Importers | Adoption |
| **P1** | Conflict Detection | UX |
| **P2** | Auto-Generated Docs | Delight |
| **P2** | Module System | Community |
| **P3** | Typing Analysis | Stickiness |
| **P3** | Layout Awareness | Portability |

---

## Foundational Features (Added to Steering)

The following three features are foundational and have been added to the steering documents:

1. **Safe Mode / Emergency Exit** - Trust & Safety foundation
2. **Progressive Complexity** - Adoption & onboarding foundation
3. **Script Testing Framework** - Quality & AI-friendly foundation
