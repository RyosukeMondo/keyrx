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

// =============================================================================
// WebSocket Testing Helpers (MSW-based)
// =============================================================================

/**
 * WebSocket Testing Helpers
 *
 * These utilities use MSW (Mock Service Worker) v2 for robust WebSocket testing.
 * MSW automatically intercepts WebSocket connections and provides type-safe message handling.
 *
 * Basic Usage Pattern:
 * ```typescript
 * import { renderWithProviders, setDaemonState, sendLatencyUpdate, waitForWebSocketConnection } from '../tests/testUtils';
 *
 * test('handles WebSocket messages', async () => {
 *   const { getByText } = renderWithProviders(<MyComponent />);
 *
 *   // Wait for WebSocket connection (optional, MSW connects automatically)
 *   await waitForWebSocketConnection();
 *
 *   // Simulate daemon state update
 *   setDaemonState({ activeProfile: 'gaming', layer: 'fn' });
 *
 *   // Assert component updated
 *   await waitFor(() => {
 *     expect(getByText('gaming')).toBeInTheDocument();
 *   });
 * });
 * ```
 *
 * Available WebSocket Helpers:
 * - `setDaemonState(state)` - Simulate daemon state changes (profile, layer, modifiers)
 * - `sendLatencyUpdate(stats)` - Simulate latency metric broadcasts
 * - `sendKeyEvent(event)` - Simulate key press/release events
 * - `sendServerMessage(channel, data)` - Low-level: send custom messages
 * - `waitForWebSocketConnection()` - Wait for connection (usually not needed)
 * - `simulateDisconnect()` - Simulate WebSocket disconnect (TODO: not yet implemented)
 *
 * See `keyrx_ui/src/test/mocks/websocketHelpers.ts` for detailed documentation and examples.
 */
export {
  setDaemonState,
  sendLatencyUpdate,
  sendKeyEvent,
  sendServerMessage,
  waitForWebSocketConnection,
  simulateDisconnect,
} from '../src/test/mocks/websocketHelpers';

/**
 * Re-export testing utilities for convenience
 */
export * from '@testing-library/react';
export { userEvent } from '@testing-library/user-event';

/**
 * Re-export test factories for easy access
 */
export * from './factories';

// =============================================================================
// Enhanced Rendering Helpers
// =============================================================================

/**
 * Render a page component with all necessary providers.
 * Shorthand for renderWithProviders with router enabled.
 *
 * @example
 *   renderPage(<ProfilesPage />);
 *   renderPage(<DevicesPage />, { routerInitialEntries: ['/devices'] });
 */
export function renderPage(
  ui: ReactElement,
  options: Omit<TestRenderOptions, 'wrapWithRouter'> = {}
): RenderResult {
  return renderWithProviders(ui, { ...options, wrapWithRouter: true });
}

/**
 * Render a component without any providers (pure rendering).
 * Useful for testing presentational components in isolation.
 *
 * @example
 *   renderPure(<Button onClick={() => {}}>Click me</Button>);
 */
export function renderPure(
  ui: ReactElement,
  options: Omit<TestRenderOptions, 'wrapWithWasm' | 'wrapWithReactQuery' | 'wrapWithRouter'> = {}
): RenderResult {
  return renderWithProviders(ui, {
    ...options,
    wrapWithWasm: false,
    wrapWithReactQuery: false,
    wrapWithRouter: false,
  });
}

// =============================================================================
// User Interaction Helpers
// =============================================================================

/**
 * Setup userEvent for tests.
 * Returns a configured userEvent instance with sensible defaults.
 *
 * @example
 *   const user = setupUser();
 *   await user.click(button);
 *   await user.type(input, 'hello');
 */
export function setupUser() {
  return userEvent.setup({
    // Add slight delay to simulate real user typing
    delay: null, // null = no delay (faster tests), or set to number for realistic typing
  });
}

/**
 * Type into an input field and wait for updates.
 * Combines userEvent.type with common assertions.
 *
 * @example
 *   await typeIntoField(screen.getByLabelText('Name'), 'John Doe');
 */
export async function typeIntoField(
  element: HTMLElement,
  text: string,
  options?: Parameters<typeof userEvent.type>[2]
): Promise<void> {
  const user = setupUser();
  await user.clear(element);
  await user.type(element, text, options);
}

/**
 * Click an element and wait for updates.
 *
 * @example
 *   await clickElement(screen.getByRole('button', { name: 'Save' }));
 */
export async function clickElement(
  element: HTMLElement,
  options?: Parameters<typeof userEvent.click>[1]
): Promise<void> {
  const user = setupUser();
  await user.click(element, options);
}

/**
 * Select an option from a select element.
 *
 * @example
 *   await selectOption(screen.getByLabelText('Layout'), 'ANSI_104');
 */
export async function selectOption(
  element: HTMLElement,
  value: string | string[]
): Promise<void> {
  const user = setupUser();
  await user.selectOptions(element, value);
}

// =============================================================================
// Async Testing Utilities
// =============================================================================

/**
 * Wait for an element to appear and return it.
 * Shorthand for findBy* queries with better error messages.
 *
 * @example
 *   const button = await waitForElement(() => screen.getByRole('button'));
 */
export async function waitForElement<T>(
  callback: () => T,
  options?: { timeout?: number; interval?: number }
): Promise<T> {
  const { waitFor } = await import('@testing-library/react');
  return waitFor(callback, options);
}

/**
 * Wait for loading states to complete.
 * Useful after triggering data fetches or async operations.
 *
 * @example
 *   await waitForLoadingToFinish();
 *   expect(screen.queryByText('Loading...')).not.toBeInTheDocument();
 */
export async function waitForLoadingToFinish(
  loadingText = 'Loading...',
  timeout = 3000
): Promise<void> {
  const { waitFor, screen } = await import('@testing-library/react');
  await waitFor(() => expect(screen.queryByText(loadingText)).not.toBeInTheDocument(), {
    timeout,
  });
}

/**
 * Wait for an API call to complete and data to render.
 * Combines waiting for loading states and data presence.
 *
 * @example
 *   await waitForData(() => screen.getByText('Profile loaded'));
 */
export async function waitForData<T>(
  callback: () => T,
  timeout = 5000
): Promise<T> {
  return waitForElement(callback, { timeout });
}

// =============================================================================
// Assertion Helpers
// =============================================================================

/**
 * Assert that an element has accessible name (ARIA label).
 * Useful for accessibility testing.
 *
 * @example
 *   assertAccessibleName(screen.getByRole('button'), 'Save profile');
 */
export function assertAccessibleName(element: HTMLElement, expectedName: string): void {
  const { screen } = require('@testing-library/react');
  const accessibleName = element.getAttribute('aria-label') || element.textContent;
  expect(accessibleName).toBe(expectedName);
}

/**
 * Assert that a form field has a validation error.
 *
 * @example
 *   await assertFieldError(screen.getByLabelText('Email'), 'Invalid email');
 */
export function assertFieldError(field: HTMLElement, errorMessage: string): void {
  // Check for aria-invalid
  expect(field).toHaveAttribute('aria-invalid', 'true');

  // Check for error message via aria-describedby
  const errorId = field.getAttribute('aria-describedby');
  if (errorId) {
    const errorElement = document.getElementById(errorId);
    expect(errorElement).toHaveTextContent(errorMessage);
  }
}

/**
 * Assert that an element is visible and accessible.
 * Checks both DOM presence and accessibility tree.
 *
 * @example
 *   assertVisibleAndAccessible(screen.getByRole('button', { name: 'Submit' }));
 */
export function assertVisibleAndAccessible(element: HTMLElement): void {
  expect(element).toBeVisible();
  expect(element).toBeInTheDocument();
  // Element should be in accessibility tree (not aria-hidden)
  expect(element).not.toHaveAttribute('aria-hidden', 'true');
}

// =============================================================================
// Debug Utilities
// =============================================================================

/**
 * Log the current accessibility tree.
 * Useful for debugging accessibility issues.
 *
 * @example
 *   logAccessibilityTree();
 */
export function logAccessibilityTree(container?: HTMLElement): void {
  const { screen, logRoles } = require('@testing-library/react');
  if (container) {
    logRoles(container);
  } else {
    logRoles(document.body);
  }
}

/**
 * Log the current DOM tree (pretty printed).
 *
 * @example
 *   logDOMTree();
 */
export function logDOMTree(container?: HTMLElement, maxLength?: number): void {
  const { screen } = require('@testing-library/react');
  if (container) {
    screen.debug(container, maxLength);
  } else {
    screen.debug(undefined, maxLength);
  }
}
