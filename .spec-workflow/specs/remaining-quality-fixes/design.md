# Design: Remaining Code Quality Fixes

## Architecture Overview

### Current State
```
KeyConfigModal (641 lines)
├── Modal wrapper
├── Mapping type selection UI (5 types)
├── Key selection tabs
├── Configuration forms for each type
└── Save/cancel logic

KeyConfigPanel (634 lines)
├── Similar to KeyConfigModal but inline
├── Mapping type selection UI (2 types)
├── Key selection tabs
├── Configuration forms
└── Save/clear logic

MetricsPage (532 lines)
├── WebSocket connection management
├── Metrics cards (4 cards)
├── Latency chart with recharts
├── Event log with virtualization
└── State snapshot display
```

### Target Architecture
```
KeyConfigModal (<400 lines)
├── Modal wrapper
├── MappingTypeSelector (shared)
├── KeySelectionTabs (shared)
└── MappingConfigForm (shared)

KeyConfigPanel (<400 lines)
├── Panel wrapper
├── MappingTypeSelector (shared)
├── KeySelectionTabs (shared)
└── MappingConfigForm (shared)

MetricsPage (<400 lines)
├── MetricsStatsCards (4 stat cards)
├── LatencyChart (recharts wrapper)
├── EventLogList (virtualized list)
└── StateSnapshot (current state display)
```

## Shared Components (DRY)

### 1. MappingTypeSelector Component
**File**: `src/components/keyConfig/MappingTypeSelector.tsx`

**Props**:
```typescript
interface MappingTypeSelectorProps {
  selectedType: MappingType;
  onChange: (type: MappingType) => void;
  supportedTypes: MappingType[];
  layout?: 'horizontal' | 'vertical';
}
```

**Purpose**: Reusable mapping type selector for both Modal and Panel

### 2. KeySelectionTabs Component
**File**: `src/components/keyConfig/KeySelectionTabs.tsx`

**Props**:
```typescript
interface KeySelectionTabsProps {
  activeTab: 'keyboard' | 'modifier' | 'lock' | 'layer';
  onTabChange: (tab: string) => void;
  availableTabs: string[];
  onKeySelect: (keyCode: string) => void;
  layoutKeys?: SVGKey[];
}
```

**Purpose**: Tabbed key selection UI with keyboard, modifier, lock, layer tabs

### 3. MappingConfigForm Component
**File**: `src/components/keyConfig/MappingConfigForm.tsx`

**Props**:
```typescript
interface MappingConfigFormProps {
  mappingType: MappingType;
  currentConfig?: Partial<KeyMapping>;
  onChange: (config: Partial<KeyMapping>) => void;
  onValidate: (config: Partial<KeyMapping>) => ValidationResult;
}
```

**Purpose**: Form fields for each mapping type (simple, modifier, tap_hold, etc)

## KeyConfigModal Components

### 4. KeyConfigModal (Refactored)
**Responsibility**: Orchestrate modal, delegate to shared components

**Extraction**: Keep modal wrapper, use shared components for content

**Target**: <400 lines

## KeyConfigPanel Components

### 5. KeyConfigPanel (Refactored)
**Responsibility**: Orchestrate inline panel, delegate to shared components

**Extraction**: Keep panel wrapper, use shared components for content

**Target**: <400 lines

## MetricsPage Components

### 6. MetricsStatsCards Component
**File**: `src/components/metrics/MetricsStatsCards.tsx`

**Props**:
```typescript
interface MetricsStatsCardsProps {
  latencyStats: LatencyStats;
  eventCount: number;
  connected: boolean;
}
```

**Purpose**: Display 4 metric cards (latency, throughput, CPU, memory)

### 7. LatencyChart Component
**File**: `src/components/metrics/LatencyChart.tsx`

**Props**:
```typescript
interface LatencyChartProps {
  data: LatencyDataPoint[];
  maxDataPoints?: number;
}
```

**Purpose**: Recharts line chart for latency over time

### 8. EventLogList Component
**File**: `src/components/metrics/EventLogList.tsx`

**Props**:
```typescript
interface EventLogListProps {
  events: EventLogEntry[];
  maxEvents?: number;
}
```

**Purpose**: Virtualized event log with react-window

### 9. StateSnapshot Component
**File**: `src/components/metrics/StateSnapshot.tsx`

**Props**:
```typescript
interface StateSnapshotProps {
  state: StateSnapshot;
}
```

**Purpose**: Display current state (active layer, modifiers, locks)

### 10. MetricsPage (Refactored)
**Responsibility**: Orchestrate metrics page, delegate to components

**Extraction**: Keep WebSocket subscription, use components for display

**Target**: <400 lines

## Data Flow

### KeyConfig Shared Components Flow
```
User selects mapping type
  → MappingTypeSelector.onChange
  → Parent (Modal/Panel) updates state
  → MappingConfigForm receives new type
  → Form fields update

User selects key
  → KeySelectionTabs.onKeySelect
  → Parent updates config
  → MappingConfigForm receives key
```

### MetricsPage Flow
```
WebSocket event arrives
  → MetricsStore updates
  → MetricsPage receives new data
  → MetricsStatsCards re-renders with new stats
  → LatencyChart adds data point
  → EventLogList adds event
```

## Migration Strategy

### Phase 1: Extract Shared KeyConfig Components
1. Create MappingTypeSelector component
2. Create KeySelectionTabs component
3. Create MappingConfigForm component
4. Add tests for each

### Phase 2: Refactor KeyConfigModal
1. Import shared components
2. Replace inline UI with shared components
3. Simplify orchestration logic
4. Verify tests pass

### Phase 3: Refactor KeyConfigPanel
1. Import shared components
2. Replace inline UI with shared components
3. Simplify orchestration logic
4. Verify tests pass

### Phase 4: Extract MetricsPage Components
1. Create MetricsStatsCards component
2. Create LatencyChart component
3. Create EventLogList component
4. Create StateSnapshot component
5. Add tests for each

### Phase 5: Refactor MetricsPage
1. Import components
2. Replace inline JSX with components
3. Simplify page logic
4. Verify tests pass

### Phase 6: Cleanup
1. Fix ESLint errors
2. Apply Prettier
3. Verify metrics
4. Update documentation

## File Structure
```
src/
├── components/
│   ├── keyConfig/
│   │   ├── MappingTypeSelector.tsx
│   │   ├── MappingTypeSelector.test.tsx
│   │   ├── KeySelectionTabs.tsx
│   │   ├── KeySelectionTabs.test.tsx
│   │   ├── MappingConfigForm.tsx
│   │   └── MappingConfigForm.test.tsx
│   ├── metrics/
│   │   ├── MetricsStatsCards.tsx
│   │   ├── MetricsStatsCards.test.tsx
│   │   ├── LatencyChart.tsx
│   │   ├── LatencyChart.test.tsx
│   │   ├── EventLogList.tsx
│   │   ├── EventLogList.test.tsx
│   │   ├── StateSnapshot.tsx
│   │   └── StateSnapshot.test.tsx
│   ├── KeyConfigModal.tsx (refactored, <400 lines)
│   └── KeyConfigPanel.tsx (refactored, <400 lines)
└── pages/
    └── MetricsPage.tsx (refactored, <400 lines)
```

## Success Criteria
- ✅ 3 shared keyConfig components created
- ✅ 4 metrics components created
- ✅ KeyConfigModal <500 lines
- ✅ KeyConfigPanel <500 lines
- ✅ MetricsPage <500 lines
- ✅ All functions ≤50 lines
- ✅ DRY principle applied
- ✅ All tests pass
- ✅ ESLint 0 errors
