# KeyFlow User Guide

KeyFlow is a visual flow editor that helps you understand and manage how your keyboard devices connect to hardware profiles and keymaps in KeyRx. This guide will help you get started and make the most of KeyFlow's features.

## Table of Contents

1. [What is KeyFlow?](#what-is-keyflow)
2. [Understanding the Visualization](#understanding-the-visualization)
3. [Node Types](#node-types)
4. [Edge Meanings](#edge-meanings)
5. [Managing Slots](#managing-slots)
6. [Navigation and Interaction](#navigation-and-interaction)
7. [Keyboard Shortcuts](#keyboard-shortcuts)
8. [Troubleshooting](#troubleshooting)

---

## What is KeyFlow?

KeyFlow provides a visual representation of your KeyRx configuration, showing:

- **Connected devices** and their current state
- **Hardware profiles** that define keyboard layouts
- **Keymaps** that contain your key bindings
- **Slots** that connect devices to profiles and keymaps
- **Priority order** for fallback configurations

Think of KeyFlow as a "wiring diagram" for your keyboard configuration. It shows how everything connects together and helps you understand which keymap will be used when you plug in a device.

### Key Concepts

**Device**: A physical keyboard connected to your computer (e.g., "Keychron Q1")

**Hardware Profile**: Defines the physical layout of a keyboard (number of keys, key positions)

**Keymap**: Your custom key bindings and layers for a specific keyboard

**Slot**: A connection between a device and a profile+keymap pair, with a priority number

**Priority**: Determines which slot is active when multiple slots exist (lower number = higher priority)

---

## Understanding the Visualization

The KeyFlow canvas displays three columns of nodes connected by edges:

```
┌─────────────┐         ┌──────────────┐         ┌─────────────┐
│   Devices   │────────▶│   Profiles   │────────▶│   Keymaps   │
│  (Left)     │         │   (Middle)   │         │   (Right)   │
└─────────────┘         └──────────────┘         └─────────────┘
```

### Layout

- **Left column**: Device nodes (your physical keyboards)
- **Middle column**: Hardware profile nodes (layout definitions)
- **Right column**: Keymap nodes (your key bindings)
- **Edges (connections)**: Show slot relationships between nodes

### Colors and Visual Encoding

**Device Nodes**:
- Green: Connected and active
- Blue: Connected but inactive
- Red: Error state
- Grey: Disconnected

**Profile Nodes**:
- Blue: Valid profile
- Red: Invalid or missing profile

**Keymap Nodes**:
- Green: Valid keymap
- Red: Compilation failed or invalid

**Edges (Lines)**:
- Solid line: Active slot (currently in use)
- Dashed line: Inactive slot (fallback)
- Green: Valid connection
- Red: Invalid connection (missing profile/keymap)
- Yellow: Warning (potential issue)
- Priority badge: Number showing slot priority

---

## Node Types

### Device Nodes

Device nodes represent physical keyboards connected to your computer.

**Information displayed**:
- Device name (e.g., "Keychron Q1")
- VID/PID (Vendor ID / Product ID)
- Serial number
- Connection status (connected/disconnected)
- Active slot count / total slot count
- Error indicator (if any issues exist)

**Status indicators**:
- Checkmark icon: Device connected and working
- Warning icon: Connection issue
- Error icon: Critical error

**Tooltips**: Hover over a device node to see detailed information including VID/PID/serial.

### Hardware Profile Nodes

Profile nodes represent keyboard layout definitions.

**Information displayed**:
- Profile name
- Layout type (e.g., "ANSI 60%", "ISO TKL")
- Validation status

**Status indicators**:
- Blue border: Valid profile
- Red border: Invalid profile (missing or corrupted)

**Tooltips**: Hover to see profile details including layout type and key count.

### Keymap Nodes

Keymap nodes represent your custom key bindings.

**Information displayed**:
- Keymap name
- Layer count
- Compilation status

**Status indicators**:
- Green border: Valid keymap
- Red border: Compilation failed or invalid

**Tooltips**: Hover to see keymap details including number of layers.

---

## Edge Meanings

Edges (the connecting lines) represent **slots** - the connections between devices and their configurations.

### Edge Visual Properties

**Line Style**:
- **Solid line**: This slot is currently active (in use)
- **Dashed line**: This slot is inactive (fallback configuration)

**Color**:
- **Green**: Connection is valid, profile and keymap exist
- **Red**: Invalid connection (profile or keymap missing)
- **Yellow**: Warning (potential configuration issue)

**Priority Badge**:
- Number shown on the edge indicates priority (1 = highest priority)
- Lower priority slots act as fallbacks

### Understanding Slot Priority

When a device has multiple slots:

1. The slot with **priority 1** is tried first
2. If that fails, priority 2 is tried, then 3, etc.
3. Only one slot can be **active** at a time (shown with solid line)
4. Other slots remain as fallbacks (shown with dashed lines)

**Example**:
```
Device "Keychron Q1" has two slots:
- Slot 1 (Priority 1): Ergo-60 profile + Custom keymap → Active (solid green line)
- Slot 2 (Priority 2): Default profile + Base keymap → Fallback (dashed green line)
```

If Slot 1 fails (e.g., profile deleted), the device automatically falls back to Slot 2.

---

## Managing Slots

KeyFlow allows full CRUD (Create, Read, Update, Delete) operations on device slots.

### Adding a New Slot

1. Click the **"+"** button on a device node
2. In the dialog, select:
   - **Hardware Profile** from the dropdown
   - **Keymap** from the dropdown
3. Click **"Add Slot"**
4. The new slot is automatically assigned the next available priority
5. Changes are saved automatically

**Validation**:
- You cannot create duplicate slot combinations (same profile + keymap)
- Both profile and keymap must exist

### Deleting a Slot

**Method 1: Context menu**
1. Right-click on an edge (slot connection)
2. Select **"Delete Slot"** from the context menu
3. Confirm deletion in the dialog

**Method 2: Keyboard**
1. Click on an edge to select it
2. Press **Delete** key
3. Confirm deletion in the dialog

**Note**: Deleting a slot removes the connection permanently. Changes are saved automatically.

### Reordering Slots (Changing Priority)

1. Click and drag a **priority badge** on an edge
2. Drag it to the desired position
3. Other priorities automatically adjust
4. Release to confirm the new order
5. Changes are saved automatically

**Example**: To make a fallback slot the primary:
- Drag its priority badge to position 1
- The previous priority 1 becomes priority 2

### Toggling Slot Active Status

1. Right-click on an edge
2. Select **"Toggle Active/Inactive"**
3. The edge style changes to reflect new status
4. Changes are saved automatically

**Note**: Typically only one slot per device should be active at a time. The system automatically manages this based on priority and availability.

---

## Navigation and Interaction

### Canvas Navigation

**Pan (move the view)**:
- Click and drag on empty canvas space
- Two-finger drag on trackpad
- Middle mouse button drag

**Zoom**:
- Mouse wheel / trackpad pinch
- Ctrl + Plus: Zoom in
- Ctrl + Minus: Zoom out
- Ctrl + 0: Reset zoom to 100%

**Fit to view**:
- Zoom and pan are constrained to keep content visible
- Canvas automatically expands to fit all nodes

### Moving Nodes

**Mouse**:
1. Click and drag a node to move it
2. Release to drop in new position
3. Position is saved automatically
4. Nodes are constrained within canvas bounds

**Keyboard**:
1. Press Tab to focus a node
2. Use arrow keys to move (10px per press):
   - Up Arrow: Move up
   - Down Arrow: Move down
   - Left Arrow: Move left
   - Right Arrow: Move right
3. Position updates are saved automatically

**Reset Layout**:
- To reset all nodes to default positions, use the reset button in the toolbar (if available)
- Default layout: Devices on left, profiles in middle, keymaps on right, evenly spaced

### Selecting Elements

**Nodes**:
- Click on a node to focus it (keyboard navigation)
- Focused node shows a visible focus indicator

**Edges**:
- Click on an edge to select it
- Selected edge shows thicker border and different color
- Selected edge can be deleted with Delete key

**Clear selection**:
- Press Escape key
- Click on empty canvas space

### Hover Effects

**Nodes**:
- Hover to see detailed tooltip with additional information
- Tooltip includes VID/PID for devices, layout type for profiles, layer count for keymaps

**Edges**:
- Hover to see thicker line for easier visibility
- Shows priority and status information

---

## Keyboard Shortcuts

KeyFlow is fully keyboard-accessible. All features can be accessed without a mouse.

### Navigation

| Shortcut | Action |
|----------|--------|
| Tab | Navigate to next node |
| Shift+Tab | Navigate to previous node |
| Up/Down/Left/Right | Move focused node by 10 pixels |

### Actions

| Shortcut | Action |
|----------|--------|
| Enter | Open node details dialog |
| Delete | Delete selected edge (with confirmation) |
| Escape | Clear selection / deselect |

### Zoom

| Shortcut | Action |
|----------|--------|
| Ctrl+0 | Reset zoom to 100% |
| Ctrl+Plus (=) | Zoom in by 10% |
| Ctrl+Minus (-) | Zoom out by 10% |

### Help

| Shortcut | Action |
|----------|--------|
| Ctrl+? | Show keyboard shortcuts help dialog |

**Accessibility Note**: All interactive elements have keyboard focus indicators and screen reader labels. KeyFlow meets WCAG 2.1 AA accessibility standards.

---

## Troubleshooting

### Empty State

**Symptom**: Canvas shows "Connect a device to get started"

**Cause**: No devices are currently registered in KeyRx

**Solution**:
1. Connect a supported keyboard to your computer
2. KeyRx should automatically detect and register it
3. Refresh the KeyFlow page if needed
4. Check the Devices page to verify device detection

### Disconnected Devices

**Symptom**: Canvas shows "No connected devices, waiting for connection..."

**Cause**: Devices exist in configuration but are not currently connected

**Solution**:
1. Plug in one or more keyboards
2. Wait a few seconds for device detection
3. Nodes should appear automatically (fade-in animation)
4. If not, check USB connection and device compatibility

### Invalid Edges (Red Lines)

**Symptom**: Red edges connecting nodes

**Cause**: Slot references a profile or keymap that doesn't exist

**Solution**:
1. Click on the red edge to see error tooltip
2. Common issues:
   - **Profile deleted**: Create a new profile or delete the slot
   - **Keymap deleted**: Create a new keymap or delete the slot
   - **Name mismatch**: Check profile/keymap IDs match
3. Delete the invalid slot and create a new valid one

### Validation Errors

**Symptom**: Red exclamation badge on a node

**Cause**: Node has configuration errors

**Solution**:
1. Click on the validation badge to see error details
2. Read the error message and suggested fix
3. Common errors:
   - **Missing profile**: "Hardware profile 'ergo-60' not found. Create it in Profiles page."
   - **Orphaned slot**: "Slot references non-existent keymap"
   - **Priority conflict**: "Multiple slots with same priority"
4. Follow the suggested fix in the error message
5. Navigate to the appropriate page to fix the issue

### Loading Errors

**Symptom**: Error view shows "Failed to load configuration"

**Cause**: FFI bridge connection failed or runtime config inaccessible

**Solution**:
1. Click the **"Retry"** button
2. If error persists:
   - Check FFI bridge is running (see FFI_DEBUGGING.md)
   - Verify runtime config file exists and is valid
   - Check application logs for detailed error messages
   - Restart the application

### Slow Performance

**Symptom**: Canvas feels sluggish, animations stutter

**Cause**: Too many nodes, slow rendering, or memory issues

**Solution**:
1. Check number of devices and slots (KeyFlow is optimized for up to 50 devices)
2. Close other resource-intensive applications
3. Reset zoom to 100% (Ctrl+0)
4. Check DevTools for performance issues (see PROFILING.md)
5. If performance persists, reduce number of devices or slots

### Auto-Save Issues

**Symptom**: Changes not persisting, save indicator shows error

**Cause**: FFI call failed, permission issue, or backend error

**Solution**:
1. Check the save indicator in bottom-right corner:
   - Spinner: Saving in progress
   - Checkmark: Saved successfully
   - Error icon: Save failed
2. If save failed, click retry or refresh the page
3. Check application logs for error details
4. Verify backend is running and accessible
5. Check file permissions for configuration storage

### Position Reset

**Symptom**: Node positions reset after closing and reopening

**Cause**: SharedPreferences storage failure or browser cache cleared

**Solution**:
1. Node positions should persist automatically
2. If resetting, check browser local storage is enabled
3. Check for browser privacy modes that clear storage
4. Manually arrange nodes and verify save indicator confirms save
5. Check application logs for storage errors

### Screen Reader Issues

**Symptom**: Screen reader not announcing elements or changes

**Cause**: Semantic labels missing or screen reader compatibility issue

**Solution**:
1. Enable screen reader (TalkBack on Android, VoiceOver on iOS/macOS)
2. Navigate with Tab key to hear node labels
3. Each node should announce: name, type, status
4. Changes should be announced (e.g., "Node moved to new position")
5. If issues persist, check ACCESSIBILITY.md for known issues

### Common Error Messages

**"Hardware profile 'xxx' not found. Create it in Profiles page."**
- The slot references a profile that doesn't exist
- Go to Profiles page and create the missing profile, or delete the slot

**"Keymap 'xxx' compilation failed."**
- The keymap has syntax errors or invalid configuration
- Go to Keymaps page and fix the keymap, or create a new one

**"FFI bridge connection failed. Retry?"**
- Backend connection lost or FFI initialization failed
- Click Retry or restart the application

**"Failed to save changes. Retrying..."**
- Temporary save failure, system is retrying automatically
- Wait for retry or manual retry if error persists

---

## Tips and Best Practices

### Organizing Your Canvas

1. **Use consistent layout**: Keep devices on left, profiles in middle, keymaps on right
2. **Group related items**: Place related profiles/keymaps close together
3. **Minimize edge crossings**: Arrange nodes to reduce overlapping edges
4. **Use zoom effectively**: Zoom in for detailed work, zoom out for overview
5. **Reset when needed**: If layout gets messy, reset to default and rearrange

### Managing Slots Effectively

1. **Keep priority 1 as your primary**: Ensure your main configuration is priority 1
2. **Create fallbacks**: Add priority 2+ slots for backup configurations
3. **Avoid duplicates**: Don't create multiple slots with same profile+keymap
4. **Clean up unused slots**: Delete slots you no longer need
5. **Test changes**: After reordering priorities, verify the correct slot is active

### Performance Optimization

1. **Limit device count**: KeyFlow performs best with <50 devices
2. **Reduce slot count**: Keep slots per device reasonable (3-5 is ideal)
3. **Close unused tabs**: Free up browser resources
4. **Use virtualization**: KeyFlow automatically virtualizes off-screen nodes
5. **Profile when needed**: Use PROFILING.md guide to diagnose issues

### Accessibility

1. **Use keyboard navigation**: Tab through nodes, arrow keys to move
2. **Enable focus indicators**: Clear visual indicators show focused elements
3. **Use high contrast**: Colors meet WCAG AA standards
4. **Enlarge touch targets**: 48x48px minimum for all interactive elements
5. **Enable screen reader**: Full semantic labels for all elements

---

## Getting Help

If you encounter issues not covered in this guide:

1. **Check application logs**: See LOGGING.md for log locations
2. **Review FFI debugging**: See FFI_DEBUGGING.md for bridge issues
3. **Profile performance**: See PROFILING.md for performance analysis
4. **Check issue tracker**: Search for known issues
5. **Report bugs**: Include logs, screenshots, and reproduction steps

---

## Advanced Features

### Real-Time Updates

KeyFlow automatically updates when:
- Devices connect or disconnect (fade-in/fade-out animations)
- Configuration changes in other parts of the app
- Slots are added, deleted, or modified
- Hardware profiles or keymaps are created/deleted

**Latency**: Updates appear within 100ms of the event.

### Validation System

KeyFlow includes a comprehensive validation system:
- **Real-time validation**: Errors detected immediately
- **Node-level errors**: Badges on nodes with issues
- **Edge-level errors**: Red edges for invalid connections
- **Actionable messages**: Clear error text with suggested fixes
- **Suggested actions**: Links to fix issues in other pages

### Auto-Save

All changes are saved automatically:
- **Debouncing**: Saves occur 500ms after last change
- **Optimistic updates**: UI updates immediately, rollback on failure
- **Retry logic**: Exponential backoff on save failures (max 3 retries)
- **Status indicator**: Bottom-right corner shows save status
- **Conflict resolution**: Last write wins in concurrent scenarios

### Performance Features

KeyFlow is optimized for smooth 60fps performance:
- **Virtualization**: Only renders visible nodes (+margin)
- **RepaintBoundary**: Isolates repaints per node
- **Custom RenderObject**: Efficient edge rendering
- **Path caching**: Bezier curves cached for performance
- **Selective rebuilds**: Only affected widgets rebuild on changes
- **Target**: <16ms per frame with 250+ nodes

---

## Glossary

**Active Slot**: The slot currently in use for a device (solid line)

**Canvas**: The interactive drawing area where nodes and edges are displayed

**Edge**: A connection line between nodes representing a slot

**Fallback Slot**: An inactive slot that serves as backup (dashed line)

**FFI Bridge**: The interface between the Dart UI and Rust backend

**Hardware Profile**: A keyboard layout definition (key positions, count)

**Keymap**: A set of key bindings and layer configurations

**Node**: A visual element representing a device, profile, or keymap

**Priority**: Numeric order determining which slot is tried first (1 = highest)

**Runtime Config**: The current configuration state including all devices and slots

**Slot**: A connection between a device and a profile+keymap pair

**Validation**: Automatic checking of configuration for errors

**Virtualization**: Rendering optimization that skips off-screen elements

---

## Version Information

This guide is for KeyFlow version 1.0 (part of KeyRx).

For the latest updates and changes, check the release notes and changelog.

Last updated: 2025-12-15
