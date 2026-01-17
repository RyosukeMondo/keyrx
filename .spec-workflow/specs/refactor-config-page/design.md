# Design: Refactor ConfigPage Component

## Architecture Overview

### Current State
```
ConfigPage (940 lines, 892-line function)
├── Profile management logic
├── Code panel state
├── Sync engine management
├── Keyboard visualization
├── Key palette integration
├── Configuration panel
├── Device management
├── Layer management
└── Save/load operations
```

### Target Architecture
```
ConfigPage (orchestrator, <200 lines)
├── ProfileSelector (handles profile selection/creation)
├── ConfigurationLayout (main layout container)
│   ├── KeyboardVisualizerContainer (keyboard display + interaction)
│   ├── ConfigurationPanel (right sidebar)
│   │   ├── DeviceSelector
│   │   ├── LayerSwitcher
│   │   ├── CurrentMappingsSummary
│   │   └── KeyConfigPanel
│   └── CodePanelContainer (bottom panel, collapsible)
└── Custom Hooks
    ├── useProfileSelection
    ├── useCodePanel
    ├── useKeyboardLayout
    └── useConfigSync
```

## Component Breakdown

### 1. ConfigPage (Orchestrator)
**Responsibility**: Top-level composition and coordination

**State**: Minimal - delegates to children
- Current profile name
- Global UI state (if any)

**Props**:
```typescript
interface ConfigPageProps {
  profileName?: string;
}
```

**Key Changes**:
- Remove all business logic
- Delegate to child components
- Coordinate between children via props/callbacks
- Target: <200 lines, <50 line functions

### 2. ProfileSelector Component
**Responsibility**: Profile selection and creation UI

**File**: `src/components/config/ProfileSelector.tsx`

**State**:
- Selected profile name
- Profile creation modal state

**Props**:
```typescript
interface ProfileSelectorProps {
  value: string;
  onChange: (profileName: string) => void;
  profiles: Profile[];
  isLoading?: boolean;
  onCreateProfile: () => Promise<void>;
}
```

**Dependencies**:
- `useProfiles` hook
- `useCreateProfile` hook
- `useActiveProfileQuery` hook

### 3. ConfigurationLayout Component
**Responsibility**: Main layout structure (keyboard + config panel + code panel)

**File**: `src/components/config/ConfigurationLayout.tsx`

**Props**:
```typescript
interface ConfigurationLayoutProps {
  profileName: string;
  children: React.ReactNode;
}
```

**Features**:
- Responsive layout grid
- Resizable panels
- Collapsible code panel

### 4. KeyboardVisualizerContainer Component
**Responsibility**: Keyboard display and interaction

**File**: `src/components/config/KeyboardVisualizerContainer.tsx`

**State**:
- Selected physical key
- Keyboard layout type
- Layout keys (memoized)

**Props**:
```typescript
interface KeyboardVisualizerContainerProps {
  profileName: string;
  activeLayer: string;
  mappings: KeyMapping[];
  onKeyClick: (keyCode: string) => void;
  selectedKeyCode?: string;
}
```

**Hook**: `useKeyboardLayout`
```typescript
function useKeyboardLayout(initialLayout: LayoutType) {
  const [layout, setLayout] = useState(initialLayout);
  const layoutKeys = useMemo(() => parseLayoutKeys(layout), [layout]);
  return { layout, setLayout, layoutKeys };
}
```

### 5. ConfigurationPanel Component
**Responsibility**: Right sidebar with config controls

**File**: `src/components/config/ConfigurationPanel.tsx`

**Composition**:
- DeviceSelector (existing)
- LayerSwitcher (existing)
- CurrentMappingsSummary (existing)
- KeyConfigPanel (existing)
- KeyPalette (existing)

**Props**:
```typescript
interface ConfigurationPanelProps {
  profileName: string;
  selectedPhysicalKey: string | null;
  selectedPaletteKey: PaletteKey | null;
  onPaletteKeySelect: (key: PaletteKey) => void;
  onSaveMapping: (mapping: KeyMapping) => void;
  onClearMapping: (keyCode: string) => void;
}
```

### 6. CodePanelContainer Component
**Responsibility**: Collapsible code editor panel

**File**: `src/components/config/CodePanelContainer.tsx`

**State**:
- Is open
- Panel height
- Code content

**Props**:
```typescript
interface CodePanelContainerProps {
  profileName: string;
  rhaiCode: string;
  onChange: (code: string) => void;
  syncEngine: RhaiSyncEngine;
}
```

**Hook**: `useCodePanel`
```typescript
function useCodePanel(initialHeight: number = 300) {
  const [isOpen, setIsOpen] = useState(false);
  const [height, setHeight] = useState(initialHeight);

  const toggleOpen = useCallback(() => setIsOpen(prev => !prev), []);
  const setHeightWithPersistence = useCallback((h: number) => {
    setHeight(h);
    localStorage.setItem('codePanel.height', String(h));
  }, []);

  return { isOpen, height, toggleOpen, setHeight: setHeightWithPersistence };
}
```

### 7. useProfileSelection Hook
**Responsibility**: Profile selection logic with fallbacks

**File**: `src/hooks/useProfileSelection.ts`

**API**:
```typescript
function useProfileSelection(options: {
  profileNameProp?: string;
  profileNameRoute?: string;
  profileNameQuery?: string;
}) {
  // Logic for: manual > prop > route > query > active > 'Default'
  const [manualSelection, setManualSelection] = useState<string | null>(null);
  const { data: activeProfileName } = useActiveProfileQuery();

  const selectedProfileName = manualSelection
    ?? options.profileNameProp
    ?? options.profileNameRoute
    ?? options.profileNameQuery
    ?? activeProfileName
    ?? 'Default';

  return { selectedProfileName, setSelectedProfileName: setManualSelection };
}
```

### 8. useConfigSync Hook
**Responsibility**: Encapsulate sync engine management

**File**: `src/hooks/useConfigSync.ts`

**API**:
```typescript
function useConfigSync(profileName: string) {
  const syncEngine = useRhaiSyncEngine({
    storageKey: `profile-${profileName}`,
    debounceMs: 500,
    onStateChange: (state) => console.debug('Sync state:', state),
    onError: (error, direction) => console.error('Sync error:', { error, direction }),
  });

  const [syncStatus, setSyncStatus] = useState<'saved' | 'unsaved' | 'saving'>('saved');
  const [lastSaveTime, setLastSaveTime] = useState<Date | null>(null);

  return { syncEngine, syncStatus, lastSaveTime, setSyncStatus, setLastSaveTime };
}
```

## Data Flow

### Profile Selection Flow
```
User selects profile
  → ProfileSelector.onChange
  → ConfigPage.setSelectedProfile
  → ProfileSelector, KeyboardVisualizerContainer, ConfigurationPanel (via props)
```

### Key Mapping Flow
```
User clicks physical key
  → KeyboardVisualizerContainer.onKeyClick
  → ConfigPage updates selectedPhysicalKey
  → ConfigurationPanel receives selectedPhysicalKey prop
  → KeyConfigPanel displays mapping UI

User selects palette key
  → KeyPalette.onChange
  → ConfigurationPanel.onPaletteKeySelect
  → ConfigPage updates selectedPaletteKey
  → ConfigurationPanel.onSaveMapping
  → configStore.setMapping
  → syncEngine.syncToRhai
```

### Code Sync Flow
```
Visual changes (mappings)
  → configStore.setMapping
  → syncEngine.syncToRhai
  → CodePanelContainer receives updated rhaiCode

Code editor changes
  → MonacoEditor.onChange
  → syncEngine.syncFromRhai
  → configStore updates
  → KeyboardVisualizerContainer re-renders with new mappings
```

## Technical Decisions

### Decision 1: Custom Hooks vs Context
**Choice**: Custom hooks for local state, existing configStore for shared state
**Rationale**:
- configStore already exists and works well
- Custom hooks reduce coupling for profile/layout/panel state
- Avoid context provider hell

### Decision 2: Container vs Presentational Pattern
**Choice**: Containers handle state/logic, existing components remain presentational
**Rationale**:
- Minimal changes to existing working components
- Clear separation of concerns
- Testability improvements

### Decision 3: PropDrilling vs Context
**Choice**: Accept some prop drilling for clarity
**Rationale**:
- Component tree is shallow (2-3 levels max)
- Explicit props improve code readability
- Easier to track data flow

### Decision 4: File Organization
**Choice**: Create `src/components/config/` directory for config-page-specific components
**Rationale**:
- Clear namespace
- Related components grouped
- Doesn't pollute main components directory

## Migration Strategy

### Phase 1: Extract Hooks (Non-Breaking)
1. Create `useProfileSelection` hook
2. Create `useCodePanel` hook
3. Create `useKeyboardLayout` hook
4. Create `useConfigSync` hook
5. Update ConfigPage to use hooks
6. Test: existing tests should pass

### Phase 2: Extract Containers (Non-Breaking)
1. Create `KeyboardVisualizerContainer`
2. Create `CodePanelContainer`
3. Create `ConfigurationPanel`
4. Update ConfigPage to use containers
5. Test: existing tests should pass

### Phase 3: Extract ProfileSelector
1. Create `ProfileSelector` component
2. Update ConfigPage to use ProfileSelector
3. Test: existing tests should pass

### Phase 4: Extract ConfigurationLayout
1. Create `ConfigurationLayout` component
2. Update ConfigPage to use ConfigurationLayout
3. Test: existing tests should pass

### Phase 5: Cleanup
1. Remove unused code from ConfigPage
2. Verify ConfigPage is <200 lines
3. Add/update tests for each component
4. Update documentation

## Testing Strategy

### Unit Tests
- Each hook: test state management and side effects
- Each container: test with mocked hooks and child components
- ProfileSelector: test profile selection and creation

### Integration Tests
- ConfigPage composition: verify all components work together
- Data flow: test prop passing and callback execution
- Sync engine: test visual→code and code→visual sync

### Regression Tests
- Existing ConfigPage tests must pass
- E2E tests must pass without modification

## File Structure
```
src/
├── components/
│   └── config/
│       ├── ProfileSelector.tsx
│       ├── ProfileSelector.test.tsx
│       ├── ConfigurationLayout.tsx
│       ├── ConfigurationLayout.test.tsx
│       ├── KeyboardVisualizerContainer.tsx
│       ├── KeyboardVisualizerContainer.test.tsx
│       ├── ConfigurationPanel.tsx
│       ├── ConfigurationPanel.test.tsx
│       ├── CodePanelContainer.tsx
│       └── CodePanelContainer.test.tsx
├── hooks/
│   ├── useProfileSelection.ts
│   ├── useProfileSelection.test.ts
│   ├── useCodePanel.ts
│   ├── useCodePanel.test.ts
│   ├── useKeyboardLayout.ts
│   ├── useKeyboardLayout.test.ts
│   ├── useConfigSync.ts
│   └── useConfigSync.test.ts
└── pages/
    ├── ConfigPage.tsx (refactored)
    └── ConfigPage.test.tsx (updated)
```

## Success Criteria
- ✅ ConfigPage.tsx reduced to <200 lines
- ✅ All functions ≤50 lines
- ✅ 8 new files created (4 components + 4 hooks)
- ✅ All existing tests pass
- ✅ New tests added for all new components/hooks
- ✅ ESLint 0 errors, Prettier applied
- ✅ No performance degradation
- ✅ User-facing behavior unchanged
