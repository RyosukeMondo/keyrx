# KeyRx UI

Web-based configuration interface for KeyRx keyboard remapper. Built with React, TypeScript, and Vite.

## Features

- **Bidirectional Rhai Sync**: Visual editor and code editor stay in sync in real-time
- **Device-Aware Configuration**: Configure global and device-specific mappings with multi-device selection
- **Visual Configuration Editor**: QMK-style drag-and-drop interface for keyboard remapping
- **Real-time Simulator**: Test key mappings with WASM-powered simulation
- **Profile Management**: Create, edit, and activate profiles with auto-generation
- **Device Management**: Monitor connected devices and configure device-specific mappings
- **Metrics Dashboard**: View keystroke statistics and latency metrics
- **Accessibility**: WCAG 2.2 Level AA compliant with full keyboard navigation support

## Architecture Overview

### Rhai-Driven Configuration

KeyRx UI is designed with the Rhai script as the **Single Source of Truth (SSOT)** for all configuration. The architecture ensures:

- **Bidirectional Sync**: Changes in the visual editor immediately update the Rhai script, and vice versa
- **Device-Aware**: Device scope (global vs device-specific) is determined by Rhai `device()` blocks, not UI toggles
- **Zero Configuration Drift**: Visual editor always reflects exactly what the Rhai script defines
- **Graceful Degradation**: If parsing fails, the visual editor shows the last valid state with error messages

### Key Components

```
┌─────────────────┐     ┌──────────────────┐     ┌─────────────────┐
│   Visual Editor │────>│ RhaiSyncEngine   │────>│   Code Editor   │
│  (KeyMapping[]) │<────│  (Orchestrator)  │<────│  (Rhai Script)  │
└─────────────────┘     └──────────────────┘     └─────────────────┘
                              │     │
                    ┌─────────┘     └─────────┐
                    ▼                         ▼
            ┌──────────────┐         ┌──────────────┐
            │ RhaiParser   │         │ RhaiCodeGen  │
            │ (parse AST)  │         │ (generate)   │
            └──────────────┘         └──────────────┘
                                            │
                                            ▼
                                    ┌──────────────┐
                                    │RhaiFormatter │
                                    │  (format)    │
                                    └──────────────┘
```

## Rhai Utilities

### RhaiParser

Parses Rhai scripts into structured AST for the visual editor.

**Location**: `src/utils/rhaiParser.ts`

**Key Functions**:

```typescript
import {
  parseRhaiScript,
  extractDevicePatterns,
  hasGlobalMappings,
  getMappingsForDevice,
  validateAST
} from '@/utils/rhaiParser';

// Parse Rhai script into AST
const result = parseRhaiScript(rhaiCode);
if (result.success) {
  const ast = result.ast;
  console.log('Global mappings:', ast.globalMappings);
  console.log('Device blocks:', ast.deviceBlocks);
} else {
  console.error('Parse error:', result.error);
  console.error('Line:', result.error.line);
  console.error('Suggestion:', result.error.suggestion);
}

// Extract device patterns from AST
const devices = extractDevicePatterns(ast);
// Returns: ['SERIAL_123', 'SERIAL_456']

// Check if AST has global mappings
const hasGlobal = hasGlobalMappings(ast);

// Get mappings for specific device
const mappings = getMappingsForDevice(ast, 'SERIAL_123');

// Validate AST structure
const errors = validateAST(ast);
if (errors.length > 0) {
  console.error('Validation errors:', errors);
}
```

**AST Structure**:

```typescript
interface RhaiAST {
  globalMappings: KeyMapping[];
  deviceBlocks: DeviceBlock[];
  comments: string[];
}

interface DeviceBlock {
  pattern: string;  // Device serial number or pattern
  mappings: KeyMapping[];
  layers?: LayerBlock[];
}

interface ParseError {
  line: number;
  column: number;
  message: string;
  suggestion: string;
}
```

**Supported Mapping Types**:

- **Simple**: `map(Key::A, Key::B);`
- **Tap-Hold**: `map(Key::CapsLock, tap_hold(Key::Escape, Key::LCtrl, 200));`
- **Macro**: `map(Key::F13, macro([Key::H, Key::E, Key::L, Key::L, Key::O]));`
- **Layer Switch**: `map(Key::Space, layer_switch("nav"));`

**Error Handling**:

Parse errors include line numbers and actionable suggestions:

```typescript
{
  line: 42,
  column: 15,
  message: "Expected ';' after map() call",
  suggestion: "Add semicolon at end of line"
}
```

### RhaiCodeGen

Generates clean, formatted Rhai code from visual editor state.

**Location**: `src/utils/rhaiCodeGen.ts`

**Key Functions**:

```typescript
import {
  generateRhaiScript,
  generateDeviceBlock,
  generateMapping,
  generateComment
} from '@/utils/rhaiCodeGen';

// Generate complete Rhai script
const mappings: KeyMapping[] = [
  { keyCode: 'A', type: 'simple', simple: 'VK_B' }
];
const rhaiCode = generateRhaiScript({
  globalMappings: mappings,
  deviceBlocks: [],
  comments: []
});

// Generate device-specific block
const deviceBlock = generateDeviceBlock({
  pattern: 'SERIAL_123',
  mappings: [
    { keyCode: 'CapsLock', type: 'simple', simple: 'VK_ESCAPE' }
  ]
});
// Output:
// device("SERIAL_123") {
//     map(Key::CapsLock, Key::Escape);
// }

// Generate individual mapping
const mapping = generateMapping({
  keyCode: 'CapsLock',
  type: 'tap_hold',
  tapHold: {
    tap: 'VK_ESCAPE',
    hold: 'MD_CTRL',
    timeoutMs: 200
  }
});
// Output: map(Key::CapsLock, tap_hold(Key::Escape, Key::LCtrl, 200));
```

**Code Generation Rules**:

1. **Indentation**: 4 spaces per level
2. **Grouping**: Device-specific mappings grouped in single `device()` block
3. **Comments**: Preserved from original Rhai (where possible)
4. **Formatting**: Consistent spacing, aligned brackets

**Performance**: <50ms for 1,000 mappings

### RhaiFormatter

Formats Rhai code with consistent style and indentation.

**Location**: `src/utils/rhaiFormatter.ts`

**Key Functions**:

```typescript
import {
  formatRhaiScript,
  applyDefaultFormatOptions
} from '@/utils/rhaiFormatter';

// Format with default options
const formatted = formatRhaiScript(rhaiCode);

// Format with custom options
const customFormatted = formatRhaiScript(rhaiCode, {
  indentSize: 2,          // Spaces per indent level (default: 4)
  maxLineLength: 100,     // Max line length (default: 80)
  preserveComments: true, // Keep comments (default: true)
  alignBrackets: true     // Align closing brackets (default: true)
});

// Apply default formatting options
const options = applyDefaultFormatOptions({ indentSize: 2 });
```

**Formatting Rules**:

- **Indentation**: 4 spaces (configurable)
- **Line Length**: Max 80 characters (configurable)
- **Comments**: Preserved at original positions
- **Brackets**: Aligned and consistently spaced
- **Blank Lines**: Removed excess, added between blocks

**Performance**: <50ms for 1,000 lines

### RhaiSyncEngine

Orchestrates bidirectional sync between visual and code editors.

**Location**: `src/components/RhaiSyncEngine.tsx`

**Usage**:

```typescript
import { useRhaiSyncEngine } from '@/components/RhaiSyncEngine';

function ConfigPage() {
  const {
    // State
    syncState,        // 'idle' | 'parsing' | 'generating' | 'syncing' | 'error'
    parseError,       // ParseError | null
    currentAST,       // RhaiAST | null
    currentCode,      // string
    visualMappings,   // KeyMapping[]

    // Actions
    onVisualChange,   // (mappings: KeyMapping[]) => void
    onCodeChange,     // (code: string) => void
    forceSync,        // () => void
    clearError,       // () => void

    // Getters
    getDevicePatterns,    // () => string[]
    hasGlobalMappings,    // () => boolean
    getMappingsForDevice  // (pattern: string) => KeyMapping[]
  } = useRhaiSyncEngine({
    initialCode: rhaiScript,
    debounceMs: 500,
    onSyncComplete: (ast) => console.log('Sync complete', ast),
    onError: (error) => console.error('Sync error', error)
  });

  return (
    <div>
      {/* Visual Editor */}
      <KeyboardVisualizer
        mappings={visualMappings}
        onMappingChange={onVisualChange}
      />

      {/* Code Editor */}
      <MonacoEditor
        value={currentCode}
        onChange={onCodeChange}
      />

      {/* Sync Status */}
      {syncState === 'parsing' && <Spinner>Parsing...</Spinner>}
      {syncState === 'generating' && <Spinner>Generating...</Spinner>}
      {parseError && (
        <ErrorBanner>
          Line {parseError.line}: {parseError.message}
          <br />
          Suggestion: {parseError.suggestion}
        </ErrorBanner>
      )}
    </div>
  );
}
```

**State Machine**:

```
idle ──(code change)──> parsing ──(success)──> syncing ──> idle
  │                         │
  │                      (error)
  │                         │
  └─────────────────────> error ──(clear)──> idle

idle ──(visual change)──> generating ──> syncing ──> idle
```

**Debouncing**: Code editor changes are debounced (default: 500ms) to prevent excessive parsing.

**Persistence**: Sync state is persisted to localStorage for recovery after page refresh.

**Error Handling**: Parse errors display the last valid state in the visual editor with a warning banner.

## Component Documentation

### Shared KeyConfig Components

Reusable components extracted from KeyConfigModal and KeyConfigPanel for DRY code organization.

**Location**: `src/components/keyConfig/`

#### MappingTypeSelector

Radio button/tab selector for choosing mapping types.

**Location**: `src/components/keyConfig/MappingTypeSelector.tsx`

**Props**:
- `selectedType: MappingType` - Currently selected type
- `onChange: (type: MappingType) => void` - Callback when type changes
- `supportedTypes: MappingType[]` - Available types (default: all 5 types)
- `layout?: 'horizontal' | 'vertical'` - Display layout (default: horizontal)

**Usage**:
```typescript
import { MappingTypeSelector } from '@/components/keyConfig/MappingTypeSelector';

<MappingTypeSelector
  selectedType="simple"
  onChange={(type) => setMappingType(type)}
  supportedTypes={['simple', 'tap_hold']}
  layout="horizontal"
/>
```

#### KeySelectionTabs

Tabbed interface for key selection (keyboard, modifier, lock, layer).

**Location**: `src/components/keyConfig/KeySelectionTabs.tsx`

**Props**:
- `activeTab: string` - Currently active tab
- `onTabChange: (tab: string) => void` - Callback when tab changes
- `availableTabs: string[]` - Tabs to display
- `onKeySelect: (keyCode: string) => void` - Callback when key is selected
- `layoutKeys?: SVGKey[]` - Optional keyboard layout keys

**Usage**:
```typescript
import { KeySelectionTabs } from '@/components/keyConfig/KeySelectionTabs';

<KeySelectionTabs
  activeTab="keyboard"
  onTabChange={setActiveTab}
  availableTabs={['keyboard', 'modifier', 'lock']}
  onKeySelect={handleKeySelect}
  layoutKeys={keyboardLayout}
/>
```

#### MappingConfigForm

Dynamic form that renders fields based on mapping type.

**Location**: `src/components/keyConfig/MappingConfigForm.tsx`

**Props**:
- `mappingType: MappingType` - Type of mapping being configured
- `currentConfig?: MappingConfig` - Current configuration values
- `onChange: (config: MappingConfig) => void` - Callback when config changes
- `onValidate?: (config: MappingConfig) => ValidationResult` - Optional validation
- `layoutKeys?: SVGKey[]` - Optional keyboard layout
- `enableKeyboardView?: boolean` - Enable keyboard/palette toggle (default: true)

**Mapping Types**:
- `simple`: Target key selection
- `modifier`: Modifier key selection
- `lock`: Lock key selection
- `tap_hold`: Tap key, hold key, and threshold input
- `layer_active`: Layer number input

**Usage**:
```typescript
import { MappingConfigForm } from '@/components/keyConfig/MappingConfigForm';

<MappingConfigForm
  mappingType="tap_hold"
  currentConfig={{ tapAction: 'Escape', holdAction: 'LCtrl', threshold: 200 }}
  onChange={handleConfigChange}
  onValidate={validateConfig}
/>
```

### Shared Metrics Components

Reusable components extracted from MetricsPage for modular metrics display.

**Location**: `src/components/metrics/`

#### MetricsStatsCards

Displays 4 stat cards (latency, throughput, CPU, memory).

**Location**: `src/components/metrics/MetricsStatsCards.tsx`

**Props**:
- `latencyStats: LatencyStats` - Latency statistics (avg, min, max, p95, p99)
- `eventCount: number` - Total events processed
- `connected: boolean` - Connection status

**Usage**:
```typescript
import { MetricsStatsCards } from '@/components/metrics/MetricsStatsCards';

<MetricsStatsCards
  latencyStats={{ avgUs: 1234, minUs: 500, maxUs: 5000, p95Us: 2000, p99Us: 3000 }}
  eventCount={42}
  connected={true}
/>
```

#### LatencyChart

Line chart for latency over time using Recharts.

**Location**: `src/components/metrics/LatencyChart.tsx`

**Props**:
- `data: LatencyDataPoint[]` - Array of {timestamp, latencyUs} data points
- `maxDataPoints?: number` - Maximum points to display (default: 60)

**Usage**:
```typescript
import { LatencyChart } from '@/components/metrics/LatencyChart';

<LatencyChart
  data={latencyHistory}
  maxDataPoints={100}
/>
```

#### EventLogList

Virtualized event log list using react-window.

**Location**: `src/components/metrics/EventLogList.tsx`

**Props**:
- `events: EventLogEntry[]` - Array of event log entries
- `maxEvents?: number` - Maximum events to display

**Usage**:
```typescript
import { EventLogList } from '@/components/metrics/EventLogList';

<EventLogList
  events={eventLog}
  maxEvents={1000}
/>
```

#### StateSnapshot

Displays current daemon state (layer, modifiers, locks).

**Location**: `src/components/metrics/StateSnapshot.tsx`

**Props**:
- `state: StateSnapshot` - Current state snapshot from daemon

**Usage**:
```typescript
import { StateSnapshot } from '@/components/metrics/StateSnapshot';

<StateSnapshot
  state={{
    activeLayer: 'base',
    activeModifiers: ['Ctrl', 'Shift'],
    activeLocks: [],
    tapHoldTimers: 2,
    queuedEvents: 0
  }}
/>
```

### Refactored ConfigPage Components (Custom Hooks & Containers)

The ConfigPage has been refactored to use custom hooks and container components for better separation of concerns and reusability.

#### Custom Hooks

**useProfileSelection** - Profile selection with fallback priority
- **Location**: `src/hooks/useProfileSelection.ts`
- **Purpose**: Centralizes profile selection logic with automatic fallback
- **Priority**: manual > prop > route > query > active > 'Default'
- **Returns**: `{ selectedProfileName, setSelectedProfileName }`

```typescript
import { useProfileSelection } from '@/hooks/useProfileSelection';

const { selectedProfileName, setSelectedProfileName } = useProfileSelection();
```

**useCodePanel** - Code panel state with localStorage persistence
- **Location**: `src/hooks/useCodePanel.ts`
- **Purpose**: Manages collapsible code panel UI state and height
- **Persists**: Panel height to localStorage ('codePanel.height')
- **Returns**: `{ isOpen, height, toggleOpen, setHeight }`

```typescript
import { useCodePanel } from '@/hooks/useCodePanel';

const { isOpen, height, toggleOpen } = useCodePanel();
```

**useKeyboardLayout** - Keyboard layout state with memoized keys
- **Location**: `src/hooks/useKeyboardLayout.ts`
- **Purpose**: Manages layout selection and provides memoized parsed layout keys
- **Memoization**: Layout keys recalculate only when layout changes
- **Returns**: `{ layout, setLayout, layoutKeys }`

```typescript
import { useKeyboardLayout } from '@/hooks/useKeyboardLayout';

const { layout, setLayout, layoutKeys } = useKeyboardLayout('ANSI_104');
```

**useConfigSync** - RhaiSyncEngine initialization and sync status
- **Location**: `src/hooks/useConfigSync.ts`
- **Purpose**: Encapsulates config synchronization logic
- **Debounce**: 500ms for code editor changes
- **Returns**: `{ syncEngine, syncStatus, lastSaveTime, setSyncStatus, setLastSaveTime }`

```typescript
import { useConfigSync } from '@/hooks/useConfigSync';

const { syncEngine, syncStatus, setSyncStatus } = useConfigSync(profileName);
```

#### Container Components

**KeyboardVisualizerContainer** - Keyboard visualization with layout selector
- **Location**: `src/components/config/KeyboardVisualizerContainer.tsx`
- **Purpose**: Wraps KeyboardVisualizer with layout management
- **Features**: Layout dropdown, key click handling, mapping display

```typescript
import { KeyboardVisualizerContainer } from '@/components/config/KeyboardVisualizerContainer';

<KeyboardVisualizerContainer
  profileName={profileName}
  activeLayer={activeLayer}
  mappings={keyMappings}
  onKeyClick={handleKeyClick}
  selectedKeyCode={selectedKey}
  initialLayout="ANSI_104"
/>
```

**CodePanelContainer** - Collapsible code editor panel
- **Location**: `src/components/config/CodePanelContainer.tsx`
- **Purpose**: Collapsible panel with draggable resize handle
- **Features**: Toggle button, height persistence, sync status indicators, parse error display

```typescript
import { CodePanelContainer } from '@/components/config/CodePanelContainer';

<CodePanelContainer
  profileName={profileName}
  rhaiCode={code}
  onChange={handleCodeChange}
  syncEngine={syncEngine}
/>
```

**ConfigurationPanel** - Unified configuration controls
- **Location**: `src/components/config/ConfigurationPanel.tsx`
- **Purpose**: Composes configuration UI components into unified panel
- **Contains**: DeviceSelector, LayerSwitcher, KeyPalette, KeyConfigPanel, CurrentMappingsSummary

```typescript
import { ConfigurationPanel } from '@/components/config/ConfigurationPanel';

<ConfigurationPanel
  profileName={profileName}
  selectedPhysicalKey={selectedKey}
  selectedPaletteKey={paletteKey}
  onPaletteKeySelect={handlePaletteSelect}
  onSaveMapping={handleSave}
  onClearMapping={handleClear}
  activeLayer={activeLayer}
  availableLayers={layers}
  onLayerChange={handleLayerChange}
  devices={devices}
  selectedDevices={selectedDevices}
  globalSelected={globalSelected}
  onDeviceSelectionChange={handleDeviceChange}
  keyMappings={mappings}
  onEditMapping={handleEdit}
  layoutKeys={layoutKeys}
/>
```

**ProfileSelector** - Profile selection dropdown with create button
- **Location**: `src/components/config/ProfileSelector.tsx`
- **Purpose**: Profile dropdown with loading states and creation
- **Features**: Profile dropdown, loading indicator, create profile button

```typescript
import { ProfileSelector } from '@/components/config/ProfileSelector';

<ProfileSelector
  value={selectedProfileName}
  onChange={handleProfileChange}
  profiles={profiles}
  isLoading={isLoadingProfiles}
  disabled={!connected}
/>
```

**ConfigurationLayout** - Responsive grid layout container
- **Location**: `src/components/config/ConfigurationLayout.tsx`
- **Purpose**: Provides responsive layout structure for config page
- **Features**: CSS Grid layout, responsive breakpoints, resizable panels

```typescript
import { ConfigurationLayout } from '@/components/config/ConfigurationLayout';

<ConfigurationLayout profileName={profileName}>
  <KeyboardVisualizerContainer {...props} />
  <ConfigurationPanel {...props} />
  <CodePanelContainer {...props} />
</ConfigurationLayout>
```

### DeviceSelector

Multi-device selector with global and device-specific checkboxes.

**Location**: `src/components/DeviceSelector.tsx`

**Usage**:

```typescript
import { DeviceSelector } from '@/components/DeviceSelector';

function ConfigPage() {
  const [selectedDevices, setSelectedDevices] = useState<string[]>([]);
  const [showGlobal, setShowGlobal] = useState(true);

  return (
    <DeviceSelector
      devices={connectedDevices}
      selectedDevices={selectedDevices}
      showGlobal={showGlobal}
      onSelectionChange={(devices, global) => {
        setSelectedDevices(devices);
        setShowGlobal(global);
      }}
      disconnectedDevices={['SERIAL_DISCONNECTED']}
    />
  );
}
```

**Props**:

- `devices: Device[]` - List of connected devices
- `selectedDevices: string[]` - Selected device serials
- `showGlobal: boolean` - Whether global is selected
- `onSelectionChange: (devices: string[], global: boolean) => void` - Callback for changes
- `disconnectedDevices?: string[]` - Devices from Rhai that are disconnected (shows badge)

**Features**:

- **Multi-device checkboxes**: Select multiple devices to configure simultaneously
- **Global checkbox**: Independent global configuration option
- **Connection status badges**: Green for connected, gray for disconnected
- **WCAG 2.2 compliant**: Keyboard navigation, ARIA labels, screen reader support
- **No scope toggle**: Scope determined by Rhai script, not UI

**Accessibility**:

- `Tab`: Navigate between checkboxes
- `Space`: Toggle checkbox
- `Enter`: Toggle checkbox
- Screen readers announce device names, connection status, and selection state

### DevicesPage

Manages device metadata (name, layout) and global layout settings.

**Location**: `src/pages/DevicesPage.tsx`

**Features**:

- **Global Settings Card**: Set default keyboard layout (ANSI 104, ISO 105, JIS 109, HHKB, NUMPAD)
- **Device List**: Display all detected devices with connection status
- **Device Editing**: Rename devices and override global layout
- **Auto-save**: Changes persist automatically to daemon
- **No scope UI**: Scope field completely removed

**Global Layout API**:

- `GET /api/settings/global-layout` - Get current global layout
- `PUT /api/settings/global-layout` - Set global layout (persists to daemon)

**Device Inheritance**:

- New devices inherit the global layout by default
- Device-specific layout overrides the global setting
- Changing global layout does not affect existing device overrides

### ProfilesPage

Manages profiles with Rhai file path display and auto-generation.

**Location**: `src/pages/ProfilesPage.tsx`

**Features**:

- **Rhai File Path Display**: Each profile card shows the Rhai script path
- **Path Tooltip**: Hover to see full absolute path
- **Click to Edit**: Click path to navigate to Config page
- **Auto-Generate Default**: First-time users get a "default" profile automatically
- **Profile CRUD**: Create, edit, activate, delete profiles

**Auto-Generation**:

When the profile list is empty on first load:

1. System creates "default" profile with "blank" template
2. Profile is activated automatically
3. Notification shown: "Default profile created. Click 'Edit' to customize."
4. Error handling with retry button if daemon is offline

### ConfigPage

Device-aware configuration with bidirectional Rhai sync.

**Location**: `src/pages/ConfigPage.tsx`

**Features**:

- **Bidirectional Sync**: Visual editor ↔ Rhai script in real-time
- **Multi-Device Display**: Show global and multiple device keyboards side-by-side
- **Rhai-Driven Detection**: Device selector auto-populated from Rhai `device()` blocks
- **Parse Error Display**: Shows line numbers, error messages, and suggestions
- **Tab Switching**: Preserves sync state when switching between visual and code tabs
- **Device Filtering**: Mappings filtered by selected devices

**User Workflows**:

**Workflow 1: Load → Edit Visual → Switch to Code → Save**

1. Load profile: Rhai parsed, visual editor populated
2. Edit in visual: Mappings updated, Rhai regenerated immediately
3. Switch to code tab: See generated Rhai script
4. Save: Persists to daemon

**Workflow 2: Multi-Device Configuration**

1. Select "Global" and "Device A" in device selector
2. Both keyboards display side-by-side
3. Add mapping to "Device A" keyboard
4. Rhai script generates `device("SERIAL_A") { ... }` block
5. Save: Device-specific configuration persists

**Workflow 3: Rhai-Driven Device Detection**

1. Load profile with `device("SERIAL_123") { ... }` blocks
2. Device selector auto-populated with "SERIAL_123"
3. If device not connected, shows "disconnected" badge
4. User can still edit mappings for disconnected device

## Drag-and-Drop Configuration

The visual configuration editor provides an intuitive interface for creating key mappings without writing code.

### How to Use

1. **Navigate to Configuration Page**: Click "Config" in the navigation menu
2. **Select Profile**: Choose a profile from the dropdown in the header
3. **Select Devices**: Check "Global" or specific devices in the device selector
4. **Select Layer**: Choose which layer to edit (base, nav, num, fn)
5. **Drag Keys**: Drag keys from the palette on the left onto the keyboard visualizer
6. **Configure Mapping**: Click a mapped key to open the configuration dialog
7. **Save**: Changes auto-save to the daemon and generate Rhai script

### Keyboard Accessibility

The drag-and-drop interface is fully keyboard-accessible following the Salesforce Lightning pattern:

- **Tab**: Move focus between draggable items and drop zones
- **Space**: Grab a focused item (press again to drop)
- **Arrow Keys**: Navigate between drop zones while dragging
- **Escape**: Cancel the current drag operation
- **Enter**: Open configuration dialog for a mapped key

Screen readers will announce drag state and provide instructions for keyboard users.

### Mapping Types

The configuration editor supports four types of key mappings:

#### 1. Simple Mapping
Map a physical key directly to a virtual key, modifier, or lock.

**Example**: Remap CapsLock to Escape
```typescript
{
  keyCode: "CapsLock",
  type: "simple",
  simple: "VK_ESCAPE"
}
```

#### 2. Tap-Hold (Dual Function)
Different actions for tap vs. hold, with configurable timeout.

**Example**: CapsLock as Escape on tap, Ctrl on hold
```typescript
{
  keyCode: "CapsLock",
  type: "tap_hold",
  tapHold: {
    tap: "VK_ESCAPE",
    hold: "MD_CTRL",
    timeoutMs: 200
  }
}
```

#### 3. Macro Sequence
Execute a sequence of key presses with one key.

**Example**: Type "hello" with a single key
```typescript
{
  keyCode: "F13",
  type: "macro",
  macro: ["VK_H", "VK_E", "VK_L", "VK_L", "VK_O"]
}
```

#### 4. Layer Switch
Switch to a different layer when key is pressed.

**Example**: Access navigation layer while holding key
```typescript
{
  keyCode: "Space",
  type: "layer_switch",
  layer: "nav"
}
```

### Component Documentation

#### DragKeyPalette

Displays a palette of draggable keys organized by category (Virtual Keys, Modifiers, Locks, Layers).

```tsx
import { DragKeyPalette } from '@/components/config/DragKeyPalette';

<DragKeyPalette
  onDragStart={(key) => console.log('Started dragging', key.id)}
  onDragEnd={() => console.log('Drag ended')}
  filterCategory="vk"  // Optional: filter by category
/>
```

**Props:**
- `onDragStart?: (key: AssignableKey) => void` - Callback when drag starts
- `onDragEnd?: () => void` - Callback when drag ends
- `filterCategory?: string` - Filter by category (vk, modifier, lock, layer, macro)
- `className?: string` - Additional CSS classes

#### KeyMappingDialog

Modal dialog for configuring individual key mappings with form validation.

```tsx
import { KeyMappingDialog } from '@/components/config/KeyMappingDialog';

<KeyMappingDialog
  open={isOpen}
  onClose={() => setIsOpen(false)}
  keyCode="CapsLock"
  currentMapping={existingMapping}
  onSave={async (mapping) => {
    await api.saveMapping(mapping);
    setIsOpen(false);
  }}
/>
```

**Props:**
- `open: boolean` - Whether dialog is visible
- `onClose: () => void` - Callback to close dialog
- `keyCode: string` - Physical key being configured
- `currentMapping?: KeyMapping` - Existing mapping (for editing)
- `onSave: (mapping: KeyMapping) => Promise<void>` - Callback with new mapping

#### ProfileHeader

Displays profile context in the configuration page header.

```tsx
import { ProfileHeader } from '@/components/config/ProfileHeader';

<ProfileHeader
  profileName="my-profile"
  isActive={true}
  lastModified={new Date()}
  onProfileChange={(name) => navigate(`/config?profile=${name}`)}
  availableProfiles={['default', 'my-profile', 'gaming']}
/>
```

**Props:**
- `profileName: string` - Name of current profile
- `isActive?: boolean` - Whether profile is active in daemon
- `lastModified?: Date` - Last modification timestamp
- `onProfileChange?: (name: string) => void` - Callback to switch profiles
- `availableProfiles?: string[]` - List of available profiles

#### useDragAndDrop Hook

Custom hook for managing drag-and-drop state and operations.

```tsx
import { useDragAndDrop } from '@/hooks/useDragAndDrop';

function ConfigPage() {
  const { activeDragKey, handleDragStart, handleDragEnd, handleKeyDrop } =
    useDragAndDrop({ profileName: 'default', selectedLayer: 'base' });

  return (
    <DndContext onDragStart={handleDragStart} onDragEnd={handleDragEnd}>
      <DragKeyPalette />
      <KeyboardVisualizer onKeyDrop={handleKeyDrop} />
    </DndContext>
  );
}
```

**Parameters:**
- `profileName: string` - Profile to save mappings to
- `selectedLayer: string` - Active layer for mappings

**Returns:**
- `activeDragKey: AssignableKey | null` - Currently dragged key
- `handleDragStart: (event: DragStartEvent) => void` - Drag start handler
- `handleDragEnd: (event: DragEndEvent) => void` - Drag end handler
- `handleKeyDrop: (keyCode: string, key: AssignableKey) => Promise<void>` - Drop handler
- `isSaving: boolean` - Whether save is in progress

### Troubleshooting

#### Visual editor not syncing with code

- **Check sync state**: Look for "Parsing..." or "Generating..." indicators
- **Parse error**: Check for error banner with line number and suggestion
- **Fix**: Correct syntax error in code editor, or use "Force Sync" button
- **Debounce delay**: Wait 500ms after typing in code editor for sync to trigger

#### Device selector not showing devices from Rhai

- **Check Rhai syntax**: Ensure `device("SERIAL")` blocks are correctly formatted
- **Verify parsing**: Switch to code editor tab and check for parse errors
- **Disconnected devices**: Devices in Rhai that aren't connected show gray badge
- **Fix**: Correct `device()` block syntax or wait for device to connect

#### Types out of sync error in CI

- **Cause**: Rust structs modified but TypeScript types not regenerated
- **Fix**: Run `scripts/check-types.sh --fix` and commit generated.ts
- **Prevention**: Pre-commit hook should catch this locally

#### API validation errors

- **Check browser console**: Zod validation errors logged with endpoint and details
- **Unexpected fields**: Warnings about extra fields (not errors) - usually safe to ignore
- **Type mismatch**: Update frontend to match new backend types (check generated.ts)

#### Global layout not applying to new devices

- **Verify save**: Check for success indicator after changing global layout
- **Check daemon**: Ensure daemon is running and connected
- **API call**: Look for PUT /api/settings/global-layout in DevTools Network tab
- **Persistence**: Setting stored in daemon's settings.json file

#### Auto-generate default profile failed

- **Daemon offline**: Check if daemon is running (connection indicator in UI)
- **Retry**: Click the "Retry" button in error notification
- **Manual creation**: Create profile manually from Profiles page if auto-generate fails

#### Drag-and-drop not working

- **Check browser compatibility**: @dnd-kit requires a modern browser with Pointer Events support
- **Verify daemon connection**: Drag-and-drop saves require WebSocket connection to daemon
- **Check console for errors**: Open browser DevTools and check for error messages

#### Changes not saving

- **Verify WebSocket connection**: Check the connection indicator in the top-right corner
- **Check network requests**: Look for failed PUT /api/config requests in DevTools Network tab
- **Verify profile permissions**: Ensure you have write access to the profile directory

#### Keyboard navigation not working

- **Focus not visible**: Ensure your browser/OS allows focus indicators (check system accessibility settings)
- **Tab order incorrect**: Report as bug - keyboard navigation should follow visual layout
- **Space key not working**: Ensure focus is on a draggable item before pressing Space

## Development

### Prerequisites

- Node.js 18+
- npm 9+

### Setup

```bash
cd keyrx_ui
npm install
```

### Available Scripts

- `npm run dev` - Start development server with HMR
- `npm run build` - Build for production
- `npm run preview` - Preview production build
- `npm test` - Run unit tests
- `npm run test:coverage` - Generate coverage report
- `npm run test:a11y` - Run accessibility tests
- `npm run lint` - Run ESLint
- `npm run type-check` - Run TypeScript compiler check

### Project Structure

```
keyrx_ui/
├── src/
│   ├── api/              # API client functions
│   │   ├── schemas.ts    # Zod validation schemas
│   │   └── contracts.test.ts  # API contract tests
│   ├── components/       # Reusable UI components
│   │   ├── config/       # Configuration editor components
│   │   ├── DeviceSelector.tsx  # Multi-device selector
│   │   └── RhaiSyncEngine.tsx  # Bidirectional sync orchestrator
│   ├── contexts/         # React contexts (API, theme)
│   ├── hooks/            # Custom React hooks
│   │   ├── useDevices.ts       # Device management
│   │   ├── useProfiles.ts      # Profile management
│   │   └── useProfileConfig.ts # Configuration management
│   ├── pages/            # Top-level page components
│   │   ├── ConfigPage.tsx      # Configuration editor
│   │   ├── DevicesPage.tsx     # Device management
│   │   └── ProfilesPage.tsx    # Profile management
│   ├── services/         # Business logic services
│   │   └── ConfigStorage.ts    # Storage abstraction
│   ├── types/            # TypeScript type definitions
│   │   └── generated.ts  # Auto-generated from Rust (typeshare)
│   ├── utils/            # Utility functions
│   │   ├── rhaiParser.ts       # Rhai parsing (AST generation)
│   │   ├── rhaiCodeGen.ts      # Rhai code generation
│   │   ├── rhaiFormatter.ts    # Rhai code formatting
│   │   ├── timeFormatting.ts   # Time utilities
│   │   └── keyCodeMapping.ts   # Key code translation
│   └── App.tsx           # Root component
├── tests/                # Test utilities
│   └── testUtils.tsx     # Shared test infrastructure
├── e2e/                  # Playwright E2E tests
└── public/               # Static assets
```

### Testing Approach

**Philosophy**: All utilities have comprehensive unit tests with ≥90% coverage. Components have integration tests validating user workflows.

**Key Test Files**:

- `src/utils/rhaiParser.test.ts` - 39 tests for Rhai parsing (100% coverage)
- `src/utils/rhaiCodeGen.test.ts` - 35 tests for code generation (97.6% line coverage)
- `src/utils/rhaiFormatter.test.ts` - 43 tests for formatting (100% statement coverage)
- `src/components/RhaiSyncEngine.test.tsx` - 22 tests for sync engine (all state transitions)
- `src/components/DeviceSelector.test.tsx` - 41 tests for device selector (100% coverage)
- `src/pages/DevicesPage.test.tsx` - 41 tests for devices page (100% pass rate)
- `src/pages/ProfilesPage.test.tsx` - 36 tests for profiles page (100% pass rate)
- `src/pages/ConfigPage.test.tsx` - 28 tests for config page integration

**Round-Trip Testing**: Parser and code generator are tested together to ensure:
```typescript
const original = generateRhaiScript(ast1);
const parsed = parseRhaiScript(original);
const regenerated = generateRhaiScript(parsed.ast);
expect(regenerated).toEqual(original); // Round-trip consistency
```

### Testing

The project uses a multi-layered testing approach with separate test categories optimized for different purposes:

#### Test Categories

**Unit Tests** (Fast, <5 seconds)
- Isolated component and function tests
- MSW-based WebSocket mocking
- Use for testing individual components in isolation
- Run by default with `npm test`

**Integration Tests** (Medium, <30 seconds)
- Multi-component interactions
- Full page testing with routing
- WebSocket state management
- Use for testing component interactions and data flow

**E2E Tests** (Slow, <3 minutes)
- Full application workflows
- Real browser automation with Playwright
- Use for critical user journeys

**Accessibility Tests**
- WCAG 2.2 Level AA compliance
- Automated axe-core scanning
- Run before every commit

#### Quick Start

**Run unit tests (default):**
```bash
npm test
```

**Run all test categories:**
```bash
npm run test:all
```

**Run with coverage:**
```bash
npm run test:coverage
```

#### Test Commands Reference

| Command | Purpose | When to Use |
|---------|---------|-------------|
| `npm test` | Run unit tests | Before every commit (fast feedback) |
| `npm run test:watch` | Watch mode for unit tests | During development |
| `npm run test:unit` | Explicit unit tests | Same as `npm test` |
| `npm run test:integration` | Run integration tests | Before pushing changes |
| `npm run test:integration:watch` | Watch mode for integration | Integration test development |
| `npm run test:e2e` | Run E2E tests | Before releasing |
| `npm run test:e2e:ui` | E2E tests with Playwright UI | Debugging E2E failures |
| `npm run test:a11y` | Run accessibility tests | Before committing UI changes |
| `npm run test:coverage` | Generate coverage report | Check coverage thresholds |
| `npm run test:all` | Run all test categories | Final verification before merge |
| `npm run test:shard` | Run tests in shards for CI | CI parallel execution |
| `npm run test:integration:shard` | Run integration tests in shards | CI parallel execution |

#### Focused Test Run Scripts

For faster development iteration, use these focused test commands:

| Command | Purpose | When to Use |
|---------|---------|-------------|
| `npm run test:changed` | Run tests for changed files only | Quick feedback during active development |
| `npm run test:related` | Run tests related to changed files | After modifying shared utilities or components |
| `npm run test:failed` | Interactive watch mode with verbose output | Fixing failing tests with instant feedback |
| `npm run test:watch:smart` | Watch mode with smart filtering | Continuous testing while editing specific files |

**How they work:**

- **test:changed**: Uses git diff to detect modified files and only runs tests in those files
- **test:related**: Finds all test files that import the specified modules and runs them (requires file path argument)
- **test:failed**: Runs in watch mode with verbose output and stops on first failure, allowing quick iteration on fixing test failures
- **test:watch:smart**: Combines watch mode with changed file detection for instant feedback

**Example workflow:**

```bash
# Make changes to a component
vim src/components/ProfileCard.tsx

# Run only tests for changed files (fast)
npm run test:changed

# Fix failing tests
vim src/components/ProfileCard.test.tsx

# Re-run only the failed tests (watch mode for quick iteration)
npm run test:failed

# Once all pass, run tests related to specific file to check for side effects
npm run test:related -- src/components/ProfileCard.tsx

# Before committing, run full test suite
npm test
```

**Performance benefits:**
- **test:changed**: ~90% faster than full suite (only runs affected tests)
- **test:failed**: Stops on first failure for focused debugging, watch mode provides instant re-test on save
- **test:watch:smart**: Instant feedback (<1s) on file save

#### Parallel Test Execution

Tests run in parallel using Vitest's thread pool for faster execution:

- **Automatic thread optimization**: Uses 75% of available CPU cores (9 threads on a 12-core machine)
- **Thread pool**: Configured in `vitest.config.base.ts`
- **Shard support**: Tests can be split across multiple CI jobs for faster pipelines

**Local parallel execution** (automatic):
```bash
npm test  # Runs on 9 threads (75% of 12 cores)
```

**CI sharded execution** (manual control):
```bash
# Split tests into 3 shards, run shard 1
SHARD_INDEX=1 SHARD_COUNT=3 npm run test:shard

# Split integration tests into 2 shards, run shard 2
SHARD_INDEX=2 SHARD_COUNT=2 npm run test:integration:shard
```

**Performance benefits**:
- Local development: ~30% faster test execution with parallel threads
- CI/CD: Can split tests across multiple jobs to reduce total pipeline time

#### Test Infrastructure

The project uses MSW (Mock Service Worker) for WebSocket mocking in tests:

- **No fake timers**: Tests use real async operations with `waitFor()`
- **Automatic connection**: MSW WebSocket handlers connect automatically
- **State isolation**: WebSocket state resets between tests
- **Type-safe helpers**: `setDaemonState()`, `sendLatencyUpdate()`, `sendKeyEvent()`

**Example unit test:**
```typescript
import { render, screen, waitFor } from '@testing-library/react';
import { setDaemonState } from '@/test/mocks/websocketHelpers';
import { ActiveProfileCard } from './ActiveProfileCard';

test('displays active profile from WebSocket', async () => {
  render(<ActiveProfileCard />);

  setDaemonState({ activeProfile: 'gaming' });

  await waitFor(() => {
    expect(screen.getByText('gaming')).toBeInTheDocument();
  });
});
```

#### Coverage Requirements

- Overall: ≥80% line and branch coverage
- Critical components: ≥90% coverage
- New code: Must include tests before merge

Coverage reports are generated in `coverage/` directory:
```bash
npm run test:coverage
open coverage/index.html  # View HTML report
```

#### Detailed Guides

For comprehensive testing documentation, see:
- [Unit Testing Guide](./docs/testing/unit-testing-guide.md) - Unit test patterns and MSW WebSocket usage
- [Integration Testing Guide](./docs/testing/integration-testing-guide.md) - Full page testing and multi-component interactions

All tests must pass before merging. See `.github/workflows/ci.yml` for CI enforcement.

## Type Safety Infrastructure

### Rust-TypeScript Interface

TypeScript types are **automatically generated** from Rust structs using [typeshare](https://github.com/1Password/typeshare).

**How It Works**:

1. Annotate Rust structs with `#[typeshare]` in `keyrx_daemon`
2. Run `cargo typeshare` to generate `src/types/generated.ts`
3. Frontend imports types from `generated.ts`
4. CI/CD verifies types are in sync

**Example**:

```rust
// Rust (keyrx_daemon/src/web/handlers/devices.rs)
#[derive(Serialize, Deserialize, Debug, Clone)]
#[typeshare]
pub struct DeviceEntry {
    pub serial: String,
    pub name: String,
    pub layout: LayoutPreset,
}
```

```typescript
// Generated TypeScript (src/types/generated.ts)
export interface DeviceEntry {
  serial: string;
  name: string;
  layout: LayoutPreset;
}
```

### Runtime Validation

All API responses are validated at runtime using [Zod](https://zod.dev/).

**Location**: `src/api/schemas.ts`

**Example**:

```typescript
import { z } from 'zod';
import { validateApiResponse } from '@/api/schemas';

const DeviceEntrySchema = z.object({
  serial: z.string(),
  name: z.string(),
  layout: z.enum(['ANSI_104', 'ISO_105', 'JIS_109', 'HHKB', 'NUMPAD'])
});

// Validate API response
const response = await fetch('/api/devices');
const data = await response.json();
const validated = validateApiResponse(DeviceEntrySchema, data, '/api/devices');
// Throws descriptive error if validation fails
```

**Error Handling**:

- **Validation failures**: Throw error with endpoint name and detailed message
- **Unexpected fields**: Log as warning (not error) to allow backward compatibility
- **Type mismatches**: Clear error messages with expected vs actual types

### Contract Tests

API contracts are tested on both frontend and backend.

**Frontend**: `src/api/contracts.test.ts` (40 tests)
- Validates all REST endpoints with Zod schemas
- Validates WebSocket RPC messages
- 100% endpoint coverage

**Backend**: `keyrx_daemon/tests/api_contracts_test.rs` (25 tests)
- Tests request/response structures
- Validates request validation rules
- Tests error responses (HTTP 400 for invalid input)

### Pre-Commit Type Check

A pre-commit hook ensures TypeScript types stay in sync with Rust structs.

**Script**: `scripts/check-types.sh`

**How It Works**:

1. Regenerates TypeScript types from Rust
2. Checks for differences in `generated.ts`
3. Fails commit if types are out of sync
4. Provides clear error with diff

**Fix out-of-sync types**:

```bash
# Regenerate and commit
scripts/check-types.sh --fix
git add keyrx_ui/src/types/generated.ts
git commit
```

### CI/CD Type Check

GitHub Actions verifies type consistency on every push.

**Workflow**: `.github/workflows/ci.yml` (type-check job)

**Checks**:

- Regenerates TypeScript types from Rust
- Runs `git diff` to detect changes
- Fails if types are out of sync
- Runs TypeScript compilation (`npm run type-check`)

**If types are out of sync**, CI fails with:
```
Error: TypeScript types are out of sync with Rust structs
Diff:
+ layout: LayoutPreset;  // Added in Rust
```

## Technology Stack

- **React 18**: UI framework
- **TypeScript 5**: Type safety
- **Vite**: Build tool and dev server
- **@dnd-kit**: Drag-and-drop library
- **@tanstack/react-query**: Data fetching and caching
- **React Router**: Client-side routing
- **Tailwind CSS**: Utility-first styling
- **Vitest**: Unit testing
- **Playwright**: E2E testing
- **axe-core**: Accessibility testing

## React Compiler

The React Compiler is not enabled on this template because of its impact on dev & build performances. To add it, see [this documentation](https://react.dev/learn/react-compiler/installation).

## Expanding the ESLint configuration

If you are developing a production application, we recommend updating the configuration to enable type-aware lint rules:

```js
export default defineConfig([
  globalIgnores(['dist']),
  {
    files: ['**/*.{ts,tsx}'],
    extends: [
      // Other configs...

      // Remove tseslint.configs.recommended and replace with this
      tseslint.configs.recommendedTypeChecked,
      // Alternatively, use this for stricter rules
      tseslint.configs.strictTypeChecked,
      // Optionally, add this for stylistic rules
      tseslint.configs.stylisticTypeChecked,

      // Other configs...
    ],
    languageOptions: {
      parserOptions: {
        project: ['./tsconfig.node.json', './tsconfig.app.json'],
        tsconfigRootDir: import.meta.dirname,
      },
      // other options...
    },
  },
])
```

You can also install [eslint-plugin-react-x](https://github.com/Rel1cx/eslint-react/tree/main/packages/plugins/eslint-plugin-react-x) and [eslint-plugin-react-dom](https://github.com/Rel1cx/eslint-react/tree/main/packages/plugins/eslint-plugin-react-dom) for React-specific lint rules:

```js
// eslint.config.js
import reactX from 'eslint-plugin-react-x'
import reactDom from 'eslint-plugin-react-dom'

export default defineConfig([
  globalIgnores(['dist']),
  {
    files: ['**/*.{ts,tsx}'],
    extends: [
      // Other configs...
      // Enable lint rules for React
      reactX.configs['recommended-typescript'],
      // Enable lint rules for React DOM
      reactDom.configs.recommended,
    ],
    languageOptions: {
      parserOptions: {
        project: ['./tsconfig.node.json', './tsconfig.app.json'],
        tsconfigRootDir: import.meta.dirname,
      },
      // other options...
    },
  },
])
```
