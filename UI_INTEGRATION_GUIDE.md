# UI Fixes Integration Guide

## Quick Integration Steps

### 1. Add Toast Provider (REQUIRED)

Edit `src/App.tsx` or `src/main.tsx`:

```typescript
import { ToastProvider } from './components/ToastProvider';

function App() {
  return (
    <QueryClientProvider client={queryClient}>
      <ToastProvider /> {/* Add this */}
      <Router>
        <ErrorBoundary>
          <Routes>
            {/* your routes */}
          </Routes>
        </ErrorBoundary>
      </Router>
    </QueryClientProvider>
  );
}
```

### 2. Update Components to Use Toast Hook

Example migration:

**Before:**
```typescript
try {
  await saveData();
  console.log('Saved');
} catch (error) {
  console.error('Failed:', error);
}
```

**After:**
```typescript
import { useToast } from '@/hooks/useToast';

const { success, error } = useToast();

try {
  await saveData();
  success('Saved successfully');
} catch (err) {
  error(err);  // Automatically extracts error message
}
```

### 3. Add Input Validation

**Before:**
```typescript
const [name, setName] = useState('');

const handleSubmit = () => {
  if (!name) {
    alert('Name required');
    return;
  }
  // ...
};
```

**After:**
```typescript
import { validateProfileName } from '@/utils/validation';
import { useToast } from '@/hooks/useToast';

const [name, setName] = useState('');
const [error, setError] = useState('');
const { error: showError } = useToast();

const handleSubmit = () => {
  const result = validateProfileName(name);
  if (!result.valid) {
    setError(result.error);
    showError(result.error);
    return;
  }
  // ...
};
```

### 4. Add Debouncing to Search Inputs

**Before:**
```typescript
<input onChange={(e) => search(e.target.value)} />
```

**After:**
```typescript
import { useMemo } from 'react';
import { debounce } from '@/utils/debounce';

const debouncedSearch = useMemo(
  () => debounce((query: string) => search(query), 300),
  []
);

<input onChange={(e) => debouncedSearch(e.target.value)} />
```

### 5. Fix Type Assertions

**Before:**
```typescript
const data = response as ApiResponse;
```

**After:**
```typescript
import { safeJsonParse } from '@/utils/typeGuards';

const result = safeJsonParse<ApiResponse>(json, apiResponseSchema);
if (result.success) {
  const data = result.data;
} else {
  console.error('Parse failed:', result.error);
}
```

### 6. Add Cleanup to useEffect

**Before:**
```typescript
useEffect(() => {
  window.addEventListener('resize', handleResize);
}, []);
```

**After:**
```typescript
useEffect(() => {
  window.addEventListener('resize', handleResize);
  return () => window.removeEventListener('resize', handleResize);
}, []);
```

### 7. Fix Race Conditions

**Before:**
```typescript
const [count, setCount] = useState(0);
setCount(count + 1);  // Stale closure
```

**After:**
```typescript
const [count, setCount] = useState(0);
setCount(prev => prev + 1);  // Functional update
```

### 8. Add Accessibility Labels

**Before:**
```typescript
<button onClick={onDelete}>
  <TrashIcon />
</button>
```

**After:**
```typescript
<button onClick={onDelete} aria-label="Delete profile">
  <TrashIcon aria-hidden="true" />
</button>
```

### 9. Add Loading States

**Before:**
```typescript
const { data } = useQuery(...);
return <div>{data?.name}</div>;
```

**After:**
```typescript
const { data, isLoading } = useQuery(...);

if (isLoading) {
  return <LoadingSkeleton />;
}

return <div>{data?.name ?? 'Unknown'}</div>;
```

### 10. Add Error Boundaries

Edit `src/App.tsx`:

```typescript
import { ErrorBoundary } from './components/ErrorBoundary';

<ErrorBoundary
  fallback={(error, reset) => (
    <div>
      <h1>Something went wrong</h1>
      <button onClick={reset}>Try again</button>
    </div>
  )}
>
  <YourComponent />
</ErrorBoundary>
```

## Testing Checklist

- [ ] Run `npm install` to ensure sonner is installed
- [ ] Run `npm test` to verify all tests pass
- [ ] Run `npm run test:a11y` to check accessibility
- [ ] Open app in browser and verify toasts appear
- [ ] Test keyboard navigation (Tab, Enter, Space, Escape)
- [ ] Verify no console errors
- [ ] Check React DevTools for memory leaks
- [ ] Test error scenarios (network failures, invalid input)
- [ ] Verify loading states appear correctly
- [ ] Test debounced inputs (search should wait 300ms)

## Component-by-Component Migration

### High Priority (User-Facing)

1. **ProfilesPage.tsx**
   - [x] Already has loading states
   - [ ] Add useToast for errors
   - [x] Has optimistic updates
   - [x] Has input validation

2. **ConfigPage.tsx**
   - [x] Already has loading states
   - [ ] Add useToast for save errors
   - [ ] Add debouncing to code editor
   - [x] Has null checks

3. **DevicesPage.tsx**
   - [x] Already has loading states
   - [ ] Add useToast for rename/delete
   - [x] Has inline error display
   - [ ] Add aria-labels to toggle switches

4. **DashboardPage.tsx**
   - [x] Fixed memory leaks
   - [x] Fixed stale closures
   - [ ] Add aria-live for status updates

### Medium Priority (Internal)

5. **Modal.tsx**
   - [ ] Add focus trap
   - [ ] Add Escape key handler
   - [x] Has aria-modal

6. **Input.tsx**
   - [ ] Add real-time validation display
   - [ ] Add aria-invalid on errors

7. **Button.tsx**
   - [x] Has aria-labels
   - [ ] Add loading spinner variant

### Low Priority (Already Good)

8. **ErrorBoundary.tsx** - ✅ Already complete
9. **LoadingSkeleton.tsx** - ✅ Already complete
10. **Card.tsx** - ✅ Already complete

## New Utility Usage Examples

### Type Guards
```typescript
import { isDefined, isNonEmptyString, getErrorMessage } from '@/utils/typeGuards';

if (isDefined(data?.field)) {
  // data.field is guaranteed non-null
}

if (isNonEmptyString(input)) {
  // input is a non-empty string
}

const message = getErrorMessage(error, 'Default message');
```

### Validation
```typescript
import { validateProfileName, validateDeviceName, sanitizeInput } from '@/utils/validation';

const result = validateProfileName(name);
if (!result.valid) {
  console.error(result.error);
}

const safe = sanitizeInput(userInput);  // Prevents XSS
```

### Debouncing
```typescript
import { debounce, throttle, debounceAsync } from '@/utils/debounce';

const debouncedFn = debounce(() => console.log('Called'), 300);
const throttledFn = throttle(() => console.log('Called'), 1000);
const debouncedAsync = debounceAsync(async () => fetch('/api'), 500);
```

### Toast
```typescript
import { useToast } from '@/hooks/useToast';

const { success, error, info, warning, promise } = useToast();

success('Operation successful');
error(new Error('Failed'));
info('Please note...');
warning('Be careful');

promise(
  fetchData(),
  {
    loading: 'Loading...',
    success: 'Loaded!',
    error: 'Failed to load'
  }
);
```

## Performance Optimization Checklist

- [ ] Memoize expensive computations with `useMemo`
- [ ] Memoize callbacks with `useCallback`
- [ ] Wrap pure components with `React.memo`
- [ ] Use virtualization for long lists (react-window)
- [ ] Debounce search inputs (300ms)
- [ ] Debounce auto-save (1000ms)
- [ ] Throttle scroll handlers (100ms)
- [ ] Lazy load routes with `React.lazy`
- [ ] Code split large components
- [ ] Optimize images and assets

## Accessibility Checklist

- [ ] All interactive elements have aria-labels
- [ ] All images have alt text
- [ ] All forms have labels
- [ ] All buttons have descriptive text or aria-label
- [ ] Keyboard navigation works (Tab, Enter, Space, Escape)
- [ ] Focus trap in modals
- [ ] Skip to content link
- [ ] Color contrast meets WCAG AA (4.5:1)
- [ ] Heading hierarchy is logical (h1 → h2 → h3)
- [ ] aria-live regions for dynamic content
- [ ] No accessibility violations (run axe-core)

## Error Handling Checklist

- [ ] All promises have `.catch()`
- [ ] All async functions have try/catch
- [ ] Errors display in UI (toasts or inline)
- [ ] Network errors show retry button
- [ ] Validation errors show inline
- [ ] Error boundaries catch render errors
- [ ] Console.error for debugging
- [ ] User-friendly error messages
- [ ] Error state persists until resolved
- [ ] Errors can be dismissed

## Memory Leak Prevention Checklist

- [ ] All useEffect hooks have cleanup functions
- [ ] All event listeners removed on unmount
- [ ] All timers cleared on unmount
- [ ] All intervals cleared on unmount
- [ ] All WebSocket subscriptions unsubscribed
- [ ] All fetch requests cancelled on unmount
- [ ] Refs don't hold DOM references after unmount
- [ ] No circular references in closures

## Next Steps

1. **Week 1:** Add ToastProvider, migrate critical errors
2. **Week 2:** Add debouncing, fix race conditions
3. **Week 3:** Complete accessibility audit
4. **Week 4:** Performance optimization and testing

## Support

- File issues: `WS6-UI-XXX` prefix
- Documentation: `UI_FIXES_SUMMARY.md`
- Test examples: `tests/*.test.tsx`
- Utilities: `src/utils/*.ts`
