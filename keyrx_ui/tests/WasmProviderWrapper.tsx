import React, { ReactNode } from 'react';
import { vi } from 'vitest';

// Mock WASM context value for testing
const mockWasmContextValue = {
  isWasmReady: true,
  isLoading: false,
  error: null,
  validateConfig: vi.fn().mockResolvedValue([]),
  runSimulation: vi.fn().mockResolvedValue(null),
};

// Create a mock context to avoid importing the real one
const WasmContext = React.createContext(mockWasmContextValue);

/**
 * Set the isWasmReady state for testing scenarios where WASM is unavailable
 * This allows testing component behavior when WASM fails to load
 *
 * @example
 * ```typescript
 * import { setMockWasmReady } from '../tests/WasmProviderWrapper';
 *
 * test('handles WASM unavailable', () => {
 *   setMockWasmReady(false);
 *   render(<Component />);
 *   // ... test fallback behavior
 * });
 * ```
 */
export function setMockWasmReady(ready: boolean) {
  mockWasmContextValue.isWasmReady = ready;
  if (!ready) {
    mockWasmContextValue.error = new Error('WASM not available');
  } else {
    mockWasmContextValue.error = null;
  }
}

/**
 * WasmProviderWrapper - Test utility for wrapping components with WASM context
 *
 * This wrapper provides a mock WasmProvider context for testing components
 * that depend on useWasmContext hook. It prevents "useWasmContext must be used
 * within WasmProvider" errors in tests.
 *
 * @example
 * ```typescript
 * import { render } from '@testing-library/react';
 * import { WasmProviderWrapper } from '../tests/WasmProviderWrapper';
 * import { MonacoEditor } from './MonacoEditor';
 *
 * test('renders MonacoEditor with WASM context', () => {
 *   render(
 *     <WasmProviderWrapper>
 *       <MonacoEditor value="" onChange={() => {}} />
 *     </WasmProviderWrapper>
 *   );
 * });
 * ```
 *
 * @param children - React components to wrap with WASM context
 * @returns Wrapped component tree with mock WASM context provider
 */
export function WasmProviderWrapper({ children }: { children: ReactNode }): JSX.Element {
  return <WasmContext.Provider value={mockWasmContextValue}>{children}</WasmContext.Provider>;
}

/**
 * Hook to access mock WASM context in tests
 * Use this instead of the production useWasmContext for testing
 */
export function useWasmContext() {
  const context = React.useContext(WasmContext);
  if (!context) {
    throw new Error('useWasmContext must be used within WasmProvider');
  }
  return context;
}

/**
 * Get the mock WASM context value for test assertions
 * Useful for verifying that WASM functions were called correctly
 *
 * @example
 * ```typescript
 * import { getMockWasmContext } from '../tests/WasmProviderWrapper';
 *
 * test('calls validateConfig on input', async () => {
 *   const mockContext = getMockWasmContext();
 *   // ... render component and trigger validation
 *   expect(mockContext.validateConfig).toHaveBeenCalledWith('some code');
 * });
 * ```
 */
export function getMockWasmContext() {
  return mockWasmContextValue;
}
