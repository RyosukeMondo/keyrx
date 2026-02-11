# UI/UX Refinement Guide

## Introduction

The UI/UX refinement features provide an enhanced user experience for managing KeyRX keyboard configurations. These improvements include visual drag-and-drop configuration editing, auto-save functionality, persistent profile activation, and real-time device layout management.

### Key Features

- **Visual Configuration Editor**: Drag-and-drop interface for key assignment
- **Auto-Save**: Automatic persistence of changes with debouncing
- **Profile-Configuration Integration**: Seamless profile activation and compilation
- **Device Layout Persistence**: Remember layout preferences for each device
- **Active Profile Display**: Real-time indication of which profile is active
- **WASM Simulator Integration**: Test configurations before activation

## Visual Configuration Editor

The visual configuration editor provides an intuitive drag-and-drop interface for creating and modifying keyboard mappings without writing code.

### Accessing the Visual Editor

1. Open the KeyRX web UI (default: `http://localhost:5173`)
2. Navigate to **Profiles** page
3. Click on a profile name or select **Configure** from the profile menu
4. The configuration page opens with the **Visual Editor** tab active by default

Alternatively, navigate directly:
```
http://localhost:5173/config?profile=<profile-name>
```

### Editor Interface

The visual editor is divided into three main sections:

```
┌────────────────────────────────────────────────────────────────┐
│ Visual Configuration Editor                    [Saving...] [✓] │
├────────────────────────────────────────────────────────────────┤
│ [Visual Editor] [Code Editor]                                  │
├─────────────────────────────┬──────────────────────────────────┤
│ Device Scope                │ Key Assignment                    │
│ ○ Global  ⦿ Device-Specific │ [All] [Virtual Keys] [Modifiers] │
│ Device: [Logitech MX Keys▼] │ [Locks] [Layers] [Macros]        │
│                             │                                   │
│ Layer Selector              │ Search: [________]               │
│ Layer: [base ▼]             │                                  │
│                             │ [A] [B] [C] [Ctrl] [Shift]       │
│ Keyboard Layout             │ [Enter] [Space] [Tab]            │
│ ┌───────────────────────┐   │ [...draggable keys...]           │
│ │  [Q][W][E][R][T]...  │   │                                  │
│ │  [A][S][D][F][G]...  │   │                                  │
│ │  [Z][X][C][V][B]...  │   │                                  │
│ └───────────────────────┘   │                                  │
└─────────────────────────────┴──────────────────────────────────┘
```

#### Device Scope Toggle

Switch between global and device-specific configurations:

- **Global**: Mappings apply to all keyboards
- **Device-Specific**: Mappings apply only to the selected device

**To use:**
1. Click the **Device-Specific** radio button
2. Select a device from the dropdown menu
3. Configure mappings specific to that device
4. Switch back to **Global** for universal mappings

#### Layer Selector

Choose which layer to edit:

1. Click the **Layer** dropdown
2. Select from available layers:
   - `base` - Default layer
   - `nav` - Navigation layer
   - `num` - Number pad layer
   - `fn` - Function layer
   - `gaming` - Gaming layer
3. Mappings are created on the selected layer

#### Keyboard Visualizer

Interactive keyboard representation showing current mappings:

- **Click a key**: Opens the Key Assignment Popup for detailed configuration
- **Drop zone**: Accepts dragged keys from the Key Assignment Panel
- **Visual feedback**: Keys show their current mapping labels
- **Hover effects**: Highlight drop zones during drag operations

#### Key Assignment Panel

Categorized palette of assignable keys:

**Categories:**
- **All**: Show all available keys
- **Virtual Keys**: Standard keyboard keys (A-Z, 0-9, etc.)
- **Modifiers**: Ctrl, Shift, Alt, Super/Win
- **Locks**: Caps Lock, Num Lock, Scroll Lock
- **Layers**: Layer switching keys
- **Macros**: Recorded macro playback keys

**Features:**
- **Search**: Filter keys by name
- **Drag source**: Drag any key to the keyboard visualizer
- **Keyboard accessible**: Tab through keys, Enter/Space to select

### Creating Key Mappings

#### Method 1: Drag and Drop

1. Select the target layer from the Layer Selector
2. Find the key you want to assign in the Key Assignment Panel
   - Use category tabs to filter keys
   - Use the search box to find specific keys
3. Click and drag the key from the panel
4. Drop it onto the target key in the Keyboard Visualizer
5. The mapping is created immediately

**Example:**
```
Drag "Ctrl" from Modifiers → Drop on "CapsLock" key
Result: CapsLock now acts as Ctrl
```

#### Method 2: Click to Configure

1. Click any key in the Keyboard Visualizer
2. The Key Assignment Popup opens
3. Use tabs to select assignment type:
   - **Key**: Assign to another key
   - **Modifier**: Make this key a modifier
   - **Lock**: Assign a lock function
   - **Layer**: Assign layer switching
   - **Macro**: Assign a macro
4. Configure tap-hold behavior (optional):
   - **Tap**: Action when key is tapped quickly
   - **Hold**: Action when key is held down
5. Click **Save** to apply the mapping

**Example:**
```
Click on Space key
→ Select "Layer" tab
→ Tap: Space (keep normal behavior)
→ Hold: Activate Navigation Layer
→ Save
Result: Space when tapped, Nav layer when held
```

### Auto-Save Functionality

All changes are automatically saved with the following behavior:

- **Debounce Period**: 500ms (changes are batched)
- **Visual Feedback**:
  - `Saving...` - Save in progress
  - `✓ Saved 3:45:12 PM` - Save successful with timestamp
  - `✗ Save failed` - Save error occurred
- **Validation**: Configurations are validated before saving
- **Retry Logic**: Failed saves retry up to 3 times with exponential backoff

**Important Notes:**
- Changes persist immediately after the debounce period
- Invalid configurations prevent saving (validation errors shown)
- Auto-save only occurs when editing in Code mode
- Visual editor changes update immediately but sync to backend on tab switch

### Code Editor Mode

Switch between Visual and Code editing:

1. Click the **Code Editor** tab
2. Edit Rhai configuration directly with:
   - **Syntax Highlighting**: Rhai language support
   - **Auto-completion**: Smart code suggestions
   - **Error Detection**: Real-time validation
   - **Line Numbers**: Easy navigation
3. Press `Ctrl+S` to manually trigger save (or wait for auto-save)
4. Switch back to **Visual Editor** to see visual representation

**Keyboard Shortcuts (Code Mode):**
- `Ctrl+S`: Save configuration
- `Ctrl+Z`: Undo
- `Ctrl+Y` / `Ctrl+Shift+Z`: Redo
- `F8`: Navigate to next error
- `Ctrl+F`: Find
- `Ctrl+H`: Replace

## Profile-Configuration Integration

### Profile Activation Workflow

When you activate a profile, the following occurs automatically:

1. **Configuration Load**: `.rhai` source file is read
2. **Compilation**: Rhai code is compiled to `.krx` binary
3. **Validation**: Compiled binary is validated
4. **Persistence**: Active profile marker is saved to `~/.config/keyrx/active_profile.txt`
5. **Daemon Reload**: KeyRX daemon reloads with the new configuration
6. **UI Update**: Active profile indicator updates across all pages

### Activating a Profile

**From Profiles Page:**
1. Navigate to **Profiles** page
2. Find the profile you want to activate
3. Click the **Activate** button
4. Wait for activation to complete (usually < 1 second)
5. The **[Active]** badge appears next to the profile name

**From Configuration Page:**
- The currently edited profile is indicated in the breadcrumb
- Activate via the Profiles page (activation from config page not yet implemented)

### Active Profile Indicator

The active profile is displayed in multiple locations:

- **Profiles Page**: `[Active]` badge next to profile name
- **Metrics Page**: Header shows active profile name
- **Simulator Page**: Profile selector shows active profile
- **WebSocket Events**: Real-time updates when profile changes

### Compilation Error Handling

If compilation fails during activation:

1. **Error Display**: Toast notification with error details
2. **Line/Column Information**: Specific error location shown
3. **Rollback**: Previous active profile remains active
4. **Edit Prompt**: Link to edit the configuration

**Example Error:**
```
Failed to activate profile 'gaming'
Syntax error at line 12, column 5: unexpected token ']'
[Edit Configuration]
```

## Device Layout Persistence

### Auto-Save Device Layouts

The system remembers your preferred keyboard layout for each device:

**Supported Layouts:**
- ANSI 104
- ISO 105
- JIS 109
- HHKB
- Numpad

### Using Device Layouts

**From Devices Page:**
1. Navigate to **Devices** page
2. Find your connected keyboard
3. Select the layout from the dropdown
4. The selection saves automatically after 500ms
5. Visual feedback shows save status:
   - `💾 Saving...` - In progress
   - `✓ Saved` - Success (fades after 2s)
   - `✗ Failed` - Error

**Persistence:**
- Layouts are stored in `~/.config/keyrx/device_layouts.json`
- Survives daemon restarts
- Per-device (keyed by serial number)
- Automatically loaded on device connection

## WASM Simulator Integration

### Testing Configurations

Test keyboard configurations without activating them:

1. Navigate to **Simulator** page
2. Select a profile from the dropdown:
   ```
   Profile: [gaming ▼]
   ```
3. The WASM simulator loads the profile's `.krx` configuration
4. Click keys or press them on your keyboard to see mapped output
5. Verify mappings work as expected before activation

### Simulator Features

- **Real-time Simulation**: See key mappings in action
- **Layer Visualization**: See which layer is active
- **Modifier State**: Visual indication of active modifiers
- **Event Log**: See input and output events
- **No Side Effects**: Safe testing without affecting system

### Profile Switching in Simulator

1. Select different profiles from the dropdown
2. The simulator reloads the new configuration
3. Test multiple profiles without activating them
4. Compare behavior between profiles

## Metrics Page Integration

### Active Profile Display

The Metrics page shows the currently active profile:

**Header Section:**
```
┌─────────────────────────────────────────────┐
│ Active Profile: gaming                       │
│ Uptime: 2h 34m    Events: 1,234            │
└─────────────────────────────────────────────┘
```

**Real-time Updates:**
- Profile name updates via WebSocket when activated
- Shows "No Active Profile" when none is active
- Updates within 1 second of activation

## Best Practices

### Configuration Workflow

**Recommended workflow:**

1. **Create or Duplicate** a profile
2. **Edit Visually**: Use drag-and-drop for common mappings
3. **Fine-tune in Code**: Switch to Code mode for advanced features
4. **Validate**: Check for errors (auto-validated)
5. **Test in Simulator**: Verify behavior before activation
6. **Activate**: Apply to your keyboard
7. **Monitor**: Check Metrics page for any issues

### Visual Editor Tips

**Efficient Key Assignment:**
- Use category filters to quickly find keys
- Use search for specific keys (e.g., "ctrl", "layer")
- Drag multiple keys in sequence without switching tabs
- Click for complex assignments (tap-hold, macros)

**Layer Organization:**
- Start with `base` layer for primary mappings
- Use `nav` for arrow key alternatives (e.g., HJKL)
- Use `num` for numpad on compact keyboards
- Use `fn` for function keys and media controls
- Use `gaming` for game-specific layouts

**Device-Specific Configurations:**
- Use Global for mappings that work on all keyboards
- Use Device-Specific for keyboards with unique layouts
- Example: Numpad-only config for external numpad device

### Auto-Save Best Practices

**Code Editor:**
- Make small, incremental changes
- Wait for "Saved" confirmation before closing tab
- If you see errors, fix before navigating away
- Use `Ctrl+S` to force immediate save

**Visual Editor:**
- Changes apply immediately visually
- Backend sync occurs on tab switch or page navigation
- No need to manually save in visual mode

### Performance Tips

**Large Configurations:**
- Break complex configs into multiple layers
- Use device-specific scopes to reduce profile size
- Test in simulator before activating
- Monitor compilation time (should be < 1s)

**Multiple Devices:**
- Create separate profiles for different device types
- Use device layout persistence for quick switching
- Name profiles descriptively (e.g., `laptop-builtin`, `external-mech`)

## Troubleshooting

### Visual Editor Issues

**Keys won't drop onto keyboard**

**Symptoms:** Dragging a key doesn't create a mapping.

**Solutions:**
1. Ensure you're dropping on a valid key (not empty space)
2. Check browser console for JavaScript errors
3. Try refreshing the page
4. Use click-to-configure instead

**Key Assignment Popup won't open**

**Symptoms:** Clicking a key has no effect.

**Solutions:**
1. Ensure the keyboard visualizer is fully loaded
2. Check that you're clicking on an actual key (not spacing)
3. Try using drag-and-drop instead
4. Refresh the page if the issue persists

**Search filter not working**

**Symptoms:** Typing in search doesn't filter keys.

**Solutions:**
1. Clear the search box and try again
2. Refresh the page
3. Check that JavaScript is enabled

### Auto-Save Issues

**Changes not persisting**

**Symptoms:** Edits are lost after page refresh.

**Solutions:**
1. Wait for "Saved" confirmation before navigating away
2. Check for validation errors preventing save
3. Verify daemon is running: `systemctl status keyrx-daemon`
4. Check file permissions on `~/.config/keyrx/profiles/`

**"Save failed" error**

**Symptoms:** Red "✗ Save failed" message appears.

**Solutions:**
1. Check validation errors in the error panel
2. Switch to Code Editor to see specific errors
3. Verify disk space: `df -h ~/.config`
4. Check daemon logs: `journalctl -u keyrx-daemon -f`
5. Try manual save with `Ctrl+S`

**Auto-save too fast/slow**

**Note:** Auto-save debounce is fixed at 500ms for optimal UX.

**If needed:**
- Rapid changes are batched automatically
- Use `Ctrl+S` for immediate save in Code mode

### Profile Activation Issues

**Profile activation fails**

**Symptoms:** Clicking "Activate" shows error or has no effect.

**Solutions:**
1. Check error message for compilation errors
2. Edit the `.rhai` file to fix syntax errors
3. Validate configuration in Code Editor
4. Check daemon logs for detailed error information
5. Verify the profile directory exists and is readable

**Compilation errors**

**Symptoms:** Error message with line/column information.

**Solutions:**
1. Click "Edit Configuration" in the error toast
2. Navigate to the indicated line in Code Editor
3. Fix the syntax error (e.g., missing semicolon, bracket)
4. Wait for auto-save or press `Ctrl+S`
5. Try activating again

**Active profile not updating in UI**

**Symptoms:** Profile activates but UI doesn't update.

**Solutions:**
1. Refresh the page
2. Check WebSocket connection status
3. Verify daemon is broadcasting state updates
4. Check browser console for WebSocket errors

### Device Layout Issues

**Layout selection not saving**

**Symptoms:** Layout dropdown reverts after page refresh.

**Solutions:**
1. Wait for "✓ Saved" confirmation
2. Check daemon is running
3. Verify file permissions on `~/.config/keyrx/device_layouts.json`
4. Check browser console for API errors

**Wrong layout loads for device**

**Symptoms:** Device shows different layout than selected.

**Solutions:**
1. Verify the device serial number is correct
2. Re-select the correct layout and wait for save
3. Check `device_layouts.json` file for corruption:
   ```bash
   cat ~/.config/keyrx/device_layouts.json
   ```
4. Delete the file to reset all layouts (they'll need to be re-selected)

## API Reference

### Profile Configuration Endpoints

#### Get Profile Configuration
```http
GET /api/profiles/{name}/config
```

**Response:**
```json
{
  "name": "gaming",
  "source": "layer \"base\" {\n  map KEY_A to KEY_B;\n}\n",
  "compiled_at": 1703980800,
  "layer_count": 3
}
```

#### Update Profile Configuration
```http
PUT /api/profiles/{name}/config
Content-Type: application/json

{
  "source": "layer \"base\" {\n  map KEY_A to KEY_B;\n}\n"
}
```

**Response:** `200 OK` with compilation result

#### Activate Profile
```http
POST /api/profiles/{name}/activate
```

**Response:**
```json
{
  "success": true,
  "compilation_time_ms": 45,
  "profile_name": "gaming"
}
```

**Error Response (Compilation Failed):**
```json
{
  "success": false,
  "error": {
    "message": "Syntax error at line 12, column 5",
    "line": 12,
    "column": 5
  }
}
```

#### Get Active Profile
```http
GET /api/profiles/active
```

**Response:**
```json
{
  "active_profile": "gaming"
}
```

Or:
```json
{
  "active_profile": null
}
```

### Device Layout Endpoints

#### Save Device Layout
```http
PUT /api/devices/{serial}/layout
Content-Type: application/json

{
  "layout": "ANSI_104"
}
```

**Response:** `200 OK`

#### Get Device Layout
```http
GET /api/devices/{serial}/layout
```

**Response:**
```json
{
  "serial": "KB001",
  "layout": "ANSI_104"
}
```

**Valid layout values:** `ANSI_104`, `ISO_105`, `JIS_109`, `HHKB`, `NUMPAD`

### WebSocket Events

#### Daemon State Update
```json
{
  "type": "daemon-state",
  "data": {
    "uptime_secs": 9345,
    "total_events": 12456,
    "active_profile": "gaming"
  }
}
```

The `active_profile` field is included in all daemon-state events and updates in real-time when profiles are activated.

## File Locations

All UI/UX refinement features use the following file structure:

```
~/.config/keyrx/
├── profiles/
│   ├── gaming/
│   │   ├── config.rhai       # Editable source
│   │   └── config.krx        # Compiled binary
│   └── programming/
│       ├── config.rhai
│       └── config.krx
├── active_profile.txt         # Current active profile name
└── device_layouts.json        # Device layout preferences
```

**File Formats:**

`active_profile.txt`:
```
gaming
```

`device_layouts.json`:
```json
{
  "KB001": "ANSI_104",
  "KB002": "ISO_105"
}
```

## Related Features

- **Profile Management**: Create, duplicate, and delete profiles
- **Code Editor**: Advanced Rhai configuration editing
- **WASM Simulator**: Test configurations safely
- **Metrics Dashboard**: Monitor keyboard event metrics

## See Also

- [Profile Management](./profile-management.md) - Complete profile lifecycle documentation
- [WASM Simulation](./wasm-simulation.md) - Configuration testing guide
- [Configuration Validation](./config-validation.md) - Validation rules and error handling
