# Design: Refactor SimulatorPage Component

## Architecture Overview

### Current State
```
SimulatorPage (712 lines, 671-line function)
├── WebSocket connection management
├── Event collection and filtering
├── Simulation controls (start/stop/clear)
├── Event injection form
├── Event list display
└── Statistics display
```

### Target Architecture
```
SimulatorPage (orchestrator, <200 lines)
├── SimulationControls (start/stop/clear/stats)
├── EventInjectionForm (key selection, event type, inject button)
├── EventList (virtualized list with filtering)
└── useSimulation hook (state management)
```

## Component Breakdown

### 1. SimulatorPage (Orchestrator)
**Responsibility**: Coordinate simulation components

**State**: Minimal - delegates to useSimulation hook

**File**: `src/pages/SimulatorPage.tsx` (refactored)

### 2. SimulationControls Component
**File**: `src/components/simulator/SimulationControls.tsx`

**Props**:
```typescript
interface SimulationControlsProps {
  isRunning: boolean;
  eventCount: number;
  onStart: () => void;
  onStop: () => void;
  onClear: () => void;
  statistics?: SimulationStatistics;
}
```

### 3. EventInjectionForm Component
**File**: `src/components/simulator/EventInjectionForm.tsx`

**Props**:
```typescript
interface EventInjectionFormProps {
  onInjectEvent: (keyCode: string, eventType: 'press' | 'release') => void;
  disabled?: boolean;
}
```

### 4. EventList Component
**File**: `src/components/simulator/EventList.tsx`

**Props**:
```typescript
interface EventListProps {
  events: KeyEvent[];
  maxEvents?: number;
  onClear?: () => void;
  virtualizeThreshold?: number; // Default 100
}
```

**Features**:
- Virtualized rendering with react-window or similar
- Auto-scroll to latest event
- Event highlighting

### 5. useSimulation Hook
**File**: `src/hooks/useSimulation.ts`

**API**:
```typescript
interface UseSimulationOptions {
  maxEvents?: number;
  autoStart?: boolean;
}

function useSimulation(options?: UseSimulationOptions) {
  const [events, setEvents] = useState<KeyEvent[]>([]);
  const [isRunning, setIsRunning] = useState(false);

  const addEvent = useCallback((event: KeyEvent) => { ... }, [maxEvents]);
  const clearEvents = useCallback(() => { ... }, []);
  const start = useCallback(() => { ... }, []);
  const stop = useCallback(() => { ... }, []);

  // WebSocket subscription
  useEffect(() => { ... }, [isRunning]);

  return {
    events,
    isRunning,
    addEvent,
    clearEvents,
    start,
    stop,
    statistics: computeStatistics(events),
  };
}
```

## Data Flow

### Event Collection Flow
```
WebSocket event
  → useSimulation.addEvent
  → events state updated
  → EventList re-renders (virtualized)
```

### Control Flow
```
User clicks Start
  → SimulationControls.onStart
  → useSimulation.start
  → WebSocket subscription activated
  → Events start flowing
```

### Injection Flow
```
User fills form + clicks Inject
  → EventInjectionForm.onInjectEvent
  → SimulatorPage calls injection API
  → Event appears in EventList
```

## Migration Strategy

1. Extract useSimulation hook (non-breaking)
2. Extract EventList component
3. Extract SimulationControls component
4. Extract EventInjectionForm component
5. Update SimulatorPage to use all components
6. Add tests for each component
7. Cleanup and verify

## File Structure
```
src/
├── components/
│   └── simulator/
│       ├── SimulationControls.tsx
│       ├── SimulationControls.test.tsx
│       ├── EventInjectionForm.tsx
│       ├── EventInjectionForm.test.tsx
│       ├── EventList.tsx
│       └── EventList.test.tsx
├── hooks/
│   ├── useSimulation.ts
│   └── useSimulation.test.ts
└── pages/
    ├── SimulatorPage.tsx (refactored)
    └── SimulatorPage.test.tsx (updated)
```

## Success Criteria
- ✅ SimulatorPage.tsx <300 lines
- ✅ 4 new files (3 components + 1 hook)
- ✅ EventList virtualized for performance
- ✅ All tests pass
- ✅ ESLint 0 errors
