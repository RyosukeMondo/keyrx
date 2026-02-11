# Visual Configuration Builder Guide

## Introduction

The **Visual Configuration Builder** provides a drag-and-drop interface for creating keyboard remapping configurations without writing code. It's ideal for users who prefer a graphical approach to building their keyboard layouts.

### Key Benefits

- **No Code Required**: Build configurations visually with drag-and-drop
- **Instant Visualization**: See your keyboard layout and mappings in real-time
- **Live Code Preview**: Watch Rhai code generate automatically as you work
- **Import/Export**: Load existing Rhai configs or export your visual work
- **Error Prevention**: Visual interface prevents many common syntax errors
- **Layer Management**: Easily organize mappings across multiple layers

## Getting Started

### Opening the Visual Builder

1. Launch the KeyRX web interface (default: http://localhost:8080)
2. Click **"Visual Builder"** in the navigation menu
3. The visual builder interface will load with four main panels

### Interface Overview

```
┌─────────────────────────────────────────────────────────────┐
│ Visual Configuration Builder                     [Import] [Export] │
├──────────────────┬──────────────────────────────────────────┤
│ Layer Panel      │ Virtual Keyboard                         │
│                  │                                          │
│ • Base Layer     │ [Q] [W] [E] [R] [T] [Y] [U] [I] [O] [P] │
│ • Gaming         │ [A] [S] [D] [F] [G] [H] [J] [K] [L] [;] │
│                  │ [Z] [X] [C] [V] [B] [N] [M] [,] [.] [/] │
│ [+ Add Layer]    │                                          │
│                  │ Drop keys here to create mappings        │
├──────────────────┼──────────────────────────────────────────┤
│ Modifier Panel   │ Code Preview                             │
│                  │                                          │
│ • Shift          │ layer "base" {                           │
│ • Ctrl           │     map KEY_A to KEY_B;                  │
│ • Alt            │ }                                        │
│                  │                                          │
│ [+ Add Modifier] │ [Copy Code]                              │
└──────────────────┴──────────────────────────────────────────┘
```

**Four Main Panels:**

1. **Layer Panel** (top-left): Manage configuration layers
2. **Virtual Keyboard** (top-right): Drag-and-drop keyboard interface
3. **Modifier Panel** (bottom-left): Configure modifiers and lock keys
4. **Code Preview** (bottom-right): View generated Rhai code in real-time

## Building Your First Configuration

### Step 1: Create a Layer

1. The "base" layer is created automatically
2. Click **[+ Add Layer]** to create additional layers
3. Click the pencil icon to rename a layer
4. Drag layers to reorder them

**Example:**
```
Base Layer (active)
Gaming Layer
Numpad Layer
```

### Step 2: Create a Simple Mapping

1. Select the layer you want to add mappings to (e.g., "base")
2. On the virtual keyboard, drag a key to another key position
3. The mapping is created instantly

**Visual Example:**
```
Drag KEY_A → Drop on KEY_B

Result: map KEY_A to KEY_B;
```

**You'll see:**
- Source key (KEY_A) shows a mapping indicator
- The Code Preview updates with the new mapping
- The mapping appears in the selected layer

### Step 3: Add Modifiers

1. Click **[+ Add Modifier]** in the Modifier Panel
2. Select a modifier type (Shift, Ctrl, Alt, Super)
3. Drag a key from the keyboard onto the modifier
4. The key becomes a modifier

**Example:**
```
Drag KEY_CAPSLOCK → Drop on "Ctrl" modifier

Result:
modifier ctrl = KEY_CAPSLOCK;

layer "base" {
    map ctrl + KEY_A to KEY_B;  // Ctrl-based mappings now available
}
```

### Step 4: Add Lock Keys

1. Click **[+ Add Lock]** in the Modifier Panel
2. Select a lock type (CapsLock, NumLock, ScrollLock)
3. Drag a key to assign it as a lock key

**Example:**
```
Drag KEY_NUMLOCK → Drop on "NumLock" lock

Result: lock numlock = KEY_NUMLOCK;
```

## Advanced Features

### Multi-Layer Mappings

Create different behaviors for the same key across layers:

1. Create multiple layers (e.g., "base", "gaming", "numpad")
2. Switch to each layer and create different mappings
3. The same physical key can do different things in each layer

**Example Workflow:**

**Layer: base**
```
Drag KEY_H → KEY_LEFT
Result: map KEY_H to KEY_LEFT;  // Vim-style navigation
```

**Layer: gaming**
```
Drag KEY_H → KEY_H
Result: map KEY_H to KEY_H;  // Normal H key for gaming
```

### Using Modifiers in Mappings

**Method 1: Drag with Modifier Held**
1. Hold Ctrl, Shift, or Alt on your physical keyboard
2. Drag a key on the virtual keyboard
3. The mapping includes the modifier

**Method 2: Click Modifier First**
1. Click a modifier checkbox in the Modifier Panel
2. Drag a key to create a modified mapping
3. Uncheck the modifier when done

**Visual Result:**
```
Modifier: Ctrl + KEY_C → KEY_ESC

Generated Code:
modifier ctrl = KEY_LEFTCTRL;

layer "base" {
    map ctrl + KEY_C to KEY_ESC;
}
```

### Layer Ordering

Layer order matters in KeyRX:

1. **Higher layers** are checked first for mappings
2. **Drag layers** in the Layer Panel to reorder
3. **First match wins** - if a key is mapped in layer 2, layer 1's mapping won't be used

**Best Practice:**
```
1. Specific layers (e.g., "gaming") at the top
2. General layers (e.g., "base") at the bottom
```

### Highlighting Mapped Keys

The virtual keyboard highlights keys based on their state:

- **Blue outline**: Key is mapped in the current layer
- **Gray background**: Key is unmapped
- **Orange outline**: Key is assigned as a modifier
- **Purple outline**: Key is assigned as a lock

## Import/Export Workflow

### Exporting Your Configuration

1. Build your configuration visually
2. Click **[Export]** button at the top-right
3. Choose a filename (e.g., `my-config.rhai`)
4. The Rhai code is downloaded as a file

**What gets exported:**
```rhai
// All modifiers
modifier ctrl = KEY_LEFTCTRL;
modifier shift = KEY_LEFTSHIFT;

// All locks
lock capslock = KEY_CAPSLOCK;

// All layers with their mappings
layer "base" {
    map KEY_A to KEY_B;
    map ctrl + KEY_C to KEY_ESC;
}

layer "gaming" {
    map KEY_W to KEY_UP;
}
```

### Importing an Existing Configuration

1. Click **[Import]** button at the top-right
2. Select a `.rhai` file from your computer
3. The visual builder parses and visualizes the configuration

**What happens:**
- Layers are created automatically
- Mappings appear on the virtual keyboard
- Modifiers and locks populate the Modifier Panel
- Code Preview shows the original Rhai code

**Import Limitations:**

The visual builder supports basic Rhai syntax:
- ✅ Simple mappings: `map KEY_A to KEY_B;`
- ✅ Modifier mappings: `map ctrl + KEY_C to KEY_ESC;`
- ✅ Layer definitions: `layer "name" { ... }`
- ✅ Modifier definitions: `modifier ctrl = KEY_LEFTCTRL;`
- ✅ Lock definitions: `lock capslock = KEY_CAPSLOCK;`

Advanced features not supported in visual mode:
- ❌ Tap-hold actions
- ❌ Custom Rhai functions
- ❌ Conditional logic
- ❌ Variables and expressions

**Workaround for Advanced Features:**
1. Build basic structure in Visual Builder
2. Export to Rhai file
3. Edit the Rhai file manually to add advanced features
4. Load in Config Editor for full validation

## Code Preview Panel

### Real-Time Code Generation

The Code Preview panel shows Rhai code as you build:

- **Instant Updates**: Changes appear immediately
- **Syntax Highlighting**: Rhai keywords and strings are colored
- **Read-Only**: View-only to prevent manual edits (use Config Editor for that)
- **Copy Button**: Click to copy the entire configuration

### Understanding Generated Code

**Structure:**
```rhai
// 1. Modifier definitions (top)
modifier ctrl = KEY_LEFTCTRL;
modifier shift = KEY_LEFTSHIFT;

// 2. Lock definitions
lock capslock = KEY_CAPSLOCK;

// 3. Layers (in order)
layer "base" {
    map KEY_A to KEY_B;
    map ctrl + KEY_C to KEY_ESC;
}

layer "gaming" {
    map KEY_W to KEY_UP;
}
```

**Code Formatting:**
- 4-space indentation for layer contents
- Alphabetically sorted mappings within layers
- Comments showing layer descriptions

## Keyboard Navigation

The Visual Builder is fully keyboard-accessible:

### Layer Panel
- **Tab**: Navigate between layers and buttons
- **Enter/Space**: Select layer, click buttons
- **Arrow Keys**: Reorder layers (when focused)
- **Delete**: Remove selected layer

### Virtual Keyboard
- **Tab**: Navigate between keys
- **Enter/Space**: Start drag operation
- **Arrow Keys**: Move dragged key
- **Enter**: Drop key at current position
- **Escape**: Cancel drag

### Modifier Panel
- **Tab**: Navigate between modifiers
- **Enter/Space**: Toggle modifier checkbox
- **Delete**: Remove modifier/lock

### Code Preview
- **Ctrl+A**: Select all code
- **Ctrl+C**: Copy code to clipboard

## Tips and Best Practices

### Organizing Layers

**Strategy 1: Purpose-Based Layers**
```
base       → Default mappings for all applications
vim        → Vim-style navigation (Ctrl+H/J/K/L → arrows)
gaming     → Gaming-specific (WASD unmapped)
numpad     → Number pad emulation on right hand
```

**Strategy 2: Application-Based Layers**
```
base       → System-wide defaults
browser    → Browser shortcuts
ide        → IDE shortcuts
terminal   → Terminal navigation
```

### Naming Conventions

**Good Layer Names:**
- ✅ `base` - Clear, short, lowercase
- ✅ `vim-nav` - Descriptive with hyphen
- ✅ `gaming` - Purpose is obvious

**Avoid:**
- ❌ `Layer1` - Not descriptive
- ❌ `My Super Cool Vim Navigation Layer` - Too long
- ❌ `layer_with_underscores` - Use hyphens instead

### Keeping Configurations Manageable

**Size Guidelines:**
- **Layers**: 3-5 layers is typical, 10+ becomes hard to manage
- **Mappings per Layer**: 10-30 mappings is reasonable
- **Total Mappings**: Under 100 for maintainability

**When to Split:**
```
Bad: One "base" layer with 80 mappings
     ↓
Good: Split into:
      - base (20 core mappings)
      - navigation (15 arrow/movement mappings)
      - editing (15 text editing shortcuts)
      - media (10 media control mappings)
```

### Testing Your Configuration

**Workflow:**
1. Build configuration visually
2. Click **[Export]** and save as `test-config.rhai`
3. Open **Config Editor** and load the file
4. Run validation to catch any issues
5. Use the simulator to test key combinations
6. Iterate based on feedback

## Common Workflows

### Workflow 1: Vim-Style Navigation

**Goal**: Map Ctrl+H/J/K/L to arrow keys for Vim-style navigation

**Steps:**
1. Create layer "vim-nav"
2. Add modifier: Ctrl → KEY_LEFTCTRL
3. Create mappings:
   - Drag KEY_H to KEY_LEFT (while holding Ctrl)
   - Drag KEY_J to KEY_DOWN (while holding Ctrl)
   - Drag KEY_K to KEY_UP (while holding Ctrl)
   - Drag KEY_L to KEY_RIGHT (while holding Ctrl)
4. Export as `vim-nav.rhai`

**Result:**
```rhai
modifier ctrl = KEY_LEFTCTRL;

layer "vim-nav" {
    map ctrl + KEY_H to KEY_LEFT;
    map ctrl + KEY_J to KEY_DOWN;
    map ctrl + KEY_K to KEY_UP;
    map ctrl + KEY_L to KEY_RIGHT;
}
```

### Workflow 2: Swapping Caps Lock and Escape

**Goal**: Make Caps Lock act as Escape (common for Vim users)

**Steps:**
1. Select "base" layer
2. Drag KEY_CAPSLOCK to KEY_ESC
3. (Optional) Drag KEY_ESC to KEY_CAPSLOCK to swap completely
4. Export as `caps-to-esc.rhai`

**Result:**
```rhai
layer "base" {
    map KEY_CAPSLOCK to KEY_ESC;
    map KEY_ESC to KEY_CAPSLOCK;  // Optional swap
}
```

### Workflow 3: Gaming Layer (Disable Remapping)

**Goal**: Create a gaming layer where keys work normally (bypass other layers)

**Steps:**
1. Create layer "gaming"
2. For each key you want normal (e.g., WASD):
   - Drag KEY_W to KEY_W (maps to itself)
   - Drag KEY_A to KEY_A
   - Drag KEY_S to KEY_S
   - Drag KEY_D to KEY_D
3. Drag "gaming" layer to the top (highest priority)

**Why this works:**
- Higher layers are checked first
- Mapping a key to itself bypasses lower layer remappings
- Keys not mapped in "gaming" fall through to "base"

## Troubleshooting

### Import Not Working

**Problem**: "Failed to parse Rhai file"

**Causes:**
- File contains advanced Rhai features (tap-hold, functions, variables)
- Syntax errors in the file
- File is not valid Rhai

**Solution:**
1. Open file in **Config Editor** first
2. Check validation errors
3. Simplify the configuration (remove advanced features)
4. Try importing again

### Mappings Not Appearing

**Problem**: Created a mapping but it doesn't show in Code Preview

**Causes:**
- Dropped key in wrong area
- Layer is not selected
- Browser bug (rare)

**Solution:**
1. Ensure correct layer is selected (blue highlight)
2. Drag completely to the target key (wait for drop highlight)
3. Refresh the page and try again

### Export File Is Empty

**Problem**: Downloaded `.rhai` file is empty or has no mappings

**Causes:**
- No mappings created yet
- Browser blocked download
- Code generation error

**Solution:**
1. Check that you have created at least one mapping
2. Look in Code Preview - if it shows code, export should work
3. Try using **[Copy Code]** button instead, paste into text editor
4. Save manually as `.rhai` file

### Virtual Keyboard Not Responding

**Problem**: Cannot drag keys or click buttons

**Causes:**
- JavaScript error (check browser console)
- Browser compatibility issue
- Page not fully loaded

**Solution:**
1. Press F12, check Console tab for errors
2. Refresh the page
3. Try a different browser (Chrome, Firefox, Edge recommended)
4. Clear browser cache

## Keyboard Shortcuts

| Shortcut | Action |
|----------|--------|
| `Ctrl+I` | Open Import dialog |
| `Ctrl+E` | Export configuration |
| `Ctrl+Shift+L` | Add new layer |
| `Ctrl+Shift+M` | Add new modifier |
| `Ctrl+Shift+C` | Copy code preview |
| `Delete` | Remove selected item (layer/modifier) |
| `F1` | Show help |

## Accessibility Features

The Visual Builder follows WCAG 2.1 Level AA standards:

- **Keyboard Navigation**: Full keyboard support, no mouse required
- **Screen Reader Support**: ARIA labels on all interactive elements
- **High Contrast**: Works with browser high-contrast modes
- **Focus Indicators**: Clear visual focus for keyboard navigation
- **Drag-and-Drop Alternative**: Keyboard-based drag-and-drop available

### Screen Reader Announcements

- "Key [name] dragged"
- "Dropped on key [name], mapping created"
- "Layer [name] selected"
- "Modifier [name] added"
- "Configuration exported successfully"

## Next Steps

After creating your configuration visually:

1. **Validate**: Open in **Config Editor** for full validation
2. **Test**: Use the **Simulator** to test key combinations
3. **Deploy**: Copy the `.rhai` file to your KeyRX config directory
4. **Iterate**: Reload in Visual Builder to make adjustments

## Related Documentation

- [Configuration Validation Guide](config-validation.md) - Validate and debug Rhai configs
- [WASM Simulation Guide](wasm-simulation.md) - Test configurations before deploying
- [Profile Management Guide](profile-management.md) - Manage multiple configurations
- [Macro Recorder Guide](macro-recorder.md) - Record and replay macro sequences

## Frequently Asked Questions

### Can I edit the generated code directly?

No, the Code Preview is read-only. To edit code:
1. Click **[Copy Code]** in the Code Preview
2. Open **Config Editor**
3. Paste and edit the code
4. Use Config Editor's validation and save features

### How do I create tap-hold actions visually?

Tap-hold actions are not supported in the Visual Builder. Workaround:
1. Build basic mappings visually
2. Export to Rhai file
3. Manually add tap-hold actions in a text editor
4. Load in Config Editor for validation

### Can I create macros visually?

Not directly. Use the **Macro Recorder** feature:
1. Record macro using Macro Recorder
2. Export macro as Rhai code
3. Import into Visual Builder (if simple)
4. Or integrate manually in Config Editor

### What's the difference between Visual Builder and Config Editor?

| Feature | Visual Builder | Config Editor |
|---------|---------------|---------------|
| Input Method | Drag-and-drop | Text editing |
| Code Visibility | Generated automatically | Direct editing |
| Features Supported | Basic mappings | All Rhai features |
| Learning Curve | Easy | Moderate |
| Best For | Simple configs, beginners | Advanced configs, power users |

**Recommendation**: Start with Visual Builder, graduate to Config Editor for advanced features.

### Can I use both Visual Builder and Config Editor together?

Yes! Common workflow:
1. Build structure in Visual Builder (layers, basic mappings)
2. Export to `.rhai` file
3. Open in Config Editor
4. Add advanced features (tap-hold, conditionals, functions)
5. Save and deploy

**Note**: Once you add advanced features, you cannot re-import into Visual Builder.
