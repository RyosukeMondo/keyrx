import React, { ReactNode } from 'react';
import { vi } from 'vitest';

/**
 * Shared mock WASM context value
 * This is used both by vi.mock() in test files and by test assertion helpers
 */
export const mockWasmContextValue = {
  isWasmReady: true,
  isLoading: false,
  error: null as Error | null,
  validateConfig: vi.fn().mockResolvedValue([]),
  runSimulation: vi.fn().mockResolvedValue(null),
};

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
 *   renderWithProviders(<Component />);
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
 * WasmProviderWrapper - Test utility component
 *
 * NOTE: This component is not needed when using vi.mock() to mock the WasmContext.
 * It's kept for backwards compatibility with tests that don't use mocking.
 *
 * @param children - React components to wrap
 * @returns Pass-through of children
 */
export function WasmProviderWrapper({ children }: { children: ReactNode }): JSX.Element {
  return <>{children}</>;
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
 *   renderWithProviders(<MonacoEditor value="test" />);
 *   await waitFor(() => {
 *     expect(mockContext.validateConfig).toHaveBeenCalledWith('test');
 *   });
 * });
 * ```
 */
export function getMockWasmContext() {
  return mockWasmContextValue;
}
