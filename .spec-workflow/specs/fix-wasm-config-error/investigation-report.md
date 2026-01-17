# WASM ConfigPage Error - Investigation Report

## Date: 2026-01-18

## Summary
Investigated React Error #185 occurring in ConfigPage. Root cause identified as infinite re-render loop caused by improper useEffect dependency arrays and state management patterns.

## What is React Error #185?

React Error #185 is: **"Maximum update depth exceeded"**

This error occurs when a component repeatedly calls setState inside useEffect or lifecycle methods, creating an infinite loop. React stops the loop after detecting excessive re-renders to prevent a crash.

### Sources:
- [Minified React error #185 – React](https://react.dev/errors/185)
- [Troubleshooting Minified React Error 185: A Step-by-Step Guide](https://www.dhiwise.com/post/how-to-resolve-minified-react-error-185-a-step-by-step-guide)
- [Minified React Error #185 Explained: Stop Infinite Rerenders](https://enstacked.com/minified-react-error-185/)

## Root Cause Analysis

### 1. Primary Issue: Circular Dependency in useEffect (ConfigPage.tsx:154-165)

**Location:** `keyrx_ui/src/pages/ConfigPage.tsx:154-165`

**Problem Code:**
```typescript
useEffect(() => {
  // Mark as unsaved when code changes (except during save)
  if (syncStatus === 'saved' && syncEngine.state === 'idle') {
    const currentCode = syncEngine.getCode();
    const originalCode = profileConfig?.source || '';
    if (currentCode !== originalCode) {
      setSyncStatus('unsaved');  // ⚠️ Updates state
    }
  }
  // eslint-disable-next-line react-hooks/exhaustive-deps
}, [syncEngine.state, profileConfig?.source, syncStatus]);  // ⚠️ syncStatus in deps
```

**Why This Causes Infinite Loop:**
1. `syncStatus` is both read (line 157) and written (line 161) in the effect
2. `syncStatus` is in the dependency array (line 165)
3. When the condition is true, it updates `syncStatus` from 'saved' to 'unsaved'
4. This state change triggers the effect to run again
5. The condition fails now (since status is 'unsaved'), but this pattern can still cause excessive re-renders when status changes back to 'saved'

**Additional Problem:**
The ESLint warning `react-hooks/exhaustive-deps` is disabled, hiding the fact that `setSyncStatus` should be in the dependency array (but adding it would make the loop worse).

### 2. Function Call in useEffect Body

**Location:** `keyrx_ui/src/pages/ConfigPage.tsx:158`

**Problem:**
```typescript
const currentCode = syncEngine.getCode();
```

While `getCode()` is implemented as a stable callback (`useCallback(() => codeRef.current, [])` in RhaiSyncEngine.tsx:382), calling functions inside useEffect that depend on external state can lead to issues. The code is reading from a ref, which is good, but the pattern is still risky.

### 3. WasmProvider Integration - ✅ CORRECT

**Location:** `keyrx_ui/src/App.tsx:20-46`

**Status:** The WasmProvider is correctly positioned in the component tree:
```typescript
<ErrorBoundary>
  <WasmProvider>
    <LayoutPreviewProvider>
      <BrowserRouter>
        <Layout>
          <Routes>
            <Route path="/config" element={<ConfigPage />} />
```

- ✅ WasmProvider wraps all routes including ConfigPage
- ✅ WasmProvider is before LayoutPreviewProvider (correct hierarchy)
- ✅ MonacoEditor can access WasmContext via useWasmContext hook
- ✅ CodePanelContainer contains MonacoEditor which uses WASM

### 4. useWasmContext Hook - ✅ CORRECT

**Location:** `keyrx_ui/src/contexts/WasmContext.tsx:180-186`

**Status:** Properly throws error if used outside provider:
```typescript
export function useWasmContext() {
  const context = useContext(WasmContext);
  if (!context) {
    throw new Error('useWasmContext must be used within WasmProvider');
  }
  return context;
}
```

This is the correct pattern. If ConfigPage had a WASM context issue, we'd see this error instead of Error #185.

## Secondary Issues Found

### 1. Another Problematic useEffect (ConfigPage.tsx:130-152)

**Location:** `keyrx_ui/src/pages/ConfigPage.tsx:130-152`

**Potential Issue:**
```typescript
useEffect(() => {
  if (profileConfig?.source) {
    syncEngine.onCodeChange(profileConfig.source);  // Could trigger state changes
    setSyncStatus('saved');
  } else if (configMissing) {
    const defaultTemplate = `...`;
    syncEngine.onCodeChange(defaultTemplate);  // Could trigger state changes
    setSyncStatus('unsaved');
  }
  // eslint-disable-next-line react-hooks/exhaustive-deps
}, [profileConfig, configMissing, selectedProfileName]);
```

**Concern:**
- Calls `syncEngine.onCodeChange()` which updates internal refs and triggers parsing
- This could cause `syncEngine.state` to change from 'idle' to 'parsing'
- Which would trigger the other useEffect (lines 154-165)
- Creating a cascade of effects

### 2. Disabled ESLint Warnings

Multiple `eslint-disable react-hooks/exhaustive-deps` comments throughout the file indicate dependency array issues were "solved" by disabling the warning rather than fixing the root cause.

## Verification: WASM Health Check

Ran comprehensive check of WASM integration:

- ✅ WasmProvider exists in App.tsx
- ✅ WasmProvider wraps ConfigPage route
- ✅ MonacoEditor uses useWasmContext hook
- ✅ useWasmContext throws error if used outside provider
- ✅ WASM module loading is handled correctly with proper error boundaries

**Conclusion:** The issue is NOT related to WASM context availability. It's purely a React hooks infinite loop issue.

## Recommended Fixes

### Fix #1: Remove syncStatus from Dependency Array

**Current:**
```typescript
useEffect(() => {
  if (syncStatus === 'saved' && syncEngine.state === 'idle') {
    const currentCode = syncEngine.getCode();
    const originalCode = profileConfig?.source || '';
    if (currentCode !== originalCode) {
      setSyncStatus('unsaved');
    }
  }
}, [syncEngine.state, profileConfig?.source, syncStatus]);  // ❌ syncStatus causes loop
```

**Fixed:**
```typescript
useEffect(() => {
  // Mark as unsaved when code changes (except during save)
  if (syncStatus === 'saved' && syncEngine.state === 'idle') {
    const currentCode = syncEngine.getCode();
    const originalCode = profileConfig?.source || '';
    if (currentCode !== originalCode) {
      setSyncStatus('unsaved');
    }
  }
}, [syncEngine.state, profileConfig?.source]);  // ✅ Removed syncStatus
```

**Why This Works:**
- Still reads `syncStatus` to check the condition
- But doesn't re-run when `syncStatus` changes
- Only re-runs when the actual dependencies (`syncEngine.state`, `profileConfig?.source`) change

### Fix #2: Add Proper Guard for Profile Changes

**Current:**
```typescript
useEffect(() => {
  if (profileConfig?.source) {
    syncEngine.onCodeChange(profileConfig.source);
    setSyncStatus('saved');
  } else if (configMissing) {
    const defaultTemplate = `...`;
    syncEngine.onCodeChange(defaultTemplate);
    setSyncStatus('unsaved');
  }
}, [profileConfig, configMissing, selectedProfileName]);
```

**Fixed:**
```typescript
const lastProfileRef = useRef<string>(selectedProfileName);

useEffect(() => {
  // Only update if profile changed or config loaded for first time
  const profileChanged = lastProfileRef.current !== selectedProfileName;

  if (profileChanged) {
    lastProfileRef.current = selectedProfileName;

    if (profileConfig?.source) {
      syncEngine.onCodeChange(profileConfig.source);
      setSyncStatus('saved');
    } else if (configMissing) {
      const defaultTemplate = `...`;
      syncEngine.onCodeChange(defaultTemplate);
      setSyncStatus('unsaved');
    }
  }
}, [profileConfig, configMissing, selectedProfileName, syncEngine, setSyncStatus]);
```

### Fix #3: Enable ESLint Warnings

Remove all `// eslint-disable-next-line react-hooks/exhaustive-deps` comments and properly fix the dependency arrays instead of hiding the warnings.

## Testing Plan

### 1. Development Server Testing
```bash
cd keyrx_ui
npm run dev
```
- Navigate to ConfigPage
- Check browser console for errors
- Verify no infinite re-renders in React DevTools Profiler

### 2. Production Build Testing
```bash
npm run build
cd ..
cargo build --release
./target/release/keyrx_daemon
```
- Navigate to http://localhost:9871
- Test ConfigPage loads without Error #185
- Verify WASM loads correctly

### 3. Manual Testing Checklist
- [ ] ConfigPage loads without errors
- [ ] Switching profiles doesn't cause re-render loops
- [ ] Code editor changes update sync status correctly
- [ ] Visual editor changes update code correctly
- [ ] Save button works properly
- [ ] Browser cache doesn't show stale UI

## Next Steps

1. Apply fixes to ConfigPage.tsx
2. Remove ESLint suppressions
3. Run development server and verify no errors
4. Add tests for provider hierarchy (Task 8)
5. Add pre-commit WASM verification (Task 9)
6. Create troubleshooting documentation (Task 10)

## Files Analyzed

- ✅ `keyrx_ui/src/pages/ConfigPage.tsx` - Contains the bugs
- ✅ `keyrx_ui/src/App.tsx` - WasmProvider correctly positioned
- ✅ `keyrx_ui/src/contexts/WasmContext.tsx` - Proper context implementation
- ✅ `keyrx_ui/src/components/MonacoEditor.tsx` - Uses useWasmContext correctly
- ✅ `keyrx_ui/src/hooks/useConfigSync.ts` - Creates stable syncEngine
- ✅ `keyrx_ui/src/components/RhaiSyncEngine.tsx` - getCode() is stable

## Conclusion

**Root Cause:** Infinite re-render loop in ConfigPage.tsx caused by:
1. `syncStatus` in useEffect dependency array while also being updated in the effect body
2. Cascading effects between multiple useEffect hooks
3. Disabled ESLint warnings hiding the problems

**NOT Caused By:**
- WASM context issues (WasmProvider is correctly positioned)
- Missing providers in component tree
- WASM loading failures

**Fix Complexity:** Low - Remove `syncStatus` from dependency array and add proper guards for profile changes.
