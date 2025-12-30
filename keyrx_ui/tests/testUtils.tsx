/**
 * Shared test utilities for React component tests.
 *
 * Provides common testing helpers to reduce duplication across test files:
 * - renderWithProviders: Wraps components with necessary providers
 * - createMockStore: Creates mock Zustand stores for testing
 * - waitForAsync: Helper for waiting on async operations
 */

import { ReactElement } from 'react';
import { render, RenderOptions } from '@testing-library/react';
import { DndContext } from '@dnd-kit/core';
import type { ConfigState } from '../src/types/configBuilder';

/**
 * Custom render options extending RTL's RenderOptions
 */
export interface CustomRenderOptions extends Omit<RenderOptions, 'wrapper'> {
  /**
   * Whether to wrap the component with DndContext (for drag-and-drop components)
   * @default false
   */
  withDnd?: boolean;

  /**
   * Initial state for the config builder store
   * Note: This doesn't actually inject the store, just provides a pattern for tests
   * Tests should use useConfigBuilderStore.getState() to set initial state
   */
  initialConfigState?: Partial<ConfigState>;
}

/**
 * Renders a React component wrapped with common test providers.
 *
 * Automatically wraps components with:
 * - DndContext (if withDnd is true)
 *
 * @param ui - The React element to render
 * @param options - Render options including provider flags
 * @returns Render result from @testing-library/react
 *
 * @example
 * ```tsx
 * // Basic usage
 * renderWithProviders(<MyComponent />);
 *
 * // With drag-and-drop support
 * renderWithProviders(<VirtualKeyboard />, { withDnd: true });
 *
 * // With custom container
 * renderWithProviders(<MyComponent />, { container: document.body });
 * ```
 */
export function renderWithProviders(
  ui: ReactElement,
  options: CustomRenderOptions = {}
) {
  const { withDnd = false, initialConfigState, ...renderOptions } = options;

  // Build the wrapper component based on options
  function Wrapper({ children }: { children: React.ReactNode }) {
    if (withDnd) {
      return <DndContext>{children}</DndContext>;
    }
    return <>{children}</>;
  }

  // If initial config state is provided, set it before rendering
  // Note: This is a pattern suggestion - tests should call useConfigBuilderStore.getState().setConfig()
  // before calling renderWithProviders if they need to set initial state
  if (initialConfigState) {
    // This is just a type hint for developers - actual state setting should happen in the test
    console.warn(
      'initialConfigState provided but not applied. Use useConfigBuilderStore.getState().setConfig() before rendering.'
    );
  }

  return render(ui, { wrapper: Wrapper, ...renderOptions });
}

/**
 * Creates a mock Zustand store for testing.
 *
 * Returns a partial ConfigState that can be used to initialize the store
 * or verify expected state in tests.
 *
 * @param initialState - Partial state to merge with defaults
 * @returns A complete ConfigState object with defaults applied
 *
 * @example
 * ```tsx
 * // Create a mock store with custom state
 * const mockState = createMockStore({
 *   layers: [
 *     { id: 'base', name: 'base', mappings: [], isBase: true },
 *     { id: 'layer1', name: 'symbols', mappings: [], isBase: false },
 *   ],
 *   currentLayerId: 'layer1',
 * });
 *
 * // Use it to set store state in tests
 * useConfigBuilderStore.getState().setConfig(mockState);
 * ```
 */
export function createMockStore(
  initialState?: Partial<ConfigState>
): ConfigState {
  const defaultState: ConfigState = {
    layers: [
      {
        id: 'base',
        name: 'base',
        mappings: [],
        isBase: true,
      },
    ],
    modifiers: [],
    locks: [],
    currentLayerId: 'base',
    isDirty: false,
  };

  return {
    ...defaultState,
    ...initialState,
  };
}

/**
 * Waits for an async operation with a configurable timeout.
 *
 * Wraps a callback in a Promise and waits for it to complete or timeout.
 * Useful for testing async behavior that isn't covered by waitFor.
 *
 * @param callback - The async operation to wait for
 * @param timeout - Maximum time to wait in milliseconds (default: 1000)
 * @returns Promise that resolves when callback completes or timeout expires
 *
 * @example
 * ```tsx
 * // Wait for a WebSocket connection
 * await waitForAsync(async () => {
 *   expect(mockWebSocket.readyState).toBe(WebSocket.OPEN);
 * }, 2000);
 *
 * // Wait for state update
 * await waitForAsync(async () => {
 *   const state = useConfigBuilderStore.getState();
 *   expect(state.layers.length).toBe(2);
 * });
 * ```
 */
export async function waitForAsync(
  callback: () => void | Promise<void>,
  timeout: number = 1000
): Promise<void> {
  return new Promise<void>((resolve, reject) => {
    const timeoutId = setTimeout(() => {
      reject(new Error(`waitForAsync timed out after ${timeout}ms`));
    }, timeout);

    Promise.resolve(callback())
      .then(() => {
        clearTimeout(timeoutId);
        resolve();
      })
      .catch((error) => {
        clearTimeout(timeoutId);
        reject(error);
      });
  });
}

/**
 * Common mock data for profiles used in tests
 */
export const mockProfiles = [
  {
    name: 'default',
    rhai_path: '/path/to/default.rhai',
    krx_path: '/path/to/default.krx',
    modified_at: 1234567890,
    layer_count: 2,
    is_active: true,
  },
  {
    name: 'gaming',
    rhai_path: '/path/to/gaming.rhai',
    krx_path: '/path/to/gaming.krx',
    modified_at: 1234567891,
    layer_count: 3,
    is_active: false,
  },
  {
    name: 'work',
    rhai_path: '/path/to/work.rhai',
    krx_path: '/path/to/work.krx',
    modified_at: 1234567892,
    layer_count: 1,
    is_active: false,
  },
];

/**
 * Common mock data for keyboard events used in dashboard tests
 */
export const mockKeyboardEvents = [
  {
    timestamp: 1234567890000,
    event_type: 'KeyPress',
    key_code: 65, // A
    description: 'Key A pressed',
  },
  {
    timestamp: 1234567891000,
    event_type: 'KeyRelease',
    key_code: 65,
    description: 'Key A released',
  },
  {
    timestamp: 1234567892000,
    event_type: 'Remapped',
    key_code: 66, // B
    description: 'Key B remapped to C',
  },
];
