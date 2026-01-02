import React, { ReactElement } from 'react';
import { render, RenderOptions, RenderResult } from '@testing-library/react';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { MemoryRouter } from 'react-router-dom';
import { WasmProviderWrapper } from './WasmProviderWrapper';

/**
 * Options for renderWithProviders
 */
export interface TestRenderOptions extends Omit<RenderOptions, 'wrapper'> {
  /**
   * Whether to wrap the component with WasmProvider context
   * @default true - Most components use WASM for validation
   */
  wrapWithWasm?: boolean;

  /**
   * Whether to wrap the component with React Query provider
   * @default true - Most components use React Query for data fetching
   */
  wrapWithReactQuery?: boolean;

  /**
   * Whether to wrap the component with React Router (MemoryRouter)
   * @default false - Only needed for components using routing hooks (useParams, useSearchParams, etc.)
   */
  wrapWithRouter?: boolean;

  /**
   * Initial route entries for MemoryRouter
   * @default ['/'] - Single root route
   */
  routerInitialEntries?: string[];

  /**
   * Custom QueryClient for testing
   * If not provided, a new QueryClient with test-optimized defaults will be created
   */
  queryClient?: QueryClient;
}

/**
 * Create a test-optimized QueryClient
 * Disables retries and reduces delays for faster tests
 */
function createTestQueryClient(): QueryClient {
  return new QueryClient({
    defaultOptions: {
      queries: {
        // Disable retries in tests for faster failures
        retry: false,
        // Disable refetching for deterministic tests
        refetchOnWindowFocus: false,
        refetchOnMount: false,
        refetchOnReconnect: false,
        // Short stale time for tests
        staleTime: 0,
        gcTime: 0,
      },
      mutations: {
        // Disable retries in tests
        retry: false,
      },
    },
    logger: {
      // Suppress error logs in tests
      log: () => {},
      warn: () => {},
      error: () => {},
    },
  });
}

/**
 * Custom render function that wraps components with necessary providers
 *
 * This helper provides a consistent test setup by automatically wrapping
 * components with:
 * - React Router MemoryRouter (for routing context)
 * - React Query QueryClientProvider (for data fetching/caching)
 * - WasmProvider (for WASM-based validation and simulation)
 *
 * Provider nesting order (outer to inner):
 * 1. MemoryRouter (outermost - provides routing context)
 * 2. QueryClientProvider (middle - provides data layer)
 * 3. WasmProvider (innermost - provides WASM context)
 * 4. Component under test
 *
 * @example
 * ```typescript
 * import { renderWithProviders } from '../tests/testUtils';
 * import { MonacoEditor } from './MonacoEditor';
 *
 * test('renders editor with validation', () => {
 *   const { getByRole } = renderWithProviders(
 *     <MonacoEditor value="" onChange={() => {}} />
 *   );
 *   expect(getByRole('textbox')).toBeInTheDocument();
 * });
 * ```
 *
 * @example
 * ```typescript
 * // Enable Router wrapping for components using routing hooks
 * renderWithProviders(<ConfigPage />, { wrapWithRouter: true });
 * ```
 *
 * @example
 * ```typescript
 * // Disable WASM wrapping for components that don't use it
 * renderWithProviders(<SimpleButton />, { wrapWithWasm: false });
 * ```
 *
 * @example
 * ```typescript
 * // Use custom QueryClient for specific test scenarios
 * const customClient = new QueryClient({ ... });
 * renderWithProviders(<DataComponent />, { queryClient: customClient });
 * ```
 *
 * @param ui - React component to render
 * @param options - Rendering options including provider configuration
 * @returns RenderResult from @testing-library/react with additional helpers
 */
export function renderWithProviders(
  ui: ReactElement,
  options: TestRenderOptions = {}
): RenderResult {
  const {
    wrapWithWasm = true,
    wrapWithReactQuery = true,
    wrapWithRouter = false,
    routerInitialEntries = ['/'],
    queryClient,
    ...renderOptions
  } = options;

  // Create QueryClient for this test if not provided
  const testQueryClient = queryClient || createTestQueryClient();

  // Build wrapper component with proper nesting (outer to inner)
  // Router > QueryClient > WASM > Component
  let element = ui;

  // Layer 3: WasmProvider (innermost, closest to component)
  if (wrapWithWasm) {
    element = <WasmProviderWrapper>{element}</WasmProviderWrapper>;
  }

  // Layer 2: QueryClientProvider
  if (wrapWithReactQuery) {
    element = (
      <QueryClientProvider client={testQueryClient}>{element}</QueryClientProvider>
    );
  }

  // Layer 1: MemoryRouter (outermost)
  if (wrapWithRouter) {
    element = (
      <MemoryRouter initialEntries={routerInitialEntries}>{element}</MemoryRouter>
    );
  }

  return render(element, renderOptions);
}

/**
 * Re-export testing utilities for convenience
 */
export * from '@testing-library/react';
export { userEvent } from '@testing-library/user-event';
